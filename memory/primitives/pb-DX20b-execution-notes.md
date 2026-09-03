# PB-DX20b — execution notes

> `scutemob-222`, v4 queue rank 9. Closes **OOS-DX20-10** (self-labelled HIGH) and **OOS-DX20-5**.
> Registry of record: `docs/audits/decision-point-audit.md`.

---

## §0 — Stage 0, written BEFORE any production line changed

Everything in this section was committed before the first engine edit. Nothing below is
back-filled; the corrections that execution forced are recorded in later sections and the
first drafts are left standing so the deltas are readable.

### §0.1 Pre-edit baseline (measured, not remembered)

`cargo test --workspace --no-fail-fast` to a file on this branch, before any edit:

- **4,991 passed / 0 failed / 5 ignored**, **59** result-producing targets.
- This **reproduces PB-DX50's close pin exactly** (4,991 / 0 / 5, 59 targets), so the branch
  starts where the last collect said it did.
- Passing test NAME set captured to a file for the close-out set-diff (4,974 names; the gap to
  4,991 is duplicate names across targets, which a set collapses — stated so the delta arithmetic
  is not read as a discrepancy).

### §0.2 Wire prediction (predicted here, gate-computed later)

| axis | at HEAD | predicted | confidence | reason |
|---|---|---|---|---|
| `PROTOCOL_VERSION` | **40** | **41** — ONE bump | HIGH | `KeywordAbility` is a declared protocol closure root (`protocol_schema.rs:103`); `KeywordAbility::Enchant(EnchantTarget)` → `EnchantTarget::Filtered(EnchantFilter)`, so adding a field to `EnchantFilter` changes a declaration inside the closure. The fingerprint is a **shape** digest, so a new field moves it. |
| `PROTOCOL` closure type count | **98** | **98 — unchanged** | HIGH | the new field is `Vec<CardType>`; `CardType` is already in the closure via the existing `has_card_type: Option<CardType>`. No new type is reachable. |
| `HASH_SCHEMA_VERSION` | **79** | **80** — ONE bump | HIGH | `impl HashInto for EnchantFilter` (`hash.rs:1714`) gains a line, and `EnchantTarget` is hashed at `:1696` inside `Characteristics.keywords`. The declaration text moves, so the hash-schema fingerprint moves. |
| `HASH` closure type count | **131** | **131 — unchanged** | HIGH | same reason as the protocol count. |

**ONE bump each for the whole PB.** The stop-condition, stated in advance: if either gate moves
in a way the field addition does not explain, or if either does **not** move, stop and re-derive
rather than re-pinning.

Card-def edits move neither: a def is data, not a declaration, and `CardRegistry`/`CardDefinition`
are `CLOSURE_MUST_NOT_CONTAIN` members on the protocol side.

### §0.3 Site census — the memo says three; re-derived here because a site list is a floor

The v4 memo row 9 cell says *"three sites"*. Re-derived at HEAD by reading every consumer of
`EnchantTarget`/`EnchantFilter` outside the card-def corpus and the test tree:

| # | site | what it does | is it an independent arithmetic? |
|---|---|---|---|
| 1 | `casting::enchant_target_to_requirement` (`casting.rs:5640`) | lowers `EnchantTarget` → `TargetRequirement`; `Filtered(f)` maps six fields onto `TargetFilter` | **YES** — this is the cast/offer arithmetic |
| 2 | `sba::enchant_filter_matches` (`sba.rs:1052`), reached via `sba::matches_enchant_target` (`:1019`) | hand-rolled six-field predicate over `Characteristics` | **YES** — a SECOND, independent copy of the same arithmetic |
| 3 | the CR 303.4a cast gate (`casting.rs:3862-3910`) | calls `sba::matches_enchant_target` | **NO** — a consumer of site 2, not a third arithmetic |
| 4 | `sba.rs:1157-1180`, the CR 704.5m SBA | calls `sba::matches_enchant_target` | **NO** — a consumer of site 2 |
| 5 | `queries::spell_target_requirements` (`queries.rs:167`) | calls `casting::card_def_target_requirements` | **NO** — a consumer of site 1, by PB-DX20's own design |

**Correction to the memo's cell.** "Three sites" is right about the *count of places that
mention the predicate* and wrong about the shape: there are **two arithmetics and three
consumers**, and the consumers are already shared. The expressiveness gap therefore has to be
closed in **two** places or the two arithmetics drift on the new field — which is exactly the
failure mode AC 7308's *"ONE arithmetic"* clause forbids.

**Shipped design (decided at stage 0):** one `pub(crate) fn enchant_filter_to_target_filter`
does the lowering, and `sba::enchant_filter_matches` is rewritten to *call it* and then hand off
to `effects::matches_filter` — the same predicate `validate_object_satisfies_requirement` runs on
the cast path. After that there is exactly ONE place that knows what an `EnchantFilter` field
means. The controller clause stays split, because `matches_filter` takes only `Characteristics`
and controller is not a characteristic — that split is stated, not hidden, and both halves read
the same `EnchantControllerConstraint`.

