// SPDX-License-Identifier: Apache-2.0

#![no_std]

//! AidFlow Escrow Contract
//!
//! Milestone-gated program funding, attestation, and release.
//!
//! Trust model (two-key): the **oracle** attests that a milestone's evidence is
//! valid; the **admin** (multisig) then releases that milestone's funds. Both
//! identities live in the Config contract and are queried at call time, so
//! rotating them there rotates them here.
//!
//! Locked design decisions (see project memory, 2026-08-06):
//!   1. Custody — `release` moves the milestone amount to the VoucherRegistry
//!      address; voucher issuance is the VoucherRegistry contract's job.
//!   2. Refund — admin-only, unspent-only, gated to a Completed program.
//!      Funder self-reclaim on stall is deferred to post-audit.
//!   3. Pause — the Config circuit breaker blocks ALL state-changing calls,
//!      including `attest_milestone`.
//!   4. Token — a single settlement token (USDC) is fixed at init; every
//!      program must use it. Multi-token is deferred to v2.
//!
//! Invariants enforced regardless of the above:
//!   * Each milestone releases at most once (no double-release).
//!   * Checks-Effects-Interactions ordering around every token transfer.
//!   * Checked arithmetic on all balances.
//!   * A milestone cannot be released without a prior attestation.
//!
//! @stable

use aidflow_contract_types::{Milestone, ProgramStatus};
use soroban_sdk::{
    contract, contractevent, contractimpl, contracttype, token, Address, BytesN, Env, Symbol, Vec,
};

/// Instance-storage TTL bounds (ledgers, ~5s each): bump at ~23 days, extend to ~30.
const TTL_THRESHOLD: u32 = 397_440;
const TTL_EXTEND_TO: u32 = 518_400;

/// Persistent-storage TTL bounds for program/attestation entries (~30/37 days).
const P_TTL_THRESHOLD: u32 = 518_400;
const P_TTL_EXTEND_TO: u32 = 639_360;

#[contracttype]
#[derive(Clone)]
pub enum DataKey {
    /// Config/AccessControl contract address (instance)
    Config,
    /// Settlement token (USDC) address (instance)
    Token,
    /// VoucherRegistry address that released funds flow to (instance)
    VoucherRegistry,
    /// Global program counter (instance)
    ProgramCounter,
    /// Program record by id (persistent)
    Program(u64),
    /// Attestation record by (program_id, milestone_id) (persistent)
    Attestation(u64, u32),
    /// Whether a milestone has been released, by (program_id, milestone_id) (persistent)
    Released(u64, u32),
}

/// A program: the funder's milestone-gated escrow.
///
/// @stable
#[contracttype]
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct Program {
    pub id: u64,
    pub funder: Address,
    pub token: Address,
    pub milestones: Vec<Milestone>,
    pub status: ProgramStatus,
    pub funded_amount: i128,
    pub released_amount: i128,
}

/// Evidence that an oracle attested a milestone.
///
/// @stable
#[contracttype]
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct AttestationRecord {
    pub evidence_hash: BytesN<32>,
    pub attested_at: u64,
    pub oracle: Address,
}

/// Event emitted when a milestone is attested
#[contractevent]
pub struct MilestoneAttested {
    pub program_id: u64,
    pub milestone_id: u32,
    pub evidence_hash: BytesN<32>,
}

/// Event emitted when a milestone's funds are released
#[contractevent]
pub struct Released {
    pub program_id: u64,
    pub milestone_id: u32,
    pub amount: i128,
}

/// Event emitted when a new program is created
#[contractevent]
pub struct ProgramCreated {
    pub program_id: u64,
    pub funder: Address,
}

/// Event emitted when a program is funded
#[contractevent]
pub struct Funded {
    pub program_id: u64,
    pub amount: i128,
}

/// Event emitted when unspent funds are refunded
#[contractevent]
pub struct Refunded {
    pub program_id: u64,
    pub amount: i128,
}

/// Event emitted when expired funds are reclaimed
#[contractevent]
pub struct Reclaimed {
    pub program_id: u64,
    pub amount: i128,
}

#[contract]
pub struct EscrowContract;

#[contractimpl]
impl EscrowContract {
    /// Initialize with the Config contract, the settlement token (USDC), and
    /// the VoucherRegistry address that released funds flow to.
    ///
    /// Requires the Config admin's authorization, binding init to the intended
    /// admin so it cannot be front-run if deploy and init are separate txs.
    /// Can only be called once.
    ///
    /// @stable
    pub fn init(env: Env, config_contract: Address, token: Address, voucher_registry: Address) {
        if env.storage().instance().has(&DataKey::Config) {
            panic!("already initialized");
        }

        // Bind initialization to the admin defined in Config.
        let admin = cfg_admin(&env, &config_contract);
        admin.require_auth();

        env.storage()
            .instance()
            .set(&DataKey::Config, &config_contract);
        env.storage().instance().set(&DataKey::Token, &token);
        env.storage()
            .instance()
            .set(&DataKey::VoucherRegistry, &voucher_registry);
        env.storage()
            .instance()
            .set(&DataKey::ProgramCounter, &0u64);
        bump_instance_ttl(&env);
    }

