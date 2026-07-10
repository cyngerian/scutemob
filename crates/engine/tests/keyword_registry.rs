//! SR-5 — the machine gate that stops a new `KeywordAbility` variant from being
//! silently inert.
//!
//! `state::keyword_registry::handling` is an exhaustive match, so a new variant is
//! already a *compile* error. These tests close the two holes a compile error
//! cannot: that the variant list `all_keywords()` is complete, and that every
//! declaration in the registry still describes the source tree.
//!
//! Audit: `docs/sr-5-keyword-catchall-audit.md`.

use std::collections::{BTreeMap, BTreeSet};
use std::path::{Path, PathBuf};

use mtg_engine::state::keyword_registry::{all_keywords, handling, KeywordHandling};
use mtg_engine::state::types::KeywordAbility;

/// Crate source trees the site scan walks, workspace-relative.
///
/// `crates/card-defs/` is *not* scanned: a card definition naming a keyword is
/// card data, not engine behavior. Before SR-6 the defs lived under
/// `crates/engine/src/cards/defs/` and were skipped by a path filter; now they
/// are simply outside every scan root, which is a stronger form of the same
/// exclusion. `site_scan_is_not_vacuous` asserts they never reappear.
const SCAN_ROOTS: &[&str] = &["crates/engine/src", "crates/card-types/src"];

/// Files that mention `KeywordAbility::<V>` without dispatching on it, and so are
/// excluded from the site scan:
///
/// * `card-types/src/state/types.rs` — the declaration itself.
/// * `engine/src/state/hash.rs` — a mechanical discriminant table (CR-agnostic; it
///   assigns every variant a byte for state hashing). It is exhaustive, so it is a
///   second compile gate, but naming a keyword there is not handling it.
/// * `engine/src/state/keyword_registry.rs` — this registry.
const EXCLUDED: &[&str] = &[
    "crates/card-types/src/state/types.rs",
    "crates/engine/src/state/hash.rs",
    "crates/engine/src/state/keyword_registry.rs",
];

/// The workspace root: `crates/engine/` is two levels down from it.
fn workspace_root() -> PathBuf {
    PathBuf::from(env!("CARGO_MANIFEST_DIR"))
        .parent()
        .and_then(Path::parent)
        .expect("engine manifest dir is <workspace>/crates/engine")
        .to_path_buf()
}

/// Blank out comments, string literals, and char literals so that a later `contains`
/// cannot match a keyword named only in prose.
///
/// A doc comment that says "see `KeywordAbility::Flying`" must not count as a
/// dispatch site — otherwise the anti-rot direction of these tests is vacuous.
///
/// Blanking is char-for-char (newlines survive), so positions in the result line up
/// with `src` only for ASCII input. Nothing depends on that: callers search and slice
/// entirely within the returned string and never map an index back onto `src`.
fn strip_comments_and_literals(src: &str) -> String {
    let b: Vec<char> = src.chars().collect();
    let mut out: Vec<char> = b.clone();
    let n = b.len();
    let mut i = 0;
    let blank = |out: &mut Vec<char>, from: usize, to: usize, b: &[char]| {
        for (k, slot) in out.iter_mut().enumerate().take(to).skip(from) {
            if b[k] != '\n' {
                *slot = ' ';
            }
        }
    };
    while i < n {
        let c = b[i];
        if c == '/' && i + 1 < n && b[i + 1] == '/' {
            let mut j = i;
            while j < n && b[j] != '\n' {
                j += 1;
            }
            blank(&mut out, i, j, &b);
            i = j;
        } else if c == '/' && i + 1 < n && b[i + 1] == '*' {
            // Rust block comments nest.
            let mut depth = 1;
            let mut j = i + 2;
            while j < n && depth > 0 {
                if b[j] == '/' && j + 1 < n && b[j + 1] == '*' {
                    depth += 1;
                    j += 2;
                } else if b[j] == '*' && j + 1 < n && b[j + 1] == '/' {
                    depth -= 1;
                    j += 2;
                } else {
                    j += 1;
                }
            }
            blank(&mut out, i, j, &b);
            i = j;
        } else if c == 'r' && i + 1 < n && (b[i + 1] == '"' || b[i + 1] == '#') {
            // Raw string: r"..." / r#"..."# / r##"..."## ...
            let mut hashes = 0;
            let mut j = i + 1;
            while j < n && b[j] == '#' {
                hashes += 1;
                j += 1;
            }
            if j < n && b[j] == '"' {
                j += 1;
                loop {
                    if j >= n {
                        break;
                    }
                    if b[j] == '"' && b[j + 1..].iter().take(hashes).all(|&h| h == '#') {
                        j += 1 + hashes;
                        break;
                    }
                    j += 1;
                }
                blank(&mut out, i, j.min(n), &b);
                i = j;
            } else {
                i += 1;
            }
        } else if c == '"' {
            let mut j = i + 1;
            while j < n {
                if b[j] == '\\' {
                    j += 2;
                    continue;
                }
                if b[j] == '"' {
                    j += 1;
                    break;
                }
                j += 1;
            }
            blank(&mut out, i, j.min(n), &b);
            i = j;
        } else if c == '\'' {
            // A char literal is 'x' or '\x'. A lifetime (`'static`) has no closing
            // quote in that window, so leave it alone.
            if i + 2 < n && b[i + 1] == '\\' {
                let mut j = i + 2;
                while j < n && b[j] != '\'' {
                    j += 1;
                }
                blank(&mut out, i, (j + 1).min(n), &b);
                i = j + 1;
            } else if i + 2 < n && b[i + 2] == '\'' {
                blank(&mut out, i, i + 3, &b);
                i += 3;
            } else {
                i += 1;
            }
        } else {
            i += 1;
        }
    }
    out.into_iter().collect()
}

