//! PB-DX43 (CR 305.6/305.7): the corpus roster gate for the intrinsic "{T}: Add [symbol]" mana
//! ability every basic-land-typed object has.
//!
//! SR-36: every population here is enumerated from `all_cards()`, never grepped from source.
//!
//! ## Two independent census axes (PB-DX26's durable lesson)
//!
//! **A roster derived from one declaration construct measures that construct.** `pb-plan-DX43.md`
//! §3 found this the hard way: the "payload rule" (walk every `LayerModification` that confers a
//! basic land subtype) finds exactly 5 conferring defs -- and STRUCTURALLY cannot see
//! `awaken_the_woods` or `overlord_of_the_hauntwoods`, which confer a basic land subtype through
//! a `TokenSpec`, not a `LayerModification`, at all. R1 is the payload axis; R2 is the inverse
//! (`TokenSpec`) axis. Neither subsumes the other -- see R1/R2's own doc comments.
//!
//! ## Structural JSON walk (mirrors `pb_dx42a_continuous_condition_roster.rs`)
//!
//! Both `LayerModification`-shaped nodes and `TokenSpec`-shaped nodes are matched by their
//! serialized FIELD SET, not by their parent key, so the walk also reaches nodes nested at
//! arbitrary depth inside `Effect::Repeat`, `Effect::ForEach`, `Effect::Conditional`, etc. (e.g.
//! `awaken_the_woods`' `TokenSpec` is nested inside `Effect::Repeat { effect: Box<Effect::
//! CreateToken> }`).

use mtg_engine::{
    all_cards, calculate_characteristics, enrich_spec_from_def, CardDefinition, CardType,
    Completeness, GameState, GameStateBuilder, ManaAbility, ManaColor, ObjectId, ObjectSpec,
    PlayerId, SubType, TokenSpec, ZoneId,
};
use serde_json::Value;
use std::collections::BTreeSet;

fn p(n: u64) -> PlayerId {
    PlayerId(n)
}

fn find_object(state: &GameState, name: &str) -> ObjectId {
    state
        .objects()
        .iter()
        .find(|(_, o)| o.characteristics.name == name)
        .map(|(&id, _)| id)
        .unwrap_or_else(|| panic!("object '{}' not found", name))
}

// ── CR 305.6's basic land type names ────────────────────────────────────────────────────────

const BASIC_LAND_TYPE_NAMES: [&str; 5] = ["Plains", "Island", "Swamp", "Mountain", "Forest"];

fn is_basic_land_type_str(s: &str) -> bool {
    BASIC_LAND_TYPE_NAMES.contains(&s)
}

fn complete_defs() -> Vec<CardDefinition> {
    all_cards()
        .into_iter()
        .filter(|d| d.completeness == Completeness::Complete)
        .collect()
}

// ── Generic single-key-variant JSON helper ──────────────────────────────────────────────────

/// If `v` is a single-key JSON object (the shape serde produces for a struct/tuple enum
/// variant), returns `(key, payload)`.
fn variant_key(v: &Value) -> Option<(&str, &Value)> {
    match v {
        Value::Object(m) if m.len() == 1 => {
            let (k, val) = m.iter().next().unwrap();
            Some((k.as_str(), val))
        }
        _ => None,
    }
}

// ── R1: the payload-derived conferring population ───────────────────────────────────────────

/// The four `LayerModification` variants that can, structurally, name a land subtype:
/// `AddSubtypes`, `SetLandTypes`, `SetTypeLine` (all carry `SubType` payloads) and
/// `SetCardTypes` (carries only `CardType` -- listed per the plan for completeness, but its
/// payload can never itself name a `SubType`, so `payload_names_basic_land_type` always returns
/// `false` for it; see the `SetCardTypes` arm below).
const LAND_TYPE_CONFERRING_VARIANTS: [&str; 4] =
    ["AddSubtypes", "SetLandTypes", "SetTypeLine", "SetCardTypes"];

