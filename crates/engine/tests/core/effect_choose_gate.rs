//! SR-33 gates: the `Effect::Choose` / `MayPayOrElse` / `AddManaChoice` stubs cannot ship
//! as `Complete`, and every `Complete` land produces exactly the colours it prints.
//!
//! ## Why this file exists
//!
//! These three are M7-era stubs that never grew their interactive half. Each takes a field
//! that looks like a choice and ignores it:
//!
//! | variant | what it actually does |
//! |---|---|
//! | `Effect::Choose` | always executes `choices.first()`; `prompt` and the rest are inert |
//! | `Effect::MayPayOrElse` | discards `cost`/`payer`, always takes `or_else` — the payment is never offered |
//! | `Effect::AddManaChoice` | adds one **colorless** mana; has no field for which colours are legal, and ignores `count` |
//!
//! Nothing observed this: all three compile, all three execute, and a def built on any of
//! them passes every other gate while silently doing one fixed thing.
//!
//! That is how 88 dual/tri lands shipped `Complete` while producing only their first
//! colour. `{T}: Add {G} or {U}` was authored as `Choose{[AddMana(G), AddMana(U)]}`,
//! which (a) always added `{G}` and (b) was not recognised by `try_as_tap_mana_ability`,
//! so the land registered **zero** mana abilities and used the stack — a CR 605.1a/605.3b
//! violation on Tropical Island, every shockland, and the check/fast/temple/guildgate
//! cycles. See `memory/decisions.md` (2026-07-17) for why those were rewritten to one
//! activated ability per colour rather than the stub being implemented.
//!
//! The gates below close the hole in both directions: no new def may reach for a stub and
//! call itself finished, and no `Complete` land may drop a printed colour or invent an
//! unprinted one.
//!
//! **Delete the stub gates when interactive choice lands** (a general `MakeChoice` Command
//! plus a colour list on `AddManaChoice`); at that point the variants stop being stubs and
//! the constraint is wrong. The colour gate should outlive them.
//!
//! ## Why a serde walk rather than a match on `Effect`
//!
//! The stub can hide anywhere in a def's effect tree — nested under `Sequence`,
//! `ForEach`, `Conditional`, a `Replacement`, or a token's granted ability. An
//! exhaustive recursive matcher over `Effect` would have to enumerate every recursion
//! site and would silently under-report the day someone adds a new one (the exact
//! failure mode SR-5 and SR-8 were filed for). Walking `serde_json::to_value(&def)`
//! reaches every field of the whole `CardDefinition` by construction, so a new nesting
//! site is covered the moment it exists. `Effect` is externally tagged, so a variant
//! is an object key.
//!
//! **The one way this walk can go blind**: a `#[serde(skip)]` on an effect-bearing field
//! would remove that field from the tree, and these gates would keep passing while seeing
//! less. There is no such attribute in `card_definition.rs` today (checked), and SR-8's
//! `PROTOCOL_SCHEMA_FINGERPRINT` would fail on the wire-shape change — but it would not
//! say *this*, so it is written down here.

use std::collections::{BTreeSet, HashMap};
use std::sync::Arc;

use mtg_engine::cards::{Completeness, TypeLine};
use mtg_engine::{
    all_cards, enrich_spec_from_def, process_command, AbilityDefinition, CardDefinition,
    CardRegistry, Command, Cost, Effect, EffectAmount, GameState, GameStateBuilder, ManaColor,
    ObjectId, ObjectSpec, PlayerId, PlayerTarget, Step, ZoneId,
};

// ── serde-tree helpers ────────────────────────────────────────────────────────

/// True if `key` appears anywhere in the value tree as an object key.
fn contains_key(v: &serde_json::Value, key: &str) -> bool {
    match v {
        serde_json::Value::Object(map) => map
            .iter()
            .any(|(k, child)| k == key || contains_key(child, key)),
        serde_json::Value::Array(items) => items.iter().any(|i| contains_key(i, key)),
        _ => false,
    }
}

fn def_uses(def: &CardDefinition, variant: &str) -> bool {
    let json = serde_json::to_value(def).expect("CardDefinition serializes");
    contains_key(&json, variant)
}

// ── The stub gates ────────────────────────────────────────────────────────────

/// A `Complete` def may not contain `Effect::Choose`: the choice is not implemented,
/// so the card always does its first option and nothing says so.
#[test]
fn no_complete_def_uses_the_choose_stub() {
    let offenders: Vec<String> = all_cards()
        .into_iter()
        .filter(|d| d.completeness.is_complete() && def_uses(d, "Choose"))
        .map(|d| d.name)
        .collect();

    assert!(
        offenders.is_empty(),
        "`Effect::Choose` always executes `choices.first()` (effects/mod.rs) — a def using \
         it does one fixed thing regardless of what the card prints, so it is not Complete. \
         Either model the choice explicitly (for a mana ability: one activated ability per \
         colour, and the player chooses via `TapForMana {{ ability_index }}` — see \
         `tainted_field.rs`), or mark the def `Completeness::known_wrong(\"...\")`. \
         Offenders: {offenders:?}"
    );
}

/// A `Complete` def may not contain `Effect::MayPayOrElse`: it always declines, so the
/// "may" is not a choice and the `or_else` branch always fires.
///
/// Note `Effect::MayPayThenEffect` is deliberately **not** gated here — it honours its
/// `payer` and pays when able, which is a documented deterministic-but-legal game choice
/// (CR 118.12). It is a weaker claim than these two stubs, not the same defect.
#[test]
fn no_complete_def_uses_the_may_pay_or_else_stub() {
    let offenders: Vec<String> = all_cards()
        .into_iter()
        .filter(|d| d.completeness.is_complete() && def_uses(d, "MayPayOrElse"))
        .map(|d| d.name)
        .collect();

    assert!(
        offenders.is_empty(),
        "`Effect::MayPayOrElse` discards `cost`/`payer` and unconditionally executes \
         `or_else` (effects/mod.rs) — the payment is never offered and never collected, so \
         the optional clause always resolves one way. Mark such a def \
         `Completeness::known_wrong(\"...\")`. Offenders: {offenders:?}"
    );
}

