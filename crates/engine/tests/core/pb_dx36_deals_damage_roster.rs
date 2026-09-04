//! PB-DX36 (`OOS-CARDS2-6`) — the census, the class gate and the exhaustiveness
//! gates for the "deals damage" trigger family.
//!
//! Design record: `memory/primitives/pb-DX36-execution-notes.md` §0 (binding) and
//! `memory/primitives/pb-plan-DX36.md` step 8. Behavioural probes for the
//! primitive itself live in `primitives::pb_dx36_damage_trigger_dispatch`; this
//! file is the SR-36 roster (walk `all_cards()`, never grep source) plus two
//! structural class gates.
//!
//! # R1 — the census, walking BOTH "deals damage" families
//!
//! Two disjoint axes, each over EVERY face's oracle text (front, back,
//! adventure — PB-DX27's `/review` lesson: a single-face oracle axis is blind
//! to a transformed face or an Adventure half):
//!
//! * **self family** ("this creature/permanent deals \[combat\] damage") —
//!   `TriggerCondition::WhenDealsDamage` / `WhenDealsCombatDamageToPlayer`.
//! * **enchanted family** ("enchanted creature deals \[combat\] damage") —
//!   `TriggerCondition::WhenEnchantedCreatureDealsDamageToPlayer`.
//!
//! **The task brief's self-family "still blocked" list (`Warren Instigator`,
//! `Tandem Lookout`) is a FLOOR, not the population** (dispatch hygiene 6): a
//! narrowed needle (`"whenever this creature/permanent deals"`, tight enough to
//! exclude Infect's reminder text and effect-clause false positives) plus a
//! `Completeness` filter (to exclude `Mist Intruder`/`Poisonous Viper`, whose
//! matching text is served by the native Ingest/Poisonous keyword machinery,
//! not this primitive) finds **TEN**. See `STILL_BLOCKED_SELF_FAMILY_MEMBERS`'s
//! own doc for the per-member breakdown and the two disclosed non-self-trigger
//! matches (`Tandem Lookout`, a Soulbond GRANT; `The Reaver Cleaver`, an
//! Equipment's `WhenEquippedCreatureDealsCombatDamageToPlayer`).
//!
//! # R2 — reconciliation with PB-DX47's inverse ratchet
//!
//! `core::pb_dx47_dispatch_path_roster` already owns the
//! `WhenDealsCombatDamageToPlayer`-declaring population (`DECLARING_MEMBERS`,
//! 25) and its own inverse "prints but doesn't declare" axis
//! (`prints_but_does_not_declare`, ratcheted at <= 22). This file's self-family
//! axis is NOT a second census of that same declaration — it is scoped to the
//! **new** `WhenDealsDamage` condition and to members that print the trigger
//! without declaring EITHER condition (still blocked). A def already counted by
//! PB-DX47's `DECLARING_MEMBERS` is excluded here by construction: this file's
//! "new condition" bucket only counts `WhenDealsDamage` declarations, a
//! DIFFERENT `TriggerCondition` variant PB-DX47's axis cannot see (it matches
//! `WhenDealsCombatDamageToPlayer` alone). The partition is: PB-DX47 owns the
//! narrower (combat-to-player) condition's population; this file owns the
//! general (`WhenDealsDamage`) condition's population and the enchanted family
//! entirely. `Goblin Lackey` is the one member that MOVED between the two
//! files' populations (PB-DX47's own roster doc records the departure,
//! 26 -> 25); it is a member of THIS file's `WhenDealsDamage`-declaring set.
//!
//! **What the disjointness actually rests on, stated because it is NOT the
//! needle** (PB-DX36 `/review`, final NIT). Eight of the ten
//! `STILL_BLOCKED_SELF_FAMILY_MEMBERS` — Ancient Brass Dragon, Ancient Bronze
//! Dragon, Biting-Palm Ninja, Dokuchi Silencer, Frodo, Hellkite Tyrant, The
//! Reaver Cleaver, Walker of Secret Ways — PRINT PB-DX47's own phrase, *"deals
//! combat damage to a player"*. The two rosters do not double-count them, and
//! all four cross-set intersections were computed empty by the reviewer — but
//! the reason is that **PB-DX47's inverse ratchet is `Complete`-only and all
//! eight of these are `partial`**. The disjointness therefore rests on the
//! `Completeness` filter, not on the needle: **promoting any one of the eight
//! moves it between the two rosters simultaneously**, and whoever does that must
//! adjust both, not just this one.
//!
//! # R3 — the class gate: no second dispatcher for any of the 7 new `TriggerEvent`s
//!
//! PB-DX47's `r3` shape (`memory/primitives/pb-DX36-execution-notes.md`
//! cross-cites `OOS-DX47-7`/`OOS-DX51-6`: a gate keyed on ONE syntactic spelling
//! is defeated by a `let`-binding or a multi-line rewrite). Keyed on the
//! MECHANISM: a card-registry or characteristics-list WALK near one of the seven
//! new `TriggerEvent::X` variant names, outside the two functions that are
//! allowed to name them (`rules::abilities::queue_damage_source_triggers`, the
//! sole dispatcher, and `testing::replay_harness::build_face_triggered_abilities`,
//! the sole `TriggerCondition` -> `TriggerEvent` lowering).
//!
//! **`/review` HIGH 2 defeated this gate TWICE, and the first draft's own
//! narrative — "keyed on the mechanism, defeated once, now safe" — was itself
//! the overclaim: keying on the WALK mechanism says nothing about the FILE SET
//! a walk is searched over, or the SPELLING a variant name must take to be
//! found, and both of those are separate axes with their own bypasses.**
//!
//! * **Bypass A (file set).** The scan was `std::fs::read_dir(crates/engine/
//!   src/rules)` — one directory, non-recursive. A second dispatcher planted in
//!   `crates/engine/src/effects/mod.rs` (the file `GameEvent::DamageDealt` is
//!   emitted from at four of its five sites — the single most likely place a
//!   future author writes one) was never scanned at all; every gate in this
//!   file stayed green. Fixed by walking the shared, workspace-wide
//!   `workspace_src_files_checked()` (PB-DX49) instead of a hand-rolled
//!   `read_dir`.
//! * **Bypass B (name spelling).** The name check required the literal
//!   substring `TriggerEvent::{name}`. A `use crate::state::game_object::
//!   TriggerEvent::{SelfDealsDamage as SDD, ..};` beside a walk marker spells
//!   the bare name exactly ONCE, at the import, and the walk body that follows
//!   compares against the alias (`SDD`), never the qualified path — the
//!   literal `TriggerEvent::SelfDealsDamage` (unbroken by the `{`) never
//!   appears anywhere in the file. Fixed by matching the bare name at a WORD
//!   BOUNDARY (`word_boundary_occurs`) instead of behind the `TriggerEvent::`
//!   prefix — but that fix alone was STILL insufficient, because the window
//!   this gate scans was forward-only from each marker, and an import's bare
//!   name sits textually BEFORE the marker it gives meaning to, never after.
//!   The window had to become BIDIRECTIONAL (`walk_adjacent_event_names`'s own
//!   doc) before the bare-word fix actually closed the bypass.
//!
//! Both bypasses were re-executed against the FINAL gate (workspace walk +
//! bidirectional bare-word matching) and reproduce RED; the workspace was
//! restored (`git diff --stat` empty) after each. See
//! `memory/primitives/pb-DX36-execution-notes.md` for the fix-cycle record.
//!
//! # R4 — both new lowering `match`es are exhaustive with no wildcard arm
//!
//! Plan step 4: `match (combat_only, recipient)` and `match recipient` must each
//! have no `_ =>` arm, so a third `DamageRecipient` value is a compile error
//! rather than a silent drop -- the exact failure mode `combat_only` itself was.

