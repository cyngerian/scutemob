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

use imbl::OrdSet;
use mtg_engine::cards::card_definition::{DelayedReturnDestination, ManaRestriction};
use mtg_engine::state::hash::HashInto;
use mtg_engine::state::stubs::DelayedTriggerTiming;
use mtg_engine::{
    AnsweredEffectChoice, CardEffectTarget, CardType, Color, ContinuousEffect, CounterType, Effect,
    EffectAmount, EffectChoiceAnswer, EffectChoiceQuestion, EffectDuration, EffectFilter, EffectId,
    EffectLayer, GameState, GameStateBuilder, HashSchemaEpoch, KeywordAbility, LayerModification,
    ManaColor, ManaPool, ObjectId, ObjectSpec, PendingEffectChoice, PlayerId, PlayerTarget,
    ProtectionQuality, Step, SubType, SuperType, TargetFilter, ZoneId, HASH_SCHEMA_HISTORY,
    HASH_SCHEMA_VERSION,
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
// PB-OS11 (2026-07-19): re-pinned on the 62→63 bump — version 62 became a
// superseded row and joined the frozen prefix. Its bytes (the v62
// fingerprints) are unchanged; the digest moved only because the prefix
// gained a member.
// PB-DP5 (2026-07-26): re-pinned on the 63→64 bump — version 63 became a
// superseded row and joined the frozen prefix.
// PB-DP7 (2026-07-26): re-pinned on the 64→65 bump — version 64 became a
// superseded row and joined the frozen prefix.
// PB-DP8 fix cycle (2026-07-26): re-pinned on the 66→67 bump — versions 65 and 66
// became superseded rows and joined the frozen prefix.
// PB-DX5 (2026-08-01): re-pinned on the 69→70 bump — version 69 became a
// superseded row and joined the frozen prefix.
// ENG-1 (2026-08-02): re-pinned on the 70→71 bump — version 70 became a
// superseded row and joined the frozen prefix.
// ENG-2 (2026-08-02): re-pinned on the 71→72 bump — version 71 became a
// superseded row and joined the frozen prefix.
// PB-DX21 (2026-08-04): re-pinned on the 72→73 bump — version 72 became a
// superseded row and joined the frozen prefix.
// PB-DX27 rider (2026-08-13): re-pinned on the 74→75 bump — version 74 became
// a superseded row and joined the frozen prefix.
// PB-DX44 stage 2a (2026-08-15): re-pinned on the 76→77 bump — version 76
// became a superseded row and joined the frozen prefix.
// PB-DX50 (2026-09-03): re-pinned on the 78→79 bump — version 78 became a
// superseded row and joined the frozen prefix. (PB-DX45 did the same on 77→78.)
// PB-DX20b (2026-09-03): re-pinned on the 79→80 bump — version 79 became a
// superseded row and joined the frozen prefix. Appended in the `/review` fix
// cycle: the digest below moved in `0be8d904` while this log did not, so for
// one commit the current value was attributed to PB-DX50's 78→79 re-pin. That
// is what this log exists to prevent, and every prior re-pinning batch had
// extended it.
const FROZEN_HISTORY_PREFIX_DIGEST: &str =
    "185572da77abae71a1a21204a4cbb8231a65c256858851e3a101738172bca5d0";

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
        //
        // PB-DX5 / CR 611.2c: `affected_set` is populated `Some(..)` here
        // ON PURPOSE, not left at the `None` every other pre-existing
        // `ContinuousEffect` literal in the repo backfilled with -- a `None`
        // fixture would still move both digests (a new hashed field always
        // does) but would leave the new `HashInto` feed itself unexercised by
        // the canonical fixture, the exact gap several prior
        // `HASH_SCHEMA_HISTORY` rows had to admit. `ObjectId(1)` is an
        // arbitrary id; this fixture never runs `process_command`, so nothing
        // depends on it referring to a real object.
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
            affected_set: Some(OrdSet::unit(ObjectId(1))),
            condition: None,
        })
        // ENG-1 fix cycle, review Finding 1: a `Discard`-shaped
        // `pending_effect_choice` + one `effect_choice_answers` entry, so both
        // new `HashInto` arms (`EffectChoiceQuestion::Discard`,
        // `EffectChoiceAnswer::Discard`) are genuinely fed into
        // `stream_fingerprint` rather than left to the version-sentinel byte
        // alone. `ObjectId(101..103)` are synthetic, same convention as the
        // `ObjectId(1)` above on `affected_set` -- this fixture never runs
        // `process_command`, so nothing depends on them naming real objects.
        // `count: 2` is deliberately neither 0 nor `hand.len()` (3), so the
        // question's own short-circuit-adjacent fields both carry real,
        // distinguishable bytes.
        .pending_effect_choice(PendingEffectChoice {
            choice_id: 1,
            player: PlayerId(2),
            source: ObjectId(2),
            question: EffectChoiceQuestion::Discard {
                hand: vec![ObjectId(101), ObjectId(102), ObjectId(103)],
                count: 2,
            },
            index: 1,
        })
        .effect_choice_answer(AnsweredEffectChoice {
            question: EffectChoiceQuestion::Discard {
                hand: vec![ObjectId(201), ObjectId(202)],
                count: 1,
            },
            answer: EffectChoiceAnswer::Discard {
                chosen: vec![ObjectId(202)],
            },
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
        HASH_SCHEMA_VERSION, 83,
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
//   - `GameState`'s own `public_state_hash` / `private_state_hash`, which select
//     fields deliberately (public omits hidden zones) rather than hashing all of
//     them — covered by the SR-17 decl + stream digests, not by a "every field is
//     hashed" rule (which would be wrong for it).
//
// **PB-DX7 (2026-08-11) correction**: this doc used to also list enum `HashInto`
// impls as out of scope ("they match on variants, not fields ... remain covered
// by the SR-17 declaration digest"). That was true only in the narrow sense that
// a NEW or REMOVED variant moves `decl_fingerprint` — it said nothing about a
// FIELD silently dropped from an EXISTING variant's arm, which is exactly
// **OOS-DP9-13**: `EffectChoiceQuestion::SearchLibrary { candidates,
// may_fail_to_find }` rewritten as `{ candidates, .. }` moved no digest and
// stayed clippy-clean. The struct half above is joined below by
// `every_hashed_enum_variant_field_is_hashed_or_allowlisted`, which parses every
// hashed enum's declared variants against its `match self` arms the same way —
// enum `HashInto` impls are no longer a documented gap in this gate's scope,
// they are the second half of it.
//
// Also corrected: OOS-DP7-11 additionally affected the STRUCT half above via the
// 5 path-qualified struct impls (`impl HashInto for crate::…::Foo`), which the
// original bare-name-keyed scanner silently skipped entirely — closed by
// `hashinto_impl_bodies()` keying on the bare name (last `::` segment)
// regardless of spelling; see `every_hashed_type_resolves_to_a_declaration` and
// `every_hashed_struct_is_parsed_by_named_field_structs` below.

/// `(type, field)` pairs deliberately NOT fed to that type's `HashInto` AT ALL —
/// `self.<field>` never appears in the impl body.
///
/// Each entry must name a real declared field of a named-field struct that has a
/// `HashInto` impl, and that field must be genuinely absent from the impl body
/// (`not_hashed_allowlist_has_no_dead_entries` enforces both — a stale or bogus
/// entry fails the build).
///
/// **Empty today**: every field of every hashed struct is at least referenced by
/// `HashInto`. SR-19 closed the `StackObject.cast_from_top_with_bonus` gap by
/// hashing it fully. `PendingTrigger.embedded_effect` is referenced (hashed as
/// `.is_some()`, not fully) — see `PARTIALLY_HASHED` below, PB-DX7's third
/// disposition category for that different shape of coverage. The mechanism
/// here exists for a *future* field with a sound reason to be excluded entirely
/// (pure runtime scratch, or a value fully derived from other hashed fields);
/// such a field lands here with a one-line rationale instead of silently
/// dropping out of the hash.
const NOT_HASHED: &[(&str, &str)] = &[];

/// `(type, field, reason)` triples for fields whose ONLY appearance in their
/// type's `HashInto` impl body is `self.<field>.<summariser>(..).hash_into(..)`
/// — a method call whose RETURN VALUE is hashed, discarding the field's actual
/// content (e.g. `.is_some()`). This is a **different** disposition from
/// `NOT_HASHED`: the field is not silently absent, it is silently reduced.
///
/// **PB-DX7 (2026-08-11), coordinator-directed**: the widened struct gate
/// (`every_hashed_struct_field_is_hashed_or_allowlisted`) treated ANY
/// occurrence of `self.<field>` as "covered", so `self.on_cast_effect
/// .is_some().hash_into(hasher)` passed clean — the gate reporting success
/// while checking a technicality of its own matcher, the same shape as the two
/// primitive holes this batch closes, one level down. Each entry here makes
/// that reduction visible and re-reviewable (it is not a claim the reduction
/// is wrong — both known entries are deliberate, documented decisions, quoted
/// below):
///
/// - `PendingTrigger.embedded_effect` (`hash.rs`, `impl HashInto for
///   PendingTrigger`): "presence suffices for divergence detection because the
///   effect is a copy of the source ability's, redundant with `source` +
///   `ability_index`" (SR-19, `HASH_SCHEMA_HISTORY` entry 40).
/// - `PlayFromTopPermission.on_cast_effect` (`hash.rs`, `impl HashInto for
///   crate::state::stubs::PlayFromTopPermission`): "intentionally not hashed —
///   it's a pure effect descriptor (same as how individual `ContinuousEffectDef`
///   fields work)".
///
/// A field genuinely NOT on this list is fully hashed by every OTHER
/// occurrence of `self.<field>` in the body — e.g. a `.len()` prefix followed
/// by a separate element-wise iteration over the same field counts as full
/// coverage (the field's real values are hashed via the loop; `struct_field_coverage`
/// below classifies per-occurrence and aggregates, not per-field-as-a-whole).
const PARTIALLY_HASHED: &[(&str, &str, &str)] = &[
    (
        "PendingTrigger",
        "embedded_effect",
        "presence suffices for divergence detection -- the effect is a copy of \
         the source ability's, redundant with source + ability_index (SR-19, \
         HASH_SCHEMA_HISTORY entry 40)",
    ),
    (
        "PlayFromTopPermission",
        "on_cast_effect",
        "intentionally not hashed -- it's a pure effect descriptor (same as how \
         individual ContinuousEffectDef fields work)",
    ),
];

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

/// `HashInto` impl targets that are std/primitive types, not a declaration under
/// `SCAN_ROOTS` — `every_hashed_type_resolves_to_a_declaration` treats resolving to
/// one of these as equivalent to resolving to a struct/enum declaration. Each entry
/// carries a one-line reason so the list itself explains why it is not a gap.
const HASHED_PRIMITIVE_TARGETS: [(&str, &str); 8] = [
    ("u8", "std integer type"),
    ("u32", "std integer type"),
    ("u64", "std integer type"),
    ("i32", "std integer type"),
    ("usize", "std integer type"),
    ("bool", "std bool type"),
    ("String", "std String type"),
    ("str", "std str type"),
];

fn hash_rs_path() -> PathBuf {
    workspace_root().join("crates/engine/src/state/hash.rs")
}

/// One parsed `impl HashInto for <T> { ... }` block.
struct ImplBody {
    /// The impl target exactly as spelled in `hash.rs` (e.g.
    /// `crate::state::stubs::FlashGrant`, or a bare `PendingTrigger`).
    spelling: String,
    /// The impl's body text (comments already stripped, outer braces excluded).
    body: String,
}

/// Bodies of every `impl HashInto for <T> { ... }` in `state/hash.rs`, keyed by
/// `<T>`'s **bare name** (its last `::` segment) regardless of how the impl target
/// happens to be spelled.
///
/// **OOS-DP7-11 / PB-DX7**: this used to key on the exact written token, so a
/// path-qualified spelling (`impl HashInto for crate::…::FlashGrant`) produced a
/// key (`crate::…::FlashGrant`) that no bare-name lookup (`bodies.get("FlashGrant")`)
/// could ever hit — the struct/enum coverage gates silently skipped every
/// path-qualified impl. Do NOT rename any call site in `hash.rs` to "fix" this —
/// the gate must not depend on how the impl is spelled; normalizing the KEY here is
/// what makes that true.
///
/// Rust forbids duplicate trait impls of the same concrete type, so two DIFFERENT
/// spellings can only collide on the SAME bare name if they name two DIFFERENT
/// types declared in different modules with the same identifier — which the gate
/// must not silently resolve one way; see the `panic!` below.
///
/// Generic blanket impls (`impl<T> HashInto for …`) are skipped: the needle is the
/// non-generic `impl HashInto for ` form.
fn hashinto_impl_bodies() -> BTreeMap<String, ImplBody> {
    let raw = std::fs::read_to_string(hash_rs_path()).expect("readable hash.rs");
    let src = strip_comments(&raw);
    let b = src.as_bytes();
    let needle = "impl HashInto for ";
    let mut out: BTreeMap<String, ImplBody> = BTreeMap::new();
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
        let bare = ty.rsplit("::").next().unwrap_or(&ty).to_string();
        if let Some(existing) = out.get(&bare) {
            panic!(
                "duplicate HashInto bare-name key `{bare}`: both `{}` and `{ty}` resolve to it. \
                 Rust forbids duplicate trait impls of the SAME type, so this can only happen if \
                 the bare-name scanner is folding two DIFFERENT types with the same identifier \
                 onto one key -- the gate must not silently pick one over the other. Disambiguate \
                 by scoping `hashinto_impl_bodies` to consider more of the path, not just the \
                 last `::` segment.",
                existing.spelling
            );
        }
        out.insert(
            bare,
            ImplBody {
                spelling: ty,
                body: src[j..end].to_string(),
            },
        );
        from = end;
    }
    out
}

/// True iff `decl` is a `pub struct` declaration (as opposed to `pub enum` or a
/// `pub type` alias). Reads the keyword straight out of `Decl.hash_text`, which
/// embeds it verbatim (`index_declarations` builds it as `"{attrs} {kw}{name}
/// {body}"`), rather than re-scanning source.
fn decl_is_struct(decl: &Decl) -> bool {
    !decl.is_alias && decl.hash_text.contains("pub struct ")
}

/// True iff `decl` is a `pub enum` declaration.
fn decl_is_enum(decl: &Decl) -> bool {
    !decl.is_alias && decl.hash_text.contains("pub enum ")
}

/// A `pub struct`'s declared body shape: `{ .. }` fields (in scope for
/// `named_field_structs()` / the field-coverage gate) vs `( .. )` / `;` (tuple or
/// unit — no named field for a "field is hashed" rule to apply to, e.g.
/// `ObjectId(u32)` or `SubType(String)`, both of which hash `self.0`).
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum StructShape {
    NamedFields,
    TupleOrUnit,
}

/// Every `pub struct`'s declared shape under the scan roots, independent of
/// `named_field_structs()`'s narrower named-field-only result. Used only to tell
/// a legitimately-out-of-scope tuple/unit struct apart from a named-field struct
/// that `named_field_structs()` failed to parse.
fn all_struct_shapes() -> BTreeMap<String, StructShape> {
    let root = workspace_root();
    let mut out: BTreeMap<String, StructShape> = BTreeMap::new();
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
                let mut j = after + name.len();
                while j < b.len() && b[j] != b'{' && b[j] != b'(' && b[j] != b';' {
                    j += 1;
                }
                if j >= b.len() {
                    continue;
                }
                let shape = if b[j] == b'{' {
                    StructShape::NamedFields
                } else {
                    StructShape::TupleOrUnit
                };
                out.entry(name).or_insert(shape);
                from = match b[j] {
                    b'{' => match_delim(b, j, b'{', b'}'),
                    b'(' => match_delim(b, j, b'(', b')'),
                    _ => j + 1,
                };
            }
        }
    }
    out
}

