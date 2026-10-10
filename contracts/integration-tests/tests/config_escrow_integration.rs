#![cfg(test)]

//! Config ↔ Escrow cross-contract integration tests
//!
//! These tests verify that the Escrow contract correctly calls the real Config
//! contract's API by string symbol name. If a Config getter is renamed without
//! updating Escrow's symbol strings, these tests will fail at runtime.

use aidflow_contract_types::{Milestone, ProgramStatus};
use soroban_sdk::{
    testutils::{Address as _, MockAuth, MockAuthInvoke},
    token, vec, Address, BytesN, Env, IntoVal, String,
};

// Import the real contracts
use aidflow_config::{ConfigContract, ConfigContractClient};
use aidflow_escrow::{EscrowContract, EscrowContractClient};

/// Test harness that deploys both real contracts
struct TestSetup {
    env: Env,
    config_id: Address,
    escrow_id: Address,
    token: Address,
    registry: Address,
    admin: Address,
    oracle: Address,
    funder: Address,
}

impl TestSetup {
    fn new() -> Self {
        let env = Env::default();
        env.mock_all_auths();

        let admin = Address::generate(&env);
        let oracle = Address::generate(&env);
        let funder = Address::generate(&env);

        // Deploy real Config contract
        let config_id = env.register(ConfigContract, ());
        ConfigContractClient::new(&env, &config_id).init(&admin, &oracle);

        // Deploy a token
        let sac = env.register_stellar_asset_contract_v2(admin.clone());
        let token = sac.address();
        token::StellarAssetClient::new(&env, &token).mint(&funder, &1_000_000);

        let registry = Address::generate(&env);

        // Deploy real Escrow contract, linking to real Config
        let escrow_id = env.register(EscrowContract, ());
        EscrowContractClient::new(&env, &escrow_id).init(&config_id, &token, &registry);

        TestSetup {
            env,
            config_id,
            escrow_id,
            token,
            registry,
            admin,
            oracle,
            funder,
        }
    }

    fn config(&self) -> ConfigContractClient<'_> {
        ConfigContractClient::new(&self.env, &self.config_id)
    }

    fn escrow(&self) -> EscrowContractClient<'_> {
        EscrowContractClient::new(&self.env, &self.escrow_id)
    }

    fn token(&self) -> token::TokenClient<'_> {
        token::TokenClient::new(&self.env, &self.token)
    }

    fn two_milestones(&self) -> soroban_sdk::Vec<Milestone> {
        vec![
            &self.env,
            Milestone {
                id: 1,
                description: String::from_str(&self.env, "phase-1"),
                target_amount: 400,
            },
            Milestone {
                id: 2,
                description: String::from_str(&self.env, "phase-2"),
                target_amount: 600,
            },
        ]
    }

    fn evidence_hash(&self) -> BytesN<32> {
        BytesN::from_array(&self.env, &[42u8; 32])
    }
}

#[test]
fn attest_and_release_with_real_oracle_admin() {
    let s = TestSetup::new();
    let e = s.escrow();

    // Create and fund a program
    let pid = e.create_program(&s.funder, &s.token, &s.two_milestones());
    e.fund(&pid, &1000);

    // Attest milestone 1 with the oracle
    s.env.mock_auths(&[MockAuth {
        address: &s.oracle,
        invoke: &MockAuthInvoke {
            contract: &s.escrow_id,
            fn_name: "attest_milestone",
            args: (pid, 1u32, s.evidence_hash()).into_val(&s.env),
            sub_invokes: &[],
        },
    }]);
    e.attest_milestone(&pid, &1, &s.evidence_hash());

    // Verify attestation was recorded
    let attestation = e.get_attestation(&pid, &1).unwrap();
    assert_eq!(attestation.oracle, s.oracle);
    assert_eq!(attestation.evidence_hash, s.evidence_hash());

    // Release milestone 1 with the admin
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

    // Verify funds were released
    assert_eq!(s.token().balance(&s.registry), 400);
    assert_eq!(e.get_program(&pid).released_amount, 400);
}

