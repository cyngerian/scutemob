# PB-DX7 review — SR-19 gate holes (`scutemob-207`)

**Reviewer**: primitive-impl-reviewer (Opus)
**Date**: 2026-08-11
**Branch**: `feat/pb-dx7-sr-19-gate-holes-the-hashed-field-gate-skips-path-qua`
**Commits reviewed**: `0c47700f` (riders), `a169fbec` (registry rows), `71b02209` (the gate)
**Seeds**: `OOS-DP7-11`, `OOS-DP9-13` (primary); `OOS-DP10-1`, `OOS-DP9-10` residual (riders)

> **Method disclosure, stated first because this batch's subject is claims that overstate their
> own reach.** This review session had **no shell tool**. Nothing below was executed — no `cargo
> test`, no `git diff`, no revert. Every finding is derived by reading source with `Read`/`Grep`.
> Where a claim could only be settled by execution I say so rather than implying I checked it.
> The adversarial findings (H1, H2, M3, M4) are all *static* constructions: I name the exact edit
> and the exact line of gate code that lets it through, so each is cheap for the fix phase to
> confirm by running the revert.

---

## Verdict: **needs-fix** — 2 HIGH, 5 MEDIUM, 9 LOW

Three of the four claimed closures hold. **`OOS-DP9-10`'s does not**, and `OOS-DP9-13`'s holds
for the *spelling* that was filed rather than the *class* it names.

The engine-side work (Part A) is good and the seed's prescription was followed exactly: the
scanner is keyed on the bare name, **zero call sites were renamed** (verified — all 15
path-qualified impls are still spelled path-qualified in `hash.rs`), and a mis-keyed impl now
fails by name. The enum gate (Part B) is a large, careful, genuinely new instrument. The three
coordinator-directed follow-ups are proportionate and their allowlists are mostly honest.

What the batch did not do is turn its own adversary on its own artefacts hard enough. Its two
new coverage predicates both answer **"does this token appear?"**, never **"does anything reach
the hasher?"** — so the exact defect each gate names is still reachable one spelling away. And
the `OOS-DP9-10` rider ships a census that is a count of one *spelling* of an unordered container
while the registry row now records it as a count of unordered containers; three files inside the
gate's own scan root carry 19 of them and are pinned at zero.

---

## Findings

