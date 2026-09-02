//! PB-DX47 (`OOS-DX24-4`): the two-dispatch-path roster gates.
//!
//! `rules/abilities.rs`'s `GameEvent::CombatDamageDealt` arm used to dispatch a
//! `WhenDealsCombatDamageToPlayer` trigger **twice** — once as
//! `PendingTriggerKind::Normal` from the layer-resolved runtime lowering that
//! `collect_triggers_for_event` reads, and once as
//! `PendingTriggerKind::CardDefETB` from a raw card-registry scan standing
//! immediately below it. Neither suppressed the other.
//! `crates/simulator/tests/pb_dx47_double_push_probe.rs` measured the
//! consequence on a game built through the production pregame path: two
//! `PendingTrigger`s for one event, and `drana_liberator_of_malakir` — a
//! `Complete`, deck-legal def printing ONE `+1/+1` counter — putting **two** on
//! its lone attacker.
//!
//! * **R1** — the affected population, by NAME, with the deck-legal `Complete`
//!   subset called out. The v4 memo's conditional *"18 deck-legal `Complete`
//!   defs if real"* is treated as a FLOOR (dispatch hygiene 6), and the figure
//!   is re-derived at HEAD rather than trusted.
//! * **R2** — the **inverse axis**, mandatory rather than optional: the memo's
//!   figure and R1 both key on ONE declaration construct
//!   (`AbilityDefinition::Triggered { WhenDealsCombatDamageToPlayer }`), and a
//!   roster derived from one construct measures that construct (PB-DX26,
//!   PB-DX43, PB-DX15a). This row starts from the printed ORACLE TEXT of every
//!   face instead, and pins the defs that print the trigger and express it some
//!   other way.
//! * **R3** — the **class** sweep, and the reason this file is a gate rather
//!   than a note. The defect is not "this event"; it is "two dispatchers". R3
//!   cross-checks every `TriggerCondition` the runtime lowering
//!   (`build_face_ability_vectors`) converts against every `TriggerCondition`
//!   the queue sites in `rules/abilities.rs` scan out of the card registry, and
//!   fails if the two sets intersect. A second `OOS-DX24-4` is now a red test,
//!   not a five-month-old comment.
//! * **R4** — the deleted scan stays deleted: no `CardDefETB` push may sit in
//!   the `CombatDamageDealt` arm.
//! * **R5** — what the lowering must carry for it to be a safe sole survivor.
//!   `targets` it DOES carry, which discharges PB-EF3 A2 / EF-W-MISS-10's whole
//!   justification for the scan. `modes` it does NOT — and this batch got that
//!   consequence wrong twice before measuring it. First draft: "modal exposure is
//!   zero" (`r5b` refuted it — the population is ONE, `glissa_sunslayer`,
//!   `partial`, so deck-legal exposure is zero). Second draft: "a real capability
//!   the fix gives up" (`primitives::pb_dx47_modal_trigger_mode_zero::t1` refuted
//!   that — nothing modal was ever offered on EITHER path, so the behavioural
//!   delta is zero). `OOS-DX47-3` stays open as the structural gap only.
//! * **`t_census_report`** — PRINTS every axis. Every population figure this
//!   batch publishes is read off this test's output rather than transcribed
//!   (PB-DX8's rule, restated by PB-DX28's MEDIUM and again by PB-DX45's).

use std::collections::BTreeSet;

use mtg_engine::cards::card_definition::{AbilityDefinition, TriggerCondition};
use mtg_engine::{all_cards, CardDefinition};

use crate::decision_site_walk::is_effectively_complete;

/// Source of the runtime lowering — the (A) dispatch path.
const LOWERING_SRC: &str = include_str!("../../src/testing/replay_harness.rs");
/// Source of the trigger queue sites — where the (B) dispatch path lived.
const ABILITIES_SRC: &str = include_str!("../../src/rules/abilities.rs");

// ─────────────────────────────────────────────────────────────────────────────
// Shared derivations
// ─────────────────────────────────────────────────────────────────────────────