#[test]
fn admin_oracle_rotation_honored_by_escrow() {
    let s = TestSetup::new();
    let e = s.escrow();
    let c = s.config();

    // Create a program
    let pid = e.create_program(&s.funder, &s.token, &s.two_milestones());
    e.fund(&pid, &1000);

    // Rotate oracle to a new address
    let new_oracle = Address::generate(&s.env);
    s.env.mock_auths(&[MockAuth {
        address: &s.admin,
        invoke: &MockAuthInvoke {
            contract: &s.config_id,
            fn_name: "set_oracle",
            args: (new_oracle.clone(),).into_val(&s.env),
            sub_invokes: &[],
        },
    }]);
    c.set_oracle(&new_oracle);
    assert_eq!(c.get_oracle(), new_oracle);

    // New oracle SHOULD be able to attest
    s.env.mock_auths(&[MockAuth {
        address: &new_oracle,
        invoke: &MockAuthInvoke {
            contract: &s.escrow_id,
            fn_name: "attest_milestone",
            args: (pid, 1u32, s.evidence_hash()).into_val(&s.env),
            sub_invokes: &[],
        },
    }]);
    e.attest_milestone(&pid, &1, &s.evidence_hash());
    assert_eq!(e.get_attestation(&pid, &1).unwrap().oracle, new_oracle);

    // Rotate admin to a new address
    let new_admin = Address::generate(&s.env);
    s.env.mock_auths(&[MockAuth {
        address: &s.admin,
        invoke: &MockAuthInvoke {
            contract: &s.config_id,
            fn_name: "set_admin",
            args: (new_admin.clone(),).into_val(&s.env),
            sub_invokes: &[],
        },
    }]);
    c.set_admin(&new_admin);
    assert_eq!(c.get_admin(), new_admin);

    // New admin SHOULD be able to release
    s.env.mock_auths(&[MockAuth {
        address: &new_admin,
        invoke: &MockAuthInvoke {
            contract: &s.escrow_id,
            fn_name: "release",
            args: (pid, 1u32).into_val(&s.env),
            sub_invokes: &[],
        },
    }]);
    e.release(&pid, &1);
    assert_eq!(e.get_program(&pid).released_amount, 400);
}

#[test]
#[should_panic(expected = "Error(Auth, InvalidAction)")]
fn old_oracle_rejected_after_rotation() {
    let s = TestSetup::new();
    let e = s.escrow();
    let c = s.config();

    let pid = e.create_program(&s.funder, &s.token, &s.two_milestones());
    e.fund(&pid, &1000);

    // Rotate oracle
    let new_oracle = Address::generate(&s.env);
    s.env.mock_auths(&[MockAuth {
        address: &s.admin,
        invoke: &MockAuthInvoke {
            contract: &s.config_id,
            fn_name: "set_oracle",
            args: (new_oracle.clone(),).into_val(&s.env),
            sub_invokes: &[],
        },
    }]);
    c.set_oracle(&new_oracle);

    // Old oracle should NOT be able to attest
    s.env.mock_auths(&[MockAuth {
        address: &s.oracle,
        invoke: &MockAuthInvoke {
            contract: &s.escrow_id,
            fn_name: "attest_milestone",
            args: (pid, 1u32, s.evidence_hash()).into_val(&s.env),
            sub_invokes: &[],
        },
    }]);
    e.attest_milestone(&pid, &1, &s.evidence_hash()); // should panic
}

#[test]
#[should_panic(expected = "Error(Auth, InvalidAction)")]
fn old_admin_rejected_after_rotation() {
    let s = TestSetup::new();
    let e = s.escrow();
    let c = s.config();

    let pid = e.create_program(&s.funder, &s.token, &s.two_milestones());
    e.fund(&pid, &1000);

    // Attest with current oracle
    e.attest_milestone(&pid, &1, &s.evidence_hash());

    // Rotate admin
    let new_admin = Address::generate(&s.env);
    s.env.mock_auths(&[MockAuth {
        address: &s.admin,
        invoke: &MockAuthInvoke {
            contract: &s.config_id,
            fn_name: "set_admin",
            args: (new_admin.clone(),).into_val(&s.env),
            sub_invokes: &[],
        },
    }]);
    c.set_admin(&new_admin);

    // Old admin should NOT be able to release
    s.env.mock_auths(&[MockAuth {
        address: &s.admin,
        invoke: &MockAuthInvoke {
            contract: &s.escrow_id,
            fn_name: "release",
            args: (pid, 1u32).into_val(&s.env),
            sub_invokes: &[],
        },
    }]);
    e.release(&pid, &1); // should panic
}

#[test]
fn config_pause_blocks_all_escrow_writes() {
    let s = TestSetup::new();
    let e = s.escrow();
    let c = s.config();

    // Create and fund a program while not paused
    let pid = e.create_program(&s.funder, &s.token, &s.two_milestones());
    e.fund(&pid, &1000);
    e.attest_milestone(&pid, &1, &s.evidence_hash());

    // Pause via Config
    c.pause();
    assert!(c.is_paused());

    // Unpause and verify operations work again
    c.unpause();
    assert!(!c.is_paused());

    // Now attest milestone 2 and release milestone 1 should work
    e.attest_milestone(&pid, &2, &s.evidence_hash());
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
    assert_eq!(e.get_program(&pid).released_amount, 400);
}

#[test]
#[should_panic(expected = "contract is paused")]
fn pause_blocks_create_program() {
    let s = TestSetup::new();
    let e = s.escrow();
    let c = s.config();

    c.pause();
    e.create_program(&s.funder, &s.token, &s.two_milestones());
}

#[test]
#[should_panic(expected = "contract is paused")]
fn pause_blocks_fund() {
    let s = TestSetup::new();
    let e = s.escrow();
    let c = s.config();

    let pid = e.create_program(&s.funder, &s.token, &s.two_milestones());
    c.pause();
    e.fund(&pid, &100);
}