| # | Sev | File:line | Description |
|---|-----|-----------|-------------|
| H1 | **HIGH** | `tests/core/unordered_iteration_ratchet.rs:154-169` | **The ratchet's needle cannot see the codebase's dominant idiom.** `casting.rs` (9), `resolution.rs` (7) and `turn_actions.rs` (3) carry `HashSet::new()` sites, measure **0**, and are pinned at **0**. `OOS-DP9-10`'s closure does not hold. |
| H2 | **HIGH** | `tests/core/hash_schema.rs:2007-2043`, `:2896`, `:2823` | **`FieldCoverage::Full` means "the token appears", not "the value is hashed".** `SearchLibrary { candidates, may_fail_to_find } => { …; let _ = may_fail_to_find; }` drops the field from the hash and stays green — the filed `OOS-DP9-13` defect, one spelling over. |
| M3 | MEDIUM | `tests/core/hash_schema.rs:2731-2745`, `:2776-2784`, `:3418` | **No arm is required to feed the hasher anything.** A Unit-variant arm rewritten `Foo::A => {}` passes the enum gate, and `first_integer_literal` returns `None` so the discriminant ratchet skips it. Two such arms = two values with identical digests, all gates green. |
| M4 | MEDIUM | `tests/core/hash_schema.rs:2884-2923` vs `:2813-2820` | **The Named branch does not reject a `_` binding; the Tuple branch does.** `{ field: _ }` classifies `Full` whenever the arm body contains any standalone `_` token (`\|_\|`, `let _ =`, `Foo(_)`). V11 only ever tested the tuple side. |
| M5 | MEDIUM | `tests/core/hash_schema.rs:2197-2199`; `state/hash.rs:8449-8453` | **The GameState gate stops one level short of its own rationale.** `public_state_hash` hand-hashes collection elements field-by-field; a 4th field on `AdditionalLandPlaySource` is silently unhashed — the struct gate skips types with no `HashInto` impl in silence, the same `else { continue }` shape Part A just fixed. |
| M6 | MEDIUM | `tests/core/hash_schema.rs:3797` | **The new GameState gate uses the weaker matcher the same commit just replaced.** It calls `body_references_field`, not `struct_field_coverage`, so `self.<field>.is_some().hash_into(..)` in `public_state_hash` reads as full coverage — the exact `PARTIALLY_HASHED` shape, one level up, in the same file. |
| M7 | MEDIUM | `tests/core/hash_schema.rs:2398-2415` | **Both `PARTIALLY_HASHED_VARIANT_FIELDS` reasons cite a line range that contains no such reasoning**, and one of the two arms has no in-source comment at all. "Documented in-source at `hash.rs:4105-4111`" is false. |
| L1 | LOW | `tests/core/hash_schema.rs:3739-3748`; audit row `OOS-DX7-3` | `loop_detection_hashes`'s reason cites "state/mod.rs's own doc on this field" for a claim that doc does not make. The claim is true; the citation points at the wrong file. |
| L2 | LOW | `state/hash.rs:6793-6796` vs `:796-806` | Two comments shipped in the same commit contradict each other about whether `HASH_SCHEMA_HISTORY` inherited the discriminant error. Also "entries below" — the history is above. |
| L3 | LOW | audit rows `OOS-DP9-13`, `OOS-DX7-3` | "3 of its 45 fields reached neither `public_state_hash` **nor any stated exclusion list**" is wrong for 2 of the 3: `public_state_hash`'s own doc (`hash.rs:8283-8289`) lists both under "Excludes:". |
| L4 | LOW | `tests/core/hash_schema.rs:3232-3237` | `enum_coverage_scanners_are_not_vacuous` floors **all declared enums** (~109+) against `MIN_HASHED_ENUMS` (52, a floor for *hashed* enums). Wrong population, ~2× slack. |
| L5 | LOW | `tests/core/hash_schema.rs:1441-1492`, `:1662-1668` | The impl scanner silently drops anything its literal needle `"impl HashInto for "` misses; `MIN_HASHINTO_IMPLS = 80` against a live 139 leaves 59 impls of silent headroom. |
| L6 | LOW | `tests/core/decision_gate.rs:1077-1176` | The new cross-check compares a **replica** of the copy's walk to the canonical walk, not the copy. If `pb_dp9_effect_choice.rs::json_contains_variant` drifts, this test stays green. |
| L7 | LOW | `tests/core/hash_schema.rs:3384-3403` | `first_integer_literal` reads the first digit run anywhere in the arm body, including inside `u32`/`i64`/`x2`. An arm with a cast before its tag is measured with the wrong tag. |
| L8 | LOW | `tests/core/hash_schema.rs:3521-3686` | Over-widening: ~165 lines of hand-built `Effect` values, permanently maintained, proving one sample point that can essentially only fail if `HashInto` is broken far more visibly elsewhere. |
| L9 | LOW | `tests/core/hash_schema.rs:1730-1747` | `every_hashed_struct_is_parsed_by_named_field_structs`'s non-`pub` half is structurally unreachable — it only inspects targets absent from `decls.index`, which `every_hashed_type_resolves_to_a_declaration` already reddens on. |

---

## Finding details

### H1 — the unordered-container ratchet counts one spelling, and three files in its own scan root carry the other

**File**: `crates/engine/tests/core/unordered_iteration_ratchet.rs:154-169` (`unordered_container_count`), `:103-122` (`UNORDERED_CEILINGS`)
**Claimed closure**: `OOS-DP9-10` residual — *"there is no gate for the shape"* — recorded in the
registry as **CLOSED**, with **"27 occurrences across 6 files"** and *"every unlisted file under
those roots is pinned at ZERO — the load-bearing half"*.

The needle is the literal type-annotation form:

```rust
let map_needle = format!("Hash{}<", "Map");
let set_needle = format!("Hash{}<", "Set");
```

That form is **not how most of the scanned tree writes an unordered container.** Measured by grep
over the gate's own three scan dirs:

| file | ceiling | `Hash{Map,Set}<` (what the gate counts) | `Hash{Map,Set}::{new,…}` constructions |
|---|---|---|---|
| `rules/casting.rs` | **0 (unlisted)** | **0** | **9** (`:3000, :3401, :3583, :4487, :5629, :5695, :5826, :5931, :7852`) |
| `rules/resolution.rs` | **0 (unlisted)** | **0** | **7** |
| `rules/turn_actions.rs` | **0 (unlisted)** | **0** | **3** |
| `effects/mod.rs` | 3 | 3 | 16 |
| `rules/replacement.rs` | 11 | 11 | 9 |
| — total under the 3 scan dirs — | | **27** | **54** |

`crates/engine/src/rules/casting.rs` has **nine** `let mut seen = std::collections::HashSet::new();`
sites and `grep 'HashSet<' casting.rs` returns **no matches**. The file's live count is 0, its
ceiling is 0, and the gate is green.

**The concrete defeat** — exactly the defect the gate names, in the file the gate's own revert row
V5 used:

```rust
// crates/engine/src/rules/layers.rs  (unlisted -> ceiling 0)
let mut best = std::collections::HashMap::new();
best.insert(id, score);
let winner = best.into_iter().max_by_key(|(_, s)| *s);   // unordered iteration -> outcome
```

