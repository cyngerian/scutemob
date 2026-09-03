# PB-DX20b — implementation plan

Task `scutemob-222`. v4 queue rank 9. Closes **OOS-DX20-10** (HIGH) + **OOS-DX20-5**.
Stage-0 predictions and the site census are in `memory/primitives/pb-DX20b-execution-notes.md` §0
— **read that first**; it is committed and must not be back-edited.

## §1 The defect

CR 702.5a: *"Enchant [object or player]"* restricts what an Aura may target (CR 303.4a) and what
it may remain attached to (CR 704.5m). `imprisoned_in_the_moon` (`Complete`, deck-legal) prints
*"Enchant creature, land, or planeswalker"* and declares `EnchantTarget::Permanent`, which also
admits artifacts, enchantments and battles. `sba::matches_enchant_target`'s `Permanent` arm is a
bare `true`, so both the cast gate and the CR 704.5m SBA accept an illegal attachment; PB-DX20
made the offer layer enumerate every permanent, so a human can now click an artifact.

`EnchantFilter` cannot express the printed line: it has `has_card_type` (ONE type) and
`has_subtypes` (OR over **sub**types) but no OR over card **types**. `TargetFilter` — which
`enchant_target_to_requirement` already lowers onto — **already has `has_card_types`**.

## §2 Engine change

### §2.1 `crates/card-types/src/state/types.rs`

Add to `EnchantFilter`, directly after `has_card_type`:

```rust
/// Must have at least one of these card types (OR semantics). Empty = no restriction.
/// CR 702.5a — "Enchant creature, land, or planeswalker".
/// The single-type `has_card_type` above is an independent AND conjunct; a filter may
/// carry both, and both must then hold.
#[serde(default)]
pub has_card_types: Vec<CardType>,
```

Update the struct's doc-comment example block to include the new pattern. Update
`EnchantTarget::Filtered`'s doc block likewise.

### §2.2 ONE lowering, consumed by both arithmetics — this is AC 7308

`casting.rs`: extract the `Filtered(f)` arm's body into

```rust
pub(crate) fn enchant_filter_to_target_filter(f: &EnchantFilter) -> TargetFilter
```

carrying the new `has_card_types: f.has_card_types.clone()` alongside the six existing fields
and keeping `..Default::default()` for the rest (PB-DX20 §3.4 — a future `TargetFilter` field
must not silently acquire meaning here). `enchant_target_to_requirement`'s `Filtered` arm becomes
`TargetRequirement::TargetPermanentWithFilter(enchant_filter_to_target_filter(f))`.

`sba.rs`: **rewrite** `enchant_filter_matches` to stop hand-rolling the six-field predicate:

```rust
fn enchant_filter_matches(f, chars, aura_controller, target_controller) -> bool {
    let tf = super::casting::enchant_filter_to_target_filter(f);
    if !crate::effects::matches_filter(chars, &tf) { return false; }
    match f.controller { ...unchanged... }
}
```

This is what makes the two arithmetics ONE. Justification, to be stated in the function's own
doc: `matches_filter` is the predicate `validate_object_satisfies_requirement` already runs on
the cast/offer path for the requirement this same lowering produces, and it covers all six
fields (`has_card_type`, `has_subtype`, `has_subtypes`, `basic`, `nonbasic`, and now
`has_card_types`) — verified by reading `effects/mod.rs::matches_filter`. **Controller stays
split**, because `matches_filter` takes only `Characteristics` and controller is not a
characteristic; say so in the doc rather than leaving it to be discovered.

**Do NOT delete the CR 303.4a gate's call to `matches_enchant_target`.** PB-DX20 keeps it so
cast-time and SBA-time agree; that property is now guaranteed by construction instead of by two
copies agreeing, which is strictly better, and deleting the call would trade away a different
property (see the block comment at `casting.rs:3844`).

### §2.3 `state/hash.rs`

Add `self.has_card_types.hash_into(hasher);` to `impl HashInto for EnchantFilter`, immediately
after `has_card_type`, so the hash order matches declaration order.

### §2.4 Wire

