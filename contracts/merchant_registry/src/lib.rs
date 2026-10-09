// SPDX-License-Identifier: Apache-2.0

#![no_std]

//! AidFlow MerchantRegistry Contract
//!
//! Admin-managed allowlist of merchants, and the redemption path through which
//! beneficiaries spend settlement tokens at an active merchant. Redemption moves
//! tokens straight from the beneficiary to the merchant's payout address; the
//! registry never custodies funds and keeps no balances.
//!
//! Design decisions (2026-10-09):
//!   1. Admin and the pause flag come from the Config contract (queried at call
//!      time), like Escrow and VoucherRegistry. `init` is bound to that admin.
//!   2. The Config circuit breaker blocks `register` and `redeem`, but NOT
//!      `deactivate`: removing a merchant only reduces risk, so it must stay
//!      available during an incident.
//!   3. `register` is an upsert. Re-registering a merchant updates its payout and
//!      category and reactivates it (there is no separate reactivate call).
//!
//! @stable

use aidflow_contract_types::{Merchant, CATEGORY_FOOD, CATEGORY_OTHER};
use soroban_sdk::{
    contract, contractevent, contractimpl, contracttype, token, Address, Env, Symbol, Vec,
};

/// Instance-storage TTL bounds (ledgers, ~5s each): bump at ~23 days, extend to ~30.
const TTL_THRESHOLD: u32 = 397_440;
const TTL_EXTEND_TO: u32 = 518_400;

/// Persistent-storage TTL bounds for merchant entries (~30/37 days).
const P_TTL_THRESHOLD: u32 = 518_400;
const P_TTL_EXTEND_TO: u32 = 639_360;

#[contracttype]
#[derive(Clone)]
pub enum DataKey {
    /// Config/AccessControl contract address (instance)
    Config,
    /// Settlement token address (instance)
    Token,
    /// Merchant record by address (persistent)
    Merchant(Address),
}

/// Event emitted when a merchant is registered
#[contractevent]
pub struct MerchantRegistered {
    pub merchant: Address,
    pub category: u32,
}

/// Event emitted when a voucher is redeemed
#[contractevent]
pub struct VoucherRedeemed {
    pub from: Address,
    pub merchant: Address,
    pub amount: i128,
}

/// Event emitted when a merchant is deactivated
#[contractevent]
pub struct MerchantDeactivated {
    pub merchant: Address,
}

#[contract]
pub struct MerchantRegistryContract;

#[contractimpl]
impl MerchantRegistryContract {
    /// Initialize with the Config contract and the settlement token. Requires
    /// the Config admin's authorization so it cannot be front-run. Can only be
    /// called once.
    ///
    /// @stable
    pub fn init(env: Env, config_contract: Address, token: Address) {
        if env.storage().instance().has(&DataKey::Config) {
            panic!("already initialized");
        }
        let admin: Address = env.invoke_contract(
            &config_contract,
            &Symbol::new(&env, "get_admin"),
            Vec::new(&env),
        );
        admin.require_auth();

        env.storage()
            .instance()
            .set(&DataKey::Config, &config_contract);
        env.storage().instance().set(&DataKey::Token, &token);
        bump_instance_ttl(&env);
    }

    /// Register (or update and reactivate) a merchant. Requires admin auth.
    /// `category` must be one of the `CATEGORY_*` values (1..=5).
    ///
    /// @stable
    pub fn register(env: Env, merchant: Address, payout: Address, category: u32) {
        require_admin(&env);
        ensure_not_paused(&env);
        if !(CATEGORY_FOOD..=CATEGORY_OTHER).contains(&category) {
            panic!("invalid category");
        }

        save_merchant(
            &env,
            &Merchant {
                address: merchant.clone(),
                payout,
                category,
                active: true,
            },
        );
        bump_instance_ttl(&env);

        MerchantRegistered { merchant, category }.publish(&env);
    }

    /// Deactivate a merchant. Requires admin auth. Allowed while paused.
    ///
    /// @stable
    pub fn deactivate(env: Env, merchant: Address) {
        require_admin(&env);

        let mut record = load_merchant(&env, &merchant).expect("merchant not found");
        if !record.active {
            panic!("merchant already inactive");
        }
        record.active = false;
        save_merchant(&env, &record);
        bump_instance_ttl(&env);

        MerchantDeactivated { merchant }.publish(&env);
    }