`unordered_container_count` sees no `HashMap<`. Both `unordered_scanner_is_not_vacuous` and
`unordered_container_surface_is_ratcheted` stay green. **V5 reddened only because the implement
phase wrote the probe with an explicit type annotation** — the one spelling the gate can see.
A revert matrix that picks the spelling the gate was written for cannot discover this class; that
observation is this batch's own thesis (`MR-M11-01`, `OOS-DP7-11`) applied to its own rider.

The module doc names two residuals — block comments and cross-crate type aliases — and calls them
*"deliberately obscure code that review would reject"*. Type-inferred construction is neither
obscure nor rejected: **16 of the existing sites are written that way**, in the files the gate
scans. The stated residual is not the only one, and the unstated one is the dominant one.

Nothing live is broken today: all 19 constructions in the three zero-pinned files were inspected
and every one is either an empty `&HashSet::new()` argument or a `seen.insert(x)` membership
filter. The defect is entirely in the gate's reach and in the census the registry now records as
fact.

**Fix**:
1. Widen the needle to whole-token `HashMap` / `HashSet` (or add the `HashMap::`, `HashSet::`,
   `collections::HashMap`, `collections::HashSet` forms), re-measure every ceiling, and add a
   control asserting the counter sees `let x = std::collections::HashSet::new();`.
2. Correct the module docs, the `OOS-DP9-10` registry row and
   `memory/primitives/pb-DX7-execution-notes.md` §4.2 — "27 occurrences across 6 files" is a
   census of one spelling, not of unordered containers.
3. Re-run V5 with an **inferred-type** probe, not an annotated one, and record that row.
4. If the re-measure is too large to land here, **revert the rider and re-open `OOS-DP9-10`'s
   residual** rather than leave a CLOSED row backed by a partial scan.

---

### H2 — `FieldCoverage::Full` means "the token appears", so the filed defect is still reachable

**File**: `crates/engine/tests/core/hash_schema.rs:2007-2043` (`token_coverage`), consumed at
`:2823` (tuple) and `:2895` (named)

```rust
if body[after..].starts_with(".hash_into(") {
    all_partial = false;                       // Full
} else if let Some(method) = summariser_chained_to_hash_into(body, after) {
    summarisers.insert(method);                // Partial
} else {
    all_partial = false;                       // <-- Full, for ANY other use
}
```

The `else` arm classifies **every non-summariser occurrence as `Full`**, including occurrences
that feed nothing. `FieldCoverage::Full`'s own doc hedges this (*"the value's real content
plausibly reaches the hasher somewhere"*), but the test is named
`every_hashed_enum_variant_field_is_hashed_or_allowlisted` and the closure claim is that
`OOS-DP9-13` — *"a hashed ENUM variant can silently drop a field"* — is CLOSED.

**The concrete defeat**, in `hash.rs`, on the seed's own card:

```rust
EffectChoiceQuestion::SearchLibrary { candidates, may_fail_to_find } => {
    1u8.hash_into(hasher);
    candidates.hash_into(hasher);
    let _ = may_fail_to_find;          // was: may_fail_to_find.hash_into(hasher);
}
```

No `..`, no `_` binding, every declared field bound, the binding appears as a whole token →
`Full` → green. `let _ = x;` triggers no default rustc or clippy lint, so `-D warnings` is clean
too. The field is gone from the byte stream and every gate in the suite passes — which is
verbatim the sentence `OOS-DP9-13` was filed as. The same holds for the more plausible accident,
a binding used in a guard rather than a feed (`if *may_fail_to_find { }`).

This weakness is inherited from `body_references_field` and predates the batch on the struct side,
so it is not a regression. But the batch (a) extended it to a brand-new population of 1,252
variants, (b) named the test "is hashed", and (c) filed `OOS-DX7-2` whose residual paragraph
covers the *neighbouring* case (a multi-level summariser chain) and not this one.

**Fix**: tighten `token_coverage`'s `Full` arm to require that at least one occurrence is
(i) `<token>.hash_into(`, (ii) inside a `for … in <token>`/`&<token>` iteration whose body
hashes, or (iii) an explicit cast fed to `hash_into` (`(*<token> as u8).hash_into(` — the live
shape at `hash.rs:3202`, `:3279`, `:7739`); classify everything else `NotReferenced` (fail-closed)
rather than `Full` (fail-open). If that is too large for this batch, at minimum **file the residual
on `OOS-DX7-2` in the terms above** and reword the test's own doc so the name is not read as a
guarantee it does not give.

---

### M3 — nothing requires an arm to feed the hasher at all

**File**: `crates/engine/tests/core/hash_schema.rs:2731-2745` (the no-`match self` branch),
`:2776-2784` (the Unit-variant arm branch), `:3418` (`enum_discriminant_collisions`)