`sba.rs` keeping its own *entry point* is deliberate and is a different property from the one
being unified: PB-DX20 kept the CR 303.4a gate calling the SBA's predicate so that cast-time and
SBA-time agree. That property survives — it is now *guaranteed by construction* rather than by
two hand-written copies agreeing.

### §0.4 Census predictions (to be refuted or confirmed by execution)

Predicted before the roster test was written:

- Members needing **OR over card types**: `imprisoned_in_the_moon` (`Complete`, deck-legal, LIVE)
  and `kayas_ghostform` (`partial`).
- Members needing **a controller clause only**: `breath_of_fury` (`partial`) — printed
  *"Enchant creature you control"*, declared `EnchantTarget::Creature`. **This is a THIRD census
  member and neither seed row nor the v4 memo cell names it.** It needs no new expressiveness at
  all: `EnchantFilter.controller` has existed since PB-DX20.
- **Coverage flips predicted: ZERO, on both defs, and the memo's cell is wrong about one of
  them.** Row 9 predicts *"0 flips; +1 `partial` unblocked (`kayas_ghostform`)"*.
  `kayas_ghostform`'s own `Completeness::partial` note already says the Enchant line is **"NOT
  blocked"** and names a different blocker — a trigger keyed to the *enchanted permanent's* zone
  change plus a return from graveyard-or-exile. Repairing the Enchant line does not touch that,
  so the def stays `partial`. `breath_of_fury` likewise stays `partial` (Aura re-attachment).
  `imprisoned_in_the_moon` stays `Complete` — now honestly. Predicted coverage: **1,137/1,803 =
  63.1%, unmoved, 0 flips.**

---

## §1 — What shipped

CR 702.5a: *"Enchant is a static ability, written 'Enchant [object or player].' The enchant
ability restricts what an Aura spell can target and what an Aura can enchant."*

`EnchantFilter` gains **`has_card_types: Vec<CardType>`** — the OR over card **types**, beside
the existing single `has_card_type` and the existing OR over **sub**types. It lowers onto the
**already existing** `TargetFilter.has_card_types`; no parallel OR mechanism was built.

**The structural half is the batch.** `casting::enchant_filter_to_target_filter` is now the
only place in the tree that knows what an `EnchantFilter` field means.
`sba::enchant_filter_matches`'s hand-rolled six-field predicate is **deleted**; it calls the
lowering and hands off to `effects::matches_filter` — the predicate
`validate_object_satisfies_requirement` already runs on the cast/offer path for the requirement
that same lowering produces. The controller clause stays split, and the function's own doc says
so with the reason: `matches_filter` takes only `Characteristics`, and controller is a property
of the `GameObject`.

The CR 303.4a gate's call to `matches_enchant_target` is deliberately **kept**. PB-DX20 put it
there so cast-time and SBA-time agree; that property survives and is now guaranteed by
construction rather than by two hand-written copies agreeing.

Three card defs, printed lines MCP-verified 2026-09-03, no `Completeness` marker moved:

| def | printed | was | now |
|---|---|---|---|
| `imprisoned_in_the_moon` (`Complete`, deck-legal) | Enchant creature, land, or planeswalker | `Permanent` | `Filtered[Creature, Land, Planeswalker]` |
| `kayas_ghostform` (`partial`) | Enchant creature or planeswalker you control | `Creature` | `Filtered[Creature, Planeswalker]` + `You` |
| `breath_of_fury` (`partial`) | Enchant creature you control | `Creature` | `Filtered{Creature}` + `You` |

## §2 — Where the inherited documents were wrong

### §2.1 The memo's site cell — corrected at stage 0, before any code

Row 9 says *"three sites"*. It is **two arithmetics and three consumers**, and the consumers
were already shared. §0.3 has the table. Had the batch taken the cell at its word and patched
"three sites" one at a time, the new field would have been carried in two independent copies —
which is the drift the AC's *"ONE arithmetic"* clause exists to forbid.

### §2.2 The memo's coverage cell — refuted by execution

Row 9 says *"0 flips; +1 `partial` unblocked (`kayas_ghostform`)"*. **The second half is
false, and the def's own note already said so**: its `Completeness::partial` marker reads
*"NOT blocked: 'Enchant creature or planeswalker'"* and names a different blocker — a trigger
keyed to the **enchanted permanent's** zone change, plus a return from graveyard-or-exile.
Predicted false in §0.4 before regeneration; confirmed by regeneration.