use std::collections::BTreeSet;
use std::path::Path;

use mtg_engine::cards::card_definition::{AbilityDefinition, TriggerCondition};
use mtg_engine::{all_cards, CardDefinition};

use crate::decision_site_walk::is_effectively_complete;
// `workspace_src_files_checked` is SHARED, not mirrored -- PB-DX50's own precedent
// (`pb_dx50_mutate_site_roster.rs` / `pb_dx50_effect_choice_gate_sites.rs` both import it
// from here rather than re-deriving the walk). This is deliberately NOT the same
// convention as `all_oracle_text`/`declares_when_deals_damage` above: a FILE WALK is
// infrastructure this batch's thesis is not about, where the ORACLE-TEXT/CONDITION
// derivations above ARE the subject and are duplicated on purpose so this file's own
// census cannot silently inherit a second file's bug.
use crate::pb_dx49_saga_blanking_roster::workspace_src_files_checked;

// ─────────────────────────────────────────────────────────────────────────────
// Shared derivations
// ─────────────────────────────────────────────────────────────────────────────

/// Every face's printed text, joined and lowercased (mirrors
/// `pb_dx47_dispatch_path_roster::all_oracle_text`; duplicated here per this
/// tree's own convention -- PB-DX47's `strip_line_comments`/`condition_names_in`
/// are duplicated in their own file rather than shared, for the same reason:
/// a subject this batch's own thesis is about should not depend on a SECOND
/// file's helper staying correct).
fn all_oracle_text(def: &CardDefinition) -> String {
    let mut out = def.oracle_text.to_lowercase();
    for face in [def.back_face.as_ref(), def.adventure_face.as_ref()]
        .into_iter()
        .flatten()
    {
        out.push('\n');
        out.push_str(&face.oracle_text.to_lowercase());
    }
    out
}

fn all_ability_lists(def: &CardDefinition) -> Vec<&[AbilityDefinition]> {
    let mut out: Vec<&[AbilityDefinition]> = vec![def.abilities.as_slice()];
    if let Some(face) = def.back_face.as_ref() {
        out.push(face.abilities.as_slice());
    }
    if let Some(face) = def.adventure_face.as_ref() {
        out.push(face.abilities.as_slice());
    }
    out
}

