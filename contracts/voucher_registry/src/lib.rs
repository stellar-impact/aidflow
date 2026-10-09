#![no_std]

//! AidFlow VoucherRegistry Contract
//!
//! Custodies released milestone funds and turns them into per-beneficiary
//! vouchers: batch issuance (admin), pull-based claims (recipient), and expiry
//! (anyone, after the deadline) that returns unclaimed funds to Escrow.
//!
//! Design decisions (approved 2026-10-08):
//!   1. `issue_batch` is admin-gated (Config admin), not "escrow auth": Escrow
//!      never calls the registry and has no recipient list.
//!   2. Solvency — `outstanding` tracks the sum of Unclaimed vouchers and may
//!      never exceed the registry's token balance, so it cannot over-issue.
//!   3. `expire` transfers the amount back to Escrow and calls
//!      `reclaim_expired`, which re-credits the program's accounting.
//!   4. The Config circuit breaker blocks issue, claim and expire.
//!
//! Invariants: status moves only Unclaimed -> Claimed or Unclaimed -> Expired;
//! checks-effects-interactions around every transfer; checked arithmetic.
//!
//! @stable

use aidflow_contract_types::{Voucher, VoucherStatus};
use soroban_sdk::{
    contract, contractevent, contractimpl, contracttype, token, vec, Address, Env, IntoVal, Symbol,
    Vec,
};

/// Instance-storage TTL bounds (ledgers, ~5s each): bump at ~23 days, extend to ~30.
const TTL_THRESHOLD: u32 = 397_440;
const TTL_EXTEND_TO: u32 = 518_400;

/// Persistent-storage TTL bounds for voucher entries (~30/37 days).
const P_TTL_THRESHOLD: u32 = 518_400;
const P_TTL_EXTEND_TO: u32 = 639_360;

/// Maximum recipients in one `issue_batch` call. Each voucher is one persistent
/// ledger write and a Soroban transaction allows 50 writes / 100 footprint
/// entries, so ~45 is the hard ceiling; 40 leaves headroom.
const MAX_BATCH: u32 = 40;

#[contracttype]
#[derive(Clone)]
pub enum DataKey {
    /// Config/AccessControl contract address (instance)
    Config,
    /// Escrow contract that expired funds return to (instance)
    Escrow,
    /// Settlement token address (instance)
    Token,
    /// Global voucher counter (instance)
    VoucherCounter,
    /// Sum of Unclaimed voucher amounts (instance)
    Outstanding,
    /// Voucher record by id (persistent)
    Voucher(u64),
}

/// Event emitted when a voucher is issued
#[contractevent]
pub struct VoucherIssued {
    pub voucher_id: u64,
    pub program_id: u64,
    pub recipient: Address,
    pub amount: i128,
}

/// Event emitted when a voucher is claimed
#[contractevent]
pub struct VoucherClaimed {
    pub voucher_id: u64,
    pub recipient: Address,
    pub amount: i128,
}

/// Event emitted when a voucher expires
#[contractevent]
pub struct VoucherExpired {
    pub voucher_id: u64,
    pub amount: i128,
}

#[contract]
pub struct VoucherRegistryContract;

#[contractimpl]
impl VoucherRegistryContract {
    /// Initialize with the Config contract, the Escrow contract and the
    /// settlement token. Requires the Config admin's authorization so it cannot
    /// be front-run. Can only be called once.
    ///
    /// @stable
    pub fn init(env: Env, config_contract: Address, escrow_contract: Address, token: Address) {
        if env.storage().instance().has(&DataKey::Config) {
            panic!("already initialized");
        }
        let admin: Address = env.invoke_contract(
            &config_contract,
            &Symbol::new(&env, "get_admin"),
            Vec::new(&env),
        );
        admin.require_auth();

        let s = env.storage().instance();
        s.set(&DataKey::Config, &config_contract);
        s.set(&DataKey::Escrow, &escrow_contract);
        s.set(&DataKey::Token, &token);
        s.set(&DataKey::VoucherCounter, &0u64);
        s.set(&DataKey::Outstanding, &0i128);
        bump_instance_ttl(&env);
    }