/// A `Complete` def may not contain `Effect::AddManaChoice`: it adds one **colorless**
/// mana regardless of what the card prints.
///
/// This is the third member of the same stub family, and the least obvious. The variant's
/// name says "choice" and its fields are `{ player, count }` — it has nowhere to record
/// *which* colours are legal, and its only execution site shares an arm with
/// `AddManaAnyColor` whose body is `mana_pool.add(ManaColor::Colorless, 1)`
/// ("For now, add colorless" — `effects/mod.rs`). So a card printing `{T}, Pay 1 life:
/// Add {U} or {R}` adds `{C}`: not one of its colours, and not a colour it prints at all.
/// `count` is ignored too, so "add three mana" adds one.
///
/// **`AddManaAnyColor` and its two siblings share this defect and are now gated the same
/// way — see [`no_complete_def_uses_an_any_color_mana_stub`] (SR-37 / SF-11).** The doc
/// comment here previously claimed an *asymmetry* justified blocking only `AddManaChoice`:
/// that `AddManaAnyColor` "escapes into a real `ManaAbility` with `any_color: true` and so
/// never reaches [the colorless] arm." That claim was **false**. `handle_tap_for_mana`
/// step 8 does exactly what the stack arm does — `mana_pool.add(ManaColor::Colorless, ...)`
/// — so escaping into a `ManaAbility` changes nothing; both paths add `{C}`. Per CR
/// 106.1a/106.1b colorless is a mana *type*, not a colour, so `{C}` is outside the legal
/// option set for "any color" on either path. SR-37 deleted the asymmetry and extended the
/// gate; the deferral note (Birds of Paradise, Command Tower, `sr34-roster-reconciliation.md`
/// §1) is discharged — those defs are demoted. **Delete all of it when a colour channel for
/// `any_color` mana lands.**
#[test]
fn no_complete_def_uses_the_add_mana_choice_stub() {
    let offenders: Vec<String> = all_cards()
        .into_iter()
        .filter(|d| d.completeness.is_complete() && def_uses(d, "AddManaChoice"))
        .map(|d| d.name)
        .collect();

    assert!(
        offenders.is_empty(),
        "`Effect::AddManaChoice` adds one colorless mana and ignores `count` \
         (effects/mod.rs, the arm it shares with AddManaAnyColor) — it has no field for \
         which colours are legal, so it cannot express \"Add {{U}} or {{R}}\". Author one \
         activated ability per colour instead, or mark the def \
         `Completeness::known_wrong(\"...\")`. Offenders: {offenders:?}"
    );
}

/// PB-EF12 (CR 106.1a/106.1b/605.3b) narrowed this gate: `Effect::AddManaAnyColor` is no
/// longer a blanket stub. A `any_color: true` `ManaAbility` (a `Cost::Tap`-family ability
/// `try_as_tap_mana_ability` lowers) now resolves to a real chosen colour via
/// `Command::TapForMana.chosen_color`, validated + rejected-on-Colorless by
/// `handle_tap_for_mana` — that is **served**, not a stub, and a `Complete` def using it is
/// fine. What remains genuinely stubbed:
///
/// - `Effect::AddManaAnyColorRestricted` / `Effect::AddManaOfAnyColorAmount` — never lowered
///   by `try_as_tap_mana_ability` (no restriction-list / dynamic-amount channel on
///   `ManaAbility`), so both variants still resolve through `execute_effect` and still add
///   `ManaColor::Colorless` unconditionally. **Always flagged**, regardless of registration.
/// - A plain `Effect::AddManaAnyColor` that is **not served** — the ability sits on a
///   triggered/ETB effect, a spell effect, or an activation cost `mana_ability_cost_components`
///   refuses (sacrifice-a-DIFFERENT-permanent via `Cost::Sacrifice(filter)`,
///   `Cost::RemoveCounter`, …) — none of these lower into a `ManaAbility`, so they still
///   resolve through `execute_effect` and still add `ManaColor::Colorless`. Detected via
///   [`registers_any_color_mana_ability`]: **flagged iff the def registers zero** `any_color`
///   mana abilities (so every occurrence of `AddManaAnyColor` in the def is a stub, not a
///   served tap ability).
///
/// **Known hole (documented, not closed)**: a def with BOTH a served tap `any_color` ability
/// AND a separate unserved `AddManaAnyColor` (e.g. on a triggered ability) would pass this
/// gate, because `registers_any_color_mana_ability` only asks "does the def register at least
/// one", not "does every occurrence resolve through a served path". No such def exists in the
/// corpus today (checked: every def using `AddManaAnyColor` on more than one clause either
/// serves all of them or none — see the restore/held-back rosters in
/// `memory/primitives/pb-review-EF12.md`); `no_complete_def_has_a_mixed_served_and_unserved_any_color_stub`
/// asserts the corpus currently has no such case.
#[test]
fn no_complete_def_uses_an_any_color_mana_stub() {
    let defs = defs_map();
    let offenders: Vec<String> = all_cards()
        .into_iter()
        .filter(|d| d.completeness.is_complete())
        .filter(|d| {
            let always_flag =
                def_uses(d, "AddManaAnyColorRestricted") || def_uses(d, "AddManaOfAnyColorAmount");
            if always_flag {
                return true;
            }
            def_uses(d, "AddManaAnyColor") && !registers_any_color_mana_ability(d, &defs)
        })
        .map(|d| d.name)
        .collect();

    assert!(
        offenders.is_empty(),
        "`Effect::AddManaAnyColorRestricted` / `Effect::AddManaOfAnyColorAmount` add one \
         **colorless** mana unconditionally (effects/mod.rs) — colorless is a mana type, not a \
         colour (CR 106.1a/106.1b), and neither is lowered into a real `ManaAbility` by \
         `try_as_tap_mana_ability`, so PB-EF12's colour channel never reaches them. A plain \
         `Effect::AddManaAnyColor` that registers no `any_color` mana ability (a triggered/ETB \
         effect, or an activation cost `try_as_tap_mana_ability` cannot lower) is the same \
         defect — still unserved, still adds Colorless. Author one activated ability per \
         colour, wire the ability so it lowers into a `ManaAbility` (Cost::Tap family), or mark \
         the def `Completeness::known_wrong(\"...\")`. Offenders: {offenders:?}"
    );
}

/// Counts how many times `key` appears anywhere in the value tree as an object key
/// (unlike [`contains_key`], which only asks whether it appears at all). Used to detect
/// a def with more than one `AddManaAnyColor` occurrence.
fn count_key_occurrences(v: &serde_json::Value, key: &str) -> usize {
    match v {
        serde_json::Value::Object(map) => map
            .iter()
            .map(|(k, child)| (if k == key { 1 } else { 0 }) + count_key_occurrences(child, key))
            .sum(),
        serde_json::Value::Array(items) => {
            items.iter().map(|i| count_key_occurrences(i, key)).sum()
        }
        _ => 0,
    }
}

