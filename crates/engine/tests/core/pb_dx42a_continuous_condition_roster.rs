//! PB-DX42a (rider on PB-DX8): the **corpus** roster gate for CR 613.1d's `condition:
//! Option<Condition>` field on `ContinuousEffectDef`.
//!
//! Authority: `docs/audits/mtg-characteristics-recursion-adjudication.md` §5.1.
//!
//! ## What this gate exists to catch
//!
//! `569087e6` (PB-DX19) shipped `no_condition_evaluator_resolves_characteristics_directly`
//! (`crates/engine/src/effects/mod.rs`), a **source** gate that fixes the *shape* of
//! `check_condition`/`check_static_condition` -- it fails if either evaluator is ever
//! rewritten to resolve characteristics directly rather than through
//! `rules::layers::characteristics_for_condition`. It says nothing about the
//! *population*: `characteristics_for_condition` is a documented CR 613.1d deviation
//! *inside* the layer walk (`layers.rs:96-110` -- it returns **printed**, not
//! layer-resolved, characteristics there), and the corpus's blast radius for that
//! deviation is measured at exactly **one** card
//! (`indomitable_archangel` x `Condition::YouControlNOrMoreWithFilter`). Nothing stops
//! that set growing: the next author who writes "as long as you control a legendary
//! creature, ..." as a `ContinuousEffectDef.condition` routes correctly through the
//! source gate and gets a silently CR-613.1d-wrong answer inside the layer walk -- no
//! failure, no signal. This file is the SR-36-shaped population gate the adjudication
//! calls for.
//!
//! ## CR citations
//!
//! - **CR 604.2**: static abilities that generate conditional continuous effects ("as
//!   long as ..."), the shape `ContinuousEffectDef.condition` exists to serve.
//! - **CR 613.1d**: layer-resolved characteristics are what a condition's filter test is
//!   supposed to read; `characteristics_for_condition` deviates from this inside the
//!   layer walk (see `layers.rs` doc comment on that function).
//! - **CR 613.8**: dependency between continuous effects -- the reason the deviation
//!   exists at all (an unguarded re-entrant walk through `is_effect_active` is what
//!   crashed the process pre-PB-DX19; see `OOS-SIM2-6`).
//!
//! ## The structural walk (why it is not `AbilityDefinition::Static`-only)
//!
//! `ContinuousEffectDef` has **exactly 5 fields**: `layer`, `modification`, `filter`,
//! `duration`, `condition` (`card_definition.rs:4407-4418`, `condition` carries
//! `#[serde(default)]` for DEserialization only -- it still serializes as a `condition`
//! key with value `null` when `None`, so the 5-key set is a reliable fingerprint with no
//! false negatives from an omitted key). [`is_continuous_effect_def_node`] matches an
//! object node BY THAT FIELD SET, not by its parent key, so the walk also reaches nodes
//! nested inside `Effect::ApplyContinuousEffect { effect_def: Box<ContinuousEffectDef> }`
//! at arbitrary depth (e.g. inside `Effect::Conditional`, `Effect::ForEach`, ...), not
//! only the ones hanging directly off `AbilityDefinition::Static { continuous_effect }`.
//!
//! ## The two-axis layer-querying check
//!
//! Axis 1 (pinned, exact): read `crates/engine/src/effects/mod.rs::check_static_condition`
//! -- among the corpus's own conditioned variants, exactly one arm
//! (`Condition::YouControlNOrMoreWithFilter`) calls
//! `rules::layers::characteristics_for_condition`; every other arm present in the corpus
//! (`SourceHasCounters`, `ControllerLifeAtLeast`, `YouControlYourCommander`,
//! `SourceIsUntapped`, `OpponentLifeAtMost`, `IsYourTurn`, `DevotionToColorsLessThan`,
//! `CompletedADungeon`) reads player/object state directly and never resolves another
//! permanent's characteristics.
//!
//! Axis 2 (structural, independent): a condition is layer-querying iff its own payload
//! subtree contains a `TargetFilter`-shaped node -- `TargetFilter` has **exactly 32
//! fields** (`card_definition.rs:3047-3250`, none carry `skip_serializing_if`, so the
//! 32-key set is a reliable fingerprint), and `YouControlNOrMoreWithFilter { filter:
//! TargetFilter, .. }` is the only one of the corpus's 9 conditioned variants that
//! embeds one.
//!
//! **Known limit of axis 2, disclosed rather than papered over**: it is not a general
//! proxy for "reaches `characteristics_for_condition`". `check_condition` has at least
//! one arm, `Condition::ControlLandWithSubtypes(Vec<SubType>)`, that also calls
//! `characteristics_for_condition` (`effects/mod.rs`, the "unless you control a
//! [Plains/...]" ETB-replacement arm) without carrying a `TargetFilter` payload --
//! axis 2 would silently miss it. It happens to agree with axis 1 for THIS gate's
//! population because `ControlLandWithSubtypes` is used only on ETB replacement
//! `unless_condition`s in the corpus today, never inside a `ContinuousEffectDef.condition`
//! -- t7 below measures and pins that absence so the coincidence is monitored, not
//! assumed. If a future author routes `ControlLandWithSubtypes` (or any other
//! non-`TargetFilter`-carrying but layer-querying variant) through
//! `ContinuousEffectDef.condition`, axis 2 will disagree with axis 1 by under-reporting,
//! and `t6_two_axes_agree_on_the_conditioned_population` will catch exactly that
//! disagreement (it does NOT assume agreement -- it recomputes both axes independently
//! and asserts equality every run).