fn declares_when_deals_damage(def: &CardDefinition) -> bool {
    all_ability_lists(def)
        .into_iter()
        .flat_map(|a| a.iter())
        .any(|a| {
            matches!(
                a,
                AbilityDefinition::Triggered {
                    trigger_condition: TriggerCondition::WhenDealsDamage { .. },
                    ..
                }
            )
        })
}

fn declares_when_deals_combat_damage_to_player(def: &CardDefinition) -> bool {
    all_ability_lists(def)
        .into_iter()
        .flat_map(|a| a.iter())
        .any(|a| {
            matches!(
                a,
                AbilityDefinition::Triggered {
                    trigger_condition: TriggerCondition::WhenDealsCombatDamageToPlayer,
                    ..
                }
            )
        })
}

fn declares_when_enchanted_creature_deals_damage(def: &CardDefinition) -> bool {
    all_ability_lists(def).into_iter().flat_map(|a| a.iter()).any(|a| {
        matches!(
            a,
            AbilityDefinition::Triggered {
                trigger_condition: TriggerCondition::WhenEnchantedCreatureDealsDamageToPlayer { .. },
                ..
            }
        )
    })
}

/// Self-family needle: the card's own oracle text says a permanent (implicitly
/// itself, "this creature"/"this permanent"/"~") deals damage. Broad by design
/// (over-collection is the safe direction for a census axis) and then narrowed
/// by hand-reading each hit -- the members list below is the narrowed result,
/// not the raw needle hit count, which is printed separately for disclosure.
const SELF_FAMILY_NEEDLES: &[&str] = &[
    "whenever this creature deals",
    "whenever this permanent deals",
];

/// Enchanted-family needle.
const ENCHANTED_FAMILY_NEEDLE: &str = "enchanted creature deals";

fn prints_self_family(def: &CardDefinition) -> bool {
    let text = all_oracle_text(def);
    SELF_FAMILY_NEEDLES.iter().any(|n| text.contains(n))
}

fn prints_enchanted_family(def: &CardDefinition) -> bool {
    all_oracle_text(def).contains(ENCHANTED_FAMILY_NEEDLE)
}

// ─────────────────────────────────────────────────────────────────────────────
// R1a -- self family: the "new condition" population (declares WhenDealsDamage)
// ─────────────────────────────────────────────────────────────────────────────

/// Corpus defs declaring `TriggerCondition::WhenDealsDamage` (any face), by
/// name.
const WHEN_DEALS_DAMAGE_MEMBERS: &[&str] = &["Exalted Angel", "Goblin Lackey"];

#[test]
fn r1a_when_deals_damage_population_is_pinned_by_name() {
    let actual: BTreeSet<String> = all_cards()
        .into_iter()
        .filter(declares_when_deals_damage)
        .map(|d| d.name)
        .collect();
    let expected: BTreeSet<String> = WHEN_DEALS_DAMAGE_MEMBERS
        .iter()
        .map(|s| s.to_string())
        .collect();
    assert_eq!(
        actual, expected,
        "PB-DX36 R1a: the set of defs declaring TriggerCondition::WhenDealsDamage moved."
    );
    // Non-vacuity floor.
    assert_eq!(actual.len(), 2);
}

/// The deck-legal (`Complete`) subset of `WHEN_DEALS_DAMAGE_MEMBERS` -- ONE,
/// `Exalted Angel` (this batch's flip); `Goblin Lackey` stays `partial` (its
/// effect is still `Effect::Nothing` -- two independent blockers survive:
/// no filtered hand-to-battlefield put, no costless "may").
#[test]
fn r1b_when_deals_damage_deck_legal_subset_is_exalted_angel_alone() {
    let actual: BTreeSet<String> = all_cards()
        .into_iter()
        .filter(declares_when_deals_damage)
        .filter(is_effectively_complete)
        .map(|d| d.name)
        .collect();
    assert_eq!(
        actual,
        BTreeSet::from(["Exalted Angel".to_string()]),
        "PB-DX36 R1b: the deck-legal Complete subset of WhenDealsDamage declarers moved."
    );
}

// ─────────────────────────────────────────────────────────────────────────────
// R1b -- self family: prints the trigger but declares NEITHER condition
// (still blocked)
// ─────────────────────────────────────────────────────────────────────────────

