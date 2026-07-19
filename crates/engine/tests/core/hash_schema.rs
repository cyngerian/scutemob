//! SR-17: the anti-rot gate behind `HASH_SCHEMA_VERSION`.
//!
//! This is the state-hash analogue of SR-8's `protocol_schema.rs`. SR-8 named the
//! disease — "a hand-bumped version constant next to a growing type is correct
//! only while every future author remembers it" — and cured it for the
//! `Command`/`GameEvent` **protocol** wire, whose `CLOSURE_MUST_NOT_CONTAIN`
//! *deliberately* excludes `GameState`. `HASH_SCHEMA_VERSION` was left with the
//! same disease: guarded only by ~29 `assert_eq!(HASH_SCHEMA_VERSION, 39)`
//! sentinels that force you to *notice* a bump (a sentinel reddens) but never to
//! *make* one. Change the serialized shape of `GameState`, or edit a `HashInto`
//! impl, and the number keeps lying while every sentinel stays green.
//!
//! M10 replay logs and rewind snapshots key on this number. A forgotten bump lets
//! this build accept an incompatible `ReplayLog` / snapshot; the corruption then
//! surfaces far from its cause (invariant #9). So the version is pinned by two
//! digests, both recomputed here and compared against
//! [`mtg_engine::HASH_SCHEMA_HISTORY`]'s row for the current version.
//!
//! ## Two axes, two digests — because they move independently
//!
//! The serialized *shape* of `GameState` and the *byte stream* its `HashInto`
//! impls feed are two different things, and either can move without the other
//! (SR-16 is the worked example: it changed `PendingTrigger`'s serde shape while
//! the hash stream was provably unchanged). One digest cannot cover both:
//!
//! 1. **`decl_fingerprint`** — a source scan of the `GameState` **serde** type
//!    closure. It indexes every `pub enum`/`struct`/`type` under the scan roots,
//!    walks the type positions transitively from `GameState`, and digests the
//!    normalized declaration text (attributes included). Catches a new/removed/
//!    retyped field, a new enum variant, a `#[serde(skip|rename|default)]`
//!    toggle. **Blind to `HashInto`** — those impls are hand-written code, not
//!    type declarations.
//! 2. **`stream_fingerprint`** — blake3 of the actual hash bytes
//!    (`public_state_hash` ++ every player's `private_state_hash`) over a fixed,
//!    richly-populated fixture. Catches a reordered / added / dropped `HashInto`
//!    feed or a changed discriminant byte. **Blind to serde-only shape** — a
//!    `#[serde(rename)]` never reaches the hasher.
//!
//! ## Serde closure, not hash closure — and skip-awareness
//!
//! The declaration digest tracks what `GameState` *serializes* (a rewind snapshot
//! writes the whole struct), so it includes `history: Vector<GameEvent>` even
//! though the hash stream excludes it, and it **excludes** `card_registry`, which
//! is `#[serde(skip)]` and reconstructed on load. That skip-awareness is a
//! deliberate divergence from SR-8's scanner (whose protocol roots have no bare
//! `#[serde(skip)]` field pointing off-closure): a bare `#[serde(skip)]` named
//! field is dropped from the *traversal* view (its type never enters the
//! closure — `card_registry` otherwise drags in `CardRegistry` → `CardDefinition`
//! → the entire card DSL, none of which is on the state wire) while the field and
//! its attribute stay in the *hashed* text (so adding or removing the skip is
//! itself caught). `serde_skip_is_load_bearing` proves the divergence is real.
//!
//! ## Disjoint from the protocol closure
//!
//! SR-8 asserts its closure does not contain `GameState`; this asserts the mirror
//! boundary from the state side — the `GameState` serde closure must not contain
//! the protocol's exclusive wire frames (`Command`, `ReplayLog`, `Envelope`). The
//! two closures *overlap* on the shared card DSL (`Effect`, `Characteristics`) and
//! on `GameEvent` (state's `history` is a `Vector<GameEvent>`), which is expected
//! and correct — a `GameEvent` shape change legitimately moves both versions. The
//! boundary that keeps the two *version concerns* separable is that neither
//! whole-frame leaks into the other; see [`CLOSURE_MUST_NOT_CONTAIN`].
//!
//! Per the SR-5 lesson ("assert the denominator"), every derived set here has a
//! non-vacuity guard: an index that finds nothing, a closure that walks nowhere, a
//! scan root that contributes nothing, an empty-closure digest, or a fixture that
//! hashes to the empty stream all fail loudly rather than passing forever.

use std::collections::{BTreeMap, BTreeSet};
use std::path::{Path, PathBuf};

use mtg_engine::{
    CardType, Color, ContinuousEffect, CounterType, EffectDuration, EffectFilter, EffectId,
    EffectLayer, GameState, GameStateBuilder, HashSchemaEpoch, KeywordAbility, LayerModification,
    ManaColor, ManaPool, ObjectSpec, PlayerId, Step, SubType, SuperType, ZoneId,
    HASH_SCHEMA_HISTORY, HASH_SCHEMA_VERSION,
};

/// Crates whose types may appear in a serialized `GameState`. `card-defs` is
/// deliberately absent: the card *definitions* live behind `#[serde(skip)]
/// card_registry` and are reconstructed on load, never serialized with the state.
const SCAN_ROOTS: [&str; 2] = ["crates/engine/src", "crates/card-types/src"];

/// The single root of the state serde closure. `GameState` is the whole
/// serialized unit (a rewind snapshot is one of these); everything the hash and
/// the wire cover is reachable from it.
const STATE_ROOTS: [&str; 1] = ["GameState"];

/// Types whose serialized shape is owned by someone else (std, `imbl`). Anything
/// reachable from `GameState` that is neither indexed nor listed here fails
/// `every_referenced_type_resolves` — the guard against a silent under-inclusion.
const EXTERNAL_TYPES: [&str; 24] = [
    "u8", "u16", "u32", "u64", "usize", "i8", "i16", "i32", "i64", "isize", "f32", "f64", "bool",
    "char", "str", "String", "Vec", "Option", "Box", "Arc", "Rc", "OrdMap", "OrdSet", "Vector",
];

/// Floors for the non-vacuity guards. Deliberately well below the real values —
/// they catch a scanner that broke, not a codebase that grew.
const MIN_INDEXED_TYPES: usize = 150;
const MIN_CLOSURE_TYPES: usize = 90;

/// Types that must be in the `GameState` serde closure. If one vanishes the
/// walker lost an edge and the digest went blind to it.
///
/// `GameEvent` proves `history: Vector<GameEvent>` is walked; `Effect` /
/// `Characteristics` prove the walk crosses into `card-types` and down through the
/// card DSL; `PendingTrigger` proves the `pending_triggers` payload is covered.
const CLOSURE_MUST_CONTAIN: [&str; 13] = [
    "GameState",
    "TurnState",
    "PlayerState",
    "GameObject",
    "StackObject",
    "CombatState",
    "Characteristics",
    "Effect",
    "KeywordAbility",
    "ManaCost",
    "GameEvent",
    "PendingTrigger",
    "TriggerData",
];