A Unit variant has no fields, so the enum gate checks exactly one thing about it: that the arm
pattern carries no payload. It never looks at the arm body. So:

```rust
GiftType::Food => {}          // was: GiftType::Food => 0u8.hash_into(hasher),
GiftType::Card => {}          // was: GiftType::Card => 1u8.hash_into(hasher),
```

passes `every_hashed_enum_variant_field_is_hashed_or_allowlisted` (both arms present, both
payload-free), and passes `discriminant_collisions_are_ratcheted_at_their_known_bad_state`
because `first_integer_literal` returns `None` for an empty body and the arm is skipped
(`:3415-3420`). Two genuinely different `GiftType` values now produce **identical byte streams** —
which is the precise harm the whole SR-19 programme exists to prevent — with every gate in the
file green. `stream_fingerprint_is_pinned` catches it only if the canonical fixture happens to
carry a `GiftType`, and the fixture's own doc records that it does not populate
`pending_triggers` or `stack_objects`.

The bare-cast branch has the same shape: `(*self as u8).hash_into(hasher);` replaced by
`let _ = hasher;` in `impl HashInto for EffectLayer` leaves all variants Unit, so the branch's
only assertion still holds. (The fully-empty body is caught, but by `unused_variables` under
`-D warnings`, not by this gate.)

**Fix**: assert every arm body contains at least one `.hash_into(` occurrence, and that a
no-`match self` impl body contains one too; and make `enum_discriminant_collisions` treat an arm
with **no** integer literal as a reportable finding rather than a skip.

---

### M4 — the Named branch does not reject a `_` binding, though the Tuple branch does

**File**: `crates/engine/tests/core/hash_schema.rs:2884-2923`; compare `:2813-2820`

The tuple branch is explicit:

```rust
if s == "_" {
    violations.push(format!("{bare}::{variant_name}.{idx}: tuple field bound to `_` …"));
    continue;
}
```

The named branch has no equivalent. `{ candidates, may_fail_to_find: _ }` resolves `binding` to
the string `"_"` and calls `variant_field_coverage(arm_body, "_")`, which searches the arm body
for `_` as a whole token. Today `hash.rs` contains no standalone `_` in any arm body (grepped:
only `Some(InterveningIf::CardDef(_))` inside a comment and `ZoneId::Hand(_)` in
`public_state_hash`), so the hole is **latent, not live**. It becomes live the moment an arm body
contains a closure `|_|`, a `let _ =`, or a `matches!(x, Foo(_))` — at which point
`{ field: _ }` classifies `Full` and the field is silently dropped.

V11 proved the tuple side reddens. Nothing proved the named side, and the named side is where
`OOS-DP9-13`'s own example lives.

**Fix**: mirror the tuple branch's `_` rejection in the named branch (reject any binding that is
exactly `_`, and reject one that starts with `_` unless allowlisted), and add the missing revert
row.

---

### M5 — the GameState gate stops one level short of its own rationale

**File**: `crates/engine/tests/core/hash_schema.rs:2197-2199`; `crates/engine/src/state/hash.rs:8449-8453`

`OOS-DX7-3`'s argument for the new gate is exact and I agree with it: *"a 46th field added
tomorrow moves `decl_fingerprint` and forces a human to look at a fingerprint mismatch — it does
not tell them they added a field and did not feed it."*

That argument applies unchanged one level down, and nothing in the batch answers it there.
`public_state_hash` hand-hashes several collections **element field by element field**:

```rust
for src in self.additional_land_play_sources.iter() {
    src.source.hash_into(&mut hasher);
    src.controller.hash_into(&mut hasher);
    src.count.hash_into(&mut hasher);
}
```

`AdditionalLandPlaySource` (`crates/card-types/src/state/stubs.rs:737-744`) has **no
`impl HashInto`**, and the struct gate's very first statement is a silent skip for exactly that
case:

```rust
let Some(impl_body) = bodies.get(ty) else {
    continue; // struct without a HashInto impl — out of this gate's scope
};
```

So a 4th field on `AdditionalLandPlaySource` is unhashed, and: the struct gate skips the type; the
enum gate does not apply; `every_gamestate_field_is_in_public_hash_or_allowlisted` sees
`self.additional_land_play_sources` referenced and passes; all four allowlists stay clean. The
same shape covers `pending_echo_payments` / `pending_cumulative_upkeep_payments` /
`pending_recover_payments` / `prevention_counters` / `pending_commander_zone_choices` /
`dungeon_state`.

Note this `else { continue }` is *the same construct* Part A was dispatched to remove — the
execution notes §3.1 correctly identify `let Some(body) = bodies.get(ty) else { continue }` as the
mechanism of `OOS-DP7-11`. Part A fixed the key; the `continue` itself is still there, now for a
different reason.

