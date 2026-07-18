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
//! - **External types.** `imbl::OrdMap`, `Vec`, `Option` etc. are allowlisted in
//!   [`EXTERNAL_TYPES`]; a `Cargo.toml` bump that changes `im`'s serialized form
//!   moves the wire without moving this digest.
//! - **Variant reordering is a false positive, deliberately.** serde's external
//!   tagging keys on variant *names*, so a pure reorder is wire-compatible, yet
//!   it moves the digest because the digest hashes declaration text in order.
//!   The cost is one needless version bump; the alternative is a
//!   variant-sorting normalizer, which is more code that can be wrong in the
//!   *unsafe* direction. Accepted.
//! - **Formatting churn is a false positive too.** `normalize_ws` collapses runs
//!   of whitespace, but rustfmt rewrapping a long field type also *inserts
//!   tokens* — a trailing comma inside `Vector<\n AbilityInstance,\n>` — so the
//!   normalized text moves. Verified. `cargo fmt --check` is a CI gate, so the
//!   tree is always canonical for the pinned toolchain; this can therefore only
//!   fire when **rustfmt's version changes**, which is exactly what `scutemob-63`
//!   (SR-11, pin the toolchain) exists to prevent. Both false positives err in
//!   the safe direction: a spurious bump, never a missed one.
//!
//! Per the SR-5 lesson ("assert the denominator"), every derived set here has a
//! non-vacuity guard: an index that finds nothing, a closure that walks nowhere,
//! or a scan root that contributes nothing all fail loudly rather than digesting
//! the empty string forever.

use std::collections::{BTreeMap, BTreeSet};
use std::path::{Path, PathBuf};

use mtg_engine::{ProtocolEpoch, PROTOCOL_HISTORY, PROTOCOL_SCHEMA_FINGERPRINT, PROTOCOL_VERSION};

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

// ── SR-27: frozen baseline for the append-only PROTOCOL_HISTORY ───────────────
//
// These pin version 2's identity a *second* time, independently of
// `PROTOCOL_HISTORY`'s row in `rules/protocol.rs`. Re-pinning that shipped row
// (or `PROTOCOL_SCHEMA_FINGERPRINT`, which the tail row mirrors) without bumping
// `PROTOCOL_VERSION` makes `protocol_schema_fingerprint_is_pinned` pass again — but
// leaves the protocol.rs row disagreeing with these constants, so
// `history_is_append_only` fails. That is the whole point: it converts "remember to
// bump the version" from a process guarantee into a machine one (SR-8/SR-17 pattern).
//
// **FROZEN — do not edit.** Only ever *append* new rows to `PROTOCOL_HISTORY`.
const BASELINE_VERSION: u32 = 2;
const BASELINE_FINGERPRINT: &str =
    "ba7907d9f51a65acba39ccf020a14bd6234f637731c934490a7cbf749e5f97b6";

// Digest over the **frozen prefix** — every `PROTOCOL_HISTORY` row except the
// current (tail) one. The tail is the working row for the live schema, validated by
// recomputation (`protocol_schema_fingerprint_is_pinned`); every row behind it is
// shipped-and-superseded and must never change again. This freezes all of them at
// once, generalizing `history_is_append_only`'s baseline check forward to every
// future version.
//
// With a single history row the prefix is empty, so this pins the digest of the
// empty prefix; it becomes load-bearing on the first bump, when version 2 enters the
// prefix and its bytes lock here. On every bump you append a row AND re-pin this.
//
// **FROZEN — do not edit except by appending to `PROTOCOL_HISTORY`.**
// PB-EF9 (2026-07-18): re-pinned on the 13→14 bump — version 13 (the former
// tail) joined the frozen prefix when version 14 shipped. This is the
// digest of the twelve-row prefix `[version 2, version 3, version 4, version 5, version 6, version 7, version 8, version 9, version 10, version 11, version 12, version 13]`.
const FROZEN_HISTORY_PREFIX_DIGEST: &str =
    "648f47c35743fb50f826ba32ab25cabc1bdb73471eb6f7ca8c7b31593c96e343";

