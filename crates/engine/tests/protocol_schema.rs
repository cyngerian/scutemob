//! SR-8: the anti-rot gate behind `PROTOCOL_VERSION`.
//!
//! `rules/protocol.rs` puts a version tag on serialized `Command` / `GameEvent`
//! streams and rejects any version but its own. That is only worth anything if
//! `PROTOCOL_VERSION` actually changes when the wire format does — otherwise two
//! builds with incompatible shapes both claim v1 and mis-deserialize each other
//! *silently*, which is precisely the failure the task exists to prevent.
//!
//! Nothing about a hand-bumped constant makes that true. So this suite computes
//! the wire shape from source and pins it:
//!
//! 1. Index every `pub enum` / `pub struct` / `pub type` under the scan roots.
//! 2. Walk the **type positions** (named-field types, tuple-variant elements)
//!    transitively from the wire frames `Command`, `GameEvent` and `ReplayLog`.
//!    That closure — currently 90 types — is the wire surface.
//! 3. Digest the normalized declaration text of the closure, attributes
//!    included, and compare against `PROTOCOL_SCHEMA_FINGERPRINT`.
//!
//! Attributes are hashed because they are wire-visible: `#[serde(rename)]`
//! renames a field, `#[serde(skip)]` deletes one (see `scutemob-68` / SR-16 for
//! a live instance of that hazard), `#[serde(default)]` changes what a missing
//! field means.
//!
//! ## What this gate does not catch
//!
//! - **Semantic drift.** Redefining what an existing `u32` *means* leaves the
//!   shape identical. Bump `PROTOCOL_VERSION` by hand for those.
//! - **External types.** `im::OrdMap`, `Vec`, `Option` etc. are allowlisted in
//!   [`EXTERNAL_TYPES`]; a `Cargo.toml` bump that changes `im`'s serialized form
//!   moves the wire without moving this digest.
//! - **Variant reordering is a false positive, deliberately.** serde's external
//!   tagging keys on variant *names*, so a pure reorder is wire-compatible, yet
//!   it moves the digest because the digest hashes declaration text in order.
//!   The cost is one needless version bump; the alternative is a
//!   variant-sorting normalizer, which is more code that can be wrong in the
//!   *unsafe* direction. Accepted.
//!
//! Per the SR-5 lesson ("assert the denominator"), every derived set here has a
//! non-vacuity guard: an index that finds nothing, a closure that walks nowhere,
//! or a scan root that contributes nothing all fail loudly rather than digesting
//! the empty string forever.

use std::collections::{BTreeMap, BTreeSet};
use std::path::{Path, PathBuf};

use mtg_engine::{PROTOCOL_SCHEMA_FINGERPRINT, PROTOCOL_VERSION};

/// Crates whose types may appear on the wire. `card-defs` is deliberately absent:
/// card *definitions* are not sent, only the runtime types derived from them.
const SCAN_ROOTS: [&str; 2] = ["crates/engine/src", "crates/card-types/src"];

/// Every type that is serialized as a wire frame in its own right.
///
/// `Command` and `GameEvent` *are* the protocol (invariants #3 and #4).
/// `ReplayLog` is the third: `encode_replay_log` puts it on the wire, and its
/// own frame (`hash_schema_version` + `commands`) is wire format even though
/// nothing reachable from `Command`/`GameEvent` mentions it. Leaving it out
/// meant a new field on `ReplayLog` changed the replay-log format with nothing
/// forcing a `PROTOCOL_VERSION` bump — a process guarantee, which is the thing
/// this whole gate exists to delete.
///
/// The remaining frame, `Envelope<T>`, is generic and so cannot be walked here;
/// its two field *names* are pinned instead by
/// `the_envelope_frame_has_exactly_the_expected_fields` in
/// `tests/protocol_roundtrip.rs`.
const PROTOCOL_ROOTS: [&str; 3] = ["Command", "GameEvent", "ReplayLog"];