/// Documents and pins the known hole in [`no_complete_def_uses_an_any_color_mana_stub`]: a
/// `Complete` def with a served `any_color` tap ability AND a second, unserved
/// `Effect::AddManaAnyColor` occurrence elsewhere (e.g. a triggered ability) would pass that
/// gate, because it only checks "does the def register at least one served any_color
/// ability", not "does every AddManaAnyColor occurrence resolve through a served path".
///
/// Asserts the corpus has no such def today: a served (`registers_any_color_mana_ability`)
/// `Complete` def must have **exactly one** `AddManaAnyColor` occurrence in its whole effect
/// tree — if it had two, the second could be an unserved stub hiding behind the first's
/// coverage. If this test starts failing, the gate above needs a per-occurrence check
/// (count servable abilities vs. count occurrences), not a "does it have ≥1" check.
#[test]
fn no_complete_def_has_a_mixed_served_and_unserved_any_color_stub() {
    let defs = defs_map();
    let offenders: Vec<String> = all_cards()
        .into_iter()
        .filter(|d| {
            d.completeness.is_complete()
                && registers_any_color_mana_ability(d, &defs)
                && count_key_occurrences(
                    &serde_json::to_value(d).expect("serializes"),
                    "AddManaAnyColor",
                ) > 1
        })
        .map(|d| d.name)
        .collect();

    assert!(
        offenders.is_empty(),
        "these Complete defs mix a served any_color tap ability with a second \
         AddManaAnyColor occurrence elsewhere — the coarse gate above would miss an unserved \
         second occurrence. Needs the stronger per-occurrence check: {offenders:?}"
    );
}

/// Both gates must be able to fail. A gate over a corpus that happens to be clean proves
/// nothing about the gate — if `def_uses` silently stopped finding anything (a serde
/// rename, a walk that misses a nesting site), the tests above would still pass and the
/// stub could walk back in.
///
/// This pins both directions, including the nested case the serde walk exists for.
#[test]
fn stub_gates_are_not_vacuous() {
    let bare = |effect: Effect| CardDefinition {
        card_id: mtg_engine::state::CardId("synthetic-probe".to_string()),
        name: "Synthetic Probe".to_string(),
        oracle_text: "Choose one — ...".to_string(),
        types: TypeLine {
            supertypes: Default::default(),
            card_types: [mtg_engine::state::CardType::Instant]
                .iter()
                .copied()
                .collect(),
            subtypes: Default::default(),
        },
        abilities: vec![mtg_engine::cards::AbilityDefinition::Activated {
            cost: mtg_engine::cards::Cost::Tap,
            effect,
            timing_restriction: None,
            targets: vec![],
            activation_condition: None,
            activation_zone: None,
            once_per_turn: false,
            modes: None,
        }],
        ..Default::default()
    };

    let add_g = Effect::AddMana {
        player: PlayerTarget::Controller,
        mana: mtg_engine::cards::helpers::mana_pool(0, 0, 0, 0, 1, 0),
    };
    let choose = Effect::Choose {
        prompt: "probe".to_string(),
        choices: vec![add_g.clone(), add_g.clone()],
    };

    // Positive: a bare stub is seen.
    assert!(
        def_uses(&bare(choose.clone()), "Choose"),
        "gate must detect a top-level Effect::Choose"
    );
    // Positive: a stub nested two levels deep is seen — this is the whole reason the
    // walk is a serde tree traversal and not a match on the ability's top-level effect.
    assert!(
        def_uses(
            &bare(Effect::Sequence(vec![Effect::Sequence(vec![choose])])),
            "Choose"
        ),
        "gate must detect an Effect::Choose nested inside Sequence(Sequence(..))"
    );
    // Negative: a def with no stub is not flagged, so the gate is not simply always-true.
    assert!(
        !def_uses(&bare(add_g.clone()), "Choose"),
        "gate must not flag a def with no Effect::Choose"
    );

    // SR-37 / SF-11: the any-color stub gate matches three more Effect variants by exact
    // serde key. Pin each — a `#[serde(rename)]` or a variant rename would silently make
    // `no_complete_def_uses_an_any_color_mana_stub` vacuous on a clean corpus, the exact
    // "hole in the checker" this task is named for.
    use mtg_engine::cards::card_definition::{EffectAmount, ManaRestriction};
    let any_color_probes = [
        (
            Effect::AddManaAnyColor {
                player: PlayerTarget::Controller,
            },
            "AddManaAnyColor",
        ),
        (
            Effect::AddManaAnyColorRestricted {
                player: PlayerTarget::Controller,
                restriction: ManaRestriction::CreatureSpellsOnly,
            },
            "AddManaAnyColorRestricted",
        ),
        (
            Effect::AddManaOfAnyColorAmount {
                player: PlayerTarget::Controller,
                amount: EffectAmount::Fixed(1),
            },
            "AddManaOfAnyColorAmount",
        ),
    ];
    for (effect, key) in any_color_probes {
        assert!(
            def_uses(&bare(effect), key),
            "the any-color gate must detect Effect::{key}"
        );
    }
    // Negative: a plain single-colour AddMana matches none of the three any-color keys
    // (exact-key matching — `AddManaAnyColor` must not be found in a bare `AddMana` def).
    let plain = bare(add_g);
    for key in [
        "AddManaAnyColor",
        "AddManaAnyColorRestricted",
        "AddManaOfAnyColorAmount",
    ] {
        assert!(
            !def_uses(&plain, key),
            "a plain AddMana def must not be flagged for {key}"
        );
    }
}