**Fix**: either (a) give `AdditionalLandPlaySource` a real `HashInto` impl and fold it whole (a
behavioural `hash.rs` edit — out of scope for a test-only batch, so file it), or (b) add a
`HAND_HASHED_ELEMENT_TYPES` roster to the new GameState gate asserting that for each such
collection every declared field of the element type is referenced inside its own loop body, or at
minimum (c) file the residual on `OOS-DX7-3` — it currently claims no such residual.

---

### M6 — the GameState gate uses the weaker matcher the same commit replaced

**File**: `crates/engine/tests/core/hash_schema.rs:3797` (and `:3848`)

The batch's third disposition category exists because *"`body_references_field` matches
`self.<field>` as a substring regardless of what follows, so `self.on_cast_effect.is_some()
.hash_into(hasher)` passed as 'covered' — the gate succeeding on a technicality of its own
matcher"* (execution notes §7). It then wrote `FieldCoverage` / `struct_field_coverage` and
refactored both halves of the SR-19 gate onto one classifier explicitly so *"a reviewer should
never find the two halves disagreeing about what counts as coverage"* (`:1943-1952`).

The new GameState gate, landed in the same commit, calls `body_references_field` — the matcher
that had just been diagnosed. `self.day_night.is_some().hash_into(&mut hasher)` in
`public_state_hash` would read as full coverage with no `PARTIALLY_HASHED`-equivalent to record
it. There is no third disposition category at this level and no note saying why one is not
needed.

Today nothing in `public_state_hash` is a summariser feed (I read all 42 references at
`hash.rs:8306-8468` — `self.players.len()`, `self.dungeon_state.len()` and the hand/library
`z.len()` reads are all length prefixes followed by a full iteration, matching the audit's own
rule). So this is a consistency/fail-open defect, not a live one.

**Fix**: switch both GameState assertions to `struct_field_coverage` and add a
`PARTIALLY_HASHED_GAMESTATE` allowlist (shipping empty), or state in the gate's doc why the
weaker predicate is deliberate here.

---

### M7 — both `PARTIALLY_HASHED_VARIANT_FIELDS` reasons cite lines that contain no such reasoning

**File**: `crates/engine/tests/core/hash_schema.rs:2398-2415`

Both entries cite `hash.rs:4105-4111`. Those lines are:

```
4105  impl HashInto for StackObjectKind {
4106      fn hash_into(&self, hasher: &mut Hasher) {
4107          match self {
4108              StackObjectKind::Spell { source_object } => {
4109                  0u8.hash_into(hasher);
4110                  source_object.hash_into(hasher);
4111              }
```

— the impl header and the `Spell` arm. No reasoning about `embedded_effect` appears there.
The real locations are:

* `TriggeredAbility.embedded_effect` — reasoning at **`hash.rs:4132-4138`**, feed at `:4139`.
* `ActivatedAbility.embedded_effect` — feed at **`hash.rs:4120`**, and **there is no comment on
  that arm at all**. The entry's phrase "mirrors PendingTrigger's identical reasoning" is an
  inference the reviewer must make, not something documented in-source as the entry implies.
  (The reasoning it mirrors is at `hash.rs:3578-3587`.)
* `ForecastAbility.embedded_effect` — the doc at `:2393` cites `hash.rs:4210`; the actual full
  feed is at **`:4237`**.

The other two allowlists check out: `PlayFromTopPermission.on_cast_effect`'s reason is quoted
**verbatim** from `hash.rs:2965-2966` ✓, and `PendingTrigger.embedded_effect`'s
"(SR-19, `HASH_SCHEMA_HISTORY` entry 40)" resolves to a real entry 40 at `hash.rs:351-368` that
does describe the change ✓ (the quoted wording itself is from `hash.rs:3585-3587`).

For a batch whose stated lesson is *"a gate cited in a comment is a claim like any other"*,
shipping two allowlist entries whose in-source citation is wrong — and one whose "documented
in-source" premise is not true — is the finding this batch would file against anyone else.

**Fix**: re-cite to `hash.rs:4132-4138` (TriggeredAbility) and `hash.rs:3578-3587` (the shared
reasoning); state plainly that the `ActivatedAbility` arm carries no comment, and either add one
(comment-only, in scope) or say the entry's reason is inferred from its siblings. Correct
`:2393`'s `4210` → `4237`.

---

### LOW findings