/// Types whose serialized shape is owned by someone else (std, `im`).
///
/// Anything reachable from the protocol roots that is neither indexed nor listed
/// here fails `every_referenced_type_resolves`. That is the guard against a
/// silent under-inclusion: a wire-bearing type we simply forgot to hash.
const EXTERNAL_TYPES: [&str; 24] = [
    "u8", "u16", "u32", "u64", "usize", "i8", "i16", "i32", "i64", "isize", "f32", "f64", "bool",
    "char", "str", "String", "Vec", "Option", "Box", "Arc", "Rc", "OrdMap", "OrdSet", "Vector",
];

/// Floors for the non-vacuity guards. Deliberately well below the real values —
/// they catch a scanner that broke, not a codebase that grew.
const MIN_INDEXED_TYPES: usize = 150;
const MIN_CLOSURE_TYPES: usize = 80;

/// Types that must be on the wire. If one of these vanishes from the closure the
/// scanner is broken, not the protocol.
///
/// `Effect` and `KeywordAbility` prove the walk crosses into `card-types` and
/// down through `Characteristics`. `RoomIndex` proves `pub type` aliases are
/// indexed — it is a `usize` alias, so a scanner that only understood `enum` and
/// `struct` would leave it, and any retarget of it, outside the digest.
const CLOSURE_MUST_CONTAIN: [&str; 9] = [
    "Command",
    "GameEvent",
    "Characteristics",
    "Effect",
    "KeywordAbility",
    "ManaCost",
    "ObjectId",
    "PlayerId",
    "RoomIndex",
];

/// Types that must **not** be on the wire.
///
/// `GameState` bounding the closure is the whole reason protocol versioning and
/// `HASH_SCHEMA_VERSION` can stay separate concerns. If whole-state sync ever
/// leaks into a `GameEvent`, that is a design decision that must be made
/// deliberately — so it fails here first.
const CLOSURE_MUST_NOT_CONTAIN: [&str; 4] =
    ["GameState", "PlayerState", "StackObject", "CardDefinition"];

/// The workspace root: `crates/engine/` is two levels down from it.
fn workspace_root() -> PathBuf {
    PathBuf::from(env!("CARGO_MANIFEST_DIR"))
        .parent()
        .and_then(Path::parent)
        .expect("engine manifest dir is <workspace>/crates/engine")
        .to_path_buf()
}

// ── Source scanning ──────────────────────────────────────────────────────────

/// Length of the string/char literal starting at `b[i]`, or `None` if one does
/// not start there. Handles raw strings (`r"…"`, `r#"…"#`, …), which do not
/// honour backslash escapes.
///
/// Literals are *skipped*, never blanked: a `#[serde(rename = "x")]` is wire
/// format, so its contents must survive into the digest.
fn literal_len(b: &[u8], i: usize) -> Option<usize> {
    let n = b.len();
    // Raw string: `r` (not a token tail) followed by `#`* then `"`.
    if b[i] == b'r' && (i == 0 || !(b[i - 1].is_ascii_alphanumeric() || b[i - 1] == b'_')) {
        let mut hashes = 0;
        let mut j = i + 1;
        while j < n && b[j] == b'#' {
            hashes += 1;
            j += 1;
        }
        if j < n && b[j] == b'"' {
            j += 1;
            while j < n {
                if b[j] == b'"' && b[j + 1..].iter().take(hashes).all(|&c| c == b'#') {
                    return Some(j + 1 + hashes - i);
                }
                j += 1;
            }
            return Some(n - i);
        }
    }
    if b[i] == b'"' {
        let mut j = i + 1;
        while j < n {
            match b[j] {
                b'\\' => j += 2,
                b'"' => return Some(j + 1 - i),
                _ => j += 1,
            }
        }
        return Some(n - i);
    }
    None
}

