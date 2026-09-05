//! PB-DX57 (`OOS-DX28-5`): the ONE enumeration of the `AbilityDefinition` variants
//! that declare `targets`, derived from the enum's own declaration.
//!
//! # The seed
//!
//! > A gate that walks a hand-written list of `AbilityDefinition` variants cannot see a
//! > member using a variant the list omits, and is green until one does.
//! > `pb_dx28_chosen_object_roster.rs`'s R3 enumerated `["Triggered", "Spell", "Activated"]`.
//! > `Connive // Concoct`'s Concoct half is an `AbilityDefinition::Fuse` — a split card's
//! > half — so R3 reported "R1 found a ChosenObject but no node carries it" and could not
//! > distinguish a migration it could not SEE from one that had not happened. **The general
//! > seed: no gate enumerates the `AbilityDefinition` variants that declare `targets`, so
//! > every such walk in the suite is an independent hand-written claim.**
//!
//! # The seed's own instance had ALREADY REGROWN, and that is this module's headline
//!
//! PB-DX28 widened R3 from three variants to six (`Triggered`, `Spell`, `Activated`,
//! `Fuse`, `LoyaltyAbility`, `SagaChapter`) and wrote in R3's doc that the two extra
//! entries were "included for completeness — the point of listing them is that the day one
//! [uses `ChosenObject`], this row sees it."
//!
//! The declaration carries **eight**. R3's six omit `Aftermath` and `Splice`.
//!
//! * `AbilityDefinition::Aftermath.targets` has existed since the variant did. It was
//!   never in any draft of the list — a straight omission, invisible because no corpus
//!   Aftermath half has yet used the effect R3 hunts.
//! * `AbilityDefinition::Splice.targets` **did not exist when PB-DX28 wrote the six**.
//!   PB-DX18 (`OOS-M11-5`, `scutemob-225`) added it for CR 702.47a — the spliced card's
//!   targets are announced as part of the host spell — and nothing in the tree reddened.
//!
//! So the hand-written list went stale **within one batch of being widened, by the ordinary
//! act of authoring a rule**, and the gate that depends on it reported success for a
//! fortnight. That is the seed stated as a measurement rather than as a risk, and it is why
//! this module DERIVES the set instead of pinning a seventh and eighth name into a literal.
//!
//! # Why a derivation and not a longer pinned list
//!
//! A pinned list checked against the declaration (`t7`'s shape in
//! `pb_dx42a_continuous_condition_roster.rs`) is the right repair when the list encodes a
//! JUDGEMENT the declaration does not — there, "which variants query a characteristic
//! layer" is a semantic claim about eight names. Here the list encodes no judgement at all:
//! *"declares a `targets` field"* is a syntactic property of the declaration, so a literal
//! adds a second place to be wrong and nothing else. The derivation cannot desync.
//!
//! What a derivation CAN do is break silently — a parser that returns four names where the
//! truth is eight makes every consumer under-cover exactly as a stale literal would. So the
//! derivation is guarded on **two independent axes plus a floor**, and never on itself:
//!
//! * **Axis 1 (source)** — parse `pub enum AbilityDefinition`'s body out of
//!   `crates/card-types/src/cards/card_definition.rs` and keep the variants carrying a
//!   `targets:` field.
//! * **Axis 2 (corpus)** — serde-walk `all_cards()` and observe which externally-tagged
//!   `AbilityDefinition` variant nodes actually carry a `"targets"` key. Axis 2 is derived
//!   from real data and knows nothing about axis 1's `targets:` regex, so the two can
//!   disagree; `d3` asserts axis 2 ⊆ axis 1 and PRINTS the residual (declared but unused by
//!   the corpus) rather than asserting it empty, because an unused variant is not a defect.
//! * **A raise-only FLOOR** on the count (`MIN_TARGET_DECLARING_VARIANTS`), so a parser that
//!   silently narrows fails. A floor rather than an equality: a NEW target-declaring variant
//!   must be picked up automatically and must not require a test edit — requiring one is
//!   precisely how the six went stale.
//!
//! # What this module does NOT claim
//!
//! It answers *"which `AbilityDefinition` variants declare a `targets` field"*. It does not
//! answer *"which abilities can target"* — a `Static` or a `Keyword` variant can reach a
//! target through the effect it lowers to, and `KeywordAbility::Enchant` carries a
//! CR 303.4a requirement that `casting::enchant_target_to_requirement` synthesises with no
//! `targets` field anywhere (PB-DX20). A walk that needs THAT question needs a different
//! enumeration and must not reach for this one.

