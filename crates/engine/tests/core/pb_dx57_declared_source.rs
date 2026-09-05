//! PB-DX57 (`OOS-DX28-1`): the ONE declaration parser the field-set / variant-list
//! fingerprints in this test target are pinned against.
//!
//! # The seed
//!
//! > **A hand-maintained structural fingerprint keyed on EXACT field-set equality goes blind,
//! > corpus-wide and silently, on any field addition.** `TARGET_FILTER_FIELDS` recognised a
//! > serialized node as a `TargetFilter` by comparing its key set to a 32-entry `&[&str]`.
//! > Adding `TargetFilter.owner` as the 33rd field stopped it matching **anything** — no
//! > compile error, and a failure message that pointed nowhere near the cause. **The seed is
//! > the CLASS**: nothing has enumerated how many other hand-maintained field-set
//! > fingerprints exist in the suite, and each is a gate that reports green while checking
//! > nothing the moment its subject grows a field.
//!
//! # Why one parser rather than eighteen
//!
//! The stage-0 census enumerated **31** members of the class in this suite: 13 already pinned
//! against their declaration, **18 not**. The repair for all 18 is the same three lines —
//! read the declaration out of source, collect the names, `assert_eq!` — and the tree already
//! held **five independent hand-written copies** of that parser
//! (`pb_dx42a:807`, `pb_dx43:520`, `pending_trigger_shape.rs:277`,
//! `pb_dx20b_enchant_line_roster.rs:1195`, `pb_dx49:2303`), each with its own anchoring rules
//! and its own bugs. Writing a nineteenth would be the same mistake at a larger scale, so the
//! parser lives here once and the pins call it.
//!
//! # The three things a declaration parser gets wrong, all of which are recorded failures
//!
//! * **Comments.** Rust doc comments in these enums are English prose full of commas and of
//!   `Identifier::Variant`-shaped tokens. A parser that splits before stripping `//` reads
//!   prose as declarations — measured on `AbilityDefinition`: **204** "variants" against a
//!   true **68**, including `the`, `it` and `CR`. That is `OOS-DX32-6` (*a text scan cannot
//!   tell code from a comment*) arriving inside a parser rather than inside a gate.
//! * **Nesting.** A struct-like variant's body contains `,` and `{`; a tuple variant's
//!   contains `,` and `(`. `t7`'s own first draft anchored on the nearest `}` and *"landed
//!   INSIDE the pattern list and silently returned three of the eight"*. Depth must count
//!   both bracket kinds.
//! * **Emptiness.** A parser that returns `{}` makes every `assert_eq!` against it trivially
//!   true — the seed's own failure mode re-entering through the fix. **Every function here
//!   panics on an empty result rather than returning it**, so a caller cannot accidentally
//!   compare against nothing. This is stricter than the tree's existing copies, three of which
//!   leave the floor to the caller.
//!
//! # What is NOT here, deliberately
//!
//! There is no `declared_*` helper for a FUNCTION's match arms. Several census members must be
//! pinned against a function body rather than a type (`SUPPORTED_ARMS` against
//! `resolve_pending_object_choices`, `DECIDABLE_COST_TAGS` against `can_pay_optional_cost`),
//! and `pb_dx39_source_relative_roster::r1` already holds a working derivation of that shape.
//! Generalising a match-arm parser is a genuinely harder problem than generalising a
//! declaration parser — arms have guards, `|` alternatives and nested matches — and a
//! half-right shared one would be worse than the specific ones. Stated rather than left as an
//! apparent omission.

use std::collections::BTreeSet;
use std::path::{Path, PathBuf};

/// `crates/card-types/src/cards/card_definition.rs` — where most of the DSL lives.
pub const CARD_DEFINITION_RS: &str = "crates/card-types/src/cards/card_definition.rs";
/// `crates/card-types/src/state/types.rs`.
pub const STATE_TYPES_RS: &str = "crates/card-types/src/state/types.rs";
/// `crates/card-types/src/state/continuous_effect.rs`.
pub const CONTINUOUS_EFFECT_RS: &str = "crates/card-types/src/state/continuous_effect.rs";
/// `crates/card-types/src/state/game_object.rs`.
#[allow(
    dead_code,
    reason = "consumed by pins added in this batch's later stages and by any \
                             future pin against TriggerEvent; declared here so the path \
                             literal is not re-typed per call site"
)]
pub const GAME_OBJECT_RS: &str = "crates/card-types/src/state/game_object.rs";
/// `crates/engine/src/rules/events.rs` — one of the two declaration files that is NOT in
/// `card-types`.
#[allow(dead_code, reason = "as above, for GameEvent")]
pub const EVENTS_RS: &str = "crates/engine/src/rules/events.rs";