Coverage: **1,137 / 1,803 = 63.1%**, clean 1,137 / todo 519 / empty 147 — byte-identical to
PB-DX50's close. **0 flips**, exactly as predicted. Self-dating churn reverted
(`authoring-status-prev.json`'s `generated`/`git_head` only).

Because no marker moved, the `CORPUS_COMPLETE` **set** is unmoved as well as its count — so no
seeded fixture was re-dealt. That is `OOS-CARDS2-3`'s budget checked and found not owed, rather
than assumed away: PB-DX26's lesson is that a stable *count* is not a stable *deal*, and here
the membership is what was checked.

### §2.3 The census was short by one, and the axis that found it is not the seeds' axis

Neither `OOS-DX20-10`, nor `OOS-DX20-5`, nor the v4 memo cell names **`breath_of_fury`** —
printed *"Enchant creature you control"*, declared `EnchantTarget::Creature`, so the controller
clause was silently dropped. It needs **no new expressiveness**: `EnchantFilter.controller` has
existed since PB-DX20. It is repaired here rather than filed.

And a fourth correction, forced by the roster's own execution: **the population needing a
`Filtered` filter is SEVEN, not the six an OR-or-controller substring axis finds.**
`awaken_the_ancient` prints *"Enchant Mountain"* — no `" or "`, no comma, no controller clause
— and still cannot be expressed by any bare variant. A substring axis would have pinned six and
called it measured. Both populations are now pinned separately (`NEEDS_FILTER_DEFS` = 7,
`NEEDS_OR_OR_CONTROLLER_DEFS` = 6).

### §2.4 A consequence in a NEIGHBOURING batch that no document anticipated

**PB-DX49's Pair A is dead, and this batch is what killed it.** `r4a_pair_a_depends_on_oos_dx20_10`
existed *because* `imprisoned_in_the_moon` declared `EnchantTarget::Permanent`: an enchantment
is a permanent, so the Aura could legally attach to `binding_the_old_gods`
(`Enchantment — Saga`) and blank it. With the printed filter in place, the two card-type sets
are disjoint, so CR 303.4a refuses the cast and CR 704.5m detaches.

PB-DX49 wrote that row to go RED and demand re-adjudication rather than silently vacate, and it
did exactly that. Re-adjudicated, not deleted:
`r4a_pair_a_is_dead_since_oos_dx20_10_closed` **computes** the death from the intersection of
the two type sets, so widening the filter — or reverting to `Permanent` — resurrects the pair
loudly. It vacates no behavioural coverage — **but the first draft of the reason was false, and
the `/review` proved it.** The draft said *"nothing outside that one roster file names Imprisoned
in the Moon"*; `git grep -ln` at the merge base returns **five** `.rs` files, one of them
`primitives::pb_dx20_keyword_carried_target_requirements`, which named it in a LIVE `assert_eq!`
— the wrong-way-round pin this batch inverts. The conclusion survives on the narrower true
reason: that reference was a **roster pin**, not a Pair-A fixture, and no fixture anywhere ever
drove Pair A. PB-DX49's deck-legal blanker × Saga coverage rests on Pair B (`Reality Shift` ×
`Binding the Old Gods`), which never sat behind `OOS-DX20-10`. Corrected in the test's own doc
comment as well as here: **a right conclusion with a wrong premise is still a finding**, because
the premise is what the next batch reuses. The rename is disclosed as this batch's single leaver.

`ReachRow.enchant` also had to change type: `EnchantTarget::Filtered` carries `Vec`s and
`REACH_ROWS` is a `const`. It now pins a const-expressible card-type slice — which is the part
that decides reach — rather than degrading to a `{:?}` string compare.

## §3 — The gate that has to exist, and why the compiler will not do its job

**Adding a field to `EnchantFilter` produces ZERO compile errors anywhere in the workspace.**
Every construction site — engine, tests, all 1,803 card defs — uses `..Default::default()`, and
`#[serde(default)]` covers deserialization. Reported by the stage-1 runner, and **re-executed
independently by the coordinator in an isolated worktree**: with an eighth field planted,
`cargo build --workspace` printed `Finished dev profile ... in 21.18s` and every one of the ten
behavioural probes stayed green.

So `r5_every_enchant_filter_field_is_lowered` is not decoration. It parses `EnchantFilter`'s
field list out of its own declaration and compares it against the field list
`enchant_filter_to_target_filter` reads, and it is the **only** thing in the tree that catches
an unlowered field. Its failure message under the planted field, verbatim:

```
PB-DX20b r5: EnchantFilter's field list moved. live only: ["legendary"]; pinned only: [].
If a field was ADDED, lower it in casting::enchant_filter_to_target_filter (nothing else
will tell you — see this test's doc) and add it here.
```

Row **R5b** proves the second half separately: planting the field *and* adding it to the pin,
while leaving the lowering untouched, still reddens `r5` — on the *unlowered* assertion. A pin
that only checked the field list would have been satisfied by the update that hides the bug.

## §4 — Tests

**5,015 / 0 / 5** full-workspace on branch `scutemob-222`, **60** result-producing targets
(59 → 60: one new simulator test binary), `--workspace --no-fail-fast` to a file, residual list
empty. Baseline **4,991 / 0 / 5** over 59 targets, measured on this branch BEFORE any edit and
reproducing PB-DX50's close pin exactly.