Both fingerprints will move. Take the new values **from the failing gates' own output** — never
transcribe, never invent. `PROTOCOL_VERSION` 40 → 41, `HASH_SCHEMA_VERSION` 79 → 80, ONE bump
each. Append a history row to each (never edit a shipped row), re-pin both
`FROZEN_HISTORY_PREFIX_DIGEST`s, and re-pin **47 HASH + 13 PROTOCOL** sentinels — that census is
multi-line-aware (PB-DX50's lesson: a same-line regex under-counts, and a survivor check written
with the re-pin's own regex is not a check). Verify survivors with an INDEPENDENT multi-line scan.
`history_is_append_only` and `frozen_prefix_is_pinned` must be green on both gates.

## §3 Card-def edits

| def | printed (MCP-verified 2026-09-03) | declared at HEAD | ships as |
|---|---|---|---|
| `imprisoned_in_the_moon` (`Complete`, deck-legal) | Enchant creature, land, or planeswalker | `Permanent` | `Filtered(EnchantFilter { has_card_types: vec![Creature, Land, Planeswalker], ..default })` |
| `kayas_ghostform` (`partial`) | Enchant creature or planeswalker you control | `Creature` | `Filtered(EnchantFilter { has_card_types: vec![Creature, Planeswalker], controller: You, ..default })` |
| `breath_of_fury` (`partial`) | Enchant creature you control | `Creature` | `Filtered(EnchantFilter { has_card_type: Some(Creature), controller: You, ..default })` |

`breath_of_fury` is a **stage-0 census find that no seed row and no memo cell names**. It needs
no new expressiveness — `EnchantFilter.controller` has existed since PB-DX20 — so it is repaired
here rather than filed.

Update each def's in-source comment/`Completeness` note so no note outlives the commit that
falsifies it (`OOS-DX47-6`'s shape). `kayas_ghostform`'s note currently says *"NOT blocked:
'Enchant creature or planeswalker' … the def currently declares Enchant(Creature), which wrongly
narrows legal targets and drops 'you control'"* — that sentence is now false and must be
rewritten to name only the surviving blocker (the trigger keyed to the ENCHANTED permanent's
zone change, plus the return from graveyard-or-exile).

**Completeness re-adjudication, predicted: NO flip on any of the three.** `kayas_ghostform` and
`breath_of_fury` both keep `partial` on blockers this batch does not touch;
`imprisoned_in_the_moon` keeps `Complete`, now honestly. Regenerate coverage and NAME any flip
that contradicts this.

## §4 Tests

### §4.1 `crates/engine/tests/primitives/pb_dx20b_enchant_card_type_or.rs` (new)

Behavioural probes, each with an executed revert row:

- `t1` cast: `imprisoned_in_the_moon` targeting a **creature** — accepted.
- `t2` cast: targeting a **land** — accepted.
- `t3` cast: targeting a **planeswalker** — accepted.
- `t4` cast: targeting an **artifact** — REFUSED (this is `OOS-DX20-10`'s live defect; must be
  accepted at merge-base and refused at HEAD).
- `t5` cast: targeting an **enchantment** — REFUSED.
- `t6` **CR 704.5m, the load-bearing SBA half**: attach the Aura legally to a creature, then make
  the attachment illegal *without moving the Aura* (a `LayerModification` that removes the
  Creature card type, or animate/de-animate), run SBAs, assert the Aura is in its owner's
  **graveyard**. Then the control: an attachment that stays legal is NOT detached.
- `t7` `kayas_ghostform`: the controller clause — a creature **you** control is legal, an
  opponent's creature is not, and a planeswalker you control **is** legal (which the HEAD
  declaration refuses — the narrowing half of `OOS-DX20-5`).
- `t8` `breath_of_fury`: opponent's creature refused, own creature accepted.
- `t9` **the two arithmetics agree**: for a matrix of `EnchantFilter`s × object shapes, assert
  `sba::matches_enchant_target` and `matches_filter(chars, enchant_filter_to_target_filter(f))`
  return the same answer. This is the structural pin on "ONE arithmetic" — and note in the
  docstring that it is a *consistency* pin, which PB-DX20's own durable lesson says proves
  consistency and not correctness, so `t1`-`t8` are the correctness half.

### §4.2 `crates/engine/tests/core/pb_dx20b_enchant_line_roster.rs` (new)

- `r1` **the census, PRINTED** (AC 7310): walk `all_cards()` — never grep source (SR-36) — and for
  every def carrying an `Enchant` keyword OR whose `oracle_text` contains a printed
  `Enchant …` line, parse the printed line and compare it against the declared `EnchantTarget`.
  Print the full table with `--nocapture`, and assert the set of **mismatches** is exactly the
  repaired three (i.e. empty after the repair, with a named-list failure message).
  Both axes are needed and they do not nest: a def can declare the keyword with no printed line
  (bestow) and a def can print the line with no keyword (`animate_dead`, `curse_of_opulence`).
- `r2` **inverse axis**: every def whose printed Enchant line contains `" or "` or `", "`
  (an OR over classes) or `" you control"` / `"an opponent controls"` (a controller clause) is
  named, and each must declare a `Filtered` filter that expresses it. Print the population.
- `r3` **the wrong-way-round pin from PB-DX20 is INVERTED**: `pb_dx20_keyword_carried_target_
  requirements.rs`'s roster of `Complete` Auras declaring `EnchantTarget::Permanent` must now be
  **EMPTY**. Edit that assertion in place (it names itself as the one to invert) and rewrite its
  comment so it no longer describes an open defect.
- `r4` **non-vacuity floors**, each a named count, so a corpus move is reported rather than
  silently re-tuned.
- `r5` **structural gate**: `EnchantFilter`'s field list is parsed from its own declaration in
  `card-types/src/state/types.rs` and compared against the field list
  `enchant_filter_to_target_filter` reads, so a seventh `EnchantFilter` field cannot be added
  without being lowered. (`OOS-DX28-1`'s recommended repair; PB-DX43 applied the same shape to
  `TOKEN_SPEC_FIELDS`.) Prove it red by planting a field.

### §4.3 `crates/simulator/tests/pb_dx20b_enchant_offer_channel.rs` (new)

Offer-vs-cast agreement, **both directions** (SR-38, AC 7311):

- `c1` the `LegalAction` set for casting `imprisoned_in_the_moon` on a board carrying a creature,
  a land, a planeswalker, an artifact and an enchantment offers **exactly** the first three as
  targets — assert the SET, not a `>= 1`.
- `c2` every offered target, when actually submitted through `LocalGame::submit`, is **accepted**
  (no clean offer followed by a refusal).
- `c3` no printed-legal class is refused (drive a land target end to end and assert the Aura is
  attached, not merely that the command returned `Ok`).
- `c4` bot path unchanged: a `StubProvider` A/B over the same seed, reported as measured.

### §4.4 `tools/play-server` HTTP probe (AC 7311)

A probe through genuine `POST /api/game/action` with a **non-default** answer — i.e. not the
first offered target — landing the Aura on a specific printed-legal permanent, asserting the
resolved attachment. Add it to `tools/play-server/src/main.rs`'s `#[cfg(test)]` module beside
the existing HTTP probes. If a `play-server` session cannot be made to hold the needed board,
say so explicitly and state which combination is untested (PB-DX45's disclosure standard) rather
than quietly substituting a weaker probe.

## §5 Gates, against the FINAL tree

`cargo test --workspace --no-fail-fast` to a file; delta itemised by test NAME by set-diffing
against `scratchpad/baseline-names.txt`; `clippy --workspace --all-targets -- -D warnings`;
`cargo fmt --check`; `tools/check-defs-fmt.sh`; `npm run build` **only if** `git diff main..HEAD
--numstat -- tools/play-server tools/play-frontend` is non-empty (say which, either way);
coverage regenerated via `python3 tools/authoring-report.py` with self-dating churn reverted and
flips NAMED; benches either a merge-base A/B or the words "not measured".

**Card-def edits re-deal every seeded fixture** (`OOS-CARDS2-3`) — only if `CORPUS_COMPLETE`
moves. Predicted: it does not (no marker changes). Re-observe seeded constants by an EXECUTED
sweep if any seeded pin reddens; do not re-tune a pin without running the sweep.

## §6 Revert matrix

Every new gate and probe proven red by an executed revert, recorded in the execution notes with
the verbatim failure line. Any honestly UNDISCRIMINATED row disclosed **in the test file's own
module doc**, not only in `memory/`.