/// Corpus defs whose oracle text prints a self-referential "deals \[combat\]
/// damage" trigger, declare NEITHER `WhenDealsDamage` NOR
/// `WhenDealsCombatDamageToPlayer` (PB-DX47's condition) on any face, AND are
/// not `Complete` -- still structurally blocked, by name.
///
/// **The plan's own two named members (`Warren Instigator`, `Tandem Lookout`)
/// are a FLOOR, and this census found TEN, not two** (dispatch hygiene 6 --
/// treat a brief's member list as a floor). The other eight are all genuinely
/// blocked on a DSL gap this primitive does not close (d20 rolls, reflexive
/// "when you do" triggers, `CounterType::Menace`, hidden-info reveals, "gain
/// control of all artifacts") -- each has its own `TODO`/`Completeness::partial`
/// note naming a DIFFERENT blocker, verified individually (none declares any
/// `trigger_condition` at all for this clause). Two members are disclosed
/// rather than silently pinned as ordinary self-triggers: `Tandem Lookout`'s
/// trigger is GRANTED via Soulbond onto ANOTHER creature (invisible to a
/// per-def ability-list walk by construction -- it declares zero triggered
/// abilities of its own); `The Reaver Cleaver` is an EQUIPMENT whose printed
/// text describes what the EQUIPPED creature does, and its actual (unrelated)
/// declared condition is `WhenEquippedCreatureDealsCombatDamageToPlayer`, a
/// different `TriggerCondition` this needle cannot distinguish from a genuine
/// self-declaration. `Non-vacuity of the exclusion`: `Mist Intruder` (Ingest)
/// and `Poisonous Viper` (Poisonous, a test-only card) both print the phrase
/// and are `Complete` -- their damage-to-poison-counter behaviour is
/// implemented through the native KEYWORD machinery (CR 702.90a/702.70a), not
/// through the generic `TriggerCondition` system this primitive touches, so
/// they are correctly EXCLUDED by the `Completeness` filter rather than
/// counted as blocked.
const STILL_BLOCKED_SELF_FAMILY_MEMBERS: &[&str] = &[
    "Ancient Brass Dragon",
    "Ancient Bronze Dragon",
    "Biting-Palm Ninja",
    "Dokuchi Silencer",
    "Frodo, Sauron's Bane",
    "Hellkite Tyrant",
    "Tandem Lookout",
    "The Reaver Cleaver",
    "Walker of Secret Ways",
    "Warren Instigator",
];

#[test]
fn r1c_self_family_still_blocked_population_is_pinned_by_name() {
    let actual: BTreeSet<String> = all_cards()
        .into_iter()
        .filter(prints_self_family)
        .filter(|d| {
            !declares_when_deals_damage(d) && !declares_when_deals_combat_damage_to_player(d)
        })
        .filter(|d| !is_effectively_complete(d))
        .map(|d| d.name)
        .collect();
    let expected: BTreeSet<String> = STILL_BLOCKED_SELF_FAMILY_MEMBERS
        .iter()
        .map(|s| s.to_string())
        .collect();
    assert_eq!(
        actual, expected,
        "PB-DX36 R1c: the self-family 'prints but declares neither condition, not Complete' set \
         moved. Members: {actual:?}"
    );
}

// ─────────────────────────────────────────────────────────────────────────────
// R1d -- enchanted family
// ─────────────────────────────────────────────────────────────────────────────

/// Corpus defs printing an "enchanted creature deals ... damage" trigger, by
/// name, classified into `declares` (already carries
/// `WhenEnchantedCreatureDealsDamageToPlayer`) and `still_blocked` (does not).
#[derive(Debug, Default)]
struct EnchantedCensus {
    declares: BTreeSet<String>,
    still_blocked: BTreeSet<String>,
}

fn enchanted_census() -> EnchantedCensus {
    let mut out = EnchantedCensus::default();
    for def in all_cards().into_iter().filter(prints_enchanted_family) {
        if declares_when_enchanted_creature_deals_damage(&def) {
            out.declares.insert(def.name);
        } else {
            out.still_blocked.insert(def.name);
        }
    }
    out
}

/// `declares`: Sigil of Sleep, Curiosity, Ophidian Eye.
const ENCHANTED_DECLARING_MEMBERS: &[&str] = &["Curiosity", "Ophidian Eye", "Sigil of Sleep"];

/// `still_blocked`: Breath of Fury (blocked on Aura re-attachment, not the
/// trigger condition -- its own module doc states this explicitly).
const ENCHANTED_STILL_BLOCKED_MEMBERS: &[&str] = &["Breath of Fury"];

#[test]
fn r1d_enchanted_family_population_is_pinned_by_name() {
    let census = enchanted_census();
    let expected_declares: BTreeSet<String> = ENCHANTED_DECLARING_MEMBERS
        .iter()
        .map(|s| s.to_string())
        .collect();
    let expected_still_blocked: BTreeSet<String> = ENCHANTED_STILL_BLOCKED_MEMBERS
        .iter()
        .map(|s| s.to_string())
        .collect();
    assert_eq!(
        census.declares, expected_declares,
        "PB-DX36 R1d: the enchanted-family DECLARING population moved."
    );
    assert_eq!(
        census.still_blocked, expected_still_blocked,
        "PB-DX36 R1d: the enchanted-family STILL-BLOCKED population moved."
    );
    // Non-vacuity.
    assert!(!census.declares.is_empty());
}