use crate::decision_site_walk::find_variant_nodes;
use mtg_engine::all_cards;
use std::collections::{BTreeMap, BTreeSet};

/// Raise-only floor. At the time of writing axis 1 returns **8**: `Activated`, `Triggered`,
/// `Spell`, `LoyaltyAbility`, `SagaChapter`, `Aftermath`, `Splice`, `Fuse`. Recorded in
/// prose so a reader can see drift; asserted as `>=` so growth needs no edit here.
pub const MIN_TARGET_DECLARING_VARIANTS: usize = 8;

/// Raise-only floor on the TOTAL variant count, which is what proves the enum body was
/// parsed at all rather than a fragment of it. 68 at the time of writing.
pub const MIN_ABILITY_DEFINITION_VARIANTS: usize = 60;

fn card_definition_src() -> String {
    let p = std::path::PathBuf::from(env!("CARGO_MANIFEST_DIR"))
        .parent()
        .and_then(std::path::Path::parent)
        .expect("engine manifest dir is <workspace>/crates/engine")
        .join("crates/card-types/src/cards/card_definition.rs");
    std::fs::read_to_string(&p).unwrap_or_else(|e| panic!("{} must be readable: {e}", p.display()))
}

/// Every top-level `AbilityDefinition` variant, paired with whether its body declares a
/// `targets:` field.
///
/// # Parsing notes, each of which a first draft got wrong
///
/// * **Line comments are stripped BEFORE splitting on `,`.** The enum's doc comments are
///   English prose full of commas; splitting first turns `the, it, CR` into "variants".
///   Measured: the naive order yields 204 chunks against a true 68 (`OOS-DX32-6`'s lesson —
///   a text scan that cannot tell code from a comment — arriving at a parser rather than at
///   a gate).
/// * **Depth counts `(` as well as `{`.** Tuple variants exist, and a `,` inside
///   `Keyword(KeywordAbility)`-shaped parentheses is not a variant boundary.
/// * **`#[...]` attribute lines are dropped**, so a `#[serde(rename = "x, y")]` cannot
///   contribute a comma or a false field name.
/// * The `targets:` match requires a preceding `{` or whitespace, so a field named
///   `mode_targets:` or `spell_targets:` does not count as `targets:`.
pub fn ability_definition_variants() -> BTreeMap<String, bool> {
    let src = card_definition_src();
    let at = src.find("pub enum AbilityDefinition {").expect(
        "`pub enum AbilityDefinition {` not found in card_definition.rs — the type was \
                 renamed or moved. Re-derive this parser against wherever it now lives; do NOT \
                 replace it with a hand-written list, which is the defect OOS-DX28-5 names.",
    );
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
    assert!(
        end > body_start,
        "AbilityDefinition's enum body was never closed — the brace walk ran off the end of \
         the file, so every count below would be taken from a truncated body"
    );

    let clean: String = src[body_start..end]
        .lines()
        .map(|l| l.split("//").next().unwrap_or(""))
        .filter(|l| !l.trim_start().starts_with("#["))
        .collect::<Vec<_>>()
        .join("\n");

    let mut chunks: Vec<String> = Vec::new();
    let mut depth = 0i32;
    let mut cur = String::new();
    for ch in clean.chars() {
        match ch {
            '{' | '(' => {
                depth += 1;
                cur.push(ch);
            }
            '}' | ')' => {
                depth -= 1;
                cur.push(ch);
            }
            ',' if depth == 0 => {
                chunks.push(std::mem::take(&mut cur));
            }
            _ => cur.push(ch),
        }
    }
    if !cur.trim().is_empty() {
        chunks.push(cur);
    }

    let mut out = BTreeMap::new();
    for chunk in chunks {
        let t = chunk.trim();
        if t.is_empty() {
            continue;
        }
        let name: String = t
            .chars()
            .take_while(|c| c.is_ascii_alphanumeric() || *c == '_')
            .collect();
        if name.is_empty() || !name.starts_with(|c: char| c.is_ascii_uppercase()) {
            continue;
        }
        let declares_targets = t
            .match_indices("targets:")
            .chain(t.match_indices("targets :"))
            .any(|(i, _)| {
                i == 0
                    || t[..i]
                        .chars()
                        .next_back()
                        .is_some_and(|c| c.is_whitespace() || c == '{')
            });
        out.insert(name, declares_targets);
    }
    out
}