/// The `PROTOCOL_HISTORY` row pinning the current `PROTOCOL_VERSION`.
fn current_epoch() -> ProtocolEpoch {
    *PROTOCOL_HISTORY
        .iter()
        .find(|e| e.version == PROTOCOL_VERSION)
        .unwrap_or_else(|| {
            panic!(
                "PROTOCOL_HISTORY has no row for the current PROTOCOL_VERSION \
                 ({PROTOCOL_VERSION}). Append a row when you bump the version."
            )
        })
}

/// Digest over the frozen prefix — every `PROTOCOL_HISTORY` row except the tail.
fn compute_frozen_prefix_digest() -> String {
    let mut hasher = blake3::Hasher::new();
    hasher.update(b"mtg-engine protocol schema frozen-prefix v1\n");
    let n = PROTOCOL_HISTORY.len();
    for e in &PROTOCOL_HISTORY[..n.saturating_sub(1)] {
        hasher.update(&e.version.to_le_bytes());
        hasher.update(e.fingerprint.as_bytes());
    }
    hasher.finalize().to_hex().to_string()
}

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

/// The serde attribute keys that reshape the wire through a conversion type or
/// function whose *shape* the fingerprint cannot see (it hashes only the key's name
/// argument, not the referenced type/function). Longest-first is not required — the
/// argument-boundary check in `serde_conversion_attrs` disambiguates overlaps.
const CONVERSION_KEYS: [&str; 6] = [
    "with",
    "from",
    "into",
    "try_from",
    "serialize_with",
    "deserialize_with",
];

/// True iff `token` appears in `text` as a whole identifier — bounded on each side
/// by a non-`[A-Za-z0-9_]` byte (or a string boundary).
///
/// SR-27: the derive guards used bare `text.contains("Serialize")`, which a substring
/// satisfies — `Serializer`, `SerializeStruct`, or any identifier that merely *ends*
/// in the token would pass, and the check would no longer prove the wire type carries
/// the `Serialize` *derive*. Anchoring rejects the look-alikes. (The `"Deserialize"`
/// case named in the task is only a near-miss under ASCII case rules — `contains`
/// is case-sensitive — but `Serializer`/`SerializeStruct` are real substrings, so the
/// anchoring is not cosmetic; `serialize_guard_is_token_anchored` proves it.)
fn has_derive_token(text: &str, token: &str) -> bool {
    let b = text.as_bytes();
    let tlen = token.len();
    let is_ident = |c: u8| c.is_ascii_alphanumeric() || c == b'_';
    let mut from = 0;
    while let Some(rel) = text[from..].find(token) {
        let at = from + rel;
        let before_ok = at == 0 || !is_ident(b[at - 1]);
        let after = at + tlen;
        let after_ok = after >= b.len() || !is_ident(b[after]);
        if before_ok && after_ok {
            return true;
        }
        from = at + 1;
    }
    false
}

