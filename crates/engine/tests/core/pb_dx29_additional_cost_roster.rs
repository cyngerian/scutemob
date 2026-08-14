//! PB-DX29 (`OOS-UI2-4`) roster sweep for the keyword-carried additional costs
//! (SR-36 — enumerate `all_cards()`, never grep source).
//!
//! # Why this file exists rather than more rows in `ui2_additional_cost_roster.rs`
//!
//! UI-2 wrote `r3b_squad_marker_and_squad_cost_declare_the_same_defs` because the corpus
//! had failed it: `galadhrim_brigade` shipped `Complete` and deck-legal carrying
//! `KeywordAbility::Squad` and **no** `AbilityDefinition::Squad { cost }`, so
//! `casting.rs::get_squad_cost` returned `None` and every `squad_count > 0` cast was
//! refused — on the very card the first human playtest tried to Squad.
//!
//! That gate is **Squad-specific**, and the same defect promptly recurred one enum
//! variant over, in the opposite direction: `nocturnal_hunger` is `Complete` and
//! deck-legal, carries `AbilityDefinition::Gift { GiftType::Food }` and **no**
//! `KeywordAbility::Gift` marker, and `casting.rs`'s gift block gates on the marker
//! **before** it looks the cost up. Its printed "Gift a Food" was unpayable by any
//! client and nothing anywhere could fail.
//!
//! **A gate written for one variant measures that variant.** [`R2`](r2_every_keyword_carried_cost_declares_its_marker_and_its_cost)
//! is the general form: every keyword-carried additional cost in the DSL, checked in
//! both directions, driven by one table so a new kind is a table row rather than a new
//! test somebody has to remember to write.
//!
//! # Gates
//!
//! * **R1** — the exact per-kind rosters, pinned. A def gaining or losing one of these
//!   costs must be looked at by a human.
//! * **R2** — for every kind, `{defs with the marker} == {defs with the cost}`, both
//!   directions, with one declared exception (see [`FUSE_DATA_CARRIERS`]).
//! * **R3** — every pay-N-times cost is bounded, plus a non-vacuity floor on the
//!   hybrid/Phyrexian shapes `view.rs::format_mana_cost_compact` was taught to render
//!   (UI-2's R4, generalised past Squad — see that test's own doc for what widening it
//!   found, and why the fix was the formatter rather than a narrower gate).
//! * **R4** — the DECK-LEGAL population per kind, pinned. This is the number that says
//!   what a human can actually lose today, and PB-DX29's own scope decisions rest on it.
//! * **R5** — non-vacuity floors for all of the above.
//!
//! What membership asserts, and does NOT (PB-DX4's `BASELINE` lesson, same wording):
//! membership means only that this def declares this specific shape. It says nothing
//! about whether the def is otherwise oracle-correct.

use mtg_engine::{AbilityDefinition, CardDefinition, KeywordAbility, ManaCost};
use std::collections::BTreeSet;

