//! PB-DX57 (`OOS-DX28-1`): the CLASS census — every hand-maintained field-set / variant-list
//! fingerprint in the test suite, PRINTED, with its pinning status.
//!
//! # The seed, and why a census is the deliverable
//!
//! > A hand-maintained structural fingerprint keyed on EXACT field-set equality goes blind,
//! > corpus-wide and silently, on any field addition. `TARGET_FILTER_FIELDS` recognised a
//! > serialized node as a `TargetFilter` by comparing its key set to a 32-entry `&[&str]`.
//! > Adding `TargetFilter.owner` as the 33rd field stopped it matching **anything** — no
//! > compile error, and a failure message that pointed nowhere near the cause (root-caused only
//! > by diffing against a pristine worktree). **The seed is the CLASS, not this instance**:
//! > nothing has enumerated how many other hand-maintained field-set fingerprints exist in the
//! > suite, and each is a gate that reports green while checking nothing the moment its subject
//! > grows a field.
//!
//! *"Nothing has enumerated how many"* is the whole seed, so the enumeration is the work and
//! this file is where it lives. It is a **test** rather than a memo entry because a memo cannot
//! notice when one of its rows stops being true.
//!
//! # The census
//!
//! **35 members**, from a search space of ~150 classified candidates and ~110 individually
//! named rejected near-misses (needle lists for source-text gates, file/function allowlists,
//! expected-value pins, ratchet ceilings, card-name lists — none of which mirrors a
//! declaration). **22 were UNPINNED; 13 were already pinned** and are the pattern the repairs
//! copy.
//!
//! # The two axes, and which is a ceiling
//!
//! * The **`const`/`static` axis is a CEILING.** Ten grep spellings cover every Rust form in
//!   which a `const` or `static` can hold a string list, and each hit was classified by opening
//!   either the constant or the declaration it claims to mirror.
//! * The **inline axis is a FLOOR.** An inline `for variant in [...]` has no keyword to anchor
//!   on, and **four members are that shape** — including the seed's own `OOS-DX28-5` instance.
//!
//! Two methodology findings from the census are worth carrying, because both are this class
//! happening to the instrument that measures it:
//!
//! * A `static` grep returned **0** while `pub static ROWS` existed, because it anchored on a
//!   bare `static`. That is `OOS-DX20b-5`'s lesson reproduced *inside a census whose subject is
//!   exactly that*, and it was caught by re-running with a second spelling, not by reading.
//! * **A `const` whose TYPE is a struct slice hides its string literals from every `&[&str]`
//!   grep** — `&[Row]`, `&[UnreadField]`, `&[NamingSiteRow]`, `&[ReachRow]`. Four members are
//!   that shape and all four were found only by reading the file.
//!
//! # What this file asserts, and what it cannot
//!
//! It cannot mechanically decide *"is this constant pinned"* — that is a judgement about what a
//! test asserts, and encoding it would mean re-deriving the census from the thing being
//! censused. What it CAN do, and does:
//!
//! * **`c1`** — every recorded row's file still exists and still contains its needle. That is
//!   `OOS-DX52-1` (*an allowlist whose quoted fragment has rotted keeps passing*) applied to
//!   the census itself: a row naming a constant that was renamed or deleted is a row nobody can
//!   re-adjudicate, and it would otherwise sit here reading as coverage.
//! * **`c2`** — a raise-only ratchet on the DERIVED population of slice-typed `const`
//!   declarations across the test tree, so a new one is a deliberate act that has to be
//!   adjudicated into this census rather than joining it in silence. A ratchet on a derived
//!   count is the only mechanical half available; it is stated as a floor on attention, not as
//!   a proof.
//! * **`c3`** — prints the census.

use std::collections::BTreeSet;
use std::path::PathBuf;

/// `(file, needle, what it enumerates, pin status as of PB-DX57)`.
///
/// `needle` is a fragment that must still occur in the file — the constant's name where it has
/// one, or a distinctive fragment of the inline expression where it does not.
type Row = (&'static str, &'static str, &'static str, &'static str);

