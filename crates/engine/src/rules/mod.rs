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
pub mod protection;
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

/// CR 702.11a / CR 702.18a / CR 702.16b: Validate that a target is not protected by
/// hexproof, shroud, or protection from the source.
///
/// - Hexproof prevents targeting by opponents (CR 702.11a).
/// - Shroud prevents targeting by anyone (CR 702.18a).
/// - Protection from X prevents targeting by sources with quality X (CR 702.16b).
///
/// `source_chars` is the characteristics of the spell or ability doing the targeting.
/// Pass `None` when the source characteristics are unavailable (protection check is skipped).
///
/// Used by both `casting::validate_targets` and `abilities::handle_activate_ability`.
pub(crate) fn validate_target_protection(
    keywords: &im::OrdSet<crate::state::types::KeywordAbility>,
    controller: crate::state::player::PlayerId,
    caster: crate::state::player::PlayerId,
    source_chars: Option<&crate::state::game_object::Characteristics>,
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
    // CR 702.16b: protection from the source blocks targeting.
    if let Some(sc) = source_chars {
        protection::check_protection_targeting(keywords, sc)?;
    }
    Ok(())
}