/// PB-EF12 non-vacuity: `no_complete_def_uses_an_any_color_mana_stub` was narrowed from
/// "flag every AddManaAnyColor occurrence" to "flag only unserved ones". Prove both
/// directions of that narrowing, not just that the key-detection primitive still works.
#[test]
fn served_vs_unserved_any_color_gate_logic_is_not_vacuous() {
    let any_color_effect = Effect::AddManaAnyColor {
        player: PlayerTarget::Controller,
    };

    // Served: a Cost::Tap activated ability lowers into a real any_color ManaAbility
    // (try_as_tap_mana_ability). Must NOT be flagged.
    let served = CardDefinition {
        name: "EF12 Served Any Color Probe".to_string(),
        oracle_text: "{T}: Add one mana of any color.".to_string(),
        types: TypeLine {
            supertypes: Default::default(),
            card_types: [mtg_engine::state::CardType::Artifact]
                .iter()
                .copied()
                .collect(),
            subtypes: Default::default(),
        },
        abilities: vec![AbilityDefinition::Activated {
            cost: Cost::Tap,
            effect: any_color_effect.clone(),
            timing_restriction: None,
            targets: vec![],
            activation_condition: None,
            activation_zone: None,
            once_per_turn: false,
            modes: None,
        }],
        ..Default::default()
    };
    let mut served_defs = defs_map();
    served_defs.insert(served.name.clone(), served.clone());
    assert!(
        registers_any_color_mana_ability(&served, &served_defs),
        "a Cost::Tap AddManaAnyColor ability must register as a served any_color ManaAbility"
    );
    assert!(
        !def_uses(&served, "AddManaAnyColor")
            || registers_any_color_mana_ability(&served, &served_defs),
        "a served plain AddManaAnyColor must NOT be flagged by the refined gate"
    );

    // Unserved: a Triggered ability's effect never lowers into a ManaAbility at all —
    // it stays a stack-resolution stub that still adds Colorless. Must BE flagged.
    let unserved = CardDefinition {
        name: "EF12 Unserved Any Color Probe".to_string(),
        oracle_text: "Whenever this enters, add one mana of any color.".to_string(),
        types: TypeLine {
            supertypes: Default::default(),
            card_types: [mtg_engine::state::CardType::Creature]
                .iter()
                .copied()
                .collect(),
            subtypes: Default::default(),
        },
        abilities: vec![AbilityDefinition::Triggered {
            once_per_turn: false,
            trigger_condition:
                mtg_engine::cards::card_definition::TriggerCondition::WhenEntersBattlefield,
            effect: any_color_effect,
            intervening_if: None,
            targets: vec![],
            modes: None,
            trigger_zone: None,
        }],
        ..Default::default()
    };
    let mut unserved_defs = defs_map();
    unserved_defs.insert(unserved.name.clone(), unserved.clone());
    assert!(
        !registers_any_color_mana_ability(&unserved, &unserved_defs),
        "a triggered-ability AddManaAnyColor must NOT register as a served any_color ManaAbility"
    );
    assert!(
        def_uses(&unserved, "AddManaAnyColor")
            && !registers_any_color_mana_ability(&unserved, &unserved_defs),
        "an unserved plain AddManaAnyColor (triggered effect) must still BE flagged by the \
         refined gate — the narrowing must not have gone blind to the real stub case"
    );

    // Real-corpus positive: Birds of Paradise is Complete, served, and must not appear in
    // the live gate's offender list.
    let defs = defs_map();
    let bop = defs
        .get("Birds of Paradise")
        .expect("Birds of Paradise has a def");
    assert!(
        bop.completeness.is_complete(),
        "Birds of Paradise should be Complete post-PB-EF12"
    );
    assert!(
        registers_any_color_mana_ability(bop, &defs),
        "Birds of Paradise must register a served any_color mana ability"
    );
}

// ── Lands produce every colour they print ─────────────────────────────────────

fn symbol_to_color(c: char) -> Option<ManaColor> {
    Some(match c {
        'W' => ManaColor::White,
        'U' => ManaColor::Blue,
        'B' => ManaColor::Black,
        'R' => ManaColor::Red,
        'G' => ManaColor::Green,
        'C' => ManaColor::Colorless,
        _ => return None,
    })
}