    /// Issue up to 40 vouchers for a program. Requires admin auth. The total
    /// issued plus everything already outstanding must be covered by the
    /// registry's token balance. Returns the new voucher ids.
    ///
    /// @stable
    pub fn issue_batch(
        env: Env,
        program_id: u64,
        recipients: Vec<(Address, i128)>,
        expiry: u64,
    ) -> Vec<u64> {
        require_admin(&env);
        ensure_not_paused(&env);

        let count = recipients.len();
        if count == 0 {
            panic!("batch must not be empty");
        }
        if count > MAX_BATCH {
            panic!("batch exceeds 40 recipients");
        }
        if expiry <= env.ledger().timestamp() {
            panic!("expiry must be in the future");
        }

        // Validate amounts and total them before writing anything.
        let mut total: i128 = 0;
        for i in 0..count {
            let (_, amount) = recipients.get(i).unwrap();
            if amount <= 0 {
                panic!("voucher amount must be positive");
            }
            total = total.checked_add(amount).expect("batch total overflow");
        }

        // Solvency: outstanding + total <= token balance held by this contract.
        let outstanding = get_outstanding(&env);
        let new_outstanding = outstanding
            .checked_add(total)
            .expect("outstanding overflow");
        let here = env.current_contract_address();
        if new_outstanding > token_client(&env).balance(&here) {
            panic!("insufficient registry balance");
        }

        let mut counter: u64 = env
            .storage()
            .instance()
            .get(&DataKey::VoucherCounter)
            .unwrap();
        let mut ids: Vec<u64> = Vec::new(&env);
        for i in 0..count {
            let (recipient, amount) = recipients.get(i).unwrap();
            counter = counter.checked_add(1).expect("counter overflow");
            let voucher = Voucher {
                id: counter,
                program_id,
                recipient: recipient.clone(),
                amount,
                status: VoucherStatus::Unclaimed,
                expiry,
                claimed_at: None,
            };
            save_voucher(&env, &voucher);
            ids.push_back(counter);
            VoucherIssued {
                voucher_id: counter,
                program_id,
                recipient,
                amount,
            }
            .publish(&env);
        }

        env.storage()
            .instance()
            .set(&DataKey::VoucherCounter, &counter);
        env.storage()
            .instance()
            .set(&DataKey::Outstanding, &new_outstanding);
        bump_instance_ttl(&env);
        ids
    }

    /// Claim a voucher: pays its amount to the recipient. Requires the
    /// recipient's auth and `now <= expiry`.
    ///
    /// @stable
    pub fn claim(env: Env, voucher_id: u64) {
        ensure_not_paused(&env);
        let mut voucher = load_voucher(&env, voucher_id);
        voucher.recipient.require_auth();

        if voucher.status != VoucherStatus::Unclaimed {
            panic!("voucher is not unclaimed");
        }
        let now = env.ledger().timestamp();
        if now > voucher.expiry {
            panic!("voucher has expired");
        }

        // EFFECTS
        voucher.status = VoucherStatus::Claimed;
        voucher.claimed_at = Some(now);
        save_voucher(&env, &voucher);
        sub_outstanding(&env, voucher.amount);

        // INTERACTIONS
        let here = env.current_contract_address();
        token_client(&env).transfer(&here, &voucher.recipient, &voucher.amount);

        VoucherClaimed {
            voucher_id,
            recipient: voucher.recipient,
            amount: voucher.amount,
        }
        .publish(&env);
    }

    /// Expire an unclaimed voucher once `now > expiry` and return its funds to
    /// Escrow (crediting the program via `reclaim_expired`). Anyone may call.
    ///
    /// @stable
    pub fn expire(env: Env, voucher_id: u64) {
        ensure_not_paused(&env);
        let mut voucher = load_voucher(&env, voucher_id);

        if voucher.status != VoucherStatus::Unclaimed {
            panic!("voucher is not unclaimed");
        }
        if env.ledger().timestamp() <= voucher.expiry {
            panic!("voucher has not expired");
        }

        // EFFECTS
        voucher.status = VoucherStatus::Expired;
        save_voucher(&env, &voucher);
        sub_outstanding(&env, voucher.amount);

        // INTERACTIONS: return funds, then let Escrow re-credit the program.
        let escrow: Address = env.storage().instance().get(&DataKey::Escrow).unwrap();
        let here = env.current_contract_address();
        token_client(&env).transfer(&here, &escrow, &voucher.amount);
        env.invoke_contract::<()>(
            &escrow,
            &Symbol::new(&env, "reclaim_expired"),
            vec![
                &env,
                voucher.program_id.into_val(&env),
                voucher.amount.into_val(&env),
            ],
        );

        VoucherExpired {
            voucher_id,
            amount: voucher.amount,
        }
        .publish(&env);
    }