/// Every `.rs` file under `SCAN_ROOTS`, excluding `EXCLUDED`, as workspace-relative
/// paths.
fn scanned_files() -> Vec<String> {
    fn walk(dir: &Path, acc: &mut Vec<PathBuf>) {
        for entry in std::fs::read_dir(dir).expect("readable dir") {
            let path = entry.expect("readable entry").path();
            if path.is_dir() {
                walk(&path, acc);
            } else if path.extension().is_some_and(|e| e == "rs") {
                acc.push(path);
            }
        }
    }
    let root = workspace_root();
    let mut acc = Vec::new();
    for scan_root in SCAN_ROOTS {
        walk(&root.join(scan_root), &mut acc);
    }
    let mut files: Vec<String> = acc
        .into_iter()
        .map(|p| {
            p.strip_prefix(&root)
                .expect("under workspace root")
                .to_string_lossy()
                .replace('\\', "/")
        })
        .filter(|p| !EXCLUDED.contains(&p.as_str()))
        .collect();
    files.sort();
    files
}

/// `variant name -> set of workspace-relative files whose *code* names it`.
fn actual_sites() -> BTreeMap<String, BTreeSet<String>> {
    let root = workspace_root();
    let names: Vec<String> = all_keywords().iter().map(variant_name).collect();
    let mut map: BTreeMap<String, BTreeSet<String>> =
        names.iter().map(|n| (n.clone(), BTreeSet::new())).collect();

    for file in scanned_files() {
        let src = std::fs::read_to_string(root.join(&file)).expect("readable source");
        let code = strip_comments_and_literals(&src);
        for name in &names {
            let needle = format!("KeywordAbility::{name}");
            let mut from = 0;
            while let Some(hit) = code[from..].find(&needle) {
                let end = from + hit + needle.len();
                // Reject a prefix match: `KeywordAbility::Flash` inside
                // `KeywordAbility::Flashback`.
                let boundary = code[end..]
                    .chars()
                    .next()
                    .is_none_or(|c| !c.is_alphanumeric() && c != '_');
                if boundary {
                    map.get_mut(name)
                        .expect("known variant")
                        .insert(file.clone());
                    break;
                }
                from = end;
            }
        }
    }
    map
}

/// `Ward(2)` -> `Ward`. `Debug` is derived, so the variant name is the prefix.
fn variant_name(kw: &KeywordAbility) -> String {
    let dbg = format!("{kw:?}");
    dbg.split(['(', ' ', '{'])
        .next()
        .expect("non-empty")
        .to_string()
}