/// Replace comments with a single space each, leaving string literals intact.
///
/// Block comments nest, per the Rust grammar. Lifetimes (`'a`) are why char
/// literals are not treated as literals here — they never contain `//` or braces
/// in these declarations, so ignoring them is safe and avoids mis-parsing `&'a`.
fn strip_comments(src: &str) -> String {
    let b = src.as_bytes();
    let n = b.len();
    // Byte-wise, not `b[i] as char`: that would mangle any multi-byte character
    // living outside a comment or literal.
    let mut out: Vec<u8> = Vec::with_capacity(n);
    let mut i = 0;
    while i < n {
        if let Some(len) = literal_len(b, i) {
            out.extend_from_slice(&b[i..i + len]);
            i += len;
        } else if b[i] == b'/' && i + 1 < n && b[i + 1] == b'/' {
            while i < n && b[i] != b'\n' {
                i += 1;
            }
            out.push(b' ');
        } else if b[i] == b'/' && i + 1 < n && b[i + 1] == b'*' {
            let (mut depth, mut j) = (1usize, i + 2);
            while j < n && depth > 0 {
                if b[j] == b'/' && j + 1 < n && b[j + 1] == b'*' {
                    depth += 1;
                    j += 2;
                } else if b[j] == b'*' && j + 1 < n && b[j + 1] == b'/' {
                    depth -= 1;
                    j += 2;
                } else {
                    j += 1;
                }
            }
            i = j;
            out.push(b' ');
        } else {
            out.push(b[i]);
            i += 1;
        }
    }
    String::from_utf8(out).expect("comment stripping preserves UTF-8 boundaries")
}

/// Index of the byte just past the delimiter matching the one at `open`,
/// skipping string literals.
fn match_delim(b: &[u8], open: usize, o: u8, c: u8) -> usize {
    let n = b.len();
    let mut depth = 0usize;
    let mut i = open;
    while i < n {
        if let Some(len) = literal_len(b, i) {
            i += len;
            continue;
        }
        if b[i] == o {
            depth += 1;
        } else if b[i] == c {
            depth -= 1;
            if depth == 0 {
                return i + 1;
            }
        }
        i += 1;
    }
    n
}

/// A `pub enum` / `pub struct` / `pub type` declaration: its container
/// attributes, header, and body, exactly as they affect serde.
struct Decl {
    /// Attributes + `pub enum Name {…}`, whitespace-normalized.
    hash_text: String,
    /// Body with attributes removed — used only to find type references.
    traversal_body: String,
    /// `pub type X = Y;`. Aliases are transparent to serde, so `traversal_body`
    /// holds the bare target type rather than a braced body, and type references
    /// are read straight out of it.
    is_alias: bool,
}

/// Remove `#[…]` spans (bracket-matched, string-aware).
fn strip_attributes(src: &str) -> String {
    let b = src.as_bytes();
    let n = b.len();
    let mut out: Vec<u8> = Vec::with_capacity(n);
    let mut i = 0;
    while i < n {
        if let Some(len) = literal_len(b, i) {
            out.extend_from_slice(&b[i..i + len]);
            i += len;
        } else if b[i] == b'#' && i + 1 < n && b[i + 1] == b'[' {
            i = match_delim(b, i + 1, b'[', b']');
            out.push(b' ');
        } else {
            out.push(b[i]);
            i += 1;
        }
    }
    String::from_utf8(out).expect("attribute stripping preserves UTF-8 boundaries")
}

fn normalize_ws(s: &str) -> String {
    s.split_whitespace().collect::<Vec<_>>().join(" ")
}