    /// Create a new aid program with milestones. The funder authorizes creation.
    /// The program token must equal the configured settlement token.
    ///
    /// Returns the new program_id.
    ///
    /// @stable
    pub fn create_program(
        env: Env,
        funder: Address,
        token: Address,
        milestones: Vec<Milestone>,
    ) -> u64 {
        funder.require_auth();
        ensure_not_paused(&env);

        // Decision #4: single settlement token.
        let configured: Address = env.storage().instance().get(&DataKey::Token).unwrap();
        if token != configured {
            panic!("token must be the configured settlement token");
        }

        if milestones.is_empty() {
            panic!("program needs at least one milestone");
        }
        // Validate: positive amounts and unique milestone ids.
        let len = milestones.len();
        for i in 0..len {
            let mi = milestones.get(i).unwrap();
            if mi.target_amount <= 0 {
                panic!("milestone target_amount must be positive");
            }
            for j in (i + 1)..len {
                if milestones.get(j).unwrap().id == mi.id {
                    panic!("duplicate milestone id");
                }
            }
        }

        let mut counter: u64 = env
            .storage()
            .instance()
            .get(&DataKey::ProgramCounter)
            .unwrap();
        counter = counter.checked_add(1).expect("counter overflow");
        env.storage()
            .instance()
            .set(&DataKey::ProgramCounter, &counter);
        bump_instance_ttl(&env);

        let program = Program {
            id: counter,
            funder: funder.clone(),
            token,
            milestones,
            status: ProgramStatus::Active,
            funded_amount: 0,
            released_amount: 0,
        };
        save_program(&env, &program);

        ProgramCreated {
            program_id: counter,
            funder,
        }
        .publish(&env);
        counter
    }

    /// Fund a program: transfers `amount` of the settlement token from the
    /// funder into the contract. Only the program's funder may fund it.
    ///
    /// @stable
    pub fn fund(env: Env, program_id: u64, amount: i128) {
        ensure_not_paused(&env);
        if amount <= 0 {
            panic!("amount must be positive");
        }

        let mut program = load_program(&env, program_id);
        if program.status != ProgramStatus::Active {
            panic!("program is not active");
        }
        program.funder.require_auth();

        // EFFECTS before INTERACTIONS.
        program.funded_amount = program
            .funded_amount
            .checked_add(amount)
            .expect("funded_amount overflow");
        save_program(&env, &program);

        // INTERACTIONS: pull tokens from the funder into this contract.
        let token: Address = env.storage().instance().get(&DataKey::Token).unwrap();
        let here = env.current_contract_address();
        token::TokenClient::new(&env, &token).transfer(&program.funder, &here, &amount);

        Funded { program_id, amount }.publish(&env);
    }

    /// Attest a milestone with an evidence hash. Requires oracle auth.
    /// Blocked when paused (decision #3). Overwrites are allowed until release.
    ///
    /// @stable
    pub fn attest_milestone(
        env: Env,
        program_id: u64,
        milestone_id: u32,
        evidence_hash: BytesN<32>,
    ) {
        let oracle = require_oracle(&env);
        ensure_not_paused(&env);

        let program = load_program(&env, program_id);
        if program.status != ProgramStatus::Active {
            panic!("program is not active");
        }
        if !milestone_exists(&program, milestone_id) {
            panic!("unknown milestone id");
        }
        if is_released(&env, program_id, milestone_id) {
            panic!("milestone already released");
        }

        let record = AttestationRecord {
            evidence_hash: evidence_hash.clone(),
            attested_at: env.ledger().timestamp(),
            oracle,
        };
        let key = DataKey::Attestation(program_id, milestone_id);
        env.storage().persistent().set(&key, &record);
        bump_persistent_ttl(&env, &key);

        MilestoneAttested {
            program_id,
            milestone_id,
            evidence_hash,
        }
        .publish(&env);
    }

    /// Release a milestone's funds to the VoucherRegistry. Requires admin auth
    /// and a prior attestation. Each milestone releases at most once.
    ///
    /// @stable
    pub fn release(env: Env, program_id: u64, milestone_id: u32) {
        require_admin(&env);
        ensure_not_paused(&env);

        let mut program = load_program(&env, program_id);
        if program.status != ProgramStatus::Active {
            panic!("program is not active");
        }

        // Must be attested and not yet released.
        if !env
            .storage()
            .persistent()
            .has(&DataKey::Attestation(program_id, milestone_id))
        {
            panic!("milestone not attested");
        }
        if is_released(&env, program_id, milestone_id) {
            panic!("milestone already released");
        }

        let amount = milestone_amount(&program, milestone_id).expect("unknown milestone id");

        // Funds available must cover this milestone.
        let available = program
            .funded_amount
            .checked_sub(program.released_amount)
            .expect("accounting underflow");
        if available < amount {
            panic!("insufficient funded balance for milestone");
        }

        // EFFECTS: mark released, bump released_amount, complete if all released.
        let rkey = DataKey::Released(program_id, milestone_id);
        env.storage().persistent().set(&rkey, &true);
        bump_persistent_ttl(&env, &rkey);
        program.released_amount = program
            .released_amount
            .checked_add(amount)
            .expect("released_amount overflow");
        if all_released(&env, &program) {
            program.status = ProgramStatus::Completed;
        }
        save_program(&env, &program);

        // INTERACTIONS: move the milestone amount to the VoucherRegistry.
        let token: Address = env.storage().instance().get(&DataKey::Token).unwrap();
        let registry: Address = env
            .storage()
            .instance()
            .get(&DataKey::VoucherRegistry)
            .unwrap();
        let here = env.current_contract_address();
        token::TokenClient::new(&env, &token).transfer(&here, &registry, &amount);

        Released {
            program_id,
            milestone_id,
            amount,
        }
        .publish(&env);
    }