/// **THE enumeration** — the `AbilityDefinition` variants that declare a `targets` field.
///
/// Every walk in this test target that needs to find target-declaring ability nodes must
/// call this rather than write its own list. `pb_dx28_chosen_object_roster::R3` is the
/// consumer the seed was filed about.
pub fn target_declaring_ability_variants() -> BTreeSet<String> {
    ability_definition_variants()
        .into_iter()
        .filter(|(_, declares)| *declares)
        .map(|(name, _)| name)
        .collect()
}

/// Axis 2: which `AbilityDefinition` variant nodes does the CORPUS actually serialize with a
/// `"targets"` key? Derived from data, independently of axis 1's regex.
fn corpus_observed_target_declaring_variants() -> BTreeSet<String> {
    let all_variants: Vec<String> = ability_definition_variants().into_keys().collect();
    let mut observed = BTreeSet::new();
    for def in all_cards() {
        let json = serde_json::to_value(&def).expect("CardDefinition serializes");
        for variant in &all_variants {
            if observed.contains(variant) {
                continue;
            }
            if find_variant_nodes(&json, variant)
                .iter()
                .any(|n| n.get("targets").is_some())
            {
                observed.insert(variant.clone());
            }
        }
    }
    observed
}

// ── D1: the parse reached the whole enum ─────────────────────────────────────

#[test]
fn d1_ability_definition_parse_is_not_vacuous() {
    let all = ability_definition_variants();
    assert!(
        all.len() >= MIN_ABILITY_DEFINITION_VARIANTS,
        "the AbilityDefinition parser found only {} variants (floor {}). Every consumer of \
         `target_declaring_ability_variants` would silently under-cover. Either the enum \
         shrank drastically (say so and lower the floor deliberately) or the parser is \
         reading a fragment of the body.",
        all.len(),
        MIN_ABILITY_DEFINITION_VARIANTS
    );
    // Structural sanity that a comma-splitting bug cannot survive: every parsed name must
    // be a plausible Rust variant identifier. The 204-chunk first draft failed here on
    // `the`, `it` and `CR`, which is how the comment-stripping order was found.
    for name in all.keys() {
        assert!(
            name.chars().next().is_some_and(|c| c.is_ascii_uppercase())
                && name.chars().all(|c| c.is_ascii_alphanumeric() || c == '_'),
            "parsed a non-identifier as an AbilityDefinition variant: {name:?} — the chunk \
             splitter is picking up prose, which means comments are reaching it"
        );
    }
    // And the two halves must both be non-empty: a parser that classifies EVERY variant as
    // target-declaring is as broken as one that classifies none, and a bare `>= 8` floor
    // cannot tell them apart.
    let declaring = all.values().filter(|d| **d).count();
    assert!(
        declaring > 0 && declaring < all.len(),
        "the targets: classifier returned a degenerate split ({declaring} of {} declaring) — \
         it is matching everything or nothing rather than reading the field",
        all.len()
    );
}

// ── D2: the enumeration itself ───────────────────────────────────────────────

#[test]
fn d2_target_declaring_variants_floor() {
    let set = target_declaring_ability_variants();
    assert!(
        set.len() >= MIN_TARGET_DECLARING_VARIANTS,
        "only {} AbilityDefinition variants were derived as target-declaring (floor {}): \
         {set:?}.\nThis is a raise-only floor. If a variant was genuinely REMOVED from the \
         enum, lower the floor deliberately in the same commit and say which. If it was not, \
         the parser has narrowed and every consumer is now under-covering in silence — which \
         is OOS-DX28-5 itself.",
        set.len(),
        MIN_TARGET_DECLARING_VARIANTS
    );
    // Each derived name must really carry the field, checked a second way (per-variant
    // chunk text), so a classifier bug that over-reports is caught as well as one that
    // under-reports. The floor above is blind to over-reporting by construction.
    let src = card_definition_src();
    for name in &set {
        let at = src
            .find(&format!("\n    {name} {{"))
            .or_else(|| src.find(&format!("\n    {name}(")))
            .unwrap_or_else(|| panic!("variant {name} was derived but cannot be re-found"));
        let window = &src[at..(at + 4000).min(src.len())];
        assert!(
            window.contains("targets:"),
            "{name} was classified as target-declaring but no `targets:` occurs within its \
             declaration window — the classifier is attributing a field to the wrong variant"
        );
    }
}

// ── D3: the two axes ─────────────────────────────────────────────────────────