/// Container attributes immediately above `decl_start`, minus the ones that
/// cannot affect the wire (`#[allow(…)]`; doc comments are already stripped).
///
/// Bracket-matched, not line-based. rustfmt wraps a long derive across lines:
///
/// ```text
/// #[derive(
///     Clone, Copy, Debug, Default, PartialEq, Eq, PartialOrd, Ord, Hash, Serialize, Deserialize,
/// )]
/// pub enum EnchantControllerConstraint {
/// ```
///
/// A line-based walk sees `)]`, decides it is not an attribute, and drops the
/// container's entire serde config out of the digest — silently. That type is
/// real and is in the closure; `every_closure_type_shows_its_serialize_derive`
/// is the guard that caught it.
///
/// Each candidate `#[` is confirmed by re-running the *forward*, string-aware
/// `match_delim` and checking it lands exactly on the `]` we started from, so a
/// `]` inside a string literal cannot fake an attribute boundary.
fn preceding_attributes(src: &str, decl_start: usize) -> String {
    let b = src.as_bytes();
    let mut end = decl_start;
    let mut spans: Vec<(usize, usize)> = Vec::new();

    loop {
        while end > 0 && b[end - 1].is_ascii_whitespace() {
            end -= 1;
        }
        if end == 0 || b[end - 1] != b']' {
            break;
        }
        // Scan back for a `#[` whose forward match ends exactly at `end`.
        let mut found = None;
        let mut i = end - 1;
        while i > 0 {
            i -= 1;
            if b[i] == b'[' && i > 0 && b[i - 1] == b'#' && match_delim(b, i, b'[', b']') == end {
                found = Some(i - 1);
                break;
            }
        }
        let Some(start) = found else { break };
        spans.push((start, end));
        end = start;
    }

    spans.reverse();
    let kept: Vec<&str> = spans
        .into_iter()
        .map(|(s, e)| &src[s..e])
        .filter(|a| !a.trim_start().starts_with("#[allow"))
        .collect();
    normalize_ws(&kept.join(" "))
}

/// What a source scan yields: the type index, the per-root denominators, and
/// any name declared more than once.
struct ScanResult {
    index: BTreeMap<String, Decl>,
    /// scan root -> type names declared under it (per-root denominator guard)
    by_root: BTreeMap<String, BTreeSet<String>>,
    /// Names declared twice. `index` keeps the first, so a collision means the
    /// digest may be hashing the wrong declaration.
    collisions: BTreeSet<String>,
}

/// Every `pub enum` / `pub struct` / `pub type` under the scan roots, keyed by
/// type name.
fn index_declarations() -> ScanResult {
    let root = workspace_root();
    let mut index: BTreeMap<String, Decl> = BTreeMap::new();
    let mut by_root: BTreeMap<String, BTreeSet<String>> = BTreeMap::new();
    let mut collisions: BTreeSet<String> = BTreeSet::new();

    for scan_root in SCAN_ROOTS {
        let mut files = Vec::new();
        walk(&root.join(scan_root), &mut files);
        files.sort();
        let names = by_root.entry(scan_root.to_string()).or_default();

        for file in files {
            let raw = std::fs::read_to_string(&file).expect("readable source");
            let src = strip_comments(&raw);
            let b = src.as_bytes();

            for kw in ["pub enum ", "pub struct ", "pub type "] {
                let is_alias = kw == "pub type ";
                let mut from = 0;
                while let Some(rel) = src[from..].find(kw) {
                    let at = from + rel;
                    from = at + kw.len();
                    // Must start a token (not `...pub enum` inside an ident).
                    if at > 0 && (b[at - 1].is_ascii_alphanumeric() || b[at - 1] == b'_') {
                        continue;
                    }
                    let after = at + kw.len();
                    let name: String = src[after..]
                        .chars()
                        .take_while(|c| c.is_ascii_alphanumeric() || *c == '_')
                        .collect();
                    if name.is_empty() {
                        continue;
                    }

                    let body = if is_alias {
                        // `pub type X = <target>;` — the target is the shape.
                        let eq = match src[after + name.len()..].find('=') {
                            Some(p) => after + name.len() + p + 1,
                            None => continue,
                        };
                        let semi = src[eq..].find(';').map(|p| eq + p).unwrap_or(src.len());
                        src[eq..semi].to_string()
                    } else {
                        // Find the delimiter that opens the body: `{`, `(`, or `;`.
                        let mut j = after + name.len();
                        while j < b.len() && b[j] != b'{' && b[j] != b'(' && b[j] != b';' {
                            j += 1;
                        }
                        if j >= b.len() || b[j] == b';' {
                            String::new() // unit struct
                        } else {
                            let (o, c) = if b[j] == b'{' {
                                (b'{', b'}')
                            } else {
                                (b'(', b')')
                            };
                            src[j..match_delim(b, j, o, c)].to_string()
                        }
                    };

                    names.insert(name.clone());
                    if index.contains_key(&name) {
                        collisions.insert(name.clone());
                    }
                    // First declaration wins. Sound only while names are unique —
                    // asserted by `declared_type_names_are_unique`.
                    index.entry(name.clone()).or_insert_with(|| Decl {
                        hash_text: normalize_ws(&format!(
                            "{} {}{} {}",
                            preceding_attributes(&src, at),
                            kw,
                            name,
                            body
                        )),
                        traversal_body: strip_attributes(&body),
                        is_alias,
                    });
                }
            }
        }
    }
    ScanResult {
        index,
        by_root,
        collisions,
    }
}

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