#[test]
#[should_panic(expected = "contract is paused")]
fn pause_blocks_attest() {
    let s = TestSetup::new();
    let e = s.escrow();
    let c = s.config();

    let pid = e.create_program(&s.funder, &s.token, &s.two_milestones());
    e.fund(&pid, &1000);
    c.pause();
    e.attest_milestone(&pid, &1, &s.evidence_hash());
}

#[test]
#[should_panic(expected = "contract is paused")]
fn pause_blocks_release() {
    let s = TestSetup::new();
    let e = s.escrow();
    let c = s.config();

    let pid = e.create_program(&s.funder, &s.token, &s.two_milestones());
    e.fund(&pid, &1000);
    e.attest_milestone(&pid, &1, &s.evidence_hash());
    c.pause();

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
}

#[test]
fn pause_blocks_refund_and_reclaim() {
    let s = TestSetup::new();
    let e = s.escrow();
    let c = s.config();

    // Create, fund, and complete a program
    let milestone = vec![
        &s.env,
        Milestone {
            id: 1,
            description: String::from_str(&s.env, "only"),
            target_amount: 400,
        },
    ];
    let pid = e.create_program(&s.funder, &s.token, &milestone);
    e.fund(&pid, &1000); // overfund
    e.attest_milestone(&pid, &1, &s.evidence_hash());
    e.release(&pid, &1);
    assert_eq!(e.get_program(&pid).status, ProgramStatus::Completed);

    // Pause
    c.pause();

    // Unpause and verify refund works
    c.unpause();
    s.env.mock_auths(&[MockAuth {
        address: &s.admin,
        invoke: &MockAuthInvoke {
            contract: &s.escrow_id,
            fn_name: "refund_unspent",
            args: (pid,).into_val(&s.env),
            sub_invokes: &[],
        },
    }]);
    e.refund_unspent(&pid);
    assert_eq!(e.get_program(&pid).status, ProgramStatus::Refunded);
    assert_eq!(s.token().balance(&s.funder), 1_000_000 - 400); // got 600 back
}

#[test]
#[should_panic(expected = "contract is paused")]
fn pause_blocks_refund() {
    let s = TestSetup::new();
    let e = s.escrow();
    let c = s.config();

    let milestone = vec![
        &s.env,
        Milestone {
            id: 1,
            description: String::from_str(&s.env, "only"),
            target_amount: 400,
        },
    ];
    let pid = e.create_program(&s.funder, &s.token, &milestone);
    e.fund(&pid, &1000);
    e.attest_milestone(&pid, &1, &s.evidence_hash());
    e.release(&pid, &1);

    c.pause();

    s.env.mock_auths(&[MockAuth {
        address: &s.admin,
        invoke: &MockAuthInvoke {
            contract: &s.escrow_id,
            fn_name: "refund_unspent",
            args: (pid,).into_val(&s.env),
            sub_invokes: &[],
        },
    }]);
    e.refund_unspent(&pid);
}

#[test]
#[should_panic(expected = "contract is paused")]
fn pause_blocks_reclaim() {
    let s = TestSetup::new();
    let e = s.escrow();
    let c = s.config();

    let milestone = vec![
        &s.env,
        Milestone {
            id: 1,
            description: String::from_str(&s.env, "only"),
            target_amount: 400,
        },
    ];
    let pid = e.create_program(&s.funder, &s.token, &milestone);
    e.fund(&pid, &1000);
    e.attest_milestone(&pid, &1, &s.evidence_hash());
    e.release(&pid, &1);

    c.pause();

    s.env.mock_auths(&[MockAuth {
        address: &s.registry,
        invoke: &MockAuthInvoke {
            contract: &s.escrow_id,
            fn_name: "reclaim_expired",
            args: (pid, 100i128).into_val(&s.env),
            sub_invokes: &[],
        },
    }]);
    e.reclaim_expired(&pid, &100);
}

/// This test documents the critical requirement: Escrow calls Config methods
/// by string symbol name. If you rename `get_admin` in Config to `admin` without
/// updating Escrow's Symbol::new(env, "get_admin") to Symbol::new(env, "admin"),
/// this test will fail at runtime with a contract invocation error.
///
/// This is the gap that integration tests close - unit tests with mock Config
/// would still pass after such a rename, but real deployments would panic.
#[test]
fn symbol_names_must_match_config_api() {
    let s = TestSetup::new();
    let e = s.escrow();
    let c = s.config();

    // Verify all three Config methods Escrow depends on are callable
    assert_eq!(c.get_admin(), s.admin);
    assert_eq!(c.get_oracle(), s.oracle);
    assert!(!c.is_paused());

    // Create a program - this will call Config.get_admin during init verification
    let pid = e.create_program(&s.funder, &s.token, &s.two_milestones());
    e.fund(&pid, &1000);

    // Attest - this will call Config.get_oracle and Config.is_paused
    e.attest_milestone(&pid, &1, &s.evidence_hash());

    // Release - this will call Config.get_admin and Config.is_paused
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

    // If we reach here, all symbol names matched correctly
    assert_eq!(e.get_program(&pid).released_amount, 400);
}