    /// Redeem `amount` from a beneficiary to an active merchant's payout
    /// address. Requires `from`'s auth.
    ///
    /// @stable
    pub fn redeem(env: Env, from: Address, merchant: Address, amount: i128) {
        from.require_auth();
        ensure_not_paused(&env);
        if amount <= 0 {
            panic!("amount must be positive");
        }

        let record = load_merchant(&env, &merchant).expect("merchant not registered");
        if !record.active {
            panic!("merchant is not active");
        }

        // No registry state changes, so the only interaction is the transfer.
        let token: Address = env.storage().instance().get(&DataKey::Token).unwrap();
        token::TokenClient::new(&env, &token).transfer(&from, &record.payout, &amount);
        bump_instance_ttl(&env);

        VoucherRedeemed {
            from,
            merchant,
            amount,
        }
        .publish(&env);
    }

    /// Whether the merchant is registered and active. False if unknown.
    ///
    /// @stable
    pub fn is_active(env: Env, merchant: Address) -> bool {
        load_merchant(&env, &merchant)
            .map(|m| m.active)
            .unwrap_or(false)
    }

    /// Get a merchant's record.
    ///
    /// @stable
    pub fn get_merchant(env: Env, merchant: Address) -> Merchant {
        load_merchant(&env, &merchant).expect("merchant not found")
    }
}

// ---- internal helpers ----

fn bump_instance_ttl(env: &Env) {
    env.storage()
        .instance()
        .extend_ttl(TTL_THRESHOLD, TTL_EXTEND_TO);
}

fn save_merchant(env: &Env, merchant: &Merchant) {
    let key = DataKey::Merchant(merchant.address.clone());
    env.storage().persistent().set(&key, merchant);
    env.storage()
        .persistent()
        .extend_ttl(&key, P_TTL_THRESHOLD, P_TTL_EXTEND_TO);
}