/// The keyword-carried additional-cost kinds: a presence marker AND a cost-bearing
/// ability variant, where `casting.rs` gates on the marker and then reads the variant.
///
/// **Mutate is deliberately absent.** Its marker/cost pair has the same shape
/// (`KeywordAbility::Mutate` + `AbilityDefinition::MutateCost`), but the cast is reached
/// through `LegalAction::CastWithMutate` and `AltCostKind::Mutate` rather than through an
/// announced `AdditionalCost` a client composes, and `legal_actions.rs`'s mutate loop
/// already refuses to offer a mutate cast when `MutateCost` is missing — so a
/// marker-only mutate def is suppressed at the offer rather than 422'd at the cast. It is
/// covered by R2m below instead, which asserts the same equality with that reason stated.
///
/// **Assist, Retrace and Jump-Start are absent because no cost-bearing variant exists**
/// for them: they are marker-only *by design* (`AltCostKind::Retrace`/`JumpStart` live on
/// `CastSpellData.alt_cost`, and Assist's amount is announced, not printed). A
/// two-set equality is unwritable for them and its absence is not an oversight.
///
/// **Collect Evidence is absent** because it has no `KeywordAbility` variant at all — it
/// is gated on `AbilityDefinition::CollectEvidence` alone, so there is no second set to
/// disagree with. That also means SR-5's keyword registry does not see it, which is
/// recorded as a seed rather than fixed here.
const KEYWORD_CARRIED_COSTS: &[(&str, KeywordAbility, fn(&AbilityDefinition) -> bool)] = &[
    ("Squad", KeywordAbility::Squad, |a| {
        matches!(a, AbilityDefinition::Squad { .. })
    }),
    ("Replicate", KeywordAbility::Replicate, |a| {
        matches!(a, AbilityDefinition::Replicate { .. })
    }),
    ("Entwine", KeywordAbility::Entwine, |a| {
        matches!(a, AbilityDefinition::Entwine { .. })
    }),
    ("Escalate", KeywordAbility::Escalate, |a| {
        matches!(a, AbilityDefinition::Escalate { .. })
    }),
    ("Splice", KeywordAbility::Splice, |a| {
        matches!(a, AbilityDefinition::Splice { .. })
    }),
    ("Offspring", KeywordAbility::Offspring, |a| {
        matches!(a, AbilityDefinition::Offspring { .. })
    }),
    ("Gift", KeywordAbility::Gift, |a| {
        matches!(a, AbilityDefinition::Gift { .. })
    }),
    ("Fuse", KeywordAbility::Fuse, |a| {
        matches!(a, AbilityDefinition::Fuse { .. })
    }),
];

/// The one declared exception to R2, and it is a real one rather than a shrug.
///
/// `connive.rs` (Connive // Concoct) carries `AbilityDefinition::Fuse` **without**
/// `KeywordAbility::Fuse`, deliberately and with the reason stated in its own source:
/// neither half of that split card has Fuse, and the `Fuse` variant is being used purely
/// as the data carrier for the right half's name, cost and types. `casting.rs:1279`
/// gates the fuse cast on the marker, so the cost data is inert exactly as intended.
///
/// This entry exists so the exception is a *named def* rather than a hole in the gate:
/// a SECOND def acquiring the same shape reddens R2 and has to justify itself here.
const FUSE_DATA_CARRIERS: &[&str] = &["Connive // Concoct"];

fn defs_with_marker(defs: &[CardDefinition], keyword: &KeywordAbility) -> BTreeSet<String> {
    defs.iter()
        .filter(|d| {
            d.abilities
                .iter()
                .any(|a| matches!(a, AbilityDefinition::Keyword(k) if k == keyword))
        })
        .map(|d| d.name.clone())
        .collect()
}

fn defs_with_cost(
    defs: &[CardDefinition],
    is_cost: fn(&AbilityDefinition) -> bool,
) -> BTreeSet<String> {
    defs.iter()
        .filter(|d| d.abilities.iter().any(is_cost))
        .map(|d| d.name.clone())
        .collect()
}

fn deck_legal(defs: &[CardDefinition], names: &BTreeSet<String>) -> BTreeSet<String> {
    defs.iter()
        .filter(|d| names.contains(&d.name) && d.completeness.is_complete())
        .map(|d| d.name.clone())
        .collect()
}

/// Every `ManaCost` declared by one of the cost-bearing variants above, with its def
/// name and the kind it belongs to.
fn declared_costs(defs: &[CardDefinition]) -> Vec<(String, &'static str, ManaCost)> {
    let mut out = Vec::new();
    for def in defs {
        for ability in &def.abilities {
            let entry = match ability {
                AbilityDefinition::Squad { cost } => Some(("Squad", cost)),
                AbilityDefinition::Replicate { cost } => Some(("Replicate", cost)),
                AbilityDefinition::Entwine { cost } => Some(("Entwine", cost)),
                AbilityDefinition::Escalate { cost } => Some(("Escalate", cost)),
                AbilityDefinition::Offspring { cost } => Some(("Offspring", cost)),
                AbilityDefinition::MutateCost { cost } => Some(("MutateCost", cost)),
                _ => None,
            };
            if let Some((kind, cost)) = entry {
                out.push((def.name.clone(), kind, cost.clone()));
            }
        }
    }
    out
}