fn str_array_names_basic_land_type(payload: &Value) -> bool {
    match payload {
        Value::Array(items) => items
            .iter()
            .any(|it| it.as_str().map(is_basic_land_type_str).unwrap_or(false)),
        _ => false,
    }
}

fn payload_names_basic_land_type(variant: &str, payload: &Value) -> bool {
    match variant {
        "AddSubtypes" | "SetLandTypes" => str_array_names_basic_land_type(payload),
        "SetTypeLine" => payload
            .get("subtypes")
            .map(str_array_names_basic_land_type)
            .unwrap_or(false),
        // `SetCardTypes(OrdSet<CardType>)` -- its payload is a bare array of `CardType`
        // strings ("Land", "Artifact", ...), never a `SubType`. It cannot structurally confer
        // a basic land TYPE (only a card TYPE), so this arm is always false. Kept as an
        // explicit arm (rather than folded into `_`) so the non-match is a decision, not an
        // omission -- SR-5's idiom applied to a test-file match.
        "SetCardTypes" => false,
        _ => false,
    }
}

fn collect_land_type_conferring(v: &Value, card_name: &str, out: &mut BTreeSet<String>) {
    if let Some((k, payload)) = variant_key(v) {
        if LAND_TYPE_CONFERRING_VARIANTS.contains(&k) && payload_names_basic_land_type(k, payload) {
            out.insert(card_name.to_string());
        }
    }
    match v {
        Value::Object(m) => {
            for child in m.values() {
                collect_land_type_conferring(child, card_name, out);
            }
        }
        Value::Array(items) => {
            for item in items {
                collect_land_type_conferring(item, card_name, out);
            }
        }
        _ => {}
    }
}

fn conferring_population() -> BTreeSet<String> {
    let mut out = BTreeSet::new();
    for def in complete_defs() {
        let json = serde_json::to_value(&def).expect("CardDefinition serializes");
        collect_land_type_conferring(&json, &def.name, &mut out);
    }
    out
}

/// The payload-derived conferring population, pinned BY NAME. This is the memo's "5" (measured
/// 2026-08-14): every `Complete` def whose `LayerModification` payload names a basic land
/// subtype. **This axis is structurally blind to `awaken_the_woods` and `overlord_of_the_
/// hauntwoods`** -- see R2, which finds them.
#[test]
fn r1_payload_derived_conferring_population_is_pinned() {
    let actual = conferring_population();
    let expected: BTreeSet<String> = [
        "Blood Moon",
        "Magus of the Moon",
        "Urborg, Tomb of Yawgmoth",
        "Yavimaya, Cradle of Growth",
        "Dryad of the Ilysian Grove",
    ]
    .into_iter()
    .map(String::from)
    .collect();

    assert_eq!(
        actual, expected,
        "the payload-derived (LayerModification) conferring population has changed. If this \
         GREW, name the new member in the close notes and re-check whether it needs a probe in \
         pb_dx43_intrinsic_land_mana.rs. If it SHRANK, a card def lost its conferring static -- \
         re-check the def."
    );
    // NOTE (`/review` Issue 7): a `>= 5` floor here would be DEAD CODE. The `assert_eq!` above
    // already pins an exact 5-element set, so any walk that went vacuous fails there first and a
    // trailing floor can never be the assertion that fires. The first draft carried one; it was
    // deleted rather than left to read as evidence it was not. The genuine non-vacuity guard in
    // this file is R3's `>= 40`, which sits alone with no equality assert in front of it.
}

// ── R2: the inverse (TokenSpec) population ───────────────────────────────────────────────────

