//! # Withdraw Event Emission Module
//!
//! Provides security-hardened helpers for emitting events during the
//! `withdraw()` lifecycle. All event emission is centralised here so that
//! the main contract function stays readable and every event payload is
//! validated in one place.
//!
//! ## Events emitted by `withdraw()`
//!
//! | Topic 1    | Topic 2            | Data                   | Condition                          |
//! |------------|--------------------|------------------------|------------------------------------|
//! | `campaign` | `fee_transferred`  | `(Address, i128)`      | Platform fee is configured         |
//! | `campaign` | `nft_batch_minted` | `u32`                  | NFT contract set and ≥1 mint done  |
//! | `campaign` | `withdrawn`        | `(Address, i128, u32)` | Always on successful withdraw      |
//!
//! ## Security assumptions
//!
//! * All amounts are validated to be non-negative before emission.
//! * The `withdrawn` event is emitted **after** state mutation (status set to
//!   `Successful`, `TotalRaised` zeroed) so off-chain indexers observe a
//!   consistent final state.
//! * `emit_fee_transferred` is only called when `fee > 0` to prevent
//!   misleading zero-fee events.
//! * `emit_nft_batch_minted` is only called when `minted_count > 0`.
//! * `emit_withdrawn` always fires exactly once per successful `withdraw()`
//!   invocation — callers must not call it more than once.

#![allow(missing_docs)]

use soroban_sdk::{Address, Env};

// ── Fee transferred ──────────────────────────────────────────────────────────

/// Emit a `fee_transferred` event.
///
/// # Arguments
/// * `env`              – The contract environment.
/// * `platform_address` – Recipient of the platform fee.
/// * `fee`              – Fee amount transferred (must be > 0).
///
/// # Panics
/// * If `fee` is zero or negative — a zero-fee event is misleading and
///   indicates a logic error in the caller.
///
/// # Event payload
/// ```text
/// topics : ("campaign", "fee_transferred")
/// data   : (Address, i128)   // (platform_address, fee)
/// ```
pub fn emit_fee_transferred(env: &Env, platform_address: &Address, fee: i128) {
    assert!(fee > 0, "fee_transferred: fee must be positive");
    env.events()
        .publish(("campaign", "fee_transferred"), (platform_address, fee));
}

// ── NFT batch minted ─────────────────────────────────────────────────────────

/// Emit a single `nft_batch_minted` summary event.
///
/// Replaces the previous per-contributor `nft_minted` event pattern.
/// Emitting one summary event instead of N individual events caps gas
/// consumption when the contributor list is large.
///
/// # Arguments
/// * `env`          – The contract environment.
/// * `minted_count` – Number of NFTs minted in this batch (must be > 0).
///
/// # Panics
/// * If `minted_count` is zero — callers must guard against emitting an
///   empty-batch event.
///
/// # Event payload
/// ```text
/// topics : ("campaign", "nft_batch_minted")
/// data   : u32   // number of NFTs minted
/// ```
pub fn emit_nft_batch_minted(env: &Env, minted_count: u32) {
    assert!(
        minted_count > 0,
        "nft_batch_minted: minted_count must be positive"
    );
    env.events()
        .publish(("campaign", "nft_batch_minted"), minted_count);
}

// ── Withdrawn ────────────────────────────────────────────────────────────────

/// Emit the `withdrawn` event that signals a successful campaign withdrawal.
///
/// This is the canonical terminal event for a successful campaign. It carries
/// the creator address, the net payout (after any platform fee), and the
/// number of NFTs minted in this call.
///
/// # Arguments
/// * `env`             – The contract environment.
/// * `creator`         – The campaign creator who received the payout.
/// * `creator_payout`  – Net amount transferred to the creator (must be > 0).
/// * `nft_minted_count`– Number of NFTs minted (0 if no NFT contract set).
///
/// # Panics
/// * If `creator_payout` is zero or negative — a zero-payout withdrawal
///   indicates a logic error upstream.
///
/// # Event payload
/// ```text
/// topics : ("campaign", "withdrawn")
/// data   : (Address, i128, u32)   // (creator, creator_payout, nft_minted_count)
/// ```
pub fn emit_withdrawn(env: &Env, creator: &Address, creator_payout: i128, nft_minted_count: u32) {
    assert!(
        creator_payout > 0,
        "withdrawn: creator_payout must be positive"
    );
    env.events().publish(
        ("campaign", "withdrawn"),
        (creator, creator_payout, nft_minted_count),
    );
}