/// Type references in a declaration body, taken from **type positions only**:
/// the text after `:` in a named field, and the contents of a tuple variant's
/// parentheses.
///
/// Scanning for capitalized identifiers anywhere in the body would also pick up
/// variant *names* (`Command::CastSpell`), pulling unrelated types into the
/// closure whenever a variant happens to share a type's name.
fn type_references(body: &str) -> BTreeSet<String> {
    let b = body.as_bytes();
    let n = b.len();
    let mut spans: Vec<&str> = Vec::new();
    let mut i = 0;
    while i < n {
        if let Some(len) = literal_len(b, i) {
            i += len;
        } else if b[i] == b':' && i + 1 < n && b[i + 1] != b':' && (i == 0 || b[i - 1] != b':') {
            // Named field: capture the type up to the field-terminating `,`/`}`.
            let mut depth = 0usize;
            let mut j = i + 1;
            while j < n {
                match b[j] {
                    b'<' | b'(' | b'[' => depth += 1,
                    b'>' | b')' | b']' => {
                        if depth == 0 {
                            break;
                        }
                        depth -= 1;
                    }
                    b',' | b'}' if depth == 0 => break,
                    _ => {}
                }
                j += 1;
            }
            spans.push(&body[i + 1..j]);
            i = j;
        } else if b[i] == b'(' {
            let end = match_delim(b, i, b'(', b')');
            spans.push(&body[i + 1..end.saturating_sub(1)]);
            i = end;
        } else {
            i += 1;
        }
    }

    let mut out = BTreeSet::new();
    for span in spans {
        out.extend(capitalized_idents(span));
    }
    out
}

/// Identifiers starting with an uppercase letter — i.e. type names, by Rust
/// convention. Applied only to text already known to be a type position.
fn capitalized_idents(text: &str) -> BTreeSet<String> {
    let mut out = BTreeSet::new();
    let mut cur = String::new();
    for ch in text.chars().chain(std::iter::once(' ')) {
        if ch.is_ascii_alphanumeric() || ch == '_' {
            cur.push(ch);
        } else if cur.chars().next().is_some_and(|c| c.is_ascii_uppercase()) {
            out.insert(std::mem::take(&mut cur));
        } else {
            cur.clear();
        }
    }
    out
}