/// Core of [`declared_non_pub`], operating on an already-comment-stripped source
/// string so it has isolated positive/negative controls (see
/// `struct_and_enum_scope_scanners_are_not_vacuous`).
///
/// True iff `src` declares `name` as a `struct` or `enum` whose visibility is
/// NOT a bare `pub` — `pub(crate) struct Foo`, or no modifier at all. A
/// declaration that does not exist in `src` at all returns `false` (that case is
/// `every_hashed_type_resolves_to_a_declaration`'s to catch, not this function's).
fn src_declares_non_pub(src: &str, name: &str) -> bool {
    let b = src.as_bytes();
    for kw in ["struct ", "enum "] {
        let mut from = 0;
        while let Some(rel) = src[from..].find(kw) {
            let at = from + rel;
            from = at + kw.len();
            if at > 0 && (b[at - 1].is_ascii_alphanumeric() || b[at - 1] == b'_') {
                continue;
            }
            let after = at + kw.len();
            let matched: String = src[after..]
                .chars()
                .take_while(|c| c.is_ascii_alphanumeric() || *c == '_')
                .collect();
            if matched != name {
                continue;
            }
            let before = src[..at].trim_end();
            if before.ends_with("pub") {
                let idx = before.len() - "pub".len();
                let bb = before.as_bytes();
                let ok_before_pub =
                    idx == 0 || !(bb[idx - 1].is_ascii_alphanumeric() || bb[idx - 1] == b'_');
                if ok_before_pub {
                    continue; // exactly bare `pub` -- already covered elsewhere
                }
            }
            return true;
        }
    }
    false
}

/// True iff some declaration of `name` as a `struct`/`enum` exists under the scan
/// roots whose visibility is NOT a bare `pub`.
fn declared_non_pub(name: &str) -> bool {
    let root = workspace_root();
    for scan_root in SCAN_ROOTS {
        let mut files = Vec::new();
        walk(&root.join(scan_root), &mut files);
        for file in files {
            let raw = std::fs::read_to_string(&file).unwrap_or_default();
            let src = strip_comments(&raw);
            if src_declares_non_pub(&src, name) {
                return true;
            }
        }
    }
    false
}

/// **AC 6383 / OOS-DP7-11, part 1 — the fail-BY-NAME requirement.** Every
/// `HashInto` impl target must resolve to either a declared struct/enum under
/// `SCAN_ROOTS`, or an explicit `HASHED_PRIMITIVE_TARGETS` entry. Anything else
/// fails by name, so a future mis-keyed or mis-spelled impl target is loud
/// instead of silently invisible to every other gate in this file (which all key
/// off the same `hashinto_impl_bodies()` map).
#[test]
fn every_hashed_type_resolves_to_a_declaration() {
    let bodies = hashinto_impl_bodies();
    let decls = index_declarations();
    let allow: BTreeMap<&str, &str> = HASHED_PRIMITIVE_TARGETS.iter().copied().collect();

    let mut unresolved: Vec<String> = Vec::new();
    for (bare, impl_body) in &bodies {
        if decls.index.contains_key(bare) || allow.contains_key(bare.as_str()) {
            continue;
        }
        unresolved.push(format!(
            "{bare} (spelled `{}` in hash.rs)",
            impl_body.spelling
        ));
    }
    assert!(
        unresolved.is_empty(),
        "\n\nThese HashInto impl targets resolve to NEITHER a declared struct/enum under \
         SCAN_ROOTS NOR an entry in HASHED_PRIMITIVE_TARGETS -- the gate cannot classify them, \
         which is exactly how a mis-keyed or mis-spelled impl target goes silent:\n  {}\n",
        unresolved.join("\n  ")
    );

    assert!(
        bodies.len() >= MIN_HASHINTO_IMPLS,
        "found only {} `impl HashInto for` blocks in hash.rs; the impl scanner is broken \
         (expected >= {})",
        bodies.len(),
        MIN_HASHINTO_IMPLS
    );
}

/// **PB-DX7 review L5 fix (2026-08-12).** `hashinto_impl_bodies()` can silently
/// drop an impl two ways: the needle `"impl HashInto for "` misses a spelling
/// variant (a line break after `for`, a double space, `impl<'a> HashInto for
/// Foo<'a>`), or `ty.is_empty()`/the missing-`{`-body check silently
/// `continue`s past a malformed match. `MIN_HASHINTO_IMPLS = 80` against a
/// live 139 leaves 59 impls of headroom before that floor would even notice.
/// This asserts an EXACT ratchet instead: an independent raw count of the
/// literal needle (comment-stripped, no parsing) must equal the number of
/// impls `hashinto_impl_bodies()` actually parsed. If they diverge, the
/// scanner silently dropped something the raw count still sees.
#[test]
fn hashinto_impl_bodies_parses_every_raw_occurrence() {
    let bodies = hashinto_impl_bodies();
    let raw = std::fs::read_to_string(hash_rs_path()).expect("readable hash.rs");
    let src = strip_comments(&raw);
    let raw_count = src.matches("impl HashInto for ").count();
    assert_eq!(
        bodies.len(),
        raw_count,
        "hashinto_impl_bodies() parsed {} impls, but the raw needle \"impl HashInto for \" \
         (comment-stripped) occurs {raw_count} times -- the scanner silently dropped {} impl(s) \
         (a malformed match, an unparseable body, or a spelling it doesn't recognise)",
        bodies.len(),
        raw_count.saturating_sub(bodies.len())
    );
}

/// **AC 6383 / OOS-DP7-11, part 2.** For every `HashInto` impl target classified
/// as a struct: if its own declaration has named fields, it must appear in
/// `named_field_structs()` (else `every_hashed_struct_field_is_hashed_or_allowlisted`
/// silently never checks its fields at all — exactly the class this batch closes).
/// A tuple/unit-shaped struct target is legitimately out of that gate's scope and
/// is skipped, not flagged.
///
/// Separately: is any hashed struct/enum declared without a *bare* `pub`? Every
/// scanner in this file greps the literal needle `"pub struct "` / `"pub enum "`,
/// so `pub(crate) struct Foo` matches none of them — a THIRD instance of
/// OOS-DP7-11's "the gate can't see this spelling" class, distinct from the
/// bare-vs-path-qualified one Part A fixes above.
///
/// **PB-DX7 review L9 (2026-08-12): this second half CANNOT fire independently
/// today, stated rather than left implied.** It only inspects targets absent
/// from `decls.index` — but `every_hashed_type_resolves_to_a_declaration`
/// already asserts (and reddens on) exactly that emptiness independently, for
/// every target not also on `HASHED_PRIMITIVE_TARGETS`. So a non-`pub` hashed
/// type is always caught by that OTHER test first; this half never produces a
/// finding the other test didn't already produce. It is a better, more
/// specific error message layered on the same underlying failure, not an
/// independently-reachable gate — kept for the message quality, not removed,
/// but not to be read as adding coverage beyond `every_hashed_type_resolves_
/// to_a_declaration`.
#[test]
fn every_hashed_struct_is_parsed_by_named_field_structs() {
    let bodies = hashinto_impl_bodies();
    let decls = index_declarations();
    let structs = named_field_structs();
    let shapes = all_struct_shapes();

    let mut missing: Vec<String> = Vec::new();
    let mut checked_named = 0usize;
    for (bare, impl_body) in &bodies {
        let Some(decl) = decls.index.get(bare) else {
            continue; // reported by every_hashed_type_resolves_to_a_declaration
        };
        if !decl_is_struct(decl) {
            continue;
        }
        match shapes.get(bare) {
            Some(StructShape::NamedFields) => {
                checked_named += 1;
                if !structs.contains_key(bare) {
                    missing.push(format!("{bare} (spelled `{}`)", impl_body.spelling));
                }
            }
            Some(StructShape::TupleOrUnit) => {
                // Legitimately out of the field-coverage gate's scope.
            }
            None => {
                missing.push(format!(
                    "{bare} (spelled `{}`) -- resolved as a struct declaration but \
                     all_struct_shapes() could not classify its body shape",
                    impl_body.spelling
                ));
            }
        }
    }
    assert!(
        missing.is_empty(),
        "\n\nThese HashInto struct impl targets declare NAMED fields but are missing from \
         named_field_structs() -- every_hashed_struct_field_is_hashed_or_allowlisted never \
         checks their fields at all:\n  {}\n",
        missing.join("\n  ")
    );
    assert!(
        checked_named > 0,
        "zero named-field struct impl targets were checked; the shape classifier is broken"
    );

    let mut non_pub_found: Vec<String> = Vec::new();
    for (bare, impl_body) in &bodies {
        if decls.index.contains_key(bare) {
            continue; // found by the bare "pub struct "/"pub enum " needle already
        }
        if declared_non_pub(bare) {
            non_pub_found.push(format!("{bare} (spelled `{}`)", impl_body.spelling));
        }
    }
    assert!(
        non_pub_found.is_empty(),
        "\n\nThese HashInto impl targets are declared WITHOUT a bare `pub` (e.g. `pub(crate) \
         struct Foo`), so every scanner's literal \"pub struct \"/\"pub enum \" needle cannot \
         see them at all -- a THIRD instance of OOS-DP7-11's class. Fix the scanners to also \
         match restricted-visibility declarations, or make the type bare `pub` if nothing \
         requires the restriction:\n  {}\n",
        non_pub_found.join("\n  ")
    );
}