/// Variant names as declared in `state/types.rs`, parsed from the source embedded
/// at compile time.
///
/// Rust cannot enumerate an enum's variants, so `all_keywords()` is hand-written
/// and could silently drift. This re-derives the truth from the declaration.
fn declared_variants() -> BTreeSet<String> {
    const TYPES_RS: &str = include_str!("../../card-types/src/state/types.rs");
    let code = strip_comments_and_literals(TYPES_RS);
    let start = code
        .find("pub enum KeywordAbility {")
        .expect("KeywordAbility declaration");
    let open = start + code[start..].find('{').expect("open brace");

    let mut depth = 0usize;
    let mut end = open;
    for (i, c) in code[open..].char_indices() {
        match c {
            '{' | '(' | '[' => depth += 1,
            '}' | ')' | ']' => {
                depth -= 1;
                if depth == 0 && c == '}' {
                    end = open + i;
                    break;
                }
            }
            _ => {}
        }
    }

    // Inside the enum body, a variant name is the identifier that opens each
    // comma-separated item at nesting depth 0. The body has no attributes (checked
    // by `declared_variants_parser_is_not_vacuous`), and doc comments are already
    // blanked, so this needs no other special cases.
    let body = &code[open + 1..end];
    let mut names = BTreeSet::new();
    let mut depth = 0usize;
    let mut token = String::new();
    let mut expect_ident = true;
    for c in body.chars() {
        if depth == 0 && expect_ident && (c.is_alphanumeric() || c == '_') {
            token.push(c);
            continue;
        }
        if !token.is_empty() {
            names.insert(std::mem::take(&mut token));
            expect_ident = false;
        }
        match c {
            '{' | '(' | '[' => depth += 1,
            '}' | ')' | ']' => depth -= 1,
            ',' if depth == 0 => expect_ident = true,
            _ => {}
        }
    }
    if !token.is_empty() {
        names.insert(token);
    }
    names
}

/// `all_keywords()` must name every variant the enum declares — no more, no fewer.
///
/// This is the test-failure half of the SR-5 gate. The compile-error half lives in
/// `handling()`. Together: a new variant cannot compile until it is classified, and
/// once classified it cannot be omitted from the list the other tests iterate.
#[test]
fn all_keywords_covers_every_variant() {
    let declared = declared_variants();
    let listed: BTreeSet<String> = all_keywords().iter().map(variant_name).collect();

    let missing: Vec<_> = declared.difference(&listed).collect();
    let extra: Vec<_> = listed.difference(&declared).collect();

    assert!(
        missing.is_empty(),
        "KeywordAbility variants declared in state/types.rs but absent from \
         keyword_registry::all_keywords(): {missing:?}. Add them, and classify them \
         in handling()."
    );
    assert!(
        extra.is_empty(),
        "keyword_registry::all_keywords() names variants that no longer exist: {extra:?}"
    );
}

/// Guards `all_keywords_covers_every_variant` against a parser that silently finds
/// nothing. A test that compares two empty sets always passes.
#[test]
fn declared_variants_parser_is_not_vacuous() {
    let declared = declared_variants();
    assert!(
        declared.len() > 100,
        "the state/types.rs parser found only {} variants — it is broken, and the \
         completeness test it feeds is vacuous",
        declared.len()
    );
    // Anchors spanning the whole declaration: first, last, payload-carrying, and
    // the two whose names are prefixes of other variants.
    for anchor in [
        "Deathtouch",
        "Flash",
        "Flashback",
        "Flying",
        "ProtectionFrom",
        "Ward",
        "Exert",
    ] {
        assert!(
            declared.contains(anchor),
            "parser missed the known variant {anchor}"
        );
    }
    // The parser assumes no `#[..]` attributes sit between variants.
    const TYPES_RS: &str = include_str!("../../card-types/src/state/types.rs");
    let code = strip_comments_and_literals(TYPES_RS);
    let start = code.find("pub enum KeywordAbility {").expect("declaration");
    let open = start + code[start..].find('{').expect("open brace");
    let close = open + code[open..].find("\n}").expect("closing brace");
    assert!(
        !code[open..close].contains('#'),
        "KeywordAbility gained an attribute on a variant; declared_variants() must learn to skip it"
    );
}

/// The comment stripper must actually blank prose, or the site scan below would
/// count a doc comment as a dispatch site and the anti-rot check would be vacuous.
#[test]
fn comment_stripper_blanks_prose_and_strings() {
    let src = r#"
/// See KeywordAbility::Flying for details.
let s = "KeywordAbility::Haste";
/* KeywordAbility::Trample */
let real = KeywordAbility::Menace;
"#;
    let code = strip_comments_and_literals(src);
    assert!(
        !code.contains("KeywordAbility::Flying"),
        "doc comment survived"
    );
    assert!(
        !code.contains("KeywordAbility::Haste"),
        "string literal survived"
    );
    assert!(
        !code.contains("KeywordAbility::Trample"),
        "block comment survived"
    );
    assert!(
        code.contains("KeywordAbility::Menace"),
        "real code was blanked"
    );
    assert_eq!(
        code.len(),
        src.len(),
        "stripper must blank in place, not delete (this input is ASCII, so chars == bytes)"
    );
}