/// The deck-legal subset of the enchanted-family declarers -- ONE, `Sigil of
/// Sleep`; `Curiosity` and `Ophidian Eye` stay `partial` (a costless-optional
/// "you may draw a card" the DSL cannot express, `OOS-DX48-2`).
#[test]
fn r1e_enchanted_family_deck_legal_subset_is_sigil_of_sleep_alone() {
    let actual: BTreeSet<String> = all_cards()
        .into_iter()
        .filter(declares_when_enchanted_creature_deals_damage)
        .filter(is_effectively_complete)
        .map(|d| d.name)
        .collect();
    assert_eq!(actual, BTreeSet::from(["Sigil of Sleep".to_string()]));
}

// ─────────────────────────────────────────────────────────────────────────────
// t_census_report -- prints every axis (PB-DX8's rule: publish the figure, do
// not transcribe it)
// ─────────────────────────────────────────────────────────────────────────────

/// Run with `cargo test -p mtg-engine --test core pb_dx36 -- --nocapture`.
#[test]
fn t_census_report() {
    let self_needle_hits: Vec<String> = all_cards()
        .into_iter()
        .filter(prints_self_family)
        .map(|d| d.name)
        .collect();
    let enchanted = enchanted_census();
    println!("PB-DX36 census report");
    println!(
        "  self family: WhenDealsDamage declared = {:?}",
        WHEN_DEALS_DAMAGE_MEMBERS
    );
    println!(
        "  self family: still blocked (declares neither condition) = {:?}",
        STILL_BLOCKED_SELF_FAMILY_MEMBERS
    );
    println!(
        "  self family: raw needle hits ({} total, before narrowing) = {:?}",
        self_needle_hits.len(),
        self_needle_hits
    );
    println!("  enchanted family: declares = {:?}", enchanted.declares);
    println!(
        "  enchanted family: still blocked = {:?}",
        enchanted.still_blocked
    );
}

// ─────────────────────────────────────────────────────────────────────────────
// R3 -- class gate: no second dispatcher for any of the 7 new TriggerEvents
// ─────────────────────────────────────────────────────────────────────────────

const NEW_TRIGGER_EVENTS: &[&str] = &[
    "EnchantedCreatureDealsCombatDamageToPlayer",
    "EnchantedCreatureDealsCombatDamageToOpponent",
    "EnchantedCreatureDealsAnyDamageToPlayer",
    "EnchantedCreatureDealsAnyDamageToOpponent",
    "SelfDealsDamage",
    "SelfDealsDamageToPlayer",
    "SelfDealsDamageToOpponent",
];

/// Ability-list / registry WALK markers -- the mechanism, not one spelling.
/// Mirrors `pb_dx47_dispatch_path_roster::REGISTRY_WALK_MARKERS` plus the
/// characteristics-list idiom `collect_triggers_for_event` itself uses, so a
/// hand-rolled second dispatcher written EITHER way is caught.
const WALK_MARKERS: &[&str] = &[
    "effective_abilities(",
    "abilities.iter()",
    "triggered_abilities.iter()",
    "for ability in abilities",
    "for ability in ",
    ".iter()",
    "collect_triggers_for_event(",
];

/// How far past a walk marker a `TriggerEvent::X` name still counts as that
/// walk's payload. Mirrors PB-DX47's `SCAN_WINDOW` (measured stable there).
const SCAN_WINDOW: usize = 1_500;

fn strip_line_comments(src: &str) -> String {
    src.lines()
        .map(|line| match line.find("//") {
            Some(i) => &line[..i],
            None => line,
        })
        .collect::<Vec<_>>()
        .join("\n")
}

/// Whether `needle` occurs in `haystack` as a bare, WORD-BOUNDARY token — the
/// character immediately before and immediately after each occurrence (if any)
/// is not an identifier byte (`[A-Za-z0-9_]`).
///
/// **This is `/review` HIGH 2's bypass B fix.** The prior check required the
/// LITERAL substring `TriggerEvent::{name}`, so `use crate::TriggerEvent::
/// {SelfDealsDamage as SDD, ..};` followed by bare `SDD` at a second dispatcher
/// never spelled the qualified path anywhere and the gate stayed green. Word
/// boundaries are enough to disambiguate the one real overlap in
/// [`NEW_TRIGGER_EVENTS`] -- `SelfDealsDamage` is a strict PREFIX of
/// `SelfDealsDamageToPlayer`/`SelfDealsDamageToOpponent` -- because the byte
/// immediately after a `SelfDealsDamage` match embedded in either longer name
/// is `T` (an identifier byte), which fails the end-boundary check for the
/// SHORT needle at that position.
fn word_boundary_occurs(haystack: &str, needle: &str) -> bool {
    fn is_ident_byte(b: u8) -> bool {
        b.is_ascii_alphanumeric() || b == b'_'
    }
    let bytes = haystack.as_bytes();
    let mut start = 0usize;
    while let Some(rel) = haystack[start..].find(needle) {
        let at = start + rel;
        let start_ok = at == 0 || !is_ident_byte(bytes[at - 1]);
        let end = at + needle.len();
        let end_ok = end >= bytes.len() || !is_ident_byte(bytes[end]);
        if start_ok && end_ok {
            return true;
        }
        start = at + 1;
    }
    false
}

