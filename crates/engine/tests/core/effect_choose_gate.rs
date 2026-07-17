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
    all_cards, enrich_spec_from_def, process_command, CardDefinition, CardRegistry, Command,
    Effect, GameState, GameStateBuilder, ManaColor, ObjectId, ObjectSpec, PlayerId, PlayerTarget,
    Step, ZoneId,
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
/// Unlike `AddManaAnyColor` — which escapes into a real `ManaAbility` with `any_color:
/// true` via `try_as_tap_mana_ability` and so never reaches that arm — `AddManaChoice` is
/// not recognised there, so its users always route through the stack and into the
/// colorless arm. That asymmetry is why sharing the match arm is not the harmless
/// simplification it looks like.
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
        !def_uses(&bare(add_g), "Choose"),
        "gate must not flag a def with no Effect::Choose"
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

/// Colours printed in this card's own plain `{T}: Add ...` clauses.
///
/// Two exclusions, both load-bearing — without them this reports cards that are not the
/// SR-33 defect:
///
/// 1. **The tap must be the whole cost.** `{1}, {T}: Add {R}{W}` (a Signet) and
///    `{T}, Pay 1 life: Add {B} or {G}` (a horizon land) are mana abilities by CR 605.1a,
///    but `enrich_spec_from_def` only lowers `Cost::Tap` into a `ManaAbility`, so they are
///    *uniformly* unregistered regardless of how their colours are modelled. That is a
///    real and separate gap (filed, see the SR-33 follow-ups) — folding it in here would
///    make this gate assert a defect it is not about and cannot fix.
/// 2. **The clause must be this card's own.** `Creatures you control have "{T}: Add {G}"`
///    (Citanul Hierophants) grants the ability to *other* objects; the granting card has
///    no mana ability of its own and is correct as written.
fn printed_tap_mana_colors(oracle: &str) -> BTreeSet<ManaColor> {
    let mut out = BTreeSet::new();
    for (idx, _) in oracle.match_indices("{T}: Add ") {
        // (1) Anything other than a clause boundary before the {T} means an extra cost.
        // Strip spaces only — a newline *is* the boundary, so it must survive the trim.
        let before = oracle[..idx].trim_end_matches(' ');
        if !(before.is_empty() || before.ends_with('\n') || before.ends_with('(')) {
            continue;
        }
        // (2) An odd number of preceding quotes means we are inside a granted ability.
        if oracle[..idx].matches('"').count() % 2 == 1 {
            continue;
        }
        // Walk the clause symbol by symbol, stopping at the first token that is not a
        // mana symbol or a separator — that token ends the "Add" clause. Anything the
        // parser does not understand ends the clause rather than being skipped over, so
        // it can only ever under-report, never invent a colour.
        let mut rest = &oracle[idx + "{T}: Add ".len()..];
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
            out.insert(color);
            rest = r;
        }
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
        .flat_map(|ma| ma.produces.keys().copied())
        .collect()
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
            },
        )
        .is_err(),
        "index 2 does not exist on a two-colour land"
    );
}

/// `Completeness` is load-bearing for these gates, so pin that the demoted cards really
/// are demoted rather than trusting the gate's own emptiness.
#[test]
fn sr33_demoted_cards_carry_truthful_markers() {
    let defs = defs_map();
    for name in ["Path to Exile", "Rhystic Study", "Cankerbloom"] {
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