/// The transitive type closure of `Command` and `GameEvent`, plus every
/// referenced name that resolved to nothing.
fn protocol_closure(
    index: &BTreeMap<String, Decl>,
) -> (BTreeSet<String>, BTreeMap<String, BTreeSet<String>>) {
    let external: BTreeSet<&str> = EXTERNAL_TYPES.iter().copied().collect();
    let mut seen: BTreeSet<String> = BTreeSet::new();
    let mut unresolved: BTreeMap<String, BTreeSet<String>> = BTreeMap::new();
    let mut queue: Vec<String> = PROTOCOL_ROOTS.iter().map(|s| s.to_string()).collect();

    while let Some(name) = queue.pop() {
        if !seen.insert(name.clone()) {
            continue;
        }
        let Some(decl) = index.get(&name) else {
            continue;
        };
        let referenced_types = if decl.is_alias {
            // `pub type X = Vec<Y>;` — the whole target is a type position.
            capitalized_idents(&decl.traversal_body)
        } else {
            type_references(&decl.traversal_body)
        };
        for referenced in referenced_types {
            if external.contains(referenced.as_str()) {
                continue;
            }
            if index.contains_key(&referenced) {
                if !seen.contains(&referenced) {
                    queue.push(referenced);
                }
            } else {
                unresolved
                    .entry(referenced)
                    .or_default()
                    .insert(name.clone());
            }
        }
    }
    (seen, unresolved)
}

/// The digest pinned by `PROTOCOL_SCHEMA_FINGERPRINT`.
fn compute_fingerprint(index: &BTreeMap<String, Decl>, closure: &BTreeSet<String>) -> String {
    let mut hasher = blake3::Hasher::new();
    hasher.update(b"mtg-engine protocol schema v1\n");
    hasher.update(format!("types={}\n", closure.len()).as_bytes());
    // BTreeSet iterates sorted, so file order and walk order cannot move the digest.
    for name in closure {
        let decl = index.get(name).expect("closure members are indexed");
        hasher.update(name.as_bytes());
        hasher.update(b"\n");
        hasher.update(decl.hash_text.as_bytes());
        hasher.update(b"\n");
    }
    hasher.finalize().to_hex().to_string()
}

// ── Tests ────────────────────────────────────────────────────────────────────

/// Non-vacuity: the scanner found a real codebase.
///
/// Without this, a broken parser digests the empty set and every other test in
/// this file passes forever (SR-5 learned that the hard way).
#[test]
fn scanner_is_not_vacuous() {
    let ScanResult { index, by_root, .. } = index_declarations();
    assert!(
        index.len() >= MIN_INDEXED_TYPES,
        "indexed only {} pub types; the declaration scanner is broken (expected >= {})",
        index.len(),
        MIN_INDEXED_TYPES
    );
    for scan_root in SCAN_ROOTS {
        let declared = by_root.get(scan_root).map(|s| s.len()).unwrap_or(0);
        assert!(
            declared > 0,
            "scan root {scan_root} contributed no type declarations — per-root denominator guard \
             (SR-6): a cross-crate derived set needs a per-root floor, not just a total."
        );
    }
}

/// The index is keyed by bare type name and keeps the **first** declaration it
/// sees. That is sound only while names are unique across the scan roots.
///
/// If two modules ever declare the same name, the digest silently hashes
/// whichever file sorted first — and real changes to the wire-bearing one go
/// undetected. Nothing about the scan makes that visible, so assert it.
#[test]
fn declared_type_names_are_unique() {
    let ScanResult { collisions, .. } = index_declarations();
    assert!(
        collisions.is_empty(),
        "these type names are declared more than once under the scan roots: {collisions:?}\n\
         `index_declarations` keeps only the first, so the fingerprint may be hashing the wrong \
         declaration and would not move when the real wire type changes. Disambiguate the names, \
         or key the index by module path."
    );
}

/// An `EXTERNAL_TYPES` entry suppresses that *bare name* everywhere in the walk.
/// So if the workspace ever declares a type named `Vector`, `Box`, `OrdMap`, …,
/// it would be silently treated as somebody else's type and left out of the
/// digest, wire-bearing or not.
#[test]
fn no_workspace_type_shadows_an_external_type_name() {
    let ScanResult { index, .. } = index_declarations();
    let shadowed: Vec<&str> = EXTERNAL_TYPES
        .iter()
        .copied()
        .filter(|name| index.contains_key(*name))
        .collect();
    assert!(
        shadowed.is_empty(),
        "the workspace declares {shadowed:?}, which are also in EXTERNAL_TYPES. The closure walk \
         matches on bare names, so these types are skipped as 'external' and their shape is NOT \
         in the fingerprint. Rename the workspace type, or drop it from EXTERNAL_TYPES."
    );
}