/// R1 — the per-kind rosters, pinned EXACT on the COST axis.
///
/// The cost axis rather than the marker axis because that is what `casting.rs` actually
/// charges; R2 then proves the marker axis agrees, so pinning one pins both.
#[test]
fn r1_keyword_carried_cost_rosters_are_pinned() {
    let defs = mtg_engine::all_cards();
    let expected: &[(&str, &[&str])] = &[
        ("Squad", &["Galadhrim Brigade", "Ultramarines Honour Guard"]),
        ("Replicate", &["Train of Thought"]),
        (
            "Entwine",
            &["Goblin War Party", "Promise of Power", "Tooth and Nail"],
        ),
        ("Escalate", &["Blessed Alliance", "Collective Resistance"]),
        ("Splice", &["Glacial Ray"]),
        ("Offspring", &["Flowerfoot Swordmaster"]),
        ("Gift", &["Dawn's Truce", "Nocturnal Hunger"]),
        (
            "Fuse",
            &["Connive // Concoct", "Turn // Burn", "Wear // Tear"],
        ),
    ];
    for (label, keyword, is_cost) in KEYWORD_CARRIED_COSTS {
        let found = defs_with_cost(&defs, *is_cost);
        let want: BTreeSet<String> = expected
            .iter()
            .find(|(l, _)| l == label)
            .map(|(_, names)| names.iter().map(|s| s.to_string()).collect())
            .unwrap_or_else(|| panic!("{label}: no expected roster declared in this test"));
        assert_eq!(
            found, want,
            "R1 ({label}: defs carrying the cost-bearing `AbilityDefinition`) has changed. \
             Membership asserts ONLY that the def declares this shape -- nothing about whether \
             it is otherwise oracle-correct. If a card was ADDED, confirm its printed cost, \
             confirm it also carries `KeywordAbility::{keyword:?}` (R2 will fail otherwise), and \
             confirm `crates/simulator/src/legal_actions.rs`'s plan builder offers it. If \
             REMOVED, confirm that was intentional.\nFound:    {found:?}\nExpected: {want:?}"
        );
    }
}

/// **R2** — every keyword-carried cost declares BOTH halves, in both directions.
///
/// This is `ui2_additional_cost_roster::r3b` generalised from Squad to every kind, and
/// it is the gate that would have caught `nocturnal_hunger` the day it was authored.
///
/// A **marker without a cost** means `casting.rs`'s `get_*_cost` lookup returns `None`
/// and the announced cost is refused. A **cost without a marker** means the keyword gate
/// in front of that lookup refuses first — the same 422 reached one line earlier. Both
/// are silent from the corpus's point of view: nothing red, no card visibly wrong.
#[test]
fn r2_every_keyword_carried_cost_declares_its_marker_and_its_cost() {
    let defs = mtg_engine::all_cards();
    let carriers: BTreeSet<String> = FUSE_DATA_CARRIERS.iter().map(|s| s.to_string()).collect();

    let mut failures: Vec<String> = Vec::new();
    for (label, keyword, is_cost) in KEYWORD_CARRIED_COSTS {
        let marker = defs_with_marker(&defs, keyword);
        let cost = defs_with_cost(&defs, *is_cost);

        let marker_only: Vec<&String> = marker.difference(&cost).collect();
        if !marker_only.is_empty() {
            failures.push(format!(
                "{label}: these defs carry `KeywordAbility::{keyword:?}` but no cost-bearing \
                 `AbilityDefinition`, so the cost is UNPAYABLE -- `casting.rs`'s `get_*_cost` \
                 returns `None` and every announcement is refused. Author the cost from the \
                 printed line (`goblin_war_party.rs` is the reference: BOTH variants, always): \
                 {marker_only:?}"
            ));
        }

        // The cost-without-marker direction, minus the one declared exception.
        let cost_only: Vec<&String> = cost
            .difference(&marker)
            .filter(|n| !(*label == "Fuse" && carriers.contains(*n)))
            .collect();
        if !cost_only.is_empty() {
            failures.push(format!(
                "{label}: these defs carry the COST and not `KeywordAbility::{keyword:?}`. \
                 `casting.rs` gates on the marker BEFORE it looks the cost up, so the cost is \
                 dead and any announcement is refused with \"spell does not have \
                 {}\". This is exactly `nocturnal_hunger`'s defect (PB-DX29). If the def is a \
                 deliberate DATA CARRIER like `connive.rs`, add it to `FUSE_DATA_CARRIERS` with \
                 its reason -- do not delete this check: {cost_only:?}",
                label.to_lowercase()
            ));
        }
    }
    assert!(failures.is_empty(), "{}", failures.join("\n\n"));
}