/// Every ability list a `WhenDealsCombatDamageToPlayer` declaration can hide in:
/// the front face's, and every alternate face's.
///
/// **Not `def.abilities` alone.** PB-DX27's `/review` found the oracle axis
/// reading `def.oracle_text` while a `CardFace` carries its own; the structural
/// axis has the identical hazard one field over, and PB-DX24's own Q4 fixture is
/// a BACK-face `WhenDealsCombatDamageToPlayer`, so the back face is not
/// hypothetical here.
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

/// How many `WhenDealsCombatDamageToPlayer` triggers this def declares, across
/// every face.
fn declared_count(def: &CardDefinition) -> usize {
    all_ability_lists(def)
        .into_iter()
        .flat_map(|abilities| abilities.iter())
        .filter(|a| {
            matches!(
                a,
                AbilityDefinition::Triggered {
                    trigger_condition: TriggerCondition::WhenDealsCombatDamageToPlayer,
                    ..
                }
            )
        })
        .count()
}

/// Every face's printed text, joined.
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

fn structural_members() -> Vec<CardDefinition> {
    let mut v: Vec<CardDefinition> = all_cards()
        .into_iter()
        .filter(|d| declared_count(d) > 0)
        .collect();
    v.sort_by(|a, b| a.name.cmp(&b.name));
    v
}

// ─────────────────────────────────────────────────────────────────────────────
// R1 — the structural population
// ─────────────────────────────────────────────────────────────────────────────

/// Every corpus def declaring at least one `WhenDealsCombatDamageToPlayer`
/// trigger, by name — **26**.
///
/// # This list was 30 in its first draft, and the gate below caught it
///
/// The first draft was typed from `grep -l WhenDealsCombatDamageToPlayer
/// crates/card-defs/src/defs/*.rs`, which returns **30 files**. Four of them —
/// `bident_of_thassa`, `exalted_angel`, `moria_marauder`, `parapet_thrasher` —
/// name the variant only inside a `// TODO` or a `Completeness` note explaining
/// why they CANNOT use it (they each need a non-self, subtype-filtered variant
/// the DSL does not have).
///
/// That is SR-36's rule verbatim — *enumerate `all_cards()` for rosters, never
/// grep source* — broken inside the batch whose own subject is a false comment,
/// and it is `OOS-CARDS2-7`'s shape a second time: a derivation keyed on source
/// TEXT counts prose. It was caught only because this gate re-derives from the
/// compiled corpus instead of trusting the constant beside it. Filed as
/// `OOS-DX47-2`.
const DECLARING_MEMBERS: &[&str] = &[
    "Ancient Copper Dragon",
    "Ancient Gold Dragon",
    "Ancient Silver Dragon",
    "Balefire Dragon",
    "Bloated Contaminator",
    "Cavern-Hoard Dragon",
    "Drana, Liberator of Malakir",
    "Glissa Sunslayer",
    "Goblin Lackey",
    "Grateful Apparition",
    "Higure, the Still Wind",
    "Ink-Eyes, Servant of Oni",
    "Lathril, Blade of the Elves",
    "Lightning, Army of One",
    "Mist-Syndicate Naga",
    "Mistblade Shinobi",
    "Moon-Circuit Hacker",
    "Moonblade Shinobi",
    "Ninja of the Deep Hours",
    "Ragavan, Nimble Pilferer",
    "Scroll Thief",
    "Sea-Dasher Octopus",
    "Skullsnatcher",
    "Teneb, the Harvester",
    "Throat Slitter",
    "Thrummingbird",
];

/// The deck-legal (`Complete`) subset — **18**, which is exactly the v4 memo's
/// conditional figure.
///
/// The memo's *"18 deck-legal `Complete` defs if real"* is treated as a FLOOR
/// per dispatch hygiene 6, because six batches in a row have found a filed
/// member list short — and PB-DX45 found the first recorded OVER-count, so it is
/// re-derived rather than confirmed. **It reproduces exactly.** That is worth
/// saying plainly: a memo figure agreeing with a re-derivation is the outcome
/// this discipline is FOR, not evidence that the check was unnecessary. R2 is
/// what keeps the agreement from being self-congratulatory — it looks on a
/// different axis, where the two do NOT agree.
const DECK_LEGAL_MEMBERS: &[&str] = &[
    "Ancient Copper Dragon",
    "Ancient Gold Dragon",
    "Ancient Silver Dragon",
    "Balefire Dragon",
    "Bloated Contaminator",
    "Cavern-Hoard Dragon",
    "Drana, Liberator of Malakir",
    "Higure, the Still Wind",
    "Lightning, Army of One",
    "Mist-Syndicate Naga",
    "Mistblade Shinobi",
    "Moonblade Shinobi",
    "Ninja of the Deep Hours",
    "Scroll Thief",
    "Sea-Dasher Octopus",
    "Teneb, the Harvester",
    "Throat Slitter",
    "Thrummingbird",
];