/// Non-vacuity + positive/negative controls for the two scope-classification
/// helpers this test adds (`all_struct_shapes`, `src_declares_non_pub`), in the
/// style of `coverage_scanners_are_not_vacuous`.
#[test]
fn struct_and_enum_scope_scanners_are_not_vacuous() {
    let shapes = all_struct_shapes();
    assert!(
        shapes.len() >= MIN_NAMED_STRUCTS,
        "found only {} pub structs of any shape; all_struct_shapes is broken (expected >= {})",
        shapes.len(),
        MIN_NAMED_STRUCTS
    );
    assert_eq!(
        shapes.get("PlayerState").copied(),
        Some(StructShape::NamedFields),
        "positive control failed: PlayerState is a named-field struct"
    );
    assert_eq!(
        shapes.get("SubType").copied(),
        Some(StructShape::TupleOrUnit),
        "positive control failed: SubType(String) is a tuple struct"
    );

    // src_declares_non_pub controls, isolated from real source files.
    assert!(
        src_declares_non_pub("pub(crate) struct Foo { x: u8 }", "Foo"),
        "positive control failed: pub(crate) struct is not a bare pub"
    );
    assert!(
        src_declares_non_pub("struct Foo;", "Foo"),
        "positive control failed: an unmarked struct is not a bare pub"
    );
    assert!(
        !src_declares_non_pub("pub struct Foo { x: u8 }", "Foo"),
        "negative control failed: bare `pub struct Foo` must not be flagged"
    );
    assert!(
        !src_declares_non_pub("pub struct Bar { x: u8 }", "Foo"),
        "negative control failed: matched the wrong name"
    );
    assert!(
        !src_declares_non_pub("pub struct FooBar { x: u8 }", "Foo"),
        "token-boundary control failed: `Foo` matched inside `FooBar`"
    );
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

/// How a field/binding is covered by a `HashInto` impl body (or one enum
/// arm's body), examined per-occurrence and AGGREGATED across every
/// whole-token occurrence of its access expression (`self.<field>` for a
/// struct, or the bare bound identifier for an enum arm — a token can appear
/// more than once, e.g. a `.len()` prefix AND a separate element-wise
/// iteration over the same value).
///
/// Shared by both halves of the SR-19 gate (`struct_field_coverage` for
/// structs, `variant_field_coverage` for enum arm bindings) — the same
/// "summariser passed a technicality" failure mode applies identically to
/// both, and a reviewer should never find the two halves disagreeing about
/// what counts as coverage for the same underlying shape (PB-DX7 follow-up,
/// 2026-08-11: `PendingTrigger.embedded_effect` and
/// `StackObjectKind::ActivatedAbility.embedded_effect` are the SAME field
/// name, the SAME `.is_some()` shape, the SAME documented reasoning — one
/// classifier serving both closes the risk of the two verdicts drifting
/// apart the way they had before this follow-up).
#[derive(Debug, Clone, PartialEq, Eq)]
enum FieldCoverage {
    /// The token never appears in the body at all. This is the ONLY state a
    /// `NOT_HASHED`/`NOT_HASHED_VARIANT_FIELDS` allowlist entry may satisfy
    /// (their own dead-entry guards check raw textual absence, independent of
    /// this classifier) — see `Unverified` below for why that separation
    /// matters.
    NotReferenced,
    /// At least one occurrence is RECOGNISABLY fed to a hasher: a bare
    /// `<token>.hash_into(..)`, a summariser chained to `.hash_into(..)` (the
    /// same evidence `Partial` uses, promoted — one direct feed anywhere
    /// makes the whole value `Full` even if OTHER occurrences are
    /// summarisers), a `for … in <token>`/`&<token>` loop whose body hashes,
    /// a `(*<token> as TYPE).hash_into(..)` discriminant cast, or a
    /// `match <token> {`/`match &<token> {` block whose arms hash. See
    /// `token_reaches_hasher` for the exhaustive list this batch's review
    /// (H2) required be surveyed, not assumed.
    Full,
    /// EVERY occurrence is exactly `<token>.<method>(..).hash_into(..)` for
    /// some non-`hash_into` method, and NONE reaches a hasher any other
    /// recognised way — the value's actual content is discarded and only a
    /// derived summary is fed. Carries the summariser method name(s) seen,
    /// for diagnostics.
    Partial(BTreeSet<String>),
    /// The token DOES appear in the body, but NO occurrence matches any
    /// recognised hashing shape (direct, summariser, iteration, cast, or
    /// match) — e.g. `let _ = <token>;`, a guard-only read
    /// (`if *<token> { … }` with no feed), or a genuinely new idiom this
    /// classifier has not been taught. **PB-DX7 review H2 fix (2026-08-11)**:
    /// the ORIGINAL classifier folded this case into `Full` (fail-OPEN — the
    /// exact `OOS-DP9-13` shape, one spelling over: `let _ =
    /// may_fail_to_find;` passed as "covered"). It is deliberately a FOURTH
    /// state, not folded into `NotReferenced`: the two `NOT_HASHED*`
    /// allowlists' own dead-entry guards require raw textual ABSENCE, so a
    /// field that IS textually present but unverifiably fed cannot be waved
    /// through by either of them — it always fails
    /// `every_hashed_struct_field_is_hashed_or_allowlisted` /
    /// `every_hashed_enum_variant_field_is_hashed_or_allowlisted`, forcing a
    /// human to either feed the value in a recognised shape or extend the
    /// classifier for a genuinely new one (never to silently loosen the rule
    /// for everyone, per the coordinator's explicit instruction).
    Unverified,
}

/// Index of the first non-whitespace byte at or after `i` — rustfmt wraps a
/// long chain onto a new line (`self.permanents_put_into_graveyard_this_turn`
/// followed by `.hash_into(&mut hasher)` on the NEXT line is real, live code
/// at `hash.rs:8409-8410`), so every "is this token immediately followed by
/// X" check below must tolerate whitespace/newlines between the token and
/// what follows it, not just adjacent bytes.
fn skip_ws(b: &[u8], mut i: usize) -> usize {
    while i < b.len() && b[i].is_ascii_whitespace() {
        i += 1;
    }
    i
}

/// If `body[after..]` starts with `.<ident>(<args>)` immediately followed by
/// `.hash_into(`, returns the method identifier (the "summariser"). Else
/// `None` — including when the method IS `hash_into` itself (that's the FULL
/// shape, handled by the caller directly).
fn summariser_chained_to_hash_into(body: &str, after: usize) -> Option<String> {
    let b = body.as_bytes();
    let n = b.len();
    let after = skip_ws(b, after);
    if after >= n || b[after] != b'.' {
        return None;
    }
    let mut i = after + 1;
    let name_start = i;
    while i < n && (b[i].is_ascii_alphanumeric() || b[i] == b'_') {
        i += 1;
    }
    if i == name_start {
        return None;
    }
    let method = &body[name_start..i];
    if method == "hash_into" || i >= n || b[i] != b'(' {
        return None;
    }
    let call_end = match_delim(b, i, b'(', b')');
    let call_end = skip_ws(b, call_end);
    if body[call_end..].starts_with(".hash_into(") {
        Some(method.to_string())
    } else {
        None
    }
}

/// If `body[at..]` is a bare-field-access chain — `.digit` or `.ident` with
/// NO parens (a tuple index or a named field projection, never a method
/// call) — immediately followed by `.hash_into(`, returns true. This is
/// structurally LOSSLESS (a field/tuple-index projection, not a summarising
/// call), so it counts as `Full`, unlike a method-call chain
/// (`summariser_chained_to_hash_into`'s subject), which is lossy and stays
/// `Partial`. Surveyed live: `st.0.hash_into(hasher)`
/// (`ManaRestriction::SubtypeOnly` and 5 siblings, all wrapping `SubType`),
/// `onto_subtype.0.hash_into(hasher)` (`AbilityDefinition::Splice`),
/// `default.0.hash_into(hasher)` (`Effect::ChooseCreatureType`).
fn field_chain_directly_hashed(body: &str, after: usize) -> bool {
    let b = body.as_bytes();
    let after = skip_ws(b, after);
    if after >= b.len() || b[after] != b'.' {
        return false;
    }
    let seg_start = after + 1;
    let mut j = seg_start;
    while j < b.len() && (b[j].is_ascii_alphanumeric() || b[j] == b'_') {
        j += 1;
    }
    if j == seg_start || b.get(j) == Some(&b'(') {
        return false; // empty segment, or it's a method call -- not this shape
    }
    let j = skip_ws(b, j);
    body[j..].starts_with(".hash_into(")
}

/// If the token occurrence at `[at, after)` is immediately wrapped in `(` or
/// `(*` (a parenthesised, possibly-dereferenced expression), and — after an
/// OPTIONAL single chain segment (`.ident` or `.ident(args)`) — is cast
/// (`as TYPE`) and the whole parenthesised expression is immediately
/// followed by `.hash_into(`, returns true. Covers both a bare cast
/// (`(self.current_room as u64).hash_into(hasher)`, `(*c as
/// u8).hash_into(hasher)`) and a cast of a method result
/// (`(self.designations.bits() as u32).hash_into(hasher)` — `.bits()` on a
/// `bitflags` value is a LOSSLESS read of its whole representation, not a
/// summary, so casting and hashing it is `Full`, unlike `.is_some()`/`.len()`
/// with no cast, which stay `Partial`/uncounted).
fn cast_wrapped_and_hashed(body: &str, at: usize, after: usize) -> bool {
    let b = body.as_bytes();
    let preceded_by_paren = at > 0 && b[at - 1] == b'(';
    let preceded_by_paren_deref = at >= 2 && b[at - 1] == b'*' && b[at - 2] == b'(';
    if !(preceded_by_paren || preceded_by_paren_deref) {
        return false;
    }
    let mut j = skip_ws(b, after);
    if j < b.len() && b[j] == b'.' {
        let mut k = j + 1;
        while k < b.len() && (b[k].is_ascii_alphanumeric() || b[k] == b'_') {
            k += 1;
        }
        if k > j + 1 {
            if k < b.len() && b[k] == b'(' {
                k = match_delim(b, k, b'(', b')');
            }
            j = k;
        }
    }
    j = skip_ws(b, j);
    let Some(rest) = body[j..].strip_prefix("as ") else {
        return false;
    };
    let rest = rest.trim_start();
    let ty_len = rest
        .bytes()
        .take_while(|c| c.is_ascii_alphanumeric() || *c == b'_' || *c == b':')
        .count();
    if ty_len == 0 {
        return false;
    }
    let after_ty = rest[ty_len..].trim_start();
    let Some(after_close) = after_ty.strip_prefix(')') else {
        return false;
    };
    after_close.starts_with(".hash_into(")
}

/// True iff `body` contains `if let Some(<name>) = <token>` or `if let
/// Some(<name>) = &<token>`, and the `if` block's body contains at least one
/// `.hash_into(` call — the `if let Some(kw) = grants_keyword { kw.hash_into
/// (hasher); } else { 0u8.hash_into(hasher); }` idiom, and its compound form
/// with a nested iteration (`if let Some(costs) = &self.mode_costs { true.
/// hash_into(hasher); for cost in costs { cost.hash_into(hasher); } } else {
/// false.hash_into(hasher); }`, `ModeSelection.mode_costs`/`.mode_targets`).
/// Deliberately does not require the REBOUND name itself be what feeds the
/// hasher (only that the block does, somewhere) — matching the same
/// body-contains-a-feed pragmatism `token_match_body_hashes` and
/// `token_iteration_body_hashes` already use for their blocks.
fn token_if_let_some_body_hashes(body: &str, token: &str) -> bool {
    let b = body.as_bytes();
    let mut from = 0usize;
    while let Some(rel) = body[from..].find("if let Some(") {
        let at = from + rel;
        let paren_open = at + "if let Some(".len() - 1;
        let close = match_delim(b, paren_open, b'(', b')');
        let mut k = close;
        while k < b.len() && b[k].is_ascii_whitespace() {
            k += 1;
        }
        let Some(rest) = body[k..].strip_prefix('=') else {
            from = at + 1;
            continue;
        };
        let scrutinee_start = k + 1 + (rest.len() - rest.trim_start().len());
        let mut j = scrutinee_start;
        while j < b.len() && b[j] != b'{' {
            j += 1;
        }
        if j >= b.len() {
            from = at + 1;
            continue;
        }
        let scrutinee = body[scrutinee_start..j]
            .trim()
            .trim_start_matches('&')
            .trim();
        // Bare token (`if let Some(costs) = &self.mode_costs`), OR a collection lookup
        // on it (`if let Some(obj) = self.objects.get(&obj_id)` -- surveyed live:
        // `hash.rs:8342`, filtering `self.objects` down to the subset reachable from
        // PUBLIC zones only, CR-required hidden-info exclusion of hand/library; the
        // looked-up value's FULL content is hashed, nothing is lost, it is gated by
        // membership, not summarised).
        let is_scrutinee = scrutinee == token
            || scrutinee.starts_with(&format!("{token}.get("))
            || scrutinee.starts_with(&format!("{token}.get_mut("));
        if is_scrutinee {
            let end = match_delim(b, j, b'{', b'}');
            if body[j..end].contains(".hash_into(") {
                return true;
            }
        }
        from = j;
    }
    false
}

/// True iff `body` contains `match <token> {` or `match &<token> {`, and that
/// match's ENTIRE block contains at least one `.hash_into(` call in some arm
/// — the `match &self.field { None => 0u8.hash_into(hasher), Some(x) => {
/// … x.hash_into(hasher) … } }` idiom used for a hand-matched `Option<T>` (or
/// small inline enum) rather than a nested `HashInto` impl. Surveyed live:
/// 8 struct-side sites (`hash.rs:2334/2342/2353/2825/2844/2958/2975/8434` —
/// the last is `GameState.day_night` inside `public_state_hash` itself).
fn token_match_body_hashes(body: &str, token: &str) -> bool {
    let b = body.as_bytes();
    for prefix in ["match &", "match "] {
        let needle = format!("{prefix}{token}");
        let mut from = 0usize;
        while let Some(rel) = body[from..].find(&needle) {
            let at = from + rel;
            let after = at + needle.len();
            let ok_after = b
                .get(after)
                .is_none_or(|c| !(c.is_ascii_alphanumeric() || *c == b'_'));
            let ok_before = at == 0 || !(b[at - 1].is_ascii_alphanumeric() || b[at - 1] == b'_');
            if ok_after && ok_before {
                let mut j = after;
                while j < b.len() && b[j] != b'{' {
                    j += 1;
                }
                if j < b.len() {
                    let end = match_delim(b, j, b'{', b'}');
                    if body[j..end].contains(".hash_into(") {
                        return true;
                    }
                }
            }
            from = after;
        }
    }
    false
}

/// True iff `body` contains a `for` loop iterating `<token>` — bare,
/// `&<token>`, `<token>.iter()`, or `&<token>.iter()`/`.into_iter()` — whose
/// body contains at least one `.hash_into(` call. The
/// `(self.field.len() as u64).hash_into(hasher); for x in &self.field {
/// x.hash_into(hasher); }` length-prefix-then-iterate idiom (e.g.
/// `PendingZoneChange.already_applied`, `GameState.players`) relies on this.
fn token_iteration_body_hashes(body: &str, token: &str) -> bool {
    let b = body.as_bytes();
    let mut from = 0usize;
    while let Some(rel) = body[from..].find("for ") {
        let at = from + rel;
        let after_for = at + "for ".len();
        let Some(in_rel) = body[after_for..].find(" in ") else {
            from = after_for;
            continue;
        };
        let in_at = after_for + in_rel;
        let src_start = in_at + " in ".len();
        let mut j = src_start;
        while j < b.len() && b[j] != b'{' {
            j += 1;
        }
        if j >= b.len() {
            from = src_start;
            continue;
        }
        let iter_src = body[src_start..j].trim();
        let stripped = iter_src.trim_start_matches('&');
        let stripped = stripped
            .trim_end_matches(".into_iter()")
            .trim_end_matches(".iter()")
            .trim();
        if stripped == token {
            let end = match_delim(b, j, b'{', b'}');
            if body[j..end].contains(".hash_into(") {
                return true;
            }
        }
        from = j;
    }
    false
}

/// Classify every whole-token occurrence of `needle` in `body` and aggregate.
/// **PB-DX7 review H2 fix (2026-08-11)**: `Full` now requires that at least
/// ONE occurrence — or a body-level shape keyed on the same token — is
/// recognisably fed to a hasher (direct, summariser, iteration, cast, or
/// match; see `FieldCoverage::Full`'s doc for the full list, each surveyed
/// against real `hash.rs` shapes before being added, per the coordinator's
/// instruction). An occurrence matching none of them is `Unverified`, not
/// `Full` — the ORIGINAL bug's fail-open `else` arm is gone. The shared core
/// both `struct_field_coverage` (`needle = "self.<field>"`) and
/// `variant_field_coverage` (`needle = "<binding>"`, no prefix — an enum arm
/// binds a bare local name) delegate to.
fn token_coverage(body: &str, needle: &str) -> FieldCoverage {
    let b = body.as_bytes();
    let mut from = 0;
    let mut any = false;
    let mut has_full = token_iteration_body_hashes(body, needle)
        || token_match_body_hashes(body, needle)
        || token_if_let_some_body_hashes(body, needle);
    let mut has_partial = false;
    let mut summarisers: BTreeSet<String> = BTreeSet::new();

    while let Some(rel) = body[from..].find(needle) {
        let at = from + rel;
        let after = at + needle.len();
        let ok_after = b
            .get(after)
            .is_none_or(|c| !(c.is_ascii_alphanumeric() || *c == b'_'));
        let ok_before = at == 0 || !(b[at - 1].is_ascii_alphanumeric() || b[at - 1] == b'_');
        if !(ok_after && ok_before) {
            from = at + 1;
            continue;
        }
        any = true;
        if body[skip_ws(b, after)..].starts_with(".hash_into(") {
            has_full = true;
        } else if let Some(method) = summariser_chained_to_hash_into(body, after) {
            has_partial = true;
            summarisers.insert(method);
        } else if field_chain_directly_hashed(body, after)
            || cast_wrapped_and_hashed(body, at, after)
        {
            has_full = true;
        }
        // Else: this specific occurrence matches no recognised shape. It
        // contributes neither Full nor Partial evidence on its own — a
        // body-level shape (iteration/match/if-let, computed above) or
        // another occurrence may still supply it.
        from = after;
    }

    if !any {
        FieldCoverage::NotReferenced
    } else if has_full {
        FieldCoverage::Full
    } else if has_partial {
        FieldCoverage::Partial(summarisers)
    } else {
        FieldCoverage::Unverified
    }
}

/// Coverage of a struct field, examined across every whole-token
/// `self.<field>` occurrence in that struct's `HashInto` impl body.
fn struct_field_coverage(body: &str, field: &str) -> FieldCoverage {
    token_coverage(body, &format!("self.{field}"))
}

/// Coverage of an enum arm's bound variable, examined across every
/// whole-token occurrence of `binding` in that ONE ARM's body (not the whole
/// enum impl — a bound name is scoped to its own arm, unlike a struct field's
/// `self.` access which is valid anywhere in the impl).
fn variant_field_coverage(arm_body: &str, binding: &str) -> FieldCoverage {
    token_coverage(arm_body, binding)
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
    let pt = &bodies
        .get("PendingTrigger")
        .expect("PendingTrigger impl")
        .body;
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
            &bodies.get("StackObject").expect("StackObject impl").body,
            "cast_from_top_with_bonus"
        ),
        "SR-19: StackObject must now hash cast_from_top_with_bonus"
    );

    // struct_field_coverage controls, isolated from real source.
    assert_eq!(
        struct_field_coverage("self.x.is_some().hash_into(hasher);", "x"),
        FieldCoverage::Partial(["is_some".to_string()].into_iter().collect()),
        "positive control failed: a lone summariser-then-hash_into is Partial"
    );
    assert_eq!(
        struct_field_coverage("self.x.hash_into(hasher);", "x"),
        FieldCoverage::Full,
        "positive control failed: a bare feed is Full"
    );
    assert_eq!(
        struct_field_coverage("for y in &self.x { y.hash_into(hasher); }", "x"),
        FieldCoverage::Full,
        "positive control failed: an iteration-shaped occurrence is Full, not Partial"
    );
    assert_eq!(
        struct_field_coverage(
            "self.x.len().hash_into(hasher); for y in &self.x { y.hash_into(hasher); }",
            "x"
        ),
        FieldCoverage::Full,
        "positive control failed: a len()-prefix PLUS a separate iteration must aggregate to \
         Full, mirroring every real `.len()` site in hash.rs (they are all followed by \
         iteration, per PB-DX7's coordinator-directed audit)"
    );
    assert_eq!(
        struct_field_coverage("self.y.hash_into(hasher);", "x"),
        FieldCoverage::NotReferenced,
        "negative control failed: field `x` does not appear at all"
    );

    // The two real PARTIALLY_HASHED entries are genuinely Partial, not Full.
    assert_eq!(
        struct_field_coverage(pt, "embedded_effect"),
        FieldCoverage::Partial(["is_some".to_string()].into_iter().collect()),
        "PendingTrigger.embedded_effect must classify as Partial (self.embedded_effect.is_some())"
    );
    assert_eq!(
        struct_field_coverage(
            &bodies
                .get("PlayFromTopPermission")
                .expect("PlayFromTopPermission impl")
                .body,
            "on_cast_effect"
        ),
        FieldCoverage::Partial(["is_some".to_string()].into_iter().collect()),
        "PlayFromTopPermission.on_cast_effect must classify as Partial \
         (self.on_cast_effect.is_some())"
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
    let partial_allow: BTreeSet<(&str, &str)> = PARTIALLY_HASHED
        .iter()
        .map(|(t, f, _reason)| (*t, *f))
        .collect();

    let mut covered = 0usize;
    let mut fields_checked = 0usize;
    let mut violations: Vec<String> = Vec::new();

    for (ty, fields) in &structs {
        let Some(impl_body) = bodies.get(ty) else {
            continue; // struct without a HashInto impl — out of this gate's scope
        };
        let body = &impl_body.body;
        covered += 1;
        for f in fields {
            fields_checked += 1;
            match struct_field_coverage(body, f) {
                FieldCoverage::Full => {}
                FieldCoverage::NotReferenced => {
                    if !allow.contains(&(ty.as_str(), f.as_str())) {
                        violations.push(format!("{ty}.{f} -- never referenced at all"));
                    }
                }
                FieldCoverage::Partial(summarisers) => {
                    if !partial_allow.contains(&(ty.as_str(), f.as_str())) {
                        let methods = summarisers.into_iter().collect::<Vec<_>>().join(", ");
                        violations.push(format!(
                            "{ty}.{f} -- PARTIAL coverage only: every occurrence is \
                             `self.{f}.{{{methods}}}(..).hash_into(..)`, discarding the \
                             field's actual value, and it is not on PARTIALLY_HASHED"
                        ));
                    }
                }
                FieldCoverage::Unverified => {
                    violations.push(format!(
                        "{ty}.{f} -- UNVERIFIED: self.{f} is referenced in the body, but no \
                         occurrence matches a recognised hashing shape (direct feed, \
                         summariser, iteration, cast, or match) -- its value may never reach \
                         the hasher (PB-DX7 review H2)"
                    ));
                }
            }
        }
    }

    assert!(
        violations.is_empty(),
        "\n\nThese struct fields are declared but never fully fed to their type's `HashInto` \
         impl, and are not on the NOT_HASHED or PARTIALLY_HASHED allowlists:\n  {}\n\n\
         Two independent game states differing only in such a field hash IDENTICALLY, so \
         distributed-verification / replay divergence-detection is blind to it (SR-7 gotcha 5). \
         For a field never referenced at all: either add `self.<field>.hash_into(hasher)` to the \
         impl (and bump HASH_SCHEMA_VERSION per the state/hash.rs checklist), or, if the field is \
         genuinely not game state, add it to NOT_HASHED with a one-line rationale. For a field \
         only referenced via a summarising method (e.g. `.is_some()`): either hash the field's \
         real value instead (bump HASH_SCHEMA_VERSION), or, if the summary is a deliberate \
         reduction, add it to PARTIALLY_HASHED with a one-line rationale.\n",
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

/// The `PARTIALLY_HASHED` allowlist is honest: every entry names a real
/// declared field of a hashed struct whose coverage is GENUINELY `Partial`
/// (every occurrence a summariser chained to `.hash_into`) — not `Full` (the
/// field got fully hashed and the entry is now stale) and not `NotReferenced`
/// (the field's only feed was removed entirely, which is `NOT_HASHED`'s
/// territory, not this one's). Mirrors `not_hashed_allowlist_has_no_dead_entries`.
#[test]
fn partially_hashed_allowlist_has_no_dead_entries() {
    let bodies = hashinto_impl_bodies();
    let structs = named_field_structs();

    for (ty, field, _reason) in PARTIALLY_HASHED {
        let fields = structs.get(*ty).unwrap_or_else(|| {
            panic!(
                "PARTIALLY_HASHED entry ({ty}, {field}): `{ty}` is not a named-field struct \
                 under the scan roots"
            )
        });
        assert!(
            fields.iter().any(|f| f == field),
            "PARTIALLY_HASHED entry ({ty}, {field}): `{ty}` declares no field named `{field}` \
             (dead entry — remove it or fix the name)"
        );
        let impl_body = bodies.get(*ty).unwrap_or_else(|| {
            panic!("PARTIALLY_HASHED entry ({ty}, {field}): `{ty}` has no `impl HashInto for {ty}`")
        });
        match struct_field_coverage(&impl_body.body, field) {
            FieldCoverage::Partial(_) => {} // legitimate
            FieldCoverage::Full => panic!(
                "PARTIALLY_HASHED entry ({ty}, {field}): `{ty}::{field}` is now FULLY hashed — \
                 remove this entry (dead)."
            ),
            FieldCoverage::NotReferenced => panic!(
                "PARTIALLY_HASHED entry ({ty}, {field}): `{ty}::{field}` is not referenced at \
                 all any more — this is NOT_HASHED's territory now, not PARTIALLY_HASHED's \
                 (dead entry)."
            ),
            FieldCoverage::Unverified => panic!(
                "PARTIALLY_HASHED entry ({ty}, {field}): `{ty}::{field}` no longer classifies \
                 as a recognised summariser shape (Unverified) — either the site changed shape \
                 or the classifier regressed; re-derive the entry's status, do not assume it \
                 is still Partial."
            ),
        }
    }
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
        let impl_body = bodies.get(*ty).unwrap_or_else(|| {
            panic!("NOT_HASHED entry ({ty}, {field}): `{ty}` has no `impl HashInto for {ty}`")
        });
        assert!(
            !body_references_field(&impl_body.body, field),
            "NOT_HASHED entry ({ty}, {field}): `{ty}`'s HashInto DOES hash `{field}` — remove it \
             from the allowlist (dead entry). The allowlist is for fields that are NOT hashed."
        );
    }
}

// ════════════════════════════════════════════════════════════════════════════
// SR-19 Part B (PB-DX7 / OOS-DP9-13): HashInto-vs-enum-variant field-coverage gate
// ════════════════════════════════════════════════════════════════════════════
//
// The struct half above (`every_hashed_struct_field_is_hashed_or_allowlisted`)
// only ever covered per-type struct impls, which uniformly read `self.<field>`.
// Enum impls match on `self` and bind each variant's payload to LOCAL names, so
// the very same "field silently dropped from HashInto" failure mode existed for
// enums with no gate at all: rewriting
// `EffectChoiceQuestion::SearchLibrary { candidates, may_fail_to_find }` as
// `{ candidates, .. }` drops `may_fail_to_find` from the hash stream and every
// gate in the workspace stayed green (**OOS-DP9-13**, reproduced at HEAD before
// this batch — see `memory/primitives/pb-DX7-gate-spec.md` §1).
//
// This gate parses each hashed enum's `impl HashInto for <T>` body and its own
// declared variant list, and asserts:
//   - a `(*self as u8)`-shaped impl (no `match self` at all) only ever declares
//     Unit variants — a data-carrying variant there could never reach the
//     hasher, since the whole impl is a bare cast;
//   - otherwise every declared variant has a matching arm in the top-level
//     `match self { … }`, no `_ =>` (or bare-identifier) catch-all exists once
//     any variant carries data, and every field a variant's pattern binds is
//     both present (no `..` rest pattern, no bare `_`, exact tuple arity) and
//     actually referenced in that arm's body — or is on the
//     `NOT_HASHED_VARIANT_FIELDS` allowlist with a stated reason.

/// One declared enum variant's payload shape.
#[derive(Debug, Clone, PartialEq, Eq)]
enum VariantKind {
    Unit,
    Tuple(usize),
    Named(Vec<String>),
}

/// One declared enum variant.
#[derive(Debug, Clone)]
struct Variant {
    name: String,
    kind: VariantKind,
}

/// `(enum, variant, field, reason)` — fields of a hashed enum's variant that are
/// deliberately NOT fed to that enum's `HashInto` AT ALL (the bound identifier
/// never appears in the arm body). `field` for a tuple variant is its
/// zero-based index as a string (`"0"`, `"1"`, …). Dead entries fail
/// `not_hashed_variant_fields_allowlist_has_no_dead_entries`.
///
/// **Empty today.** See PB-DX7's handoff for the disposition of every field this
/// gate newly put in scope. For a variant field that IS referenced but only via
/// a summarising method (e.g. `.is_some()`), see `PARTIALLY_HASHED_VARIANT_FIELDS`
/// below — a different disposition for a different shape of coverage.
const NOT_HASHED_VARIANT_FIELDS: &[(&str, &str, &str, &str)] = &[];

/// `(enum, variant, field, reason)` — variant fields whose ONLY appearance in
/// their arm's body is `<binding>.<summariser>(..).hash_into(..)`, discarding
/// the field's actual content. The enum-side mirror of `PARTIALLY_HASHED`
/// (structs); see that constant's doc for the shared rationale.
///
/// **PB-DX7 follow-up (2026-08-11), coordinator-directed.** Found while
/// implementing the struct half: the identical `.is_some()` shape exists on
/// two `StackObjectKind` arms carrying a field also named `embedded_effect` —
/// SAME field name, SAME shape, SAME documented reasoning as
/// `PendingTrigger.embedded_effect` (see `PARTIALLY_HASHED`), and until this
/// entry existed the enum gate silently reported them as fully covered while
/// the struct gate (after PB-DX7's first pass) correctly reported their
/// struct-side sibling as partial — two halves of one gate disagreeing about
/// the same failure mode in the same file.
///
/// `StackObjectKind::ForecastAbility.embedded_effect` deliberately does NOT
/// appear here — it hashes the full effect (`embedded_effect.hash_into(hasher)`,
/// `hash.rs:4241`), and the dead-entry guard would reject an entry for it if
/// one were ever added by mistake. That asymmetry (two variants summarise the
/// same field name, one hashes it fully) is exactly what this allowlist exists
/// to make visible.
///
/// **PB-DX7 review M7 fix (2026-08-11): both citations below were WRONG.**
/// Both originally cited `hash.rs:4105-4111`, which is the impl HEADER and
/// the `Spell` arm — no reasoning about `embedded_effect` appears there. The
/// coordinator approved the entries without checking the cites, and named
/// that as their own error too; a false citation is worse than none, and
/// this batch's own subject is exactly claims that don't hold up when
/// checked. `ActivatedAbility`'s arm ORIGINALLY carried no comment of its
/// own (feed at `hash.rs:4120`, unmodified); a comment mirroring
/// `TriggeredAbility`'s was ADDED there in this same fix (comment-only) so
/// the "documented in-source" premise is now literally true rather than
/// inferred. `TriggeredAbility`'s pre-existing reasoning is at
/// `hash.rs:4136-4142`, its feed at `:4143`.
const PARTIALLY_HASHED_VARIANT_FIELDS: &[(&str, &str, &str, &str)] = &[
    (
        "StackObjectKind",
        "ActivatedAbility",
        "embedded_effect",
        "presence suffices for divergence detection -- the effect is a copy of \
         the source ability's, redundant with source_object + ability_index \
         (hash.rs:4120-4123, comment added by PB-DX7 review M7 fix, mirroring \
         TriggeredAbility's pre-existing reasoning)",
    ),
    (
        "StackObjectKind",
        "TriggeredAbility",
        "embedded_effect",
        "presence suffices for divergence detection -- the effect is a copy of \
         the source ability's, redundant with source_object + ability_index \
         (hash.rs:4136-4142, MR-B12-04 / SR-19)",
    ),
];

/// Split `s` at depth-0 commas (bracket/paren/brace/angle-aware, string-literal
/// aware). Shared by the enum-body variant splitter and the tuple-pattern /
/// named-pattern binding splitters below — the same idiom `struct_field_names`
/// uses for a struct body, generalized to return the raw segments rather than
/// just the field-name-before-a-colon.
fn split_depth0_commas(s: &str) -> Vec<&str> {
    let b = s.as_bytes();
    let n = b.len();
    let mut out: Vec<&str> = Vec::new();
    let mut depth = 0i32;
    let mut start = 0usize;
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
                out.push(&s[start..i]);
                start = i + 1;
            }
            _ => {}
        }
        i += 1;
    }
    out.push(&s[start..]);
    out
}