/// The 22 members that were UNPINNED when PB-DX57 censused them.
const UNPINNED_AT_CENSUS: &[Row] = &[
    (
        "crates/engine/tests/core/pb_dx28_chosen_object_roster.rs",
        "ability_target_shapes",
        "AbilityDefinition variants declaring `targets`",
        "PINNED by PB-DX57 (derived; OOS-DX28-5)",
    ),
    (
        "crates/engine/tests/core/pb_dx28_chosen_object_roster.rs",
        "SIMPLE_TARGET_VARIANTS",
        "unit variants of TargetRequirement",
        "PINNED by PB-DX57 (partition)",
    ),
    (
        "crates/engine/tests/core/pb_dx28_chosen_object_roster.rs",
        "FILTER_TARGET_VARIANTS",
        "payload variants of TargetRequirement",
        "PINNED by PB-DX57 (partition)",
    ),
    (
        "crates/engine/tests/core/pb_dx28_chosen_object_roster.rs",
        "SUPPORTED_ARMS",
        "Effect arms resolve_pending_object_choices walks",
        "PINNED by PB-DX57 (from the function)",
    ),
    (
        "crates/engine/tests/core/pb_dx28_chosen_object_roster.rs",
        "max_cmc_amount",
        "TargetFilter fields filter_matches_object_untargeted does NOT implement",
        "PINNED by PB-DX57 (two-sided)",
    ),
    (
        "crates/engine/tests/primitives/pb_dx20b_enchant_card_type_or.rs",
        "T10_FIELDS",
        "fields of EnchantFilter",
        "PINNED by PB-DX57",
    ),
    (
        "crates/engine/tests/core/pb_dx20b_enchant_line_roster.rs",
        "VARIANT_POPULATION",
        "variants of EnchantTarget",
        "PINNED by PB-DX57",
    ),
    (
        "crates/engine/tests/core/pb_dx39_source_relative_roster.rs",
        "ATTACHED_FILTERS",
        "EffectFilter variants reading the source's attached_to",
        "PINNED by PB-DX57",
    ),
    (
        "crates/engine/tests/core/pb_dx39_source_relative_roster.rs",
        "SOURCE_MOVING_COSTS",
        "Cost variants that move the source off the battlefield",
        "PINNED by PB-DX57 (partition)",
    ),
    (
        "crates/engine/tests/core/pb_dx39_source_relative_roster.rs",
        "CED_KEYS",
        "4 of the 5 fields of ContinuousEffectDef",
        "PINNED by PB-DX57 (SUBSET, deliberately)",
    ),
    (
        "crates/engine/tests/core/pb_dx43_land_type_roster.rs",
        "LAND_TYPE_CONFERRING_VARIANTS",
        "LayerModification variants that can name a land subtype",
        "PINNED by PB-DX57",
    ),
    (
        "crates/engine/tests/core/pb_dx45_may_pay_roster.rs",
        "DECIDABLE_COST_TAGS",
        "Cost variants can_pay_optional_cost decides",
        "PINNED by PB-DX57 (two-sided)",
    ),
    (
        "crates/engine/tests/core/pb_dx36_deals_damage_roster.rs",
        "NEW_TRIGGER_EVENTS",
        "the damage-family TriggerEvent variants",
        "PINNED by PB-DX57 (re-keyed semantically)",
    ),
    (
        "crates/engine/tests/core/pb_rs1_roster_sweep.rs",
        "EFFECTS",
        "the look-at-top Effect family",
        "PINNED by PB-DX57 (subset + stated exclusions)",
    ),
    (
        "crates/engine/tests/primitives/pb_eng2_targets_announced.rs",
        "AbilityActivated",
        "the stack-push GameEvent variants",
        "PINNED by PB-DX57 (cross-target copy)",
    ),
    (
        "crates/engine/tests/core/pb_dx48_announcement_site_roster.rs",
        "PERMANENT_TARGETED_FIELDS",
        "fields of GameEvent::PermanentTargeted",
        "PINNED by PB-DX57",
    ),
    (
        "crates/engine/tests/scripts/unread_init_fields.rs",
        "UNREAD_INIT_FIELDS",
        "InitialState-family fields build_initial_state never reads",
        "PINNED by PB-DX57 (declared-side; read-side residual STATED)",
    ),
    (
        "crates/engine/tests/core/decision_site_walk.rs",
        "pub static ROWS",
        "the decision-carrying Effect variants — WIDEST blast radius in the census",
        "PINNED by PB-DX57 (total classification)",
    ),
    (
        "crates/engine/tests/core/cards2_printed_field_fidelity.rs",
        "ABILITY_COST_KEYWORDS",
        "keyword-ability variants whose printed clause charges mana",
        "PINNED by PB-DX57 (derived + stated exclusions)",
    ),
    (
        "crates/engine/tests/core/pb_dx49_saga_blanking_roster.rs",
        "face_down_making_effect_variants",
        "Effect variants that put a face-down permanent onto the battlefield",
        "PINNED by PB-DX57 (derived from effects/mod.rs)",
    ),
    // The two members outside this batch's permitted diff surface. See `c4`.
    (
        "tools/play-server/src/view.rs",
        "true_label",
        "the complete field set of AnswerShapeView::BinaryChoice",
        "NOT PINNED — in tools/, outside PB-DX57's 0-engine-lines criterion; FILED",
    ),
    (
        "tools/play-server/src/main.rs",
        "landStackKey",
        "the PermanentView fields that make two lands non-interchangeable",
        "NOT PINNED — in tools/, outside PB-DX57's 0-engine-lines criterion; FILED",
    ),
];