#[test]
fn r1_declaring_population_is_pinned_by_name() {
    let actual: BTreeSet<String> = structural_members()
        .iter()
        .map(|d| d.name.clone())
        .collect();
    let expected: BTreeSet<String> = DECLARING_MEMBERS.iter().map(|s| s.to_string()).collect();
    assert_eq!(
        actual, expected,
        "PB-DX47 R1: the set of defs declaring WhenDealsCombatDamageToPlayer \
         moved. A new member is a card that would have double-dispatched before \
         this batch; re-read the roster doc before re-pinning."
    );
    // Non-vacuity floor: a pin against an empty set is green forever.
    assert_eq!(
        actual.len(),
        26,
        "R1: 26 DEFS, not the 30 FILES a `grep -l` returns — see the roster doc"
    );
}

#[test]
fn r1b_deck_legal_subset_reproduces_the_memo_figure() {
    let actual: BTreeSet<String> = structural_members()
        .into_iter()
        .filter(is_effectively_complete)
        .map(|d| d.name)
        .collect();
    let expected: BTreeSet<String> = DECK_LEGAL_MEMBERS.iter().map(|s| s.to_string()).collect();
    assert_eq!(
        actual,
        expected,
        "PB-DX47 R1b: the deck-legal Complete subset moved. The v4 memo's \
         conditional figure is 18 and this re-derivation returned {}; the memo \
         figure is a FLOOR, never a ceiling (dispatch hygiene 6).",
        actual.len()
    );
    assert_eq!(
        actual.len(),
        18,
        "the v4 memo's conditional '18 deck-legal Complete defs' — re-derived at \
         HEAD, not transcribed"
    );
}

// ─────────────────────────────────────────────────────────────────────────────
// R2 — the inverse axis
// ─────────────────────────────────────────────────────────────────────────────

/// `Complete` defs whose printed text carries "deals combat damage to a player"
/// but which declare NO `WhenDealsCombatDamageToPlayer` trigger.
///
/// These are **not** members of the repaired class — they never reached either
/// dispatch path through that `TriggerCondition`, so they never double-pushed.
/// They are pinned because the structural axis cannot see them, and because a
/// def that MOVES into this set is a def that lost its declaration. Each is
/// classified in the roster doc; the shapes are (i) keyword-derived triggers
/// with their own queue arms (Ingest, Afflict, Dethrone, Training), (ii)
/// equipment/aura triggers that key on the EQUIPPED creature dealing the damage
/// (`WhenEquippedCreatureDealsCombatDamageToPlayer`,
/// `WhenEnchantedCreatureDealsDamageToPlayer`), and (iii) "whenever a creature
/// you control deals combat damage to a player" (`WhenAnyCreatureDeals…`), which
/// is a different `TriggerEvent` with a different queue site.
#[test]
fn r2_inverse_oracle_axis_is_pinned() {
    const NEEDLE: &str = "deals combat damage to a player";
    let mut prints_but_does_not_declare: Vec<String> = all_cards()
        .into_iter()
        .filter(is_effectively_complete)
        .filter(|d| all_oracle_text(d).contains(NEEDLE))
        .filter(|d| declared_count(d) == 0)
        .map(|d| d.name)
        .collect();
    prints_but_does_not_declare.sort();

    // The count, not the names: this set is a WATCHED population, not a repair
    // list, and pinning 40-odd names would make every unrelated card-authoring
    // batch edit this file. The ratchet is what matters — it cannot grow in
    // silence, and it cannot shrink to zero (which would mean the needle stopped
    // matching, i.e. the gate stopped looking).
    assert!(
        !prints_but_does_not_declare.is_empty(),
        "R2 non-vacuity: the oracle needle matched nothing, so this axis is \
         measuring nothing"
    );
    assert!(
        prints_but_does_not_declare.len() <= 40,
        "PB-DX47 R2 ratchet: {} Complete defs print \"{NEEDLE}\" without \
         declaring the trigger (ceiling 40). A jump means either a new \
         keyword-derived family or a def that lost its declaration — the second \
         is a live defect. Members: {prints_but_does_not_declare:?}",
        prints_but_does_not_declare.len()
    );
}

