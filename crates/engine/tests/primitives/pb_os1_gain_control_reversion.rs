//! PB-OS1 roster sweep: enumerate every `all_cards()` def whose `Effect::GainControl`
//! uses an `EffectDuration::UntilEndOfTurn` or `EffectDuration::UntilYourNextTurn(_)`
//! duration -- the set of cards affected by the gain-control reversion fix.
//!
//! CR 611.2b/c, 613.7, 514.2. Enumerated from the compiled registry (`all_cards()`), NOT
//! from source-text grep, per the SR-34/36 lesson (a source grep can both over- and
//! under-count; the registry walk is the ground truth of what each def actually does).

use mtg_engine::cards::card_definition::{AbilityDefinition, Effect};
use mtg_engine::state::EffectDuration;

/// Recursively collect every `EffectDuration` attached to a `GainControl` effect
/// reachable from `effect`, walking every combinator wrapper that can nest an `Effect`.
fn collect_gain_control_durations(effect: &Effect, out: &mut Vec<EffectDuration>) {
    match effect {
        Effect::GainControl { duration, .. } => out.push(*duration),
        Effect::Conditional {
            if_true, if_false, ..
        } => {
            collect_gain_control_durations(if_true, out);
            collect_gain_control_durations(if_false, out);
        }
        Effect::Repeat { effect, .. } => collect_gain_control_durations(effect, out),
        Effect::ForEach { effect, .. } => collect_gain_control_durations(effect, out),
        Effect::Choose { choices, .. } => {
            for c in choices {
                collect_gain_control_durations(c, out);
            }
        }
        Effect::Sequence(effects) => {
            for e in effects {
                collect_gain_control_durations(e, out);
            }
        }
        Effect::MayPayOrElse { or_else, .. } => collect_gain_control_durations(or_else, out),
        Effect::MayPayThenEffect { then, .. } => collect_gain_control_durations(then, out),
        _ => {}
    }
}

/// Walk every `Effect`-bearing field of an `AbilityDefinition`, including modal
/// `ModeSelection.modes` on the variants that carry one.
fn collect_from_ability(ability: &AbilityDefinition, out: &mut Vec<EffectDuration>) {
    match ability {
        AbilityDefinition::Activated { effect, modes, .. } => {
            collect_gain_control_durations(effect, out);
            if let Some(modes) = modes {
                for m in &modes.modes {
                    collect_gain_control_durations(m, out);
                }
            }
        }
        AbilityDefinition::Triggered { effect, modes, .. } => {
            collect_gain_control_durations(effect, out);
            if let Some(modes) = modes {
                for m in &modes.modes {
                    collect_gain_control_durations(m, out);
                }
            }
        }
        AbilityDefinition::Spell { effect, modes, .. } => {
            collect_gain_control_durations(effect, out);
            if let Some(modes) = modes {
                for m in &modes.modes {
                    collect_gain_control_durations(m, out);
                }
            }
        }
        AbilityDefinition::LoyaltyAbility { effect, .. } => {
            collect_gain_control_durations(effect, out);
        }
        AbilityDefinition::SagaChapter { effect, .. } => {
            collect_gain_control_durations(effect, out);
        }
        AbilityDefinition::Aftermath { effect, .. } => {
            collect_gain_control_durations(effect, out);
        }
        AbilityDefinition::Fuse { effect, .. } => {
            collect_gain_control_durations(effect, out);
        }
        AbilityDefinition::Splice { effect, .. } => {
            collect_gain_control_durations(effect, out);
        }
        AbilityDefinition::Forecast { effect, .. } => {
            collect_gain_control_durations(effect, out);
        }
        _ => {}
    }
}

/// PB-OS1 roster sweep -- enumerates the full registry and asserts the exact set of
/// cards whose `GainControl` uses a duration this PB's fix reverts
/// (`UntilEndOfTurn` / `UntilYourNextTurn(_)`). If a future card authoring session adds
/// or removes a `GainControl` card with one of these durations, this test's assertion
/// must be updated to match -- it is a roster pin, not a tautology.
///
/// FINDING (deviation from the plan's preliminary roster): the registry walk shows only
/// **2** cards in scope -- `sarkhan_vol` and `zealous_conscripts` -- NOT 3. The plan's
/// listed third card, `karrthus_tyrant_of_jund`, models its "gain control ... for as long
/// as you control [this]" ability with `EffectDuration::Indefinite` (see
/// `karrthus_tyrant_of_jund.rs`'s own comment: "no stated duration"), not
/// `UntilEndOfTurn`/`UntilYourNextTurn` -- so it is untouched by either expiry pass this
/// PB fixes. Indefinite never reverts at all (a distinct, out-of-scope bug: karrthus's
/// "for as long as" duration arguably belongs on `WhileYouControlSource`, matching
/// Dragonlord Silumgar/Olivia Voldaren's modeling of the same oracle-text pattern -- but
/// that is a card-def authoring judgment call, not this PB's engine fix, and is flagged
/// as a follow-up rather than silently changed here).
#[test]
fn pb_os1_gain_control_reversion_roster() {
    let mut affected: Vec<String> = Vec::new();
    let mut other_durations: Vec<(String, EffectDuration)> = Vec::new();

    for card in mtg_engine::all_cards() {
        let mut durations: Vec<EffectDuration> = Vec::new();
        for ability in &card.abilities {
            collect_from_ability(ability, &mut durations);
        }
        for d in durations {
            match d {
                EffectDuration::UntilEndOfTurn | EffectDuration::UntilYourNextTurn(_) => {
                    affected.push(card.name.clone());
                }
                other => other_durations.push((card.name.clone(), other)),
            }
        }
    }
    affected.sort();
    affected.dedup();

    eprintln!(
        "PB-OS1 roster sweep: {} card(s) affected by the gain-control reversion fix: {:?}",
        affected.len(),
        affected
    );
    eprintln!(
        "PB-OS1 roster sweep: {} GainControl use(s) with an out-of-scope duration (not reverted by this fix): {:?}",
        other_durations.len(),
        other_durations
    );

    assert!(
        affected.contains(&"Sarkhan Vol".to_string())
            || affected
                .iter()
                .any(|n| n.eq_ignore_ascii_case("sarkhan vol")),
        "sarkhan_vol should be in the UntilEndOfTurn/UntilYourNextTurn GainControl roster"
    );
    assert!(
        affected
            .iter()
            .any(|n| n.eq_ignore_ascii_case("zealous conscripts")),
        "zealous_conscripts should be in the roster"
    );
    // karrthus_tyrant_of_jund is intentionally NOT asserted here -- see the FINDING in
    // this test's doc comment. It uses EffectDuration::Indefinite, out of scope for
    // this PB's fix.
    assert_eq!(
        affected.len(),
        2,
        "PB-OS1 in-scope roster should be exactly {{sarkhan_vol, zealous_conscripts}}; \
         if this count changes, a card was authored/re-authored with an \
         UntilEndOfTurn/UntilYourNextTurn GainControl -- update this pin"
    );
}