/// Read a workspace-relative source file. Panics with the path on failure, because a silent
/// `unwrap_or_default()` here would hand every caller an empty parse.
pub fn read_workspace_file(rel: &str) -> String {
    let root: PathBuf = PathBuf::from(env!("CARGO_MANIFEST_DIR"))
        .parent()
        .and_then(Path::parent)
        .expect("engine manifest dir is <workspace>/crates/engine")
        .to_path_buf();
    let p = root.join(rel);
    std::fs::read_to_string(&p).unwrap_or_else(|e| panic!("{} must be readable: {e}", p.display()))
}

/// Strip `//` line comments and `/* */` block comments.
///
/// Both, not just `//`: PB-DX8's `/* */` defeat is on record — the byte-identical sentence
/// reddened as a line comment and left every test green as a block comment. Line lengths are
/// preserved (comment bytes become spaces) so a caller can still map offsets back to lines.
fn strip_comments(src: &str) -> String {
    let b = src.as_bytes();
    let mut out = String::with_capacity(src.len());
    let mut i = 0usize;
    let mut in_str = false;
    let mut in_char = false;
    while i < b.len() {
        let c = b[i] as char;
        if in_str {
            out.push(c);
            if c == '\\' && i + 1 < b.len() {
                out.push(b[i + 1] as char);
                i += 2;
                continue;
            }
            if c == '"' {
                in_str = false;
            }
            i += 1;
            continue;
        }
        if in_char {
            out.push(c);
            if c == '\\' && i + 1 < b.len() {
                out.push(b[i + 1] as char);
                i += 2;
                continue;
            }
            if c == '\'' {
                in_char = false;
            }
            i += 1;
            continue;
        }
        if c == '"' {
            in_str = true;
            out.push(c);
            i += 1;
            continue;
        }
        if c == '/' && i + 1 < b.len() && b[i + 1] == b'/' {
            while i < b.len() && b[i] != b'\n' {
                out.push(' ');
                i += 1;
            }
            continue;
        }
        if c == '/' && i + 1 < b.len() && b[i + 1] == b'*' {
            let mut depth = 1usize;
            out.push_str("  ");
            i += 2;
            while i < b.len() && depth > 0 {
                if b[i] == b'/' && i + 1 < b.len() && b[i + 1] == b'*' {
                    depth += 1;
                    out.push_str("  ");
                    i += 2;
                } else if b[i] == b'*' && i + 1 < b.len() && b[i + 1] == b'/' {
                    depth -= 1;
                    out.push_str("  ");
                    i += 2;
                } else {
                    out.push(if b[i] == b'\n' { '\n' } else { ' ' });
                    i += 1;
                }
            }
            continue;
        }
        out.push(c);
        i += 1;
    }
    out
}

/// The body of `<kw> <name> {` … `}`, brace-matched, with comments stripped.
fn declaration_body(src: &str, header: &str, what: &str) -> String {
    let clean = strip_comments(src);
    let at = clean.find(header).unwrap_or_else(|| {
        panic!(
            "{what}: `{header}` not found. The declaration was renamed, moved, or its \
             visibility changed. Re-point this pin at wherever it now lives — do NOT delete \
             the pin and keep the hand-written list, which is the defect OOS-DX28-1 names."
        )
    });
    let body_start = clean[at..].find('{').expect("declaration has a body") + at + 1;
    let mut depth = 1usize;
    let mut end = None;
    for (i, ch) in clean[body_start..].char_indices() {
        match ch {
            '{' => depth += 1,
            '}' => {
                depth -= 1;
                if depth == 0 {
                    end = Some(body_start + i);
                    break;
                }
            }
            _ => {}
        }
    }
    let end = end.unwrap_or_else(|| {
        panic!("{what}: the body of `{header}` is never closed — the brace walk ran off the end")
    });
    clean[body_start..end].to_string()
}