/// Axis 2 (what the corpus serializes) must be a SUBSET of axis 1 (what the declaration
/// says). The residual — declared but never used with targets by any corpus def — is
/// PRINTED, not asserted empty: an unused variant is not a defect, and asserting it empty
/// would make this gate fail on the ordinary act of adding a variant before authoring a
/// card that uses it.
#[test]
fn d3_corpus_observation_is_a_subset_of_the_declaration() {
    let declared = target_declaring_ability_variants();
    let observed = corpus_observed_target_declaring_variants();

    assert!(
        !observed.is_empty(),
        "non-vacuity: no corpus def serializes ANY AbilityDefinition variant node with a \
         `targets` key. Either all_cards() is empty or the JSON walk is not reaching ability \
         nodes, and in either case d3 proves nothing."
    );
    let extra: BTreeSet<&String> = observed.difference(&declared).collect();
    assert!(
        extra.is_empty(),
        "the corpus serializes a `targets` key on AbilityDefinition variant(s) {extra:?} that \
         the source derivation did NOT classify as target-declaring. The two axes disagree, \
         so the source parser is wrong (not the corpus). Fix the parser — a consumer using \
         the derivation would miss exactly these."
    );

    let unused: Vec<&String> = declared.difference(&observed).collect();
    println!(
        "PB-DX57 / OOS-DX28-5 — AbilityDefinition variants declaring `targets`\n\
         \x20 axis 1 (declaration, {}): {declared:?}\n\
         \x20 axis 2 (corpus-observed, {}): {observed:?}\n\
         \x20 declared but unused by any corpus def ({}): {unused:?}",
        declared.len(),
        observed.len(),
        unused.len()
    );
}

// ── D4: the seed's own instance, recorded by execution ───────────────────────

/// PB-DX28's widened six were `["Triggered","Spell","Activated","Fuse","LoyaltyAbility",
/// "SagaChapter"]`. This test asserts, by execution, that the declaration carries MORE than
/// those six — i.e. that the hand-written list this module replaces was already stale when
/// this batch found it — and names the difference so the record is a measurement.
///
/// It is deliberately phrased as *"the historical six are a strict subset"* rather than
/// *"the missing two are Aftermath and Splice"*: the point survives a later variant being
/// added or removed, and pinning the two names would re-create a hand-maintained list inside
/// the module written to remove one.
#[test]
fn d4_the_historical_hand_written_six_were_already_short() {
    const PB_DX28_WIDENED_SIX: &[&str] = &[
        "Triggered",
        "Spell",
        "Activated",
        "Fuse",
        "LoyaltyAbility",
        "SagaChapter",
    ];
    let declared = target_declaring_ability_variants();
    let historical: BTreeSet<String> = PB_DX28_WIDENED_SIX.iter().map(|s| s.to_string()).collect();

    let historical_bogus: Vec<&String> = historical.difference(&declared).collect();
    assert!(
        historical_bogus.is_empty(),
        "PB-DX28's six named {historical_bogus:?}, which the declaration does not classify as \
         target-declaring — this test's historical baseline has itself rotted and must be \
         re-stated rather than trusted"
    );
    let missed: Vec<&String> = declared.difference(&historical).collect();
    assert!(
        !missed.is_empty(),
        "the historical six now cover the whole declared set. That is not a failure of the \
         engine — it means a variant was REMOVED — but it does invalidate this test's record \
         of the seed, so re-state it against whatever the current evidence is."
    );
    println!(
        "PB-DX57 / OOS-DX28-5 — the hand-written list PB-DX28 widened to six omitted \
         {} declared target-declaring variant(s): {missed:?}",
        missed.len()
    );
}