/// Colours printed in this card's own tap-for-mana `... Add ...` clauses.
///
/// **SR-34 update.** Originally scoped to a bare `{T}: Add ...` clause only — before
/// SR-34, `enrich_spec_from_def` lowered nothing else into a `ManaAbility`, so widening
/// this parser to a composite cost would have asserted a defect (the missing lowering)
/// this gate was not filed to catch. SR-34 widened the lowering itself to any cost
/// payable through `Command::TapForMana` (mana + tap + pay-life + sacrifice-self —
/// `mana_ability_cost_components` in `testing/replay_harness.rs`), so this parser now
/// widens in lockstep: a clause's *cost* only needs to **include** `{T}` somewhere (CR
/// 106.12: "tap for mana" means the activation cost includes `{T}`, not that it *is*
/// `{T}`), which covers `{1}, {T}: Add {R}{W}` (Signets), `{T}, Pay 1 life: Add {B} or
/// {G}` (horizon lands), and `{W/B}, {T}: Add ...` (filter lands) alongside the original
/// bare-`{T}` case.
///
/// Three exclusions remain, all load-bearing — without them this reports cards that are
/// not the SR-33/SR-34 defect:
///
/// 1. **The clause must be this card's own.** `Creatures you control have "{T}: Add {G}"`
///    (Citanul Hierophants) grants the ability to *other* objects; the granting card has
///    no mana ability of its own and is correct as written.
/// 2. **A dynamic-amount clause has its *amount* excluded but its *colour* compared
///    (SR-38 / SG-3).** This parser reads *colours*, never *amounts* — `{T}: Add {G} for
///    each creature you control` (`Effect::AddManaScaled`) prints a colour AND a count. The
///    count this parser cannot verify; the colour it can. Earlier iterations dropped the
///    whole clause from *both* sides (`printed_tap_mana_colors` skipped a "for each" / "equal
///    to" clause; `registered_colors` filtered on `scaled_amount.is_none()`) — sound only
///    while symmetric, and it discarded a checkable colour to stay that way. SR-38 narrows
///    the exclusion to amounts only: the printed side now keeps the colour parsed before the
///    dynamic tail, and the registered side reads scaled abilities' `produces.keys()` (the
///    `{colour: 1}` marker `try_as_tap_mana_ability` sets), so both carry the colour and the
///    amount lives only in `scaled_amount`, which neither inspects. The gain: a scaled
///    ability registering an outright WRONG colour is now caught (it passed vacuously before).
///    The amount remains verified by *activation*, elsewhere: SF-8 (SR-36, `scutemob-92`) gave
///    `handle_tap_for_mana` a `ManaAbility::scaled_amount` resolution step, so Gaea's Cradle
///    taps for the real creature count, not exactly 1 regardless of board state. Every
///    bare-`Cost::Tap` scaled source (Gaea's Cradle, Elvish Archdruid, Priest of Titania,
///    Marwyn the Nurturer, Circle of Dreams Druid, Howlsquad Heavy —
///    `tests/casting/mana_filter.rs::test_add_mana_scaled_orphan_fix_all_cards`) and all three
///    composite-cost scaled sources SF-8 unblocked — Cabal Coffers, Cabal Stronghold and Crypt
///    of Agadeem (`cabal_coffers_is_a_real_mana_ability`, `cabal_stronghold_counts_only_basic_swamps`,
///    `crypt_of_agadeem_counts_only_black_creature_cards_in_graveyard`, all in
///    `primitives/primitive_sr36_scaled_mana_and_life_costs.rs`) — assert the count with a
///    board decoy its filter must exclude, so a filter degraded to a raw count fails rather
///    than passing on a coincidentally-equal number. NB: an `Effect::AddManaOfAnyColorAmount`
///    clause ("Add an amount of ... equal to ...") parses NO colour on the printed side (the
///    symbol walk breaks on the leading "an"), so it contributes nothing here regardless; its
///    colour blindness is exclusion 3's concern (SF-12), not this one's.
/// 3. **"Add one mana of any color" — now handled (SR-37 / SF-12), was invisible on both
///    sides.** Previously: on the *printed* side this parser required a `{` after the cost
///    (the `strip_prefix('{')` walk below), and "Add one mana of any color." has no brace,
///    so `printed` was empty and the caller's `if printed.is_empty() { continue; }` skipped
///    the card; on the *registered* side `registered_colors` read only `ma.produces.keys()`,
///    empty for an `any_color: true` `ManaAbility` (probed: Mana Confluence reports
///    `produces={} any_color=true`). So Mana Confluence / Birds of Paradise / Command Tower —
///    CR 106.1a/106.1b: printing "any color" but producing `{C}` (SF-11) — passed vacuously.
///    SR-37 closes both halves, and they had to land together (the finding's warning): the
///    printed side now seeds all five colours on the "one mana of any color" phrasing (see
///    the `clause_line` check below), and `registered_colors` now reports the `{Colorless}`
///    an `any_color` ability truly adds. An any-color land that produces `{C}` therefore
///    fails with `missing {W,U,B,R,G}, invented {C}` — the honest report — rather than being
///    skipped. In practice every such def is now `known_wrong` (SF-11 demotions), so this
///    gate, scoped to `Complete`, does not fire on them; the machinery is a **backstop** and
///    is pinned non-vacuously by `land_color_gate_is_not_blind_to_any_color_lands`.
///    NB: `registered_colors` maps `any_color` to the true production `{Colorless}`, not to
///    "all five" — the finding's suggested "claims all five" would make the gate *pass* an
///    any-color land (both sides five); reporting the real `{C}` is what makes it fail.
fn printed_tap_mana_colors(oracle: &str) -> BTreeSet<ManaColor> {
    let mut out = BTreeSet::new();
    for (idx, _) in oracle.match_indices(": Add ") {
        // The clause boundary: start of oracle text, a newline, or an opening paren
        // (reminder text) — whichever is closest before `idx`.
        let before_all = &oracle[..idx];
        let boundary = before_all.rfind(['\n', '(']).map(|p| p + 1).unwrap_or(0);
        let cost = &before_all[boundary..];
        // SR-34: the cost must include {T} somewhere — a cost with no {T} at all
        // (Druids' Repository's "Remove a charge counter: Add...", Ashnod's Altar's
        // "Sacrifice a creature: Add...") is not a tap-mana source (CR 106.12) and is
        // out of this gate's scope regardless of the SR-34 lowering widening.
        if !cost.contains("{T}") {
            continue;
        }
        // SR-34: "Sacrifice a/an <noun>" (sacrificing a DIFFERENT permanent, not this
        // one) needs a caller-supplied ObjectId `Command::TapForMana` has no payload
        // for, so `mana_ability_cost_components` refuses it and the ability stays a
        // stack-using activated ability regardless of the {T} in its cost (Phyrexian
        // Tower: "{T}, Sacrifice a creature: Add {B}{B}."). Distinct from "Sacrifice
        // this" (self-sacrifice, e.g. Treasure tokens), which IS lowerable via
        // `ManaAbility::sacrifice_self` and must not be excluded here.
        if cost.contains("Sacrifice a ") || cost.contains("Sacrifice an ") {
            continue;
        }
        // An odd number of preceding quotes means we are inside a granted ability.
        if oracle[..idx].matches('"').count() % 2 == 1 {
            continue;
        }
        // Walk the clause symbol by symbol, stopping at the first token that is not a
        // mana symbol or a separator — that token ends the "Add" clause. Anything the
        // parser does not understand ends the clause rather than being skipped over, so
        // it can only ever under-report, never invent a colour.
        let mut rest = &oracle[idx + ": Add ".len()..];
        let mut clause_colors = BTreeSet::new();
        // SR-37 / SF-12: "one mana of any color" prints all five colours but writes no
        // brace, so the symbol walk below finds nothing and the clause used to vanish.
        // Detect the phrasing (bounded to this clause's own line) and seed all five.
        // Paired with `registered_colors` mapping an `any_color: true` ManaAbility to
        // {Colorless}: an any-color land that produces {C} then fails this gate with
        // `missing {W,U,B,R,G}, invented {C}` — the honest report (CR 106.1a/106.1b).
        let clause_line = oracle[idx + ": Add ".len()..]
            .split('\n')
            .next()
            .unwrap_or("");
        if clause_line.contains("mana of any color")
            || clause_line.contains("mana of any of the colors")
            || clause_line.contains("mana of any one color")
        {
            clause_colors.extend([
                ManaColor::White,
                ManaColor::Blue,
                ManaColor::Black,
                ManaColor::Red,
                ManaColor::Green,
            ]);
        }
        loop {
            rest = rest.trim_start_matches([' ', ',']);
            if let Some(r) = rest.strip_prefix("or ") {
                rest = r;
                continue;
            }
            let Some(r) = rest.strip_prefix('{') else {
                break;
            };
            let Some((sym, r)) = r.split_at_checked(1) else {
                break;
            };
            let Some(r) = r.strip_prefix('}') else { break };
            let Some(color) = sym.chars().next().and_then(symbol_to_color) else {
                break;
            };
            clause_colors.insert(color);
            rest = r;
        }
        // Exclusion 2 (narrowed to amounts only — SR-38 / SG-3). A dynamic-amount clause
        // ("... {B} for each basic Swamp" / "Add {C} equal to ...") prints a colour AND a
        // count. This parser reads *colours*, never amounts, so the colour parsed before the
        // dynamic tail (`clause_colors`) is verifiable and the amount is not — dropping the
        // whole clause discarded a checkable colour along with the uncheckable count.
        // `registered_colors` now reads scaled abilities' `produces.keys()` symmetrically, so
        // both sides carry the colour and the amount lives only in `scaled_amount`, which
        // neither side inspects. This lets the gate catch a scaled ability that registers an
        // outright WRONG colour (previously passed vacuously) while still leaving amounts to
        // the activation tests named in the doc comment. NB: a leading "an amount of ..."
        // (e.g. `Effect::AddManaOfAnyColorAmount`) parses NO colour — the symbol walk breaks
        // on "an", so `clause_colors` is empty and such clauses still contribute nothing here.
        out.extend(clause_colors);
    }
    out
}

