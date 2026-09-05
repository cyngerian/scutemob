# Course Correction — September 2026 — Addendum

<!-- last_updated: 2026-09-05 -->

**Status**: REVIEWED 2026-09-05; dispositions recorded in `docs/course-correction-2026-09.md` §9 (A1 accepted with a battlefield qualifier, A2 accepted and scheduled after P1, A3/A4/A5 accepted).
Written 2026-09-05 from a second, independent audit of the same tree
(`8604207e`, main, clean working tree apart from the parent draft). Nothing here
is filed as a task. Each §2–§6 section carries a **Review** box for the parent
draft's author; where accepted, the draft tasks in §7 fold into the parent's
CC-list.

**Relation to the parent draft**: this document does not restate the parent's
findings. §1 records which of its figures were re-measured and reproduced. §2–§6
are recommendations the parent does not make, or makes in a form this audit
would change, each with the evidence and the reasoning.

---

## 0. Summary

The parent draft's diagnosis is correct and its sequencing is right: context
diet, then decks, then hot-seat play, then authoring, networking last. This
addendum concurs with that order and adds five things:

| # | Recommendation | Why it is not already in the parent |
|---|---|---|
| A1 | Name the simulator's offer layer as a second legality implementation and put it under the same split-on-touch rule as the four giant functions | The parent's §5.3 covers `crates/engine` only; the recurring "clean offer, guaranteed refusal" class lives in `crates/simulator` |
| A2 | Replace the hand-written `HashInto` hasher with a derive, rather than only deleting its sentinels | The parent's CC-2 removes the symptom (45 sentinel files); the 9,155-line hasher and its source-parsing coverage gate are the cause |
| A3 | Pair every source-text gate with a behavioural probe or a written reason none can be built; demote unpaired ones to backstops | The parent keeps "any gate that catches real bugs"; the batch records show rows where only a source gate reddened, which is a coverage hole wearing a gate's clothes |
| A4 | When bannering roadmap M10–M15 HISTORICAL, reconcile the M10-pre checklist, which is wrong in both directions | The parent notes the checklist is stale; it is stale in a way that misleads the next reader about what exists |
| A5 | State explicitly what is NOT retired: fingerprint history rows, frozen-prefix digests, the fuzzer's HARD=0 ratchet, the seal gate | The parent's §3.2 lists these; this addendum restates them as a hard constraint on A2 and A3, because both touch the neighbourhood |

---

## 1. Concurrence — what was re-measured

Every figure below was taken independently on `8604207e` before the parent
draft was read. All reproduce.

| Parent claim | Independent measurement |
|---|---|
| Suite 5,316 / 0 / 5 across 72 targets | `cargo test --workspace --no-fail-fast`: 5,316 passed, 0 failed, 5 ignored, 72 result-producing targets, exit 0 |
| `resolve_top_of_stack_inner` 8,934 lines | 8,934 (brace-matched) |
| `execute_effect_inner` 6,957 lines | 6,957; the body is one `match` with 105 `Effect::` arm lines |
| `handle_cast_spell` 5,043 lines | 5,043 |
| Replacement block copied 23× | `grep -rn 'check_zone_change_replacement(' crates/engine/src`: 23 |
| CLAUDE.md 5,080 lines | 5,080; `workstream-state.md` 8,280 |
| Narrative outpaces code | Lines changed since 2026-08-01: tests 121k, `memory/` 87k, tools 26k, engine src 19k, simulator src 15k, docs 6k, CLAUDE.md 7k, card-defs 5k |
| Coverage flat, authoring stopped | `docs/authoring-status.md`: clean 1,140 / 1,803, 0 new card files in the last 30 days; clean count was 1,133 on 2026-07-26 |
| No decks, no multi-human surface | `decks/` absent; `crates/network/src` is a 4-line doc comment; `tauri-app` last touched 2026-02-20 |

Also confirmed: engine external dependencies are six (`bitflags`, `blake3`,
`imbl`, `serde`, `serde_json`, `thiserror`); no `todo!`/`unimplemented!`; 63
`unwrap`/`expect`/`panic!` sites in 94k lines; toolchain pinned; `warnings =
"deny"` in the workspace lint table.