/// The serde *conversion* attributes present in `hash_text`: `with`, `from`, `into`,
/// `try_from`, `serialize_with`, `deserialize_with`. Returns the ones found (empty
/// if none), in the fixed `CONVERSION_KEYS` order.
///
/// Every one of these reshapes the wire through a *conversion type or function*
/// named only inside the attribute string. The digest hashes the attribute text, so
/// a change to that name is caught — but a change to the referenced type's *shape*
/// (or the function's serialized output) is not, because the scanner walks field
/// type positions, not attribute arguments. That is the documented residual;
/// `no_serde_conversion_attributes_in_closure` refuses to let one enter the closure
/// silently. `serialize_with`/`deserialize_with` are the same blind spot as `with`
/// (the bytes are produced by a function), so they are covered here too.
fn serde_conversion_attrs(hash_text: &str) -> Vec<&'static str> {
    // Compact away whitespace so `serde( with = ...` and `serde(with=...` look alike.
    let compact: String = hash_text
        .chars()
        .filter(|c| !c.is_ascii_whitespace())
        .collect();
    let bytes = compact.as_bytes();
    let mut hits = Vec::new();
    // Detect `<key>=` at a serde-attribute ARGUMENT boundary — the char before the
    // key is `(` or `,`. The boundary is what disambiguates the overlapping keys
    // without ordering tricks: in `try_from=` the `from` is preceded by `_`, and in
    // `serialize_with=` the `with` is preceded by `_`, so neither is mis-reported as
    // the shorter key. It also finds a conversion arg in any position, e.g. the
    // `into` in `#[serde(from="A", into="B")]`.
    for key in CONVERSION_KEYS {
        let pat = format!("{key}=");
        let mut from = 0;
        let mut found = false;
        while let Some(rel) = compact[from..].find(&pat) {
            let at = from + rel;
            if at > 0 && (bytes[at - 1] == b'(' || bytes[at - 1] == b',') {
                found = true;
                break;
            }
            from = at + 1;
        }
        if found {
            hits.push(key);
        }
    }
    hits
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
        current_epoch().fingerprint,
        "\n\nThe serialized shape of the Command/GameEvent type closure ({} types) has changed.\n\
         Old clients and old replay logs cannot be read by this build.\n\n\
         Do ALL of these, in the same commit:\n  \
           1. bump PROTOCOL_VERSION in crates/engine/src/rules/protocol.rs \
              (currently {PROTOCOL_VERSION}), adding a History line saying what moved;\n  \
           2. set PROTOCOL_SCHEMA_FINGERPRINT and APPEND a new PROTOCOL_HISTORY row \
              (do not edit the existing one) with fingerprint:\n       {actual}\n  \
           3. update protocol_version_sentinel and FROZEN_HISTORY_PREFIX_DIGEST here.\n\n\
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
        PROTOCOL_VERSION, 14,
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
/// This is not hypothetical: three `PendingTrigger` fields were `#[serde(skip)]`
/// (`scutemob-68` / SR-16, since removed). A digest that ignored attributes would
/// have called adding or removing that skip wire-compatible when it is not.
/// (`PendingTrigger` itself is inside `GameState`, so it never reaches *this*
/// closure — see `CLOSURE_MUST_NOT_CONTAIN` — but the same reasoning holds for any
/// `#[serde(skip)]` on a type that does.)
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
        has_derive_token(&command.hash_text, "Serialize")
            && has_derive_token(&command.hash_text, "Deserialize"),
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
                .is_some_and(|d| !d.is_alias && !has_derive_token(&d.hash_text, "Serialize"))
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

// ── SR-27: append-only PROTOCOL_HISTORY (mirror of SR-17's HASH_SCHEMA_HISTORY) ──

/// The tail row and the standalone `PROTOCOL_SCHEMA_FINGERPRINT` const are two
/// hand-written copies of the same digest; keep them in lockstep so `encode`/doc
/// consumers of the const and the history agree.
#[test]
fn history_tail_matches_the_fingerprint_const() {
    let tail = PROTOCOL_HISTORY
        .last()
        .expect("PROTOCOL_HISTORY is non-empty");
    assert_eq!(
        tail.fingerprint, PROTOCOL_SCHEMA_FINGERPRINT,
        "the tail PROTOCOL_HISTORY row's fingerprint disagrees with PROTOCOL_SCHEMA_FINGERPRINT. \
         When you re-pin the fingerprint you must set BOTH (and, if the wire moved, bump the \
         version and APPEND a row rather than editing the tail)."
    );
}