/// Every `TriggerEvent::X` name (X in [`NEW_TRIGGER_EVENTS`]) within
/// [`SCAN_WINDOW`] bytes of a [`WALK_MARKERS`] hit -- BOTH DIRECTIONS -- in
/// `src`, over-collecting deliberately (over-collection can only make R3
/// REDDER).
///
/// Matches the BARE name at a word boundary (`word_boundary_occurs`), not the
/// qualified `TriggerEvent::X` path -- see that function's doc for why a
/// qualified-path check is bypassable by a `use` alias.
///
/// **The window must look BACKWARD as well as forward.** A `use` alias
/// (`use ...::TriggerEvent::{SelfDealsDamage as SDD, ..};`) spells the bare
/// name ONCE, at the import, which is textually BEFORE the walk marker its
/// aliased use sits beside (`for ability in abilities.iter() { if
/// ability.trigger_on == SDD { .. } }` never spells the name again). A
/// forward-only window can never see backward past the marker to the import
/// that gave the alias its meaning, so `SDD`/`SDDP`-shaped bypasses reproduced
/// GREEN under a forward-only window even after word-boundary matching closed
/// the qualified-path half of the same defect. Re-verified over the whole
/// workspace at HEAD with the bidirectional window before shipping it: the
/// two allowed dispatcher/lowering files are the ONLY hits (`t_census_report`
/// prints the live figure rather than trusting this claim).
fn walk_adjacent_event_names(src: &str) -> BTreeSet<String> {
    let stripped = strip_line_comments(src);
    let mut out = BTreeSet::new();
    for marker in WALK_MARKERS {
        let mut from = 0usize;
        while let Some(i) = stripped[from..].find(marker) {
            let at = from + i;
            let lo = at.saturating_sub(SCAN_WINDOW);
            let mut start = lo.min(stripped.len());
            while start > 0 && !stripped.is_char_boundary(start) {
                start -= 1;
            }
            let mut end = (at + marker.len() + SCAN_WINDOW).min(stripped.len());
            while end > at && !stripped.is_char_boundary(end) {
                end -= 1;
            }
            let window = &stripped[start..end];
            for name in NEW_TRIGGER_EVENTS {
                if word_boundary_occurs(window, name) {
                    out.insert((*name).to_string());
                }
            }
            from = at + marker.len();
        }
    }
    out
}

fn extract_function_body<'a>(stripped: &'a str, fn_name: &str) -> &'a str {
    let sig_marker = format!("fn {fn_name}(");
    let sig_start = stripped
        .find(&sig_marker)
        .unwrap_or_else(|| panic!("`fn {fn_name}(` not found in stripped source"));
    let open_brace = stripped[sig_start..]
        .find('{')
        .map(|i| sig_start + i)
        .unwrap_or_else(|| panic!("no opening brace found after `fn {fn_name}(`"));
    let mut depth = 0i32;
    let mut end = None;
    for (offset, ch) in stripped[open_brace..].char_indices() {
        match ch {
            '{' => depth += 1,
            '}' => {
                depth -= 1;
                if depth == 0 {
                    end = Some(open_brace + offset + 1);
                    break;
                }
            }
            _ => {}
        }
    }
    let end = end.unwrap_or_else(|| panic!("unbalanced braces in `fn {fn_name}` body"));
    &stripped[open_brace..end]
}

fn engine_src_path(rel: &str) -> String {
    let path = Path::new(env!("CARGO_MANIFEST_DIR")).join(rel);
    std::fs::read_to_string(&path)
        .unwrap_or_else(|e| panic!("failed to read {}: {e}", path.display()))
}