/// R2m — the same equality for Mutate, stated separately with its own reason.
///
/// Mutate's failure mode is *milder* than the eight above: `legal_actions.rs`'s mutate
/// loop reads `AbilityDefinition::MutateCost` itself and `continue`s when it is absent,
/// so a marker-only mutate def is never OFFERED rather than being offered and refused.
/// That is a better failure, not an acceptable one — the printed mutate cost is still
/// unplayable — so it is gated, just not folded into R2's message.
#[test]
fn r2m_mutate_marker_and_mutate_cost_declare_the_same_defs() {
    let defs = mtg_engine::all_cards();
    let marker = defs_with_marker(&defs, &KeywordAbility::Mutate);
    let cost = defs_with_cost(&defs, |a| matches!(a, AbilityDefinition::MutateCost { .. }));
    assert_eq!(
        marker, cost,
        "the Mutate marker set and the `AbilityDefinition::MutateCost` set disagree. A \
         marker-only def is never offered a mutate cast at all (`legal_actions.rs`'s loop \
         `continue`s on a missing cost); a cost-only def is gated out by \
         `casting.rs`'s `KeywordAbility::Mutate` check. Either way the printed mutate cost is \
         unplayable.\nmarker: {marker:?}\ncost:   {cost:?}"
    );
}

/// R3 — every pay-N-times additional cost is BOUNDED, and the hybrid/Phyrexian premise
/// UI-2 pinned for Squad is recorded as **closed** rather than silently widened.
///
/// # The history, because it is the point of this gate rather than a footnote
///
/// UI-2 wrote `ui2_additional_cost_roster::r4` asserting that no def in the corpus has a
/// hybrid or Phyrexian **Squad** cost, because `view.rs::format_mana_cost_compact`
/// rendered neither and such a cost would have displayed as strictly cheaper than it is.
/// Its own comment promised the gate would "fail loudly the day one is authored".
///
/// PB-DX29 widened that assertion past Squad and it went red immediately, on
/// `brokkos_apex_of_forever`'s `{2}{G}{G}{U/B}` mutate cost — a counter-example the
/// corpus had carried the entire time, invisible because the gate was scoped to the one
/// kind its author was building. **A gate written for one variant measures that
/// variant**, which is this file's whole thesis, arriving a second time.
///
/// The fix was the formatter, not a narrower gate: it now renders CR 107.4e hybrid,
/// CR 107.4f Phyrexian and CR 107.3 `{X}`. So this test no longer asserts absence — it
/// asserts the pay-N-times BOUND, and leaves a non-vacuity check that the formatter is
/// still being handed the shapes it was taught.
///
/// Zero mana value is checked for Squad and Replicate ONLY: both are "pay N times" costs
/// whose `max_count` walk (`legal_actions.rs::repeated_cost_max_count`) is unbounded at
/// zero. Entwine / Escalate / Offspring / Fuse are paid at most once, so a free one is
/// merely unusual, not a hang.
#[test]
fn r3_every_declared_additional_cost_is_renderable_and_bounded() {
    let defs = mtg_engine::all_cards();
    let costs = declared_costs(&defs);
    // Non-vacuity for the formatter's new arms: at least one corpus cost must actually
    // carry a hybrid or Phyrexian pip, or "the formatter renders them" is an untested
    // claim about an unreachable branch. `brokkos_apex_of_forever` is that def; if it
    // ever leaves the corpus, this floor says so instead of going quiet.
    assert!(
        costs
            .iter()
            .any(|(_, _, c)| !c.hybrid.is_empty() || !c.phyrexian.is_empty()),
        "no declared additional cost carries a hybrid or Phyrexian pip, so \
         `view.rs::format_mana_cost_compact`'s CR 107.4e/107.4f arms are unexercised by any \
         real def. Either the corpus lost `brokkos_apex_of_forever` or the walk is broken."
    );
    for (name, kind, cost) in &costs {
        if matches!(*kind, "Squad" | "Replicate") {
            assert!(
                cost.mana_value() > 0,
                "{name} ({kind}): a zero-mana-value pay-N-times cost makes the provider's \
                 affordability walk unbounded (every N stays equally free), so it returns 0 and \
                 the cost can never be offered. Either the def's cost is wrong or that helper \
                 needs a real answer for the free case."
            );
        }
    }
}