/// Non-vacuity: the closure walk actually walked, and it bottoms out where the
/// design says it does.
#[test]
fn protocol_closure_is_not_vacuous_and_is_bounded() {
    let ScanResult { index, .. } = index_declarations();
    let (closure, _) = protocol_closure(&index);

    assert!(
        closure.len() >= MIN_CLOSURE_TYPES,
        "protocol closure is only {} types; the type-position walker is broken (expected >= {})",
        closure.len(),
        MIN_CLOSURE_TYPES
    );
    for required in CLOSURE_MUST_CONTAIN {
        assert!(
            closure.contains(required),
            "{required} is reachable from Command/GameEvent but missing from the computed \
             closure — the walker lost an edge and the fingerprint is now blind to {required}"
        );
    }
    for forbidden in CLOSURE_MUST_NOT_CONTAIN {
        assert!(
            !closure.contains(forbidden),
            "{forbidden} entered the Command/GameEvent closure. Whole-state types on the wire is \
             a deliberate design change: it merges the protocol-version and HASH_SCHEMA_VERSION \
             concerns. Decide it on purpose, then update CLOSURE_MUST_NOT_CONTAIN."
        );
    }
    // Both crates must contribute, or a whole crate's worth of shape is unhashed.
    assert!(
        closure.contains("Command") && closure.contains("Characteristics"),
        "closure must span both engine (Command) and card-types (Characteristics)"
    );
}

/// Every type the protocol reaches is either hashed or explicitly declared
/// external. This is the guard against silent under-inclusion — a wire-bearing
/// type that the digest simply never sees.
#[test]
fn every_referenced_type_resolves() {
    let ScanResult { index, .. } = index_declarations();
    let (_, unresolved) = protocol_closure(&index);
    assert!(
        unresolved.is_empty(),
        "these types are reachable from Command/GameEvent but are neither indexed nor listed in \
         EXTERNAL_TYPES, so their shape is NOT covered by PROTOCOL_SCHEMA_FINGERPRINT:\n{}\n\
         Either they belong in the scan roots, or add them to EXTERNAL_TYPES to state on the \
         record that another crate owns their serialized form.",
        unresolved
            .iter()
            .map(|(t, from)| format!(
                "  {t} (referenced by {})",
                from.iter().cloned().collect::<Vec<_>>().join(", ")
            ))
            .collect::<Vec<_>>()
            .join("\n")
    );
}

/// **The gate.** If this fails, the wire format moved.
#[test]
fn protocol_schema_fingerprint_is_pinned() {
    let ScanResult { index, .. } = index_declarations();
    let (closure, _) = protocol_closure(&index);
    let actual = compute_fingerprint(&index, &closure);

    assert_eq!(
        actual,
        PROTOCOL_SCHEMA_FINGERPRINT,
        "\n\nThe serialized shape of the Command/GameEvent type closure ({} types) has changed.\n\
         Old clients and old replay logs cannot be read by this build.\n\n\
         Do BOTH of these, in the same commit:\n  \
           1. bump PROTOCOL_VERSION in crates/engine/src/rules/protocol.rs \
              (currently {PROTOCOL_VERSION}), adding a History line saying what moved;\n  \
           2. set PROTOCOL_SCHEMA_FINGERPRINT to:\n       {actual}\n\n\
         If you believe the change is wire-compatible, read the \"What this gate does not catch\" \
         note at the top of this file first — a reorder of enum variants is the one case that is \
         genuinely compatible, and it still requires a bump here.\n",
        closure.len()
    );
}