/// **The re-pin gate.** `PROTOCOL_HISTORY` is append-only and current.
///
/// - non-empty, versions strictly ascending and unique, tail == current version;
/// - every fingerprint is 64 lowercase hex, and all are pairwise distinct;
/// - the baseline row (version 2) equals the FROZEN constants above, so a re-pin of
///   that shipped row (or of `PROTOCOL_SCHEMA_FINGERPRINT`, which the tail mirrors)
///   *without* a version bump disagrees here and fails — the guarantee the plain
///   `protocol_version_sentinel` could not make.
#[test]
fn history_is_append_only() {
    assert!(
        !PROTOCOL_HISTORY.is_empty(),
        "PROTOCOL_HISTORY is empty — there is nothing pinning PROTOCOL_VERSION"
    );

    for w in PROTOCOL_HISTORY.windows(2) {
        assert!(
            w[1].version > w[0].version,
            "PROTOCOL_HISTORY is not strictly ascending / unique in version: {} then {}. \
             It is append-only — add new rows with higher versions, never reorder or duplicate.",
            w[0].version,
            w[1].version
        );
    }

    let last = PROTOCOL_HISTORY.last().expect("non-empty");
    assert_eq!(
        last.version, PROTOCOL_VERSION,
        "the last PROTOCOL_HISTORY row is version {}, but PROTOCOL_VERSION is {}. Append a row \
         for the current version (do not edit an existing one).",
        last.version, PROTOCOL_VERSION
    );

    let is_hex64 = |s: &str| {
        s.len() == 64
            && s.bytes()
                .all(|b| b.is_ascii_hexdigit() && !b.is_ascii_uppercase())
    };
    let mut seen_fingerprints = std::collections::BTreeSet::new();
    for e in PROTOCOL_HISTORY {
        assert!(
            is_hex64(e.fingerprint),
            "version {} has a malformed fingerprint (expected 64 lowercase hex chars)",
            e.version
        );
        // Fingerprints are pairwise distinct: two versions with the same shape digest
        // is a copy-paste (an appended row that never got its recomputed value) or a
        // bump with no wire change — either way the row carries no new information and
        // should not exist.
        assert!(
            seen_fingerprints.insert(e.fingerprint),
            "PROTOCOL_HISTORY version {} repeats a fingerprint already used by an earlier \
             row. Each version pins a distinct wire shape; a duplicate means a row was \
             appended without its recomputed digest, or a version was bumped with no wire \
             change (which needs no new row).",
            e.version
        );
    }

    let baseline = PROTOCOL_HISTORY
        .iter()
        .find(|e| e.version == BASELINE_VERSION)
        .expect("baseline version 2 row is present");
    assert_eq!(
        baseline.fingerprint, BASELINE_FINGERPRINT,
        "\n\nThe shipped version-{BASELINE_VERSION} row in PROTOCOL_HISTORY no longer matches the \
         FROZEN baseline constant in tests/core/protocol_schema.rs.\n\
         This is what a 're-pin without a bump' looks like: someone changed the wire shape, then \
         edited the version-{BASELINE_VERSION} fingerprint in place (or re-pinned \
         PROTOCOL_SCHEMA_FINGERPRINT, which the tail mirrors) instead of bumping PROTOCOL_VERSION \
         and appending a row. Rewriting a shipped row's identity is forbidden — bump and append.\n"
    );
}

/// **Append-only, generalized.** Every shipped-and-superseded row — the whole
/// history except the current tail — is frozen by a single digest. This carries the
/// baseline check forward to every version, not just 2: after a bump, the
/// newly-superseded row joins the prefix and locks here. Editing any past row in
/// place moves this digest and fails; a clean append leaves the pre-existing prefix
/// bytes untouched.
#[test]
fn frozen_prefix_is_pinned() {
    assert_eq!(
        compute_frozen_prefix_digest(),
        FROZEN_HISTORY_PREFIX_DIGEST,
        "\n\nThe frozen prefix of PROTOCOL_HISTORY (every row except the current tail) changed.\n\
         Either a shipped, superseded row was edited in place — forbidden, the history is \
         append-only — or you just bumped the version and a row correctly joined the prefix. If \
         the latter, re-pin FROZEN_HISTORY_PREFIX_DIGEST in tests/core/protocol_schema.rs to:\n\
         \x20      {}\n",
        compute_frozen_prefix_digest()
    );
}

// ── SR-27: token-anchored derive guard + serde-conversion gate ────────────────

/// The `Serialize`/`Deserialize` derive guards must match whole identifier tokens,
/// not substrings — otherwise a `Serializer` field or a `SerializeStruct` variant
/// name silences them.
#[test]
fn serialize_guard_is_token_anchored() {
    // The look-alikes a bare `.contains("Serialize")` would wrongly accept:
    assert!("pub struct Serializer".contains("Serialize"));
    assert!(!has_derive_token("pub struct Serializer", "Serialize"));
    assert!("enum E { SerializeStruct }".contains("Serialize"));
    assert!(!has_derive_token("enum E { SerializeStruct }", "Serialize"));

    // The real derive, in every spelling that reaches the digest, still matches:
    assert!(has_derive_token(
        "#[derive(Clone, Serialize, Deserialize)]",
        "Serialize"
    ));
    assert!(has_derive_token("#[derive(serde::Serialize)]", "Serialize"));
    assert!(has_derive_token("#[derive(Deserialize)]", "Deserialize"));
    // A container with ONLY Deserialize must NOT read as having Serialize:
    assert!(!has_derive_token(
        "#[derive(Clone, Deserialize)]",
        "Serialize"
    ));
}