/// `TokenSpec`'s serialized field set (`card_definition.rs:4085-4136`), 17 fields, none with
/// `skip_serializing_if` (the `#[serde(default)]` markers only affect DEserialization, per
/// `pb_dx42a_continuous_condition_roster.rs`'s identical note on `ContinuousEffectDef`), so this
/// is a reliable structural fingerprint with no false negatives from an omitted key.
const TOKEN_SPEC_FIELDS: &[&str] = &[
    "name",
    "power",
    "toughness",
    "colors",
    "supertypes",
    "card_types",
    "subtypes",
    "keywords",
    "count",
    "tapped",
    "enters_attacking",
    "mana_color",
    "mana_abilities",
    "activated_abilities",
    "sacrifice_at_end_step",
    "exile_at_end_step",
    "recipient",
];

fn is_token_spec_node(v: &Value) -> bool {
    match v {
        Value::Object(m) => {
            if m.len() != TOKEN_SPEC_FIELDS.len() {
                return false;
            }
            let keys: BTreeSet<&str> = m.keys().map(|k| k.as_str()).collect();
            let expected: BTreeSet<&str> = TOKEN_SPEC_FIELDS.iter().copied().collect();
            keys == expected
        }
        _ => false,
    }
}

fn collect_token_specs<'a>(v: &'a Value, out: &mut Vec<&'a Value>) {
    if is_token_spec_node(v) {
        out.push(v);
    }
    match v {
        Value::Object(m) => {
            for child in m.values() {
                collect_token_specs(child, out);
            }
        }
        Value::Array(items) => {
            for item in items {
                collect_token_specs(item, out);
            }
        }
        _ => {}
    }
}

fn token_spec_node_grants_basic_land_subtype(v: &Value) -> bool {
    let card_types_has_land = v
        .get("card_types")
        .and_then(|c| c.as_array())
        .map(|arr| arr.iter().any(|x| x.as_str() == Some("Land")))
        .unwrap_or(false);
    let subtypes_has_basic = v
        .get("subtypes")
        .map(str_array_names_basic_land_type)
        .unwrap_or(false);
    card_types_has_land && subtypes_has_basic
}

fn inverse_population() -> BTreeSet<String> {
    let mut out = BTreeSet::new();
    for def in complete_defs() {
        let json = serde_json::to_value(&def).expect("CardDefinition serializes");
        let mut nodes = Vec::new();
        collect_token_specs(&json, &mut nodes);
        if nodes
            .iter()
            .any(|n| token_spec_node_grants_basic_land_subtype(n))
        {
            out.insert(def.name.clone());
        }
    }
    out
}

/// The INVERSE population (by printed `TokenSpec`, not `LayerModification` payload) -- the axis
/// that found `awaken_the_woods` and `overlord_of_the_hauntwoods`, which R1's payload rule
/// structurally cannot see because neither confers a subtype through a `LayerModification` at
/// all: they create a TOKEN that already prints the basic land subtype. PB-DX26's durable
/// lesson, reproduced here as a second axis rather than papered over.
#[test]
fn r2_inverse_token_spec_population_is_pinned() {
    let actual = inverse_population();
    let expected: BTreeSet<String> = ["Awaken the Woods", "Overlord of the Hauntwoods"]
        .into_iter()
        .map(String::from)
        .collect();

    assert_eq!(
        actual, expected,
        "the inverse (TokenSpec) conferring population has changed. This is the axis R1 cannot \
         see -- if it grew, name the new member and check whether its TokenSpec's mana_abilities \
         already discharge the intrinsic (R5's shape) or need it (R4's shape)."
    );
    // NOTE (`/review` Issue 7): the `!actual.is_empty()` floor the first draft carried here was
    // dead for the same reason as R1's -- unreachable behind an exact 2-element `assert_eq!`.
    // The desync hazard it described (a field added to `TokenSpec` making `is_token_spec_node`
    // match nothing) is real and is now guarded properly by
    // `token_spec_field_list_matches_the_struct_declaration` below, which compares the constant
    // against the struct's own source rather than hoping a downstream count notices.
}

// ── R3: ability-index neutrality (OOS-DX26-3) ────────────────────────────────────────────────

