//! SR-15 — the machine gate that stops a new `AbilityDefinition` variant from being
//! silently inert.
//!
//! `state::ability_definition_registry::handling` is an exhaustive match, so a new
//! variant is already a *compile* error. These tests close the two holes a compile
//! error cannot: that the variant list `all_ability_definitions()` is complete, and
//! that every declaration in the registry still describes the source tree.
//!
//! Sibling of `keyword_registry.rs` (SR-5); the stripper and site-scan machinery are
//! deliberately the same, proven code. Audit:
//! `docs/sr-15-dispatch-enum-catchall-audit.md`.

use std::collections::{BTreeMap, BTreeSet};
use std::path::{Path, PathBuf};

use mtg_engine::cards::card_definition::AbilityDefinition;
use mtg_engine::state::ability_definition_registry::{
    all_ability_definitions, handling, AbilityHandling,
};

/// Crate source trees the site scan walks, workspace-relative.
///
/// `crates/card-defs/` is *not* scanned: a card definition naming a variant is card
/// data, not engine behavior. (`site_scan_is_not_vacuous` asserts it never reappears.)
///
/// SR-20 added `crates/simulator/src`: the simulator's legality layer
/// (`legal_actions.rs`) dispatches on ability definitions
/// (LoyaltyAbility/Bloodrush/MutateCost/Morph/Megamorph/Disguise) to decide which
/// actions are legal — real dispatch the registry must see and declare.
const SCAN_ROOTS: &[&str] = &[
    "crates/engine/src",
    "crates/card-types/src",
    "crates/simulator/src",
];