fn registered_colors(
    def: &CardDefinition,
    defs: &HashMap<String, CardDefinition>,
) -> BTreeSet<ManaColor> {
    let spec = enrich_spec_from_def(
        ObjectSpec::card(PlayerId(1), &def.name).in_zone(ZoneId::Battlefield),
        defs,
    );
    spec.mana_abilities
        .iter()
        // Exclusion 2, applied symmetrically to the registered side (SR-38 / SG-3). A scaled
        // ability (`scaled_amount.is_some()`) carries its colour in `produces` — `{colour: 1}`,
        // the marker `try_as_tap_mana_ability` sets — and its dynamic amount in `scaled_amount`.
        // `printed_tap_mana_colors` now keeps the colour of a "for each" / "equal to" clause
        // (dropping only the amount, which it never tracked), so this side must keep it too:
        // reading `produces.keys()` for scaled abilities exactly matches. Both sides therefore
        // compare Cabal Stronghold's `{B}` (previously dropped) while ignoring "for each basic
        // Swamp" — the amount is verified by activation in
        // `primitives/primitive_sr36_scaled_mana_and_life_costs.rs`. Before SR-38 this line
        // was `.filter(|ma| ma.scaled_amount.is_none())`, which dropped the colour too and so
        // would have passed a scaled ability registering an outright WRONG colour.
        .flat_map(|ma| {
            // PB-EF12 (CR 111.10a/605.3b): an `any_color: true` ManaAbility carries an
            // empty `produces` (the marker is only meaningful for fixed-colour
            // abilities) but now really can add any of the five colours — the choice
            // rides `Command::TapForMana.chosen_color`, validated + resolved by
            // `handle_tap_for_mana` (Colorless is rejected, CR 106.1b). Report the true
            // option set (all five), not the pre-PB-EF12 always-Colorless production, so
            // a plain "any color" land's printed and registered sets now match instead of
            // permanently mismatching by construction.
            let mut cs: Vec<ManaColor> = ma.produces.keys().copied().collect();
            if ma.any_color {
                cs.extend([
                    ManaColor::White,
                    ManaColor::Blue,
                    ManaColor::Black,
                    ManaColor::Red,
                    ManaColor::Green,
                ]);
            }
            cs
        })
        .collect()
}

/// True if `def` registers at least one `any_color: true` mana ability (i.e. the
/// "any color" clause is served through `Command::TapForMana.chosen_color` — PB-EF12 —
/// rather than being an unserved stack-resolution stub). Used by
/// [`no_complete_def_uses_an_any_color_mana_stub`] to distinguish a served plain
/// `Effect::AddManaAnyColor` (a `Cost::Tap`-family ability, lowered by
/// `try_as_tap_mana_ability`) from an unserved one (a triggered/ETB effect, a
/// non-tap-cost sacrifice-a-different-permanent cost, or `Cost::RemoveCounter` — none of
/// these lower into a `ManaAbility`, so they still resolve through `execute_effect` and
/// still add `ManaColor::Colorless`, per `memory/primitives/pb-plan-EF12.md`'s "NOT the
/// ManaAbility path" list).
fn registers_any_color_mana_ability(
    def: &CardDefinition,
    defs: &HashMap<String, CardDefinition>,
) -> bool {
    let spec = enrich_spec_from_def(
        ObjectSpec::card(PlayerId(1), &def.name).in_zone(ZoneId::Battlefield),
        defs,
    );
    spec.mana_abilities.iter().any(|ma| ma.any_color)
}

fn defs_map() -> HashMap<String, CardDefinition> {
    all_cards()
        .iter()
        .map(|d| (d.name.clone(), d.clone()))
        .collect()
}

/// CR 605.1a: every colour a `Complete` land prints on a plain `{T}: Add ...` ability
/// must be registered as a real mana ability, so `Command::TapForMana` can find it and
/// it never touches the stack.
///
/// Scoped to `Complete` defs on purpose: a non-`Complete` def is *declared* wrong and
/// `validate_deck` already rejects it, so holding it to this bar would be asserting
/// against the taxonomy. This is the broad form of the SR-33 defect — a printed colour
/// the card cannot actually produce.
///
/// **Name is misleading (SR-34 review Finding 4)**: this walks `all_cards()` and
/// filters on `printed_tap_mana_colors(&def.oracle_text)` being non-empty, not on card
/// type — it iterates every `Complete` def with a tap-mana clause, not just lands
/// (Signets fall in scope the same way). That is correct and load-bearing (it is why
/// the Signet coverage claim elsewhere holds), but the name says "land" and does not.
/// Not renamed here — the name is quoted by several `memory/` docs across SR-33/SR-34;
/// a rename should update those references in the same change.
#[test]
fn every_complete_land_registers_each_printed_tap_mana_color() {
    let defs = defs_map();
    let mut failures = Vec::new();

    for def in all_cards() {
        if !def.completeness.is_complete() {
            continue;
        }
        let printed = printed_tap_mana_colors(&def.oracle_text);
        if printed.is_empty() {
            continue;
        }
        let registered = registered_colors(&def, &defs);
        let missing: Vec<_> = printed.difference(&registered).collect();
        let invented: Vec<_> = registered.difference(&printed).collect();
        if !missing.is_empty() || !invented.is_empty() {
            failures.push(format!(
                "{}: prints {:?} but registers {:?} (missing {:?}, invented {:?})",
                def.name, printed, registered, missing, invented
            ));
        }
    }

    assert!(
        failures.is_empty(),
        "these Complete defs do not produce exactly the tap-for-mana colours they print \
         (CR 605.1a) — the SR-33 defect class. `missing` is a printed colour the card \
         cannot make (what SR-33 fixed); `invented` is a colour it makes but does not \
         print: {failures:#?}"
    );
}