/// Types that must **not** be in the `GameState` serde closure.
///
/// The first three are the protocol's exclusive wire frames — the mirror of
/// SR-8's `CLOSURE_MUST_NOT_CONTAIN`, which keeps `GameState` out of the protocol
/// closure. If a `Command` or `ReplayLog` ever became reachable from `GameState`,
/// the state-version and protocol-version concerns would merge, and that must be a
/// deliberate decision, not a silent edge.
///
/// `CardRegistry` / `CardDefinition` are the skip-awareness guard: they are
/// reachable *only* through `#[serde(skip)] card_registry`, so a skip-blind
/// traversal would drag the entire card DSL into the closure. Their absence proves
/// the skip is honoured. `serde_skip_is_load_bearing` proves it is the skip doing
/// the work, not a broken walk.
const CLOSURE_MUST_NOT_CONTAIN: [&str; 5] = [
    "Command",
    "ReplayLog",
    "Envelope",
    "CardRegistry",
    "CardDefinition",
];

// ── Frozen baseline (append-only anchor) ─────────────────────────────────────
//
// These pin version 39's identity a *second* time, independently of
// `HASH_SCHEMA_HISTORY[0]` in `state/hash.rs`. Re-pinning a shipped row there
// without bumping the version makes `declaration_fingerprint_is_pinned` /
// `stream_fingerprint_is_pinned` pass again — but leaves the hash.rs row
// disagreeing with these constants, so `baseline_row_is_frozen` fails. To move
// them you must edit a block explicitly labelled FROZEN, which is the loud,
// reviewable signal that you are rewriting shipped history rather than appending.
//
// **FROZEN — do not edit.** Only ever add *new* rows to `HASH_SCHEMA_HISTORY`.
const BASELINE_VERSION: u8 = 39;
const BASELINE_DECL_FINGERPRINT: &str =
    "9398dee6d2338d30b7c4bf02f769d8f3654b10ccd9ee38fd0afdcf11223b5419";
const BASELINE_STREAM_FINGERPRINT: &str =
    "4f335df79a80bbd3b3bbafe14b223cfdeb5c479a6e037eefafd29f0c5d635976";

// Digest over the **frozen prefix** — every `HASH_SCHEMA_HISTORY` row except the
// current (tail) one. The tail is the working row for the live schema and is
// validated by recomputation (`declaration_fingerprint_is_pinned` /
// `stream_fingerprint_is_pinned`); every row behind it is *shipped and
// superseded* and must never change again. This digest freezes all of them at
// once, generalizing `baseline_row_is_frozen` (which anchors only version 39)
// forward to every future version.
//
// With a single history row the prefix is empty, so this pins the digest of the
// empty prefix; it becomes load-bearing on the first bump, when version 39 enters
// the prefix and its bytes lock here. On every bump you append a row AND re-pin
// this (the newly-superseded row joins the prefix) — a deliberate, reviewed edit.
//
// Residual, documented honestly: the *tail* row is not frozen (it cannot be — the
// gates establish it from source and fixture), so re-pinning the *current*
// version's fingerprints in place is caught only while the current version is the
// frozen baseline (39 today). Every future change MUST append, never edit the
// tail.
//
// **FROZEN — do not edit except by appending to `HASH_SCHEMA_HISTORY`.**
// PB-OS4 COMMIT 2 (2026-07-19): re-pinned on the 56→57 bump — version 56
// became a superseded row and joined the frozen prefix. Its bytes (the v56
// fingerprints) are unchanged; the digest moved only because the prefix
// gained a member.
const FROZEN_HISTORY_PREFIX_DIGEST: &str =
    "9630d61a88a20ac1a9d7b172c924f6f4b72dfabb8c3302745ead71b9618bf058";

/// The workspace root: `crates/engine/` is two levels down from it.
fn workspace_root() -> PathBuf {
    PathBuf::from(env!("CARGO_MANIFEST_DIR"))
        .parent()
        .and_then(Path::parent)
        .expect("engine manifest dir is <workspace>/crates/engine")
        .to_path_buf()
}

// ── Source scanning (adapted from SR-8 `protocol_schema.rs`) ──────────────────