    /// Refund unspent funds (funded - released) to the funder. Admin-only,
    /// callable only once a program is Completed. Sets status to Refunded to
    /// prevent double-refund. (Stall/self-reclaim path deferred to post-audit.)
    ///
    /// @stable
    pub fn refund_unspent(env: Env, program_id: u64) {
        let _admin = require_admin(&env);
        ensure_not_paused(&env);

        let mut program = load_program(&env, program_id);
        if program.status != ProgramStatus::Completed {
            panic!("program must be completed to refund");
        }
        let unspent = program
            .funded_amount
            .checked_sub(program.released_amount)
            .expect("accounting underflow");
        if unspent <= 0 {
            panic!("nothing to refund");
        }

        // EFFECTS: close the program before transferring.
        program.status = ProgramStatus::Refunded;
        save_program(&env, &program);

        // INTERACTIONS.
        let token: Address = env.storage().instance().get(&DataKey::Token).unwrap();
        let here = env.current_contract_address();
        token::TokenClient::new(&env, &token).transfer(&here, &program.funder, &unspent);

        Refunded {
            program_id,
            amount: unspent,
        }
        .publish(&env);
    }

    /// Accept funds returned by the VoucherRegistry when a voucher expires.
    /// Callable only by the registry, which transfers `amount` to this contract
    /// immediately before calling.
    ///
    /// For an Active/Completed program the amount is credited back by lowering
    /// `released_amount` (so `refund_unspent` returns it to the funder). For an
    /// already-Refunded program it is forwarded straight to the funder, so it
    /// can never be stranded here.
    ///
    /// @stable
    pub fn reclaim_expired(env: Env, program_id: u64, amount: i128) {
        let registry: Address = env
            .storage()
            .instance()
            .get(&DataKey::VoucherRegistry)
            .unwrap();
        registry.require_auth();
        ensure_not_paused(&env);
        if amount <= 0 {
            panic!("amount must be positive");
        }

        let mut program = load_program(&env, program_id);
        if program.status == ProgramStatus::Refunded {
            let token: Address = env.storage().instance().get(&DataKey::Token).unwrap();
            let here = env.current_contract_address();
            token::TokenClient::new(&env, &token).transfer(&here, &program.funder, &amount);
        } else {
            program.released_amount = program
                .released_amount
                .checked_sub(amount)
                .filter(|v| *v >= 0)
                .expect("reclaim exceeds released amount");
            save_program(&env, &program);
        }

        Reclaimed { program_id, amount }.publish(&env);
    }

    /// Get a program's details.
    ///
    /// @stable
    pub fn get_program(env: Env, program_id: u64) -> Program {
        load_program(&env, program_id)
    }

    /// Get a milestone's attestation record, if any.
    ///
    /// @stable
    pub fn get_attestation(
        env: Env,
        program_id: u64,
        milestone_id: u32,
    ) -> Option<AttestationRecord> {
        env.storage()
            .persistent()
            .get(&DataKey::Attestation(program_id, milestone_id))
    }
}

// ---- internal helpers ----

fn bump_instance_ttl(env: &Env) {
    env.storage()
        .instance()
        .extend_ttl(TTL_THRESHOLD, TTL_EXTEND_TO);
}

fn bump_persistent_ttl(env: &Env, key: &DataKey) {
    env.storage()
        .persistent()
        .extend_ttl(key, P_TTL_THRESHOLD, P_TTL_EXTEND_TO);
}

fn config_addr(env: &Env) -> Address {
    env.storage().instance().get(&DataKey::Config).unwrap()
}

fn save_program(env: &Env, program: &Program) {
    let key = DataKey::Program(program.id);
    env.storage().persistent().set(&key, program);
    bump_persistent_ttl(env, &key);
}

fn load_program(env: &Env, program_id: u64) -> Program {
    env.storage()
        .persistent()
        .get(&DataKey::Program(program_id))
        .expect("program not found")
}

fn is_released(env: &Env, program_id: u64, milestone_id: u32) -> bool {
    env.storage()
        .persistent()
        .get(&DataKey::Released(program_id, milestone_id))
        .unwrap_or(false)
}

fn milestone_exists(program: &Program, milestone_id: u32) -> bool {
    milestone_amount(program, milestone_id).is_some()
}

fn milestone_amount(program: &Program, milestone_id: u32) -> Option<i128> {
    let len = program.milestones.len();
    for i in 0..len {
        let m = program.milestones.get(i).unwrap();
        if m.id == milestone_id {
            return Some(m.target_amount);
        }
    }
    None
}

/// True once every milestone has been released. Completion is keyed on the
/// per-milestone flags (not on `released_amount`) because `reclaim_expired`
/// can lower `released_amount` after a release.
fn all_released(env: &Env, program: &Program) -> bool {
    let len = program.milestones.len();
    for i in 0..len {
        if !is_released(env, program.id, program.milestones.get(i).unwrap().id) {
            return false;
        }
    }
    true
}

// ---- Config (access control + circuit breaker) queries ----

fn cfg_admin(env: &Env, config: &Address) -> Address {
    env.invoke_contract(config, &Symbol::new(env, "get_admin"), Vec::new(env))
}

fn require_admin(env: &Env) -> Address {
    let admin: Address = env.invoke_contract(
        &config_addr(env),
        &Symbol::new(env, "get_admin"),
        Vec::new(env),
    );
    admin.require_auth();
    admin
}