/// R4 — the DECK-LEGAL population per kind, pinned.
///
/// This is the number that decides what a human can actually lose. PB-DX29's scope rests
/// on it directly: four of the thirteen unsurfaced cost kinds have **no deck-legal member
/// at all**, so "13 of 15 kinds are invisible" (`OOS-UI2-4`) is arithmetically right and
/// materially misleading. If one of these zeros becomes non-zero, a human can newly reach
/// that kind and its picker stops being latent — which is a fact worth a failing test.
#[test]
fn r4_deck_legal_population_per_kind_is_pinned() {
    let defs = mtg_engine::all_cards();
    let expected: &[(&str, usize)] = &[
        ("Squad", 2),
        ("Replicate", 1),
        ("Entwine", 1),
        ("Escalate", 0),
        ("Splice", 1),
        ("Offspring", 0),
        ("Gift", 1),
        ("Fuse", 3),
    ];
    let mut measured: Vec<(&str, usize)> = Vec::new();
    for (label, _keyword, is_cost) in KEYWORD_CARRIED_COSTS {
        let cost = defs_with_cost(&defs, *is_cost);
        measured.push((label, deck_legal(&defs, &cost).len()));
    }
    // Printed rather than transcribed (PB-DX8's rule: publish the figure, do not quote it).
    println!("PB-DX29 R4 deck-legal population per cost kind: {measured:?}");
    assert_eq!(
        measured,
        expected.to_vec(),
        "R4: the deck-legal population of a keyword-carried additional cost has changed. A ZERO \
         becoming non-zero means a human can now reach that cost kind in a legal deck for the \
         first time -- check that its picker, its `validate_additional_cost_params` arm and its \
         SR-38 suppression all exist before updating this pin. A non-zero becoming zero is \
         usually an honest completeness demotion and only needs the number updated."
    );
}

/// R5 — non-vacuity floors. Without these, a walk that silently stopped finding anything
/// would make R2, R2m and R3 pass by examining nothing.
#[test]
fn r5_rosters_are_not_vacuous() {
    let defs = mtg_engine::all_cards();
    assert!(
        defs.len() > 1_500,
        "the corpus itself looks empty ({} defs) -- `all_cards()` is broken, and every gate in \
         this file would pass vacuously",
        defs.len()
    );
    assert_eq!(
        KEYWORD_CARRIED_COSTS.len(),
        8,
        "the kind table shrank. Every entry is a cost `casting.rs` gates on a keyword and then \
         reads from an `AbilityDefinition`; removing one removes R1/R2/R4 coverage for it."
    );
    for (label, keyword, is_cost) in KEYWORD_CARRIED_COSTS {
        assert!(
            !defs_with_cost(&defs, *is_cost).is_empty(),
            "{label}: the cost predicate matches NOTHING in the corpus, so R2 passes vacuously \
             for this kind -- the predicate is broken, not the corpus"
        );
        assert!(
            !defs_with_marker(&defs, keyword).is_empty(),
            "{label}: the marker predicate matches NOTHING in the corpus, so R2 passes vacuously \
             for this kind -- the predicate is broken, not the corpus"
        );
    }
    assert!(
        !declared_costs(&defs).is_empty(),
        "R3's `declared_costs` walk returns nothing, so R3 passes vacuously"
    );
}