/// Classify one (already depth-0-split, non-empty, trimmed) enum-variant
/// declaration segment: `Name`, `Name(T, U)`, or `Name { field: T, field2: U }`.
fn classify_variant_segment(seg: &str) -> Option<Variant> {
    let trimmed = seg.trim();
    if trimmed.is_empty() {
        return None;
    }
    let sb = trimmed.as_bytes();
    let mut k = 0usize;
    while k < sb.len() && (sb[k].is_ascii_alphanumeric() || sb[k] == b'_') {
        k += 1;
    }
    let name = trimmed[..k].to_string();
    if name.is_empty() {
        return None;
    }
    let rest = trimmed[k..].trim_start();
    let kind = if let Some(inner) = rest.strip_prefix('{').and_then(|s| {
        let t = s.trim_end();
        t.strip_suffix('}')
    }) {
        VariantKind::Named(struct_field_names(inner))
    } else if let Some(inner) = rest.strip_prefix('(').and_then(|s| {
        let t = s.trim_end();
        t.strip_suffix(')')
    }) {
        let n = split_depth0_commas(inner)
            .into_iter()
            .filter(|s| !s.trim().is_empty())
            .count();
        VariantKind::Tuple(n)
    } else {
        // Unit, possibly with an explicit `= N` discriminant -- irrelevant here.
        VariantKind::Unit
    };
    Some(Variant { name, kind })
}

/// Every `pub enum` under the scan roots → its declared variants, in declaration
/// order. Mirrors `named_field_structs()`'s parsing idiom one level deeper: a
/// struct splits at fields; an enum splits at VARIANTS, each of which may itself
/// be struct-like (`Named`), tuple-like (`Tuple`), or bare (`Unit`).
fn named_enum_variants() -> BTreeMap<String, Vec<Variant>> {
    let root = workspace_root();
    let mut out: BTreeMap<String, Vec<Variant>> = BTreeMap::new();
    for scan_root in SCAN_ROOTS {
        let mut files = Vec::new();
        walk(&root.join(scan_root), &mut files);
        files.sort();
        for file in files {
            let raw = std::fs::read_to_string(&file).expect("readable source");
            let src = strip_comments(&raw);
            let b = src.as_bytes();
            let kw = "pub enum ";
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
                let mut j = after + name.len();
                while j < b.len() && b[j] != b'{' && b[j] != b';' {
                    j += 1;
                }
                if j >= b.len() || b[j] != b'{' {
                    continue;
                }
                let end = match_delim(b, j, b'{', b'}');
                let body = strip_attributes(&src[j + 1..end - 1]);
                out.entry(name).or_insert_with(|| {
                    split_depth0_commas(&body)
                        .into_iter()
                        .filter_map(classify_variant_segment)
                        .collect()
                });
                from = end;
            }
        }
    }
    out
}