/// PB-EF12 rewrite (CR 605.3b/106.1b/111.10a) of the SF-12 non-vacuity test.
///
/// **New correct behaviour**: Mana Confluence prints "Add one mana of any color" (plain,
/// unrestricted — PB-EF12 restored it to `Complete` once the colour choice became real) and
/// registers an `any_color: true` `ManaAbility`. `registered_colors` now maps that to the
/// true five-colour option set (not the pre-PB-EF12 always-`{Colorless}` production), so
/// `printed == registered == {W,U,B,R,G}` and the gate's `missing`/`invented` sets are both
/// empty — a real "any color" land now genuinely matches what it prints, rather than being
/// skipped as `known_wrong` or permanently mismatching by construction.
///
/// **The parser is still not blind**: a synthetic decoy def prints "Add one mana of any
/// color" but (bug simulation) registers only a single fixed colour — this must still be
/// caught as a `missing`/`invented` mismatch, proving the gate is not simply "always pass any
/// def with the any-color phrasing" now.
#[test]
fn land_color_gate_is_not_blind_to_any_color_lands() {
    let defs = defs_map();
    let all_five: BTreeSet<ManaColor> = [
        ManaColor::White,
        ManaColor::Blue,
        ManaColor::Black,
        ManaColor::Red,
        ManaColor::Green,
    ]
    .into_iter()
    .collect();

    // New correct behaviour: a real, served, unrestricted any-color land.
    let def = defs
        .get("Mana Confluence")
        .expect("Mana Confluence has a def");
    assert!(
        def.completeness.is_complete(),
        "Mana Confluence should be Complete post-PB-EF12 (EF-W-PB2-3)"
    );
    let printed = printed_tap_mana_colors(&def.oracle_text);
    assert_eq!(
        printed, all_five,
        "\"Add one mana of any color\" must parse to all five colours, not empty (SF-12)"
    );
    let registered = registered_colors(def, &defs);
    assert_eq!(
        registered, all_five,
        "PB-EF12: an any_color ManaAbility must register the real five-colour option set it \
         can now produce, not the pre-fix {{Colorless}} marker"
    );
    assert!(
        printed.difference(&registered).next().is_none()
            && registered.difference(&printed).next().is_none(),
        "a plain, unrestricted any-color land's printed and registered colours must now match \
         exactly (this is the fix — the gate must not report a spurious mismatch)"
    );

    // The parser must still catch a genuine mismatch: synthetic decoy prints "any color"
    // but only registers Green (a fixed-colour bug, not an any_color ability at all).
    let mut defs2 = defs_map();
    let decoy = CardDefinition {
        name: "EF12 Decoy Any Color Land".to_string(),
        oracle_text: "{T}: Add one mana of any color.".to_string(),
        types: TypeLine {
            supertypes: Default::default(),
            card_types: [mtg_engine::state::CardType::Land]
                .iter()
                .copied()
                .collect(),
            subtypes: Default::default(),
        },
        abilities: vec![AbilityDefinition::Activated {
            cost: Cost::Tap,
            effect: Effect::AddMana {
                player: PlayerTarget::Controller,
                mana: mtg_engine::cards::helpers::mana_pool(0, 0, 0, 0, 1, 0),
            },
            timing_restriction: None,
            targets: vec![],
            activation_condition: None,
            activation_zone: None,
            once_per_turn: false,
            modes: None,
        }],
        ..Default::default()
    };
    defs2.insert(decoy.name.clone(), decoy.clone());
    let decoy_printed = printed_tap_mana_colors(&decoy.oracle_text);
    let decoy_registered = registered_colors(&decoy, &defs2);
    assert_eq!(
        decoy_printed, all_five,
        "decoy still prints all five colours"
    );
    assert_eq!(
        decoy_registered,
        [ManaColor::Green].into_iter().collect::<BTreeSet<_>>(),
        "decoy's fixed-colour (non-any_color) ability registers only Green"
    );
    assert_ne!(
        decoy_printed, decoy_registered,
        "a card claiming any-color but registering only one fixed colour must still be caught \
         as a mismatch — the gate is not vacuously true for any any-color-phrased card"
    );
}

/// SR-38 / SG-3 non-vacuity: the strengthened exclusion 2 compares a scaled clause's
/// COLOUR (dropping only its uncheckable amount). Two halves:
///
/// (a) On a real `Complete` scaled land — Cabal Stronghold, `{T}: Add {C}` plus `{3},{T}:
///     Add {B} for each basic Swamp` — the scaled `{B}` now flows through BOTH sides
///     (previously dropped from both). If it did not, the assertion below would fail, proving
///     the colour is genuinely compared rather than ignored.
/// (b) A synthetic def whose oracle prints `{R}` but whose ability registers
///     `AddManaScaled { color: Black }` is caught: `printed = {Red}`, `registered = {Black}`.
///     The pre-SG-3 code (whole-clause drop on printed + `scaled_amount.is_none()` filter on
///     registered) dropped both to the empty set and passed this vacuously.
#[test]
fn land_color_gate_compares_scaled_clause_colors() {
    let defs = defs_map();

    // (a) Real card: the scaled {B} is present on both sides (and the plain {C}).
    let stronghold = defs
        .get("Cabal Stronghold")
        .expect("Cabal Stronghold has a def");
    let printed = printed_tap_mana_colors(&stronghold.oracle_text);
    let registered = registered_colors(stronghold, &defs);
    let both: BTreeSet<ManaColor> = [ManaColor::Black, ManaColor::Colorless]
        .into_iter()
        .collect();
    assert_eq!(
        printed, both,
        "Cabal Stronghold prints {{C}} and a scaled {{B}} — both colours must be kept (SG-3)"
    );
    assert_eq!(
        registered, both,
        "the scaled {{B}} arm must register its colour, compared not filtered (SG-3)"
    );

    // (b) Synthetic wrong-colour scaled ability: prints {R}, registers Black — must mismatch.
    let mut defs2 = defs_map();
    let wrong = CardDefinition {
        name: "SG3 Wrong Colour Scaled".to_string(),
        oracle_text: "{T}: Add {R} for each creature you control.".to_string(),
        abilities: vec![AbilityDefinition::Activated {
            cost: Cost::Tap,
            effect: Effect::AddManaScaled {
                player: PlayerTarget::Controller,
                color: ManaColor::Black, // deliberately NOT the printed {R}
                count: EffectAmount::Fixed(1),
            },
            timing_restriction: None,
            targets: vec![],
            activation_condition: None,
            activation_zone: None,
            once_per_turn: false,
            modes: None,
        }],
        ..Default::default()
    };
    defs2.insert(wrong.name.clone(), wrong.clone());
    let wp = printed_tap_mana_colors(&wrong.oracle_text);
    let wr = registered_colors(&wrong, &defs2);
    assert_eq!(
        wp,
        [ManaColor::Red].into_iter().collect::<BTreeSet<_>>(),
        "the printed {{R}} of a 'for each' clause must be kept, not dropped (SG-3)"
    );
    assert_eq!(
        wr,
        [ManaColor::Black].into_iter().collect::<BTreeSet<_>>(),
        "the scaled ability's registered colour must be compared, not filtered out (SG-3)"
    );
    assert_ne!(
        wp, wr,
        "a scaled ability registering the wrong colour must now be caught, not passed vacuously"
    );
}