    /// Get a voucher's details.
    ///
    /// @stable
    pub fn get_voucher(env: Env, voucher_id: u64) -> Voucher {
        load_voucher(&env, voucher_id)
    }

    /// Sum of all Unclaimed voucher amounts.
    ///
    /// @stable
    pub fn get_outstanding(env: Env) -> i128 {
        get_outstanding(&env)
    }
}

// ---- internal helpers ----

fn bump_instance_ttl(env: &Env) {
    env.storage()
        .instance()
        .extend_ttl(TTL_THRESHOLD, TTL_EXTEND_TO);
}

fn save_voucher(env: &Env, voucher: &Voucher) {
    let key = DataKey::Voucher(voucher.id);
    env.storage().persistent().set(&key, voucher);
    env.storage()
        .persistent()
        .extend_ttl(&key, P_TTL_THRESHOLD, P_TTL_EXTEND_TO);
}

fn load_voucher(env: &Env, voucher_id: u64) -> Voucher {
    env.storage()
        .persistent()
        .get(&DataKey::Voucher(voucher_id))
        .expect("voucher not found")
}

fn get_outstanding(env: &Env) -> i128 {
    env.storage()
        .instance()
        .get(&DataKey::Outstanding)
        .unwrap_or(0)
}

fn sub_outstanding(env: &Env, amount: i128) {
    let next = get_outstanding(env)
        .checked_sub(amount)
        .filter(|v| *v >= 0)
        .expect("outstanding underflow");
    env.storage().instance().set(&DataKey::Outstanding, &next);
    bump_instance_ttl(env);
}

fn token_client(env: &Env) -> token::TokenClient<'_> {
    let token: Address = env.storage().instance().get(&DataKey::Token).unwrap();
    token::TokenClient::new(env, &token)
}

// ---- Config (access control + circuit breaker) queries ----