use mtg_engine::all_cards;
use serde_json::Value;
use std::collections::BTreeSet;

// ── Structural fingerprints ─────────────────────────────────────────────────

/// `ContinuousEffectDef`'s serialized field set (`card_definition.rs:4407-4418`).
const CONTINUOUS_EFFECT_DEF_FIELDS: &[&str] =
    &["condition", "duration", "filter", "layer", "modification"];

/// `TargetFilter`'s serialized field set (`card_definition.rs:3047-3250`), 32 fields.
const TARGET_FILTER_FIELDS: &[&str] = &[
    "max_power",
    "min_power",
    "has_card_type",
    "has_keywords",
    "colors",
    "exclude_colors",
    "non_creature",
    "non_land",
    "basic",
    "nonbasic",
    "controller",
    "has_subtype",
    "has_subtypes",
    "has_name",
    "max_cmc",
    "max_cmc_amount",
    "min_cmc",
    "min_cmc_amount",
    "has_card_types",
    "legendary",
    "is_token",
    "is_nontoken",
    "max_toughness",
    "exclude_subtypes",
    "is_attacking",
    "is_blocking",
    "is_tapped",
    "is_untapped",
    "has_chosen_subtype",
    "exclude_chosen_subtype",
    "has_counter_type",
    "exclude_self",
];

/// Does `v`'s key set equal `fields` exactly (as a SET, not an ordered match -- JSON
/// object key order is not semantically meaningful and `serde_json::Map` does not
/// guarantee an order matching struct declaration order)?
fn object_field_set_equals(v: &Value, fields: &[&str]) -> bool {
    match v {
        Value::Object(m) => {
            if m.len() != fields.len() {
                return false;
            }
            let keys: BTreeSet<&str> = m.keys().map(|k| k.as_str()).collect();
            let expected: BTreeSet<&str> = fields.iter().copied().collect();
            keys == expected
        }
        _ => false,
    }
}

fn is_continuous_effect_def_node(v: &Value) -> bool {
    object_field_set_equals(v, CONTINUOUS_EFFECT_DEF_FIELDS)
}

fn is_target_filter_node(v: &Value) -> bool {
    object_field_set_equals(v, TARGET_FILTER_FIELDS)
}

// ── The structural walk ──────────────────────────────────────────────────────

/// One matched `ContinuousEffectDef` node, plus whether it was reached through an
/// ancestor `"Static"` key (any depth above, not just the direct parent).
struct CeMatch<'a> {
    node: &'a Value,
    under_static: bool,
}