**Delta itemised by test NAME by set-diffing the two run logs: 25 additions, 1 leaver, 0
removals, 0 renames-without-a-successor.** (23 at the batch's close; §10's fix cycle adds `t10`
and `t11` and changes no name.) The single leaver is
`pb_dx49_saga_blanking_roster::r4a_pair_a_depends_on_oos_dx20_10`, whose successor
`r4a_pair_a_is_dead_since_oos_dx20_10_closed` is in the additions — §2.4 has the adjudication.
Honest reading: **24 genuine additions and 1 re-adjudicated rename.**

**A measurement error caught inside this batch's own close-out, worth recording because it
produces a plausible-looking wrong answer.** The first NAME delta was taken with
`sort` + `comm` and reported **24 additions and 2 leavers**, the extras being
`pb_ef4_…dreadhorde_invasion_lifelink…` (a leaver) and `pb_ef3_…captured_survives_attacker_removal`
(an addition) — two tests this batch never touched. Both are present, once, with `... ok`, in
**both** logs. The cause is that `sort` under `en_US.UTF-8` collates by locale (punctuation
weighted differently) while `comm` compares byte-wise, so the two disagree about ordering around
`::` and `_`. Redone as a byte-exact set difference in Python. **A delta taken with `sort` +
`comm` under a UTF-8 locale is not a delta**, and it fails in the direction that invents a
removal — the single thing this close-out procedure exists to detect.

## §5 — Wire

**PROTOCOL 40 → 41 / HASH 79 → 80, ONE bump each**, both taken from the failing gates' own
output, and **both predicted in writing before any production line changed** (`21f68337`,
§0.2) — including the prediction that neither closure's type count would move, confirmed by the
gates' own text at **98** and **131**.

History rows **appended**, never edited; both `FROZEN_HISTORY_PREFIX_DIGEST`s re-pinned;
`history_is_append_only` and `frozen_prefix_is_pinned` green on both gates; `hash_schema` and
`protocol_schema` 51/51 green.

The HASH `stream_fingerprint` moved for the **v40 reason alone** (`public_state_hash` folds
`HASH_SCHEMA_VERSION` in as its first byte): `canonical_fixture()` carries no Aura with an
`EnchantTarget::Filtered` keyword, so this is the version-sentinel-byte-only case, not a
payload-bytes case. Said in the history row rather than left for a reader to work out.

### §5.1 PB-DX50's sentinel lesson recurred inside the batch that had it available

Sentinel census, taken with a **multi-line-aware** regex before the re-pin: **47 HASH + 13
PROTOCOL**, reproducing PB-DX50's corrected figures exactly.

The first re-pin pass used `(HASH_SCHEMA_VERSION\s*,\s*)79\b` and **replaced 2 of 47**. The
missing spelling is the *type suffix*: the tree writes `HASH_SCHEMA_VERSION, 79u8`, and `\b`
between `9` and `u` is not a boundary. Caught by an independent survivor scan, not by the
re-pin. That is PB-DX45's and PB-DX50's rule arriving a third time in a new disguise: **a
re-pin is only as wide as the spelling its regex matched, and "spelling" includes the literal's
type suffix.** Final survivor check used a third, differently-shaped regex; the only `79`/`39`
left in the tree are two historical prose sentences in doc comments describing the SR-era
baseline.

## §6 — Benches: MEASURED, and the honest answer is "no regression demonstrated"

Matched-set A/B against merge base `e457f931` in an isolated `git worktree` with its own
`CARGO_TARGET_DIR`. **Four runs, because two were not enough to say anything true.**

| run | code | `sba_check` | `priority_cycle_4p` | `full_turn_4p` | `board_wipe_4p` |
|---|---|---|---|---|---|
| 1 | base | 14.607-14.684 µs | 24.153-24.303 | 215.32-216.67 | 120.71-121.22 |
| 2 | HEAD | 14.824-14.927 | 23.626-23.741 | 210.33-211.74 | 116.80-117.67 |
| 3 | **HEAD again** | 14.761-14.841 | 23.497-23.696 | 211.75-212.59 | 118.17-118.96 |
| 4 | base | 15.114-15.204 | 24.077-24.257 | 213.83-214.47 | 117.03-117.57 |

Read run 1 → 2 alone and you get *"`sba_check` +1.2%, everything else 2-4% faster"* — and
"everything else 2-4% faster" is not a thing this change can cause, which is the tell that the
comparison is contaminated. Run 3 is the control this batch would not have had if it had
stopped at two: **the same code, benched twice, moves `priority_cycle_6p` −1.5%,
`full_turn_6p` +1.2% and `board_wipe_4p` +1.4%.** And run 4 puts the merge base at
**15.114-15.204 µs on `sba_check` — SLOWER than either HEAD run.** The two merge-base runs of
identical code differ from each other by **4.1%**, which is larger than any base-vs-HEAD
difference measured.

**Claim: no regression is demonstrated, and none is claimed.** Not *"within the historical
band"* — that is the phrasing PB-DX49's `/review` refuted — but *"the same-code repeatability
band measured in this session is wider than the effect"*, which is a measurement.

