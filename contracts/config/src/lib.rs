#![no_std]

//! AidFlow Config / AccessControl Contract
//!
//! Manages admin multisig address, oracle address, and circuit breaker (pause/unpause).
//! All other contracts query this for authorization.
//!
//! @stable

use soroban_sdk::{contract, contractevent, contractimpl, contracttype, Address, Env};

/// Instance storage TTL: bump when within ~23 days of expiry, extend to ~30 days.
/// Ledgers are ~5s apart: 30 days ≈ 518_400 ledgers, 23 days ≈ 397_440 ledgers.
const TTL_THRESHOLD: u32 = 397_440;
const TTL_EXTEND_TO: u32 = 518_400;

/// Storage keys for contract data
#[contracttype]
#[derive(Clone)]
pub enum DataKey {
    Admin,
    Oracle,
    Paused,
}

/// Event emitted when admin address is updated
#[contractevent]
pub struct AdminUpdated {
    pub old_admin: Address,
    pub new_admin: Address,
}

/// Event emitted when oracle address is updated
#[contractevent]
pub struct OracleUpdated {
    pub old_oracle: Address,
    pub new_oracle: Address,
}

/// Event emitted when contracts are paused
#[contractevent]
pub struct Paused {}

/// Event emitted when contracts are unpaused
#[contractevent]
pub struct Unpaused {}

/// Extend the instance storage TTL so admin/oracle/paused never archive
/// on an infrequently-called config contract.
fn bump_instance_ttl(env: &Env) {
    env.storage()
        .instance()
        .extend_ttl(TTL_THRESHOLD, TTL_EXTEND_TO);
}

/// Config contract managing access control and circuit breaker
#[contract]
pub struct ConfigContract;

#[contractimpl]
impl ConfigContract {
    /// Initialize the contract with admin and oracle addresses.
    ///
    /// @stable
    /// Can only be called once.
    pub fn init(env: Env, admin: Address, oracle: Address) {
        // Check if already initialized
        if env.storage().instance().has(&DataKey::Admin) {
            panic!("already initialized");
        }

        // Require the admin to authorize initialization. This binds init to the
        // intended admin's signature, so it cannot be front-run if deploy and
        // init are submitted as separate transactions.
        admin.require_auth();

        // Store admin, oracle, and paused state together in instance storage.
        env.storage().instance().set(&DataKey::Admin, &admin);
        env.storage().instance().set(&DataKey::Oracle, &oracle);
        env.storage().instance().set(&DataKey::Paused, &false);

        // Extend instance TTL (covers all three keys as a unit).
        bump_instance_ttl(&env);
    }

    /// Update the admin address.
    ///
    /// @stable
    /// Requires authentication from current admin.
    pub fn set_admin(env: Env, new_admin: Address) {
        // Load current admin
        let admin: Address = env
            .storage()
            .instance()
            .get(&DataKey::Admin)
            .expect("not initialized");

        // Require auth from current admin
        admin.require_auth();

        // Update admin
        env.storage().instance().set(&DataKey::Admin, &new_admin);
        bump_instance_ttl(&env);

        // Emit event
        AdminUpdated {
            old_admin: admin,
            new_admin,
        }
        .publish(&env);
    }

    /// Update the oracle address.
    ///
    /// @stable
    /// Requires authentication from admin.
    pub fn set_oracle(env: Env, new_oracle: Address) {
        // Load admin
        let admin: Address = env
            .storage()
            .instance()
            .get(&DataKey::Admin)
            .expect("not initialized");

        // Require admin auth
        admin.require_auth();

        // Load old oracle for event
        let old_oracle: Address = env
            .storage()
            .instance()
            .get(&DataKey::Oracle)
            .expect("oracle not set");

        // Update oracle
        env.storage().instance().set(&DataKey::Oracle, &new_oracle);
        bump_instance_ttl(&env);

        // Emit event
        OracleUpdated {
            old_oracle,
            new_oracle,
        }
        .publish(&env);
    }

    /// Pause all contract operations (circuit breaker).
    ///
    /// @stable
    /// Requires authentication from admin.
    pub fn pause(env: Env) {
        // Load admin
        let admin: Address = env
            .storage()
            .instance()
            .get(&DataKey::Admin)
            .expect("not initialized");

        // Require admin auth
        admin.require_auth();

        // Set paused to true
        env.storage().instance().set(&DataKey::Paused, &true);
        bump_instance_ttl(&env);

        // Emit event
        Paused {}.publish(&env);
    }