/// R3. Every `crates/engine/src/rules/*.rs` file plus
/// `crates/engine/src/testing/replay_harness.rs` is scanned for a walk-adjacent
/// occurrence of any of the 7 new `TriggerEvent`s. Legitimate occurrences are
/// bounded to inside `queue_damage_source_triggers`'s body (`rules/abilities.rs`,
/// the sole dispatcher) and `build_face_triggered_abilities`'s body
/// (`testing/replay_harness.rs`, the sole lowering). Anything else is a second
/// dispatcher.
///
/// # This gate's FIRST draft was defeated by an executed attack, inside this
/// # same task
///
/// [`WALK_MARKERS`]'s first draft was `["effective_abilities(",
/// "abilities.iter()", "triggered_abilities.iter()", "for ability in abilities",
/// "collect_triggers_for_event("]` -- every marker keyed on the literal
/// identifier `abilities`. Planting a bypass dispatcher in `rules/mana.rs`
/// written `for ability in defs.iter() { .. } let _fake =
/// TriggerEvent::SelfDealsDamage;` (a different loop VARIABLE binding,
/// `defs.iter()` not `abilities.iter()`) left this gate GREEN -- PB-DX47's own
/// `r3` shape, defeated the same way (`OOS-DX47-7`), inside the file whose
/// header cites that exact defeat as the reason to key on mechanism rather than
/// spelling. Re-keyed: two markers, `"for ability in "` (loop-variable-name
/// agnostic) and a bare `".iter()"` (over-collects deliberately -- collection
/// noise only makes this gate REDDER, never greener), replace the two narrow
/// ones. Re-executed against the SAME planted bypass: now RED. Restored;
/// `git diff --stat crates/engine/src/rules/mana.rs` is empty.
///
/// # This gate's SECOND draft was defeated TWICE by an executed `/review`,
/// # on two axes the first fix never touched
///
/// The first draft's re-keying above closed the WALK-mechanism axis (a second
/// dispatcher written with a different loop-variable name or a different
/// walked collection). It said nothing about the FILE SET this gate is
/// searched over, or the SPELLING a `TriggerEvent` name must take to be
/// found — and both were still bypassable.
///
/// **Bypass A.** A second dispatcher was appended to
/// `crates/engine/src/effects/mod.rs` — walking
/// `obj.characteristics.triggered_abilities.iter()` and comparing
/// `ability.trigger_on` against `TriggerEvent::SelfDealsDamage` /
/// `::SelfDealsDamageToPlayer` by full qualified path, no alias, nothing
/// hidden. This gate stayed GREEN, because the scan was a single, non-recursive
/// `read_dir(crates/engine/src/rules)` and `effects/mod.rs` sits in a sibling
/// directory the scan never visited. Fixed by walking the shared,
/// non-vacuity-checked `workspace_src_files_checked()` (`pb_dx49_saga_
/// blanking_roster`, imported rather than re-derived — PB-DX50's own
/// precedent) instead. Re-executed against the SAME planted bypass: now RED
/// (`found_outside` names `crates/engine/src/effects/mod.rs` for both
/// variants). Restored; `git diff --stat crates/engine/src/effects/mod.rs` is
/// empty.
///
/// **Bypass B.** A `use` alias was appended to `crates/engine/src/rules/
/// mana.rs`: `use crate::state::game_object::TriggerEvent::{SelfDealsDamage as
/// SDD, SelfDealsDamageToPlayer as SDDP};`, consumed by a walk comparing
/// `ability.trigger_on == SDD`. The qualified-path check
/// (`window.contains("TriggerEvent::{name}")`) never matches, because the
/// import spells it `TriggerEvent::{SelfDealsDamage` (a brace, not `::`,
/// between the two) and the walk body spells only the alias. Word-boundary
/// matching on the BARE name (`word_boundary_occurs`) alone was still not
/// enough: this gate's window looks only FORWARD from a marker, and the bare
/// name occurs in the `use` line, textually BEFORE the `for ability in
/// abilities.iter()` marker it sits beside — a forward-only window can never
/// see backward to it. Fixed by making the window bidirectional
/// (`walk_adjacent_event_names`'s own doc). Re-executed against the SAME
/// planted bypass: now RED (`found_outside` names `crates/engine/src/rules/
/// mana.rs` for both aliased variants). Restored; `git diff --stat
/// crates/engine/src/rules/mana.rs` is empty.
///
/// Both re-executions, and the restore after each, are recorded in
/// `memory/primitives/pb-DX36-execution-notes.md`'s `/review` fix-cycle
/// section — this comment states the same facts at the gate itself, which is
/// the copy a future author reads before touching this function.
#[test]
fn r3_no_trigger_event_has_a_second_dispatcher() {
    // `/review` HIGH 2's bypass A fix: this used to be a single, non-recursive
    // `read_dir(crates/engine/src/rules)`, so a second dispatcher planted
    // ANYWHERE outside that one directory -- `crates/engine/src/effects/mod.rs`,
    // the file that emits `GameEvent::DamageDealt` at four of its five sites and
    // therefore the single most likely place a future author writes one -- was
    // invisible to this gate by construction. Walking the SHARED
    // `workspace_src_files_checked()` (PB-DX49) makes the reach at least as wide
    // as the subject: `queue_damage_source_triggers` is a `fn`-private helper
    // reachable only from `rules/abilities.rs` today, but the seven
    // `TriggerEvent`s themselves are `pub` and reachable from anywhere in the
    // workspace, which is exactly the shape `crates/card-defs/src` is excluded
    // for the opposite reason (declarations, not predicates) -- see that
    // function's own doc.
    const ABILITIES_LABEL: &str = "crates/engine/src/rules/abilities.rs";
    const HARNESS_LABEL: &str = "crates/engine/src/testing/replay_harness.rs";

    let mut found_outside: Vec<(String, String)> = Vec::new();

    let abilities_src = engine_src_path("src/rules/abilities.rs");
    let abilities_stripped = strip_line_comments(&abilities_src);
    let allowed_abilities_body =
        extract_function_body(&abilities_stripped, "queue_damage_source_triggers");

    let harness_src = engine_src_path("src/testing/replay_harness.rs");
    let harness_stripped = strip_line_comments(&harness_src);
    let allowed_harness_body =
        extract_function_body(&harness_stripped, "build_face_triggered_abilities");

    let hits_inside_allowed_abilities = walk_adjacent_event_names(allowed_abilities_body);
    let hits_inside_allowed_harness = walk_adjacent_event_names(allowed_harness_body);

    let files = workspace_src_files_checked();
    let files_scanned = files.len();
    for (label, path) in &files {
        let Ok(src) = std::fs::read_to_string(path) else {
            // A file the walk names but cannot read is not evidence of a second
            // dispatcher -- skipping it can only make this gate LESS sensitive,
            // never more, and `files_scanned` (below) still counts it, so an
            // unreadable file cannot silently shrink the walk's own floor.
            continue;
        };
        let hits = walk_adjacent_event_names(&src);
        if hits.is_empty() {
            continue;
        }
        if label == ABILITIES_LABEL {
            // Every walk-adjacent hit in this file must fall INSIDE the allowed
            // dispatcher's body. Checked by re-scanning that body alone and
            // requiring the two hit sets to agree.
            if hits != hits_inside_allowed_abilities {
                for h in hits.difference(&hits_inside_allowed_abilities) {
                    found_outside.push((label.clone(), h.clone()));
                }
            }
            continue;
        }
        if label == HARNESS_LABEL {
            if hits != hits_inside_allowed_harness {
                for h in hits.difference(&hits_inside_allowed_harness) {
                    found_outside.push((label.clone(), h.clone()));
                }
            }
            continue;
        }
        for h in hits {
            found_outside.push((label.clone(), h));
        }
    }

    assert!(
        found_outside.is_empty(),
        "PB-DX36 R3: a walk-adjacent occurrence of a new TriggerEvent outside the \
         one allowed dispatcher/lowering body: {found_outside:?}"
    );
    assert!(
        files_scanned >= 100,
        "R3 non-vacuity: too few workspace .rs files scanned ({files_scanned}); \
         `workspace_src_files_checked` has gone vacuous"
    );
    // The allowed bodies must actually contain every one of the seven names --
    // otherwise this gate would be vacuously satisfied by an empty dispatcher.
    for name in NEW_TRIGGER_EVENTS {
        assert!(
            hits_inside_allowed_abilities.contains(*name),
            "R3 non-vacuity: queue_damage_source_triggers never mentions TriggerEvent::{name}"
        );
        assert!(
            hits_inside_allowed_harness.contains(*name),
            "R3 non-vacuity: build_face_triggered_abilities never mentions TriggerEvent::{name}"
        );
    }
}