fn config_addr(env: &Env) -> Address {
    env.storage().instance().get(&DataKey::Config).unwrap()
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
    use soroban_sdk::testutils::{Address as _, Ledger, MockAuth, MockAuthInvoke};

    // ---- Mocks ----
    //
    // The registry reads admin/paused from Config and calls `reclaim_expired` on
    // Escrow, both by symbol name. These unit tests stand up minimal stand-ins;
    // real Config+Escrow+Registry wiring belongs to the integration test layer.
    #[contracttype]
    #[derive(Clone)]
    enum MockKey {
        Admin,
        Paused,
        Reclaims,
    }

    #[contract]
    pub struct MockConfig;

    #[contractimpl]
    impl MockConfig {
        pub fn init(env: Env, admin: Address) {
            env.storage().instance().set(&MockKey::Admin, &admin);
            env.storage().instance().set(&MockKey::Paused, &false);
        }
        pub fn get_admin(env: Env) -> Address {
            env.storage().instance().get(&MockKey::Admin).unwrap()
        }
        pub fn is_paused(env: Env) -> bool {
            env.storage().instance().get(&MockKey::Paused).unwrap()
        }
        pub fn set_paused(env: Env, paused: bool) {
            env.storage().instance().set(&MockKey::Paused, &paused);
        }
    }

    #[contract]
    pub struct MockEscrow;

    #[contractimpl]
    impl MockEscrow {
        pub fn reclaim_expired(env: Env, program_id: u64, amount: i128) {
            let mut calls: Vec<(u64, i128)> = env
                .storage()
                .instance()
                .get(&MockKey::Reclaims)
                .unwrap_or(Vec::new(&env));
            calls.push_back((program_id, amount));
            env.storage().instance().set(&MockKey::Reclaims, &calls);
        }
        pub fn reclaims(env: Env) -> Vec<(u64, i128)> {
            env.storage()
                .instance()
                .get(&MockKey::Reclaims)
                .unwrap_or(Vec::new(&env))
        }
    }

    // ---- Harness ----
    struct Setup {
        env: Env,
        registry_id: Address,
        config_id: Address,
        escrow_id: Address,
        token: Address,
        admin: Address,
    }

    const EXPIRY: u64 = 10_000;
    const FUNDS: i128 = 10_000;

    fn setup() -> Setup {
        let env = Env::default();
        env.mock_all_auths();
        env.ledger().set_timestamp(1_000);

        let admin = Address::generate(&env);
        let config_id = env.register(MockConfig, ());
        MockConfigClient::new(&env, &config_id).init(&admin);
        let escrow_id = env.register(MockEscrow, ());

        let sac = env.register_stellar_asset_contract_v2(admin.clone());
        let token = sac.address();

        let registry_id = env.register(VoucherRegistryContract, ());
        VoucherRegistryContractClient::new(&env, &registry_id).init(&config_id, &escrow_id, &token);
        // Simulate Escrow.release() having delivered funds to the registry.
        token::StellarAssetClient::new(&env, &token).mint(&registry_id, &FUNDS);

        Setup {
            env,
            registry_id,
            config_id,
            escrow_id,
            token,
            admin,
        }
    }

    fn reg<'a>(s: &Setup) -> VoucherRegistryContractClient<'a> {
        VoucherRegistryContractClient::new(&s.env, &s.registry_id)
    }

    fn tokens<'a>(s: &Setup) -> token::TokenClient<'a> {
        token::TokenClient::new(&s.env, &s.token)
    }

    fn config<'a>(s: &Setup) -> MockConfigClient<'a> {
        MockConfigClient::new(&s.env, &s.config_id)
    }

    fn escrow<'a>(s: &Setup) -> MockEscrowClient<'a> {
        MockEscrowClient::new(&s.env, &s.escrow_id)
    }

    fn one(s: &Setup, amount: i128) -> (Address, Vec<(Address, i128)>) {
        let who = Address::generate(&s.env);
        (who.clone(), vec![&s.env, (who, amount)])
    }

    fn warp(s: &Setup, ts: u64) {
        s.env.ledger().set_timestamp(ts);
    }

    // ---- Issuance ----

    #[test]
    fn issue_batch_creates_vouchers() {
        let s = setup();
        let a = Address::generate(&s.env);
        let b = Address::generate(&s.env);
        let ids = reg(&s).issue_batch(
            &7,
            &vec![&s.env, (a.clone(), 300), (b.clone(), 200)],
            &EXPIRY,
        );
        assert_eq!(ids, vec![&s.env, 1u64, 2u64]);

        let v = reg(&s).get_voucher(&1);
        assert_eq!(v.program_id, 7);
        assert_eq!(v.recipient, a);
        assert_eq!(v.amount, 300);
        assert_eq!(v.status, VoucherStatus::Unclaimed);
        assert_eq!(v.expiry, EXPIRY);
        assert_eq!(v.claimed_at, None);
        assert_eq!(reg(&s).get_voucher(&2).recipient, b);
        assert_eq!(reg(&s).get_outstanding(), 500);
    }

    #[test]
    fn ids_continue_across_batches() {
        let s = setup();
        let (_, r1) = one(&s, 10);
        let (_, r2) = one(&s, 10);
        reg(&s).issue_batch(&1, &r1, &EXPIRY);
        let ids = reg(&s).issue_batch(&1, &r2, &EXPIRY);
        assert_eq!(ids, vec![&s.env, 2u64]);
    }

    #[test]
    fn batch_of_exactly_40_succeeds() {
        let s = setup();
        let mut rs: Vec<(Address, i128)> = Vec::new(&s.env);
        for _ in 0..40 {
            rs.push_back((Address::generate(&s.env), 1));
        }
        assert_eq!(reg(&s).issue_batch(&1, &rs, &EXPIRY).len(), 40);
        assert_eq!(reg(&s).get_outstanding(), 40);
    }

    #[test]
    #[should_panic(expected = "batch exceeds 40 recipients")]
    fn batch_of_41_rejected() {
        let s = setup();
        let mut rs: Vec<(Address, i128)> = Vec::new(&s.env);
        for _ in 0..41 {
            rs.push_back((Address::generate(&s.env), 1));
        }
        reg(&s).issue_batch(&1, &rs, &EXPIRY);
    }

    #[test]
    #[should_panic(expected = "batch must not be empty")]
    fn empty_batch_rejected() {
        let s = setup();
        reg(&s).issue_batch(&1, &Vec::new(&s.env), &EXPIRY);
    }

    #[test]
    #[should_panic(expected = "voucher amount must be positive")]
    fn zero_amount_rejected() {
        let s = setup();
        let (_, r) = one(&s, 0);
        reg(&s).issue_batch(&1, &r, &EXPIRY);
    }

    #[test]
    #[should_panic(expected = "voucher amount must be positive")]
    fn negative_amount_rejected() {
        let s = setup();
        let (_, r) = one(&s, -5);
        reg(&s).issue_batch(&1, &r, &EXPIRY);
    }

    #[test]
    #[should_panic(expected = "expiry must be in the future")]
    fn expiry_equal_to_now_rejected() {
        let s = setup();
        let (_, r) = one(&s, 10);
        reg(&s).issue_batch(&1, &r, &1_000);
    }

    #[test]
    #[should_panic(expected = "insufficient registry balance")]
    fn over_issuing_beyond_balance_rejected() {
        let s = setup();
        let (_, r) = one(&s, FUNDS + 1);
        reg(&s).issue_batch(&1, &r, &EXPIRY);
    }

    #[test]
    #[should_panic(expected = "insufficient registry balance")]
    fn over_issuing_counts_outstanding() {
        let s = setup();
        let (_, r1) = one(&s, FUNDS);
        reg(&s).issue_batch(&1, &r1, &EXPIRY);
        let (_, r2) = one(&s, 1);
        reg(&s).issue_batch(&1, &r2, &EXPIRY);
    }

    #[test]
    #[should_panic(expected = "batch total overflow")]
    fn batch_total_overflow_rejected() {
        let s = setup();
        let a = Address::generate(&s.env);
        let b = Address::generate(&s.env);
        reg(&s).issue_batch(&1, &vec![&s.env, (a, i128::MAX), (b, 1)], &EXPIRY);
    }

    #[test]
    #[should_panic]
    fn issue_rejects_non_admin() {
        let s = setup();
        let imposter = Address::generate(&s.env);
        let (_, r) = one(&s, 10);
        s.env.mock_auths(&[MockAuth {
            address: &imposter,
            invoke: &MockAuthInvoke {
                contract: &s.registry_id,
                fn_name: "issue_batch",
                args: (1u64, r.clone(), EXPIRY).into_val(&s.env),
                sub_invokes: &[],
            },
        }]);
        reg(&s).issue_batch(&1, &r, &EXPIRY);
    }

    #[test]
    fn issue_succeeds_with_admin_auth_only() {
        let s = setup();
        let (_, r) = one(&s, 10);
        s.env.mock_auths(&[MockAuth {
            address: &s.admin,
            invoke: &MockAuthInvoke {
                contract: &s.registry_id,
                fn_name: "issue_batch",
                args: (1u64, r.clone(), EXPIRY).into_val(&s.env),
                sub_invokes: &[],
            },
        }]);
        assert_eq!(reg(&s).issue_batch(&1, &r, &EXPIRY).len(), 1);
    }

    // ---- Claim ----

    #[test]
    fn claim_pays_recipient() {
        let s = setup();
        let (who, r) = one(&s, 300);
        reg(&s).issue_batch(&1, &r, &EXPIRY);
        warp(&s, 2_000);
        reg(&s).claim(&1);

        assert_eq!(tokens(&s).balance(&who), 300);
        assert_eq!(tokens(&s).balance(&s.registry_id), FUNDS - 300);
        let v = reg(&s).get_voucher(&1);
        assert_eq!(v.status, VoucherStatus::Claimed);
        assert_eq!(v.claimed_at, Some(2_000));
        assert_eq!(reg(&s).get_outstanding(), 0);
    }

    #[test]
    fn claim_allowed_exactly_at_expiry() {
        let s = setup();
        let (who, r) = one(&s, 50);
        reg(&s).issue_batch(&1, &r, &EXPIRY);
        warp(&s, EXPIRY);
        reg(&s).claim(&1);
        assert_eq!(tokens(&s).balance(&who), 50);
    }

    #[test]
    #[should_panic(expected = "voucher is not unclaimed")]
    fn double_claim_rejected() {
        let s = setup();
        let (_, r) = one(&s, 50);
        reg(&s).issue_batch(&1, &r, &EXPIRY);
        reg(&s).claim(&1);
        reg(&s).claim(&1);
    }

    #[test]
    #[should_panic(expected = "voucher has expired")]
    fn claim_after_expiry_rejected() {
        let s = setup();
        let (_, r) = one(&s, 50);
        reg(&s).issue_batch(&1, &r, &EXPIRY);
        warp(&s, EXPIRY + 1);
        reg(&s).claim(&1);
    }

    #[test]
    #[should_panic(expected = "voucher is not unclaimed")]
    fn claim_after_expire_rejected() {
        let s = setup();
        let (_, r) = one(&s, 50);
        reg(&s).issue_batch(&1, &r, &EXPIRY);
        warp(&s, EXPIRY + 1);
        reg(&s).expire(&1);
        reg(&s).claim(&1);
    }

    #[test]
    #[should_panic]
    fn claim_rejects_non_recipient() {
        let s = setup();
        let (_, r) = one(&s, 50);
        reg(&s).issue_batch(&1, &r, &EXPIRY);
        let thief = Address::generate(&s.env);
        s.env.mock_auths(&[MockAuth {
            address: &thief,
            invoke: &MockAuthInvoke {
                contract: &s.registry_id,
                fn_name: "claim",
                args: (1u64,).into_val(&s.env),
                sub_invokes: &[],
            },
        }]);
        reg(&s).claim(&1);
    }

    #[test]
    #[should_panic(expected = "voucher not found")]
    fn claim_unknown_voucher_rejected() {
        let s = setup();
        reg(&s).claim(&99);
    }

    // ---- Expire ----

    #[test]
    fn expire_returns_funds_to_escrow_and_reclaims() {
        let s = setup();
        let (_, r) = one(&s, 300);
        reg(&s).issue_batch(&42, &r, &EXPIRY);
        warp(&s, EXPIRY + 1);
        reg(&s).expire(&1);

        assert_eq!(reg(&s).get_voucher(&1).status, VoucherStatus::Expired);
        assert_eq!(reg(&s).get_outstanding(), 0);
        assert_eq!(tokens(&s).balance(&s.escrow_id), 300);
        assert_eq!(tokens(&s).balance(&s.registry_id), FUNDS - 300);
        assert_eq!(escrow(&s).reclaims(), vec![&s.env, (42u64, 300i128)]);
    }

    #[test]
    #[should_panic(expected = "voucher has not expired")]
    fn expire_at_exactly_expiry_rejected() {
        let s = setup();
        let (_, r) = one(&s, 50);
        reg(&s).issue_batch(&1, &r, &EXPIRY);
        warp(&s, EXPIRY);
        reg(&s).expire(&1);
    }

    #[test]
    #[should_panic(expected = "voucher is not unclaimed")]
    fn double_expire_rejected() {
        let s = setup();
        let (_, r) = one(&s, 50);
        reg(&s).issue_batch(&1, &r, &EXPIRY);
        warp(&s, EXPIRY + 1);
        reg(&s).expire(&1);
        reg(&s).expire(&1);
    }

    #[test]
    #[should_panic(expected = "voucher is not unclaimed")]
    fn expire_after_claim_rejected() {
        let s = setup();
        let (_, r) = one(&s, 50);
        reg(&s).issue_batch(&1, &r, &EXPIRY);
        reg(&s).claim(&1);
        warp(&s, EXPIRY + 1);
        reg(&s).expire(&1);
    }

    // ---- Circuit breaker ----

    #[test]
    #[should_panic(expected = "contract is paused")]
    fn issue_blocked_when_paused() {
        let s = setup();
        config(&s).set_paused(&true);
        let (_, r) = one(&s, 10);
        reg(&s).issue_batch(&1, &r, &EXPIRY);
    }

    #[test]
    #[should_panic(expected = "contract is paused")]
    fn claim_blocked_when_paused() {
        let s = setup();
        let (_, r) = one(&s, 10);
        reg(&s).issue_batch(&1, &r, &EXPIRY);
        config(&s).set_paused(&true);
        reg(&s).claim(&1);
    }

    #[test]
    #[should_panic(expected = "contract is paused")]
    fn expire_blocked_when_paused() {
        let s = setup();
        let (_, r) = one(&s, 10);
        reg(&s).issue_batch(&1, &r, &EXPIRY);
        warp(&s, EXPIRY + 1);
        config(&s).set_paused(&true);
        reg(&s).expire(&1);
    }

    // ---- Init ----

    #[test]
    #[should_panic(expected = "already initialized")]
    fn double_init_rejected() {
        let s = setup();
        reg(&s).init(&s.config_id, &s.escrow_id, &s.token);
    }
}