fn require_oracle(env: &Env) -> Address {
    let oracle: Address = env.invoke_contract(
        &config_addr(env),
        &Symbol::new(env, "get_oracle"),
        Vec::new(env),
    );
    oracle.require_auth();
    oracle
}

fn ensure_not_paused(env: &Env) {
    let paused: bool = env.invoke_contract(
        &config_addr(env),
        &Symbol::new(env, "is_paused"),
        Vec::new(env),
    );
    if paused {
        panic!("contract is paused");
    }
}

#[cfg(test)]
mod test {
    use super::*;
    use soroban_sdk::testutils::{Address as _, MockAuth, MockAuthInvoke};
    use soroban_sdk::{vec, IntoVal, String};

    // ---- Mock Config contract ----
    //
    // Escrow reads admin/oracle/paused from the Config contract by symbol name
    // via `invoke_contract`. These unit tests exercise Escrow logic in isolation,
    // so we stand up a minimal Config exposing exactly the three functions Escrow
    // calls, plus `set_paused` to drive the circuit-breaker tests. End-to-end
    // Config+Escrow wiring is covered separately at the integration layer.
    #[contracttype]
    #[derive(Clone)]
    enum MockKey {
        Admin,
        Oracle,
        Paused,
    }

    #[contract]
    pub struct MockConfig;

    #[contractimpl]
    impl MockConfig {
        pub fn init(env: Env, admin: Address, oracle: Address) {
            env.storage().instance().set(&MockKey::Admin, &admin);
            env.storage().instance().set(&MockKey::Oracle, &oracle);
            env.storage().instance().set(&MockKey::Paused, &false);
        }
        pub fn get_admin(env: Env) -> Address {
            env.storage().instance().get(&MockKey::Admin).unwrap()
        }
        pub fn get_oracle(env: Env) -> Address {
            env.storage().instance().get(&MockKey::Oracle).unwrap()
        }
        pub fn is_paused(env: Env) -> bool {
            env.storage().instance().get(&MockKey::Paused).unwrap()
        }
        pub fn set_paused(env: Env, paused: bool) {
            env.storage().instance().set(&MockKey::Paused, &paused);
        }
    }

    // ---- Test harness ----
    //
    // Clients borrow `&env`, so we cannot store both the env and its clients in
    // one struct (self-reference). We hold the env + addresses and rebuild the
    // cheap clients per test.
    struct Setup {
        env: Env,
        escrow_id: Address,
        config_id: Address,
        token: Address,
        registry: Address,
        admin: Address,
        oracle: Address,
        funder: Address,
    }

    const INITIAL_MINT: i128 = 1_000_000;

    fn setup() -> Setup {
        let env = Env::default();
        env.mock_all_auths();

        let admin = Address::generate(&env);
        let oracle = Address::generate(&env);
        let funder = Address::generate(&env);

        let config_id = env.register(MockConfig, ());
        MockConfigClient::new(&env, &config_id).init(&admin, &oracle);

        let sac = env.register_stellar_asset_contract_v2(admin.clone());
        let token = sac.address();
        token::StellarAssetClient::new(&env, &token).mint(&funder, &INITIAL_MINT);

        let registry = Address::generate(&env);

        let escrow_id = env.register(EscrowContract, ());
        EscrowContractClient::new(&env, &escrow_id).init(&config_id, &token, &registry);

        Setup {
            env,
            escrow_id,
            config_id,
            token,
            registry,
            admin,
            oracle,
            funder,
        }
    }

    fn escrow<'a>(s: &Setup) -> EscrowContractClient<'a> {
        EscrowContractClient::new(&s.env, &s.escrow_id)
    }

    fn tokens<'a>(s: &Setup) -> token::TokenClient<'a> {
        token::TokenClient::new(&s.env, &s.token)
    }

    fn config<'a>(s: &Setup) -> MockConfigClient<'a> {
        MockConfigClient::new(&s.env, &s.config_id)
    }

    fn hash(env: &Env) -> BytesN<32> {
        BytesN::from_array(env, &[7u8; 32])
    }

    // 400 + 600 = 1000 total target.
    fn two_milestones(env: &Env) -> Vec<Milestone> {
        vec![
            env,
            Milestone {
                id: 1,
                description: String::from_str(env, "phase-1"),
                target_amount: 400,
            },
            Milestone {
                id: 2,
                description: String::from_str(env, "phase-2"),
                target_amount: 600,
            },
        ]
    }

    // Single milestone (target 400) for over-fund / refund scenarios.
    fn one_milestone(env: &Env) -> Vec<Milestone> {
        vec![
            env,
            Milestone {
                id: 1,
                description: String::from_str(env, "only"),
                target_amount: 400,
            },
        ]
    }

    // ---- Happy path & state transitions ----

    #[test]
    fn full_lifecycle_two_milestones() {
        let s = setup();
        let e = escrow(&s);
        let pid = e.create_program(&s.funder, &s.token, &two_milestones(&s.env));
        assert_eq!(pid, 1);

        e.fund(&pid, &1000);
        assert_eq!(tokens(&s).balance(&s.escrow_id), 1000);
        assert_eq!(tokens(&s).balance(&s.funder), INITIAL_MINT - 1000);
        assert_eq!(e.get_program(&pid).funded_amount, 1000);

        // Release milestone 1 (400) — program stays Active.
        e.attest_milestone(&pid, &1, &hash(&s.env));
        e.release(&pid, &1);
        assert_eq!(tokens(&s).balance(&s.registry), 400);
        let p = e.get_program(&pid);
        assert_eq!(p.released_amount, 400);
        assert_eq!(p.status, ProgramStatus::Active);

        // Release milestone 2 (600) — program completes.
        e.attest_milestone(&pid, &2, &hash(&s.env));
        e.release(&pid, &2);
        assert_eq!(tokens(&s).balance(&s.registry), 1000);
        assert_eq!(tokens(&s).balance(&s.escrow_id), 0);
        let p = e.get_program(&pid);
        assert_eq!(p.released_amount, 1000);
        assert_eq!(p.status, ProgramStatus::Completed);
    }