fn load_merchant(env: &Env, merchant: &Address) -> Option<Merchant> {
    env.storage()
        .persistent()
        .get(&DataKey::Merchant(merchant.clone()))
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
    use aidflow_contract_types::{CATEGORY_AGRICULTURE, CATEGORY_HEALTH};
    use soroban_sdk::testutils::{Address as _, MockAuth, MockAuthInvoke};
    use soroban_sdk::IntoVal;

    // Minimal Config stand-in exposing the two functions the registry calls.
    // Real Config+registry wiring belongs to the integration test layer.
    #[contracttype]
    #[derive(Clone)]
    enum MockKey {
        Admin,
        Paused,
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

    struct Setup {
        env: Env,
        registry_id: Address,
        config_id: Address,
        token: Address,
        admin: Address,
        beneficiary: Address,
        merchant: Address,
        payout: Address,
    }

    const BALANCE: i128 = 1_000;

    fn setup() -> Setup {
        let env = Env::default();
        env.mock_all_auths();

        let admin = Address::generate(&env);
        let config_id = env.register(MockConfig, ());
        MockConfigClient::new(&env, &config_id).init(&admin);

        let sac = env.register_stellar_asset_contract_v2(admin.clone());
        let token = sac.address();
        let beneficiary = Address::generate(&env);
        token::StellarAssetClient::new(&env, &token).mint(&beneficiary, &BALANCE);

        let registry_id = env.register(MerchantRegistryContract, ());
        MerchantRegistryContractClient::new(&env, &registry_id).init(&config_id, &token);

        Setup {
            registry_id,
            config_id,
            token,
            admin,
            beneficiary,
            merchant: Address::generate(&env),
            payout: Address::generate(&env),
            env,
        }
    }

    fn reg<'a>(s: &Setup) -> MerchantRegistryContractClient<'a> {
        MerchantRegistryContractClient::new(&s.env, &s.registry_id)
    }

    fn tokens<'a>(s: &Setup) -> token::TokenClient<'a> {
        token::TokenClient::new(&s.env, &s.token)
    }

    fn config<'a>(s: &Setup) -> MockConfigClient<'a> {
        MockConfigClient::new(&s.env, &s.config_id)
    }

    fn registered(s: &Setup) {
        reg(s).register(&s.merchant, &s.payout, &CATEGORY_FOOD);
    }

    // ---- Registration ----

    #[test]
    fn register_and_query() {
        let s = setup();
        assert!(!reg(&s).is_active(&s.merchant));
        reg(&s).register(&s.merchant, &s.payout, &CATEGORY_HEALTH);

        assert!(reg(&s).is_active(&s.merchant));
        let m = reg(&s).get_merchant(&s.merchant);
        assert_eq!(m.address, s.merchant);
        assert_eq!(m.payout, s.payout);
        assert_eq!(m.category, CATEGORY_HEALTH);
        assert!(m.active);
    }

    #[test]
    fn every_valid_category_accepted() {
        let s = setup();
        for c in CATEGORY_FOOD..=CATEGORY_OTHER {
            reg(&s).register(&Address::generate(&s.env), &s.payout, &c);
        }
    }

    #[test]
    #[should_panic(expected = "invalid category")]
    fn category_zero_rejected() {
        let s = setup();
        reg(&s).register(&s.merchant, &s.payout, &0);
    }

    #[test]
    #[should_panic(expected = "invalid category")]
    fn category_above_range_rejected() {
        let s = setup();
        reg(&s).register(&s.merchant, &s.payout, &6);
    }

    #[test]
    fn reregister_updates_and_reactivates() {
        let s = setup();
        registered(&s);
        reg(&s).deactivate(&s.merchant);
        assert!(!reg(&s).is_active(&s.merchant));

        let new_payout = Address::generate(&s.env);
        reg(&s).register(&s.merchant, &new_payout, &CATEGORY_AGRICULTURE);
        let m = reg(&s).get_merchant(&s.merchant);
        assert!(m.active);
        assert_eq!(m.payout, new_payout);
        assert_eq!(m.category, CATEGORY_AGRICULTURE);
    }

    #[test]
    #[should_panic]
    fn register_rejects_non_admin() {
        let s = setup();
        let imposter = Address::generate(&s.env);
        s.env.mock_auths(&[MockAuth {
            address: &imposter,
            invoke: &MockAuthInvoke {
                contract: &s.registry_id,
                fn_name: "register",
                args: (s.merchant.clone(), s.payout.clone(), CATEGORY_FOOD).into_val(&s.env),
                sub_invokes: &[],
            },
        }]);
        reg(&s).register(&s.merchant, &s.payout, &CATEGORY_FOOD);
    }

    #[test]
    fn register_succeeds_with_admin_auth_only() {
        let s = setup();
        s.env.mock_auths(&[MockAuth {
            address: &s.admin,
            invoke: &MockAuthInvoke {
                contract: &s.registry_id,
                fn_name: "register",
                args: (s.merchant.clone(), s.payout.clone(), CATEGORY_FOOD).into_val(&s.env),
                sub_invokes: &[],
            },
        }]);
        reg(&s).register(&s.merchant, &s.payout, &CATEGORY_FOOD);
        assert!(reg(&s).is_active(&s.merchant));
    }

    #[test]
    #[should_panic(expected = "contract is paused")]
    fn register_blocked_when_paused() {
        let s = setup();
        config(&s).set_paused(&true);
        registered(&s);
    }

    // ---- Deactivation ----

    #[test]
    fn deactivate_marks_inactive_but_keeps_record() {
        let s = setup();
        registered(&s);
        reg(&s).deactivate(&s.merchant);
        assert!(!reg(&s).is_active(&s.merchant));
        assert!(!reg(&s).get_merchant(&s.merchant).active);
    }

    #[test]
    fn deactivate_allowed_while_paused() {
        let s = setup();
        registered(&s);
        config(&s).set_paused(&true);
        reg(&s).deactivate(&s.merchant);
        assert!(!reg(&s).is_active(&s.merchant));
    }

    #[test]
    #[should_panic(expected = "merchant not found")]
    fn deactivate_unknown_rejected() {
        let s = setup();
        reg(&s).deactivate(&s.merchant);
    }

    #[test]
    #[should_panic(expected = "merchant already inactive")]
    fn double_deactivate_rejected() {
        let s = setup();
        registered(&s);
        reg(&s).deactivate(&s.merchant);
        reg(&s).deactivate(&s.merchant);
    }

    #[test]
    #[should_panic]
    fn deactivate_rejects_non_admin() {
        let s = setup();
        registered(&s);
        let imposter = Address::generate(&s.env);
        s.env.mock_auths(&[MockAuth {
            address: &imposter,
            invoke: &MockAuthInvoke {
                contract: &s.registry_id,
                fn_name: "deactivate",
                args: (s.merchant.clone(),).into_val(&s.env),
                sub_invokes: &[],
            },
        }]);
        reg(&s).deactivate(&s.merchant);
    }

    // ---- Redemption ----

    #[test]
    fn redeem_pays_merchant_payout() {
        let s = setup();
        registered(&s);
        reg(&s).redeem(&s.beneficiary, &s.merchant, &300);

        assert_eq!(tokens(&s).balance(&s.beneficiary), BALANCE - 300);
        assert_eq!(tokens(&s).balance(&s.payout), 300);
        // Settlement goes to the payout address, never the merchant identity,
        // and the registry itself holds nothing.
        assert_eq!(tokens(&s).balance(&s.merchant), 0);
        assert_eq!(tokens(&s).balance(&s.registry_id), 0);
    }

    #[test]
    fn redeem_full_balance() {
        let s = setup();
        registered(&s);
        reg(&s).redeem(&s.beneficiary, &s.merchant, &BALANCE);
        assert_eq!(tokens(&s).balance(&s.beneficiary), 0);
        assert_eq!(tokens(&s).balance(&s.payout), BALANCE);
    }

    #[test]
    #[should_panic(expected = "merchant not registered")]
    fn redeem_unregistered_merchant_rejected() {
        let s = setup();
        reg(&s).redeem(&s.beneficiary, &s.merchant, &100);
    }

    #[test]
    #[should_panic(expected = "merchant is not active")]
    fn redeem_deactivated_merchant_rejected() {
        let s = setup();
        registered(&s);
        reg(&s).deactivate(&s.merchant);
        reg(&s).redeem(&s.beneficiary, &s.merchant, &100);
    }

    #[test]
    fn redeem_works_again_after_reregister() {
        let s = setup();
        registered(&s);
        reg(&s).deactivate(&s.merchant);
        registered(&s);
        reg(&s).redeem(&s.beneficiary, &s.merchant, &100);
        assert_eq!(tokens(&s).balance(&s.payout), 100);
    }

    #[test]
    #[should_panic(expected = "amount must be positive")]
    fn redeem_zero_rejected() {
        let s = setup();
        registered(&s);
        reg(&s).redeem(&s.beneficiary, &s.merchant, &0);
    }

    #[test]
    #[should_panic(expected = "amount must be positive")]
    fn redeem_negative_rejected() {
        let s = setup();
        registered(&s);
        reg(&s).redeem(&s.beneficiary, &s.merchant, &-1);
    }

    #[test]
    #[should_panic]
    fn redeem_more_than_balance_rejected() {
        let s = setup();
        registered(&s);
        reg(&s).redeem(&s.beneficiary, &s.merchant, &(BALANCE + 1));
    }

    #[test]
    #[should_panic]
    fn redeem_rejects_wrong_signer() {
        let s = setup();
        registered(&s);
        let thief = Address::generate(&s.env);
        s.env.mock_auths(&[MockAuth {
            address: &thief,
            invoke: &MockAuthInvoke {
                contract: &s.registry_id,
                fn_name: "redeem",
                args: (s.beneficiary.clone(), s.merchant.clone(), 100i128).into_val(&s.env),
                sub_invokes: &[],
            },
        }]);
        reg(&s).redeem(&s.beneficiary, &s.merchant, &100);
    }

    #[test]
    #[should_panic(expected = "contract is paused")]
    fn redeem_blocked_when_paused() {
        let s = setup();
        registered(&s);
        config(&s).set_paused(&true);
        reg(&s).redeem(&s.beneficiary, &s.merchant, &100);
    }

    // ---- Init ----

    #[test]
    #[should_panic(expected = "already initialized")]
    fn double_init_rejected() {
        let s = setup();
        reg(&s).init(&s.config_id, &s.token);
    }

    /// Emitted events: payload checks and pinned topic names (indexers key on these).
    mod events {
        use super::*;
        use soroban_sdk::events::Event;
        use soroban_sdk::testutils::Events as _;
        use soroban_sdk::vec;

        fn emitted<E: Event>(s: &Setup, e: E) -> bool {
            s.env
                .events()
                .all()
                .events()
                .contains(&e.to_xdr(&s.env, &s.registry_id))
        }

        #[test]
        fn register_emits_merchant_registered() {
            let s = setup();
            registered(&s);
            assert!(emitted(
                &s,
                MerchantRegistered {
                    merchant: s.merchant.clone(),
                    category: CATEGORY_FOOD
                }
            ));
        }

        #[test]
        fn deactivate_emits_merchant_deactivated() {
            let s = setup();
            registered(&s);
            reg(&s).deactivate(&s.merchant);
            assert!(emitted(
                &s,
                MerchantDeactivated {
                    merchant: s.merchant.clone()
                }
            ));
        }

        #[test]
        fn redeem_emits_voucher_redeemed() {
            let s = setup();
            registered(&s);
            reg(&s).redeem(&s.beneficiary, &s.merchant, &300);
            assert!(emitted(
                &s,
                VoucherRedeemed {
                    from: s.beneficiary.clone(),
                    merchant: s.merchant.clone(),
                    amount: 300
                }
            ));
        }

        #[test]
        fn topic_names_are_pinned() {
            let env = Env::default();
            let a = Address::generate(&env);
            let topic = |name: &str| vec![&env, Symbol::new(&env, name).into_val(&env)];
            assert_eq!(
                MerchantRegistered {
                    merchant: a.clone(),
                    category: 1
                }
                .topics(&env),
                topic("merchant_registered")
            );
            assert_eq!(
                MerchantDeactivated {
                    merchant: a.clone()
                }
                .topics(&env),
                topic("merchant_deactivated")
            );
            assert_eq!(
                VoucherRedeemed {
                    from: a.clone(),
                    merchant: a,
                    amount: 1
                }
                .topics(&env),
                topic("voucher_redeemed")
            );
        }
    }
}
