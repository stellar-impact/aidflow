#![no_std]

//! AidFlow Contract Types
//!
//! Shared type definitions used across all AidFlow Soroban contracts.
//! All types are marked with stability annotations.

use soroban_sdk::{contracttype, Address, String};

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

/// A voucher issued to a beneficiary, redeemable for `amount` of the
/// settlement token until `expiry`.
///
/// @stable
#[contracttype]
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct Voucher {
    pub id: u64,
    pub program_id: u64,
    pub recipient: Address,
    pub amount: i128,
    pub status: VoucherStatus,
    /// Ledger timestamp (seconds) after which the voucher can no longer be claimed
    pub expiry: u64,
    pub claimed_at: Option<u64>,
}

/// Merchant categories (the `Merchant.category` values).
pub const CATEGORY_FOOD: u32 = 1;
pub const CATEGORY_HEALTH: u32 = 2;
pub const CATEGORY_EDUCATION: u32 = 3;
pub const CATEGORY_AGRICULTURE: u32 = 4;
pub const CATEGORY_OTHER: u32 = 5;

/// A merchant approved to accept voucher redemptions.
///
/// @stable
#[contracttype]
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct Merchant {
    /// Identity the merchant is registered (and looked up) under
    pub address: Address,
    /// Address that receives settlement when beneficiaries redeem
    pub payout: Address,
    /// One of the `CATEGORY_*` values
    pub category: u32,
    pub active: bool,
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