    #[test]
    fn program_ids_increment() {
        let s = setup();
        let e = escrow(&s);
        let a = e.create_program(&s.funder, &s.token, &two_milestones(&s.env));
        let b = e.create_program(&s.funder, &s.token, &two_milestones(&s.env));
        assert_eq!(a, 1);
        assert_eq!(b, 2);
    }

    #[test]
    fn fund_accumulates() {
        let s = setup();
        let e = escrow(&s);
        let pid = e.create_program(&s.funder, &s.token, &two_milestones(&s.env));
        e.fund(&pid, &300);
        e.fund(&pid, &200);
        assert_eq!(e.get_program(&pid).funded_amount, 500);
        assert_eq!(tokens(&s).balance(&s.escrow_id), 500);
    }

    #[test]
    fn overfund_then_refund_unspent() {
        let s = setup();
        let e = escrow(&s);
        let pid = e.create_program(&s.funder, &s.token, &one_milestone(&s.env));
        e.fund(&pid, &1000); // over-funds the 400 target
        e.attest_milestone(&pid, &1, &hash(&s.env));
        e.release(&pid, &1);
        assert_eq!(e.get_program(&pid).status, ProgramStatus::Completed);

        e.refund_unspent(&pid);
        // 600 unspent returns to the funder; 400 stays paid out to the registry.
        assert_eq!(tokens(&s).balance(&s.funder), INITIAL_MINT - 400);
        assert_eq!(tokens(&s).balance(&s.registry), 400);
        assert_eq!(tokens(&s).balance(&s.escrow_id), 0);
        assert_eq!(e.get_program(&pid).status, ProgramStatus::Refunded);
    }

    #[test]
    fn get_attestation_none_then_some() {
        let s = setup();
        let e = escrow(&s);
        let pid = e.create_program(&s.funder, &s.token, &two_milestones(&s.env));
        assert!(e.get_attestation(&pid, &1).is_none());
        e.attest_milestone(&pid, &1, &hash(&s.env));
        let rec = e.get_attestation(&pid, &1).unwrap();
        assert_eq!(rec.evidence_hash, hash(&s.env));
        assert_eq!(rec.oracle, s.oracle);
    }

    // ---- Two-key trust model (auth) ----

    #[test]
    fn release_succeeds_with_admin_key() {
        let s = setup();
        let e = escrow(&s);
        let pid = e.create_program(&s.funder, &s.token, &two_milestones(&s.env));
        e.fund(&pid, &1000);
        e.attest_milestone(&pid, &1, &hash(&s.env));

        // Only the admin authorizes release.
        s.env.mock_auths(&[MockAuth {
            address: &s.admin,
            invoke: &MockAuthInvoke {
                contract: &s.escrow_id,
                fn_name: "release",
                args: (pid, 1u32).into_val(&s.env),
                sub_invokes: &[],
            },
        }]);
        e.release(&pid, &1);
        assert_eq!(tokens(&s).balance(&s.registry), 400);
    }

    #[test]
    #[should_panic]
    fn release_rejects_oracle_key() {
        let s = setup();
        let e = escrow(&s);
        let pid = e.create_program(&s.funder, &s.token, &two_milestones(&s.env));
        e.fund(&pid, &1000);
        e.attest_milestone(&pid, &1, &hash(&s.env));

        // The oracle (wrong key) authorizes release — admin.require_auth() must fail.
        s.env.mock_auths(&[MockAuth {
            address: &s.oracle,
            invoke: &MockAuthInvoke {
                contract: &s.escrow_id,
                fn_name: "release",
                args: (pid, 1u32).into_val(&s.env),
                sub_invokes: &[],
            },
        }]);
        e.release(&pid, &1);
    }

    #[test]
    fn attest_succeeds_with_oracle_key() {
        let s = setup();
        let e = escrow(&s);
        let pid = e.create_program(&s.funder, &s.token, &two_milestones(&s.env));
        let h = hash(&s.env);

        s.env.mock_auths(&[MockAuth {
            address: &s.oracle,
            invoke: &MockAuthInvoke {
                contract: &s.escrow_id,
                fn_name: "attest_milestone",
                args: (pid, 1u32, h.clone()).into_val(&s.env),
                sub_invokes: &[],
            },
        }]);
        e.attest_milestone(&pid, &1, &h);
        assert!(e.get_attestation(&pid, &1).is_some());
    }

    #[test]
    #[should_panic]
    fn attest_rejects_admin_key() {
        let s = setup();
        let e = escrow(&s);
        let pid = e.create_program(&s.funder, &s.token, &two_milestones(&s.env));
        let h = hash(&s.env);

        // The admin (wrong key) authorizes attest — oracle.require_auth() must fail.
        s.env.mock_auths(&[MockAuth {
            address: &s.admin,
            invoke: &MockAuthInvoke {
                contract: &s.escrow_id,
                fn_name: "attest_milestone",
                args: (pid, 1u32, h.clone()).into_val(&s.env),
                sub_invokes: &[],
            },
        }]);
        e.attest_milestone(&pid, &1, &h);
    }