/// The 13 that were ALREADY pinned. Recorded because the acceptance criterion asks for the full
/// enumeration, and because each names a mechanism the repairs above copy.
const ALREADY_PINNED_AT_CENSUS: &[Row] = &[
    (
        "crates/engine/tests/core/pb_dx42a_continuous_condition_roster.rs",
        "CONTINUOUS_EFFECT_DEF_FIELDS",
        "fields of ContinuousEffectDef",
        "t9, declared(..)",
    ),
    (
        "crates/engine/tests/core/pb_dx42a_continuous_condition_roster.rs",
        "TARGET_FILTER_FIELDS",
        "fields of TargetFilter — THE SEED'S OWN INSTANCE",
        "t9 second half, added by PB-DX42b",
    ),
    (
        "crates/engine/tests/core/pb_dx43_land_type_roster.rs",
        "TOKEN_SPEC_FIELDS",
        "fields of TokenSpec",
        "token_spec_field_list_matches_the_struct_declaration",
    ),
    (
        "crates/engine/tests/core/pending_trigger_shape.rs",
        "EXPECTED_FIELDS",
        "fields of PendingTrigger",
        "declared_fields()",
    ),
    (
        "crates/engine/tests/core/pb_dx20b_enchant_line_roster.rs",
        "KNOWN_ENCHANT_FILTER_FIELDS",
        "fields of EnchantFilter",
        "r5_every_enchant_filter_field_is_lowered",
    ),
    (
        "crates/engine/tests/core/decision_site_walk.rs",
        "PROSE_FIELDS",
        "string-typed fields reachable from CardDefinition",
        "decision_gate T13",
    ),
    (
        "crates/engine/tests/core/pb_dx39_source_relative_roster.rs",
        "SOURCE_RELATIVE",
        "EffectFilter variants whose arm reads effect.source",
        "r1 derives from layers.rs",
    ),
    (
        "crates/engine/tests/core/pb_dx39_source_view_gates.rs",
        "SOURCE_RELATIVE_ARMS",
        "the same 20, in another test target",
        "r2_source_relative_arms_are_pinned_by_name",
    ),
    (
        "crates/engine/tests/core/pb_dx42a_continuous_condition_roster.rs",
        "NON_FILTER_LAYER_QUERYING",
        "Condition variants in the fixed-TypeChange arm",
        "t7 parses the arm — OOS-ADJ-2's widening, verified by execution in this batch",
    ),
    (
        "crates/engine/tests/core/pb_dx49_saga_blanking_roster.rs",
        "BLANKING_VARIANTS",
        "ability-blanking LayerModification variants",
        "r8, all 33 built and classified",
    ),
    (
        "crates/engine/tests/core/keyword_registry.rs",
        "REVIEWED",
        "KeywordAbility variants classified Marker",
        "marker_keywords_are_the_reviewed_set",
    ),
    (
        "crates/engine/tests/core/ability_definition_registry.rs",
        "REVIEWED",
        "AbilityDefinition variants classified Marker",
        "marker_abilities_are_the_reviewed_set",
    ),
    (
        "crates/engine/tests/core/pb_dx50_copy_additional_cost_roster.rs",
        "EXPECTED_DROPPED",
        "all 15 AdditionalCost variants",
        "r1 — the cleanest instance of the repair in the tree",
    ),
];