// ─────────────────────────────────────────────────────────────────────────────
// R3 — the CLASS: no TriggerCondition may have two dispatchers
// ─────────────────────────────────────────────────────────────────────────────

/// `TriggerCondition` names the runtime lowering converts into a
/// `TriggeredAbilityDef`, read out of `build_face_ability_vectors`' source.
///
/// Source-derived rather than hand-listed for `OOS-DX24-4`'s own reason: a
/// hand-listed set is a claim that goes stale silently, and this defect survived
/// five months behind exactly such a claim written as a comment.
fn lowered_conditions() -> BTreeSet<String> {
    condition_names_in(LOWERING_SRC)
}

/// `TriggerCondition` names the queue sites in `rules/abilities.rs` match on
/// while walking `def.effective_abilities(..)` / `def.abilities` out of the card
/// registry.
fn registry_scanned_conditions() -> BTreeSet<String> {
    condition_names_in(ABILITIES_SRC)
}

/// Pull `TriggerCondition::Foo` occurrences that appear in a
/// `trigger_condition:` position, ignoring `//` line comments.
///
/// Comment-stripping is load-bearing and is proven so by `r3b` below: this
/// file's own subject matter is a COMMENT that was false, and both source files
/// discuss these variant names in prose.
fn condition_names_in(src: &str) -> BTreeSet<String> {
    let stripped: String = src
        .lines()
        .map(|line| match line.find("//") {
            Some(i) => &line[..i],
            None => line,
        })
        .collect::<Vec<_>>()
        .join("\n");
    let mut out = BTreeSet::new();
    let mut rest = stripped.as_str();
    while let Some(i) = rest.find("trigger_condition:") {
        rest = &rest[i + "trigger_condition:".len()..];
        // The name may sit on the next line (rustfmt wraps deep matches).
        let Some(j) = rest.find("TriggerCondition::") else {
            break;
        };
        // Only accept it if nothing but whitespace separates the two.
        if !rest[..j].trim().is_empty() {
            continue;
        }
        let tail = &rest[j + "TriggerCondition::".len()..];
        let name: String = tail
            .chars()
            .take_while(|c| c.is_alphanumeric() || *c == '_')
            .collect();
        if !name.is_empty() {
            out.insert(name);
        }
    }
    out
}

/// **The class gate.** A `TriggerCondition` that is BOTH lowered into runtime
/// characteristics AND scanned out of the card registry by a queue site is
/// dispatched twice — that is `OOS-DX24-4`, stated as a property rather than as
/// one card's symptom.
///
/// Discriminating revert: restore the deleted registry scan in the
/// `CombatDamageDealt` arm — `WhenDealsCombatDamageToPlayer` re-enters the
/// intersection and this test reddens.
#[test]
fn r3_no_trigger_condition_has_two_dispatchers() {
    let lowered = lowered_conditions();
    let scanned = registry_scanned_conditions();
    let both: BTreeSet<&String> = lowered.intersection(&scanned).collect();

    // `WheneverYouSacrifice` is in both sets and is NOT a second dispatcher: its
    // `abilities.rs` occurrence is inside a `triggers.retain(..)` POST-FILTER
    // that refines the Normal-kind triggers the lowering produced, never a
    // second `triggers.push`. Allow-listed with that reason stated, rather than
    // by loosening the gate.
    const POST_FILTER_ONLY: &[&str] = &["WheneverYouSacrifice"];
    let genuine: Vec<&&String> = both
        .iter()
        .filter(|n| !POST_FILTER_ONLY.contains(&n.as_str()))
        .collect();

    assert!(
        genuine.is_empty(),
        "PB-DX47 R3 (`OOS-DX24-4`, the CLASS): these TriggerConditions are both \
         lowered into runtime characteristics by build_face_ability_vectors AND \
         scanned out of the card registry by a queue site in rules/abilities.rs. \
         Each one dispatches TWICE per event. Found: {genuine:?}"
    );

    // Non-vacuity floors, both directions: a gate whose inputs are empty is
    // green forever, and this one derives BOTH of its inputs from source text.
    assert!(
        lowered.len() >= 20,
        "R3 non-vacuity: the lowering axis found {} conditions; the parser is \
         broken or the lowering moved",
        lowered.len()
    );
    assert!(
        scanned.len() >= 4,
        "R3 non-vacuity: the registry-scan axis found {} conditions; the parser \
         is broken",
        scanned.len()
    );
    assert!(
        both.contains(&"WheneverYouSacrifice".to_string()),
        "R3 self-check: the allowlisted post-filter member must actually be in \
         the intersection, or the allowlist is dead weight hiding a broken parser"
    );
}