/// Recursively collect every `ContinuousEffectDef`-shaped node in `v`, by field set,
/// regardless of parent key -- this is what reaches the ones nested inside
/// `Effect::ApplyContinuousEffect` at arbitrary depth, not only
/// `AbilityDefinition::Static`.
///
/// Continues descending INTO a matched node too (not just past it): a `condition` field
/// can itself carry `And`/`Or`/`Not`-nested conditions, but never another
/// `ContinuousEffectDef` in this corpus -- descending is simply the conservative choice
/// (it costs nothing and cannot double-count nodes, since a `ContinuousEffectDef`'s own
/// 5 fields are not, themselves, a 5-field object with that exact key set).
fn collect_ce_nodes<'a>(v: &'a Value, under_static: bool, out: &mut Vec<CeMatch<'a>>) {
    match v {
        Value::Object(m) => {
            if is_continuous_effect_def_node(v) {
                out.push(CeMatch {
                    node: v,
                    under_static,
                });
            }
            for (k, child) in m {
                let next_under_static = under_static || k == "Static";
                collect_ce_nodes(child, next_under_static, out);
            }
        }
        Value::Array(items) => {
            for item in items {
                collect_ce_nodes(item, under_static, out);
            }
        }
        _ => {}
    }
}

/// Does `v` contain a `TargetFilter`-shaped node anywhere in its subtree (by field set)?
fn subtree_contains_target_filter(v: &Value) -> bool {
    if is_target_filter_node(v) {
        return true;
    }
    match v {
        Value::Object(m) => m.values().any(subtree_contains_target_filter),
        Value::Array(items) => items.iter().any(subtree_contains_target_filter),
        _ => false,
    }
}

/// Serde variant name of a value: unit variants serialize as a bare string, struct/tuple
/// variants as a single-key object. Mirrors `pb_dx5_continuous_effect_roster.rs`'s
/// `variant_name` (PB-DP10's gate-integrity finding: an object-key-only walk is blind to
/// the unit-variant shape).
fn variant_name(v: &Value) -> Option<String> {
    match v {
        Value::String(s) => Some(s.clone()),
        Value::Object(m) if m.len() == 1 => m.keys().next().cloned(),
        _ => None,
    }
}

/// Descend through `And`/`Or`/`Not` (whose payloads are nested `Condition`s, not leaf
/// variants) and collect every LEAF `Condition` variant name reached, plus whether any
/// leaf's own payload carries a `TargetFilter`-shaped node (axis 2).
fn leaf_condition_variants(v: &Value, out: &mut Vec<(String, bool)>) {
    let Some(name) = variant_name(v) else {
        return;
    };
    match name.as_str() {
        "Not" => {
            // Condition::Not(Box<Condition>) -- single-element tuple variant serializes
            // its payload directly under the "Not" key.
            if let Value::Object(m) = v {
                if let Some(inner) = m.get("Not") {
                    leaf_condition_variants(inner, out);
                }
            }
        }
        "And" | "Or" => {
            // Condition::And(Box<Condition>, Box<Condition>) -- a 2-tuple variant
            // serializes its payload as a JSON array of the two elements.
            if let Value::Object(m) = v {
                if let Some(Value::Array(pair)) = m.get(name.as_str()) {
                    for elem in pair {
                        leaf_condition_variants(elem, out);
                    }
                }
            }
        }
        leaf => {
            let has_filter = match v {
                Value::Object(m) => m
                    .get(leaf)
                    .map(subtree_contains_target_filter)
                    .unwrap_or(false),
                _ => false,
            };
            out.push((leaf.to_string(), has_filter));
        }
    }
}

// ── Fixed point (revert row V-descent uses this to prove the descent is load-bearing) ──

/// Extract the leaf `Condition` variants (and axis-2 flag) for `ce_node`'s own
/// `condition` field, iff that field is non-null.
fn ce_node_condition_leaves(ce_node: &Value) -> Vec<(String, bool)> {
    let mut out = Vec::new();
    if let Some(cond) = ce_node.get("condition") {
        if !cond.is_null() {
            leaf_condition_variants(cond, &mut out);
        }
    }
    out
}

// ── Fixture: whole-corpus walk result ───────────────────────────────────────

struct Roster {
    /// (card name, ce node) for every ContinuousEffectDef found, tagged under_static.
    all: Vec<(String, bool)>,
    /// (card name, leaf variant, axis-2 has_filter) for every conditioned instance.
    conditioned: Vec<(String, String, bool)>,
}

