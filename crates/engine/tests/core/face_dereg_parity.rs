//! PB-RS4 (scutemob-146) drift guard: `remove_one_registration` (`rules/face.rs`)
//! must cover exactly the same `AbilityDefinition` families that
//! `register_static_continuous_effects` (`rules/replacement.rs`) registers.
//!
//! Both functions' `match` bodies use a `_ => {}` catch-all, so a family added to
//! registration later (a new arm in `register_static_continuous_effects`) would
//! silently reopen the CR 604.1/712.18 static-leak hole this PB closed unless
//! something forces the two lists to stay in lockstep. This is a source-scan gate
//! in the SR-5/SR-8/SR-15 style: brace-match each function's body out of its file,
//! strip comments, collect every `AbilityDefinition::<Name>` token the body names,
//! and assert the two `BTreeSet<String>`s are equal.
//!
//! Same proven stripping technique as `bare_lookup_ratchet.rs` (line-comment strip
//! only) and the same word-boundary token scan as
//! `tests/core/ability_definition_registry.rs` (so `Static` does not match inside
//! `StaticRestriction`).

use std::collections::BTreeSet;
use std::fs;
use std::path::PathBuf;

fn engine_src(rel: &str) -> String {
    let path = PathBuf::from(env!("CARGO_MANIFEST_DIR")).join(rel);
    fs::read_to_string(&path).unwrap_or_else(|e| panic!("cannot read {}: {e}", path.display()))
}

/// Strip `//`-to-end-of-line comments (matches `bare_lookup_ratchet.rs`'s technique
/// -- sufficient here because every `AbilityDefinition::<Name>` mention inside these
/// two function bodies that we must NOT count lives in a `///` or `//` comment, and
/// there are no block comments or string literals naming a variant in either body).
fn strip_line_comments(src: &str) -> String {
    src.lines()
        .map(|line| match line.find("//") {
            Some(i) => &line[..i],
            None => line,
        })
        .collect::<Vec<_>>()
        .join("\n")
}

/// Brace-match a function's body out of `src`, starting the search for `fn_name` at
/// or after `start`. Returns the substring between (and not including) the
/// function's outermost `{` and its matching `}`.
fn extract_fn_body(src: &str, fn_name: &str) -> String {
    let needle = format!("fn {fn_name}(");
    let fn_start = src
        .find(&needle)
        .unwrap_or_else(|| panic!("{fn_name} not found in source"));
    let open = fn_start
        + src[fn_start..]
            .find('{')
            .unwrap_or_else(|| panic!("{fn_name}: no opening brace found"));
    let bytes = src.as_bytes();
    let mut depth = 0i32;
    let mut i = open;
    loop {
        assert!(i < bytes.len(), "{fn_name}: unbalanced braces");
        match bytes[i] {
            b'{' => depth += 1,
            b'}' => {
                depth -= 1;
                if depth == 0 {
                    return src[open + 1..i].to_string();
                }
            }
            _ => {}
        }
        i += 1;
    }
}

/// Every `AbilityDefinition::<Name>` token in `code`, honoring a word boundary
/// after the name so `AbilityDefinition::Static` does not match inside
/// `AbilityDefinition::StaticRestriction`. Same technique as
/// `ability_definition_registry.rs::actual_sites`.
fn ability_definition_names(code: &str) -> BTreeSet<String> {
    let needle = "AbilityDefinition::";
    let mut names = BTreeSet::new();
    let mut from = 0;
    while let Some(hit) = code[from..].find(needle) {
        let start = from + hit + needle.len();
        let end = code[start..]
            .char_indices()
            .find(|(_, c)| !c.is_alphanumeric() && *c != '_')
            .map(|(i, _)| start + i)
            .unwrap_or(code.len());
        if end > start {
            names.insert(code[start..end].to_string());
        }
        from = end.max(start + 1);
    }
    names
}

fn registration_families() -> BTreeSet<String> {
    let src = engine_src("src/rules/replacement.rs");
    let body = extract_fn_body(&src, "register_static_continuous_effects");
    let code = strip_line_comments(&body);
    ability_definition_names(&code)
}

fn deregistration_families() -> BTreeSet<String> {
    let src = engine_src("src/rules/face.rs");
    let body = extract_fn_body(&src, "remove_one_registration");
    let code = strip_line_comments(&body);
    ability_definition_names(&code)
}

/// The drift guard: the two family sets must be identical.
#[test]
fn registration_and_deregistration_cover_the_same_ability_families() {
    let registered = registration_families();
    let deregistered = deregistration_families();

    let missing_from_dereg: Vec<_> = registered.difference(&deregistered).collect();
    let extra_in_dereg: Vec<_> = deregistered.difference(&registered).collect();

    assert!(
        missing_from_dereg.is_empty(),
        "register_static_continuous_effects (rules/replacement.rs) registers these \
         AbilityDefinition families that remove_one_registration (rules/face.rs) \
         does not remove: {missing_from_dereg:?}. A permanent that transforms away \
         from a face declaring one of these leaks a stale registration (CR 604.1 / \
         712.18) -- add a matching arm to remove_one_registration. See the PB-RS4 \
         plan (memory/primitives/pb-plan-RS4.md) §5."
    );
    assert!(
        extra_in_dereg.is_empty(),
        "remove_one_registration (rules/face.rs) removes these AbilityDefinition \
         families that register_static_continuous_effects (rules/replacement.rs) \
         never registers: {extra_in_dereg:?}. Either the registration side is \
         missing an arm, or the deregistration arm is dead code -- reconcile them."
    );
}

/// Guards the gate above against a broken extractor silently comparing two empty
/// sets (which would always pass).
#[test]
fn parity_scan_is_not_vacuous() {
    let registered = registration_families();
    let deregistered = deregistration_families();
    assert!(
        registered.len() >= 10,
        "registration_families() found only {} names -- the extractor is broken \
         (expected >= 10: Static + the nine PB-RS4 families)",
        registered.len()
    );
    assert!(
        deregistered.len() >= 10,
        "deregistration_families() found only {} names -- the extractor is broken",
        deregistered.len()
    );
    // Anchors: Static/StaticRestriction (prefix-collision pair) and the two-entry
    // CdaModifyPowerToughness family must both appear in both sets.
    for anchor in ["Static", "StaticRestriction", "CdaModifyPowerToughness"] {
        assert!(
            registered.contains(anchor),
            "registration_families() missed anchor {anchor}"
        );
        assert!(
            deregistered.contains(anchor),
            "deregistration_families() missed anchor {anchor}"
        );
    }
}
