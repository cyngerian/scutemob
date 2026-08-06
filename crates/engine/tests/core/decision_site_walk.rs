//! PB-DP10 canonical decision-site walk — shared by `decision_gate.rs` and (by rewire)
//! `effect_choose_gate.rs` / `pb_rs1_roster_sweep.rs`.
//!
//! ## Why this file exists
//!
//! `docs/audits/decision-point-audit.md` §3.1 counts 21 rows / 277 defs where the engine
//! makes a choice the CR gives to the player, but nothing machine-checks that figure and
//! it grows silently with every card authored. This file supplies the walk; `decision_gate.rs`
//! supplies the frozen baseline and the tests.
//!
//! ## The unit-variant hole (the batch's headline technical finding)
//!
//! serde's default external tagging makes a **struct or tuple** `Effect` variant an object
//! key (`Effect::Scry { .. }` → `{"Scry": {..}}`), but a **unit** variant serializes as a
//! bare JSON **string** (`Effect::Proliferate` → `"Proliferate"`, pinned by
//! `decision_gate.rs`'s `T2`). Every walk that predates this one
//! (`effect_choose_gate.rs::contains_key`, `pb_rs1_roster_sweep.rs::contains_key`,
//! `primitives/pb_dp9_effect_choice.rs::roster::json_contains_variant`) matches object keys
//! only, so a verbatim reuse would report **0** for `Effect::Proliferate`'s ~25 `Complete`
//! defs while looking green. [`def_contains_variant`] matches **both** shapes.
//!
//! A bare unit-variant string match risks a false positive: a card's `oracle_text`, `name`,
//! or a granted ability's prose `description` could — by construction of the test, not by
//! any real card in the corpus — equal a variant name exactly. [`PROSE_FIELDS`] denylists
//! the free-text `String` fields reachable from a `CardDefinition`, and a unit-variant string
//! match is suppressed when it is the direct value of a field so named (see `decision_gate.rs`'s
//! `T3`).
//!
//! ## Two compound rows
//!
//! `triggered_targets` (CR 603.3d) and `modal_trigger` (CR 603.3c) are not expressible as a
//! variant-name match: they are `AbilityDefinition::Triggered` nodes qualified by a *field*
//! of that same node (`targets` non-empty / `modes` non-null). [`find_variant_nodes`] returns
//! every subtree keyed by a variant name so a caller can inspect fields of the matched node
//! itself, rather than merely asking whether the key exists anywhere in the tree.
//!
//! ## No `#[serde(skip)]` (checked, PB-DP9's own note repeated)
//!
//! `crates/card-types/src/cards/card_definition.rs` and `crates/card-types/src/state/game_object.rs`
//! carry no `#[serde(skip)]` / `skip_serializing_if` attribute. Nothing is hidden from this walk.

use mtg_engine::CardDefinition;
use serde_json::Value;
use std::collections::BTreeSet;

// ── The walk ─────────────────────────────────────────────────────────────────

/// Free-text `String` (or `Option<String>`) fields reachable from a `CardDefinition`, plus
/// the three `Completeness` payload tags. A bare unit-variant string match under one of
/// these keys is suppressed — a card's oracle text or a marker's note literally spelling a
/// variant name is not a false positive.
///
/// `first_name` / `second_name` are NOT reachable from `CardDefinition` today (they exist
/// only on `CardRegistry`'s duplicate-id error, checked by source grep) — kept here anyway
/// per the plan's list; a denylist entry that never matches costs nothing and is cheaper to
/// keep than to prove absent every time the DSL grows.
pub const PROSE_FIELDS: &[&str] = &[
    "name",
    "oracle_text",
    "subtype",
    "prompt",
    "first_name",
    "second_name",
    "has_name",
    "card_id",
    "description",
    // Review finding PB-DP10 #6: `string_field_name` (`decision_gate.rs`'s `T13`) originally
    // recognized only literal `String`/`Option<String>`/`Vec<String>` field types, but
    // `SubType` and `CardId` are BOTH newtype-over-`String` structs (`SubType(pub String)`,
    // `CardId(pub String)` -- the only two such types in `crates/card-types/src`) and serde
    // serializes a single-field newtype struct transparently, i.e. as the same bare JSON
    // string a literal `String` field would produce. The eight entries below are every
    // `SubType`/`CardId`-typed field T13 now finds reachable from `CardDefinition` (across
    // `cards/card_definition.rs`, `state/types.rs`, `state/replacement_effect.rs`) that was
    // NOT already covered by an existing entry above.
    "pair_card_id",
    "melded_card_id",
    "onto_subtype",
    "has_subtype",
    "has_subtypes",
    "exclude_subtypes",
    "spell_subtype_filter",
    "default",
    "Inert",
    "Partial",
    "KnownWrong",
];