Two review-process points in the parent are worth endorsing explicitly because
they are the largest per-batch cost drivers: rank `/review` findings and log
LOW instead of fixing it in-cycle; and cap gate-defeat work at one executed
defeat per NEW gate (the parent's change-class table already says this). The
PB-DX56 record shows eleven executed defeats in one batch.

---

## 2. A1 — The simulator's offer layer is a second legality implementation

### 2.1 Evidence

`crates/simulator/src/legal_actions.rs` is 6,878 lines. Measured in
`crates/simulator/src`:

| Read style | Count |
|---|---|
| Raw `.characteristics.` reads (printed values, no layer walk) | 43 |
| `calculate_characteristics(` (layer-resolved) | 25 |
| `queries::` (the engine's read-only query module, `rules/queries.rs`) | 18 |

The batch records name the consequence repeatedly, under the name SR-38: the
offer layer offers an action and the engine refuses it. PB-DX20, PB-DX29,
PB-DX44, PB-DX45, PB-DX50, PB-DX51 (`OOS-DX51-3`), PB-DX55 and PB-DX20b
(`OOS-DX20b-1`) each found or created one. `OOS-DX20b-1` is the clearest
statement: the `DeclareAttackers` eligible list is built from raw printed card
types three lines from a raw `status.tapped` read, so a Layer-4 type change or a
granted Defender is invisible to the offer.

### 2.2 Reasoning

Every SR-38 defect has the same mechanism: the simulator re-derives a legality
predicate the engine already owns, and the two drift. The tree already holds
the correct direction: `rules/queries.rs` (PB-DX20, PB-DX29, PB-DX43, PB-DX55)
is a read-only module the offer layer can consume so that offer and validation
are one arithmetic. PB-DX55's `check_block_pair` collapsed two hand-rolled
copies into one query and reduced `combat.rs` by 131 lines. The pattern works;
it is just not applied as a rule.

This matters for the pod-first plan specifically. P1 (hot-seat) adds human
seats to the offer layer, and P2 (authoring) makes new cards reachable through
it. Both increase the offer surface. A raw read that is dormant today becomes a
422 in front of a pod member.

### 2.3 Recommendation

1. Add `crates/simulator/src/legal_actions.rs` and `targeting.rs` to the
   parent's §5.3 split-on-touch rule: a batch that touches an offer routes it
   through `rules/queries.rs`, adding a query if none exists, and never a raw
   `obj.characteristics.` read.
2. Add a ratchet in the style of SR-25's `bare_lookup_ratchet`: a per-file
   ceiling on raw `.characteristics.` reads in `crates/simulator/src`, pinned
   at today's measured counts and lowered on touch, never raised. This is the
   one new source gate this addendum proposes, and it is paired (see A3) with
   the existing SR-38 channel probes.
3. Do not schedule a wholesale sweep. The parent's "no big-bang refactor"
   applies here equally; 43 sites is a ceiling to walk down, not a task.

**Review**: [x] reviewed by the parent author and the owner 2026-09-05 — disposition in the parent doc §9

---

## 3. A2 — Replace the hand-written hasher, not just its sentinels

### 3.1 Evidence

| Item | Measurement |
|---|---|
| `crates/engine/src/state/hash.rs` | 9,155 lines |
| `impl HashInto for` blocks in it | 151 |
| `crates/engine/tests/core/hash_schema.rs` | 4,586 lines; polices field coverage by parsing the impl bodies as source text (PB-DX7's `hashinto_impl_bodies()`, `PARTIALLY_HASHED` categories, discriminant ratchets) |
| Test files pinning the HASH or PROTOCOL literal | 45 (the parent counts 48 with a wider pattern) |
| Wire bumps since 2026-08-01 | HASH 70 → 85, PROTOCOL 32 → 44 |

Each bump re-pins every sentinel file, then survivor-scans on two axes
(`OOS-DX36-8`, `OOS-DX20b-5`, `OOS-DX18-3`). The records describe this ritual
in PB-DX18, DX20b, DX36, DX45, DX50, DX51, DX52, DX53 and DX56, and three of
them record the ritual itself going wrong (a regex too narrow, a regex too wide,
a survivor scan blind on one axis).

### 3.2 Reasoning

The hasher's own header states its purpose: deterministic field-order hashing
with explicit control over which fields contribute to the public versus the
private hash. That is exactly what a derive macro with two attributes provides
(`#[hash(skip)]`, `#[hash(private)]`), and a derive is exhaustive by
construction: a new field is hashed unless annotated, which inverts today's
failure mode (`OOS-DP9-13`, a field silently absent from the stream) into a
compile-visible one. The source-parsing coverage half of `hash_schema.rs`
exists only because a hand-written impl can omit a field; under a derive it
has nothing to check and can be deleted, along with PB-DX7's `PARTIALLY_HASHED`
bookkeeping. What remains of the gate is the part that is genuinely
load-bearing: the declaration fingerprint, the stream fingerprint over the
canonical fixture, the append-only history rows and the frozen-prefix digest.
None of those depend on how the bytes are produced.

The parent's CC-2 deletes the 45 sentinel files. That removes the re-pin
ritual. It does not remove the reason every batch that adds a state field has
to hand-edit a 9,000-line file and then prove by execution that it did so
completely. A2 removes that.

Costs, stated so they can be weighed: one wire bump, because the stream
fingerprint will move once when the derive replaces the impls (it must not move
again, and that is the acceptance test); `imbl` containers need a manual impl
each, of which there are a handful; the public/private split must be preserved
attribute-for-attribute, so the migration is a mechanical transcription of
today's impls into annotations with a diff review. It is engine-crate work and
the parent's rule is "no engine work before P1". The argument for scheduling it
in the P0/P1 gap anyway is the same one the parent makes for the context diet:
every later batch that touches state gets cheaper, and P2 authoring will touch
state.

### 3.3 Recommendation

1. Sequence: CC-2 first (delete sentinels, assert against the constant), then
   A2 as its own task, before P2 authoring waves begin.
2. Acceptance: `hash_schema` and `protocol_schema` green; exactly one new
   history row per gate; `public_state_hash` and `private_state_hash` of the
   canonical fixture and of five fuzz seeds byte-identical before and after
   except for the version byte; the source-parsing coverage tests deleted with
   the reason recorded in `docs/engine-invariants.md` SR-8.
3. A2 does not touch the declaration fingerprint, the history rows, the frozen
   prefix, or PROTOCOL. See A5.

**Review**: [x] reviewed by the parent author and the owner 2026-09-05 — disposition in the parent doc §9

---

## 4. A3 — Source-text gates are proxies; pair or demote them

### 4.1 Evidence

The parent counts 486 tests in 53 files that read engine source text. The batch
records contain rows where a revert reddened ONLY a source gate and no
behavioural probe:

| Batch | Row | What the record says |
|---|---|---|
| PB-DX52 | R6 | undoing the CR 702.16b protection fix reddened only `r7b`, a source gate; closed by `t10`, a behavioural probe written because of the row (`OOS-DX52-2`) |
| PB-DX54 | R2, R3 | each reddens one source gate and no behavioural probe; disclosed in the test's own doc; R2's probe is unbuildable behind `OOS-DX54-4` |
| PB-DX42b | R7 | restoring `OOS-DX42b-1` reddened only a vocabulary gate |
| PB-DX49 | `r7` | "exactly one predicate exists" was true and unenforced until a source gate was added; the finding itself records that the gate keys on text |

The records also show source gates being defeated by spelling: a `use` alias
(PB-DX36, PB-DX49), a commented-out call (PB-DX56, `OOS-DX32-6`), an argument
swap that compiles (PB-DX56), field order (PB-DX48 `r2`), a multi-line borrow
(PB-DX51 `r1d`), a `/* */` block (PB-DX8). Each defeat cost a re-key and a
re-executed bypass.

### 4.2 Reasoning

A source gate proves a line is spelled a certain way. It cannot prove the line
does anything, and the effort spent hardening one against spelling is effort
not spent on the behavioural probe that would make it unnecessary. The records
already say this in `OOS-DX52-2`: "a row that reddens only a source gate is
telling you the behaviour has no probe." This addendum proposes making that
sentence a rule rather than a lesson.

Some source gates are the right tool: exhaustiveness rosters over `all_cards()`
(SR-36), the keyword registry (SR-5), the seal gate (SR-3), the declaration
fingerprint (SR-8), and ratchets whose subject is a count. Those measure a
property of the source itself. The gates this addendum targets are the ones
that stand in for a behaviour: "site X calls helper Y", "no second predicate
exists", "arm Z consults field W".

### 4.3 Recommendation

1. Add to `memory/conventions.md`: a new source gate that stands in for a
   behaviour must ship with a behavioural probe that reddens under the same
   revert, or with a one-line reason in the gate's doc why no probe can be
   built and a seed ID for it. The parent's change-class table already limits
   new gates to one executed defeat; this adds the pairing.
2. No retroactive sweep. When a batch touches an existing unpaired source gate
   (re-keying it after a defeat is the usual occasion), it writes the probe
   then instead of hardening the regex.
3. A gate that has a paired probe is a backstop and does not need bypass work
   when re-keyed; the probe is the verdict.

**Review**: [x] reviewed by the parent author and the owner 2026-09-05 — disposition in the parent doc §9

---

## 5. A4 — Reconcile the M10-pre checklist before bannering it

### 5.1 Evidence

`docs/mtg-engine-roadmap.md` §M10-pre has five unticked boxes. Checked against
the tree:

| Item | Box | State at HEAD |
|---|---|---|
| Layer bypass audit fixes (9 HIGH) | unticked | W3-LC audit closed 55 sites (memory); the audit doc has no closure banner, so this is undetermined from the docs alone |
| Diagnostic events `SBAFired` / `CostCalculated` / `TriggerEvaluated` | unticked | 0 occurrences in `rules/events.rs`; genuinely absent |
| Stress-test scripts S-01..S-05 in `stress-tests/` | unticked | directory does not exist; genuinely absent |
| Resolution suspension (`ChoiceRequest` / `PendingResolution`) | unticked | shipped as PB-DP9's CR 608.2d `EffectChoiceQuestion` channel, 2026-07-27; the roadmap names types that were never built while the capability exists under other names |
| Formal LKI snapshot system | unticked | `lki_object_snapshot` has 21 call sites (SR-24, PB-LKI-CC); shipped |

### 5.2 Reasoning

The parent's CC-8 banners M10–M15 HISTORICAL. A historical banner on a
checklist that is wrong in both directions preserves the error: the next reader
sees two shipped capabilities as missing and two missing ones as equally
missing, with no way to tell them apart. The fix is ten lines and belongs in
the same commit.

### 5.3 Recommendation

Amend CC-8: before the banner, tick the two shipped items with a pointer to
where they shipped, leave the two absent items unticked with a one-line
disposition each (won't-do, or a seed ID if a pod blocker names them), and
resolve the layer-bypass row by checking the audit doc's nine sites against
HEAD once and recording the answer in the audit doc.

**Review**: [x] reviewed by the parent author and the owner 2026-09-05 — disposition in the parent doc §9

---

## 6. A5 — What must not be retired, stated as a constraint

The parent's §3.2 lists what is kept. This addendum restates four items as a
hard constraint on A2 and A3, because both proposals work in their
neighbourhood and a later reader could take "delete the coverage half of
`hash_schema.rs`" as licence for more:

- `HASH_SCHEMA_HISTORY` and `PROTOCOL` history rows, append-only, and both
  `FROZEN_HISTORY_PREFIX_DIGEST` pins. These are the only record of what each
  wire version meant.
- The declaration fingerprint and stream fingerprint gates themselves.
- The fuzzer's `[profile.fuzz]` and its HARD-equals-zero ratchet
  (`--stop-on-error` halting is the property; PB-DX56 just made it true).
- The SR-3 seal gate (`cargo build --workspace` with `GameState` sealed).

The records show these four catching real defects more consistently than any
other gate family. Nothing in A1–A4 touches them.

**Review**: [x] reviewed by the parent author and the owner 2026-09-05 — disposition in the parent doc §9

---

## 7. Draft tasks (fold into the parent's CC-list if accepted)

- **CC-15** (simulator, tests): raw-`characteristics` ratchet over
  `crates/simulator/src`, pinned at measured counts; §5.3 split-on-touch rule
  extended to `legal_actions.rs` and `targeting.rs`; rule recorded in
  `memory/conventions.md`. Acceptance: ratchet green at HEAD; lowering any
  ceiling by one reddens it; the rule text names `rules/queries.rs` as the
  destination.
- **CC-16** (engine, after CC-2, before P2): derive-based `HashInto` with
  `skip`/`private` attributes; 151 impls transcribed; source-parsing coverage
  tests deleted; one history row per gate. Acceptance as in §3.3.
- **CC-17** (doc): the pair-or-demote rule for source gates in
  `memory/conventions.md`; cross-reference from `docs/engine-invariants.md`.
- **CC-8 amendment** (doc): M10-pre checklist reconciled per §5.3 in the same
  commit as the HISTORICAL banner.

None of these changes the parent's sequencing. CC-15 and CC-17 are one
coordinator session alongside CC-1..CC-4. CC-16 is one dispatchable task.

---

## 8. How to re-measure

All commands from the repo root.

- Offer-layer read style:
  `grep -rc '\.characteristics\.' crates/simulator/src | awk -F: '{s+=$2} END{print s}'`
  and the same for `calculate_characteristics` and `queries::`.
- Hasher shape: `grep -c 'impl HashInto for' crates/engine/src/state/hash.rs`;
  `wc -l crates/engine/src/state/hash.rs crates/engine/tests/core/hash_schema.rs`.
- Sentinel files: `grep -rlE 'HASH_SCHEMA_VERSION, *[0-9]+|== *8[0-9]u8|, 85,' crates tools --include=*.rs | wc -l`.
- Source-gate-only revert rows: grep the execution notes for "only a source
  gate" / "no behavioural probe".
- M10-pre state: `grep -cE 'SBAFired|CostCalculated|TriggerEvaluated' crates/engine/src/rules/events.rs`;
  `ls test-data/generated-scripts/stress-tests`; `grep -rc lki_object_snapshot crates/engine/src`.
- Suite: `cargo test --workspace --no-fail-fast > log 2>&1; grep -E '^test result' log | awk '{p+=$4;f+=$6;i+=$8} END{print p,f,i,NR}'`.