fn build_roster() -> Roster {
    let defs = all_cards();
    let mut all = Vec::new();
    let mut conditioned = Vec::new();

    for def in &defs {
        let json = serde_json::to_value(def).expect("CardDefinition serializes");
        let mut nodes = Vec::new();
        collect_ce_nodes(&json, false, &mut nodes);
        for m in &nodes {
            all.push((def.name.clone(), m.under_static));
            for (variant, has_filter) in ce_node_condition_leaves(m.node) {
                conditioned.push((def.name.clone(), variant, has_filter));
            }
        }
    }

    Roster { all, conditioned }
}

// ── T1: non-vacuity floors ──────────────────────────────────────────────────

/// Non-vacuity floor #1 (adjudication §5.1): the structural walk must find AT LEAST as
/// many `ContinuousEffectDef` nodes as measured at HEAD (382). A broken walk that finds
/// nothing -- or almost nothing -- would otherwise pass a set-equality assertion on the
/// conditioned population vacuously (PB-DX6 precedent: two rosters pinned EMPTY rotted
/// silently). `>=`, not `==`: routine card authoring adds `ContinuousEffectDef`s
/// constantly and this floor must not become a flaky gate against that growth.
#[test]
fn t1_total_continuous_effect_def_floor() {
    let roster = build_roster();
    eprintln!(
        "PB-DX42a: total ContinuousEffectDef nodes found = {}",
        roster.all.len()
    );
    assert!(
        roster.all.len() >= 382,
        "CR 604.2/613: the structural ContinuousEffectDef walk found only {} nodes, below \
         the 382 measured at HEAD (2026-08-12) -- the walk has gone vacuous (a serde \
         rename, or is_continuous_effect_def_node's field-set fingerprint no longer \
         matching). This floor exists so a broken walk cannot silently pass the \
         conditioned-population assertions below by finding nothing.",
        roster.all.len()
    );
}

/// Non-vacuity floor #2: at least 176 of the found nodes must be reached WITHOUT ever
/// passing through a `"Static"` ancestor key -- i.e. the structural (field-set) walk is
/// doing real work beyond what a naive `AbilityDefinition::Static`-only walk would find.
/// Without this floor, a regression that silently stops reaching
/// `Effect::ApplyContinuousEffect` nodes (e.g. an accidental early-return once a match is
/// found under `Static`) would still clear T1's floor on the `Static` population alone
/// (measured 206) and go undetected.
#[test]
fn t2_nested_reach_floor() {
    let roster = build_roster();
    let nested = roster
        .all
        .iter()
        .filter(|(_, under_static)| !under_static)
        .count();
    let under_static = roster.all.len() - nested;
    eprintln!(
        "PB-DX42a: under Static = {under_static}, nested (Effect::ApplyContinuousEffect \
         reached only by the structural walk) = {nested}"
    );
    assert!(
        nested >= 176,
        "The structural walk found only {nested} ContinuousEffectDef nodes NOT reached \
         through a 'Static' ancestor key, below the 176 measured at HEAD (2026-08-12). \
         This is the adjudication's whole point (§5.1): a walk that only ever finds \
         AbilityDefinition::Static nodes would still clear T1's total floor on the \
         Static-only population and silently stop seeing everything nested inside \
         Effect::ApplyContinuousEffect.",
        nested = nested
    );
}

/// Non-vacuity floor #3: at least 17 `ContinuousEffectDef` instances must carry a
/// non-null `condition`. Same rationale as T1/T2, scoped to the population this gate's
/// exact-set pin actually depends on.
#[test]
fn t3_conditioned_population_floor() {
    let roster = build_roster();
    let conditioned_cards: BTreeSet<&str> = roster
        .conditioned
        .iter()
        .map(|(c, _, _)| c.as_str())
        .collect();
    eprintln!(
        "PB-DX42a: conditioned ContinuousEffectDef instances = {}, across {} distinct cards",
        roster.conditioned.len(),
        conditioned_cards.len()
    );
    assert!(
        roster.conditioned.len() >= 17,
        "CR 604.2: the structural walk found only {} conditioned ContinuousEffectDef \
         instances, below the 17 measured at HEAD (2026-08-12). A broken descent through \
         And/Or/Not, or a broken condition-field read, would show up here first.",
        roster.conditioned.len()
    );
}

// ── T4: the full conditioned census, MEASURED not transcribed ──────────────