/// The serde-conversion scan actually detects the attributes it is meant to reject —
/// the non-vacuity proof for `no_serde_conversion_attributes_in_closure`, whose real
/// assertion is an *absence* (the closure has none today). A compile-valid
/// `serde(with/from/into)` needs a companion module or `From`/`Into` impl that a text
/// patch cannot inject, so the positive case is proven here rather than by an
/// adversarial source edit.
#[test]
fn serde_conversion_scan_detects_the_attribute() {
    assert_eq!(
        serde_conversion_attrs("pub struct X { }"),
        Vec::<&str>::new()
    );
    assert_eq!(
        serde_conversion_attrs("#[serde(with = \"ser_mod\")] pub struct X { }"),
        vec!["with"]
    );
    assert_eq!(
        serde_conversion_attrs("#[serde(from = \"u32\")] pub struct X(u32);"),
        vec!["from"]
    );
    assert_eq!(
        serde_conversion_attrs("#[serde(try_from = \"u32\")] pub struct X(u32);"),
        vec!["try_from"]
    );
    assert_eq!(
        serde_conversion_attrs("#[serde(into = \"u32\")] pub struct X(u32);"),
        vec!["into"]
    );
    // serialize_with / deserialize_with — the field-level conversion *functions*,
    // the same digest blind spot as `with` and the ones the plain-needle version
    // missed. Their `with` / their `_with=` boundary must not be mis-read as `with`.
    assert_eq!(
        serde_conversion_attrs("#[serde(serialize_with = \"s\")] pub x: u8,"),
        vec!["serialize_with"]
    );
    assert_eq!(
        serde_conversion_attrs("#[serde(deserialize_with = \"d\")] pub x: u8,"),
        vec!["deserialize_with"]
    );
    // A conversion arg in the SECOND position (after a comma) is still found, and
    // both members of a combined attr are reported — not just the first.
    assert_eq!(
        serde_conversion_attrs("#[serde(from = \"A\", into = \"B\")] pub struct X;"),
        vec!["from", "into"]
    );
    assert_eq!(
        serde_conversion_attrs("#[serde(serialize_with=\"s\",deserialize_with=\"d\")] pub x: u8,"),
        vec!["serialize_with", "deserialize_with"]
    );
    // The benign, non-conversion serde attrs must NOT trip it — including
    // `skip_serializing_if`, whose `serializing` must not read as `serialize_with`.
    assert_eq!(
        serde_conversion_attrs(
            "#[serde(default)] #[serde(skip)] #[serde(rename = \"x\")] \
             #[serde(rename_all = \"snake_case\")] #[serde(skip_serializing_if = \"f\")]"
        ),
        Vec::<&str>::new()
    );
}

/// **The gate.** No `serde(with|from|into|try_from|serialize_with|deserialize_with)`
/// attribute may appear anywhere in the wire closure. Such an attribute reshapes the
/// wire through a conversion type or function the digest tracks only by *name*, not by
/// *shape* — a documented blind spot. This keeps it from going live silently: adding
/// one fails here, forcing a deliberate decision (and, if kept, walking the conversion
/// type into the digest).
#[test]
fn no_serde_conversion_attributes_in_closure() {
    let ScanResult { index, .. } = index_declarations();
    let (closure, _) = protocol_closure(&index);

    // Denominator guard: a broken closure walk must not pass this vacuously.
    assert!(
        closure.len() >= MIN_CLOSURE_TYPES,
        "closure is only {} types; the walk is broken and this gate would be vacuous",
        closure.len()
    );

    let offenders: Vec<String> = closure
        .iter()
        .filter_map(|name| {
            let decl = index.get(name)?;
            let attrs = serde_conversion_attrs(&decl.hash_text);
            (!attrs.is_empty()).then(|| format!("{name}: serde({})", attrs.join(", ")))
        })
        .collect();

    assert!(
        offenders.is_empty(),
        "these wire-closure types carry a serde conversion attribute:\n  {}\n\
         serde(with/from/into/try_from/serialize_with/deserialize_with) reshapes the wire via a \
         conversion type or function the fingerprint only tracks by name, not by shape — so that \
         type's fields can drift silently. If this is \
         intended, walk the conversion type into the closure (add it to the scan / roots) and \
         update this gate; otherwise remove the attribute.",
        offenders.join("\n  ")
    );
}