/// **`d5` — the CONSUMER gate.** No walk in this test target may hand-write its own list of
/// target-declaring `AbilityDefinition` variants; it must call
/// `target_declaring_ability_variants()`.
///
/// # Why this test exists
///
/// `d1`–`d4` police the enumeration's DEFINITION and say nothing whatever about its CONSUMERS.
/// The adversarial pass proved it by execution: a SECOND walk added to this same test target
/// with its own six-element list — short by `Splice` and `Fuse`, `d4`'s exact historical error
/// — left every test in the target GREEN, and so did a typed `matches!` version of the same
/// thing. **That is PB-DX50's `r3` (*a gate on a predicate's DEFINITION says nothing about its
/// CONSUMER*) committed inside a batch that cites `r3` in three other files**, and the module
/// doc above claimed *"every walk in this test target that needs to find target-declaring
/// ability nodes calls it"* while nothing enforced the claim. Either make the claim true or
/// narrow it; this is making it true.
///
/// # What it keys on, and the residuals
///
/// A list of ≥3 of the eight variant names, as string literals, within a 400-byte window,
/// outside this file. That over-collects deliberately (over-collection can only make it fire
/// more) and is answered by an allowlist whose reason is re-checked, not by narrowing.
///
/// **Stated residuals**, because a gate that overclaims is this batch's subject:
/// * A walk in a DIFFERENT test target (`primitives`, `rules`, `simulator`) is not scanned.
///   Those targets cannot import from `core` at all — `tests/*/main.rs` may contain only bare
///   `mod x;` lines, so `#[path]` sharing is forbidden and there is no shared test crate — so
///   the enumeration is not even available to them. That is a real gap and it is the reason
///   the tree's established answer (`pb_dp9_effect_choice.rs:2641`) is *keep the copy, document
///   it, cross-check BY VALUE*.
/// * A walk that names its variants through `const`s declared elsewhere, or builds them from
///   `format!`, is invisible to a string-literal scan.
#[test]
fn d5_no_other_walk_in_this_target_hand_writes_the_variant_list() {
    /// `(file, reason)` — re-checked below, because an allowlist whose reason is not checked is
    /// a comment (`OOS-DX52-1`).
    const ALLOWED: &[(&str, &str)] = &[(
        "pb_dx57_ability_target_variants.rs",
        "this file DECLARES the enumeration; d4 quotes PB-DX28's historical six as a baseline",
    )];

    let dir = std::path::PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("tests/core");
    let declared = target_declaring_ability_variants();
    let mut offenders: Vec<String> = Vec::new();
    let mut scanned = 0usize;

    for entry in std::fs::read_dir(&dir)
        .expect("tests/core is readable")
        .flatten()
    {
        let path = entry.path();
        if path.extension().is_none_or(|x| x != "rs") {
            continue;
        }
        let name = path
            .file_name()
            .unwrap_or_default()
            .to_string_lossy()
            .to_string();
        scanned += 1;
        let raw = std::fs::read_to_string(&path).unwrap_or_default();
        // Comments stripped: this file's own doc names six variants in prose, and a scan that
        // reads a comment as code fires on its own documentation (`OOS-DX32-6`).
        let code: String = raw
            .lines()
            .map(|l| l.split("//").next().unwrap_or(""))
            .collect::<Vec<_>>()
            .join("\n");
        let quoted: Vec<usize> = declared
            .iter()
            .flat_map(|v| {
                let needle = format!("\"{v}\"");
                let mut hits = Vec::new();
                let mut from = 0usize;
                while let Some(rel) = code[from..].find(&needle) {
                    hits.push(from + rel);
                    from = from + rel + 1;
                }
                hits
            })
            .collect();
        if quoted.len() < 3 {
            continue;
        }
        let mut sorted = quoted.clone();
        sorted.sort_unstable();
        let clustered = sorted.windows(3).any(|w| w[2] - w[0] < 400);
        if !clustered {
            continue;
        }
        if ALLOWED.iter().any(|(f, _)| *f == name) {
            continue;
        }
        offenders.push(name);
    }

    assert!(
        scanned >= 60,
        "d5 scanned only {scanned} files in tests/core — a walk that reaches nothing reports no \
         offenders"
    );
    // The allowlist's reasons must still be about files that exist and still cluster, or the
    // exemption is protecting nothing and reads as coverage.
    for (f, reason) in ALLOWED {
        assert!(
            dir.join(f).exists(),
            "allowlisted file {f} no longer exists — delete the row"
        );
        assert!(reason.len() > 30, "allowlist row {f} has no stated reason");
    }
    assert!(
        offenders.is_empty(),
        "file(s) {offenders:?} in tests/core hand-write a list of target-declaring \
         AbilityDefinition variants instead of calling \
         `pb_dx57_ability_target_variants::target_declaring_ability_variants()`.\n\
         A hand-written list goes stale the moment the enum grows — measured: PB-DX28's list \
         was widened to six and was short by two within one batch, because PB-DX18 added \
         `Splice.targets` and NOTHING in the tree reddened (the compiler cannot see it, both \
         wire gates exclude the type, and the walk was a literal).\n\
         If a list here is deliberately narrower than the derivation, allowlist it WITH the \
         reason — but read the derivation first: 'I only need Triggered and Spell' is exactly \
         what the six said."
    );
}