**L1** — `GAMESTATE_NOT_IN_PUBLIC_HASH`'s `loop_detection_hashes` entry
(`hash_schema.rs:3740-3748`) attributes *"two independent engine instances processing the SAME
legal game may accumulate DIFFERENT hash histories"* to *"(state/mod.rs's own doc on this
field)"*. `state/mod.rs:288-296` says only *"metadata, not game state"*. The quoted claim lives
at **`hash.rs:8286-8289`**, in `public_state_hash`'s own doc. Substance verified and sound;
citation wrong. `OOS-DX7-3` repeats the misattribution ("its own doc says…"), as does execution
notes §9. Fix: re-cite to `hash.rs:8286-8289`.

*(The `history` entry's reason I re-verified independently and it holds: `grep '\.history()'`
across the workspace returns exactly one hit, `crates/engine/tests/rules/replacement_effects.rs:812`;
field-level reads are `engine.rs:944`/`:3534` pushes plus the two accessors at `state/mod.rs:669`/`:908`.
Worth noting the field's own doc says "Append-only event log for **triggers that look back at
history**" — the intent exists even though no caller does; the allowlist's caveat covers this.
The `card_registry` reason is verified at `state/mod.rs:438-444`.)*

**L2** — `hash.rs:6793-6796` says the discriminant error is *"inherited by several
`HASH_SCHEMA_HISTORY` entries below"*. `hash.rs:796-799`, shipped in the same commit, says the
opposite and says it was **checked**: *"none of the 18 colliding variant names, and none of their
9 shared tag values, appear anywhere in the numbered history above — so no SPECIFIC row above is
factually wrong."* The execution notes §12 record the check and its result. The stale claim
survived into the `Effect` header. Also: the history is *above* line 6793, not below. Fix: reword
6793-6796 to match the verified result.

**L3** — `OOS-DP9-13`'s and `OOS-DX7-3`'s closure text says three GameState fields *"reached
neither `public_state_hash` nor any stated exclusion list"*. `public_state_hash`'s doc
(`hash.rs:8283-8289`) has an explicit **"Excludes:"** block naming *"Event history (O(n) in game
length)"* and *"`loop_detection_hashes`"*. Only `card_registry` was genuinely unstated. The real
gap — which both rows also state correctly — is that nothing **checked** the list. Fix: drop
"nor any stated exclusion list" from both rows, or narrow it to `card_registry`.

**L4** — `enum_coverage_scanners_are_not_vacuous` (`hash_schema.rs:3232-3237`) asserts
`enum_variants.len() >= MIN_HASHED_ENUMS`. `named_enum_variants()` returns **every** `pub enum`
under the scan roots (~109+ by grep: 89 in `card-types/src`, 20 in `engine/src`), not the 79
hashed ones. The floor is against the wrong denominator and carries ~2× unintended slack. Fix:
either add a distinct `MIN_DECLARED_ENUMS` measured on the real population, or floor the hashed
intersection here as `every_hashed_enum_variant_field_is_hashed_or_allowlisted` already does.

**L5** — `hashinto_impl_bodies()` (`:1441-1492`) finds impls by the literal needle
`"impl HashInto for "`. Anything it misses (a line break after `for`, a double space, a
lifetime-parameterised concrete target `impl<'a> HashInto for Foo<'a>`) is **silently absent from
every gate in the file**, including `every_hashed_type_resolves_to_a_declaration`, which can only
inspect what the scanner already parsed. The `ty.is_empty() { continue; }` at `:1456` is a second
silent drop. `MIN_HASHINTO_IMPLS = 80` against a live 139 (verified: 139 line-anchored
occurrences) means 59 impls could vanish before any floor fires. Fix: pin the impl count as an
exact ratchet with a stated value (the `KNOWN_EFFECT_DISCRIMINANT_COLLISIONS` pattern) rather
than a 2/3 floor, and/or assert the raw `"impl HashInto"` occurrence count matches the parsed
count.

**L6** — `pb_dp9_roster_walks_agree_by_value` (`decision_gate.rs:1114-1176`) compares
`key_only_contains_variant` — a *replica* of the copy — against the canonical walk. I diffed the
replica against the real copy (`primitives/pb_dp9_effect_choice.rs:2379-2389`) and it is
byte-faithful today ✓. But the failure mode `OOS-DP10-1` is filed against is the **copy**
drifting, and if the copy drifts the replica does not, so this test stays green. The row's
closure is therefore narrower than "cross-checked by value against the canonical walk" implies:
it proves the *algorithm* is blind to unit variants (with a real discriminating control —
`Proliferate` 23 vs 0 — which is good work), not that the copy still is that algorithm. Fix:
state the residual in the row and in the test doc; the durable fix (promote the walk to a shared
home) is correctly recorded as out of scope.

**L7** — `first_integer_literal` (`:3384-3403`) skips string literals but not identifiers, so the
first digit run in `(count as u32)` or `let n: i64` is read as the arm's tag. Every `Effect` arm
happens to open with its tag today, so the measurement is right; a future arm that casts before
tagging is measured wrong, in either direction. Fix: require the digit run to be immediately
followed by `u8`/`u16`/`u32`/`u64` and preceded by a non-identifier byte, or anchor on the first
`N…u8.hash_into(` occurrence.