/// Split a declaration body into top-level `,`-separated chunks, counting BOTH `{}` and `()`
/// so a struct-like or tuple variant's internal commas are not boundaries, and `<>` so a
/// `Vec<A, B>`-shaped generic cannot split either.
fn top_level_chunks(body: &str) -> Vec<String> {
    let mut out = Vec::new();
    let mut depth = 0i32;
    let mut cur = String::new();
    let mut prev = ' ';
    for ch in body.chars() {
        match ch {
            '{' | '(' | '[' => {
                depth += 1;
                cur.push(ch);
            }
            '}' | ')' | ']' => {
                depth -= 1;
                cur.push(ch);
            }
            // `->` is not a generic open; `=>` is not either. Only treat `<` as a bracket when
            // it follows an identifier character, which is how a generic argument list opens.
            '<' if prev.is_ascii_alphanumeric() || prev == '_' => {
                depth += 1;
                cur.push(ch);
            }
            '>' if depth > 0 && prev != '-' && prev != '=' => {
                depth -= 1;
                cur.push(ch);
            }
            ',' if depth == 0 => out.push(std::mem::take(&mut cur)),
            _ => cur.push(ch),
        }
        prev = ch;
    }
    if !cur.trim().is_empty() {
        out.push(cur);
    }
    out
}

/// Remove any leading `#[...]` attributes (possibly several, possibly nested) from a
/// declaration chunk.
///
/// Load-bearing: a `#[serde(default)]`, `#[serde(rename = "x")]` or `#[serde(skip)]` sits
/// between a field's doc comment and the field itself throughout this DSL, and a parser that
/// reads the chunk's first identifier without removing it reads `serde`, or nothing at all.
fn strip_leading_attributes(mut s: &str) -> &str {
    loop {
        s = s.trim_start();
        if !s.starts_with("#[") {
            return s;
        }
        let mut depth = 0usize;
        let mut end = None;
        for (i, ch) in s.char_indices() {
            match ch {
                '[' => depth += 1,
                ']' => {
                    depth -= 1;
                    if depth == 0 {
                        end = Some(i + 1);
                        break;
                    }
                }
                _ => {}
            }
        }
        match end {
            Some(e) => s = &s[e..],
            None => return s,
        }
    }
}

fn leading_identifier(chunk: &str) -> String {
    strip_leading_attributes(chunk)
        .chars()
        .take_while(|c| c.is_ascii_alphanumeric() || *c == '_')
        .collect()
}

/// Every variant name declared by `pub enum <enum_name>` in the given workspace-relative file.
///
/// Panics if the parse is empty — see the module doc.
pub fn declared_enum_variants(rel: &str, enum_name: &str) -> BTreeSet<String> {
    let src = read_workspace_file(rel);
    let body = declaration_body(
        &src,
        &format!("pub enum {enum_name} {{"),
        &format!("declared_enum_variants({rel}, {enum_name})"),
    );
    let out: BTreeSet<String> = top_level_chunks(&body)
        .into_iter()
        .filter_map(|chunk| {
            let name = leading_identifier(&chunk);
            (!name.is_empty() && name.starts_with(|c: char| c.is_ascii_uppercase())).then_some(name)
        })
        .collect();
    assert!(
        !out.is_empty(),
        "declared_enum_variants({rel}, {enum_name}) parsed ZERO variants. Every assert_eq! \
         against this set would be trivially satisfiable, which is OOS-DX28-1's own failure \
         mode re-entering through its fix."
    );
    out
}

