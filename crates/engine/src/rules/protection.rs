//! Protection keyword enforcement (CR 702.16).
//!
//! Protection is a keyword ability that grants four protections (DEBT):
//! - **D**amage: all damage from sources with the specified quality is prevented (CR 702.16e)
//! - **E**nchanting: this permanent can't be enchanted or equipped by sources with the quality (CR 702.16c, 702.16d)
//! - **B**locking: this creature can't be blocked by creatures with the specified quality (CR 702.16f)
//! - **T**argeting: this permanent can't be targeted by spells or abilities from sources with the quality (CR 702.16b)
//!
//! Protection is a static ability that applies continuously while the permanent is on the battlefield.
use crate::state::error::GameStateError;
use crate::state::game_object::{Characteristics, ObjectId};
use crate::state::player::PlayerId;
use crate::state::types::{KeywordAbility, ProtectionQuality};
use crate::state::GameState;
use im::OrdSet;
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
/// - `FromSuperType(s)`: source has supertype `s`.
/// - `FromName(n)`: source's name equals `n`.
/// - `FromPlayer(p)`: source is controlled by player `p` (CR 702.16k); requires
///   `source_controller` to be supplied — when `None`, this quality never matches.
/// - `FromAll`: matches every source.
///
/// `source_controller` is the player who controls the source (the spell/ability,
/// damage source, blocker, or attachment). Pass `None` when it is unavailable;
/// only `FromPlayer` depends on it.
///
/// Performance (MR-M9.4-10): this is a linear scan of the keyword set. The scan
/// is intentionally left as-is — keyword sets are tiny (CR-bounded, in practice
/// well under 20 elements even on heavily-modified permanents), so a structured
/// lookup or range query over the `OrdSet` would add complexity without a
/// measurable win. Confirmed a non-bottleneck; no micro-optimization warranted.
pub fn has_protection_from_source(
    keywords: &OrdSet<KeywordAbility>,
    source_chars: &Characteristics,
    source_controller: Option<PlayerId>,
) -> bool {
    for kw in keywords {
        if let KeywordAbility::ProtectionFrom(quality) = kw {
            if matches_quality(quality, source_chars, source_controller) {
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
    source_controller: Option<PlayerId>,
) -> Result<(), GameStateError> {
    if has_protection_from_source(target_keywords, source_chars, source_controller) {
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
    source_controller: Option<PlayerId>,
) -> bool {
    has_protection_from_source(target_keywords, source_chars, source_controller)
}
/// CR 702.16f: Check whether a blocker is prevented from blocking an attacker by protection.
///
/// Returns `true` if the attacker has protection from a quality that the blocker matches.
/// Note: the attacker has the protection; the blocker is the potential source.
pub fn protection_prevents_blocking(
    attacker_keywords: &OrdSet<KeywordAbility>,
    blocker_chars: &Characteristics,
    blocker_controller: Option<PlayerId>,
) -> bool {
    has_protection_from_source(attacker_keywords, blocker_chars, blocker_controller)
}
/// CR 702.16c / 702.16d: Check if an aura or equipment is illegal on its target due to protection.
///
/// Returns `true` if the target permanent's keywords include protection from a quality
/// that the aura/equipment source matches. When true, the SBA removes the attachment.
pub fn attachment_is_illegal_due_to_protection(
    target_keywords: &OrdSet<KeywordAbility>,
    attachment_chars: &Characteristics,
    attachment_controller: Option<PlayerId>,
) -> bool {
    has_protection_from_source(target_keywords, attachment_chars, attachment_controller)
}
/// CR 702.16b/e: Check if a single `ProtectionQuality` matches a source's characteristics.
///
/// Convenience helper for player protection checks, where protection qualities are stored
/// as a `Vec<ProtectionQuality>` on `PlayerState` rather than inside `KeywordAbility` entries.
pub fn has_protection_from_source_quality(
    quality: &ProtectionQuality,
    source_chars: &Characteristics,
    source_controller: Option<PlayerId>,
) -> bool {
    matches_quality(quality, source_chars, source_controller)
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
    // The caster controls the targeting spell/ability, so it is the source's
    // controller for `FromPlayer` (CR 702.16k) protection checks.
    if let Some(sc) = source_chars {
        check_protection_targeting(target_keywords, sc, Some(caster))?;
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
    blocker_controller: Option<PlayerId>,
) -> bool {
    !protection_prevents_blocking(attacker_keywords, blocker_chars, blocker_controller)
}
// ---------------------------------------------------------------------------
// Internal helpers
// ---------------------------------------------------------------------------
/// Returns true if a `ProtectionQuality` matches the given source.
///
/// `source_controller` is the player controlling the source; it is only consulted
/// for `FromPlayer` (CR 702.16k). When the quality is `FromPlayer` and the
/// controller is unknown (`None`), the quality does not match.
fn matches_quality(
    quality: &ProtectionQuality,
    source_chars: &Characteristics,
    source_controller: Option<PlayerId>,
) -> bool {
    match quality {
        ProtectionQuality::FromColor(c) => source_chars.colors.contains(c),
        ProtectionQuality::FromCardType(ct) => source_chars.card_types.contains(ct),
        ProtectionQuality::FromSubType(st) => source_chars.subtypes.contains(st),
        ProtectionQuality::FromSuperType(st) => source_chars.supertypes.contains(st),
        ProtectionQuality::FromName(n) => &source_chars.name == n,
        ProtectionQuality::FromPlayer(p) => source_controller == Some(*p),
        ProtectionQuality::FromAll => true,
    }
}