/// The comment-stripping in `condition_names_in` is load-bearing, proven by
/// execution rather than asserted.
///
/// `OOS-DX32-6` (a `/* */`-wrapped roster row left the compiled roster while
/// every gate stayed green) and PB-DX8's `/review` finding (narrowing to `//`
/// silently dropped block comments) both apply here. This test states the bound
/// honestly: `//` line comments ARE stripped, block comments are NOT, and the
/// corpus of both source files carries zero `/* */` blocks today.
#[test]
fn r3b_comment_stripping_is_load_bearing_and_its_bound_is_stated() {
    let planted = "fn fake() {\n\
        // trigger_condition: TriggerCondition::PbDx47PlantedInAComment,\n\
        let _ = 1;\n\
    }";
    assert!(
        !condition_names_in(planted).contains("PbDx47PlantedInAComment"),
        "a `//` comment must not contribute a condition name"
    );
    let real = "fn real() {\n    trigger_condition: TriggerCondition::PbDx47PlantedForReal,\n}";
    assert!(
        condition_names_in(real).contains("PbDx47PlantedForReal"),
        "non-vacuity: a real occurrence must be found, or the parser matches \
         nothing and `r3` is green for the wrong reason"
    );
    // The STATED residual, not a silent one.
    assert!(
        !LOWERING_SRC.contains("/*") && !ABILITIES_SRC.contains("/*"),
        "PB-DX47 R3b: `condition_names_in` strips `//` line comments only. Both \
         source files are currently free of `/* */` blocks, which is what makes \
         that sufficient. A block comment appearing in either would silently \
         widen `r3`'s inputs — widen the stripper at that point, do not \
         allowlist."
    );
}

// ─────────────────────────────────────────────────────────────────────────────
// R4 — the deleted scan stays deleted
// ─────────────────────────────────────────────────────────────────────────────

/// The `CombatDamageDealt` arm must contain no card-registry push for this
/// trigger. Keyed on the arm's own text so it cannot be satisfied by the
/// condition name merely disappearing from the file.
#[test]
fn r4_combat_damage_arm_has_no_registry_dispatch() {
    let start = ABILITIES_SRC
        .find("GameEvent::CombatDamageDealt { assignments } => {")
        .expect("the CombatDamageDealt arm must exist");
    // Bound the arm at the next top-level `GameEvent::` arm of the same match.
    let rest = &ABILITIES_SRC[start + 10..];
    let end = rest
        .find("\n            GameEvent::")
        .map(|i| start + 10 + i)
        .unwrap_or(ABILITIES_SRC.len());
    let arm = &ABILITIES_SRC[start..end];

    let code_only: String = arm
        .lines()
        .map(|line| match line.find("//") {
            Some(i) => &line[..i],
            None => line,
        })
        .collect::<Vec<_>>()
        .join("\n");

    assert!(
        !code_only.contains("TriggerCondition::WhenDealsCombatDamageToPlayer"),
        "PB-DX47 R4 (`OOS-DX24-4`): the CombatDamageDealt arm scans the card \
         registry for WhenDealsCombatDamageToPlayer again. The runtime lowering \
         above it already dispatches this trigger; a second dispatcher is the \
         defect this batch removed."
    );
    // Non-vacuity: the arm slice must actually be the arm.
    assert!(
        code_only.contains("TriggerEvent::SelfDealsCombatDamageToPlayer"),
        "R4 non-vacuity: the extracted arm does not contain the lowering \
         dispatch, so the slice is wrong and this gate is measuring nothing"
    );
    assert!(
        arm.len() > 2_000,
        "R4 non-vacuity: extracted arm is only {} bytes",
        arm.len()
    );
}