    // ---- Release invariants ----

    #[test]
    #[should_panic(expected = "milestone already released")]
    fn double_release_blocked() {
        let s = setup();
        let e = escrow(&s);
        let pid = e.create_program(&s.funder, &s.token, &two_milestones(&s.env));
        e.fund(&pid, &1000);
        e.attest_milestone(&pid, &1, &hash(&s.env));
        e.release(&pid, &1);
        e.release(&pid, &1);
    }

    #[test]
    #[should_panic(expected = "milestone not attested")]
    fn release_without_attestation_blocked() {
        let s = setup();
        let e = escrow(&s);
        let pid = e.create_program(&s.funder, &s.token, &two_milestones(&s.env));
        e.fund(&pid, &1000);
        e.release(&pid, &1);
    }

    #[test]
    #[should_panic(expected = "insufficient funded balance for milestone")]
    fn release_insufficient_funded_balance() {
        let s = setup();
        let e = escrow(&s);
        let pid = e.create_program(&s.funder, &s.token, &two_milestones(&s.env));
        e.fund(&pid, &400); // only covers milestone 1
        e.attest_milestone(&pid, &1, &hash(&s.env));
        e.release(&pid, &1);
        e.attest_milestone(&pid, &2, &hash(&s.env));
        e.release(&pid, &2); // needs 600, none left
    }

    #[test]
    #[should_panic(expected = "milestone already released")]
    fn attest_after_release_blocked() {
        let s = setup();
        let e = escrow(&s);
        let pid = e.create_program(&s.funder, &s.token, &two_milestones(&s.env));
        e.fund(&pid, &1000);
        e.attest_milestone(&pid, &1, &hash(&s.env));
        e.release(&pid, &1);
        e.attest_milestone(&pid, &1, &hash(&s.env));
    }

    // ---- Circuit breaker (pause blocks every write, incl. attest) ----

    #[test]
    #[should_panic(expected = "contract is paused")]
    fn pause_blocks_create() {
        let s = setup();
        config(&s).set_paused(&true);
        escrow(&s).create_program(&s.funder, &s.token, &two_milestones(&s.env));
    }

    #[test]
    #[should_panic(expected = "contract is paused")]
    fn pause_blocks_fund() {
        let s = setup();
        let e = escrow(&s);
        let pid = e.create_program(&s.funder, &s.token, &two_milestones(&s.env));
        config(&s).set_paused(&true);
        e.fund(&pid, &100);
    }

    #[test]
    #[should_panic(expected = "contract is paused")]
    fn pause_blocks_attest() {
        let s = setup();
        let e = escrow(&s);
        let pid = e.create_program(&s.funder, &s.token, &two_milestones(&s.env));
        config(&s).set_paused(&true);
        e.attest_milestone(&pid, &1, &hash(&s.env));
    }

    #[test]
    #[should_panic(expected = "contract is paused")]
    fn pause_blocks_release() {
        let s = setup();
        let e = escrow(&s);
        let pid = e.create_program(&s.funder, &s.token, &two_milestones(&s.env));
        e.fund(&pid, &1000);
        e.attest_milestone(&pid, &1, &hash(&s.env));
        config(&s).set_paused(&true);
        e.release(&pid, &1);
    }

    #[test]
    #[should_panic(expected = "contract is paused")]
    fn pause_blocks_refund() {
        let s = setup();
        let e = escrow(&s);
        let pid = e.create_program(&s.funder, &s.token, &one_milestone(&s.env));
        e.fund(&pid, &1000);
        e.attest_milestone(&pid, &1, &hash(&s.env));
        e.release(&pid, &1);
        config(&s).set_paused(&true);
        e.refund_unspent(&pid);
    }

    // ---- create_program validation ----

    #[test]
    #[should_panic(expected = "token must be the configured settlement token")]
    fn wrong_token_rejected() {
        let s = setup();
        let other = s
            .env
            .register_stellar_asset_contract_v2(s.admin.clone())
            .address();
        escrow(&s).create_program(&s.funder, &other, &two_milestones(&s.env));
    }

    #[test]
    #[should_panic(expected = "program needs at least one milestone")]
    fn empty_milestones_rejected() {
        let s = setup();
        let empty: Vec<Milestone> = vec![&s.env];
        escrow(&s).create_program(&s.funder, &s.token, &empty);
    }

    #[test]
    #[should_panic(expected = "duplicate milestone id")]
    fn duplicate_milestone_id_rejected() {
        let s = setup();
        let ms = vec![
            &s.env,
            Milestone {
                id: 1,
                description: String::from_str(&s.env, "a"),
                target_amount: 100,
            },
            Milestone {
                id: 1,
                description: String::from_str(&s.env, "b"),
                target_amount: 200,
            },
        ];
        escrow(&s).create_program(&s.funder, &s.token, &ms);
    }

    #[test]
    #[should_panic(expected = "milestone target_amount must be positive")]
    fn nonpositive_milestone_amount_rejected() {
        let s = setup();
        let ms = vec![
            &s.env,
            Milestone {
                id: 1,
                description: String::from_str(&s.env, "bad"),
                target_amount: 0,
            },
        ];
        escrow(&s).create_program(&s.funder, &s.token, &ms);
    }

    // ---- fund / attest validation ----

    #[test]
    #[should_panic(expected = "amount must be positive")]
    fn fund_nonpositive_rejected() {
        let s = setup();
        let e = escrow(&s);
        let pid = e.create_program(&s.funder, &s.token, &two_milestones(&s.env));
        e.fund(&pid, &0);
    }