/// Every variant of `pub enum <enum_name>`, paired with the set of field names its
/// struct-like payload declares (empty for unit and tuple variants).
///
/// This is what a pin like *"the variants that declare `targets`"* or *"the variants that
/// declare `cost: ManaCost`"* must be derived from.
pub fn declared_enum_variant_fields(
    rel: &str,
    enum_name: &str,
) -> std::collections::BTreeMap<String, BTreeSet<String>> {
    let src = read_workspace_file(rel);
    let body = declaration_body(
        &src,
        &format!("pub enum {enum_name} {{"),
        &format!("declared_enum_variant_fields({rel}, {enum_name})"),
    );
    let mut out = std::collections::BTreeMap::new();
    for chunk in top_level_chunks(&body) {
        let name = leading_identifier(&chunk);
        if name.is_empty() || !name.starts_with(|c: char| c.is_ascii_uppercase()) {
            continue;
        }
        let fields: BTreeSet<String> = match (chunk.find('{'), chunk.rfind('}')) {
            (Some(a), Some(b)) if b > a => top_level_chunks(&chunk[a + 1..b])
                .into_iter()
                .filter_map(|f| {
                    let f = strip_leading_attributes(f.trim());
                    let f = f.strip_prefix("pub ").unwrap_or(f).trim_start();
                    let ident: String = f
                        .chars()
                        .take_while(|c| c.is_ascii_alphanumeric() || *c == '_')
                        .collect();
                    // A field is `name: Type`. Anything else in a variant body is an
                    // attribute, which `strip_leading_attributes` has removed above --
                    // and NOT removing it was this parser's first bug: five of the eight
                    // `AbilityDefinition` variants declaring `targets` carry a
                    // `#[serde(default)]` immediately above the field, so the naive
                    // version read `#` as the field's first character and dropped all
                    // five. It reported 3 where the truth is 8, and `p4`'s cross-check
                    // against the sibling derivation is what caught it -- on its first
                    // run, before any of this was written down.
                    (!ident.is_empty()
                        && f[ident.len()..].trim_start().starts_with(':')
                        && ident.starts_with(|c: char| c.is_ascii_lowercase() || c == '_'))
                    .then_some(ident)
                })
                .collect(),
            _ => BTreeSet::new(),
        };
        out.insert(name, fields);
    }
    assert!(
        !out.is_empty(),
        "declared_enum_variant_fields({rel}, {enum_name}) parsed ZERO variants"
    );
    out
}

/// Every `pub enum` declared in the given workspace-relative file, mapped to its variant
/// names.
///
/// Used by the `OOS-DX28-6` mechanism-note ratchet to answer *"is this `X::Y` token in an
/// author's comment a real declared identifier, or prose that happens to look like one"* —
/// which must be decided against the DECLARATION and never against corpus usage, or the
/// dictionary is learned from the thing being checked and cannot disagree with it (PB-DX8).
pub fn declared_enums_in(rel: &str) -> std::collections::BTreeMap<String, BTreeSet<String>> {
    let src = strip_comments(&read_workspace_file(rel));
    let mut out = std::collections::BTreeMap::new();
    let mut from = 0usize;
    while let Some(rel_at) = src[from..].find("pub enum ") {
        let at = from + rel_at;
        let name: String = src[at + "pub enum ".len()..]
            .chars()
            .take_while(|c| c.is_ascii_alphanumeric() || *c == '_')
            .collect();
        from = at + 1;
        if name.is_empty() {
            continue;
        }
        // Re-enter through the public API so the two paths cannot drift; it re-reads and
        // re-strips, which is cheap next to being wrong.
        if let Ok(v) = std::panic::catch_unwind(|| declared_enum_variants(rel, &name)) {
            out.insert(name, v);
        }
    }
    assert!(
        !out.is_empty(),
        "declared_enums_in({rel}) found no `pub enum` at all -- a dictionary that is empty \
         makes every 'is this a real identifier' question answer NO, so a ratchet built on it \
         reports zero offenders forever"
    );
    out
}

/// Every `pub` field name declared by `pub struct <struct_name>` in the given file.
pub fn declared_struct_fields(rel: &str, struct_name: &str) -> BTreeSet<String> {
    let src = read_workspace_file(rel);
    let body = declaration_body(
        &src,
        &format!("pub struct {struct_name} {{"),
        &format!("declared_struct_fields({rel}, {struct_name})"),
    );
    let out: BTreeSet<String> = body
        .lines()
        .map(str::trim)
        .filter_map(|l| l.strip_prefix("pub "))
        .filter_map(|l| l.split(':').next())
        .map(|n| n.trim().to_string())
        .filter(|n| !n.is_empty() && n.chars().all(|c| c.is_ascii_alphanumeric() || c == '_'))
        .collect();
    assert!(
        !out.is_empty(),
        "declared_struct_fields({rel}, {struct_name}) parsed ZERO fields"
    );
    out
}

// ── The parser's own gates ───────────────────────────────────────────────────