// ─────────────────────────────────────────────────────────────────────────────
// R5 — what the sole surviving path must carry
// ─────────────────────────────────────────────────────────────────────────────

/// The lowering must copy `targets`, because that is the entire historical
/// justification for the deleted scan.
///
/// PB-EF3 A2's comment: *"CardDefETB kind keeps the raw-index/card-registry
/// lookup authoritative for both effect and target selection (Throat Slitter's
/// 'destroy target nonblack creature that player controls' needs its declared
/// `targets` to survive auto-target selection — EF-W-MISS-10)."* That claim is
/// discharged, and by execution, not by argument: `pbd_damaged_player_filter`'s
/// end-to-end Throat Slitter probe passes through the `Normal` path once its
/// fixture stops building a NAKED object. This row is the structural half.
#[test]
fn r5_lowering_carries_declared_targets() {
    let mut with_targets = 0usize;
    for def in structural_members() {
        for abilities in all_ability_lists(&def) {
            for ability in abilities {
                if let AbilityDefinition::Triggered {
                    trigger_condition: TriggerCondition::WhenDealsCombatDamageToPlayer,
                    targets,
                    ..
                } = ability
                {
                    if !targets.is_empty() {
                        with_targets += 1;
                    }
                }
            }
        }
    }
    assert!(
        with_targets > 0,
        "R5 non-vacuity: no corpus WhenDealsCombatDamageToPlayer trigger \
         declares targets, so the EF-W-MISS-10 property this row exists to hold \
         has no live subject"
    );
    // The lowering's own source must be forwarding them.
    let idx = LOWERING_SRC
        .find("TriggerCondition::WhenDealsCombatDamageToPlayer")
        .expect("the lowering loop must exist");
    let block = &LOWERING_SRC[idx..idx + 1_400];
    assert!(
        block.contains("targets: targets.clone()"),
        "PB-DX47 R5: the runtime lowering must forward the CardDef's declared \
         `targets`. It is now the ONLY dispatch path, so dropping them silently \
         re-creates EF-W-MISS-10 with no second path to mask it."
    );
}

