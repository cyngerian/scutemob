# PB-DX7 — SR-19 gate holes: implementation spec

Task `scutemob-207`. **Test-only batch.** Target file: `crates/engine/tests/core/hash_schema.rs`
(the SR-19 half, from the `NOT_HASHED` doc block at ~line 1255 to EOF). Engine source
(`crates/engine/src/state/hash.rs`) may be edited ONLY if a real unhashed field is found and the
disposition is "hash it" (that moves HASH_SCHEMA_VERSION — see §4).

## 0. Ground truth measured at HEAD (do not re-derive from scratch; verify if you doubt it)

Derivation rule: every `^impl HashInto for <T>` in `crates/engine/src/state/hash.rs`, where `<T>` is
the maximal token of `[A-Za-z0-9_:]`. Classify by resolving `<T>`'s **last `::` segment** against
`struct`/`enum` declarations under `SCAN_ROOTS` (`crates/engine/src`, `crates/card-types/src`).

- **139 impls, 139 unique targets.**
- **52 structs**, **79 enums**, **8 primitive/std** (`u8 u32 u64 i32 usize bool String str`).
- **15 are path-qualified** (contain `::`): **5 structs** — `MergedComponent`, `FlashGrant`,
  `PlayFromTopPermission`, `PlayFromGraveyardPermission`, `SacrificedCreatureLki` — and
  **10 enums** — `FlashGrantFilter`, `PlayFromTopFilter`, `CastFromGraveyardAdditionalCost`,
  `ChosenColorRef`, `ReplacementManaSourceFilter`, `FlushResumeSite`, `PendingTriggerKind`,
  `AltCostKind`, `CraftMaterials`, `GiftType`.
- Enum impl shapes: **0** have a `_ =>` catch-all arm. **7** have no `match self` at all
  (`Color ManaColor SuperType CardType Phase Step EffectLayer`) — all are `(*self as u8)`, which
  Rust permits only for all-unit enums. The single `..` occurrence in an enum impl
  (`TriggerCondition`) is inside a **comment** and disappears under `strip_comments`.

**The brief's "10 path-qualified enums are outside the gate" is a true statement about a
misleading subset**: ALL 79 hashed enums are outside the struct gate, path-qualified or not.
Scope the enum work to all 79.

**The brief's cite `hash_schema.rs:1540-1541` is wrong** (it names the
`COVERAGE_MUST_INCLUDE` assertion inside `coverage_scanners_are_not_vacuous`). The actual silent
skip is the `let Some(body) = bodies.get(ty) else { continue };` in
`every_hashed_struct_field_is_hashed_or_allowlisted`. Re-verify by symbol, record the correction.

## 1. Both holes are REPRODUCED at HEAD — do not re-argue them, they are facts

- **OOS-DP7-11**: deleting `self.is_token.hash_into(hasher);` from
  `impl HashInto for crate::state::game_object::MergedComponent` leaves
  `cargo test -p mtg-engine --test core hash_schema` at **21 passed / 0 failed**, including
  `stream_fingerprint_is_pinned` (the canonical fixture carries no merged component, so the
  stream digest does not cover it either).
- **OOS-DP9-13**: rewriting the `EffectChoiceQuestion::SearchLibrary { candidates, may_fail_to_find }`
  arm as `{ candidates, .. }` with the `may_fail_to_find` feed dropped leaves the same 21/21 green
  **and** `cargo clippy -p mtg-engine --lib -- -D warnings` clean (the `..` silences
  `unused_variables`).

## 2. Part A — OOS-DP7-11: normalise the scanner key

In `hashinto_impl_bodies()`:

- Key the map on the **bare name** (last `::` segment). **Do NOT rename any call site in
  `hash.rs`** — the seed's whole point is that the gate must not depend on how the impl is spelled.
- Replace the current silent first-writer-wins `out.entry(ty).or_insert_with(..)` with an explicit
  **duplicate-key panic** naming both spellings. (Rust forbids duplicate trait impls, so this can
  only fire if the scanner is broken — which is exactly what we want it to say.)
- Return the original spelling too, so diagnostics can print it: make the value a small struct or a
  `(spelling, body)` pair. Your call; keep it readable.

New test **`every_hashed_type_resolves_to_a_declaration`** (this is the fail-BY-NAME requirement):
for every impl target, its bare name must either (a) resolve to a declared struct or enum under
`SCAN_ROOTS`, or (b) be on an explicit `HASHED_PRIMITIVE_TARGETS` list (the 8 above) with the list
carrying a one-line reason. Anything else fails by name. This is what makes a future
mis-keyed/mis-spelled impl loud instead of silent.

New test (or extension) **every hashed struct is actually parsed**: for every impl target classified
as a struct, assert `named_field_structs()` contains it — else fail by name. **Check first whether
any hashed struct is declared without `pub`** (`named_field_structs()` greps `"pub struct "` only);
`pub(crate) struct Foo` does NOT match that needle. If any hashed struct is non-`pub`, that is a
THIRD instance of the same class and must be fixed in the scanner and reported as a new finding.

After the fix the 5 path-qualified structs enter
`every_hashed_struct_field_is_hashed_or_allowlisted`. **Expect real findings** — e.g.
`PlayFromTopPermission.on_cast_effect` is deliberately not hashed (only `.is_some()` is);
that needs an explicit `NOT_HASHED` entry carrying its reason, not a silent pass. Work through
every one; see §4.

## 3. Part B — OOS-DP9-13: enum variant coverage