// ─────────────────────────────────────────────────────────────────────────────
// R4 -- both lowering matches are exhaustive with no wildcard arm
// ─────────────────────────────────────────────────────────────────────────────

/// R4 (plan step 4): `build_face_triggered_abilities`'s two `match`es selecting
/// `trigger_on` for the enchanted family (`match (combat_only, recipient)`) and
/// the self family (`match recipient`) must both contain NO `_ =>` wildcard arm.
/// A wildcard would make a third `DamageRecipient` value a SILENT drop rather
/// than a compile error -- the exact failure mode `combat_only` itself was
/// before this batch (execution notes §0.5(c)).
#[test]
fn r4_lowering_matches_have_no_wildcard_arm() {
    let harness_src = engine_src_path("src/testing/replay_harness.rs");
    let stripped = strip_line_comments(&harness_src);
    let body = extract_function_body(&stripped, "build_face_triggered_abilities");

    let enchanted_start = body
        .find("let trigger_on = match (*combat_only, recipient) {")
        .expect("the enchanted-family match must exist");
    let enchanted_end = body[enchanted_start..]
        .find("};")
        .map(|i| enchanted_start + i)
        .expect("the enchanted-family match must be closed");
    let enchanted_match = &body[enchanted_start..enchanted_end];
    assert!(
        !enchanted_match.contains("_ =>"),
        "R4: the enchanted-family `match (combat_only, recipient)` must not carry \
         a wildcard arm"
    );
    assert!(
        enchanted_match.matches("DamageRecipient::").count() >= 3,
        "R4 non-vacuity: the enchanted-family match must name all three \
         DamageRecipient variants"
    );

    let self_start = body
        .find("let trigger_on = match recipient {")
        .expect("the self-family match must exist");
    let self_end = body[self_start..]
        .find("};")
        .map(|i| self_start + i)
        .expect("the self-family match must be closed");
    let self_match = &body[self_start..self_end];
    assert!(
        !self_match.contains("_ =>"),
        "R4: the self-family `match recipient` must not carry a wildcard arm"
    );
    assert!(
        self_match.matches("DamageRecipient::").count() >= 3,
        "R4 non-vacuity: the self-family match must name all three DamageRecipient variants"
    );
}