/// Files that mention `AbilityDefinition::<V>` without dispatching on it:
///
/// * `card-types/src/cards/card_definition.rs` — the declaration itself.
/// * `engine/src/state/hash.rs` — a mechanical discriminant table (assigns each
///   variant a byte for state hashing). Exhaustive, so a second compile gate, but
///   naming a variant there is not handling it.
/// * `engine/src/state/ability_definition_registry.rs` — this registry.
const EXCLUDED: &[&str] = &[
    "crates/card-types/src/cards/card_definition.rs",
    "crates/engine/src/state/hash.rs",
    "crates/engine/src/state/ability_definition_registry.rs",
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
/// cannot match a variant named only in prose or a string. (Same proven code as
/// `keyword_registry.rs`; see its docs.)
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
    let names: Vec<String> = all_ability_definitions().iter().map(variant_name).collect();
    let mut map: BTreeMap<String, BTreeSet<String>> =
        names.iter().map(|n| (n.clone(), BTreeSet::new())).collect();

    for file in scanned_files() {
        let src = std::fs::read_to_string(root.join(&file)).expect("readable source");
        let code = strip_comments_and_literals(&src);
        for name in &names {
            let needle = format!("AbilityDefinition::{name}");
            let mut from = 0;
            while let Some(hit) = code[from..].find(&needle) {
                let end = from + hit + needle.len();
                // Reject a prefix match: `AbilityDefinition::Static` inside
                // `AbilityDefinition::StaticRestriction`.
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

/// `Cycling { cost }` -> `Cycling`. `Debug` is derived, so the variant name is the
/// prefix before the first `(`, ` `, or `{`.
fn variant_name(a: &AbilityDefinition) -> String {
    let dbg = format!("{a:?}");
    dbg.split(['(', ' ', '{'])
        .next()
        .expect("non-empty")
        .to_string()
}

/// Variant names as declared in `cards/card_definition.rs`, parsed from the source
/// embedded at compile time.
///
/// Rust cannot enumerate an enum's variants, so `all_ability_definitions()` is
/// hand-written and could silently drift. This re-derives the truth from the
/// declaration. Field-level `#[serde(default)]` attributes sit *inside* variant
/// braces (depth > 0) and so never confuse the depth-0 name scan below.
fn declared_variants() -> BTreeSet<String> {
    const DEF_RS: &str = include_str!("../../../card-types/src/cards/card_definition.rs");
    let code = strip_comments_and_literals(DEF_RS);
    let start = code
        .find("pub enum AbilityDefinition {")
        .expect("AbilityDefinition declaration");
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

    // A variant name is the identifier that opens each comma-separated item at
    // nesting depth 0 inside the enum body. Doc comments are already blanked.
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

/// `all_ability_definitions()` must name every variant the enum declares — no more,
/// no fewer.
///
/// This is the test-failure half of the SR-15 gate. The compile-error half lives in
/// `handling()`. Together: a new variant cannot compile until it is classified, and
/// once classified it cannot be omitted from the list the other tests iterate.
#[test]
fn all_ability_definitions_covers_every_variant() {
    let declared = declared_variants();
    let listed: BTreeSet<String> = all_ability_definitions().iter().map(variant_name).collect();

    let missing: Vec<_> = declared.difference(&listed).collect();
    let extra: Vec<_> = listed.difference(&declared).collect();

    assert!(
        missing.is_empty(),
        "AbilityDefinition variants declared in cards/card_definition.rs but absent \
         from ability_definition_registry::all_ability_definitions(): {missing:?}. Add \
         them, and classify them in handling()."
    );
    assert!(
        extra.is_empty(),
        "all_ability_definitions() names variants that no longer exist: {extra:?}"
    );
}

/// Guards `all_ability_definitions_covers_every_variant` against a parser that
/// silently finds nothing. A test comparing two empty sets always passes.
#[test]
fn declared_variants_parser_is_not_vacuous() {
    let declared = declared_variants();
    assert!(
        declared.len() > 60,
        "the card_definition.rs parser found only {} variants — it is broken, and the \
         completeness test it feeds is vacuous",
        declared.len()
    );
    // Anchors spanning the whole declaration: first, last, unit variants, and the two
    // whose names are prefixes of other variants (`Static`/`StaticRestriction`,
    // `Morph`/`Megamorph`).
    for anchor in [
        "Activated",
        "Static",
        "StaticRestriction",
        "Cipher",
        "OpeningHand",
        "Morph",
        "Megamorph",
        "CastSelfFromGraveyard",
    ] {
        assert!(
            declared.contains(anchor),
            "parser missed the known variant {anchor}"
        );
    }
}

/// The comment stripper must actually blank prose and strings, or the site scan would
/// count a doc comment or a `panic!("...AbilityDefinition::X...")` string as a
/// dispatch site and the anti-rot check would be vacuous.
#[test]
fn comment_stripper_blanks_prose_and_strings() {
    let src = r#"
/// See AbilityDefinition::Cycling for details.
let s = "AbilityDefinition::Kicker";
/* AbilityDefinition::Evoke */
let real = AbilityDefinition::Cipher;
"#;
    let code = strip_comments_and_literals(src);
    assert!(
        !code.contains("AbilityDefinition::Cycling"),
        "doc comment survived"
    );
    assert!(
        !code.contains("AbilityDefinition::Kicker"),
        "string literal survived"
    );
    assert!(
        !code.contains("AbilityDefinition::Evoke"),
        "block comment survived"
    );
    assert!(
        code.contains("AbilityDefinition::Cipher"),
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
/// * delete the last read of a variant → its `Handled` entry is now a lie → fail
/// * add a read in a file not listed → fail
/// * start branching on a `Marker` variant → fail (it is no longer a marker)
/// * stop branching on a `Handled` variant entirely → fail (it is now inert, and the
///   registry must say so deliberately)
#[test]
fn registry_sites_match_the_source_tree() {
    let actual = actual_sites();
    let mut problems = Vec::new();

    for ability in all_ability_definitions() {
        let name = variant_name(&ability);
        let found = &actual[&name];
        match handling(&ability) {
            AbilityHandling::Handled { sites } => {
                // Without this, `Handled { sites: &[] }` on a variant nothing reads
                // would satisfy the equality below ({} == {}) and pass — the one way
                // to declare a variant handled while leaving it inert.
                assert!(
                    !sites.is_empty(),
                    "{name}: declared Handled with no sites. A variant no engine code \
                     reads is not handled — classify it as a Marker (and justify that \
                     in docs/sr-15-dispatch-enum-catchall-audit.md), or give it real \
                     dispatch."
                );
                let declared: BTreeSet<String> = sites.iter().map(|s| (*s).to_string()).collect();
                if declared != *found {
                    problems.push(format!(
                        "{name}: declared Handled at {declared:?} but the source tree says {found:?}"
                    ));
                }
            }
            AbilityHandling::Marker { carrier, cr } => {
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
        "ability_definition_registry is out of date with the source tree:\n  {}",
        problems.join("\n  ")
    );
}

/// The site scan must find real files. If `scanned_files()` returned nothing (a bad
/// path, a moved crate root), `registry_sites_match_the_source_tree` would demand that
/// every variant be a Marker — and would have failed loudly. This asserts the scan's
/// denominator directly anyway, per-root.
#[test]
fn site_scan_is_not_vacuous() {
    let files = scanned_files();
    assert!(
        files.len() > 40,
        "site scan found only {} source files",
        files.len()
    );
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
        actual["Spell"].contains("crates/engine/src/rules/casting.rs"),
        "Spell should dispatch in casting.rs; the scan found {:?}",
        actual["Spell"]
    );

    // SR-20: the simulator's legality dispatch is now in scope and declared.
    assert!(files.contains(&"crates/simulator/src/legal_actions.rs".to_string()));
    assert!(
        actual["Bloodrush"].contains("crates/simulator/src/legal_actions.rs"),
        "Bloodrush should dispatch in the simulator's legal_actions.rs; the scan found {:?}",
        actual["Bloodrush"]
    );
}

/// A `Marker` variant is a claim that the rules text is implemented elsewhere (by a
/// `KeywordAbility` twin). Keep the set of them small and deliberate: an unreviewed
/// variant silently joining this class is exactly the failure SR-15 exists to prevent.
#[test]
fn marker_abilities_are_the_reviewed_set() {
    const REVIEWED: &[&str] = &["CumulativeUpkeep", "Echo", "Fading", "Vanishing"];

    let markers: BTreeSet<String> = all_ability_definitions()
        .iter()
        .filter(|a| matches!(handling(a), AbilityHandling::Marker { .. }))
        .map(variant_name)
        .collect();
    let reviewed: BTreeSet<String> = REVIEWED.iter().map(|s| (*s).to_string()).collect();

    assert_eq!(
        markers, reviewed,
        "the set of marker-only AbilityDefinition variants changed. Each entry means \
         \"this variant carries no dispatch; a KeywordAbility twin does\" — justify the \
         change in docs/sr-15-dispatch-enum-catchall-audit.md before editing this list."
    );
}

// --- SR-20: the site scanner only sees the literal token `AbilityDefinition::<V>`. ---
//
// `use ...AbilityDefinition as AD; matches!(a, AD::Vanishing { .. })` (and glob
// `use ...AbilityDefinition::*;`) dispatch on a variant without ever writing
// `AbilityDefinition::Vanishing`, so `actual_sites()` records nothing and a `Marker`
// could silently gain dispatch. Reproduced against this registry (`AD::Vanishing`)
// during the SR-20 re-audit. Same proven machinery as `keyword_registry.rs`.

/// The text of each `use ... ;` item in `code` (already comment/literal-stripped).
/// See `keyword_registry.rs` for the rationale (catches `use` in bodies too).
fn use_statements(code: &str) -> Vec<String> {
    let chars: Vec<char> = code.chars().collect();
    let n = chars.len();
    let is_ident = |c: char| c.is_alphanumeric() || c == '_';
    let mut stmts = Vec::new();
    let mut i = 0;
    while i < n {
        if chars[i] == 'u' && i + 2 < n && chars[i + 1] == 's' && chars[i + 2] == 'e' {
            let before_ok = i == 0 || !is_ident(chars[i - 1]);
            let after_ok = chars.get(i + 3).is_none_or(|&c| !is_ident(c));
            if before_ok && after_ok {
                let mut j = i + 3;
                while j < n && chars[j] != ';' {
                    j += 1;
                }
                stmts.push(chars[i + 3..j.min(n)].iter().collect());
                i = j + 1;
                continue;
            }
        }
        i += 1;
    }
    stmts
}

/// If `stmt` imports `type_name` in a bypassing form (aliased with `as`, or used as
/// a `::` path prefix to pull variants in), describe it; else `None`. Followed by
/// `,` / `}` / end it is a plain type import, which is fine.
fn forbidden_use_form(stmt: &str, type_name: &str) -> Option<String> {
    let chars: Vec<char> = stmt.chars().collect();
    let tn: Vec<char> = type_name.chars().collect();
    let n = chars.len();
    let is_ident = |c: char| c.is_alphanumeric() || c == '_';
    let mut i = 0;
    while i + tn.len() <= n {
        if chars[i..i + tn.len()] == tn[..] {
            let before_ok = i == 0 || !is_ident(chars[i - 1]);
            let after = i + tn.len();
            let after_ok = after >= n || !is_ident(chars[after]);
            if before_ok && after_ok {
                let mut k = after;
                while k < n && chars[k].is_whitespace() {
                    k += 1;
                }
                if k + 1 < n && chars[k] == ':' && chars[k + 1] == ':' {
                    return Some(format!(
                        "`use {type_name}::…` imports variants (glob `::*`, grouped `::{{…}}`, \
                         or single `::Variant`); the scanner sees only `{type_name}::Variant` at \
                         a real dispatch site, never a bare variant brought into scope this way"
                    ));
                }
                if k + 1 < n
                    && chars[k] == 'a'
                    && chars[k + 1] == 's'
                    && chars.get(k + 2).is_none_or(|&c| !is_ident(c))
                {
                    return Some(format!(
                        "`use {type_name} as …` aliases the type; dispatch through the alias \
                         (`Alias::Variant`) is invisible to the `{type_name}::` scanner"
                    ));
                }
            }
        }
        i += 1;
    }
    None
}

/// Every bypassing import of `type_name` across the scanned files, as `"file: reason"`.
fn bypassing_use_imports(type_name: &str) -> Vec<String> {
    let root = workspace_root();
    let mut hits = Vec::new();
    for file in scanned_files() {
        let src = std::fs::read_to_string(root.join(&file)).expect("readable source");
        let code = strip_comments_and_literals(&src);
        for stmt in use_statements(&code) {
            if let Some(reason) = forbidden_use_form(&stmt, type_name) {
                hits.push(format!("{file}: {reason}"));
            }
        }
    }
    hits
}

/// A `type Alias = <path>AbilityDefinition;` lets `Alias::Variant` dispatch without
/// ever writing `AbilityDefinition::` — the same blinding an aliased *import*
/// achieves. Reports each such alias whose RHS is exactly a bare path ending in
/// `type_name` (a wrapped `Vec<AbilityDefinition>` does not expose variants).
fn blinding_type_aliases(type_name: &str) -> Vec<String> {
    let root = workspace_root();
    let is_ident = |c: char| c.is_alphanumeric() || c == '_';
    let mut hits = Vec::new();
    for file in scanned_files() {
        let src = std::fs::read_to_string(root.join(&file)).expect("readable source");
        let code = strip_comments_and_literals(&src);
        let chars: Vec<char> = code.chars().collect();
        let n = chars.len();
        let mut i = 0;
        while i < n {
            let is_type_kw = chars[i] == 't'
                && i + 4 <= n
                && chars[i..i + 4] == ['t', 'y', 'p', 'e']
                && (i == 0 || !is_ident(chars[i - 1]))
                && chars.get(i + 4).is_none_or(|&c| !is_ident(c));
            if is_type_kw {
                let mut j = i + 4;
                while j < n && chars[j] != ';' {
                    j += 1;
                }
                let stmt: String = chars[i + 4..j.min(n)].iter().collect();
                if let Some((_lhs, rhs)) = stmt.split_once('=') {
                    let rhs = rhs.trim();
                    let last = rhs.rsplit("::").next().unwrap_or(rhs);
                    let bare = !rhs.is_empty() && rhs.chars().all(|c| is_ident(c) || c == ':');
                    if bare && last == type_name {
                        hits.push(format!("{file}: `type … = {rhs};` aliases {type_name}"));
                    }
                }
                i = j + 1;
                continue;
            }
            i += 1;
        }
    }
    hits
}

/// No scanned file may import `AbilityDefinition` in a form that blinds the site
/// scanner. The reproduced alias attack (`use …AbilityDefinition as AD; AD::Vanishing`)
/// is caught here.
#[test]
fn use_imports_do_not_bypass_the_scanner() {
    let hits = bypassing_use_imports("AbilityDefinition");
    assert!(
        hits.is_empty(),
        "these `use` imports let engine code dispatch on an AbilityDefinition variant \
         without writing `AbilityDefinition::Variant`, so the site scan cannot see it \
         (SR-20):\n  {}\n\
         Import the type plainly and write `AbilityDefinition::Variant` at the dispatch \
         site, or — if an alias is truly wanted — add the file to EXCLUDED and give its \
         dispatch a declared home another way.",
        hits.join("\n  ")
    );
}

/// No scanned file may create a `type` alias of the enum either — `Alias::Variant`
/// through it is just as invisible to the `AbilityDefinition::` scanner as an aliased
/// import. (SR-20 review follow-up; there are none today.)
#[test]
fn type_aliases_do_not_bypass_the_scanner() {
    let hits = blinding_type_aliases("AbilityDefinition");
    assert!(
        hits.is_empty(),
        "these `type` aliases let engine code dispatch on an AbilityDefinition variant \
         through an alias, invisible to the site scan (SR-20):\n  {}\n\
         Name the enum directly at the dispatch site instead.",
        hits.join("\n  ")
    );
}

/// Guards the two bypass gates above against a broken parser that finds nothing.
/// Asserts the splitter, the use-form classifier, and the type-alias predicate all
/// work on synthetic input covering every case, so an empty `hits` means "clean",
/// not "blind".
#[test]
fn import_bypass_detector_is_not_vacuous() {
    let stmts = use_statements("use a::B; fn f() { use c::D::*; } let unused = reuse;");
    assert_eq!(stmts.len(), 2, "use_statements miscounted: {stmts:?}");

    assert!(forbidden_use_form(" x::AbilityDefinition as AD", "AbilityDefinition").is_some());
    assert!(forbidden_use_form(" x::AbilityDefinition::*", "AbilityDefinition").is_some());
    assert!(forbidden_use_form(
        " x::AbilityDefinition::{Morph, Disguise}",
        "AbilityDefinition"
    )
    .is_some());
    assert!(forbidden_use_form(" x::AbilityDefinition::Bloodrush", "AbilityDefinition").is_some());
    assert!(forbidden_use_form(" x::y::AbilityDefinition", "AbilityDefinition").is_none());
    assert!(forbidden_use_form(" x::{AbilityDefinition, Foo}", "AbilityDefinition").is_none());
    assert!(
        forbidden_use_form(" x::AbilityDefinitionSet::*", "AbilityDefinition").is_none(),
        "whole-word check failed: matched a longer identifier"
    );

    // The type-alias predicate: a bare alias of the enum is flagged; a wrapped or
    // differently-named alias is not.
    let flagged = |rhs: &str| {
        let last = rhs.rsplit("::").next().unwrap_or(rhs);
        let bare = !rhs.is_empty()
            && rhs
                .chars()
                .all(|c| c.is_alphanumeric() || c == '_' || c == ':');
        bare && last == "AbilityDefinition"
    };
    assert!(flagged("AbilityDefinition"));
    assert!(flagged("crate::cards::card_definition::AbilityDefinition"));
    assert!(!flagged("Vec<AbilityDefinition>"));
    assert!(!flagged("AbilityDefinitionSet"));
}