/// Every `Handled` variant's declared `sites` must equal the set of engine files
/// whose code names it, and every `Marker` variant must have no such file.
///
/// This is the anti-rot check, and it runs in both directions:
///
/// * delete the last read of a keyword → its `Handled` entry is now a lie → fail
/// * add a read in a file not listed → fail
/// * start branching on a `Marker` keyword → fail (it is no longer a marker)
/// * stop branching on a `Handled` keyword entirely → fail (it is now inert, and
///   the registry must say so deliberately)
#[test]
fn registry_sites_match_the_source_tree() {
    let actual = actual_sites();
    let mut problems = Vec::new();

    for keyword in all_keywords() {
        let name = variant_name(&keyword);
        let found = &actual[&name];
        match handling(&keyword) {
            KeywordHandling::Handled { sites } => {
                // Without this, `Handled { sites: &[] }` on a keyword nothing reads
                // would satisfy the equality below ({} == {}) and pass — the one way
                // to declare a keyword handled while leaving it inert.
                assert!(
                    !sites.is_empty(),
                    "{name}: declared Handled with no sites. A keyword no engine code \
                     reads is not handled — classify it as a Marker (and justify that \
                     in docs/sr-5-keyword-catchall-audit.md), or give it real dispatch."
                );
                let declared: BTreeSet<String> = sites.iter().map(|s| (*s).to_string()).collect();
                if declared != *found {
                    problems.push(format!(
                        "{name}: declared Handled at {declared:?} but the source tree says {found:?}"
                    ));
                }
            }
            KeywordHandling::Marker { carrier, cr } => {
                assert!(!carrier.is_empty(), "{name}: Marker with an empty carrier");
                assert!(!cr.is_empty(), "{name}: Marker with no CR citation");
                if !found.is_empty() {
                    problems.push(format!(
                        "{name}: declared a Marker (behavior lives in {carrier}, CR {cr}) but \
                         engine code now branches on it in {found:?}. Reclassify it as Handled."
                    ));
                }
            }
        }
    }

    assert!(
        problems.is_empty(),
        "keyword_registry is out of date with the source tree:\n  {}",
        problems.join("\n  ")
    );
}

/// The site scan must find real files. If `scanned_files()` returned nothing (a bad
/// path, a moved crate root), `registry_sites_match_the_source_tree` would demand
/// that every keyword be a Marker — and would have failed loudly. This asserts the
/// scan's denominator directly anyway.
///
/// Since SR-6 the denominator spans two crates, so it is asserted per-root: a typo
/// in either `SCAN_ROOTS` entry that still named an existing directory would
/// otherwise shrink the scan silently.
#[test]
fn site_scan_is_not_vacuous() {
    let files = scanned_files();
    assert!(
        files.len() > 40,
        "site scan found only {} source files",
        files.len()
    );

    // Each scan root must actually contribute. `crates/card-types/` earns its place
    // in the scan because `state/dungeon.rs` dispatches on keywords.
    assert!(files.contains(&"crates/engine/src/rules/combat.rs".to_string()));
    assert!(files.contains(&"crates/card-types/src/state/dungeon.rs".to_string()));
    for scan_root in SCAN_ROOTS {
        assert!(
            files.iter().any(|f| f.starts_with(scan_root)),
            "scan root {scan_root} contributed no files"
        );
    }

    // Card definitions are data, not dispatch. They must never enter the scan.
    assert!(!files.iter().any(|f| f.starts_with("crates/card-defs/")));

    let actual = actual_sites();
    assert!(
        actual["Flying"].contains("crates/engine/src/rules/combat.rs"),
        "Flying should dispatch in combat.rs; the scan found {:?}",
        actual["Flying"]
    );
}

/// A `Marker` keyword is a claim that the rules text is implemented elsewhere. Keep
/// the set of them small and deliberate: an unreviewed keyword silently joining this
/// class is exactly the failure SR-5 exists to prevent.
#[test]
fn marker_keywords_are_the_reviewed_set() {
    const REVIEWED: &[&str] = &[
        "Adapt",
        "Bestow",
        "Buyback",
        "Cleave",
        "Cloak",
        "Craft",
        "Discover",
        "Disturb",
        "Emerge",
        "Equip",
        "Fortify",
        "Kicker",
        "Manifest",
        "Outlast",
        "Overload",
        "Prototype",
        "Transform",
        "Transmute",
    ];

    let markers: BTreeSet<String> = all_keywords()
        .iter()
        .filter(|k| matches!(handling(k), KeywordHandling::Marker { .. }))
        .map(variant_name)
        .collect();
    let reviewed: BTreeSet<String> = REVIEWED.iter().map(|s| (*s).to_string()).collect();

    assert_eq!(
        markers, reviewed,
        "the set of marker-only keywords changed. Each entry means \"this keyword \
         needs no engine dispatch\" — justify the change in \
         docs/sr-5-keyword-catchall-audit.md before editing this list."
    );
}
