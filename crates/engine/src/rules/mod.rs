//! Rules engine: turn structure, priority, stack, SBAs, layers, combat.
//!
//! Each submodule corresponds to a section of the MTG Comprehensive Rules.

pub mod abilities;
pub mod casting;
pub mod combat;
pub mod command;
pub mod commander;
pub mod engine;
pub mod events;
pub mod lands;
pub mod layers;
pub mod mana;
pub mod priority;
pub mod replacement;
pub mod resolution;
pub mod sba;
pub mod turn_actions;
pub mod turn_structure;

pub use command::Command;
pub use engine::process_command;
pub use events::{GameEvent, LossReason};
pub use layers::calculate_characteristics;

// ── Shared targeting helpers ──────────────────────────────────────────────────

/// CR 702.11a / CR 702.18a: Validate that a target is not protected by
/// hexproof or shroud.
///
/// Hexproof prevents targeting by opponents; shroud prevents targeting by anyone.
/// Used by both `casting::validate_targets` and `abilities::handle_activate_ability`.
pub(crate) fn validate_target_protection(
    keywords: &im::OrdSet<crate::state::types::KeywordAbility>,
    controller: crate::state::player::PlayerId,
    caster: crate::state::player::PlayerId,
) -> Result<(), crate::state::error::GameStateError> {
    use crate::state::types::KeywordAbility;
    if keywords.contains(&KeywordAbility::Shroud) {
        return Err(crate::state::error::GameStateError::InvalidTarget(
            "object has shroud and cannot be targeted".into(),
        ));
    }
    if keywords.contains(&KeywordAbility::Hexproof) && controller != caster {
        return Err(crate::state::error::GameStateError::InvalidTarget(
            "object has hexproof and cannot be targeted by opponents".into(),
        ));
    }
    Ok(())
}