/// R6 — **`ui2_additional_cost_roster::r5`'s premise, re-measured across all eight kinds,
/// and it does NOT hold.**
///
/// R5 asserts that no def declares an additional cost together with an `{X}` or a modal
/// spell ability, and `ActionBar.svelte`'s stage order rests on it: CR 601.2b's own
/// internal order is modes → additional costs → `{X}`, while the client bundles modes and
/// `{X}` into one `ValuePrompt` stage that runs BEFORE the cost stage. R5 walks
/// `spell_additional_costs` and Squad only.
///
/// **Escalate (CR 702.120a) and Entwine (CR 702.42a) are additional costs on modal spells
/// by definition** — `casting.rs` REQUIRES a modal spell for escalate in so many words
/// ("Escalate is a static ability of modal spells"), and entwine's whole function is
/// "choose all modes". So R5 stays green while the condition it was written to detect is
/// live, for the same reason `r3b` stayed green while `nocturnal_hunger` was broken: the
/// walk is narrower than the claim.
///
/// This test PRINTS the offenders rather than asserting their absence, because their
/// absence is not true and asserting it would be a lie that happens to pass. What it
/// asserts instead is the thing that actually matters to the client: **that the offenders
/// are exactly the modal-cost kinds and carry no `{X}`**, so the stage-order inversion is
/// modes-vs-costs (harmless: the client announces modes first, which is CR 601.2b's own
/// order) and never `{X}`-vs-costs (the half that would be wrong).
#[test]
fn r6_the_stage_order_premise_is_re_measured_across_every_kind() {
    let defs = mtg_engine::all_cards();
    let mut modal: Vec<(String, &str)> = Vec::new();
    let mut with_x: Vec<(String, &str)> = Vec::new();
    for (label, _keyword, is_cost) in KEYWORD_CARRIED_COSTS {
        for def in defs.iter().filter(|d| d.abilities.iter().any(*is_cost)) {
            let has_x = def
                .mana_cost
                .as_ref()
                .map(|c| c.x_count > 0)
                .unwrap_or(false);
            let has_modes = def.abilities.iter().any(|a| match a {
                AbilityDefinition::Spell { modes, .. } => modes.is_some(),
                _ => false,
            });
            if has_modes {
                modal.push((def.name.clone(), label));
            }
            if has_x {
                with_x.push((def.name.clone(), label));
            }
        }
    }
    println!("PB-DX29 R6 modal additional-cost defs: {modal:?}");
    println!("PB-DX29 R6 {{X}} additional-cost defs: {with_x:?}");

    // The half that must stay empty. `ActionBar` runs `ValuePrompt` (modes AND `{X}`)
    // before `CostPicker`, which puts `{X}` on the WRONG side of CR 601.2b's order. A
    // def needing both an additional cost and an `{X}` would be announced with its `{X}`
    // fixed before its cost is chosen.
    assert!(
        with_x.is_empty(),
        "these defs declare BOTH a keyword-carried additional cost and an `{{X}}`. \
         `ActionBar.svelte` runs its `ValuePrompt` (modes + X) stage BEFORE the `CostPicker` \
         stage, so the `{{X}}` is announced before the cost is chosen -- the wrong side of CR \
         601.2b's modes -> costs -> X order. Split `ValuePrompt` so X follows the cost stage \
         before authoring these: {with_x:?}"
    );

    // The half that is NOT empty, pinned by kind rather than asserted away.
    let kinds: BTreeSet<&str> = modal.iter().map(|(_, k)| *k).collect();
    let expected: BTreeSet<&str> = ["Entwine", "Escalate"].into_iter().collect();
    assert_eq!(
        kinds, expected,
        "R6: the modal additional-cost kinds are Entwine (CR 702.42a, 'choose all modes') and \
         Escalate (CR 702.120a, 'pay per additional mode') and should be exactly those two -- \
         both are modal BY DEFINITION, which is why `ui2_additional_cost_roster::r5`'s \
         Squad-and-sacrifice walk reports a clean board while this condition is live. A THIRD \
         kind appearing here means a non-modal mechanic acquired a modal def, and the client's \
         modes-before-costs ordering needs re-checking for it.\nFound: {modal:?}"
    );
}