/// Raise-only ratchet on the DERIVED population of slice-typed `const` declarations across the
/// test tree. Measured at **229** across 52 files when PB-DX57 censused it.
///
/// A ceiling rather than an equality, and a **floor on attention** rather than a proof: not
/// every slice const is a member of the class (most are needle lists, allowlists or expected
/// values, and the census names ~110 such rejections individually). What it buys is that a new
/// one cannot join in silence — somebody has to look at it and decide.
const SLICE_CONST_CEILING: usize = 240;

fn workspace_root() -> PathBuf {
    PathBuf::from(env!("CARGO_MANIFEST_DIR"))
        .parent()
        .and_then(std::path::Path::parent)
        .expect("engine manifest dir is <workspace>/crates/engine")
        .to_path_buf()
}

fn rows() -> Vec<&'static Row> {
    UNPINNED_AT_CENSUS
        .iter()
        .chain(ALREADY_PINNED_AT_CENSUS.iter())
        .collect()
}

// ── C1: the census cannot rot silently ───────────────────────────────────────

#[test]
fn c1_every_censused_row_still_names_something_that_exists() {
    let root = workspace_root();
    let mut missing_file = Vec::new();
    let mut missing_needle = Vec::new();
    for (file, needle, what, _) in rows() {
        let p = root.join(file);
        let Ok(src) = std::fs::read_to_string(&p) else {
            missing_file.push(*file);
            continue;
        };
        if !src.contains(needle) {
            missing_needle.push((*file, *needle, *what));
        }
    }
    assert!(
        missing_file.is_empty(),
        "censused file(s) no longer exist: {missing_file:?}. A census row naming a file nobody \
         can open is a row nobody can re-adjudicate — delete it and say why, or re-point it."
    );
    assert!(
        missing_needle.is_empty(),
        "censused row(s) no longer contain their needle: {missing_needle:#?}.\n\
         This is OOS-DX52-1 applied to the census itself — an entry whose quoted fragment has \
         rotted keeps passing and reads as coverage. Either the constant was renamed (re-point \
         the row) or it was deleted (delete the row and record that the member is gone)."
    );
    assert!(
        rows().len() >= 35,
        "the census holds {} rows; it held 35 when PB-DX57 measured it. Rows may be added; a \
         row REMOVED needs its reason recorded, because the count is what a later reader trusts.",
        rows().len()
    );
}

// ── C2: the class cannot silently regrow ─────────────────────────────────────

#[test]
fn c2_slice_const_population_is_ratcheted() {
    let root = workspace_root();
    let mut total = 0usize;
    let mut files = BTreeSet::new();
    for group in [
        "crates/engine/tests",
        "crates/simulator/tests",
        "crates/card-types/tests",
        "crates/card-defs/tests",
    ] {
        let dir = root.join(group);
        if !dir.exists() {
            continue;
        }
        let mut stack = vec![dir];
        while let Some(d) = stack.pop() {
            let Ok(rd) = std::fs::read_dir(&d) else {
                continue;
            };
            for e in rd.flatten() {
                let p = e.path();
                if p.is_dir() {
                    stack.push(p);
                    continue;
                }
                if p.extension().is_none_or(|x| x != "rs") {
                    continue;
                }
                let src = std::fs::read_to_string(&p).unwrap_or_default();
                // Comments stripped: a `const` named in a doc comment is prose, not a
                // declaration (`OOS-DX32-6`).
                let code: String = src
                    .lines()
                    .map(|l| l.split("//").next().unwrap_or(""))
                    .collect::<Vec<_>>()
                    .join("\n");
                let mut n = 0usize;
                let mut from = 0usize;
                while let Some(rel) = code[from..].find("const ") {
                    let at = from + rel;
                    from = at + 1;
                    let rest = &code[at + 6..];
                    let name: String = rest
                        .chars()
                        .take_while(|c| c.is_ascii_alphanumeric() || *c == '_')
                        .collect();
                    if name.is_empty()
                        || !name
                            .chars()
                            .all(|c| c.is_ascii_uppercase() || c.is_ascii_digit() || c == '_')
                    {
                        continue;
                    }
                    let after = rest[name.len()..].trim_start();
                    if let Some(ty) = after.strip_prefix(':') {
                        let ty = ty.trim_start();
                        if ty.starts_with('[') || ty.starts_with("&[") || ty.starts_with("& [") {
                            n += 1;
                        }
                    }
                }
                if n > 0 {
                    total += n;
                    files.insert(p.clone());
                }
            }
        }
    }
    assert!(
        total <= SLICE_CONST_CEILING,
        "the test tree now declares {total} slice-typed consts across {} files, above the \
         ratchet of {SLICE_CONST_CEILING} (229 when PB-DX57 censused the class).\n\
         Each new one is a CANDIDATE member of the OOS-DX28-1 class: a hand-maintained list \
         whose correctness depends on staying in sync with a declaration it mirrors. Adjudicate \
         it — if it mirrors a struct's fields or an enum's variants, PIN it against that \
         declaration and add it to UNPINNED_AT_CENSUS with its status; if it is a needle list, \
         an allowlist or an expected value, it is NOT a member and you may raise this ceiling \
         with that reason stated.\n\
         NOTE this is a floor on ATTENTION, not a proof: the inline `for x in [..]` shape has \
         no `const` to count and four members of the census are exactly that.",
        files.len()
    );
    assert!(
        total >= 200,
        "the slice-const scan found only {total}; it found 229 at census. A scan that collapses \
         makes this ratchet vacuous while green."
    );
}