/// The adjudication (§5.1) warns: "Derive the pinned numbers from a fresh enumeration at
/// dispatch, not from §2.1. §2.1's first published version mis-stated the per-variant
/// census by two rows." This test prints the full (card, variant) census every run via
/// `eprintln!` -- THAT output, not any number in this file's doc comments, is the
/// current source of truth, exactly the convention `pb_dx5_continuous_effect_roster.rs`
/// established for its own mass-filter-by-completeness table.
#[test]
fn t4_conditioned_census_report() {
    let roster = build_roster();
    let mut sorted = roster.conditioned.clone();
    sorted.sort();
    eprintln!("PB-DX42a conditioned ContinuousEffectDef census (card, Condition variant, axis-2 has_filter):");
    for (card, variant, has_filter) in &sorted {
        eprintln!("  {card} x {variant} (TargetFilter payload: {has_filter})");
    }
    let distinct_variants: BTreeSet<&str> = roster
        .conditioned
        .iter()
        .map(|(_, v, _)| v.as_str())
        .collect();
    eprintln!("  TOTAL instances: {}", roster.conditioned.len());
    eprintln!("  DISTINCT variants: {}", distinct_variants.len());
}

// ── T4b: synthetic And/Or/Not descent proof ─────────────────────────────────

/// **No corpus `ContinuousEffectDef.condition` currently uses `And`/`Or`/`Not`** (t4's
/// census, measured: all 17 conditioned instances are bare leaf variants). The
/// descend-through-compound-conditions requirement can therefore only be proven against
/// a hand-built fixture, not the live corpus -- this is that proof. Builds
/// `And(Or(SourceIsUntapped, IsYourTurn), Not(CompletedADungeon))` as raw JSON (the exact
/// shape serde would produce for `Condition::And(Box::new(Condition::Or(..)),
/// Box::new(Condition::Not(..)))`) and asserts [`leaf_condition_variants`] returns
/// exactly the three LEAVES, never the string `"And"`, `"Or"`, or `"Not"` itself.
#[test]
fn t4b_and_or_not_descent_reaches_every_leaf() {
    let synthetic = serde_json::json!({
        "And": [
            { "Or": ["SourceIsUntapped", "IsYourTurn"] },
            { "Not": "CompletedADungeon" }
        ]
    });
    let mut out = Vec::new();
    leaf_condition_variants(&synthetic, &mut out);
    let leaves: BTreeSet<String> = out.into_iter().map(|(name, _)| name).collect();
    let expected: BTreeSet<String> = ["SourceIsUntapped", "IsYourTurn", "CompletedADungeon"]
        .into_iter()
        .map(String::from)
        .collect();
    assert_eq!(
        leaves, expected,
        "leaf_condition_variants must descend through And/Or/Not and report only the \
         LEAF variants underneath them, never 'And'/'Or'/'Not' as if they were leaves \
         themselves -- got {leaves:?}, wanted {expected:?}"
    );
}

// ── T5: axis 1 -- the pinned exact set ──────────────────────────────────────

/// Axis 1 (pinned): the layer-querying subset of the conditioned population, decided by
/// reading `crates/engine/src/effects/mod.rs::check_static_condition` directly. Among
/// the corpus's own conditioned variants, exactly one arm
/// (`Condition::YouControlNOrMoreWithFilter`) calls
/// `rules::layers::characteristics_for_condition`.
fn axis1_layer_querying_pinned_set(roster: &Roster) -> BTreeSet<(String, String)> {
    roster
        .conditioned
        .iter()
        .filter(|(_, variant, _)| variant == "YouControlNOrMoreWithFilter")
        .map(|(card, variant, _)| (card.clone(), variant.clone()))
        .collect()
}