/// True iff `body` contains `token` as a whole identifier (word-boundary on both
/// sides) — the enum-arm analogue of `body_references_field`'s `self.field`
/// check, for a bound pattern-local variable rather than a struct field access.
fn body_references_token(body: &str, token: &str) -> bool {
    let b = body.as_bytes();
    let mut from = 0;
    while let Some(rel) = body[from..].find(token) {
        let at = from + rel;
        let after = at + token.len();
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

/// Split a match-arm pattern like `Foo::Bar(x, y)` / `Foo::Bar { x, y }` /
/// `Foo::Bar` into the variant's bare name (its last `::`-separated identifier)
/// and its raw payload text (the `(...)`/`{...}` including delimiters, or `""`
/// for a unit pattern).
fn split_pattern(pattern: &str) -> (String, String) {
    let pattern = pattern.trim();
    let b = pattern.as_bytes();
    let n = b.len();
    let mut i = 0usize;
    let mut last_ident_start = 0usize;
    while i < n {
        if b[i].is_ascii_alphanumeric() || b[i] == b'_' {
            i += 1;
        } else if i + 1 < n && b[i] == b':' && b[i + 1] == b':' {
            i += 2;
            last_ident_start = i;
        } else {
            break;
        }
    }
    let name = pattern[last_ident_start..i].to_string();
    let payload = pattern[i..].trim().to_string();
    (name, payload)
}

/// Parse the arms of a `match self { … }` BODY (the text strictly between the
/// outer braces) into `(pattern, arm_body)` pairs, splitting at depth-0`=>`.
/// Handles both block-bodied arms (`Pat => { … }`) and expression-bodied arms
/// terminated by a depth-0 comma (`Pat => expr,`), matching real Rust match-arm
/// syntax rather than a naive comma split (which would misparse a block-bodied
/// arm's internal commas as arm boundaries).
fn parse_match_arms(body: &str) -> Vec<(String, String)> {
    let b = body.as_bytes();
    let n = b.len();
    let mut arms = Vec::new();
    let mut pat_start = 0usize;
    let mut depth = 0i32;
    let mut i = 0usize;
    while i < n {
        if let Some(len) = literal_len(b, i) {
            i += len;
            continue;
        }
        let c = b[i];
        if c == b'<' || c == b'(' || c == b'[' || c == b'{' {
            depth += 1;
            i += 1;
        } else if c == b'>' || c == b')' || c == b']' || c == b'}' {
            depth -= 1;
            i += 1;
        } else if c == b'=' && depth == 0 && i + 1 < n && b[i + 1] == b'>' {
            let pattern = body[pat_start..i].trim().to_string();
            let mut j = i + 2;
            while j < n && b[j].is_ascii_whitespace() {
                j += 1;
            }
            let (arm_text, next) = if j < n && b[j] == b'{' {
                let end = match_delim(b, j, b'{', b'}');
                let mut k = end;
                while k < n && b[k].is_ascii_whitespace() {
                    k += 1;
                }
                if k < n && b[k] == b',' {
                    k += 1;
                }
                (body[j..end].to_string(), k)
            } else {
                let mut d2 = 0i32;
                let mut k = j;
                while k < n {
                    if let Some(len) = literal_len(b, k) {
                        k += len;
                        continue;
                    }
                    let ck = b[k];
                    if ck == b'<' || ck == b'(' || ck == b'[' || ck == b'{' {
                        d2 += 1;
                    } else if ck == b'>' || ck == b')' || ck == b']' || ck == b'}' {
                        if d2 == 0 {
                            break;
                        }
                        d2 -= 1;
                    } else if ck == b',' && d2 == 0 {
                        break;
                    }
                    k += 1;
                }
                let arm_text = body[j..k].to_string();
                let mut kk = k;
                if kk < n && b[kk] == b',' {
                    kk += 1;
                }
                (arm_text, kk)
            };
            arms.push((pattern, arm_text));
            pat_start = next;
            i = next;
        } else {
            i += 1;
        }
    }
    arms
}

/// Locate the top-level `match self { … }` in an impl body and return the text
/// strictly between its outer braces, or `None` if the body has no `match self`
/// at all (the `(*self as u8)` shape).
fn top_level_match_self_body(body: &str) -> Option<&str> {
    let start = body.find("match self")?;
    let after = start + "match self".len();
    let b = body.as_bytes();
    let mut j = after;
    while j < b.len() && b[j] != b'{' {
        j += 1;
    }
    if j >= b.len() {
        return None;
    }
    let end = match_delim(b, j, b'{', b'}');
    Some(&body[j + 1..end - 1])
}

/// True iff `body`'s top-level `match self { ... }` is assigned to a `let`
/// binding that is later fed to `.hash_into(` — the `let disc: u8 = match
/// self { A => 0, B => 1, ... }; disc.hash_into(hasher);` INDIRECT
/// discriminant idiom (surveyed live: `AltCostKind`, `DungeonId`). When this
/// holds, an individual Unit-variant arm legitimately has no `.hash_into(`
/// of its own — the bare literal it evaluates to is what gets hashed,
/// collectively, once, after the match — so the per-arm "feeds nothing"
/// check (M3) must not fire for any arm of this enum.
fn match_self_result_is_bound_and_hashed(body: &str) -> bool {
    let Some(match_pos) = body.find("match self") else {
        return false;
    };
    let before = body[..match_pos].trim_end();
    let Some(before_eq) = before.strip_suffix('=') else {
        return false;
    };
    let before_eq = before_eq.trim_end();
    let Some(let_pos) = before_eq.rfind("let ") else {
        return false;
    };
    let after_let = &before_eq[let_pos + "let ".len()..];
    let name: String = after_let
        .chars()
        .take_while(|c| c.is_ascii_alphanumeric() || *c == '_')
        .collect();
    if name.is_empty() {
        return false;
    }
    let b = body.as_bytes();
    let mut j = match_pos;
    while j < b.len() && b[j] != b'{' {
        j += 1;
    }
    if j >= b.len() {
        return false;
    }
    let end = match_delim(b, j, b'{', b'}');
    let needle = format!("{name}.hash_into(");
    body[end..].contains(&needle)
}

/// Non-vacuity floors for the enum gate. Measured at implementation time (79
/// hashed enums / 1,252 variants / 1,097 variant-fields checked) and floored at
/// roughly 2/3 — well below the real counts, so this catches a scanner that
/// broke, not a codebase that grew.
const MIN_HASHED_ENUMS: usize = 52;
const MIN_VARIANTS_CHECKED: usize = 830;
const MIN_VARIANT_FIELDS_CHECKED: usize = 730;
/// **PB-DX7 review L4 fix (2026-08-12).** `named_enum_variants()` returns
/// EVERY `pub enum` under the scan roots (measured: 109), not just the 79
/// hashed ones — `enum_coverage_scanners_are_not_vacuous` was floored against
/// `MIN_HASHED_ENUMS` (52, a floor for the HASHED subset), leaving ~2x
/// unintended slack against the true 109-enum population. Separate floor,
/// same ~2/3 rule, against the real denominator.
const MIN_DECLARED_ENUMS: usize = 72;

/// **AC 6383 / OOS-DP9-13.** Every field of every hashed enum's declared variant
/// is fed to that enum's `HashInto`, or is on the `NOT_HASHED_VARIANT_FIELDS`
/// allowlist. A field silently dropped from a variant's arm (the enum analogue
/// of the struct half's SR-7 haunt-field failure mode) fails here.
#[test]
fn every_hashed_enum_variant_field_is_hashed_or_allowlisted() {
    let bodies = hashinto_impl_bodies();
    let decls = index_declarations();
    let enum_variants = named_enum_variants();
    let allow: BTreeSet<(&str, &str, &str)> = NOT_HASHED_VARIANT_FIELDS
        .iter()
        .map(|(e, v, f, _)| (*e, *v, *f))
        .collect();
    let partial_allow: BTreeSet<(&str, &str, &str)> = PARTIALLY_HASHED_VARIANT_FIELDS
        .iter()
        .map(|(e, v, f, _)| (*e, *v, *f))
        .collect();

    let mut violations: Vec<String> = Vec::new();
    let mut enums_checked = 0usize;
    let mut variants_checked = 0usize;
    let mut variant_fields_checked = 0usize;

    for (bare, impl_body) in &bodies {
        let Some(decl) = decls.index.get(bare) else {
            continue; // reported by every_hashed_type_resolves_to_a_declaration
        };
        if !decl_is_enum(decl) {
            continue;
        }
        let Some(variants) = enum_variants.get(bare) else {
            violations.push(format!(
                "{bare}: has an `impl HashInto` but named_enum_variants() found no declaration \
                 for it"
            ));
            continue;
        };
        enums_checked += 1;
        let body = &impl_body.body;

        let Some(match_body) = top_level_match_self_body(body) else {
            // (*self as u8) shape: every declared variant must be Unit, or its
            // payload can never reach the hasher.
            for v in variants {
                variants_checked += 1;
                if v.kind != VariantKind::Unit {
                    violations.push(format!(
                        "{bare}::{}: impl has no `match self` (a bare-cast shape), but this \
                         variant carries data -- its payload can never reach the hasher",
                        v.name
                    ));
                }
            }
            // PB-DX7 review M3: even an all-Unit enum in this shape must feed SOMETHING --
            // `let _ = hasher;` in place of `(*self as u8).hash_into(hasher);` leaves every
            // variant Unit (the loop above stays green) but feeds zero bytes for every
            // variant, so all of them hash identically to each other.
            if !body.contains(".hash_into(") {
                violations.push(format!(
                    "{bare}: has no `match self` (a bare-cast shape) AND its body feeds \
                     NOTHING to the hasher -- every variant hashes identically"
                ));
            }
            continue;
        };

        // PB-DX7 review M3, exception surveyed live (AltCostKind, DungeonId): the
        // `let disc: u8 = match self { A => 0, ... }; disc.hash_into(hasher);` indirect
        // idiom legitimately has NO `.hash_into(` inside any individual arm -- the value
        // the match evaluates to is hashed once, collectively, after the match closes.
        let indirect_discriminant_hashed = match_self_result_is_bound_and_hashed(body);

        let has_data_variant = variants.iter().any(|v| v.kind != VariantKind::Unit);
        let arms = parse_match_arms(match_body);
        let mut arm_seen: BTreeSet<String> = BTreeSet::new();

        for (pattern, arm_body) in &arms {
            let trimmed_pattern = pattern.trim();
            let (variant_name, payload) = split_pattern(trimmed_pattern);

            if trimmed_pattern == "_" || (variant_name.is_empty() && !trimmed_pattern.is_empty()) {
                if has_data_variant {
                    violations.push(format!(
                        "{bare}: has a `_ =>` (or unparseable) catch-all-shaped arm `{trimmed_pattern}` \
                         while at least one variant carries data -- a new variant could fall into \
                         it silently"
                    ));
                }
                continue;
            }

            let Some(decl_variant) = variants.iter().find(|v| v.name == variant_name) else {
                violations.push(format!(
                    "{bare}: arm pattern `{trimmed_pattern}` names variant `{variant_name}`, \
                     which named_enum_variants() did not find declared on this enum"
                ));
                continue;
            };
            arm_seen.insert(variant_name.clone());
            variants_checked += 1;

            match &decl_variant.kind {
                VariantKind::Unit => {
                    if !payload.is_empty() {
                        violations.push(format!(
                            "{bare}::{variant_name}: declared as a unit variant but the arm \
                             pattern carries a payload `{payload}`"
                        ));
                    }
                    // PB-DX7 review M3: a Unit variant has no fields, so the checks above
                    // are the ONLY thing this gate examined about it -- nothing required the
                    // arm to feed the hasher anything at all. `GiftType::Food => {}` passed
                    // both this gate (payload-free, present) and the discriminant ratchet
                    // (no integer literal -> skipped). Two Unit variants with empty bodies
                    // hash IDENTICALLY (both feed zero bytes for this arm), which is the
                    // exact harm SR-19 exists to prevent. Skipped for the indirect-
                    // discriminant idiom (`AltCostKind`, `DungeonId`), where the bare
                    // literal each such arm evaluates to genuinely is hashed, just not
                    // inside the arm itself.
                    if !indirect_discriminant_hashed && !arm_body.contains(".hash_into(") {
                        violations.push(format!(
                            "{bare}::{variant_name}: this arm's body feeds NOTHING to the \
                             hasher (no `.hash_into(` call anywhere in it) -- two Unit \
                             variants with empty arm bodies hash identically"
                        ));
                    }
                }
                VariantKind::Tuple(n) => {
                    match payload.strip_prefix('(').and_then(|s| s.strip_suffix(')')) {
                        None => violations.push(format!(
                            "{bare}::{variant_name}: declared as a {n}-tuple variant but the arm \
                             pattern has no `(...)` payload (`{payload}`)"
                        )),
                        Some(inner) => {
                            let segs: Vec<String> = split_depth0_commas(inner)
                                .into_iter()
                                .map(|s| s.trim().to_string())
                                .filter(|s| !s.is_empty())
                                .collect();
                            if segs.iter().any(|s| s == "..") {
                                violations.push(format!(
                                    "{bare}::{variant_name}: tuple pattern contains a `..` rest \
                                     pattern -- a future field would silently fall outside the \
                                     binding"
                                ));
                            } else if segs.len() != *n {
                                violations.push(format!(
                                    "{bare}::{variant_name}: declared with {n} tuple field(s) but \
                                     the arm pattern binds {} (`{payload}`)",
                                    segs.len()
                                ));
                            } else {
                                for (idx, s) in segs.iter().enumerate() {
                                    variant_fields_checked += 1;
                                    let idx_str = idx.to_string();
                                    if s == "_" {
                                        violations.push(format!(
                                            "{bare}::{variant_name}.{idx}: tuple field bound to \
                                             `_` -- its value is discarded, never fed to the \
                                             hasher"
                                        ));
                                        continue;
                                    }
                                    let key =
                                        (bare.as_str(), variant_name.as_str(), idx_str.as_str());
                                    match variant_field_coverage(arm_body, s) {
                                        FieldCoverage::Full => {}
                                        FieldCoverage::NotReferenced => {
                                            if !allow.contains(&key) {
                                                violations.push(format!(
                                                    "{bare}::{variant_name}.{idx} (bound as \
                                                     `{s}`): declared but never referenced in \
                                                     the arm body"
                                                ));
                                            }
                                        }
                                        FieldCoverage::Partial(summarisers) => {
                                            if !partial_allow.contains(&key) {
                                                let methods = summarisers
                                                    .into_iter()
                                                    .collect::<Vec<_>>()
                                                    .join(", ");
                                                violations.push(format!(
                                                    "{bare}::{variant_name}.{idx} (bound as \
                                                     `{s}`): PARTIAL coverage only: every \
                                                     occurrence is `{s}.{{{methods}}}(..)\
                                                     .hash_into(..)`, discarding the field's \
                                                     actual value, and it is not on \
                                                     PARTIALLY_HASHED_VARIANT_FIELDS"
                                                ));
                                            }
                                        }
                                        FieldCoverage::Unverified => {
                                            violations.push(format!(
                                                "{bare}::{variant_name}.{idx} (bound as `{s}`): \
                                                 UNVERIFIED: `{s}` is bound but no occurrence \
                                                 matches a recognised hashing shape (direct \
                                                 feed, summariser, iteration, cast, or match) \
                                                 -- its value may never reach the hasher \
                                                 (PB-DX7 review H2)"
                                            ));
                                        }
                                    }
                                }
                            }
                        }
                    }
                }
                VariantKind::Named(fields) => {
                    match payload.strip_prefix('{').and_then(|s| s.strip_suffix('}')) {
                        None => violations.push(format!(
                            "{bare}::{variant_name}: declared as a struct-shaped variant but the \
                             arm pattern has no `{{...}}` payload (`{payload}`)"
                        )),
                        Some(inner) => {
                            let segs: Vec<String> = split_depth0_commas(inner)
                                .into_iter()
                                .map(|s| s.trim().to_string())
                                .filter(|s| !s.is_empty())
                                .collect();
                            if segs.iter().any(|s| s == "..") {
                                violations.push(format!(
                                    "{bare}::{variant_name}: struct pattern contains a `..` rest \
                                     pattern -- a future field would silently fall outside the \
                                     binding"
                                ));
                            } else {
                                let bound: BTreeMap<String, String> = segs
                                    .iter()
                                    .map(|s| match s.split_once(':') {
                                        Some((f, bnd)) => {
                                            (f.trim().to_string(), bnd.trim().to_string())
                                        }
                                        None => (s.clone(), s.clone()),
                                    })
                                    .collect();
                                for field in fields {
                                    variant_fields_checked += 1;
                                    let Some(binding) = bound.get(field) else {
                                        violations.push(format!(
                                            "{bare}::{variant_name}.{field}: declared field is \
                                             not bound by this arm's pattern at all"
                                        ));
                                        continue;
                                    };
                                    // PB-DX7 review M4: mirror the Tuple branch's `_`
                                    // rejection. `{ field: _ }` must not classify as covered
                                    // just because a standalone `_` token happens to appear
                                    // somewhere else in the arm body.
                                    if binding == "_" {
                                        violations.push(format!(
                                            "{bare}::{variant_name}.{field}: named field bound \
                                             to `_` -- its value is discarded, never fed to the \
                                             hasher"
                                        ));
                                        continue;
                                    }
                                    let key =
                                        (bare.as_str(), variant_name.as_str(), field.as_str());
                                    match variant_field_coverage(arm_body, binding) {
                                        FieldCoverage::Full => {}
                                        FieldCoverage::NotReferenced => {
                                            if !allow.contains(&key) {
                                                violations.push(format!(
                                                    "{bare}::{variant_name}.{field} (bound as \
                                                     `{binding}`): declared but never referenced \
                                                     in the arm body"
                                                ));
                                            }
                                        }
                                        FieldCoverage::Partial(summarisers) => {
                                            if !partial_allow.contains(&key) {
                                                let methods = summarisers
                                                    .into_iter()
                                                    .collect::<Vec<_>>()
                                                    .join(", ");
                                                violations.push(format!(
                                                    "{bare}::{variant_name}.{field} (bound as \
                                                     `{binding}`): PARTIAL coverage only: every \
                                                     occurrence is `{binding}.{{{methods}}}(..)\
                                                     .hash_into(..)`, discarding the field's \
                                                     actual value, and it is not on \
                                                     PARTIALLY_HASHED_VARIANT_FIELDS"
                                                ));
                                            }
                                        }
                                        FieldCoverage::Unverified => {
                                            violations.push(format!(
                                                "{bare}::{variant_name}.{field} (bound as \
                                                 `{binding}`): UNVERIFIED: `{binding}` is bound \
                                                 but no occurrence matches a recognised hashing \
                                                 shape (direct feed, summariser, iteration, \
                                                 cast, or match) -- its value may never reach \
                                                 the hasher (PB-DX7 review H2)"
                                            ));
                                        }
                                    }
                                }
                            }
                        }
                    }
                }
            }
        }

        for v in variants {
            if !arm_seen.contains(&v.name) {
                violations.push(format!(
                    "{bare}::{}: declared variant has no arm in this enum's HashInto match",
                    v.name
                ));
            }
        }
    }

    assert!(
        violations.is_empty(),
        "\n\nThese enum variant fields are declared but never fed to their enum's `HashInto` \
         impl, are missing an arm entirely, or are covered only by a catch-all, and are not on \
         the NOT_HASHED_VARIANT_FIELDS allowlist:\n  {}\n\n\
         Two independent game states differing only in such a field hash IDENTICALLY (OOS-DP9-13). \
         For each: either feed `<binding>.hash_into(hasher)` in the matching arm (and bump \
         HASH_SCHEMA_VERSION per the state/hash.rs checklist), or, if the field is genuinely not \
         game state, add it to NOT_HASHED_VARIANT_FIELDS with a one-line rationale.\n",
        violations.join("\n  ")
    );

    assert!(
        enums_checked >= MIN_HASHED_ENUMS,
        "only {enums_checked} hashed enums were checked (expected >= {MIN_HASHED_ENUMS}); the \
         enum/impl intersection is empty or nearly so -- a scanner broke"
    );
    assert!(
        variants_checked >= MIN_VARIANTS_CHECKED,
        "only {variants_checked} variants were checked (expected >= {MIN_VARIANTS_CHECKED}); \
         named_enum_variants is under-counting"
    );
    assert!(
        variant_fields_checked >= MIN_VARIANT_FIELDS_CHECKED,
        "only {variant_fields_checked} variant fields were checked (expected >= \
         {MIN_VARIANT_FIELDS_CHECKED}); the arm parser is under-counting"
    );
}

/// The `NOT_HASHED_VARIANT_FIELDS` allowlist is honest: every entry names a real
/// declared variant field of a hashed enum that is genuinely absent from that
/// variant's arm body. Mirrors `not_hashed_allowlist_has_no_dead_entries` for the
/// struct half.
#[test]
fn not_hashed_variant_fields_allowlist_has_no_dead_entries() {
    let bodies = hashinto_impl_bodies();
    let enum_variants = named_enum_variants();

    for (ty, variant, field, _reason) in NOT_HASHED_VARIANT_FIELDS {
        let field: &str = field;
        let variants = enum_variants.get(*ty).unwrap_or_else(|| {
            panic!(
                "NOT_HASHED_VARIANT_FIELDS entry ({ty}, {variant}, {field}): `{ty}` is not a \
                 declared enum under the scan roots"
            )
        });
        let decl_variant = variants
            .iter()
            .find(|v| v.name == *variant)
            .unwrap_or_else(|| {
                panic!(
                "NOT_HASHED_VARIANT_FIELDS entry ({ty}, {variant}, {field}): `{ty}` declares no \
                 variant named `{variant}` (dead entry)"
            )
            });
        let field_exists = match &decl_variant.kind {
            VariantKind::Unit => false,
            VariantKind::Tuple(n) => field.parse::<usize>().is_ok_and(|i| i < *n),
            VariantKind::Named(fields) => fields.iter().any(|f| f == field),
        };
        assert!(
            field_exists,
            "NOT_HASHED_VARIANT_FIELDS entry ({ty}, {variant}, {field}): `{ty}::{variant}` \
             declares no such field (dead entry -- remove it or fix the name/index)"
        );

        let impl_body = bodies.get(*ty).unwrap_or_else(|| {
            panic!(
                "NOT_HASHED_VARIANT_FIELDS entry ({ty}, {variant}, {field}): `{ty}` has no \
                 `impl HashInto for {ty}`"
            )
        });
        let match_body = top_level_match_self_body(&impl_body.body).unwrap_or_else(|| {
            panic!(
                "NOT_HASHED_VARIANT_FIELDS entry ({ty}, {variant}, {field}): `{ty}`'s HashInto \
                 has no `match self` -- cannot be a dead entry for a data-carrying variant"
            )
        });
        let arms = parse_match_arms(match_body);
        let arm = arms.iter().find(|(pattern, _)| {
            let (name, _payload) = split_pattern(pattern.trim());
            name == *variant
        });
        let Some((pattern, arm_body)) = arm else {
            panic!(
                "NOT_HASHED_VARIANT_FIELDS entry ({ty}, {variant}, {field}): no arm for \
                 `{ty}::{variant}` was found -- dead entry"
            )
        };
        let (_, payload) = split_pattern(pattern.trim());
        // Resolve the entry's declared FIELD (name, or tuple index) to the actual
        // LOCAL BINDING that arm's pattern uses -- for a tuple variant that is
        // positional (field "0" is whatever identifier sits at index 0 in the
        // pattern's `(...)`), not the index string itself.
        let binding = match &decl_variant.kind {
            VariantKind::Unit => unreachable!("field_exists is false for Unit above"),
            VariantKind::Tuple(_) => {
                let idx: usize = field.parse().expect("field_exists validated this parses");
                let inner = payload
                    .strip_prefix('(')
                    .and_then(|s| s.strip_suffix(')'))
                    .unwrap_or_else(|| {
                        panic!(
                            "NOT_HASHED_VARIANT_FIELDS entry ({ty}, {variant}, {field}): arm \
                             pattern `{pattern}` has no `(...)` payload"
                        )
                    });
                split_depth0_commas(inner)
                    .get(idx)
                    .map(|s| s.trim().to_string())
                    .unwrap_or_else(|| {
                        panic!(
                            "NOT_HASHED_VARIANT_FIELDS entry ({ty}, {variant}, {field}): arm \
                             pattern `{pattern}` binds fewer than {} tuple field(s)",
                            idx + 1
                        )
                    })
            }
            VariantKind::Named(_) => {
                let inner = payload
                    .strip_prefix('{')
                    .and_then(|s| s.strip_suffix('}'))
                    .unwrap_or_else(|| {
                        panic!(
                            "NOT_HASHED_VARIANT_FIELDS entry ({ty}, {variant}, {field}): arm \
                             pattern `{pattern}` has no `{{...}}` payload"
                        )
                    });
                split_depth0_commas(inner)
                    .iter()
                    .find_map(|s| match s.trim().split_once(':') {
                        Some((f, b)) if f.trim() == field => Some(b.trim().to_string()),
                        None if s.trim() == field => Some(s.trim().to_string()),
                        _ => None,
                    })
                    .unwrap_or_else(|| {
                        panic!(
                            "NOT_HASHED_VARIANT_FIELDS entry ({ty}, {variant}, {field}): arm \
                             pattern `{pattern}` does not bind field `{field}` at all -- dead \
                             entry"
                        )
                    })
            }
        };
        assert!(
            !body_references_token(arm_body, &binding),
            "NOT_HASHED_VARIANT_FIELDS entry ({ty}, {variant}, {field}): `{ty}::{variant}`'s arm \
             DOES reference `{binding}` -- remove it from the allowlist (dead entry)."
        );
    }
}

/// The `PARTIALLY_HASHED_VARIANT_FIELDS` allowlist is honest: every entry
/// names a real declared variant field whose coverage is GENUINELY `Partial`
/// (every occurrence a summariser chained to `.hash_into`) — not `Full` (the
/// binding got fully hashed and the entry is now stale, e.g. exactly the
/// `ForecastAbility.embedded_effect` asymmetry this allowlist exists to make
/// visible) and not `NotReferenced` (that is `NOT_HASHED_VARIANT_FIELDS`'s
/// territory). Mirrors `partially_hashed_allowlist_has_no_dead_entries` for
/// the struct half.
#[test]
fn partially_hashed_variant_fields_allowlist_has_no_dead_entries() {
    let bodies = hashinto_impl_bodies();
    let enum_variants = named_enum_variants();

    for (ty, variant, field, _reason) in PARTIALLY_HASHED_VARIANT_FIELDS {
        let field: &str = field;
        let variants = enum_variants.get(*ty).unwrap_or_else(|| {
            panic!(
                "PARTIALLY_HASHED_VARIANT_FIELDS entry ({ty}, {variant}, {field}): `{ty}` is not \
                 a declared enum under the scan roots"
            )
        });
        let decl_variant = variants
            .iter()
            .find(|v| v.name == *variant)
            .unwrap_or_else(|| {
                panic!(
                    "PARTIALLY_HASHED_VARIANT_FIELDS entry ({ty}, {variant}, {field}): `{ty}` \
                     declares no variant named `{variant}` (dead entry)"
                )
            });
        let field_exists = match &decl_variant.kind {
            VariantKind::Unit => false,
            VariantKind::Tuple(n) => field.parse::<usize>().is_ok_and(|i| i < *n),
            VariantKind::Named(fields) => fields.iter().any(|f| f == field),
        };
        assert!(
            field_exists,
            "PARTIALLY_HASHED_VARIANT_FIELDS entry ({ty}, {variant}, {field}): `{ty}::{variant}` \
             declares no such field (dead entry — remove it or fix the name/index)"
        );

        let impl_body = bodies.get(*ty).unwrap_or_else(|| {
            panic!(
                "PARTIALLY_HASHED_VARIANT_FIELDS entry ({ty}, {variant}, {field}): `{ty}` has no \
                 `impl HashInto for {ty}`"
            )
        });
        let match_body = top_level_match_self_body(&impl_body.body).unwrap_or_else(|| {
            panic!(
                "PARTIALLY_HASHED_VARIANT_FIELDS entry ({ty}, {variant}, {field}): `{ty}`'s \
                 HashInto has no `match self` -- cannot be a dead entry for a data-carrying \
                 variant"
            )
        });
        let arms = parse_match_arms(match_body);
        let arm = arms.iter().find(|(pattern, _)| {
            let (name, _payload) = split_pattern(pattern.trim());
            name == *variant
        });
        let Some((pattern, arm_body)) = arm else {
            panic!(
                "PARTIALLY_HASHED_VARIANT_FIELDS entry ({ty}, {variant}, {field}): no arm for \
                 `{ty}::{variant}` was found -- dead entry"
            )
        };
        let (_, payload) = split_pattern(pattern.trim());
        let binding = match &decl_variant.kind {
            VariantKind::Unit => unreachable!("field_exists is false for Unit above"),
            VariantKind::Tuple(_) => {
                let idx: usize = field.parse().expect("field_exists validated this parses");
                let inner = payload
                    .strip_prefix('(')
                    .and_then(|s| s.strip_suffix(')'))
                    .unwrap_or_else(|| {
                        panic!(
                            "PARTIALLY_HASHED_VARIANT_FIELDS entry ({ty}, {variant}, {field}): \
                             arm pattern `{pattern}` has no `(...)` payload"
                        )
                    });
                split_depth0_commas(inner)
                    .get(idx)
                    .map(|s| s.trim().to_string())
                    .unwrap_or_else(|| {
                        panic!(
                            "PARTIALLY_HASHED_VARIANT_FIELDS entry ({ty}, {variant}, {field}): \
                             arm pattern `{pattern}` binds fewer than {} tuple field(s)",
                            idx + 1
                        )
                    })
            }
            VariantKind::Named(_) => {
                let inner = payload
                    .strip_prefix('{')
                    .and_then(|s| s.strip_suffix('}'))
                    .unwrap_or_else(|| {
                        panic!(
                            "PARTIALLY_HASHED_VARIANT_FIELDS entry ({ty}, {variant}, {field}): \
                             arm pattern `{pattern}` has no `{{...}}` payload"
                        )
                    });
                split_depth0_commas(inner)
                    .iter()
                    .find_map(|s| match s.trim().split_once(':') {
                        Some((f, b)) if f.trim() == field => Some(b.trim().to_string()),
                        None if s.trim() == field => Some(s.trim().to_string()),
                        _ => None,
                    })
                    .unwrap_or_else(|| {
                        panic!(
                            "PARTIALLY_HASHED_VARIANT_FIELDS entry ({ty}, {variant}, {field}): \
                             arm pattern `{pattern}` does not bind field `{field}` at all -- \
                             dead entry"
                        )
                    })
            }
        };
        match variant_field_coverage(arm_body, &binding) {
            FieldCoverage::Partial(_) => {} // legitimate
            FieldCoverage::Full => panic!(
                "PARTIALLY_HASHED_VARIANT_FIELDS entry ({ty}, {variant}, {field}): \
                 `{ty}::{variant}`'s `{binding}` is now FULLY hashed — remove this entry (dead)."
            ),
            FieldCoverage::NotReferenced => panic!(
                "PARTIALLY_HASHED_VARIANT_FIELDS entry ({ty}, {variant}, {field}): \
                 `{ty}::{variant}`'s `{binding}` is not referenced at all any more — this is \
                 NOT_HASHED_VARIANT_FIELDS's territory now, not PARTIALLY_HASHED_VARIANT_FIELDS's \
                 (dead entry)."
            ),
            FieldCoverage::Unverified => panic!(
                "PARTIALLY_HASHED_VARIANT_FIELDS entry ({ty}, {variant}, {field}): \
                 `{ty}::{variant}`'s `{binding}` no longer classifies as a recognised \
                 summariser shape (Unverified) — re-derive the entry's status, do not assume \
                 it is still Partial."
            ),
        }
    }
}

/// Non-vacuity + positive/negative controls for the Part B scanners
/// (`named_enum_variants`, `parse_match_arms`, `split_pattern`,
/// `body_references_token`), in the style of `coverage_scanners_are_not_vacuous`.
#[test]
fn enum_coverage_scanners_are_not_vacuous() {
    let bodies = hashinto_impl_bodies();
    let enum_variants = named_enum_variants();
    assert!(
        enum_variants.len() >= MIN_DECLARED_ENUMS,
        "found only {} declared enums; named_enum_variants is broken (expected >= {})",
        enum_variants.len(),
        MIN_DECLARED_ENUMS
    );

    let ctv = enum_variants
        .get("TriggerCondition")
        .expect("TriggerCondition is declared");
    assert!(
        !ctv.is_empty(),
        "TriggerCondition was parsed with zero variants -- named_enum_variants is broken"
    );
    assert!(
        ctv.iter().any(|v| v.kind == VariantKind::Unit),
        "TriggerCondition has no Unit variant in the parsed set -- expected at least one"
    );
    assert!(
        ctv.iter()
            .any(|v| matches!(&v.kind, VariantKind::Named(fields) if !fields.is_empty())),
        "TriggerCondition has no Named variant with fields -- expected at least one"
    );

    // classify_variant_segment controls, isolated from real source.
    let unit = classify_variant_segment("Foo").expect("parses");
    assert_eq!(unit.name, "Foo");
    assert_eq!(unit.kind, VariantKind::Unit);
    let tup = classify_variant_segment("Foo(u32, PlayerId)").expect("parses");
    assert_eq!(tup.name, "Foo");
    assert_eq!(tup.kind, VariantKind::Tuple(2));
    let named = classify_variant_segment("Foo { x: u32, y: PlayerId }").expect("parses");
    assert_eq!(named.name, "Foo");
    assert_eq!(
        named.kind,
        VariantKind::Named(vec!["x".to_string(), "y".to_string()])
    );

    // split_pattern controls.
    assert_eq!(
        split_pattern("Foo::Bar"),
        ("Bar".to_string(), String::new())
    );
    assert_eq!(
        split_pattern("Foo::Bar(x, y)"),
        ("Bar".to_string(), "(x, y)".to_string())
    );
    assert_eq!(
        split_pattern("Foo::Bar { x, y }"),
        ("Bar".to_string(), "{ x, y }".to_string())
    );

    // parse_match_arms controls: one block-bodied arm (with an internal comma,
    // the exact shape a naive top-level comma split would misparse as two arms)
    // and one expression-bodied arm.
    let arms = parse_match_arms(
        "Foo::A { x, y } => { x.hash_into(hasher); y.hash_into(hasher); } Foo::B => 1u8.hash_into(hasher),",
    );
    assert_eq!(arms.len(), 2, "expected exactly 2 arms, got {arms:?}");
    assert_eq!(arms[0].0.trim(), "Foo::A { x, y }");
    assert!(arms[0].1.contains("x.hash_into(hasher)") && arms[0].1.contains("y.hash_into(hasher)"));
    assert_eq!(arms[1].0.trim(), "Foo::B");
    assert!(arms[1].1.contains("1u8.hash_into(hasher)"));

    // body_references_token controls (no `self.` prefix requirement, unlike the
    // struct half's body_references_field).
    assert!(body_references_token("filter.hash_into(hasher);", "filter"));
    assert!(!body_references_token("filter.hash_into(hasher);", "filte"));
    assert!(!body_references_token(
        "filter_kind.hash_into(hasher);",
        "filter"
    ));

    // variant_field_coverage controls, isolated from real source.
    assert_eq!(
        variant_field_coverage(
            "embedded_effect.is_some().hash_into(hasher);",
            "embedded_effect"
        ),
        FieldCoverage::Partial(["is_some".to_string()].into_iter().collect()),
        "positive control failed: a lone summariser-then-hash_into is Partial"
    );
    assert_eq!(
        variant_field_coverage("embedded_effect.hash_into(hasher);", "embedded_effect"),
        FieldCoverage::Full,
        "positive control failed: a bare feed is Full"
    );
    assert_eq!(
        variant_field_coverage("other.hash_into(hasher);", "embedded_effect"),
        FieldCoverage::NotReferenced,
        "negative control failed: `embedded_effect` does not appear at all"
    );

    // The two real PARTIALLY_HASHED_VARIANT_FIELDS entries are genuinely
    // Partial in their REAL arm bodies, not Full.
    let stack_object_kind = &bodies
        .get("StackObjectKind")
        .expect("StackObjectKind impl")
        .body;
    let match_body =
        top_level_match_self_body(stack_object_kind).expect("StackObjectKind has match self");
    for (_, variant, field, _reason) in PARTIALLY_HASHED_VARIANT_FIELDS {
        let arm = parse_match_arms(match_body)
            .into_iter()
            .find(|(pattern, _)| split_pattern(pattern.trim()).0 == *variant)
            .unwrap_or_else(|| panic!("no arm found for StackObjectKind::{variant}"));
        assert_eq!(
            variant_field_coverage(&arm.1, field),
            FieldCoverage::Partial(["is_some".to_string()].into_iter().collect()),
            "StackObjectKind::{variant}.{field} must classify as Partial \
             ({field}.is_some())"
        );
    }

    // The documented asymmetry: ForecastAbility's sibling field of the SAME
    // name hashes FULLY, and must NOT be Partial (this is what makes the
    // allowlist's absence of an entry for it meaningful rather than an
    // oversight).
    let forecast_arm = parse_match_arms(match_body)
        .into_iter()
        .find(|(pattern, _)| split_pattern(pattern.trim()).0 == "ForecastAbility")
        .expect("ForecastAbility arm exists");
    assert_eq!(
        variant_field_coverage(&forecast_arm.1, "embedded_effect"),
        FieldCoverage::Full,
        "StackObjectKind::ForecastAbility.embedded_effect must classify as Full -- the \
         asymmetry PARTIALLY_HASHED_VARIANT_FIELDS's absence of an entry for it makes visible"
    );
}

// ════════════════════════════════════════════════════════════════════════════
// PB-DX7 follow-up (2026-08-11, coordinator-directed): discriminant collisions
// ════════════════════════════════════════════════════════════════════════════
//
// §3 of the batch's spec asked "does every arm of every hashed enum hash a
// distinct discriminant literal?" and to gate it only if clean. It is NOT
// clean: `Effect`'s `HashInto` impl reuses 9 discriminant values across 18
// different variants (all other 78 hashed enums are clean). The shape strongly
// suggests two historically-separate discriminant-numbering sequences merged
// into one enum without reconciling ranges -- several `HASH_SCHEMA_HISTORY`
// entries cite "discriminant N" as if unique per enum, and for these 9 values
// it is not, regardless of whether the digests below happen to collide.
//
// A collision in the discriminant BYTE does not, by itself, prove two
// DIFFERENT `Effect` values can hash IDENTICALLY -- the subsequent field bytes
// almost always differ. But "almost always" is an argument, not a
// measurement, and this codebase's own history (`OOS-SIM2-6`) sat behind
// exactly that kind of plausible-sounding termination claim for 4.5 months.
// So it is settled by experiment below, not asserted.

/// The first bare integer literal in `s` (skipping over string literals),
/// read as an arm's hashed discriminant byte.
///
/// **PB-DX7 review L7 fix (2026-08-12).** The ORIGINAL version read the first
/// digit run ANYWHERE, including one embedded in an identifier or type name
/// (`u32`, `i64`, a local like `x2`) — so an arm that casts BEFORE its tag
/// (`(count as u32).hash_into(hasher); 56u8.hash_into(hasher);`) would have
/// been measured as tag `32`, not `56`. Latent, not live today (every real
/// `Effect` arm opens with its tag), but the fix does not require a `uN`
/// SUFFIX on the digit run — that would break the legitimate indirect-
/// discriminant idiom (`AltCostKind`/`DungeonId`'s `let disc: u8 = match
/// self { A => 0, B => 1, ... };`, whose arm bodies are BARE literals with
/// no suffix at all). Instead: a digit run immediately preceded by an
/// identifier character (a letter or `_`) is embedded in something else
/// (`u32`, `x2`) and is skipped, not returned — the scan continues past it
/// for the next digit run.
fn first_integer_literal(s: &str) -> Option<u64> {
    let b = s.as_bytes();
    let n = b.len();
    let mut i = 0;
    while i < n {
        if let Some(len) = literal_len(b, i) {
            i += len;
            continue;
        }
        if b[i].is_ascii_digit() {
            let start = i;
            let embedded_in_ident =
                start > 0 && (b[start - 1].is_ascii_alphabetic() || b[start - 1] == b'_');
            while i < n && b[i].is_ascii_digit() {
                i += 1;
            }
            if embedded_in_ident {
                continue;
            }
            return s[start..i].parse::<u64>().ok();
        }
        i += 1;
    }
    None
}

/// For one hashed enum's `HashInto` impl body, every discriminant value shared
/// by more than one variant (empty if the enum has no `match self`, or every
/// discriminant is unique), AND every named variant whose arm has NO integer
/// literal at all.
///
/// **PB-DX7 review M3 fix (2026-08-11)**: the ORIGINAL version silently
/// dropped the no-literal case (`first_integer_literal` returning `None` just
/// skipped the arm), so `GiftType::Food => {}` was invisible to this scan —
/// not merely "unique", genuinely UNSEEN, and a second empty arm
/// (`GiftType::Card => {}`) would have hashed identically with this ratchet
/// staying green throughout. Every arm with no literal is now a reportable
/// finding here, independent of the collision check (which needs at least
/// two literals to compare) — see `discriminant_collisions_are_ratcheted_at_
/// their_known_bad_state`'s companion assertion.
fn enum_discriminant_collisions(impl_body: &str) -> (BTreeMap<u64, Vec<String>>, Vec<String>) {
    let mut by_disc: BTreeMap<u64, Vec<String>> = BTreeMap::new();
    let mut no_literal: Vec<String> = Vec::new();
    let Some(match_body) = top_level_match_self_body(impl_body) else {
        return (by_disc, no_literal);
    };
    for (pattern, arm_body) in parse_match_arms(match_body) {
        let (variant_name, _payload) = split_pattern(pattern.trim());
        if variant_name.is_empty() {
            continue; // catch-all or unparseable; not this scan's concern
        }
        match first_integer_literal(&arm_body) {
            Some(disc) => by_disc.entry(disc).or_default().push(variant_name),
            None => no_literal.push(variant_name),
        }
    }
    by_disc.retain(|_, variants| variants.len() > 1);
    (by_disc, no_literal)
}

/// The pinned, known-bad state: every `(discriminant, variant_a, variant_b)`
/// pair `Effect`'s `HashInto` impl currently reuses. `Effect` is the ONLY
/// enum permitted any entry here — the whole point of a ratchet on a known-bad
/// state is that it is small and named, not a blanket exemption.
const KNOWN_EFFECT_DISCRIMINANT_COLLISIONS: &[(u64, &str, &str)] = &[
    (56, "AddCounterAmount", "AddManaScaled"),
    (57, "AddManaRestricted", "AdditionalCombatPhase"),
    (58, "AddManaAnyColorRestricted", "Fight"),
    (59, "Bite", "ChooseCreatureType"),
    (60, "AddManaOfAnyColorAmount", "CoinFlip"),
    (70, "ExileWithDelayedReturn", "PreventCombatDamageFromOrTo"),
    (71, "GainControl", "SetReturnToHandAtEndStep"),
    (73, "AddManaFilterChoice", "GrantPlayerProtection"),
    (74, "BounceAll", "PutLandFromHandOntoBattlefield"),
];

/// **Ratchet, not a "no collisions" gate** — asserting "every hashed enum's
/// discriminants are unique" would redden today on the 9 pre-existing `Effect`
/// pairs above. This pins today's known-bad state exactly and rejects any
/// movement: fixing one of the 9 (renumbering a discriminant so it's unique
/// again) requires updating this table in the SAME commit as the fix — a
/// reviewed, deliberate edit, not a silent side effect — and a NEW collision
/// anywhere, in `Effect` or any of the other 78 hashed enums, fails
/// immediately instead of silently joining the pile.
#[test]
fn discriminant_collisions_are_ratcheted_at_their_known_bad_state() {
    let bodies = hashinto_impl_bodies();
    let decls = index_declarations();

    let effect_body = &bodies.get("Effect").expect("Effect impl").body;
    let (effect_collisions, effect_no_literal) = enum_discriminant_collisions(effect_body);
    assert!(
        effect_no_literal.is_empty(),
        "\n\nThese Effect variant arms have NO integer literal at all (never seen by the \
         collision check above, which needs two literals to compare) -- feed the hasher \
         something in the arm, even a bare discriminant byte:\n  {}\n",
        effect_no_literal.join("\n  ")
    );
    let mut found_pairs: BTreeSet<(u64, String, String)> = BTreeSet::new();
    for (disc, variants) in effect_collisions {
        for i in 0..variants.len() {
            for j in (i + 1)..variants.len() {
                let (a, b) = if variants[i] <= variants[j] {
                    (variants[i].clone(), variants[j].clone())
                } else {
                    (variants[j].clone(), variants[i].clone())
                };
                found_pairs.insert((disc, a, b));
            }
        }
    }
    let pinned_pairs: BTreeSet<(u64, String, String)> = KNOWN_EFFECT_DISCRIMINANT_COLLISIONS
        .iter()
        .map(|(d, a, b)| (*d, a.to_string(), b.to_string()))
        .collect();
    assert_eq!(
        found_pairs, pinned_pairs,
        "\n\nEffect's discriminant collisions moved from the pinned KNOWN_EFFECT_DISCRIMINANT_\
         COLLISIONS set.\nIf a pair was FIXED (a discriminant renumbered so it's unique again), \
         remove its row here -- that is progress, re-pin it.\nIf a NEW pair appeared, that is a \
         fresh discriminant collision and must be fixed in hash.rs (renumber one of the two \
         arms), not pinned -- this ratchet exists to catch exactly that, not to grow.\n"
    );

    // No OTHER hashed enum may have ANY collision at all -- Effect is the
    // named, pinned exception, not the norm. AND no arm of ANY hashed enum
    // (Effect included -- checked above) may feed zero bytes.
    let mut unexpected: Vec<String> = Vec::new();
    let mut no_literal_elsewhere: Vec<String> = Vec::new();
    for (bare, impl_body) in &bodies {
        let Some(decl) = decls.index.get(bare) else {
            continue;
        };
        if !decl_is_enum(decl) {
            continue;
        }
        let (collisions, no_literal) = enum_discriminant_collisions(&impl_body.body);
        if bare != "Effect" {
            for (disc, variants) in collisions {
                unexpected.push(format!("{bare} discriminant {disc}: {variants:?}"));
            }
        }
        for v in no_literal {
            no_literal_elsewhere.push(format!("{bare}::{v}"));
        }
    }
    assert!(
        unexpected.is_empty(),
        "\n\nThese hashed enums have a NEW discriminant collision, outside the pinned `Effect` \
         exception above:\n  {}\n",
        unexpected.join("\n  ")
    );
    assert!(
        no_literal_elsewhere.is_empty(),
        "\n\nThese enum variant arms have NO integer literal at all -- two such arms on the \
         same enum feed zero bytes each and hash identically:\n  {}\n",
        no_literal_elsewhere.join("\n  ")
    );
}

/// **The experiment §3 asked for.** Builds one plausible (not deliberately
/// distinguishing) value of each of the 18 variants `Effect` shares a
/// discriminant with another variant, hashes each through the REAL `HashInto`
/// impl, and asserts all 18 resulting digests are pairwise distinct.
///
/// **What this proves and what it does not.** Distinct digests for these 18
/// SAMPLED values is evidence that the discriminant collisions are not
/// currently causing an observed hash collision — it is NOT a proof of
/// injectivity over the whole variant space (a different set of field values
/// for the same 18 variants could, in principle, coincide; this experiment
/// samples one point each, it does not enumerate). Read a green result here as
/// "no live blind spot found in this sample", not as "collisions are
/// impossible".
#[test]
fn effect_colliding_variant_digests_are_pairwise_distinct() {
    let target = || CardEffectTarget::DeclaredTarget { index: 0 };
    let target2 = || CardEffectTarget::DeclaredTarget { index: 1 };

    let values: Vec<(&str, Effect)> = vec![
        (
            "AddManaScaled",
            Effect::AddManaScaled {
                player: PlayerTarget::Controller,
                color: ManaColor::White,
                count: EffectAmount::Fixed(1),
            },
        ),
        (
            "AddCounterAmount",
            Effect::AddCounterAmount {
                target: target(),
                counter: CounterType::PlusOnePlusOne,
                count: EffectAmount::Fixed(1),
            },
        ),
        (
            "AddManaRestricted",
            Effect::AddManaRestricted {
                player: PlayerTarget::Controller,
                mana: ManaPool::default(),
                restriction: ManaRestriction::CreatureSpellsOnly,
            },
        ),
        (
            "AdditionalCombatPhase",
            Effect::AdditionalCombatPhase {
                followed_by_main: false,
            },
        ),
        (
            "AddManaAnyColorRestricted",
            Effect::AddManaAnyColorRestricted {
                player: PlayerTarget::Controller,
                restriction: ManaRestriction::CreatureSpellsOnly,
            },
        ),
        (
            "Fight",
            Effect::Fight {
                attacker: target(),
                defender: target2(),
            },
        ),
        (
            "ChooseCreatureType",
            Effect::ChooseCreatureType {
                default: SubType("Human".to_string()),
            },
        ),
        (
            "Bite",
            Effect::Bite {
                source: target(),
                target: target2(),
            },
        ),
        (
            "AddManaOfAnyColorAmount",
            Effect::AddManaOfAnyColorAmount {
                player: PlayerTarget::Controller,
                amount: EffectAmount::Fixed(1),
            },
        ),
        (
            "CoinFlip",
            Effect::CoinFlip {
                on_win: Box::new(Effect::SetReturnToHandAtEndStep),
                on_lose: Box::new(Effect::SetReturnToHandAtEndStep),
            },
        ),
        (
            "ExileWithDelayedReturn",
            Effect::ExileWithDelayedReturn {
                target: target(),
                return_timing: DelayedTriggerTiming::AtNextEndStep,
                return_tapped: false,
                return_to: DelayedReturnDestination::Battlefield,
            },
        ),
        (
            "PreventCombatDamageFromOrTo",
            Effect::PreventCombatDamageFromOrTo {
                target: target(),
                prevent_from: true,
                prevent_to: false,
            },
        ),
        ("SetReturnToHandAtEndStep", Effect::SetReturnToHandAtEndStep),
        (
            "GainControl",
            Effect::GainControl {
                target: target(),
                duration: EffectDuration::UntilEndOfTurn,
            },
        ),
        (
            "AddManaFilterChoice",
            Effect::AddManaFilterChoice {
                player: PlayerTarget::Controller,
                color_a: ManaColor::White,
                color_b: ManaColor::Blue,
            },
        ),
        (
            "GrantPlayerProtection",
            Effect::GrantPlayerProtection {
                player: PlayerTarget::Controller,
                qualities: vec![ProtectionQuality::FromColor(Color::White)],
                duration: None,
            },
        ),
        (
            "BounceAll",
            Effect::BounceAll {
                filter: TargetFilter::default(),
                max_toughness_amount: None,
            },
        ),
        (
            "PutLandFromHandOntoBattlefield",
            Effect::PutLandFromHandOntoBattlefield { tapped: false },
        ),
    ];

    assert_eq!(
        values.len(),
        18,
        "this experiment must cover all 18 colliding variants named in \
         KNOWN_EFFECT_DISCRIMINANT_COLLISIONS -- non-vacuity floor"
    );

    let mut digests: Vec<(&str, [u8; 32])> = Vec::new();
    for (name, effect) in &values {
        let mut hasher = blake3::Hasher::new();
        effect.hash_into(&mut hasher);
        digests.push((name, *hasher.finalize().as_bytes()));
    }

    let mut collisions: Vec<String> = Vec::new();
    for i in 0..digests.len() {
        for j in (i + 1)..digests.len() {
            if digests[i].1 == digests[j].1 {
                collisions.push(format!(
                    "{} and {} hash IDENTICALLY: {}",
                    digests[i].0,
                    digests[j].0,
                    hex_bytes(&digests[i].1)
                ));
            }
        }
    }
    assert!(
        collisions.is_empty(),
        "\n\nSTOP -- these sampled Effect values hash IDENTICALLY despite being different \
         variants with different field values. This is a REAL divergence-detection blind spot, \
         not a documentation defect:\n  {}\n",
        collisions.join("\n  ")
    );
}

fn hex_bytes(b: &[u8; 32]) -> String {
    b.iter().map(|byte| format!("{byte:02x}")).collect()
}

// ════════════════════════════════════════════════════════════════════════════
// PB-DX7 follow-up (2026-08-11, coordinator-directed): GameState → public_state_hash
// ════════════════════════════════════════════════════════════════════════════
//
// The struct field-coverage gate's own module doc (see the SR-19 block near the
// top of this file) explicitly carves `GameState`'s `public_state_hash` /
// `private_state_hash` out of its scope: they select fields deliberately rather
// than hashing all of them, so "every field is hashed" would be the wrong rule.
// That reasoning is sound. What was missing is that NOTHING states which fields
// are excluded, and NOTHING gates the exclusion -- SR-17's `decl_fingerprint`
// forces a human to look when a 46th field appears, but its prompt is "a
// fingerprint moved", not "you added a field and did not feed it to the public
// hash". For the largest struct in the engine that prompt is too weak -- the
// same "the gate reports success while checking nothing" shape as the two seeds
// this batch closes.
//
// Deliberately does NOT apply the same rule to `private_state_hash`: that
// function is scoped to ONE PLAYER's hidden zones (hand, library, face-down) by
// design, so "every GameState field appears in it" is simply false on its own
// terms and a gate asserting it would be wrong, not strict.

/// Extract the body (text strictly between the outer braces) of
/// `pub fn public_state_hash` in `hash.rs`.
fn public_state_hash_body() -> String {
    let raw = std::fs::read_to_string(hash_rs_path()).expect("readable hash.rs");
    let src = strip_comments(&raw);
    let start = src
        .find("pub fn public_state_hash")
        .expect("public_state_hash exists in hash.rs");
    let b = src.as_bytes();
    let brace = src[start..]
        .find('{')
        .map(|p| start + p)
        .expect("public_state_hash has a body");
    let end = match_delim(b, brace, b'{', b'}');
    src[brace + 1..end - 1].to_string()
}

/// `(field, reason)` pairs for `GameState` fields deliberately NOT referenced
/// in `public_state_hash`. Each entry must name a real declared `GameState`
/// field that is genuinely absent from `public_state_hash`'s body
/// (`gamestate_not_in_public_hash_has_no_dead_entries` enforces both).
///
/// Every reason below was independently VERIFIED by dataflow on 2026-08-11,
/// not merely copied from the field's own doc comment -- one of the three
/// (`history`) turned out to need a different, more precise reason than the
/// obvious first guess (see its entry).
const GAMESTATE_NOT_IN_PUBLIC_HASH: &[(&str, &str)] = &[
    (
        "loop_detection_hashes",
        "CR 104.4b mandatory-loop-detection bookkeeping, not game state -- two \
         independent engine instances processing the SAME legal game may \
         accumulate DIFFERENT hash histories depending on when their \
         mandatory-action sequences began (PB-DX7 review L1 fix: this claim's \
         real citation is hash.rs:8290-8293, public_state_hash's own \
         'Excludes:' doc -- NOT state/mod.rs, which only says 'metadata used \
         by the loop-detection algorithm, not actual game state'), so \
         including it would produce FALSE mismatches between two \
         genuinely-agreeing states, not catch real ones.",
    ),
    (
        "history",
        "excluded for COST, not redundancy -- hash.rs's own doc on \
         public_state_hash states the reason plainly: \"Event history (O(n) in \
         game length)\". Verified by dataflow (2026-08-11): the ONLY reader of \
         GameState::history() anywhere in crates/engine/src, crates/card-types/\
         src, crates/simulator/src, crates/view-model/src, crates/engine/tests, \
         or tools/ is a single test assertion (rules/replacement_effects.rs); \
         zero rules-decision code reads it. So no rules-visible desync is \
         currently blind to its exclusion -- every existing look-back mechanic \
         (CR 603.10a LKI snapshots, etc.) captures its own dedicated field \
         instead of scanning history, and those dedicated fields ARE hashed. If \
         a future look-back trigger starts reading history() directly, this \
         entry's soundness must be re-examined, not assumed to still hold.",
    ),
    (
        "card_registry",
        "#[serde(skip)], reconstructed on load -- static card-definition data \
         identical for every instance of a given format's card pool, not \
         per-game state. Never differs between two game states that actually \
         diverged in play.",
    ),
];

/// `(field, reason)` triples for `GameState` fields referenced in
/// `public_state_hash` ONLY via a summarising method (the enum/struct-level
/// `PARTIALLY_HASHED` shape, one level up). **Empty today** — see
/// `every_gamestate_field_is_in_public_hash_or_allowlisted`'s doc for why
/// shipping empty here is itself a finding, not an oversight.
const PARTIALLY_HASHED_GAMESTATE: &[(&str, &str)] = &[];

/// Non-vacuity floor. `GameState` has 45 fields at implementation time;
/// deliberately well below that.
const MIN_GAMESTATE_FIELDS: usize = 30;

/// **PB-DX7 follow-up, review M6 fix (2026-08-11).** Every `GameState` field
/// is either FULLY fed in `public_state_hash`'s body, on
/// `GAMESTATE_NOT_IN_PUBLIC_HASH`, or on `PARTIALLY_HASHED_GAMESTATE`. A
/// field silently added and never fed to the public hash is invisible to
/// every consumer of that digest (distributed verification / desync
/// detection) — the same SR-7-shaped blind spot the struct/enum halves above
/// close, one level up (the whole-struct selection function, not a per-type
/// impl).
///
/// ORIGINALLY this called `body_references_field` — the exact matcher
/// `PARTIALLY_HASHED` was built to stop trusting, because it reads
/// `self.<field>` as covered on mere textual presence, regardless of what
/// follows. The very next field in the real body proves the risk was live,
/// not theoretical: `self.day_night` is fed via `match self.day_night {
/// None => 0u8..., Some(Day) => 1u8..., Some(Night) => 2u8... }` — a real
/// `token_match_body_hashes` shape, correctly `Full` under
/// `struct_field_coverage`, but `body_references_field` would have called
/// ANY shape (including a stripped-down `.is_some()` summary) equally
/// "covered". Switched to `struct_field_coverage`; verified by execution
/// (2026-08-11) that every one of the 42 currently-referenced fields still
/// classifies `Full` — zero regressions, and `PARTIALLY_HASHED_GAMESTATE`
/// ships EMPTY because none of them turned out to be summarised.
#[test]
fn every_gamestate_field_is_in_public_hash_or_allowlisted() {
    let structs = named_field_structs();
    let fields = structs
        .get("GameState")
        .expect("GameState is a named-field struct under the scan roots");
    let body = public_state_hash_body();
    let allow: BTreeSet<&str> = GAMESTATE_NOT_IN_PUBLIC_HASH
        .iter()
        .map(|(f, _reason)| *f)
        .collect();
    let partial_allow: BTreeSet<&str> = PARTIALLY_HASHED_GAMESTATE
        .iter()
        .map(|(f, _reason)| *f)
        .collect();

    let mut violations: Vec<String> = Vec::new();
    for f in fields {
        match struct_field_coverage(&body, f) {
            FieldCoverage::Full => {}
            FieldCoverage::NotReferenced => {
                if !allow.contains(f.as_str()) {
                    violations.push(format!("{f} -- never referenced at all"));
                }
            }
            FieldCoverage::Partial(summarisers) => {
                if !partial_allow.contains(f.as_str()) {
                    let methods = summarisers.into_iter().collect::<Vec<_>>().join(", ");
                    violations.push(format!(
                        "{f} -- PARTIAL coverage only: every occurrence is \
                         `self.{f}.{{{methods}}}(..).hash_into(..)`, discarding the field's \
                         actual value, and it is not on PARTIALLY_HASHED_GAMESTATE"
                    ));
                }
            }
            FieldCoverage::Unverified => {
                violations.push(format!(
                    "{f} -- UNVERIFIED: self.{f} is referenced in public_state_hash, but no \
                     occurrence matches a recognised hashing shape"
                ));
            }
        }
    }

    assert!(
        violations.is_empty(),
        "\n\nThese GameState fields are declared but never fully fed in public_state_hash, \
         and are not on GAMESTATE_NOT_IN_PUBLIC_HASH or PARTIALLY_HASHED_GAMESTATE:\n  {}\n\n\
         public_state_hash is the top-level divergence-detection digest for distributed \
         verification; a field silently absent from it is invisible to every consumer of \
         that digest. Either feed `self.<field>` into public_state_hash (bump \
         HASH_SCHEMA_VERSION per the state/hash.rs checklist), or, if the field is \
         genuinely not publicly-observable game state, add it to \
         GAMESTATE_NOT_IN_PUBLIC_HASH with a one-line rationale you have actually \
         verified, not merely copied from the field's own doc comment.\n",
        violations.join("\n  ")
    );

    assert!(
        fields.len() >= MIN_GAMESTATE_FIELDS,
        "GameState was parsed with only {} fields (expected >= {MIN_GAMESTATE_FIELDS}); the \
         struct scanner lost fields",
        fields.len()
    );
}

/// The `GAMESTATE_NOT_IN_PUBLIC_HASH` allowlist is honest: every entry names a
/// real declared `GameState` field that is genuinely absent from
/// `public_state_hash`'s body. Mirrors the other three allowlist dead-entry
/// guards in this file (`NOT_HASHED`, `PARTIALLY_HASHED`,
/// `NOT_HASHED_VARIANT_FIELDS`).
#[test]
fn gamestate_not_in_public_hash_has_no_dead_entries() {
    let structs = named_field_structs();
    let fields = structs
        .get("GameState")
        .expect("GameState is a named-field struct under the scan roots");
    let body = public_state_hash_body();

    for (field, _reason) in GAMESTATE_NOT_IN_PUBLIC_HASH {
        assert!(
            fields.iter().any(|f| f == field),
            "GAMESTATE_NOT_IN_PUBLIC_HASH entry ({field}): GameState declares no field \
             named `{field}` (dead entry — remove it or fix the name)"
        );
        assert!(
            !body_references_field(&body, field),
            "GAMESTATE_NOT_IN_PUBLIC_HASH entry ({field}): public_state_hash DOES \
             reference `{field}` — remove it from the allowlist (dead entry)."
        );
    }
}

/// The `PARTIALLY_HASHED_GAMESTATE` allowlist is honest: every entry names a
/// real declared `GameState` field whose coverage is GENUINELY `Partial` in
/// `public_state_hash`. Mirrors `partially_hashed_allowlist_has_no_dead_entries`.
/// Vacuous today (the constant is empty) but present for the day it is not.
#[test]
fn partially_hashed_gamestate_has_no_dead_entries() {
    let structs = named_field_structs();
    let fields = structs
        .get("GameState")
        .expect("GameState is a named-field struct under the scan roots");
    let body = public_state_hash_body();

    for (field, _reason) in PARTIALLY_HASHED_GAMESTATE {
        assert!(
            fields.iter().any(|f| f == field),
            "PARTIALLY_HASHED_GAMESTATE entry ({field}): GameState declares no field named \
             `{field}` (dead entry — remove it or fix the name)"
        );
        match struct_field_coverage(&body, field) {
            FieldCoverage::Partial(_) => {} // legitimate
            FieldCoverage::Full => panic!(
                "PARTIALLY_HASHED_GAMESTATE entry ({field}): `{field}` is now FULLY hashed — \
                 remove this entry (dead)."
            ),
            FieldCoverage::NotReferenced => panic!(
                "PARTIALLY_HASHED_GAMESTATE entry ({field}): `{field}` is not referenced at \
                 all — this is GAMESTATE_NOT_IN_PUBLIC_HASH's territory, not \
                 PARTIALLY_HASHED_GAMESTATE's (dead entry)."
            ),
            FieldCoverage::Unverified => panic!(
                "PARTIALLY_HASHED_GAMESTATE entry ({field}): `{field}` no longer classifies \
                 as a recognised summariser shape (Unverified) — re-derive the entry's status."
            ),
        }
    }
}

/// `(collection_field, element_type)` pairs where `public_state_hash` hashes a
/// `Vector`/`OrdMap` collection's elements FIELD-BY-FIELD inline, rather than
/// delegating to the element type's own `HashInto` impl — because it doesn't
/// have one. Every entry's `element_type` must be a real named-field struct
/// under the scan roots with NO `impl HashInto` (if it later gains one, the
/// hand-hashing should be replaced by a single `.hash_into(&mut hasher)` call
/// and this entry removed).
///
/// **PB-DX7 review M5 fix (2026-08-11).** `every_gamestate_field_is_in_public_
/// hash_or_allowlisted` only checks that `self.<field>` is REFERENCED — it
/// cannot see one level deeper, into the hand-written per-element hashing
/// loop, to notice that a NEW field on the element struct was never added to
/// the loop body. That is the same "silent field-add" exposure Part A closed
/// for the top-level struct gate (`OOS-DP7-11`'s `else { continue }`), one
/// level down. `AdditionalLandPlaySource` (`crates/card-types/src/state/
/// stubs.rs:737-744`) is the ONLY genuine instance surveyed: every OTHER
/// hand-destructured collection in `public_state_hash` either unpacks a bare
/// tuple (`(PlayerId, ObjectId)`, `(PlayerId, ObjectId, ManaCost)` — fixed
/// arity, a field add would be a type error, not a silent gap) or delegates
/// to an element type that already has its own `HashInto` impl
/// (`DungeonState`, `PlayFromTopPermission`, `PlayFromGraveyardPermission`).
const HAND_HASHED_ELEMENT_TYPES: &[(&str, &str)] =
    &[("additional_land_play_sources", "AdditionalLandPlaySource")];

/// Find `for <pat> in self.<field>` / `for <pat> in self.<field>.iter()` /
/// `for <pat> in &self.<field>` in `body` and return the loop's bound
/// pattern name and its body text.
fn find_for_loop_over_self_field(body: &str, field: &str) -> Option<(String, String)> {
    let b = body.as_bytes();
    let mut from = 0usize;
    while let Some(rel) = body[from..].find("for ") {
        let at = from + rel;
        let after_for = at + "for ".len();
        let Some(in_rel) = body[after_for..].find(" in ") else {
            from = after_for;
            continue;
        };
        let in_at = after_for + in_rel;
        let pat = body[after_for..in_at].trim().to_string();
        let src_start = in_at + " in ".len();
        let mut j = src_start;
        while j < b.len() && b[j] != b'{' {
            j += 1;
        }
        if j >= b.len() {
            from = src_start;
            continue;
        }
        let iter_src = body[src_start..j].trim();
        let stripped = iter_src.trim_start_matches('&');
        let stripped = stripped
            .trim_end_matches(".into_iter()")
            .trim_end_matches(".iter()")
            .trim();
        if stripped == format!("self.{field}") {
            let end = match_delim(b, j, b'{', b'}');
            return Some((pat, body[j + 1..end - 1].to_string()));
        }
        from = j;
    }
    None
}

/// Every `HAND_HASHED_ELEMENT_TYPES` entry's element type declares fields
/// that are ALL referenced inside its own per-element loop body in
/// `public_state_hash` — closing M5.
#[test]
fn hand_hashed_gamestate_elements_cover_every_field() {
    let structs = named_field_structs();
    let bodies = hashinto_impl_bodies();
    let body = public_state_hash_body();

    let mut checked_types = 0usize;
    for (collection_field, element_type) in HAND_HASHED_ELEMENT_TYPES {
        assert!(
            !bodies.contains_key(*element_type),
            "HAND_HASHED_ELEMENT_TYPES entry ({collection_field}, {element_type}): \
             `{element_type}` now HAS an `impl HashInto` — replace the hand-hashing loop in \
             public_state_hash with a single `.hash_into(&mut hasher)` call and remove this \
             entry (dead)."
        );
        let fields = structs.get(*element_type).unwrap_or_else(|| {
            panic!(
                "HAND_HASHED_ELEMENT_TYPES entry ({collection_field}, {element_type}): \
                 `{element_type}` is not a named-field struct under the scan roots"
            )
        });
        let (pat, loop_body) = find_for_loop_over_self_field(&body, collection_field)
            .unwrap_or_else(|| {
                panic!(
                    "HAND_HASHED_ELEMENT_TYPES entry ({collection_field}, {element_type}): no \
                     `for ... in self.{collection_field}...` loop found in public_state_hash \
                     (dead entry, or the loop's shape changed)"
                )
            });
        checked_types += 1;
        let mut missing: Vec<String> = Vec::new();
        for f in fields {
            // Full is required, not mere presence — the same H2 lesson applies here:
            // `let _ = &src.count;` still contains the substring `src.count`, so a bare
            // presence check would pass it silently. token_coverage requires the field
            // actually reach a hasher.
            if token_coverage(&loop_body, &format!("{pat}.{f}")) != FieldCoverage::Full {
                missing.push(f.clone());
            }
        }
        assert!(
            missing.is_empty(),
            "\n\n`{element_type}` (hand-hashed per-element in public_state_hash's `for {pat} \
             in self.{collection_field}...` loop, HAS NO `impl HashInto` of its own) declares \
             fields never FULLY fed to the hasher in that loop body:\n  {}\n\n\
             A field silently added to `{element_type}` and never added to this loop is \
             invisible to every other gate in this file (the struct gate cannot see it -- there \
             is no `impl HashInto` to check against). Either add the missing field(s) to the \
             loop body, or give `{element_type}` a real `HashInto` impl and delegate to it.\n",
            missing.join(", ")
        );
    }
    assert!(
        checked_types > 0,
        "HAND_HASHED_ELEMENT_TYPES is non-empty but nothing was checked -- a scanner broke"
    );
}