/// Sentinel, mirroring the `HASH_SCHEMA_VERSION` convention: a bump must be
/// deliberate and must be seen in review, so it costs one more edit.
#[test]
fn protocol_version_sentinel() {
    assert_eq!(
        PROTOCOL_VERSION, 1,
        "PROTOCOL_VERSION changed. Update this sentinel and the History list in \
         rules/protocol.rs. If you bumped it *without* protocol_schema_fingerprint_is_pinned \
         failing, the wire shape did not change — make sure the bump is a deliberate semantic \
         change, not a reflex."
    );
}

/// The digest must not be the hash of nothing. `compute_fingerprint` over an
/// empty closure is a fixed string; if the pinned constant ever equals it, the
/// gate has gone vacuous while still reporting green.
#[test]
fn fingerprint_of_empty_closure_is_not_the_pinned_value() {
    let empty_index = BTreeMap::new();
    let empty_closure = BTreeSet::new();
    let empty = compute_fingerprint(&empty_index, &empty_closure);
    assert_ne!(
        empty, PROTOCOL_SCHEMA_FINGERPRINT,
        "PROTOCOL_SCHEMA_FINGERPRINT is the digest of an EMPTY closure — the scanner returned \
         nothing and the pin was updated to match it. Assert the denominator (SR-5)."
    );
}

/// Attributes are part of the wire, so they must be part of the digest.
///
/// This is not hypothetical: `#[serde(skip)]` on three `PendingTrigger` fields is
/// an open bug (`scutemob-68` / SR-16). A digest that ignored attributes would
/// have called that change wire-compatible.
#[test]
fn serde_attributes_are_inside_the_digest() {
    let ScanResult { index, .. } = index_declarations();
    let command = index.get("Command").expect("Command is indexed");
    assert!(
        command.hash_text.contains("#[serde(default)]"),
        "Command's hashed text lost its field-level serde attributes; \
         a #[serde(rename)] or #[serde(skip)] would then be invisible to the gate"
    );
    assert!(
        command.hash_text.contains("Serialize") && command.hash_text.contains("Deserialize"),
        "Command's hashed text lost its container #[derive(...)]; dropping Serialize \
         would then be invisible to the gate"
    );
    // ...and the noise that cannot affect the wire is excluded, so an #[allow]
    // churn does not force a protocol bump.
    assert!(
        !command.hash_text.contains("#[allow"),
        "#[allow(...)] leaked into the digest; it cannot affect the wire and would cause \
         spurious version bumps"
    );
}

/// Every non-alias type on the wire must show a `Serialize` derive in its hashed
/// text.
///
/// This is a denominator guard on `preceding_attributes`, which walks whole
/// lines upward and stops at the first line that is not a complete `#[…]`. A
/// rustfmt-wrapped derive (`#[derive(\n    Clone,\n)]`) would therefore be
/// dropped **silently**, taking the container's serde config out of the digest
/// with it. A type that reaches the wire without a visible `Serialize` means
/// either that happened, or the type is not actually serializable.
#[test]
fn every_closure_type_shows_its_serialize_derive() {
    let ScanResult { index, .. } = index_declarations();
    let (closure, _) = protocol_closure(&index);

    let missing: Vec<&String> = closure
        .iter()
        .filter(|name| {
            index
                .get(*name)
                .is_some_and(|d| !d.is_alias && !d.hash_text.contains("Serialize"))
        })
        .collect();

    assert!(
        missing.is_empty(),
        "these wire types have no `Serialize` in their hashed text: {missing:?}\n\
         Most likely `preceding_attributes` lost a multi-line #[derive(...)] — which means the \
         container's serde attributes are NOT in the fingerprint and a #[serde(rename_all)] \
         change would slip through the gate."
    );
}

/// The traversal view must not see attributes, or `#[serde(with = "Foo")]`-style
/// paths would inject phantom types into the closure.
#[test]
fn traversal_body_excludes_attributes() {
    let ScanResult { index, .. } = index_declarations();
    let command = index.get("Command").expect("Command is indexed");
    assert!(
        !command.traversal_body.contains("serde"),
        "attributes survived into the traversal body; type-position extraction will pick up \
         attribute arguments as if they were field types"
    );
}