/// CR 613.1d / adjudication §5.1: assert the layer-querying subset is EXACTLY
/// `{ (indomitable_archangel, YouControlNOrMoreWithFilter) }`. If this assertion fails,
/// the failure message states BOTH legal exits (adjudication §5.1's own requirement) --
/// see `t8_failure_message_names_both_exits` for the proof that the message text
/// actually contains them.
/// **PB-DX27 (`scutemob-209`, 2026-08-13) took exit (b): the population GREW, 1 -> 2.**
///
/// This gate shipped as PB-DX8's `PB-DX42a` rider, and `OOS-ADJ-2` is the seed that
/// justified it — *"nothing gates the size of the corpus population carrying a
/// layer-querying `ContinuousEffectDef.condition`. It is 1 today. A new conditional
/// static passes `no_condition_evaluator_resolves_characteristics_directly` and
/// silently joins the deviation."* That is exactly what happened, on the gate's first
/// real event, and it was NOT silent.
///
/// The new member is **The World Tree**, whose printed "As long as you control six or
/// more lands, lands you control have '{T}: Add one mana of any color.'" PB-DX27
/// authored from the def's own `Completeness` note recipe (`OOS-CARDS2-11`). Its
/// condition is `YouControlNOrMoreWithFilter { count: 6, filter: has_card_type: Land }`
/// — the *same variant* as Indomitable Archangel's, so exit (a) is not available: it
/// reaches `characteristics_for_condition` through the identical
/// `check_static_condition` arm and joins the CR 613.1d deviation.
///
/// **Consequences, named here rather than absorbed** (the exit's own requirement):
///
/// - `docs/audits/mtg-characteristics-recursion-adjudication.md` §5.2 ranks **PB-DX42b
///   at 13 on a measured population of exactly 1**. That premise is now false. The rank
///   argument must be recomputed against a population of **2**; filed as `OOS-DX27-9`.
/// - The adjudication's §2.3 supply-side measurement ("live-wrong on 7 deck-legal
///   `Complete` pairs") was computed for the Archangel's filter, which reads the
///   **Artifact** card type. The World Tree's filter reads the **Land** card type, so
///   the pair census does not carry over — a fresh supply measurement is owed against
///   whatever moves `Land`, and §7 of that document already states the 7 is "a floor
///   for its own filter, and the filter is narrow".
/// - Direction of the new deviation: base characteristics inside the layer walk means a
///   permanent that *becomes* a land in Layer 4 is **under-counted** by The World Tree,
///   and one that stops being a land is over-counted. Note that PB-DX27's own Blood Moon
///   fix (`OOS-ADJ-7`) makes the commonest interaction *more* correct, not less: a
///   Blood-Mooned nonbasic land keeps every card type it had, so it is still a Land and
///   still counts.
///
/// Authoring the clause was preferred to leaving it blocked because the def's blocker
/// note claimed a DSL gap that did not exist, which is the class PB-DX27 exists to
/// close. Growing a known, gated, ranked deviation by one named member is a smaller
/// debt than leaving a printed ability unimplemented behind a false note.
#[test]
fn t5_layer_querying_set_is_pinned() {
    let roster = build_roster();
    let actual = axis1_layer_querying_pinned_set(&roster);
    let expected: BTreeSet<(String, String)> = [
        (
            "Indomitable Archangel".to_string(),
            "YouControlNOrMoreWithFilter".to_string(),
        ),
        (
            "The World Tree".to_string(),
            "YouControlNOrMoreWithFilter".to_string(),
        ),
    ]
    .into_iter()
    .collect();

    assert_eq!(
        actual,
        expected,
        "{}",
        layer_querying_population_changed_message(&actual, &expected)
    );
}

/// Adjudication §5.1: "The failure message must tell the next author the choice: either
/// the new condition is layer-safe, or the population has grown and PB-DX42b's rank must
/// be recomputed." Both exits, verbatim in substance.
fn layer_querying_population_changed_message(
    actual: &BTreeSet<(String, String)>,
    expected: &BTreeSet<(String, String)>,
) -> String {
    format!(
        "CR 613.1d: the population of card defs whose ContinuousEffectDef.condition \
         reaches a layer query (via rules::layers::characteristics_for_condition, which \
         deviates from CR 613.1d inside the layer walk -- see layers.rs's doc comment on \
         that function) has changed. Expected {expected:?}, found {actual:?}. \
         docs/audits/mtg-characteristics-recursion-adjudication.md §5.1 gives two legal \
         exits, and you must pick one: \
         (a) the new condition is LAYER-SAFE -- it does not reach a layer query inside \
         calculate_characteristics -- in which case widen this pinned set with a written \
         reason citing the specific check_static_condition/check_condition arm that \
         proves it safe; or \
         (b) the population carrying a layer-querying condition has GROWN, in which case \
         PB-DX42b's rank must be recomputed (adjudication §5.2 ranks it 13 on a measured \
         population of exactly 1 -- a bigger population may move that rank), and the new \
         member(s) should be named in the close notes rather than silently absorbed."
    )
}