fn walk_contains(v: &Value, variant: &str, parent_key: Option<&str>) -> bool {
    match v {
        Value::Object(map) => map
            .iter()
            .any(|(k, child)| k == variant || walk_contains(child, variant, Some(k.as_str()))),
        Value::Array(items) => items.iter().any(|i| walk_contains(i, variant, parent_key)),
        Value::String(s) => {
            s == variant
                && !parent_key
                    .map(|k| PROSE_FIELDS.contains(&k))
                    .unwrap_or(false)
        }
        _ => false,
    }
}

/// Does this serialized subtree contain an externally-tagged variant named `variant`,
/// either as an object key (struct/tuple variant) or as a bare string (unit variant, not
/// suppressed by [`PROSE_FIELDS`])?
pub fn json_contains_variant(v: &Value, variant: &str) -> bool {
    walk_contains(v, variant, None)
}

/// [`json_contains_variant`] against a whole `CardDefinition`.
pub fn def_contains_variant(def: &CardDefinition, variant: &str) -> bool {
    let json = serde_json::to_value(def).expect("CardDefinition serializes");
    json_contains_variant(&json, variant)
}

fn collect_nodes<'a>(v: &'a Value, variant: &str, out: &mut Vec<&'a Value>) {
    match v {
        Value::Object(map) => {
            for (k, child) in map {
                if k == variant {
                    out.push(child);
                }
                collect_nodes(child, variant, out);
            }
        }
        Value::Array(items) => {
            for item in items {
                collect_nodes(item, variant, out);
            }
        }
        _ => {}
    }
}

/// Every subtree keyed by `variant` (struct/tuple variants only — a unit variant has no
/// subtree to return, only a bare string). Used by the two compound rows to inspect a
/// matched node's own fields.
pub fn find_variant_nodes<'a>(v: &'a Value, variant: &str) -> Vec<&'a Value> {
    let mut out = Vec::new();
    collect_nodes(v, variant, &mut out);
    out
}

fn triggered_node_has_nonempty_targets(node: &Value) -> bool {
    node.get("targets")
        .and_then(|t| t.as_array())
        .map(|a| !a.is_empty())
        .unwrap_or(false)
}

fn triggered_node_has_modes(node: &Value) -> bool {
    node.get("modes").map(|m| !m.is_null()).unwrap_or(false)
}

/// Row 1 (`triggered_targets`, CR 603.3d): does `json` contain an
/// `AbilityDefinition::Triggered` node whose OWN `targets` is non-empty? Qualified per-node,
/// so a def with a targeted `Activated` ability and a separate, untargeted `Triggered`
/// ability does not match (unlike a file-level regex conjunction).
pub fn def_has_targeted_triggered_ability(json: &Value) -> bool {
    find_variant_nodes(json, "Triggered")
        .into_iter()
        .any(triggered_node_has_nonempty_targets)
}

/// Row 12 (`modal_trigger`, CR 603.3c): does `json` contain an `AbilityDefinition::Triggered`
/// node whose own `modes` is non-null?
pub fn def_has_modal_triggered_ability(json: &Value) -> bool {
    find_variant_nodes(json, "Triggered")
        .into_iter()
        .any(triggered_node_has_modes)
}