fn basic_typed_land_defs() -> Vec<CardDefinition> {
    complete_defs()
        .into_iter()
        .filter(|d| d.types.card_types.contains(&CardType::Land))
        .filter(|d| {
            d.types
                .subtypes
                .iter()
                .any(|st| is_basic_land_type_str(&st.0))
        })
        .collect()
}

/// The load-bearing roster row (OOS-DX26-3): for every `Complete` def carrying a basic land
/// subtype in its PRINTED type line -- basics, duals, shocks, triomes, `Dryad Arbor`, ... --
/// building the object via `enrich_spec_from_def`, resolving it on the battlefield, and
/// comparing `mana_abilities` to the BASE spec's own must be an EXACT match: same length, same
/// order. The derivation must add NOTHING to any of them and must move NO existing
/// `Command::TapForMana.ability_index`. A mismatch names both possibilities in its own failure
/// message: either the def is missing a printed colour (so the derivation legitimately adds one
/// -- fix the def, not the gate) or the derivation is over-firing (an engine bug).
#[test]
fn r3_printed_basic_lands_are_ability_index_neutral() {
    let defs = basic_typed_land_defs();
    eprintln!(
        "PB-DX43 R3: {} Complete defs print a basic land subtype in their type line",
        defs.len()
    );
    assert!(
        defs.len() >= 40,
        "non-vacuity floor: only {} Complete defs print a basic land subtype, below the ~46 \
         measured at HEAD -- the population walk has gone vacuous.",
        defs.len()
    );

    for def in &defs {
        let mut all_defs = std::collections::HashMap::new();
        all_defs.insert(def.name.clone(), def.clone());
        let spec = enrich_spec_from_def(ObjectSpec::card(p(1), &def.name), &all_defs);
        let base_abilities: Vec<ManaAbility> = spec.mana_abilities.clone();

        let state = GameStateBuilder::new()
            .add_player(p(1))
            .object(spec.in_zone(ZoneId::Battlefield))
            .build()
            .unwrap_or_else(|e| panic!("state failed to build for '{}': {e:?}", def.name));

        let land_id = find_object(&state, &def.name);
        let chars = calculate_characteristics(&state, land_id).unwrap();
        let resolved: Vec<ManaAbility> = chars.mana_abilities.iter().cloned().collect();

        assert_eq!(
            resolved, base_abilities,
            "'{}' resolved mana_abilities differ from its BASE spec's -- either '{}' is \
             missing a printed colour in its def (chars.subtypes says it has a basic land type \
             the def's own abilities never author, so the derivation legitimately adds it -- \
             fix the def, not this gate) OR the CR 305.6 derivation is over-firing on an \
             already-discharged colour (an engine bug in \
             rules::layers::discharges_intrinsic_mana_ability). base={:?} resolved={:?}",
            def.name, def.name, base_abilities, resolved
        );
    }
}

// ── R4/R5: the two live-wrong TokenSpec defs, fixed for free ────────────────────────────────

/// Extracts every `TokenSpec` embedded anywhere in `def`, by deserializing each JSON node R2's
/// structural walk finds -- NOT by calling `overlord_of_the_hauntwoods`'s private
/// `everywhere_token_spec()` helper (unreachable from this crate) or by hand-copying a literal
/// that could silently drift from the real def. This tracks the actual shipped `CardDefinition`.
fn token_specs_of(def: &CardDefinition) -> Vec<TokenSpec> {
    let json = serde_json::to_value(def).expect("CardDefinition serializes");
    let mut nodes = Vec::new();
    collect_token_specs(&json, &mut nodes);
    nodes
        .into_iter()
        .map(|n| serde_json::from_value(n.clone()).expect("TokenSpec deserializes"))
        .collect()
}