    #[test]
    #[should_panic(expected = "program is not active")]
    fn fund_inactive_program_rejected() {
        let s = setup();
        let e = escrow(&s);
        let pid = e.create_program(&s.funder, &s.token, &one_milestone(&s.env));
        e.fund(&pid, &400);
        e.attest_milestone(&pid, &1, &hash(&s.env));
        e.release(&pid, &1); // -> Completed
        e.fund(&pid, &100);
    }

    #[test]
    #[should_panic(expected = "unknown milestone id")]
    fn attest_unknown_milestone_rejected() {
        let s = setup();
        let e = escrow(&s);
        let pid = e.create_program(&s.funder, &s.token, &two_milestones(&s.env));
        e.attest_milestone(&pid, &99, &hash(&s.env));
    }

    // ---- refund gating ----

    #[test]
    #[should_panic(expected = "program must be completed to refund")]
    fn refund_before_completed_rejected() {
        let s = setup();
        let e = escrow(&s);
        let pid = e.create_program(&s.funder, &s.token, &two_milestones(&s.env));
        e.fund(&pid, &1000);
        e.refund_unspent(&pid); // still Active
    }

    #[test]
    #[should_panic(expected = "nothing to refund")]
    fn refund_with_no_unspent_rejected() {
        let s = setup();
        let e = escrow(&s);
        let pid = e.create_program(&s.funder, &s.token, &two_milestones(&s.env));
        e.fund(&pid, &1000); // funds exactly the 1000 target
        e.attest_milestone(&pid, &1, &hash(&s.env));
        e.release(&pid, &1);
        e.attest_milestone(&pid, &2, &hash(&s.env));
        e.release(&pid, &2); // -> Completed, unspent == 0
        e.refund_unspent(&pid);
    }

    #[test]
    #[should_panic(expected = "program must be completed to refund")]
    fn double_refund_blocked() {
        let s = setup();
        let e = escrow(&s);
        let pid = e.create_program(&s.funder, &s.token, &one_milestone(&s.env));
        e.fund(&pid, &1000);
        e.attest_milestone(&pid, &1, &hash(&s.env));
        e.release(&pid, &1); // -> Completed
        e.refund_unspent(&pid); // -> Refunded
        e.refund_unspent(&pid); // status now Refunded, not Completed
    }

    // ---- reclaim_expired (VoucherRegistry hook) ----

    // Simulate the registry returning `amount` to escrow, as `expire` does.
    fn registry_returns(s: &Setup, pid: u64, amount: i128) {
        tokens(s).transfer(&s.registry, &s.escrow_id, &amount);
        escrow(s).reclaim_expired(&pid, &amount);
    }

    #[test]
    fn reclaim_then_refund_returns_expired_funds_to_funder() {
        let s = setup();
        let e = escrow(&s);
        let pid = e.create_program(&s.funder, &s.token, &two_milestones(&s.env));
        e.fund(&pid, &1000);
        e.attest_milestone(&pid, &1, &hash(&s.env));
        e.release(&pid, &1);
        e.attest_milestone(&pid, &2, &hash(&s.env));
        e.release(&pid, &2); // Completed, unspent == 0

        registry_returns(&s, pid, 100);
        assert_eq!(e.get_program(&pid).released_amount, 900);

        e.refund_unspent(&pid);
        assert_eq!(tokens(&s).balance(&s.funder), INITIAL_MINT - 900);
        assert_eq!(tokens(&s).balance(&s.escrow_id), 0);
    }

    #[test]
    fn reclaim_midway_does_not_block_completion() {
        let s = setup();
        let e = escrow(&s);
        let pid = e.create_program(&s.funder, &s.token, &two_milestones(&s.env));
        e.fund(&pid, &1000);
        e.attest_milestone(&pid, &1, &hash(&s.env));
        e.release(&pid, &1);
        registry_returns(&s, pid, 100);

        e.attest_milestone(&pid, &2, &hash(&s.env));
        e.release(&pid, &2);
        // released_amount (900) != total target (1000), yet the program completes.
        assert_eq!(e.get_program(&pid).status, ProgramStatus::Completed);
    }

    #[test]
    fn reclaim_after_refund_forwards_to_funder() {
        let s = setup();
        let e = escrow(&s);
        let pid = e.create_program(&s.funder, &s.token, &one_milestone(&s.env));
        e.fund(&pid, &1000);
        e.attest_milestone(&pid, &1, &hash(&s.env));
        e.release(&pid, &1);
        e.refund_unspent(&pid); // 600 back to funder; status Refunded
        assert_eq!(tokens(&s).balance(&s.funder), INITIAL_MINT - 400);

        registry_returns(&s, pid, 100);
        // Forwarded immediately: nothing stranded in escrow.
        assert_eq!(tokens(&s).balance(&s.escrow_id), 0);
        assert_eq!(tokens(&s).balance(&s.funder), INITIAL_MINT - 300);
    }

    #[test]
    #[should_panic(expected = "reclaim exceeds released amount")]
    fn reclaim_more_than_released_rejected() {
        let s = setup();
        let e = escrow(&s);
        let pid = e.create_program(&s.funder, &s.token, &one_milestone(&s.env));
        e.fund(&pid, &400);
        e.attest_milestone(&pid, &1, &hash(&s.env));
        e.release(&pid, &1);
        e.reclaim_expired(&pid, &401);
    }