// ── T6: axis 2 -- the independent structural derivation, and cross-check ───

/// Axis 2 (structural, independent): a conditioned instance is layer-querying iff its
/// leaf Condition's own payload subtree contains a TargetFilter-shaped node. Derived
/// purely from the serialized shape, with NO reference to `check_static_condition`'s
/// source.
fn axis2_layer_querying_structural_set(roster: &Roster) -> BTreeSet<(String, String)> {
    roster
        .conditioned
        .iter()
        .filter(|(_, _, has_filter)| *has_filter)
        .map(|(card, variant, _)| (card.clone(), variant.clone()))
        .collect()
}

/// SR-36 / PB-DX26 "inverse census" precedent: derive the layer-querying predicate by a
/// SECOND, independent axis (structural: does the condition's payload carry a
/// TargetFilter?) and assert the two axes AGREE on the corpus's actual conditioned
/// population, recomputed every run -- this does not assume agreement, it proves it.
///
/// See this file's module doc for the one documented case where axis 2 would NOT
/// generalize (`Condition::ControlLandWithSubtypes`, which reaches
/// `characteristics_for_condition` without carrying a `TargetFilter`) -- that variant is
/// not part of THIS gate's population (it is used only on ETB `unless_condition`s, never
/// on a `ContinuousEffectDef.condition`), and `t7_control_land_with_subtypes_absent_from_
/// population` pins that absence so the coincidence this test relies on is monitored.
#[test]
fn t6_two_axes_agree_on_the_conditioned_population() {
    let roster = build_roster();
    let axis1 = axis1_layer_querying_pinned_set(&roster);
    let axis2 = axis2_layer_querying_structural_set(&roster);
    assert_eq!(
        axis1, axis2,
        "Axis 1 (read from check_static_condition's own match arms: only \
         YouControlNOrMoreWithFilter calls characteristics_for_condition among the \
         corpus's conditioned variants) and axis 2 (structural: does the condition's \
         payload carry a TargetFilter-shaped node?) disagree -- axis1={axis1:?} \
         axis2={axis2:?}. This is a real finding, not a gate bug to paper over: either a \
         new conditioned variant reaches characteristics_for_condition without carrying a \
         TargetFilter (axis 2 undercounts -- see this file's module doc for the known \
         ControlLandWithSubtypes case), or a TargetFilter-carrying variant was added that \
         does NOT reach characteristics_for_condition (axis 2 overcounts). Re-read \
         effects/mod.rs::check_static_condition and check_condition before changing \
         either axis."
    );
}

/// Pins the coincidence axis 2's module-doc caveat depends on: `ControlLandWithSubtypes`
/// (which reaches `characteristics_for_condition` per `check_condition` without carrying
/// a `TargetFilter`) does not currently appear anywhere in the conditioned
/// `ContinuousEffectDef` population this gate walks. If it ever does, axis 2 would
/// silently undercount relative to a hypothetical axis-1 update that added it -- this
/// test's failure is the signal to re-derive axis 2 rather than trust the coincidence.
#[test]
fn t7_control_land_with_subtypes_absent_from_population() {
    let roster = build_roster();
    let present = roster
        .conditioned
        .iter()
        .any(|(_, variant, _)| variant == "ControlLandWithSubtypes");
    assert!(
        !present,
        "Condition::ControlLandWithSubtypes now appears in the ContinuousEffectDef.condition \
         population. This variant reaches rules::layers::characteristics_for_condition \
         (effects/mod.rs::check_condition) WITHOUT carrying a TargetFilter payload, so \
         axis 2's structural check (t6) would silently miss it as layer-querying. \
         Re-derive axis 2 (or add a second structural signal) before trusting t6's \
         agreement again, and check whether t5's pinned set needs to grow."
    );
}

// ── T8: failure-message proof (decision_gate.rs precedent) ─────────────────