    /// Resume contract operations.
    ///
    /// @stable
    /// Requires authentication from admin.
    pub fn unpause(env: Env) {
        // Load admin
        let admin: Address = env
            .storage()
            .instance()
            .get(&DataKey::Admin)
            .expect("not initialized");

        // Require admin auth
        admin.require_auth();

        // Set paused to false
        env.storage().instance().set(&DataKey::Paused, &false);
        bump_instance_ttl(&env);

        // Emit event
        Unpaused {}.publish(&env);
    }

    /// Check if contracts are paused.
    ///
    /// @stable
    pub fn is_paused(env: Env) -> bool {
        env.storage()
            .instance()
            .get(&DataKey::Paused)
            .unwrap_or(false)
    }

    /// Get current admin address.
    ///
    /// @stable
    pub fn get_admin(env: Env) -> Address {
        env.storage()
            .instance()
            .get(&DataKey::Admin)
            .expect("not initialized")
    }

    /// Get current oracle address.
    ///
    /// @stable
    pub fn get_oracle(env: Env) -> Address {
        env.storage()
            .instance()
            .get(&DataKey::Oracle)
            .expect("not initialized")
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use soroban_sdk::{testutils::Address as _, Env};

    #[test]
    fn test_init_sets_admin() {
        let env = Env::default();
        env.mock_all_auths();
        let contract_id = env.register(ConfigContract, ());
        let client = ConfigContractClient::new(&env, &contract_id);

        let admin = Address::generate(&env);
        let oracle = Address::generate(&env);

        client.init(&admin, &oracle);

        assert_eq!(client.get_admin(), admin);
        assert_eq!(client.get_oracle(), oracle);
        assert!(!client.is_paused());
    }

    #[test]
    #[should_panic(expected = "already initialized")]
    fn test_cannot_init_twice() {
        let env = Env::default();
        env.mock_all_auths();
        let contract_id = env.register(ConfigContract, ());
        let client = ConfigContractClient::new(&env, &contract_id);

        let admin = Address::generate(&env);
        let oracle = Address::generate(&env);

        client.init(&admin, &oracle);

        // Try to init again - should panic
        client.init(&admin, &oracle);
    }

    #[test]
    #[should_panic]
    fn test_init_requires_admin_auth() {
        // No mock_all_auths(): init must fail because admin.require_auth() is unsatisfied.
        let env = Env::default();
        let contract_id = env.register(ConfigContract, ());
        let client = ConfigContractClient::new(&env, &contract_id);

        let admin = Address::generate(&env);
        let oracle = Address::generate(&env);

        client.init(&admin, &oracle);
    }

    #[test]
    fn test_pause_requires_admin() {
        let env = Env::default();
        env.mock_all_auths();

        let contract_id = env.register(ConfigContract, ());
        let client = ConfigContractClient::new(&env, &contract_id);

        let admin = Address::generate(&env);
        let oracle = Address::generate(&env);

        client.init(&admin, &oracle);

        // Pause (with mock auth)
        client.pause();
        assert!(client.is_paused());

        // Unpause
        client.unpause();
        assert!(!client.is_paused());
    }

    #[test]
    fn test_set_admin() {
        let env = Env::default();
        env.mock_all_auths();

        let contract_id = env.register(ConfigContract, ());
        let client = ConfigContractClient::new(&env, &contract_id);

        let admin = Address::generate(&env);
        let oracle = Address::generate(&env);
        let new_admin = Address::generate(&env);

        client.init(&admin, &oracle);
        client.set_admin(&new_admin);

        assert_eq!(client.get_admin(), new_admin);
    }

    #[test]
    fn test_set_oracle() {
        let env = Env::default();
        env.mock_all_auths();

        let contract_id = env.register(ConfigContract, ());
        let client = ConfigContractClient::new(&env, &contract_id);

        let admin = Address::generate(&env);
        let oracle = Address::generate(&env);
        let new_oracle = Address::generate(&env);

        client.init(&admin, &oracle);
        client.set_oracle(&new_oracle);

        assert_eq!(client.get_oracle(), new_oracle);
    }
}