**L8 (scope)** — `effect_colliding_variant_digests_are_pairwise_distinct` (`:3521-3686`) is ~165
lines of hand-constructed `Effect` values that must be kept compiling for the life of the DSL and
that assert a property which can realistically only fail if `HashInto` is broken in a way a dozen
other gates would catch first. I checked the values are plausible rather than rigged — they are
(`ManaColor::White`, `EffectAmount::Fixed(1)`, `ManaPool::default()`, `TargetFilter::default()`,
`DeclaredTarget { index: 0 }` for every target-shaped field; the one place two indices differ,
`Fight`/`Bite`, distinguishes two fields *within* one variant and does not affect any
cross-variant pair) — and the test's stated limit ("18 sampled values, not a proof of
injectivity") matches exactly what it proves. It answered the coordinator's question honestly.
Whether it should have *shipped* rather than been run as a throwaway (as §6's two scanners were)
is a judgement call; the `discriminant_collisions_are_ratcheted_at_their_known_bad_state` ratchet
is the durable artefact and it is excellent. Fix (optional): keep the ratchet, downgrade the
experiment to 9 pairs instead of 18 values, or delete it and cite the executed result in the seed.

**L9** — `every_hashed_struct_is_parsed_by_named_field_structs`'s non-`pub` half (`:1730-1747`)
loops only over impl targets **not** in `decls.index`; any such target already reddens
`every_hashed_type_resolves_to_a_declaration`. It can never fire alone. Not harmful — it is a
better error message for a case another test catches — but the spec asked for a *finding* if any
hashed struct is non-`pub`, and the mechanism as written cannot produce one independently. Fix:
say so in its doc, or check `declared_non_pub` for **all** targets and report a target declared
both ways.

---

## Also checked, no finding

* **A `/* */` block comment cannot hide a roster entry here.** `strip_comments`
  (`hash_schema.rs:258-293`) handles nested block comments and preserves string literals — the
  PB-DX32 `OOS-DX32-6` defeat does not apply to `hash_schema.rs`. (It *does* apply to
  `unordered_iteration_ratchet.rs`, which strips `//` only — and that file states it.)
* **Guard clauses, or-patterns, `ref`/`@` bindings and nested destructuring all fail CLOSED**, not
  open: `Foo::Bar { x } if p => …` leaves `payload = "{ x } if p"`, which fails the
  `strip_prefix('{')/strip_suffix('}')` check; `Foo::A | Foo::B` resolves to variant `A` with
  payload `| Foo::B` and leaves `B` arm-less. Each produces a confusing message rather than a
  diagnosis, but none is a hole. Worth one sentence in the failure text.
* **A delegating enum impl fails closed**: no `match self` ⇒ all variants must be Unit ⇒ a
  data-carrying enum reddens.
* **`arm_seen`/missing-arm detection is unconditional**, so the `has_data_variant` qualifier on
  the catch-all rejection is belt-and-braces, not a gap: a variant covered only by `_ =>` is still
  reported arm-less.
* **A bare-identifier catch-all (`other => …`) is caught**, via the "names variant `other`, which
  named_enum_variants() did not find declared" path — the spec's requirement is met, by a
  different message than the spec anticipated.
* **The 5 newly-in-scope path-qualified structs are all genuinely covered**: `MergedComponent`
  (3/3, `hash.rs:2378-2384`), `SacrificedCreatureLki` (3/3, `:4627-4633`),
  `PlayFromGraveyardPermission` (4/4, `:2970-2983`), `PlayFromTopPermission` (7 full + 1
  `PARTIALLY_HASHED`, `:2950-2968`), `FlashGrant`. The spec's "expect real findings" produced
  exactly the one it predicted.
* **`declared_type_names_are_unique` covers the first-writer-wins risk** in
  `named_field_structs` / `named_enum_variants` / `all_struct_shapes` for bare-`pub`
  declarations, and `hashinto_impl_bodies` now panics on a duplicate bare key. Good.
* **The SR-9a boundary is respected**: `crates/engine/tests/core/main.rs:57` declares
  `mod unordered_iteration_ratchet;`.
* **Comment-only `hash.rs` edits cannot move a fingerprint**: `compute_frozen_prefix_digest`
  (`:862-872`) hashes the `HASH_SCHEMA_HISTORY` const *values*, and `decl_fingerprint` indexes
  declarations (of which `hash.rs` has none). HASH 74 unmoved is consistent with the edits made.

---

## Closure status

| Seed | Claimed | Actual | Notes |
|---|---|---|---|
| `OOS-DP7-11` | CLOSED | **HOLDS** | Bare-name key, **zero call sites renamed** (verified: all 15 path-qualified impls still path-qualified), 15 targets enter scope, fail-by-name added. Residual: L5. |
| `OOS-DP9-13` | CLOSED | **PARTIAL** | The filed spelling (`{ …, .. }` + dropped feed) is caught. The class it names — "a field silently dropped from a variant's arm" — is not: H2, M3, M4. |
| `OOS-DP10-1` | CLOSED | **HOLDS, narrower than stated** | Real equality check with a real discriminating control. Compares a replica, not the copy: L6. |
| `OOS-DP9-10` residual | CLOSED | **DOES NOT HOLD** | H1. Either fix the needle and re-measure, or re-open the row. |

---

## Counts re-derived independently

| Claim | Method | Result |
|---|---|---|
| 139 `HashInto` impls | `grep -c '^impl HashInto for '` on `hash.rs` | **139 ✓** (`grep` anywhere gives 145, not the notes' 146 — a 1-off in the "raw grep" figure only, immaterial) |
| 15 path-qualified = 5 struct + 10 enum | `grep '^impl HashInto for [a-z_]*::'` | **15 ✓**, names match the seed's list exactly |
| 52 struct + 79 enum + 8 primitive | arithmetic vs 139; path-qualified split confirmed 5/10 | **consistent**; the 52/79 split itself needs execution to confirm |
| 45 `GameState` fields | `grep -c '^    pub(crate) [a-z_]*:'` on `state/mod.rs` | **45 ✓** |
| 42 referenced in `public_state_hash` | enumerated every `self.<field>` in `hash.rs:8306-8468` | **42 ✓**, and all 42 are genuinely *fed*, not merely referenced |
| 3 exclusions | 45 − 42 | **✓**, and all three verified sound (see L1) |
| 9 collision pairs / 18 variants | pin vs the 14 reworded arm comments + 4 uncommented | **consistent ✓** |
| 14 reworded arm comments | `grep 'arm tag'` | **14 ✓** (plus 2 prose mentions) |
| 1,252 variants / 1,097 variant fields | — | **not verifiable without execution**; floors are ~2/3 as claimed |
| 27 unordered containers / 6 files | grep, both spellings | **REFUTED as a container census** — 27 is the type-annotation count; **54** constructions across **9** files under the same roots (H1) |
| tests 4,508 → 4,524 (+16) | counted `#[test]` added: 12 in `hash_schema.rs` (21→33), 3 in the ratchet, 1 in `decision_gate.rs` | **arithmetic ✓** (not executed) |
| `hash.rs` 0 non-comment changed lines | could not `git diff` (no shell). Located every changed region: `:781-806` (const doc ¶), `:6773-6796` (impl header), 14 arm comments | **all comments by inspection**; consistent with HASH 74 unmoved, but **not independently diff-verified** |

---

## Scope / over-widening assessment

**In scope and well done**: Part A, Part B, `PARTIALLY_HASHED` + `PARTIALLY_HASHED_VARIANT_FIELDS`
(the enum half in particular — shipping without it would have left the two halves of one file
disagreeing about `embedded_effect`), and the `discriminant_collisions_are_ratcheted_at_their_known_bad_state`
ratchet.

**Defensible but at the edge**: the `GameState` / `public_state_hash` gate. It is the right idea
and it is one level *up* from the batch's subject rather than sideways; M5/M6 are its unfinished
edges, not reasons to drop it.

**Should have been deferred**: the `OOS-DP9-10` rider. It is a different seed, a different
subsystem, and it ships a census the registry now records as fact that is wrong (H1). A rider
that closes a seed on a partial scan costs more than one that stays open — this file's entire
subject is gates that report success while checking a subset. Recommend: fix the needle and
re-measure in this batch, or revert the rider and leave `OOS-DP9-10` open.

**Trim candidate**: `effect_colliding_variant_digests_are_pairwise_distinct` (L8).

---

## Suggested fix ordering

1. **H1** — widen the ratchet needle, re-measure ceilings, re-run V5 with an inferred-type probe;
   correct the census in the module docs, `OOS-DP9-10`'s row and execution notes §4.2. *(Or
   revert the rider and re-open the seed.)*
2. **H2** — tighten `token_coverage`'s `Full` arm to fail closed; add the `let _ = <binding>;`
   revert row.
3. **M3, M4** — require a `.hash_into(` per arm; reject `_` in the Named branch; two revert rows.
4. **M7, L1, L2, L3** — comment/citation corrections (all comment-only, all in scope).
5. **M5, M6** — either fix, or file as residuals on `OOS-DX7-3` / `OOS-DX7-2`; both are honest
   candidates for deferral **provided the residual is written down**, which today it is not.
6. **L4–L9** — opportunistic.