Two mechanism facts bound it independently of the timing, because a noisy measurement is not
by itself evidence of absence:

- **`std::mem::size_of::<KeywordAbility>()` is 88 bytes at BOTH revisions** (executed at
  `e457f931` and at `d769a2de`). `EnchantTarget` grew 56 → 80, but it is still not the largest
  `KeywordAbility` variant, so `OrdSet<KeywordAbility>` elements did **not** grow and nothing on
  the layer-walk or SBA hot path got bigger. This was the plausible global mechanism and it is
  refuted by measurement rather than by argument.
- **`crates/engine/benches/engine_perf.rs` contains zero occurrences of `Aura` or `Enchant`.**
  `enchant_filter_matches` is not called by any bench, so the one place that gained an
  allocation is off every benched path by construction.

The allocation is real and is stated rather than hidden: `enchant_filter_to_target_filter`
builds a `TargetFilter` (cloning two `Vec`s and an `Option<SubType>`) per call, where the old
`sba.rs` code read the fields in place. It runs once per `EnchantTarget::Filtered` Aura per SBA
check — **5** deck-legal `Complete` defs reach it, and the flat variants (`Creature`, `Land`),
which are 16 of the corpus's 23 declarations, do not.

## §7 — Revert matrix

**16 rows executed: 11 engine + 5 channel. All discriminating. 0 UNDISCRIMINATED.**
Delegated matrices preserved verbatim (they were written to `scratchpad/`, which is untracked)
in `memory/primitives/pb-DX20b-reverts-engine.md` and
`memory/primitives/pb-DX20b-reverts-channel.md`.

**Re-executed independently by the coordinator**, in a fresh isolated worktree with its own
`CARGO_TARGET_DIR`, rather than accepted from the reports — the standing rule after PB-DX48
found a delegated "3/3 RED" that was red on the wrong assertion:

- **R1** (`imprisoned_in_the_moon` widened back to `EnchantTarget::Permanent`) reproduces on
  **all four surfaces**: `primitives` t4/t5/t6 RED with t1/t2/t3/t6b/t7/t8/t9 green; `core`
  r1/r2/r3/r4 RED with r1b/r5 green; `mtg-simulator` c1/c4 RED with c2/c3 green (stated
  controls — both submit only offered candidates, which stay legal under a *widening*); the
  play-server HTTP probe RED.
- **R5** (an eighth `EnchantFilter` field) reproduces exactly, including its headline: the
  workspace builds with **zero errors**, all ten behavioural probes stay green, and `r5` alone
  reddens. §3 has the verbatim message.

Two structural findings the matrix produced that argument would not have:

- **R3 + R10 together show the CR 303.4a gate is one-directional.** Under R3 (the
  `matches_filter` call deleted from the SBA predicate) t4/t5 still refuse — the gate adds
  nothing in the *accepting* direction. Under R10 (its `Filtered` arm forced to `false`, the
  "detach everything" bug) t1/t2/t3 redden — it is decisive in the *refusing* direction. A later
  batch deleting it as "covered upstream" would be half right and half wrong. Recorded in the
  test file's own module doc, not only here.
- **Two reverts were not enough for the channel suite.** R-A and R-B are both *over-wide*
  reverts and neither can redden the "no printed-legal target refused" half, because c2 and c3
  submit only offered candidates and a land. Without a third, *under-wide* revert (R-C,
  `EnchantTarget::Creature` — the `OOS-DX20-5` shape) two of five channel rows would have been
  honestly UNDISCRIMINATED. R-C reddens both.

## §8 — Reachability, and what is NOT covered

Simulator channel (`crates/simulator/tests/pb_dx20b_enchant_offer_channel.rs`), on a board
carrying a creature, a land, a planeswalker, a `Sol Ring` and an `Anointed Procession`:

- `c1` asserts the offer **SET** is exactly {creature, land, planeswalker} — not a `>= 1`, so
  an over-wide *and* an under-wide offer both redden it.
- `c2` every offered candidate is accepted by the engine (no clean offer followed by a refusal).
- `c3` a land is driven end to end and asserted by **resolution effect** — the Aura's
  `attached_to` — not by an `Ok`.
- `c4` the bot path A/B, **run on both sides**: 63 journal commands, 2 rejections, and the same
  attachment host at HEAD and under R-A. The bot's *play* is unchanged; its *answer space* is
  not (3 → 5 candidate names), and the reason is stated rather than assumed —
  `plan_targets` takes the first candidate and `legal_targets_per_slot` walks in ascending
  `ObjectId`, and the lowest-id battlefield permanent is legal under both declarations. **A
  fixture where the artifact had the lowest id would have moved the pick.**