/// Length of the string/char literal starting at `b[i]`, or `None`. Handles raw
/// strings. Literals are *skipped*, never blanked: a `#[serde(rename = "x")]` is
/// wire format and must survive into the digest.
fn literal_len(b: &[u8], i: usize) -> Option<usize> {
    let n = b.len();
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
fn strip_comments(src: &str) -> String {
    let b = src.as_bytes();
    let n = b.len();
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

/// Index of the byte just past the delimiter matching the one at `open`, skipping
/// string literals.
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

/// A `pub enum` / `pub struct` / `pub type` declaration.
struct Decl {
    /// Attributes + `pub enum Name {…}`, whitespace-normalized. Includes every
    /// serde attribute and the full body, so any wire-visible change moves it.
    hash_text: String,
    /// Body with attributes removed *and* bare-`#[serde(skip)]` fields dropped —
    /// used only to find the type references that make up the closure.
    traversal_body: String,
    /// `pub type X = Y;`. Aliases are transparent to serde.
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

/// True iff `attr` is exactly a bare `#[serde(skip)]` — the attribute that drops
/// a field from **both** serialize and deserialize, so its type is not on the
/// wire at all.
///
/// Whitespace-insensitive but otherwise exact: `#[serde(skip_serializing_if =
/// "…")]` (still conditionally on the wire) and `#[serde(default)]` do **not**
/// match, so their fields stay in the closure.
fn is_bare_serde_skip(attr: &str) -> bool {
    let compact: String = attr.chars().filter(|c| !c.is_ascii_whitespace()).collect();
    compact == "#[serde(skip)]"
}

/// Blank the *type position* of every named field carrying a bare
/// `#[serde(skip)]`, so it contributes no types to the closure walk.
///
/// Operates on the comment-stripped, attribute-bearing struct body. The field's
/// name and colon are left in place; only the type text (up to the field
/// terminator `,`/`}`, delimiter-aware) is replaced with spaces. `strip_attributes`
/// then removes the attribute itself. The field remains present in `hash_text`
/// (built from the untouched body), so toggling the skip still moves the digest.
fn blank_serde_skip_field_types(body: &str) -> String {
    let bytes = body.as_bytes();
    let n = bytes.len();
    let mut out: Vec<u8> = body.bytes().collect();
    let mut i = 0;
    while i < n {
        if let Some(len) = literal_len(bytes, i) {
            i += len;
            continue;
        }
        if bytes[i] == b'#' && i + 1 < n && bytes[i + 1] == b'[' {
            let end = match_delim(bytes, i + 1, b'[', b']');
            if is_bare_serde_skip(&body[i..end]) {
                // Find the field's type colon: first `:` after the attribute that
                // is not part of a `::` path.
                let mut j = end;
                while j < n {
                    if let Some(len) = literal_len(bytes, j) {
                        j += len;
                        continue;
                    }
                    if bytes[j] == b':'
                        && bytes.get(j + 1) != Some(&b':')
                        && (j == 0 || bytes[j - 1] != b':')
                    {
                        break;
                    }
                    j += 1;
                }
                // Blank the type up to the field terminator.
                let mut depth = 0usize;
                let mut k = j + 1;
                while k < n {
                    match bytes[k] {
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
                    k += 1;
                }
                for slot in out.iter_mut().take(k).skip(j + 1) {
                    *slot = b' ';
                }
            }
            i = end;
            continue;
        }
        i += 1;
    }
    String::from_utf8(out).expect("blanking preserves UTF-8 boundaries")
}

/// Container attributes immediately above `decl_start`, minus `#[allow(…)]`.
/// Bracket-matched, not line-based (rustfmt wraps a long `#[derive(...)]` across
/// lines; a line walk would silently drop the whole derive — SR-8's
/// `every_closure_type_shows_its_serialize_derive` caught exactly that).
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

/// What a source scan yields: the type index, the per-root denominators, and any
/// name declared more than once.
struct ScanResult {
    index: BTreeMap<String, Decl>,
    by_root: BTreeMap<String, BTreeSet<String>>,
    collisions: BTreeSet<String>,
}

/// Every `pub enum` / `pub struct` / `pub type` under the scan roots.
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
                        let eq = match src[after + name.len()..].find('=') {
                            Some(p) => after + name.len() + p + 1,
                            None => continue,
                        };
                        let semi = src[eq..].find(';').map(|p| eq + p).unwrap_or(src.len());
                        src[eq..semi].to_string()
                    } else {
                        let mut j = after + name.len();
                        while j < b.len() && b[j] != b'{' && b[j] != b'(' && b[j] != b';' {
                            j += 1;
                        }
                        if j >= b.len() || b[j] == b';' {
                            String::new()
                        } else {
                            let (o, c) = if b[j] == b'{' {
                                (b'{', b'}')
                            } else {
                                (b'(', b')')
                            };
                            src[j..match_delim(b, j, o, c)].to_string()
                        }
                    };

                    let traversal_body = if is_alias {
                        strip_attributes(&body)
                    } else {
                        strip_attributes(&blank_serde_skip_field_types(&body))
                    };

                    names.insert(name.clone());
                    if index.contains_key(&name) {
                        collisions.insert(name.clone());
                    }
                    index.entry(name.clone()).or_insert_with(|| Decl {
                        hash_text: normalize_ws(&format!(
                            "{} {}{} {}",
                            preceding_attributes(&src, at),
                            kw,
                            name,
                            body
                        )),
                        traversal_body,
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

/// Type references in a declaration body, from **type positions only**: the text
/// after `:` in a named field, and the contents of a tuple variant's parentheses.
fn type_references(body: &str) -> BTreeSet<String> {
    let b = body.as_bytes();
    let n = b.len();
    let mut spans: Vec<&str> = Vec::new();
    let mut i = 0;
    while i < n {
        if let Some(len) = literal_len(b, i) {
            i += len;
        } else if b[i] == b':' && i + 1 < n && b[i + 1] != b':' && (i == 0 || b[i - 1] != b':') {
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

/// Identifiers starting with an uppercase letter — type names, by convention.
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

/// The transitive serde-type closure of `GameState`, plus every referenced name
/// that resolved to nothing.
fn state_closure(
    index: &BTreeMap<String, Decl>,
) -> (BTreeSet<String>, BTreeMap<String, BTreeSet<String>>) {
    let external: BTreeSet<&str> = EXTERNAL_TYPES.iter().copied().collect();
    let mut seen: BTreeSet<String> = BTreeSet::new();
    let mut unresolved: BTreeMap<String, BTreeSet<String>> = BTreeMap::new();
    let mut queue: Vec<String> = STATE_ROOTS.iter().map(|s| s.to_string()).collect();

    while let Some(name) = queue.pop() {
        if !seen.insert(name.clone()) {
            continue;
        }
        let Some(decl) = index.get(&name) else {
            continue;
        };
        let referenced_types = if decl.is_alias {
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

/// The declaration digest pinned by the current row's `decl_fingerprint`.
fn compute_decl_fingerprint(index: &BTreeMap<String, Decl>, closure: &BTreeSet<String>) -> String {
    let mut hasher = blake3::Hasher::new();
    hasher.update(b"mtg-engine hash schema decl v1\n");
    hasher.update(format!("types={}\n", closure.len()).as_bytes());
    for name in closure {
        let decl = index.get(name).expect("closure members are indexed");
        hasher.update(name.as_bytes());
        hasher.update(b"\n");
        hasher.update(decl.hash_text.as_bytes());
        hasher.update(b"\n");
    }
    hasher.finalize().to_hex().to_string()
}

// ── Canonical fixture + hash-stream digest ───────────────────────────────────

/// A fixed, richly-populated `GameState` whose hash stream is pinned by the
/// current row's `stream_fingerprint`.
///
/// Built purely constructively (no `process_command`), so the digest moves only
/// on a `HashInto` change or a state-shape change — never on an unrelated rules
/// edit. It spreads objects across battlefield / hand / graveyard / library /
/// exile / command zones and gives them varied characteristics (counters, tap
/// status, damage, keywords, loyalty, types, abilities), and gives the players
/// varied life / poison / mana, plus one `ContinuousEffect`. That exercises the
/// two largest `HashInto` impls (`GameObject`/`Characteristics` and `PlayerState`)
/// plus `TurnState`, `Zone`, `ManaPool`, `ContinuousEffect` (and its `EffectFilter`
/// / `LayerModification` / `EffectLayer` / `EffectDuration` sub-impls), and both
/// the public and private hash paths.
///
/// **Coverage cap (logged, not silent — SR track rule):** the builder cannot
/// populate `stack_objects`, `combat`, `pending_triggers`, `replacement_effects`,
/// or `lki_objects` without `process_command` (which would couple the digest to
/// rules semantics rather than to the hash schema). A *pure* `HashInto` reorder
/// *within* one of those five impls — feeding an unchanged struct's fields to the
/// hasher in a different order — is therefore caught by **neither** axis: the
/// declaration digest is blind to `HashInto` by construction, and this fixture
/// never populates those zones. That is a genuine residual gap, not one the
/// declaration digest closes. The common, most-edited impls are covered;
/// `stream_is_sensitive` proves the digest reacts to the fixture. Closing the gap
/// means either injecting literals for those types (verbose, and every field
/// addition would then force a re-pin regardless of hashing) or driving a fixed
/// command sequence (couples the digest to rules semantics); both were judged not
/// worth the fragility for the marginal five impls.
fn canonical_fixture() -> GameState {
    let mut mana = ManaPool::default();
    mana.add(ManaColor::Green, 2);
    mana.add(ManaColor::Red, 1);
    GameStateBuilder::four_player()
        .at_step(Step::PreCombatMain)
        .active_player(PlayerId(2))
        .turn_number(7)
        .player_life(PlayerId(1), 22)
        .player_life(PlayerId(3), 9)
        .player_poison(PlayerId(3), 4)
        .player_mana(PlayerId(2), mana)
        // Battlefield: a counter-laden tapped creature with damage.
        .object(
            ObjectSpec::creature(PlayerId(1), "Grizzly Bear", 2, 2)
                .tapped()
                .with_counter(CounterType::PlusOnePlusOne, 3)
                .with_damage(1)
                .with_types(vec![CardType::Creature])
                .with_subtypes(vec![SubType("Bear".to_string())])
                .with_colors(vec![Color::Green]),
        )
        // Battlefield: an evasive legendary creature.
        .object(
            ObjectSpec::creature(PlayerId(2), "Serra Angel", 4, 4)
                .with_keyword(KeywordAbility::Flying)
                .with_keyword(KeywordAbility::Vigilance)
                .with_supertypes(vec![SuperType::Legendary]),
        )
        .object(ObjectSpec::land(PlayerId(1), "Forest"))
        .object(ObjectSpec::artifact(PlayerId(3), "Sol Ring"))
        .object(ObjectSpec::planeswalker(PlayerId(4), "Jace Beleren", 5))
        // Non-public zones drive the private hash and the zone spread.
        .object(
            ObjectSpec::creature(PlayerId(2), "Llanowar Elves", 1, 1)
                .in_zone(ZoneId::Hand(PlayerId(2))),
        )
        .object(ObjectSpec::card(PlayerId(1), "Lightning Bolt").in_zone(ZoneId::Hand(PlayerId(1))))
        .object(ObjectSpec::card(PlayerId(1), "Mountain").in_zone(ZoneId::Library(PlayerId(1))))
        .object(
            ObjectSpec::creature(PlayerId(3), "Dead Bear", 2, 2)
                .in_zone(ZoneId::Graveyard(PlayerId(3))),
        )
        .object(ObjectSpec::card(PlayerId(4), "Exiled Card").in_zone(ZoneId::Exile))
        // A continuous effect, so the `ContinuousEffect` HashInto family is in the
        // stream digest too (the builder can add this one without process_command).
        .add_continuous_effect(ContinuousEffect {
            id: EffectId(1000),
            source: None,
            timestamp: 1000,
            layer: EffectLayer::PtSet,
            duration: EffectDuration::UntilEndOfTurn,
            filter: EffectFilter::AllCreatures,
            modification: LayerModification::SetPowerToughness {
                power: 3,
                toughness: 3,
            },
            is_cda: false,
            condition: None,
        })
        .build()
        .expect("canonical fixture builds")
}

/// blake3 of the fixture's full hash surface: the public hash followed by every
/// player's private hash, so a `HashInto` change to either path is caught.
fn compute_stream_fingerprint(state: &GameState) -> String {
    let mut hasher = blake3::Hasher::new();
    hasher.update(b"mtg-engine hash schema stream v1\n");
    hasher.update(&state.public_state_hash());
    for pid in 1..=4u64 {
        hasher.update(&state.private_state_hash(PlayerId(pid)));
    }
    hasher.finalize().to_hex().to_string()
}

/// The `HASH_SCHEMA_HISTORY` row pinning the current `HASH_SCHEMA_VERSION`.
fn current_epoch() -> HashSchemaEpoch {
    *HASH_SCHEMA_HISTORY
        .iter()
        .find(|e| e.version == HASH_SCHEMA_VERSION)
        .unwrap_or_else(|| {
            panic!(
                "HASH_SCHEMA_HISTORY has no row for the current HASH_SCHEMA_VERSION ({HASH_SCHEMA_VERSION}). \
                 Append a row when you bump the version."
            )
        })
}

/// Digest over the frozen prefix — every row except the current (tail) one.
fn compute_frozen_prefix_digest() -> String {
    let mut hasher = blake3::Hasher::new();
    hasher.update(b"mtg-engine hash schema frozen-prefix v1\n");
    let n = HASH_SCHEMA_HISTORY.len();
    for e in &HASH_SCHEMA_HISTORY[..n.saturating_sub(1)] {
        hasher.update(&[e.version]);
        hasher.update(e.decl_fingerprint.as_bytes());
        hasher.update(e.stream_fingerprint.as_bytes());
    }
    hasher.finalize().to_hex().to_string()
}

// ── Non-vacuity guards (written first: they find the scanner's own bugs) ──────

/// The scanner found a real codebase. Without this, a broken parser digests the
/// empty set and every other test here passes forever (SR-5's hard lesson).
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
            "scan root {scan_root} contributed no type declarations — per-root denominator guard"
        );
    }
}

/// The index keeps the **first** declaration per bare name; sound only while names
/// are unique across the scan roots.
#[test]
fn declared_type_names_are_unique() {
    let ScanResult { collisions, .. } = index_declarations();
    assert!(
        collisions.is_empty(),
        "these type names are declared more than once under the scan roots: {collisions:?}\n\
         `index_declarations` keeps only the first, so the fingerprint may be hashing the wrong \
         declaration. Disambiguate the names, or key the index by module path."
    );
}

/// An `EXTERNAL_TYPES` entry suppresses that bare name everywhere in the walk. If
/// the workspace ever declares a type with the same name, its shape silently drops
/// out of the digest.
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

/// Non-vacuity: the closure walk actually walked, contains what it must, and does
/// not contain the protocol frames or the skip-hidden card registry.
#[test]
fn state_closure_is_not_vacuous_and_bounded() {
    let ScanResult { index, .. } = index_declarations();
    let (closure, _) = state_closure(&index);

    assert!(
        closure.len() >= MIN_CLOSURE_TYPES,
        "GameState serde closure is only {} types; the type-position walker is broken (expected >= {})",
        closure.len(),
        MIN_CLOSURE_TYPES
    );
    for required in CLOSURE_MUST_CONTAIN {
        assert!(
            closure.contains(required),
            "{required} is reachable from GameState but missing from the computed closure — the \
             walker lost an edge and the fingerprint is now blind to {required}"
        );
    }
    for forbidden in CLOSURE_MUST_NOT_CONTAIN {
        assert!(
            !closure.contains(forbidden),
            "{forbidden} entered the GameState serde closure. If it is a protocol frame \
             (Command/ReplayLog/Envelope), whole-frame overlap merges the state-version and \
             protocol-version concerns — decide it on purpose (mirror of SR-8's \
             CLOSURE_MUST_NOT_CONTAIN). If it is CardRegistry/CardDefinition, the `#[serde(skip)]` \
             on `GameState.card_registry` stopped being honoured and the whole card DSL is now in \
             the hash schema."
        );
    }
    assert!(
        closure.contains("GameState") && closure.contains("Characteristics"),
        "closure must span both engine (GameState) and card-types (Characteristics)"
    );
}

/// Every type the closure reaches is either hashed or explicitly external. Guards
/// against silent under-inclusion.
#[test]
fn every_referenced_type_resolves() {
    let ScanResult { index, .. } = index_declarations();
    let (_, unresolved) = state_closure(&index);
    assert!(
        unresolved.is_empty(),
        "these types are reachable from GameState but are neither indexed nor listed in \
         EXTERNAL_TYPES, so their shape is NOT covered by the declaration fingerprint:\n{}\n\
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

/// The `#[serde(skip)]` on `GameState.card_registry` is load-bearing: it is what
/// keeps `CardRegistry`/`CardDefinition` out of the closure. Prove the skip is
/// doing the work (the field and attribute are present, and skip-blind traversal
/// *would* reach the card registry) rather than a broken walk that reaches nothing.
#[test]
fn serde_skip_is_load_bearing() {
    let ScanResult { index, .. } = index_declarations();
    let game_state = index.get("GameState").expect("GameState is indexed");

    // The skip attribute and the field it guards are present in the hashed text,
    // so a toggle of the skip moves the declaration digest.
    assert!(
        game_state.hash_text.contains("#[serde(skip)]")
            && game_state.hash_text.contains("card_registry"),
        "GameState no longer shows `#[serde(skip)] card_registry` in its hashed text; either the \
         field moved or preceding-attribute/body capture broke"
    );
    // Skip-awareness removed the type from the traversal view.
    assert!(
        !game_state.traversal_body.contains("CardRegistry"),
        "skip-aware traversal failed: `CardRegistry` is still a type position in GameState's \
         traversal body, so the card DSL will be pulled into the hash schema"
    );
    // And the removal is non-trivial: a skip-blind traversal reaches the registry,
    // proving the field really does point off-closure.
    let body_start = game_state
        .hash_text
        .find("card_registry")
        .expect("card_registry field present");
    assert!(
        game_state.hash_text[body_start..].contains("CardRegistry"),
        "expected `card_registry: Arc<CardRegistry>` in the hashed text — if the field type \
         changed, update this guard"
    );
}

/// The digest must not be the hash of nothing.
#[test]
fn decl_fingerprint_of_empty_closure_is_not_pinned() {
    let empty_index = BTreeMap::new();
    let empty_closure = BTreeSet::new();
    let empty = compute_decl_fingerprint(&empty_index, &empty_closure);
    assert_ne!(
        empty,
        current_epoch().decl_fingerprint,
        "decl_fingerprint is the digest of an EMPTY closure — the scanner returned nothing and the \
         pin was updated to match it. Assert the denominator (SR-5)."
    );
}

/// Attributes are part of the wire, so they must be part of the declaration
/// digest; `#[allow]` noise must not be.
#[test]
fn serde_attributes_are_inside_the_digest() {
    let ScanResult { index, .. } = index_declarations();
    let game_state = index.get("GameState").expect("GameState is indexed");
    assert!(
        game_state.hash_text.contains("#[serde(default)]"),
        "GameState's hashed text lost its field-level serde attributes; a #[serde(rename)] or \
         #[serde(skip)] would then be invisible to the gate"
    );
    assert!(
        game_state.hash_text.contains("Serialize") && game_state.hash_text.contains("Deserialize"),
        "GameState's hashed text lost its container #[derive(...)]"
    );
    assert!(
        !game_state.hash_text.contains("#[allow"),
        "#[allow(...)] leaked into the digest; it cannot affect the wire and would cause spurious \
         version bumps"
    );
}

/// Every non-alias type in the closure must show a `Serialize` derive in its
/// hashed text — the denominator guard on `preceding_attributes` (a dropped
/// multi-line derive would silently take a container's serde config out of the
/// digest).
#[test]
fn every_closure_type_shows_its_serialize_derive() {
    let ScanResult { index, .. } = index_declarations();
    let (closure, _) = state_closure(&index);

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
        "these state types have no `Serialize` in their hashed text: {missing:?}\n\
         Most likely `preceding_attributes` lost a multi-line #[derive(...)], so the container's \
         serde attributes are NOT in the fingerprint."
    );
}

/// The traversal view must not see attributes, or `#[serde(with = \"Foo\")]`-style
/// paths would inject phantom types into the closure.
#[test]
fn traversal_body_excludes_attributes() {
    let ScanResult { index, .. } = index_declarations();
    let game_state = index.get("GameState").expect("GameState is indexed");
    assert!(
        !game_state.traversal_body.contains("serde"),
        "attributes survived into the traversal body; type-position extraction will pick up \
         attribute arguments as if they were field types"
    );
}

/// The fixture actually hashes non-trivial state — the stream digest is not the
/// hash of an empty game.
#[test]
fn stream_is_sensitive() {
    let populated = canonical_fixture();
    let empty = GameStateBuilder::four_player()
        .build()
        .expect("empty fixture builds");
    assert_ne!(
        compute_stream_fingerprint(&populated),
        compute_stream_fingerprint(&empty),
        "the canonical fixture hashes identically to an empty four-player game — it is not \
         exercising the HashInto impls it claims to, so the stream digest is vacuous"
    );
    // Determinism: two builds of the same fixture must agree, or the pin is unstable.
    assert_eq!(
        compute_stream_fingerprint(&canonical_fixture()),
        compute_stream_fingerprint(&canonical_fixture()),
        "the canonical fixture is nondeterministic; the stream digest cannot be pinned"
    );
}

// ── The gates ────────────────────────────────────────────────────────────────

/// **AC 4520.** The serialized shape of the `GameState` closure is pinned.
/// Changing it without bumping `HASH_SCHEMA_VERSION` (and appending a row) fails
/// here.
#[test]
fn declaration_fingerprint_is_pinned() {
    let ScanResult { index, .. } = index_declarations();
    let (closure, _) = state_closure(&index);
    let actual = compute_decl_fingerprint(&index, &closure);

    assert_eq!(
        actual,
        current_epoch().decl_fingerprint,
        "\n\nThe serialized shape of the GameState type closure ({} types) has changed.\n\
         Old rewind snapshots and old replay logs cannot be read by this build.\n\n\
         Do ALL of these, in the same commit:\n  \
           1. bump HASH_SCHEMA_VERSION in crates/engine/src/state/hash.rs, adding a `- N:` \
              History line saying what moved;\n  \
           2. APPEND a new HASH_SCHEMA_HISTORY row; set its decl_fingerprint to:\n       {actual}\n  \
           3. update the HASH_SCHEMA_VERSION sentinels the suite carries.\n\n\
         If the shape change is genuinely wire-compatible (only a variant reorder is), it still \
         requires a bump here — the digest hashes declaration text in order.\n",
        closure.len()
    );
}

/// **AC 4521.** The hash byte-stream over the canonical fixture is pinned. A
/// `HashInto` edit that reorders, adds, or drops a feed — invisible to the
/// declaration digest — fails here.
#[test]
fn stream_fingerprint_is_pinned() {
    let actual = compute_stream_fingerprint(&canonical_fixture());
    assert_eq!(
        actual,
        current_epoch().stream_fingerprint,
        "\n\nThe GameState hash byte-stream has changed (a HashInto impl feeds different bytes, \
         or the canonical fixture's shape moved). Two independently-hashed states now hash \
         differently than before, so hashes recorded by older builds are incomparable.\n\n\
         Do ALL of these, in the same commit:\n  \
           1. bump HASH_SCHEMA_VERSION and add a `- N:` History line;\n  \
           2. APPEND a HASH_SCHEMA_HISTORY row; set its stream_fingerprint to:\n       {actual}\n  \
           3. update the HASH_SCHEMA_VERSION sentinels.\n\n\
         If you only meant to enrich the fixture (not change the schema), that still moves this \
         digest — a fixture change and a schema change are indistinguishable here, so bump.\n"
    );
}

/// **AC 4522.** `HASH_SCHEMA_HISTORY` is append-only and current.
///
/// - non-empty, versions strictly ascending and unique, tail == current version;
/// - every fingerprint is 64 lowercase hex;
/// - the baseline row (version 39) equals the FROZEN constants above, so a re-pin
///   of that shipped row in `hash.rs` *without* a bump disagrees here and fails —
///   the guarantee the plain sentinels could not make.
#[test]
fn history_is_append_only() {
    assert!(
        !HASH_SCHEMA_HISTORY.is_empty(),
        "HASH_SCHEMA_HISTORY is empty — there is nothing pinning HASH_SCHEMA_VERSION"
    );

    for w in HASH_SCHEMA_HISTORY.windows(2) {
        assert!(
            w[1].version > w[0].version,
            "HASH_SCHEMA_HISTORY is not strictly ascending / unique in version: {} then {}. \
             It is append-only — add new rows with higher versions, never reorder or duplicate.",
            w[0].version,
            w[1].version
        );
    }

    let last = HASH_SCHEMA_HISTORY.last().expect("non-empty");
    assert_eq!(
        last.version, HASH_SCHEMA_VERSION,
        "the last HASH_SCHEMA_HISTORY row is version {}, but HASH_SCHEMA_VERSION is {}. Append a \
         row for the current version (do not edit an existing one).",
        last.version, HASH_SCHEMA_VERSION
    );

    let is_hex64 = |s: &str| {
        s.len() == 64
            && s.bytes()
                .all(|b| b.is_ascii_hexdigit() && !b.is_ascii_uppercase())
    };
    for e in HASH_SCHEMA_HISTORY {
        assert!(
            is_hex64(e.decl_fingerprint) && is_hex64(e.stream_fingerprint),
            "version {} has a malformed fingerprint (expected 64 lowercase hex chars each)",
            e.version
        );
    }

    let baseline = HASH_SCHEMA_HISTORY
        .iter()
        .find(|e| e.version == BASELINE_VERSION)
        .expect("baseline version 39 row is present");
    assert_eq!(
        (baseline.decl_fingerprint, baseline.stream_fingerprint),
        (BASELINE_DECL_FINGERPRINT, BASELINE_STREAM_FINGERPRINT),
        "\n\nThe shipped version-{BASELINE_VERSION} row in HASH_SCHEMA_HISTORY no longer matches the \
         FROZEN baseline constants in tests/core/hash_schema.rs.\n\
         This is what a 're-pin without a bump' looks like: someone changed the schema, then \
         edited the version-{BASELINE_VERSION} fingerprints in place instead of bumping the version \
         and appending a row. Rewriting a shipped row's identity is forbidden — bump \
         HASH_SCHEMA_VERSION and append.\n"
    );
}

/// **AC 4522 (append-only, generalized).** Every shipped-and-superseded row — the
/// whole history except the current tail — is frozen by a single digest. This
/// carries `baseline_row_is_frozen`'s "you may not rewrite a shipped row" forward
/// to every version, not just 39: after a bump, the newly-superseded row joins the
/// prefix and locks here. Editing any past row in place moves this digest and
/// fails; a clean append leaves the pre-existing prefix bytes untouched (you re-pin
/// only because the newly-superseded row was added).
#[test]
fn frozen_prefix_is_pinned() {
    assert_eq!(
        compute_frozen_prefix_digest(),
        FROZEN_HISTORY_PREFIX_DIGEST,
        "\n\nThe frozen prefix of HASH_SCHEMA_HISTORY (every row except the current tail) changed.\n\
         Either a shipped, superseded row was edited in place — which is forbidden, the history is \
         append-only — or you just bumped the version and a row correctly joined the prefix. If the \
         latter, re-pin FROZEN_HISTORY_PREFIX_DIGEST in tests/core/hash_schema.rs to:\n       {}\n",
        compute_frozen_prefix_digest()
    );
}

/// Sentinel, mirroring the existing `HASH_SCHEMA_VERSION` sentinels: a bump must
/// be deliberate and seen in review, so it costs one more edit here.
#[test]
fn hash_schema_version_sentinel() {
    assert_eq!(
        HASH_SCHEMA_VERSION, 57,
        "HASH_SCHEMA_VERSION changed. Update this sentinel, append a HASH_SCHEMA_HISTORY row with \
         the new fingerprints, and add a `- N:` History line in state/hash.rs."
    );
}

// ════════════════════════════════════════════════════════════════════════════
// SR-19: HashInto-vs-struct field-coverage gate
// ════════════════════════════════════════════════════════════════════════════
//
// SR-17 (above) pins *that the hash schema hasn't drifted*; it does not check
// *that a struct's `HashInto` impl covers every field the struct declares*. Those
// are different holes. SR-7 gotcha 5 named this one precisely: "When you remove
// fields from a `HashInto` impl, diff the impl against the struct rather than
// assuming they agree; nothing enforces that they do" — the two `haunt_*` fields
// were fed to the hasher by nobody for a long time, harmless only because they
// were always `None`. A field that *is* live (like `PendingTrigger.embedded_effect`
// or `StackObject.cast_from_top_with_bonus`, both closed by SR-19) creates a
// desync-detection blind spot: two genuinely-different game states hash identically
// because the distinguishing field never reaches the hasher.
//
// This gate parses each `pub struct` under the scan roots that has an
// `impl HashInto for <T>` in `state/hash.rs`, and asserts every declared field is
// either read as `self.<field>` in that impl body OR listed in the per-type
// `NOT_HASHED` allowlist. A dead-entry guard keeps the allowlist honest: every
// `NOT_HASHED` entry must name a real declared field that is *genuinely absent*
// from the body, so an entry cannot be used to wave a field through that is in fact
// hashed (or does not exist). Per the SR track rule ("assert the denominator"),
// every derived set has a non-vacuity floor.
//
// Scope: this gate covers the per-type `HashInto` *struct* impls (which uniformly
// read `self.<field>` — verified: none destructure). It does NOT cover:
//   - enum `HashInto` impls (they match on variants, not fields) — those remain
//     covered by the SR-17 declaration digest (a new variant moves `decl_fingerprint`);
//   - `GameState`'s own `public_state_hash` / `private_state_hash`, which select
//     fields deliberately (public omits hidden zones) rather than hashing all of
//     them — covered by the SR-17 decl + stream digests, not by a "every field is
//     hashed" rule (which would be wrong for it).

/// `(type, field)` pairs deliberately NOT fed to that type's `HashInto`.
///
/// Each entry must name a real declared field of a named-field struct that has a
/// `HashInto` impl, and that field must be genuinely absent from the impl body
/// (`not_hashed_allowlist_has_no_dead_entries` enforces both — a stale or bogus
/// entry fails the build).
///
/// **Empty today**: every field of every hashed struct is fed to `HashInto`. SR-19
/// closed the last two gaps (`PendingTrigger.embedded_effect`,
/// `StackObject.cast_from_top_with_bonus`) by hashing them rather than allowlisting
/// them — both affect resolution, so excluding them would be a real blind spot, not
/// a benign omission. The mechanism exists for a *future* field with a sound reason
/// to be excluded (pure runtime scratch, or a value fully derived from other hashed
/// fields); such a field lands here with a one-line rationale instead of silently
/// dropping out of the hash.
const NOT_HASHED: &[(&str, &str)] = &[];

/// Hashed structs that MUST be covered by the intersection, or a scanner lost them.
/// (The two largest impls plus the SR-19 fix sites.)
const COVERAGE_MUST_INCLUDE: [&str; 6] = [
    "PendingTrigger",
    "StackObject",
    "GameObject",
    "PlayerState",
    "Characteristics",
    "TurnState",
];

/// Non-vacuity floors. Deliberately well below the real counts.
const MIN_HASHINTO_IMPLS: usize = 80;
const MIN_NAMED_STRUCTS: usize = 60;
const MIN_COVERED_STRUCTS: usize = 30;
const MIN_FIELDS_CHECKED: usize = 200;

fn hash_rs_path() -> PathBuf {
    workspace_root().join("crates/engine/src/state/hash.rs")
}

/// Bodies of every `impl HashInto for <T> { ... }` in `state/hash.rs`, keyed by the
/// exact type token as written (bare names for struct impls; Rust forbids duplicate
/// inherent/trait impls of the same type, so bare-name keys are collision-free, and
/// a path-qualified enum impl like `crate::…::GiftType` is simply a different key
/// that no struct-name lookup hits). Generic blanket impls (`impl<T> HashInto for …`)
/// are skipped: the needle is the non-generic `impl HashInto for ` form.
fn hashinto_impl_bodies() -> BTreeMap<String, String> {
    let raw = std::fs::read_to_string(hash_rs_path()).expect("readable hash.rs");
    let src = strip_comments(&raw);
    let b = src.as_bytes();
    let needle = "impl HashInto for ";
    let mut out: BTreeMap<String, String> = BTreeMap::new();
    let mut from = 0;
    while let Some(rel) = src[from..].find(needle) {
        let at = from + rel;
        from = at + needle.len();
        let after = at + needle.len();
        let ty: String = src[after..]
            .chars()
            .take_while(|c| c.is_ascii_alphanumeric() || *c == '_' || *c == ':')
            .collect();
        if ty.is_empty() {
            continue;
        }
        // Walk to the impl's opening brace, tolerating a generic-arg suffix on the
        // type (`Option<T>` etc.) but not a `for`-loop or string before it. In
        // practice every struct impl here is `impl HashInto for Name {`.
        let mut j = after + ty.len();
        while j < b.len() && b[j] != b'{' && b[j] != b';' {
            j += 1;
        }
        if j >= b.len() || b[j] != b'{' {
            continue;
        }
        let end = match_delim(b, j, b'{', b'}');
        // First writer wins; bare-name impls are unique so this never discards a
        // struct impl.
        out.entry(ty).or_insert_with(|| src[j..end].to_string());
        from = end;
    }
    out
}

/// Every named-field `pub struct` under the scan roots → its declared field names,
/// in declaration order. Tuple structs, unit structs, and enums are excluded (they
/// have no named fields for a "field is hashed" rule to apply to).
fn named_field_structs() -> BTreeMap<String, Vec<String>> {
    let root = workspace_root();
    let mut out: BTreeMap<String, Vec<String>> = BTreeMap::new();
    for scan_root in SCAN_ROOTS {
        let mut files = Vec::new();
        walk(&root.join(scan_root), &mut files);
        files.sort();
        for file in files {
            let raw = std::fs::read_to_string(&file).expect("readable source");
            let src = strip_comments(&raw);
            let b = src.as_bytes();
            let kw = "pub struct ";
            let mut from = 0;
            while let Some(rel) = src[from..].find(kw) {
                let at = from + rel;
                from = at + kw.len();
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
                // Find the body delimiter. `{` = named struct (what we want); `(` =
                // tuple struct; `;` = unit struct — both skipped. A generic
                // parameter list `<…>` or `where` clause contains no `{`/`(`/`;`, so
                // we scan through it to the real body brace.
                let mut j = after + name.len();
                while j < b.len() && b[j] != b'{' && b[j] != b'(' && b[j] != b';' {
                    j += 1;
                }
                if j >= b.len() || b[j] != b'{' {
                    continue;
                }
                let end = match_delim(b, j, b'{', b'}');
                let body = strip_attributes(&src[j + 1..end - 1]);
                out.entry(name).or_insert_with(|| struct_field_names(&body));
                from = end;
            }
        }
    }
    out
}

/// Field names of a struct body (attributes + comments already stripped): the
/// identifier immediately before each top-level `:`.
fn struct_field_names(body: &str) -> Vec<String> {
    let b = body.as_bytes();
    let n = b.len();
    // Split into field segments at depth-0 commas.
    let mut segs: Vec<&str> = Vec::new();
    let mut depth = 0i32;
    let mut seg_start = 0usize;
    let mut i = 0;
    while i < n {
        if let Some(len) = literal_len(b, i) {
            i += len;
            continue;
        }
        match b[i] {
            b'<' | b'(' | b'[' | b'{' => depth += 1,
            b'>' | b')' | b']' | b'}' => depth -= 1,
            b',' if depth == 0 => {
                segs.push(&body[seg_start..i]);
                seg_start = i + 1;
            }
            _ => {}
        }
        i += 1;
    }
    segs.push(&body[seg_start..]);

    let mut fields = Vec::new();
    for seg in segs {
        let sb = seg.as_bytes();
        let m = sb.len();
        let mut d = 0i32;
        let mut k = 0usize;
        let mut colon = None;
        while k < m {
            if let Some(len) = literal_len(sb, k) {
                k += len;
                continue;
            }
            match sb[k] {
                b'<' | b'(' | b'[' | b'{' => d += 1,
                b'>' | b')' | b']' | b'}' => d -= 1,
                b':' if d == 0 && sb.get(k + 1) != Some(&b':') && (k == 0 || sb[k - 1] != b':') => {
                    colon = Some(k);
                    break;
                }
                _ => {}
            }
            k += 1;
        }
        let Some(c) = colon else { continue };
        let mut e = c;
        while e > 0 && sb[e - 1].is_ascii_whitespace() {
            e -= 1;
        }
        let mut s = e;
        while s > 0 && (sb[s - 1].is_ascii_alphanumeric() || sb[s - 1] == b'_') {
            s -= 1;
        }
        let name = &seg[s..e];
        if !name.is_empty() && name != "pub" {
            fields.push(name.to_string());
        }
    }
    fields
}

/// True iff `body` reads `self.<field>` as a whole token (so field `source` does
/// not match `self.source_object`, and `myself.x` does not match field `x`).
fn body_references_field(body: &str, field: &str) -> bool {
    let needle = format!("self.{field}");
    let b = body.as_bytes();
    let mut from = 0;
    while let Some(rel) = body[from..].find(&needle) {
        let at = from + rel;
        let after = at + needle.len();
        let ok_after = b
            .get(after)
            .is_none_or(|c| !(c.is_ascii_alphanumeric() || *c == b'_'));
        let ok_before = at == 0 || !(b[at - 1].is_ascii_alphanumeric() || b[at - 1] == b'_');
        if ok_after && ok_before {
            return true;
        }
        from = at + 1;
    }
    false
}

/// Non-vacuity: both scanners found a real codebase, the well-known impls resolve,
/// and the field-reference matcher has working positive and negative controls.
/// Without this, a broken parser makes the whole gate pass over the empty set.
#[test]
fn coverage_scanners_are_not_vacuous() {
    let bodies = hashinto_impl_bodies();
    let structs = named_field_structs();

    assert!(
        bodies.len() >= MIN_HASHINTO_IMPLS,
        "found only {} `impl HashInto for` blocks in hash.rs; the impl scanner is broken (expected >= {})",
        bodies.len(),
        MIN_HASHINTO_IMPLS
    );
    assert!(
        structs.len() >= MIN_NAMED_STRUCTS,
        "found only {} named-field pub structs under the scan roots; the struct scanner is broken (expected >= {})",
        structs.len(),
        MIN_NAMED_STRUCTS
    );

    for req in COVERAGE_MUST_INCLUDE {
        let fields = structs.get(req).unwrap_or_else(|| {
            panic!("`{req}` not found by named_field_structs — the struct scanner lost a well-known type")
        });
        assert!(
            !fields.is_empty(),
            "`{req}` was parsed with zero fields — struct_field_names is broken"
        );
        assert!(
            bodies.contains_key(req),
            "`{req}` has no `impl HashInto for {req}` body — the impl scanner keyed it wrong \
             (a path-qualified struct impl would slip past the bare-name lookup)"
        );
    }

    // Field-reference matcher controls.
    let pt = bodies.get("PendingTrigger").expect("PendingTrigger impl");
    assert!(
        body_references_field(pt, "source"),
        "positive control failed: PendingTrigger hashes self.source"
    );
    assert!(
        !body_references_field(pt, "no_such_field_zzz"),
        "negative control failed: matched a non-existent field"
    );
    // Token-boundary control: `source` must not match `self.source_object`.
    assert!(
        !body_references_field("self.source_object.hash_into(hasher);", "source"),
        "token-boundary control failed: field `source` matched `self.source_object`"
    );

    // The two SR-19 fixes are actually in place.
    assert!(
        body_references_field(pt, "embedded_effect"),
        "SR-19: PendingTrigger must now hash embedded_effect"
    );
    assert!(
        body_references_field(
            bodies.get("StackObject").expect("StackObject impl"),
            "cast_from_top_with_bonus"
        ),
        "SR-19: StackObject must now hash cast_from_top_with_bonus"
    );
}

/// **AC 4526.** Every field of every hashed struct is fed to that struct's
/// `HashInto`, or is on the `NOT_HASHED` allowlist. A field silently dropped from a
/// `HashInto` impl (the SR-7 haunt-field failure mode) fails here.
#[test]
fn every_hashed_struct_field_is_hashed_or_allowlisted() {
    let bodies = hashinto_impl_bodies();
    let structs = named_field_structs();
    let allow: BTreeSet<(&str, &str)> = NOT_HASHED.iter().copied().collect();

    let mut covered = 0usize;
    let mut fields_checked = 0usize;
    let mut violations: Vec<String> = Vec::new();

    for (ty, fields) in &structs {
        let Some(body) = bodies.get(ty) else {
            continue; // struct without a HashInto impl — out of this gate's scope
        };
        covered += 1;
        for f in fields {
            fields_checked += 1;
            if body_references_field(body, f) {
                continue;
            }
            if allow.contains(&(ty.as_str(), f.as_str())) {
                continue;
            }
            violations.push(format!("{ty}.{f}"));
        }
    }

    assert!(
        violations.is_empty(),
        "\n\nThese struct fields are declared but never fed to their type's `HashInto` impl, and \
         are not on the NOT_HASHED allowlist:\n  {}\n\n\
         Two independent game states differing only in such a field hash IDENTICALLY, so \
         distributed-verification / replay divergence-detection is blind to it (SR-7 gotcha 5). \
         For each: either add `self.<field>.hash_into(hasher)` to the impl (and bump \
         HASH_SCHEMA_VERSION per the state/hash.rs checklist — it changes the byte stream), or, if \
         the field is genuinely not game state (pure runtime scratch, or fully derived from other \
         hashed fields), add it to NOT_HASHED with a one-line rationale.\n",
        violations.join("\n  ")
    );

    // Denominators, so a scanner that returned nothing cannot pass this vacuously.
    assert!(
        covered >= MIN_COVERED_STRUCTS,
        "only {covered} hashed structs were checked (expected >= {MIN_COVERED_STRUCTS}); the \
         struct/impl intersection is empty or nearly so — a scanner broke"
    );
    assert!(
        fields_checked >= MIN_FIELDS_CHECKED,
        "only {fields_checked} fields were checked across all hashed structs (expected >= \
         {MIN_FIELDS_CHECKED}); struct_field_names is under-counting"
    );
}

/// The `NOT_HASHED` allowlist is honest: every entry names a real declared field of
/// a hashed struct that is genuinely absent from the impl body. A dead entry (wrong
/// type, wrong field, or a field that is actually hashed) fails here, so the
/// allowlist can never be used to wave through a field that is in fact covered — or
/// to accrue stale entries after a field is deleted or later hashed.
#[test]
fn not_hashed_allowlist_has_no_dead_entries() {
    let bodies = hashinto_impl_bodies();
    let structs = named_field_structs();

    for (ty, field) in NOT_HASHED {
        let fields = structs.get(*ty).unwrap_or_else(|| {
            panic!("NOT_HASHED entry ({ty}, {field}): `{ty}` is not a named-field struct under the scan roots")
        });
        assert!(
            fields.iter().any(|f| f == field),
            "NOT_HASHED entry ({ty}, {field}): `{ty}` declares no field named `{field}` (dead entry — \
             remove it or fix the name)"
        );
        let body = bodies.get(*ty).unwrap_or_else(|| {
            panic!("NOT_HASHED entry ({ty}, {field}): `{ty}` has no `impl HashInto for {ty}`")
        });
        assert!(
            !body_references_field(body, field),
            "NOT_HASHED entry ({ty}, {field}): `{ty}`'s HashInto DOES hash `{field}` — remove it \
             from the allowlist (dead entry). The allowlist is for fields that are NOT hashed."
        );
    }
}