/// A parser is a claim like any other. These prove it on subjects whose answers are known
/// independently — three of them from OTHER tests in this target that derive the same figure
/// by their own hand-written parsers, so the two disagree loudly rather than agreeing
/// vacuously.
#[test]
fn p1_the_parser_agrees_with_the_independent_parsers_already_in_the_tree() {
    // `pb_dx49::r8` pins LayerModification at 33 variants by its own parser.
    let lm = declared_enum_variants(CONTINUOUS_EFFECT_RS, "LayerModification");
    assert_eq!(
        lm.len(),
        33,
        "LayerModification variant count moved (this file reads {}); \
         core::pb_dx49_saga_blanking_roster::r8 pins the same enum by an independent parser \
         and must be re-checked in the same commit. If the enum really grew, both move \
         together; if only one moved, one of the two parsers is wrong.\n{lm:?}",
        lm.len()
    );
    // `pb_dx42a::t9` pins these two struct field sets against the same declarations.
    assert_eq!(
        declared_struct_fields(CARD_DEFINITION_RS, "ContinuousEffectDef").len(),
        5
    );
    assert_eq!(
        declared_struct_fields(CARD_DEFINITION_RS, "TargetFilter").len(),
        33
    );
    // `pb_dx20b_enchant_line_roster::r5` pins EnchantFilter at 7.
    assert_eq!(
        declared_struct_fields(STATE_TYPES_RS, "EnchantFilter").len(),
        7
    );
}

/// The comment stripper must remove BOTH comment kinds, and must not remove code that merely
/// LOOKS like a comment inside a string literal. Proven on synthetic input rather than on the
/// corpus, so the assertion cannot be satisfied by the corpus happening not to contain the
/// shape (`OOS-DX32-6`'s own residual: the tree carries zero `/* */` comments today, which is
/// exactly the circumstance under which an untested stripper stays green).
#[test]
fn p2_comment_stripping_handles_both_kinds_and_spares_string_literals() {
    let s = "a // Alpha, Beta\nb /* Gamma, Delta */ c\nlet t = \"// not a comment\";\n";
    let out = strip_comments(s);
    assert!(!out.contains("Alpha"), "line comment survived: {out:?}");
    assert!(!out.contains("Gamma"), "block comment survived: {out:?}");
    assert!(
        out.contains("// not a comment"),
        "a `//` inside a string literal was stripped, which would delete real code from any \
         declaration containing one: {out:?}"
    );
    assert_eq!(
        out.len(),
        s.len(),
        "the stripper changed the byte length, so offsets no longer map back to lines"
    );
}

/// The chunk splitter must not split inside a nested payload. `t7`'s own first draft anchored
/// on the nearest `}` and returned three of eight; this is the same hazard at the splitter.
#[test]
fn p3_chunk_splitting_survives_nested_payloads() {
    let body = "A { x: Vec<(u32, u32)>, y: bool }, B(u8, u8), C, D { z: BTreeMap<String, u8> }";
    let names: Vec<String> = top_level_chunks(body)
        .iter()
        .map(|c| leading_identifier(c))
        .collect();
    assert_eq!(
        names,
        vec!["A", "B", "C", "D"],
        "the splitter broke on a comma inside a payload or a generic argument list"
    );
}

/// Variant payload fields are read per variant, not pooled — the property every "which
/// variants declare field X" pin depends on. A pooled parser would report that EVERY variant
/// declares `targets` as soon as one does, and a floor-shaped assertion cannot tell the two
/// apart.
#[test]
fn p4_variant_payload_fields_are_scoped_to_their_own_variant() {
    let m = declared_enum_variant_fields(CARD_DEFINITION_RS, "AbilityDefinition");
    let with_targets: BTreeSet<&String> = m
        .iter()
        .filter(|(_, f)| f.contains("targets"))
        .map(|(n, _)| n)
        .collect();
    assert!(
        !with_targets.is_empty() && with_targets.len() < m.len(),
        "degenerate split: {} of {} AbilityDefinition variants declare `targets`, so the \
         parser is pooling fields across variants rather than scoping them",
        with_targets.len(),
        m.len()
    );
    // Cross-check BY VALUE against the sibling module's independent derivation, which reaches
    // the same answer through a different code path (a `targets:` text match on the chunk
    // rather than a parsed field list). Two derivations that agree are evidence; one that
    // agrees with itself is not.
    let sibling = crate::pb_dx57_ability_target_variants::target_declaring_ability_variants();
    let mine: BTreeSet<String> = with_targets.into_iter().cloned().collect();
    assert_eq!(
        mine, sibling,
        "the two independent derivations of `AbilityDefinition variants declaring targets` \
         disagree. One of the two parsers is wrong; do not reconcile by editing whichever is \
         easier to change."
    );
}