    #[test]
    #[should_panic(expected = "amount must be positive")]
    fn reclaim_zero_rejected() {
        let s = setup();
        let e = escrow(&s);
        let pid = e.create_program(&s.funder, &s.token, &one_milestone(&s.env));
        e.reclaim_expired(&pid, &0);
    }

    #[test]
    #[should_panic(expected = "contract is paused")]
    fn reclaim_blocked_when_paused() {
        let s = setup();
        let e = escrow(&s);
        let pid = e.create_program(&s.funder, &s.token, &one_milestone(&s.env));
        config(&s).set_paused(&true);
        e.reclaim_expired(&pid, &1);
    }

    #[test]
    #[should_panic]
    fn reclaim_rejects_non_registry_caller() {
        let s = setup();
        let e = escrow(&s);
        let pid = e.create_program(&s.funder, &s.token, &one_milestone(&s.env));
        s.env.mock_auths(&[MockAuth {
            address: &s.admin,
            invoke: &MockAuthInvoke {
                contract: &s.escrow_id,
                fn_name: "reclaim_expired",
                args: (pid, 1i128).into_val(&s.env),
                sub_invokes: &[],
            },
        }]);
        e.reclaim_expired(&pid, &1);
    }

    /// Emitted events: payload checks and pinned topic names (indexers key on these).
    mod events {
        use super::*;
        use soroban_sdk::events::Event;
        use soroban_sdk::testutils::Events as _;

        fn emitted<E: Event>(s: &Setup, e: E) -> bool {
            s.env
                .events()
                .all()
                .events()
                .contains(&e.to_xdr(&s.env, &s.escrow_id))
        }

        #[test]
        fn create_program_emits_program_created() {
            let s = setup();
            let pid = escrow(&s).create_program(&s.funder, &s.token, &one_milestone(&s.env));
            assert!(emitted(
                &s,
                ProgramCreated {
                    program_id: pid,
                    funder: s.funder.clone()
                }
            ));
        }

        #[test]
        fn fund_emits_funded() {
            let s = setup();
            let pid = escrow(&s).create_program(&s.funder, &s.token, &one_milestone(&s.env));
            escrow(&s).fund(&pid, &400);
            assert!(emitted(
                &s,
                Funded {
                    program_id: pid,
                    amount: 400
                }
            ));
        }

        #[test]
        fn attest_emits_milestone_attested() {
            let s = setup();
            let pid = escrow(&s).create_program(&s.funder, &s.token, &one_milestone(&s.env));
            escrow(&s).attest_milestone(&pid, &1, &hash(&s.env));
            assert!(emitted(
                &s,
                MilestoneAttested {
                    program_id: pid,
                    milestone_id: 1,
                    evidence_hash: hash(&s.env)
                }
            ));
        }

        #[test]
        fn release_emits_released() {
            let s = setup();
            let e = escrow(&s);
            let pid = e.create_program(&s.funder, &s.token, &one_milestone(&s.env));
            e.fund(&pid, &400);
            e.attest_milestone(&pid, &1, &hash(&s.env));
            e.release(&pid, &1);
            assert!(emitted(
                &s,
                Released {
                    program_id: pid,
                    milestone_id: 1,
                    amount: 400
                }
            ));
        }

        #[test]
        fn refund_emits_refunded() {
            let s = setup();
            let e = escrow(&s);
            let pid = e.create_program(&s.funder, &s.token, &one_milestone(&s.env));
            e.fund(&pid, &1000);
            e.attest_milestone(&pid, &1, &hash(&s.env));
            e.release(&pid, &1);
            e.refund_unspent(&pid);
            assert!(emitted(
                &s,
                Refunded {
                    program_id: pid,
                    amount: 600
                }
            ));
        }

        #[test]
        fn reclaim_emits_reclaimed() {
            let s = setup();
            let e = escrow(&s);
            let pid = e.create_program(&s.funder, &s.token, &one_milestone(&s.env));
            e.fund(&pid, &400);
            e.attest_milestone(&pid, &1, &hash(&s.env));
            e.release(&pid, &1);
            registry_returns(&s, pid, 100);
            assert!(emitted(
                &s,
                Reclaimed {
                    program_id: pid,
                    amount: 100
                }
            ));
        }

        #[test]
        fn topic_names_are_pinned() {
            let env = Env::default();
            let h = BytesN::from_array(&env, &[0u8; 32]);
            let funder = Address::generate(&env);
            let topic =
                |name: &str| soroban_sdk::vec![&env, Symbol::new(&env, name).into_val(&env)];
            assert_eq!(
                ProgramCreated {
                    program_id: 1,
                    funder
                }
                .topics(&env),
                topic("program_created")
            );
            assert_eq!(
                Funded {
                    program_id: 1,
                    amount: 1
                }
                .topics(&env),
                topic("funded")
            );
            assert_eq!(
                MilestoneAttested {
                    program_id: 1,
                    milestone_id: 1,
                    evidence_hash: h
                }
                .topics(&env),
                topic("milestone_attested")
            );
            assert_eq!(
                Released {
                    program_id: 1,
                    milestone_id: 1,
                    amount: 1
                }
                .topics(&env),
                topic("released")
            );
            assert_eq!(
                Refunded {
                    program_id: 1,
                    amount: 1
                }
                .topics(&env),
                topic("refunded")
            );
            assert_eq!(
                Reclaimed {
                    program_id: 1,
                    amount: 1
                }
                .topics(&env),
                topic("reclaimed")
            );
        }
    }
}