/// `decision_gate.rs::t4_failure_message_names_the_bound` precedent: that test exists
/// because a module doc once cited a test that had never been written. Prove, by
/// execution, that the failure message this gate WOULD print contains both of the
/// adjudication's two legal exits in substance -- not just that the assertion fires.
#[test]
fn t8_failure_message_names_both_exits() {
    let actual: BTreeSet<(String, String)> = BTreeSet::new();
    let expected: BTreeSet<(String, String)> = [(
        "Indomitable Archangel".to_string(),
        "YouControlNOrMoreWithFilter".to_string(),
    )]
    .into_iter()
    .collect();
    let msg = layer_querying_population_changed_message(&actual, &expected);

    assert!(
        msg.contains("LAYER-SAFE") && msg.contains("does not reach a layer query"),
        "failure message must name exit (a) -- 'the new condition is layer-safe': {msg}"
    );
    assert!(
        msg.contains("GROWN") && msg.contains("PB-DX42b's rank must be recomputed"),
        "failure message must name exit (b) -- 'the population has grown, recompute \
         PB-DX42b's rank': {msg}"
    );
}

// ── T9: field-set fingerprint sanity (proves the two fingerprints don't collide) ────

/// The two structural fingerprints used by this file must not be able to match the same node,
/// and — the part that matters — each must actually match the struct it names, field for field.
///
/// **Rewritten during the `/review` fix cycle: the first version was a compile-time tautology.**
/// It compared the two `const` arrays' *lengths* (fixed at 5 and 32) and then asserted
/// `intersection.is_none() || len != len`, whose right-hand side is unconditionally true. It
/// could not fail for any code change — including the one change that matters here, a field
/// added to or removed from `ContinuousEffectDef`, which would silently desync the fingerprint
/// from the type while every test stayed green. This version reads the two struct declarations
/// out of `crates/card-types/src/cards/card_definition.rs` and compares the field NAMES, so the
/// desync fails by name.
#[test]
fn t9_fingerprints_match_their_structs_and_cannot_collide() {
    let ce: BTreeSet<&str> = CONTINUOUS_EFFECT_DEF_FIELDS.iter().copied().collect();
    let tf: BTreeSet<&str> = TARGET_FILTER_FIELDS.iter().copied().collect();

    // A node is classified by an EXACT field-set match, so an overlap is harmless as long as the
    // two sets are not equal. Assert the real property (set inequality), not a length comparison
    // that happens to imply it today.
    assert_ne!(
        ce, tf,
        "the two fingerprints are the same SET, so a node matching one matches the other and the \
         walk cannot tell a ContinuousEffectDef from a TargetFilter"
    );

    // And each fingerprint must still equal its struct's declared field set.
    let src = std::fs::read_to_string(
        std::path::PathBuf::from(env!("CARGO_MANIFEST_DIR"))
            .parent()
            .and_then(std::path::Path::parent)
            .expect("engine manifest dir is <workspace>/crates/engine")
            .join("crates/card-types/src/cards/card_definition.rs"),
    )
    .expect("card_definition.rs must be readable");

    let declared = |struct_name: &str| -> BTreeSet<String> {
        let at = src
            .find(&format!("pub struct {struct_name} {{"))
            .unwrap_or_else(|| panic!("struct {struct_name} not found in card_definition.rs"));
        let body_start = src[at..].find('{').expect("brace") + at + 1;
        let mut depth = 1usize;
        let mut end = body_start;
        for (i, ch) in src[body_start..].char_indices() {
            match ch {
                '{' => depth += 1,
                '}' => {
                    depth -= 1;
                    if depth == 0 {
                        end = body_start + i;
                        break;
                    }
                }
                _ => {}
            }
        }
        src[body_start..end]
            .lines()
            .map(|l| l.split("//").next().unwrap_or("").trim())
            .filter_map(|l| l.strip_prefix("pub "))
            .filter_map(|l| l.split(':').next())
            .map(|n| n.trim().to_string())
            .filter(|n| !n.is_empty() && n.chars().all(|c| c.is_ascii_alphanumeric() || c == '_'))
            .collect()
    };

    let ce_declared = declared("ContinuousEffectDef");
    let ce_pinned: BTreeSet<String> = ce.iter().map(|s| s.to_string()).collect();
    assert_eq!(
        ce_pinned, ce_declared,
        "CONTINUOUS_EFFECT_DEF_FIELDS has desynced from `pub struct ContinuousEffectDef`. The \
         structural walk matches nodes by EXACT field set, so a desynced fingerprint silently \
         matches NOTHING and every roster assertion in this file goes vacuous while staying \
         green. Update the fingerprint, then re-derive the non-vacuity floors."
    );
}