/// The four lands SR-33 was filed on, pinned by name and exact colour set.
///
/// Before the fix each reported `mana_abilities=0` and `activated_abilities=1`; a Forest
/// reported `mana_abilities=1`. These are the empirical probes from the finding, promoted
/// to gates.
#[test]
fn named_dual_land_probes_register_both_colors() {
    let defs = defs_map();
    let cases: [(&str, [ManaColor; 2]); 4] = [
        ("Tropical Island", [ManaColor::Blue, ManaColor::Green]),
        ("Underground Sea", [ManaColor::Blue, ManaColor::Black]),
        ("Watery Grave", [ManaColor::Blue, ManaColor::Black]),
        ("Breeding Pool", [ManaColor::Blue, ManaColor::Green]),
    ];

    for (name, colors) in cases {
        let def = defs
            .get(name)
            .unwrap_or_else(|| panic!("{name} has no def"));
        let expected: BTreeSet<ManaColor> = colors.into_iter().collect();
        assert_eq!(
            registered_colors(def, &defs),
            expected,
            "{name} must register exactly its two printed colours as mana abilities"
        );
        let spec = enrich_spec_from_def(
            ObjectSpec::card(PlayerId(1), name).in_zone(ZoneId::Battlefield),
            &defs,
        );
        assert_eq!(
            spec.mana_abilities.len(),
            2,
            "{name} must register one mana ability per colour (CR 605.1a), not a \
             stack-using activated ability"
        );
    }
}

// ── End-to-end: TapForMana selects the colour, without the stack ──────────────

fn find_by_name(state: &GameState, name: &str) -> ObjectId {
    state
        .objects()
        .iter()
        .find(|(_, obj)| obj.characteristics.name == name)
        .map(|(id, _)| *id)
        .unwrap_or_else(|| panic!("object '{name}' not found"))
}

fn pool_amount(state: &GameState, player: PlayerId, color: ManaColor) -> u32 {
    let pool = &state.player(player).expect("player exists").mana_pool;
    match color {
        ManaColor::White => pool.white,
        ManaColor::Blue => pool.blue,
        ManaColor::Black => pool.black,
        ManaColor::Red => pool.red,
        ManaColor::Green => pool.green,
        ManaColor::Colorless => pool.colorless,
    }
}

/// CR 605.3b: a mana ability resolves immediately and never uses the stack. Each
/// `ability_index` must yield its own colour — this is the channel that replaces the
/// unimplemented `Effect::Choose`, so if it does not select, the fix is not a fix.
#[test]
fn tap_for_mana_produces_each_printed_color_without_using_the_stack() {
    let defs = defs_map();
    let registry = CardRegistry::new(all_cards());

    // Tropical Island: index 0 -> {G}, index 1 -> {U} (source order of the def's arms).
    for (index, color) in [(0usize, ManaColor::Green), (1usize, ManaColor::Blue)] {
        let spec = enrich_spec_from_def(
            ObjectSpec::card(PlayerId(1), "Tropical Island").in_zone(ZoneId::Battlefield),
            &defs,
        );
        let state = GameStateBuilder::new()
            .add_player(PlayerId(1))
            .add_player(PlayerId(2))
            .with_registry(Arc::clone(&registry))
            .object(spec)
            .active_player(PlayerId(1))
            .at_step(Step::PreCombatMain)
            .build()
            .expect("state builds");

        let land = find_by_name(&state, "Tropical Island");
        let (state, _events) = process_command(
            state,
            Command::TapForMana {
                player: PlayerId(1),
                source: land,
                ability_index: index,

                chosen_color: None,
            },
        )
        .unwrap_or_else(|e| panic!("TapForMana index {index} should be legal: {e:?}"));

        assert_eq!(
            pool_amount(&state, PlayerId(1), color),
            1,
            "Tropical Island ability_index {index} must add exactly that colour"
        );
        assert!(
            state.stack_objects().is_empty(),
            "CR 605.3b: a mana ability must not use the stack"
        );
    }
}

/// The pre-fix state must not be reachable: an index the card does not have is an error,
/// not a silent fallback to the first colour.
#[test]
fn tap_for_mana_rejects_an_out_of_range_ability_index() {
    let defs = defs_map();
    let registry = CardRegistry::new(all_cards());
    let spec = enrich_spec_from_def(
        ObjectSpec::card(PlayerId(1), "Tropical Island").in_zone(ZoneId::Battlefield),
        &defs,
    );
    let state = GameStateBuilder::new()
        .add_player(PlayerId(1))
        .add_player(PlayerId(2))
        .with_registry(registry)
        .object(spec)
        .active_player(PlayerId(1))
        .at_step(Step::PreCombatMain)
        .build()
        .expect("state builds");

    let land = find_by_name(&state, "Tropical Island");
    assert!(
        process_command(
            state,
            Command::TapForMana {
                player: PlayerId(1),
                source: land,
                ability_index: 2,

                chosen_color: None,
            },
        )
        .is_err(),
        "index 2 does not exist on a two-colour land"
    );
}

/// `Completeness` is load-bearing for these gates, so pin that the demoted cards really
/// are demoted rather than trusting the gate's own emptiness.
///
/// PB-EF7 (2026-07-18): Cankerbloom was removed from this roster — it no longer ships
/// `Effect::Choose` at all (rewritten onto `AbilityDefinition::Activated::modes`, CR 700.2a)
/// and is now `Completeness::Complete`. See `sr33_demoted_cards_carry_truthful_markers`'s
/// sibling coverage in `pb_ef7_modal_activated.rs` for the replacement assertion.
#[test]
fn sr33_demoted_cards_carry_truthful_markers() {
    let defs = defs_map();
    for name in ["Path to Exile", "Rhystic Study"] {
        let def = defs
            .get(name)
            .unwrap_or_else(|| panic!("{name} has no def"));
        assert!(
            matches!(def.completeness, Completeness::KnownWrong(_)),
            "{name} ships a clause that always resolves one way; it must be marked \
             known_wrong, not {:?}",
            def.completeness
        );
    }
}
