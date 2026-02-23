//! Protection keyword enforcement (CR 702.16).
//!
//! Protection is a keyword ability that grants four protections (DEBT):
//! - **D**amage: all damage from sources with the specified quality is prevented (CR 702.16e)
//! - **E**nchanting: this permanent can't be enchanted or equipped by sources with the quality (CR 702.16c, 702.16d)
//! - **B**locking: this creature can't be blocked by creatures with the specified quality (CR 702.16f)
//! - **T**argeting: this permanent can't be targeted by spells or abilities from sources with the quality (CR 702.16b)
//!
//! Protection is a static ability that applies continuously while the permanent is on the battlefield.

use im::OrdSet;

use crate::state::error::GameStateError;
use crate::state::game_object::{Characteristics, ObjectId};
use crate::state::player::PlayerId;
use crate::state::types::{KeywordAbility, ProtectionQuality};
use crate::state::GameState;

/// CR 702.16a: Check if a set of keywords contains protection from a given source.
///
/// Returns `true` if any `ProtectionFrom(q)` keyword in `keywords` matches the
/// source's characteristics. The source is described by its characteristics and
/// its card's colors (for color-based protection checks).
///
/// Matching rules:
/// - `FromColor(c)`: source has color `c` in its colors set.
/// - `FromCardType(t)`: source has card type `t`.
/// - `FromSubType(s)`: source has subtype `s`.
/// - `FromAll`: matches every source.
pub fn has_protection_from_source(
    keywords: &OrdSet<KeywordAbility>,
    source_chars: &Characteristics,
) -> bool {
    for kw in keywords {
        if let KeywordAbility::ProtectionFrom(quality) = kw {
            if matches_quality(quality, source_chars) {
                return true;
            }
        }
    }
    false
}

/// CR 702.16b: Validate that a target is not protected from the source that is targeting it.
///
/// Called from `rules/mod.rs:validate_target_protection` during both spell casting
/// (`casting.rs:validate_targets`) and ability activation
/// (`abilities.rs:handle_activate_ability`).
///
/// Returns `Err` if the target has protection from the source.
/// `source_chars` is the characteristics of the object whose spell or ability is
/// targeting the object with `target_keywords`.
pub fn check_protection_targeting(
    target_keywords: &OrdSet<KeywordAbility>,
    source_chars: &Characteristics,
) -> Result<(), GameStateError> {
    if has_protection_from_source(target_keywords, source_chars) {
        return Err(GameStateError::InvalidTarget(
            "object has protection from the source and cannot be targeted".into(),
        ));
    }
    Ok(())
}

/// CR 702.16e: Check whether a source's damage to a target is prevented by protection.
///
/// Returns `true` if the target's protection blocks all damage from the source.
/// Used in the damage application path in `effects/mod.rs`.
pub fn protection_prevents_damage(
    target_keywords: &OrdSet<KeywordAbility>,
    source_chars: &Characteristics,
) -> bool {
    has_protection_from_source(target_keywords, source_chars)
}

/// CR 702.16f: Check whether a blocker is prevented from blocking an attacker by protection.
///
/// Returns `true` if the attacker has protection from a quality that the blocker matches.
/// Note: the attacker has the protection; the blocker is the potential source.
pub fn protection_prevents_blocking(
    attacker_keywords: &OrdSet<KeywordAbility>,
    blocker_chars: &Characteristics,
) -> bool {
    has_protection_from_source(attacker_keywords, blocker_chars)
}

/// CR 702.16c / 702.16d: Check if an aura or equipment is illegal on its target due to protection.
///
/// Returns `true` if the target permanent's keywords include protection from a quality
/// that the aura/equipment source matches. When true, the SBA removes the attachment.
pub fn attachment_is_illegal_due_to_protection(
    target_keywords: &OrdSet<KeywordAbility>,
    attachment_chars: &Characteristics,
) -> bool {
    has_protection_from_source(target_keywords, attachment_chars)
}

/// Retrieve the computed characteristics of the source object, if available.
///
/// Convenience wrapper for callers that need source characteristics for protection
/// checks and have only an `ObjectId`.
pub fn source_characteristics(state: &GameState, source_id: ObjectId) -> Option<Characteristics> {
    crate::rules::layers::calculate_characteristics(state, source_id).or_else(|| {
        state
            .objects
            .get(&source_id)
            .map(|o| o.characteristics.clone())
    })
}

/// Check if a source with characteristics `source_chars` is controller `caster` targeting
/// an object with `target_keywords` controlled by `target_controller`. Returns `Ok` if the
/// targeting is allowed, or `Err` if protection blocks it.
///
/// This is the combined hexproof + shroud + protection check for targeting validation.
/// See `rules/mod.rs:validate_target_protection`.
pub fn check_full_targeting_protection(
    target_keywords: &OrdSet<KeywordAbility>,
    target_controller: PlayerId,
    caster: PlayerId,
    source_chars: Option<&Characteristics>,
) -> Result<(), GameStateError> {
    // Hexproof: can't be targeted by opponents (CR 702.11a).
    if target_keywords.contains(&KeywordAbility::Hexproof) && target_controller != caster {
        return Err(GameStateError::InvalidTarget(
            "object has hexproof and cannot be targeted by opponents".into(),
        ));
    }
    // Shroud: can't be targeted by anyone (CR 702.18a).
    if target_keywords.contains(&KeywordAbility::Shroud) {
        return Err(GameStateError::InvalidTarget(
            "object has shroud and cannot be targeted".into(),
        ));
    }
    // Protection: can't be targeted by sources it is protected from (CR 702.16b).
    if let Some(sc) = source_chars {
        check_protection_targeting(target_keywords, sc)?;
    }
    Ok(())
}

/// CR 702.16f: Check whether a potential blocker is prevented from blocking by protection.
///
/// Returns `true` if the attacker has protection from a quality that the blocker matches
/// (i.e., the blocker cannot block the attacker). The blocker is the "source" being checked
/// against the attacker's protection.
pub fn can_block(
    attacker_keywords: &OrdSet<KeywordAbility>,
    blocker_chars: &Characteristics,
) -> bool {
    !protection_prevents_blocking(attacker_keywords, blocker_chars)
}

// ---------------------------------------------------------------------------
// Internal helpers
// ---------------------------------------------------------------------------

/// Returns true if a `ProtectionQuality` matches the given source characteristics.
fn matches_quality(quality: &ProtectionQuality, source_chars: &Characteristics) -> bool {
    match quality {
        ProtectionQuality::FromColor(c) => source_chars.colors.contains(c),
        ProtectionQuality::FromCardType(ct) => source_chars.card_types.contains(ct),
        ProtectionQuality::FromSubType(st) => source_chars.subtypes.contains(st),
        ProtectionQuality::FromAll => true,
    }
}