// ── C3: the census, printed ──────────────────────────────────────────────────

#[test]
fn c3_print_the_census() {
    println!(
        "\nPB-DX57 / OOS-DX28-1 — hand-maintained field-set / variant-list fingerprints in the \
         test suite\n\
         =========================================================================\n\
         members: {}   (unpinned at census: {}   already pinned: {})\n",
        rows().len(),
        UNPINNED_AT_CENSUS.len(),
        ALREADY_PINNED_AT_CENSUS.len()
    );
    println!("-- UNPINNED AT CENSUS --");
    for (f, n, what, status) in UNPINNED_AT_CENSUS {
        println!("  {f}\n      {n}  ->  {what}\n      {status}");
    }
    println!("\n-- ALREADY PINNED AT CENSUS (the pattern the repairs copy) --");
    for (f, n, what, how) in ALREADY_PINNED_AT_CENSUS {
        println!("  {f}\n      {n}  ->  {what}\n      via {how}");
    }
}

// ── C4: the two members this batch could not take, stated in the tree ────────

/// Two censused members live in `#[cfg(test)]` modules under `tools/`, and PB-DX57's own
/// acceptance criterion requires `git diff` over `tools/` to be EMPTY. They are therefore
/// **filed, not fixed**, and this test exists so that the gap is recorded where a reader of the
/// census will see it rather than only in a memo.
///
/// Both repairs are cheap and are written down so the next batch does not re-derive them:
/// `view.rs:3761`'s `BinaryChoice` key list can be pinned against the variant's own declaration
/// **in the same file**; `main.rs:9191`'s `landStackKey` field list needs
/// `pub struct PermanentView` from `crates/view-model/src/lib.rs` — a THIRD crate — plus a
/// stated `FUNGIBLE_FIELDS` exclusion list, which is the deliverable there because it turns
/// *"these fields cannot distinguish two lands"* from an unstated assumption into a checked one.
#[test]
fn c4_the_two_unfixed_members_are_named_and_still_present() {
    let root = workspace_root();
    let unfixed: Vec<&Row> = UNPINNED_AT_CENSUS
        .iter()
        .filter(|(_, _, _, status)| status.starts_with("NOT PINNED"))
        .collect();
    assert_eq!(
        unfixed.len(),
        2,
        "PB-DX57 left exactly two censused members unpinned, both in tools/. If that changed, \
         update this test and the seed that records them."
    );
    for (file, needle, _, _) in &unfixed {
        let src = std::fs::read_to_string(root.join(file))
            .unwrap_or_else(|e| panic!("{file} must be readable: {e}"));
        assert!(
            src.contains(needle),
            "{file} no longer contains `{needle}` — the unfixed member was moved or repaired. \
             If repaired, say so and change its status; do not leave a row claiming a gap that \
             is closed."
        );
    }
    println!(
        "PB-DX57 / OOS-DX28-1 — {} censused member(s) deliberately NOT pinned (in tools/, \
         outside this batch's 0-engine-lines criterion): {:?}",
        unfixed.len(),
        unfixed
            .iter()
            .map(|(f, n, _, _)| (f, n))
            .collect::<Vec<_>>()
    );
}
