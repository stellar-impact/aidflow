#![no_std]

//! AidFlow Contract Types
//!
//! Shared type definitions used across all AidFlow Soroban contracts.
//! All types are marked with stability annotations.

use soroban_sdk::{contracttype, String};

/// Represents a single milestone in an aid program.
///
/// @stable
#[contracttype]
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct Milestone {
    /// Unique identifier for this milestone within the program
    pub id: u32,
    /// Human-readable description of the milestone
    pub description: String,
    /// Amount of tokens to be released when this milestone is completed
    pub target_amount: i128,
}

/// Status of an aid program.
///
/// @stable
#[contracttype]
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum ProgramStatus {
    /// Program is active and accepting funds/releases
    Active,
    /// All milestones completed, program finished
    Completed,
    /// Program cancelled, unspent funds refunded
    Refunded,
}

/// Status of a voucher issued to a beneficiary.
///
/// @stable
#[contracttype]
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum VoucherStatus {
    /// Voucher issued but not yet claimed
    Unclaimed,
    /// Voucher claimed by beneficiary
    Claimed,
    /// Voucher expired, funds returned to escrow
    Expired,
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_milestone_creation() {
        let env = soroban_sdk::Env::default();
        let milestone = Milestone {
            id: 1,
            description: String::from_str(&env, "Build well"),
            target_amount: 1000,
        };
        assert_eq!(milestone.id, 1);
        assert_eq!(milestone.target_amount, 1000);
    }

    #[test]
    fn test_program_status_equality() {
        assert_eq!(ProgramStatus::Active, ProgramStatus::Active);
        assert_ne!(ProgramStatus::Active, ProgramStatus::Completed);
    }

    #[test]
    fn test_voucher_status_equality() {
        assert_eq!(VoucherStatus::Unclaimed, VoucherStatus::Unclaimed);
        assert_ne!(VoucherStatus::Unclaimed, VoucherStatus::Claimed);
    }
}