// ── The taxonomy ─────────────────────────────────────────────────────────────

/// What kind of decision-site row this is, per `docs/audits/decision-point-audit.md` §3.1
/// and this batch's plan §3.
#[derive(Clone, Copy)]
pub enum DecisionClass {
    /// A real hook exists for the decision THIS row counts. `residual` names the seeds for
    /// other, still-unserved decisions on the SAME variant (e.g. `SearchLibrary`'s "which
    /// card" is served; CR 701.23h's `reveal` residual is not).
    Served {
        by: &'static str,
        residual: &'static [&'static str],
    },
    /// The engine still picks. A `Complete` def hitting this row must be in `BASELINE` or
    /// carry a non-`Complete` marker.
    AutoChosen {
        why_not_flagged_is_wrong: &'static str,
    },
    /// Barred from `Complete` entirely by the SR-33 family (`effect_choose_gate.rs`). Its
    /// own gate holds a hard zero; this row exists so `T14` can prove the two tables agree.
    Gated { by: &'static str },
    /// The row is real (named in the audit) but there is no player choice to hook — the
    /// engine's behavior IS the only legal behavior.
    NoDecision { why: &'static str },
}

/// One of the 22 decision-site rows.
pub struct Row {
    pub id: &'static str,
    pub cr: &'static str,
    pub site: &'static str,
    pub class: DecisionClass,
    pub predicate: fn(&Value) -> bool,
}

fn p_triggered_targets(v: &Value) -> bool {
    def_has_targeted_triggered_ability(v)
}
fn p_search_library(v: &Value) -> bool {
    json_contains_variant(v, "SearchLibrary")
}
fn p_proliferate(v: &Value) -> bool {
    json_contains_variant(v, "Proliferate")
}
fn p_discard_cards(v: &Value) -> bool {
    json_contains_variant(v, "DiscardCards")
}
fn p_wheel_hand(v: &Value) -> bool {
    json_contains_variant(v, "WheelHand")
}
fn p_scry(v: &Value) -> bool {
    json_contains_variant(v, "Scry")
}
fn p_sacrifice_permanents(v: &Value) -> bool {
    json_contains_variant(v, "SacrificePermanents")
}
fn p_may_pay_then_effect(v: &Value) -> bool {
    json_contains_variant(v, "MayPayThenEffect")
}
fn p_choose_color_or_type(v: &Value) -> bool {
    json_contains_variant(v, "ChooseColor") || json_contains_variant(v, "ChooseCreatureType")
}
fn p_look_at_top_or_route(v: &Value) -> bool {
    json_contains_variant(v, "LookAtTopThenPlace") || json_contains_variant(v, "RevealAndRoute")
}
fn p_surveil(v: &Value) -> bool {
    json_contains_variant(v, "Surveil")
}
fn p_counter_unless_pays(v: &Value) -> bool {
    json_contains_variant(v, "CounterUnlessPays")
}
fn p_modal_trigger(v: &Value) -> bool {
    def_has_modal_triggered_ability(v)
}
fn p_change_targets(v: &Value) -> bool {
    json_contains_variant(v, "ChangeTargets")
}
fn p_put_on_library(v: &Value) -> bool {
    json_contains_variant(v, "PutOnLibrary")
}
fn p_bolster_amass(v: &Value) -> bool {
    json_contains_variant(v, "Bolster") || json_contains_variant(v, "Amass")
}
fn p_connive(v: &Value) -> bool {
    json_contains_variant(v, "Connive")
}
fn p_discover(v: &Value) -> bool {
    // Accepted collision (T12): also matches AbilityDefinition::Keyword(KeywordAbility::Discover)
    // (a unit variant nested under "Keyword"), which is correct — that keyword's own doc
    // says the actual discover action is invoked via Effect::Discover, so a def carrying just
    // the marker really does reach the same auto-cast site.
    json_contains_variant(v, "Discover")
}
fn p_may_pay_or_else(v: &Value) -> bool {
    json_contains_variant(v, "MayPayOrElse")
}
fn p_add_mana_filter_choice(v: &Value) -> bool {
    json_contains_variant(v, "AddManaFilterChoice")
}
fn p_choose_stub(v: &Value) -> bool {
    json_contains_variant(v, "Choose")
}
fn p_the_ring_tempts_you(v: &Value) -> bool {
    json_contains_variant(v, "TheRingTemptsYou")
}

/// All 22 §3.1 rows, classified against the engine as it exists on this branch (plan §3).
/// The table has 22 entries because the audit's row 4 ("DiscardCards ... `WheelHand`")
/// splits into two runtime predicates with two different classes: `discard_cards` was
/// AUTO-chosen until ENG-1 (2026-08-02) served it via `EffectChoiceQuestion::Discard`
/// (CR 701.9b); `wheel_hand` discards the WHOLE hand so the pick order is unobservable —
/// NO-DECISION, not AUTO, and is unaffected by ENG-1.
pub static ROWS: &[Row] = &[
    Row {
        id: "triggered_targets",
        cr: "603.3d",
        site: "abilities.rs::flush_pending_triggers / handle_choose_trigger_targets",
        class: DecisionClass::Served {
            by: "PB-DP8",
            residual: &[],
        },
        predicate: p_triggered_targets,
    },
    Row {
        id: "search_library",
        cr: "701.23a",
        site: "effects/mod.rs::execute_effect (SearchLibrary) -> EffectChoiceQuestion::SearchLibrary",
        class: DecisionClass::Served {
            by: "PB-DP9",
            residual: &["OOS-DP9-9", "OOS-DP9-3"],
        },
        predicate: p_search_library,
    },
    Row {
        id: "proliferate",
        cr: "701.34a",
        site: "effects/mod.rs (Proliferate) -- auto-selects all eligible",
        class: DecisionClass::AutoChosen {
            why_not_flagged_is_wrong:
                "the effect auto-selects every eligible permanent/player; CR 701.34a gives the choice to the player",
        },
        predicate: p_proliferate,
    },
    Row {
        id: "discard_cards",
        cr: "701.9 / 701.9b",
        site: "effects/mod.rs::execute_effect (DiscardCards) -> EffectChoiceQuestion::Discard",
        class: DecisionClass::Served {
            by: "ENG-1",
            residual: &["OOS-ENG1-1", "OOS-ENG1-2", "OOS-ENG1-3", "OOS-ENG1-4"],
        },
        predicate: p_discard_cards,
    },
    Row {
        id: "wheel_hand",
        cr: "701.9",
        site: "effects/mod.rs (WheelHand) -- discards the WHOLE hand",
        class: DecisionClass::NoDecision {
            why: "the whole hand is discarded, so there is no 'which card' pick (CR 701.9b) \
                  to hook. The CR 404.3 graveyard-order choice still exists and the engine \
                  takes it by ascending ObjectId -- that is a separate, uncounted class-B \
                  site (OOS-DP10-10), not this row",
        },
        predicate: p_wheel_hand,
    },
    Row {
        id: "scry",
        cr: "701.22a",
        site: "effects/mod.rs::execute_effect (Scry) -> EffectChoiceQuestion::Scry",
        class: DecisionClass::Served {
            by: "PB-DP9",
            residual: &[],
        },
        predicate: p_scry,
    },
    Row {
        id: "sacrifice_permanents",
        cr: "701.21a",
        site: "effects/mod.rs::sacrifice_permanents_for_player -- n lowest ids",
        class: DecisionClass::AutoChosen {
            why_not_flagged_is_wrong:
                "CR 701.21a: the player controlling the permanents chooses which to sacrifice",
        },
        predicate: p_sacrifice_permanents,
    },
    Row {
        id: "may_pay_then_effect",
        cr: "118.12",
        site: "effects/mod.rs::try_pay_optional_cost -- pays iff affordable",
        class: DecisionClass::AutoChosen {
            why_not_flagged_is_wrong:
                "a deterministic-but-legal \"pay when able\" path (CR 118.12) is still the engine choosing on the player's behalf whether to pay",
        },
        predicate: p_may_pay_then_effect,
    },
    Row {
        id: "choose_color_or_type",
        cr: "614.12a (as-enters, ReplacementModification) / 608.2d (resolution-time Effect)",
        site: "effects/mod.rs (ChooseCreatureType) + replacement.rs (ChooseColor) -- most common subtype/color among controller's own permanents",
        class: DecisionClass::AutoChosen {
            why_not_flagged_is_wrong:
                "the card asks the controller to choose a color or creature type; the engine picks the most common one it already controls",
        },
        predicate: p_choose_color_or_type,
    },
    Row {
        id: "look_at_top_or_route",
        cr: "608.2d",
        site: "effects/mod.rs (LookAtTopThenPlace / RevealAndRoute) -- optional destructured away / deterministic routing",
        class: DecisionClass::AutoChosen {
            why_not_flagged_is_wrong:
                "LookAtTopThenPlace's `optional` field is inert by construction (OOS-DP10-5); \
                 RevealAndRoute covers BOTH real CR 608.2d/401.4 order choices (Goblin \
                 Ringleader's Goblins 'in any order') AND defs whose routing the card itself \
                 determines with no choice at all (Chaos Warp, Coiling Oracle: reveal one \
                 card, deterministic destination on both branches) -- this row's count is \
                 therefore an UPPER BOUND on real decisions, not an exact one (carried into \
                 OOS-DP10-6's successor-queue ranking as a caveat)",
        },
        predicate: p_look_at_top_or_route,
    },
    Row {
        id: "surveil",
        cr: "701.25a",
        site: "effects/mod.rs::execute_effect (Surveil) -> EffectChoiceQuestion::Surveil",
        class: DecisionClass::Served {
            by: "PB-DP9",
            residual: &[],
        },
        predicate: p_surveil,
    },
    Row {
        id: "counter_unless_pays",
        cr: "118.12a",
        site: "effects/mod.rs (CounterUnlessPays) -- delegates straight to CounterSpell",
        class: DecisionClass::AutoChosen {
            why_not_flagged_is_wrong:
                "the controller never gets to pay; the engine always counters",
        },
        predicate: p_counter_unless_pays,
    },
    Row {
        id: "modal_trigger",
        cr: "603.3c",
        site: "abilities.rs -- modes_chosen = vec![0] in both the min_modes==0 and !=0 arms",
        class: DecisionClass::AutoChosen {
            why_not_flagged_is_wrong:
                "a modal TRIGGERED ability's mode is chosen by the engine, not announced by the controller (contrast PB-DP3, which fixed modal SPELLS/activated abilities)",
        },
        predicate: p_modal_trigger,
    },
    Row {
        id: "change_targets",
        cr: "115.7d",
        site: "effects/mod.rs (ChangeTargets) delegates to rules::retarget::plan_target_change (PB-DX25c) -- always declines when optional; lowest-id LEGAL candidate when must_change",
        class: DecisionClass::AutoChosen {
            why_not_flagged_is_wrong:
                "CR 115.7d gives the choice of new target(s) to a player; the engine never changes an optional retarget and picks the lowest-id legal candidate for a mandatory one (CR 115.7a legality is delegated to casting::validate_targets_inner, PB-DX25c)",
        },
        predicate: p_change_targets,
    },
    Row {
        id: "put_on_library",
        cr: "608.2d + 401.4",
        site: "effects/mod.rs (PutOnLibrary) -- sort_by_key(id), truncate(n)",
        class: DecisionClass::AutoChosen {
            why_not_flagged_is_wrong:
                "which N cards go back, and in what order, is the player's choice (CR 608.2d/401.4); the engine picks by ascending ObjectId",
        },
        predicate: p_put_on_library,
    },
    Row {
        id: "bolster_amass",
        cr: "701.39a / 701.47a",
        site: "effects/mod.rs (Bolster / Amass) -- least toughness / smallest id, tie-broken by id",
        class: DecisionClass::AutoChosen {
            why_not_flagged_is_wrong:
                "Bolster/Amass name the qualifying creature by a game-state property, but among ties the CONTROLLER chooses (CR 701.39a/701.47a), not the engine",
        },
        predicate: p_bolster_amass,
    },
    Row {
        id: "connive",
        cr: "701.50a",
        site: "effects/mod.rs (Connive) -- inlined discard, min_by_key(id)",
        class: DecisionClass::AutoChosen {
            why_not_flagged_is_wrong:
                "CR 701.9b's default discard-choice rule applies to Connive's discard half the same as it does to plain DiscardCards",
        },
        predicate: p_connive,
    },
    Row {
        id: "discover",
        cr: "701.57a",
        site: "effects/mod.rs::execute_effect -> copy::resolve_discover -- always casts",
        class: DecisionClass::AutoChosen {
            why_not_flagged_is_wrong:
                "CR 701.57a: \"you may cast that card\"; the engine always casts it",
        },
        predicate: p_discover,
    },
    Row {
        id: "may_pay_or_else",
        cr: "118.12a",
        site: "effects/mod.rs (MayPayOrElse) -- stub, always or_else",
        class: DecisionClass::Gated {
            by: "effect_choose_gate.rs::no_complete_def_uses_the_may_pay_or_else_stub",
        },
        predicate: p_may_pay_or_else,
    },
    Row {
        id: "add_mana_filter_choice",
        cr: "605.1a",
        site: "effects/mod.rs (AddManaFilterChoice) -- always one of each colour; AA/BB unreachable",
        class: DecisionClass::AutoChosen {
            why_not_flagged_is_wrong:
                "held at 0 `Complete` defs by hand-authoring discipline alone until this batch's T7 (§4 note 5) -- the SR-33 gate bars a DIFFERENT key (`AddManaChoice`) and does not reach this one",
        },
        predicate: p_add_mana_filter_choice,
    },
    Row {
        id: "choose_stub",
        cr: "700.2",
        site: "effects/mod.rs (Choose) -- stub, always choices.first()",
        class: DecisionClass::Gated {
            by: "effect_choose_gate.rs::no_complete_def_uses_the_choose_stub",
        },
        predicate: p_choose_stub,
    },
    Row {
        id: "the_ring_tempts_you",
        cr: "701.54a",
        site: "effects/mod.rs -> engine.rs::handle_ring_tempts_you -- ring-bearer = lowest id",
        class: DecisionClass::AutoChosen {
            why_not_flagged_is_wrong:
                "held at 0 `Complete` defs by hand-authoring discipline alone until this batch's T7 -- nothing else holds this zero",
        },
        predicate: p_the_ring_tempts_you,
    },
];

/// The set of row ids `def` hits, across ALL 22 rows regardless of class.
pub fn row_hits(def: &CardDefinition) -> BTreeSet<&'static str> {
    let json = serde_json::to_value(def).expect("CardDefinition serializes");
    ROWS.iter()
        .filter(|r| (r.predicate)(&json))
        .map(|r| r.id)
        .collect()
}

/// The set of row ids `def` hits, restricted to `AutoChosen` rows. This is the set
/// `decision_gate.rs`'s `BASELINE` records per def.
pub fn auto_chosen_row_hits(def: &CardDefinition) -> BTreeSet<&'static str> {
    let json = serde_json::to_value(def).expect("CardDefinition serializes");
    ROWS.iter()
        .filter(|r| matches!(r.class, DecisionClass::AutoChosen { .. }))
        .filter(|r| (r.predicate)(&json))
        .map(|r| r.id)
        .collect()
}

/// True if `def` is effectively `Complete` (the runtime form of the audit's file-glob OR --
/// `Complete` is `#[default]`, so this needs no OR (plan §4 note 10)).
pub fn is_effectively_complete(def: &CardDefinition) -> bool {
    def.completeness == mtg_engine::cards::Completeness::Complete
}