fn object_spec_from_token_spec(owner: PlayerId, ts: &TokenSpec) -> ObjectSpec {
    let mut spec = ObjectSpec::card(owner, &ts.name);
    spec.card_types = ts.card_types.iter().cloned().collect();
    spec.subtypes = ts.subtypes.iter().cloned().collect();
    spec.supertypes = ts.supertypes.iter().cloned().collect();
    spec.colors = ts.colors.iter().cloned().collect();
    spec.power = Some(ts.power);
    spec.toughness = Some(ts.toughness);
    spec.is_token = true;
    spec.mana_abilities = ts.mana_abilities.clone();
    spec.keywords = ts.keywords.iter().cloned().collect();
    spec.activated_abilities = ts.activated_abilities.clone();
    spec.zone = ZoneId::Battlefield;
    spec
}

/// `overlord_of_the_hauntwoods`'s Everywhere token already hand-authors all five basic land
/// subtypes AND all five mana abilities -- the derivation must find every one of them already
/// discharging its colour (D4) and add NOTHING, so the token resolves to EXACTLY five mana
/// abilities, not ten (the double-grant risk R1/D3's idempotence point 2 names explicitly).
#[test]
fn r4_everywhere_token_resolves_to_exactly_five_not_ten() {
    let def = complete_defs()
        .into_iter()
        .find(|d| d.name == "Overlord of the Hauntwoods")
        .expect("Overlord of the Hauntwoods must still be Complete");
    let specs = token_specs_of(&def);
    assert!(
        !specs.is_empty(),
        "no TokenSpec found in Overlord of the Hauntwoods -- the structural walk has gone \
         vacuous or the def no longer creates a token"
    );
    let everywhere = specs
        .iter()
        .find(|ts| ts.name == "Everywhere")
        .expect("an 'Everywhere' TokenSpec must be present");
    assert_eq!(
        everywhere.mana_abilities.len(),
        5,
        "sanity: the printed TokenSpec itself must already hand-author all five mana \
         abilities: {:?}",
        everywhere.mana_abilities
    );

    let obj_spec = object_spec_from_token_spec(p(1), everywhere);
    let state = GameStateBuilder::new()
        .add_player(p(1))
        .object(obj_spec)
        .build()
        .unwrap();
    let token_id = find_object(&state, "Everywhere");
    let chars = calculate_characteristics(&state, token_id).unwrap();

    assert_eq!(
        chars.mana_abilities.len(),
        5,
        "the Everywhere token must resolve to EXACTLY five mana abilities -- the five \
         hand-authored ones each already discharge their own colour's intrinsic (D4), so the \
         derivation must add nothing: {:?}",
        chars.mana_abilities
    );
    for color in [
        ManaColor::White,
        ManaColor::Blue,
        ManaColor::Black,
        ManaColor::Red,
        ManaColor::Green,
    ] {
        assert!(
            chars
                .mana_abilities
                .iter()
                .any(|ma| ma.produces.get(&color).copied() == Some(1)),
            "missing {color:?}: {:?}",
            chars.mana_abilities
        );
    }
}

/// `awaken_the_woods`' Forest Dryad token TokenSpec hand-authors ZERO mana abilities
/// (`mana_abilities: vec![]`) despite printing the Forest subtype -- the 4th live-wrong def the
/// plan's census found, fixed for free by the CR 305.6 derivation. Its token must now HAVE
/// `{T}: Add {G}`.
#[test]
fn r5_forest_dryad_token_gains_tap_add_green_for_free() {
    let def = complete_defs()
        .into_iter()
        .find(|d| d.name == "Awaken the Woods")
        .expect("Awaken the Woods must still be Complete");
    let specs = token_specs_of(&def);
    let dryad_token = specs
        .iter()
        .find(|ts| ts.name == "Forest Dryad")
        .expect("a 'Forest Dryad' TokenSpec must be present");
    assert!(
        dryad_token.mana_abilities.is_empty(),
        "sanity: the printed TokenSpec must still author ZERO mana abilities of its own -- \
         this is the whole point of the deviation this test proves is now closed: {:?}",
        dryad_token.mana_abilities
    );
    assert!(
        dryad_token
            .subtypes
            .contains(&SubType("Forest".to_string())),
        "sanity: the token must print the Forest subtype: {:?}",
        dryad_token.subtypes
    );

    let obj_spec = object_spec_from_token_spec(p(1), dryad_token);
    let state = GameStateBuilder::new()
        .add_player(p(1))
        .object(obj_spec)
        .build()
        .unwrap();
    let token_id = find_object(&state, "Forest Dryad");
    let chars = calculate_characteristics(&state, token_id).unwrap();

    assert_eq!(
        chars.mana_abilities.len(),
        1,
        "the Forest Dryad token must now resolve to exactly one mana ability -- the CR 305.6 \
         intrinsic for its Forest subtype: {:?}",
        chars.mana_abilities
    );
    assert_eq!(
        chars.mana_abilities.front().unwrap().produces,
        imbl::ordmap! { ManaColor::Green => 1 },
        "the derived ability must produce exactly {{G}}: {:?}",
        chars.mana_abilities
    );
}