New scanner `named_enum_variants()` → `BTreeMap<String, Vec<Variant>>` over `SCAN_ROOTS`, where
`Variant { name, kind }` and `kind` is `Unit` | `Tuple(n)` | `Named(Vec<field>)`. Reuse
`strip_comments` / `strip_attributes` / `match_delim` / `literal_len`; split variants at depth-0
commas exactly as `struct_field_names` does.

New test **`every_hashed_enum_variant_field_is_hashed_or_allowlisted`**. For each impl target
classified as an enum:

- If the body contains no `match self`: assert **every** declared variant is `Unit`. (That is the
  `(*self as u8)` shape; a data-carrying variant there would be a real hole.)
- Otherwise parse the arms of the top-level `match self { … }`:
  - **Reject a `_ =>` arm, and reject a bare-identifier catch-all arm**, for any enum with at least
    one field-carrying variant — a new variant must not be able to fall into a catch-all silently.
  - Map each arm to a variant by the last `::` segment of the pattern path (impls commonly
    `use … as S;` and write `S::Variant`, so match on the segment, not the full path).
  - Assert **every declared variant has an arm**.
  - `Named` variant: the pattern must bind **every** declared field by name, must contain **no `..`
    rest pattern**, and each bound name must appear as a whole token in that arm's body.
  - `Tuple(n)` variant: the pattern must have exactly `n` bindings, **no `_`**, **no `..`**, and each
    binding must appear as a whole token in that arm's body.

Allowlist **`NOT_HASHED_VARIANT_FIELDS: &[(&str, &str, &str, &str)]`** = `(enum, variant, field,
reason)`, with a **dead-entry guard** mirroring `not_hashed_allowlist_has_no_dead_entries` (the
entry must name a real declared variant field that is genuinely not fed). `field` for a tuple
variant is its **index as a string** (`"0"`, `"1"`).

Non-vacuity floors, set well below the measured reality (measure, then floor at roughly 2/3):
`MIN_HASHED_ENUMS`, `MIN_VARIANTS_CHECKED`, `MIN_VARIANT_FIELDS_CHECKED`. Plus positive/negative
controls for the new matcher, in the style of the existing `coverage_scanners_are_not_vacuous`.

**Also MEASURE (report, do not gate yet unless it comes back clean):** does every arm of every
hashed enum hash a **distinct** discriminant literal? Two variants writing the same `Nu8` collide.
If it is already clean across all 79, add it as a gate; if not, report the exceptions to me and I
will disposition them.

## 4. Part C — disposition every newly in-scope unhashed field (AC 6383)

Zero silent allowlisting. For each field newly in scope that is genuinely not fed:

- **Hash it** → edit `hash.rs`, then bump `HASH_SCHEMA_VERSION` **from the gate's own failing
  output** (`hash_schema_version_sentinel` / `stream_fingerprint_is_pinned` /
  `declaration_fingerprint_is_pinned` print the computed values — take them from there, never
  predict), append to the history per the `state/hash.rs` checklist without editing a shipped row,
  and state the justification in the commit and in the impl comment; **or**
- **Allowlist it** → the reason goes in the allowlist entry itself, on the entry, naming why the
  field is not game state (pure runtime scratch, or fully derived from other hashed fields).

Report every disposition to me with its reasoning before you finalise — I own the HASH-bump call.

PROTOCOL is expected **unmoved**; gate-execute `protocol_schema` either way and say so.

## 5. Obsolete comments in `hash.rs`

Three comment blocks in `hash.rs` (~lines 3303-3345, and the SR-19 notes near the `NOT_HASHED` doc
in the test file) instruct future authors to write impls with **bare names on purpose** because the
gate cannot see path-qualified ones, and state that the enum impls "are held by review and by
`stream_fingerprint`, not by the SR-19 scan". After this batch both statements are false. Rewrite
them to say what is now true — do not delete the history, correct it in place with the reason.
`hash.rs` comment-only edits are fine in a test-only batch (0 behavioural lines) — say so explicitly
in the handoff so the "test-only" claim stays honest.

## 6. Proof standard (AC 6385) — executed, not argued

Every new or widened gate is proven by an **executed revert**, watched red, then restored green:

1. Delete `self.is_token` from the path-qualified `MergedComponent` impl → the struct gate now fails
   **by name** (`MergedComponent.is_token`). [was green at HEAD]
2. `SearchLibrary { candidates, .. }` with the feed dropped → the new enum gate fails by name.
   [was green at HEAD, and clippy-clean]
3. Re-introduce the path-qualification skip in `hashinto_impl_bodies()` (revert the normalisation)
   → the "hashed struct is parsed"/"resolves to a declaration" gate fails.
4. A tuple-variant `_` binding, and a `_ =>` catch-all arm → each fails.
5. A bogus and a dead `NOT_HASHED_VARIANT_FIELDS` entry → the dead-entry guard fails.
6. Drop one of the new non-vacuity floors' inputs (e.g. make a scanner return empty) → the floor
   fires rather than the gate passing vacuously.

Record each row as RED/GREEN with the actual assertion text in
`memory/primitives/pb-DX7-execution-notes.md`. A row you cannot make discriminate is recorded as
**UNDISCRIMINATED with its reason**, not quietly dropped.

## 7. Close conditions

- `cargo test --workspace --no-fail-fast` to a file; baseline to beat is **4,508 / 0 / 5**,
  46 result-producing targets, residual list empty. Itemise the delta.
- `cargo clippy --workspace --all-targets -- -D warnings`, `cargo fmt --check`,
  `tools/check-defs-fmt.sh` all clean.
- **0 card-def edits.** Coverage must be proven unmoved by regenerating `tools/authoring-report.py`,
  not by an empty diff alone.