/// The one thing the lowering does NOT carry: `modes`.
///
/// `build_face_ability_vectors` pre-selects mode 0 (CR 700.2b bot fallback),
/// where the deleted registry scan let `flush_sorted`'s modal lookup read the
/// full `ModeSelection` off the card def. This is a real capability the fix
/// gives up, and it is stated rather than glossed.
///
/// # The measured population is ONE, and the first draft of this row said zero
///
/// The `abilities.rs` comment written alongside the fix claimed *"ZERO corpus
/// defs pair `modes` with this `TriggerCondition`"*. This gate refuted it on its
/// first run: `glissa_sunslayer` declares `modes: Some(ModeSelection { .. })`
/// with three modes. The claim was an assumption dressed as a measurement — in
/// a batch whose entire subject is a justifying comment nobody re-checked.
/// Corrected in place; filed as `OOS-DX47-3`.
///
/// # The behavioural delta is ZERO, and that was measured, not reasoned
///
/// The second draft of this doc called the `modes` gap "a real capability the fix
/// gives up". It is not. `primitives::pb_dx47_modal_trigger_mode_zero::t1`
/// resolves a modal `WhenDealsCombatDamageToPlayer` trigger end to end and
/// measures **+1 life — mode 0, once**; restoring the deleted scan takes it to
/// **+2**, i.e. mode 0 TWICE. Nothing modal was ever offered on either path:
/// `flush_sorted` hard-codes `modes_chosen = vec![0]` in both arms of its modal
/// branch for any `StackObjectKind::TriggeredAbility`, and `resolution.rs`'s
/// modal replacement sits outside the `is_carddef_etb` branch. `modal_trigger`
/// (CR 603.3c) is a standing `AutoChosen` row in `core::decision_site_walk`.
///
/// `glissa_sunslayer` is also `Completeness::partial`, so `validate_deck`
/// (Architecture Invariant 9) refuses it and no real game can contain it.
///
/// This row therefore watches a STRUCTURAL gap, not a live regression:
/// `TriggeredAbilityDef` has no `modes` field, so the day CR 603.3c is actually
/// served the lowering must learn to carry it — a HASH bump, out of scope here
/// (`OOS-DX47-3`).
#[test]
fn r5b_modal_exposure_is_pinned_at_one_partial_def() {
    let mut modal: Vec<String> = Vec::new();
    let mut modal_deck_legal: Vec<String> = Vec::new();
    for def in structural_members() {
        let mut is_modal = false;
        for abilities in all_ability_lists(&def) {
            for ability in abilities {
                if let AbilityDefinition::Triggered {
                    trigger_condition: TriggerCondition::WhenDealsCombatDamageToPlayer,
                    modes: Some(_),
                    ..
                } = ability
                {
                    is_modal = true;
                }
            }
        }
        if is_modal {
            modal.push(def.name.clone());
            if is_effectively_complete(&def) {
                modal_deck_legal.push(def.name.clone());
            }
        }
    }
    assert_eq!(
        modal,
        vec!["Glissa Sunslayer".to_string()],
        "PB-DX47 R5b (`OOS-DX47-3`): the set of defs pairing `modes` with \
         WhenDealsCombatDamageToPlayer moved. The surviving runtime lowering \
         pre-selects mode 0, so every member loses its modality."
    );
    assert!(
        modal_deck_legal.is_empty(),
        "PB-DX47 R5b: {modal_deck_legal:?} are DECK-LEGAL and modal. That is the \
         line this batch's deletion was allowed to cross only because it was \
         empty. Teach `build_face_ability_vectors` to carry `modes` (a HASH bump \
         — `OOS-DX47-3`) before re-pinning this."
    );
    // Non-vacuity: the walk must actually be reaching these abilities.
    assert!(
        structural_members()
            .iter()
            .map(declared_count)
            .sum::<usize>()
            >= 26,
        "R5b non-vacuity: the walk found too few declarations to be measuring \
         anything"
    );
}

// ─────────────────────────────────────────────────────────────────────────────
// Report
// ─────────────────────────────────────────────────────────────────────────────

/// PRINTS every axis. No figure in this batch's prose is transcribed.
#[test]
fn t_census_report() {
    let members = structural_members();
    let complete: Vec<&CardDefinition> = members
        .iter()
        .filter(|d| is_effectively_complete(d))
        .collect();
    let lowered = lowered_conditions();
    let scanned = registry_scanned_conditions();

    println!("── PB-DX47 census (OOS-DX24-4) ──");
    println!("axis 1 (structural, AbilityDefinition::Triggered over every face):");
    println!(
        "  defs declaring WhenDealsCombatDamageToPlayer : {}",
        members.len()
    );
    println!(
        "  of which deck-legal `Complete`               : {}",
        complete.len()
    );
    println!(
        "  total declarations (a def may declare >1)    : {}",
        members.iter().map(declared_count).sum::<usize>()
    );
    println!("  deck-legal Complete members:");
    for d in &complete {
        println!("    {}", d.name);
    }
    let inverse: Vec<String> = all_cards()
        .into_iter()
        .filter(is_effectively_complete)
        .filter(|d| all_oracle_text(d).contains("deals combat damage to a player"))
        .filter(|d| declared_count(d) == 0)
        .map(|d| d.name)
        .collect();
    println!("axis 2 (inverse, printed oracle text of every face):");
    println!(
        "  Complete defs printing the trigger but NOT declaring it : {}",
        inverse.len()
    );
    println!("class sweep:");
    println!(
        "  TriggerConditions lowered by build_face_ability_vectors : {}",
        lowered.len()
    );
    println!(
        "  TriggerConditions registry-scanned in abilities.rs      : {}",
        scanned.len()
    );
    println!(
        "  intersection (post-filter allowlist applies)            : {:?}",
        lowered.intersection(&scanned).collect::<Vec<_>>()
    );
}