// ── Fix-cycle gate (PB-DX43 `/review` Issue 8) ────────────────────────────────────────────────

/// `TOKEN_SPEC_FIELDS` must equal `pub struct TokenSpec`'s own declared field set.
///
/// **Why this exists.** `is_token_spec_node` classifies a JSON node by EXACT field-set equality
/// against that hand-maintained constant, short-circuiting on a length mismatch. That is
/// byte-for-byte the construct this project already filed as `OOS-DX28-1`: *"a gate that reports
/// green while checking nothing the moment its subject grows a field."* Add one field to
/// `TokenSpec` and the fingerprint matches **nothing** — R2, R4 and R5 would then walk an empty
/// set and pass vacuously, reporting that the inverse census is clean while examining zero defs.
///
/// The first draft of this file shipped the fragile construct **and** the seed's recommended
/// repair was not applied, which the batch's own `/review` flagged. This is that repair, reusing
/// `pb_dx42a_continuous_condition_roster.rs::t9`'s `declared(..)` closure: read the struct
/// declaration out of the source and compare field NAMES, so a desync fails **by name** with an
/// actionable message rather than by silent vacuity.
///
/// **Revert to watch red**: delete any single entry from `TOKEN_SPEC_FIELDS`, or add a field to
/// `pub struct TokenSpec`.
#[test]
fn token_spec_field_list_matches_the_struct_declaration() {
    let src = std::fs::read_to_string(
        std::path::PathBuf::from(env!("CARGO_MANIFEST_DIR"))
            .parent()
            .and_then(std::path::Path::parent)
            .expect("engine manifest dir is <workspace>/crates/engine")
            .join("crates/card-types/src/cards/card_definition.rs"),
    )
    .expect("card_definition.rs must be readable");

    let at = src
        .find("pub struct TokenSpec {")
        .expect("pub struct TokenSpec not found in card_definition.rs");
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
    let declared: BTreeSet<String> = src[body_start..end]
        .lines()
        .map(|l| l.split("//").next().unwrap_or("").trim())
        .filter_map(|l| l.strip_prefix("pub "))
        .filter_map(|l| l.split(':').next())
        .map(|n| n.trim().to_string())
        .filter(|n| !n.is_empty() && n.chars().all(|c| c.is_ascii_alphanumeric() || c == '_'))
        .collect();

    assert!(
        !declared.is_empty(),
        "the struct-declaration parser found no fields — it has broken, and this gate would \
         otherwise pass vacuously exactly like the construct it exists to protect"
    );

    let pinned: BTreeSet<String> = TOKEN_SPEC_FIELDS.iter().map(|s| s.to_string()).collect();
    assert_eq!(
        pinned, declared,
        "TOKEN_SPEC_FIELDS has desynced from `pub struct TokenSpec`. `is_token_spec_node` matches \
         nodes by EXACT field set, so a desynced fingerprint matches NOTHING and R2/R4/R5 go \
         vacuous while staying green (OOS-DX28-1's class). Update the constant to the declared \
         set, then re-derive R2's expected membership."
    );
}