HTTP (`tools/play-server/src/main.rs`), through the real router at seed **87** — read off an
executed sweep over seeds 1..=800 against `setup::build_initial_state`, not guessed — answering
with a **non-default** candidate (the opponent's Island, not `candidates[0]`) and asserting
`Sol Ring` is absent from the offer.

**Disclosed rather than overclaimed** (PB-DX45's standard): a `play-server` session installs
from a deck, so the two-turn drive can only put Islands and a Sol Ring on the battlefield. Over
HTTP this exercises **Land** (offered, chosen, resolved) and **Artifact** (present, excluded)
and nothing else. The three untested combinations are named individually — *(Creature × HTTP)*,
*(Planeswalker × HTTP)*, *(Enchantment × HTTP)* — and all five classes are covered as an exact
SET on the simulator side through the identical query pair the browser uses.

Two further disclosures in the probes' own docs rather than only here:

- **`LegalAction::CastSpell` carries no candidate list** and never has. The plan's c1 wording
  assumed one. Both real clients *derive* it (`view.rs::action_option_view` and
  `targeting::plan_targets` each call `action_target_requirements` + `legal_targets_per_slot`),
  and c1 calls that same pair on the action `StubProvider` actually offered — so it measures the
  channel, and the substitution is documented rather than silent.
- **Zero `Complete` defs carry `CardType::Battle`** (measured by an `all_cards()` walk). Battle
  is the sixth class the old `EnchantTarget::Permanent` admitted; there is no deck-legal witness,
  so it is absent from the SET assertion and said so.

## §9 — Gates, against the FINAL tree

- `cargo test --workspace --no-fail-fast` → **5,013 / 0 / 5**, 60 targets **at the batch's own
  close**; **5,015 / 0 / 5** after §10's `/review` fix cycle, which is the figure to quote.
- `cargo clippy --workspace --all-targets -- -D warnings` → clean.
- `cargo fmt --check` → clean. `tools/check-defs-fmt.sh` → clean, 1,803 defs.
- `npm run build` → **NOT run, and that is stated rather than omitted.** `node_modules` is
  absent from this worktree. It is N/A here on the frontend axis:
  `git diff main..HEAD --numstat -- tools/play-frontend` is **empty**. `tools/` is **not** zero
  — `tools/play-server/src/main.rs` moves, entirely inside its `#[cfg(test)]` module — and the
  first draft of this line would have implied it was.
- Coverage regenerated: **1,137 / 1,803 = 63.1%**, 0 flips, churn reverted.
- Engine lines, taken from `git diff main..HEAD --numstat` against the FINAL tree rather than
  remembered mid-batch (PB-DX28's re-take MEDIUM, committed again by PB-DX48):
  `crates/engine/src` is **+146 / −52** across four files — `rules/casting.rs` +48/−19,
  `rules/sba.rs` +32/−30, and `rules/protocol.rs` +27/−2 and `state/hash.rs` +39/−1, which are
  almost entirely the two appended history rows and their `- 41:`/`- 80:` doc paragraphs.
  `crates/card-types/src` is **+18 / −0**. **`crates/view-model` and `crates/simulator/src` are
  both 0** — every consumer of the Enchant restriction lives in the engine, which was measured
  before the design was chosen rather than asserted after. `tools/play-server/src/main.rs` is
  **+386 / −0**, entirely inside its `#[cfg(test)]` module.

---

## §10 — The `/review` fix cycle

**8 findings: 2 MEDIUM, 5 LOW, 1 NIT. All eight taken, none declined.** The reviewer had a
shell and used it: two of the eight were proved by planting code and running the suite, and both
of those are real holes in this batch's own gates.

### Finding 1 (MEDIUM) — a right conclusion with a false premise, in three places

The Pair-A re-adjudication justified *"this vacates no behavioural coverage"* with *"no test or
fixture outside this roster file names `Imprisoned in the Moon`"*. **False when written**, proved
by `git grep -ln ... main -- '*.rs'`: **five** files named it at the merge base, one of them
`primitives::pb_dx20_keyword_carried_target_requirements`, in a LIVE
`assert_eq!(.., vec!["Imprisoned in the Moon"])` — which is exactly the wrong-way-round pin this
batch inverts. It was committed in `0be8d904`, **the same commit that edited the file
contradicting it**.

The conclusion survives on a narrower true reason: that reference was a **roster pin**, not a
Pair-A behavioural fixture, and no fixture anywhere ever drove Pair A. Corrected in the test's own
doc comment, in CLAUDE.md and here — **a stated reason is the half the next batch reuses, so a
right conclusion with a wrong premise is a finding, not a nit.**

### Finding 2 (MEDIUM, PROVED) — `r5` is a PRESENCE gate and the batch's narrative read it as a mapping gate

`r5` asserts `declared ⊆ lowered` **by field NAME**. A field read into the WRONG `TargetFilter`
slot keeps both names present, so `r5` is blind to it — and so is `t9`, because both the offer
path and the SBA path now consume the **same** lowering, so a wrong lowering makes them agree
perfectly. **That is this batch's own unification turning a differential probe into a blind one**,
and §3's sentence *"the only thing standing between that and a silently unenforced restriction"*
read wider than the gate's actual subject.

The reviewer swapped two lines in the lowering (`basic: f.nonbasic, nonbasic: f.basic`) and **all
23 of this batch's new tests stayed green** — while the plant is live-wrong on two deck-legal
`Complete` defs (`ossification`, `dimensional_exile` both declare `basic: true`, so they would
refuse every basic land). It was caught only by three pre-existing tests in
`mechanics_e_l/enchant.rs`, a file this batch never touches.

**Closed by `t10`**, a per-field mapping matrix: for each of the seven fields, lower an
`EnchantFilter` with only that field moved, read the real `TargetFilter` back through
`spell_target_requirements`, and assert whole-struct equality against `baseline + that one slot`.
Three plant classes executed:

| plant | `r5` | `t10` |
|---|---|---|
| SWAP `basic` ↔ `nonbasic` | green | **RED** |
| DROP (`basic: false` hard-coded) | **RED** | **RED** |
| DISCARD (`has_subtypes: …take(0)…`) | green | **RED** |

**Re-executed independently by the coordinator**, not accepted from the report: with the swap
planted, `t10` is the *only* one of the twelve primitives probes that reddens (`11 passed;
1 failed`), and `t9` stays green — the blindness reproduces exactly. Restored, re-run green,
`git diff -- crates/engine/src` empty.

**And the fix cycle corrected the FINDING's own reasoning.** The brief said a dropped field is
compile-silent for all seven, which is why every field needs a row. That is wrong — a **drop** is
caught by `r5` (executed above). The class that is silent for all seven and that only `t10`
catches is the **discard**: the `f.<name>` read is present, so `r5` is satisfied, and the value
never arrives. **That is `OOS-DX7-2`'s shape one struct over** — a coverage predicate that cannot
tell `self.f.hash_into()` from `self.f.is_some().hash_into()`. Recorded in `t10`'s own docstring.

### Finding 3 (LOW, PROVED) — `r5`'s parser took one `pub ` per line

`pub nonbasic: bool, pub sneaky_zone: bool,` on one line hid the second field: declared,
serialized, hashed, never lowered, `r5` green. Reproduced, then fixed to scan **every** `pub `
occurrence — the repair `r1b` already embodied for the printed-line parser. The mitigation is
stated rather than used as an excuse: `cargo fmt --check` rejects that declaration, so the hole is
latent, not live — **but a gate that is correct only because a different gate is green has a reach
nobody has measured.**

### Finding 5 (LOW) — the CR 704.5m TRANSITION was untested

`t6`/`t6b` build the board with the Aura already attached to a printed-illegal permanent. Nothing
exercised a **legally** attached Aura *becoming* illegal — which is the case AC 7309 names and the
ordinary way CR 704.5m fires. **Closed by `t11`**: attach to a vanilla 2/2, sweep, assert it
SURVIVES and that the host's layer-resolved types contain `Creature` at that instant; install a
Layer-4 `SetCardTypes({Artifact})` on the host (CR 613.1d); assert the resolved types actually
moved; sweep again and assert `AuraFellOff` plus the Aura in its owner's graveyard by NAME and
ZONE (CR 400.7 — the old id is dead). **Isolated by a plant rather than merely shown to redden**:
giving the SBA raw `obj.characteristics` instead of `expect_characteristics` — the exact defect
`legal_actions.rs:1276` really has (`OOS-DX20b-1`) — reddens `t11` and leaves `t6`/`t6b` green.

### Finding 6 (LOW) — `c4`'s assertion (2) was near-tautological

Imprisoned's own Layer 4 makes the enchanted permanent a **Land**, one of the three printed-legal
types, so *"the host's layer-resolved types intersect {Creature, Land, Planeswalker}"* held for
any host under any declaration. Replaced by two falsifiable claims — (2a) the host's **printed**
types are in the printed-legal set, (2b) the resolved types contain `Land`, which asserts Layer 4
actually applied instead of narrating it — **and** disclosed, because the reviewer's preferred
remedy is not achievable on this fixture and that was measured rather than assumed: with (1)
temporarily deleted, `c4` under R-A is **GREEN**, since a widening cannot move a bot that takes
the first candidate in ascending `ObjectId` order and the creature is lowest on this board. The
plants that *would* move the pick redden `c4` on its precondition instead, so (2a) is never
reached. The module-doc revert table now says `c4: RED` is carried by **(1)** in every column.

### Finding 4, 7, 8 (LOW / NIT) — record and hygiene

- **4**: both `FROZEN_HISTORY_PREFIX_DIGEST` provenance logs were re-pinned without appending a
  PB-DX20b line, so for one commit the current digest was attributed to PB-DX50's 78 → 79 re-pin.
  Every prior re-pinning batch had extended that chain. Appended to both, with the incident noted
  in the comment.
- **7**: this batch's own struck memo row 9 had **8** cells against a 7-column header, so a
  renderer drops the strike note entirely. Repaired by folding it into the wire cell (rows 5 and
  8's shape). **A correction to the finding**: it said rows 6 and 7 do the same — re-measured,
  both are 7 cells and render correctly. The audit did find a real one: **rank 5 (PB-DX47) has 6
  cells**, a MISSING cell, so its wire column vanishes. Filed as `OOS-DX20b-7` and deliberately
  **not** repaired, for `OOS-DX50-11`'s reason. *And this row's own first draft was malformed the
  same way* — it wrote the header illustratively with two unescaped pipes — caught by running the
  check on itself before committing, which is the argument for making that check a gate.
- **8**: only the `Filtered` arm was unified; the eight flat variants remain two hand-written
  copies whose agreement is asserted by a doc table, covering **16 of the corpus's 23**
  declarations, and `CreatureOrPlaneswalker` is now exactly expressible as a `Filtered` filter
  with zero users. Filed as `OOS-DX20b-6`.

### Gates after the fix cycle, re-run against the FINAL tree

- `cargo test --workspace --no-fail-fast` → **5,015 / 0 / 5**, **60** targets (+2 over the batch's
  own close: `t10`, `t11`; findings 3 and 6 edited existing tests in place, so **0 name changes and
  0 leavers** in the fix cycle). Delta vs the pre-edit baseline: **25 additions, 1 leaver, 0
  removals**.
- `clippy --workspace --all-targets -- -D warnings` clean; `cargo fmt --check` clean;
  `tools/check-defs-fmt.sh` clean (1,803 defs).
- `git diff --numstat` over `crates/engine/src`, `crates/card-types/src`, `crates/card-defs`,
  `crates/simulator/src`, `crates/view-model` and `tools/` — **empty for the fix cycle**: every
  finding was closed in test or doc surfaces, and all nine transient reverts were restored from
  byte copies and verified.
- Registry audited whole against `main`: **412 rows vs 405**, 7 new ids, **0 lost**, **no existing
  row's cell count changed**, all 7 new rows exactly 4 cells.

---

## §11 — Seeds filed, and the dispatch-hygiene-8 pass

`docs/audits/decision-point-audit.md` grepped for `OOS-DX20b` **before** filing: **0 rows**, so
nothing was double-filed (dispatch hygiene 5).

| seed | one line | severity of the fact, not the fix |
|---|---|---|
| `OOS-DX20b-1` | `legal_actions.rs:1276` reads RAW printed card types for `DeclareAttackers` eligibility; `tapped`/`Defender`/`Haste` come from the same raw struct three lines away | LIVE, pre-existing, SR-38 offer soundness |
| `OOS-DX20b-2` | adding an `EnchantFilter` field is compile-silent workspace-wide; `r5` + `t10` are the only guards, and the class is every `Default`-constructed config struct in the tree | test-infrastructure, class |
| `OOS-DX20b-3` | `curse_of_opulence.rs:20`'s blocker TODO is still false (`OOS-DX20-4`'s instance), now with a machine-checked counter-statement beside it | documentation |
| `OOS-DX20b-4` | `EnchantFilter` cannot express a ZONE restriction and `animate_dead` prints one; 0 corpus exposure, census-gated | latent |
| `OOS-DX20b-5` | a test-NAME delta taken with `sort` + `comm` under a UTF-8 locale FABRICATES a removal — the one signal the standing close-out criterion exists to catch | evidence integrity, tree-wide |
| `OOS-DX20b-6` | only the `Filtered` arm was unified; the eight flat `EnchantTarget` variants are still two hand-written copies (16 of 23 declarations) whose agreement is a doc table, and `CreatureOrPlaneswalker` is now a redundant second spelling with zero users | hygiene, unguarded |
| `OOS-DX20b-7` | the v4 memo's §4 table has a row with a MISSING cell (rank 5, PB-DX47), so its wire column vanishes — `OOS-DX50-11`'s class, one file over | doc-integrity, gate-shaped |

**Dispatch hygiene 8 executed AFTER the fix cycle, not before it** — which is the whole point of
the rule, and it caught this batch's own drift: the first draft of every headline surface said
`OOS-DX20b-1..5`, and the `/review` fix cycle filed `-6` and `-7`. Re-checked against the registry
and corrected in CLAUDE.md (two places), the v4 memo row, the registry's own closure row and the
workstream-state handoff. Also re-checked and consistent after the fix cycle:

- **dispositions**: `OOS-DX20-10` and `OOS-DX20-5` are CLOSED in the registry and are described as
  CLOSED in all four narrative surfaces; no disposition was inverted by the fix cycle.
- **counts**: tests **5,015 / 0 / 5** over 60 targets, delta **25 / 1 / 0**, quoted identically in
  CLAUDE.md's tests bullet, its Last Updated narrative and the memo row.
- **next dispatch**: **PB-DX18, v4 rank 10** in CLAUDE.md (two places), the memo banner and the
  workstream-state claims table.
- **registry integrity**, audited whole against `main`: 412 rows vs 405, 7 new ids, **0 lost**, no
  existing row's cell count changed, all 7 new rows exactly 4 cells.
