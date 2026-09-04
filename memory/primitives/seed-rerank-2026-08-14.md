# Seed Re-rank v4 — the post-2026-08-02 seeds folded into the PB-DX queue (2026-08-14, task `scutemob-212`)

<!-- last_updated: 2026-08-14 -->

> **This document is the authoritative primitive queue.** It supersedes
> `memory/primitives/seed-rerank-2026-08-02.md` **§4** (the v3 PB-DX7..DX41 queue) as the thing a
> dispatcher reads to pick the next batch. **v3's §1-§3 remain canonical** — they are the filing
> record for the seeds v3 triaged and the record of *why* PB-DX19..PB-DX41 exist, and nothing here
> re-opens them. Only v3's **§4 ranking** is superseded, and only because v3's ranks 1-13 have all
> shipped and the order past 13 no longer survives contact with HEAD.
>
> **Precedents / structural models**: `seed-rerank-2026-08-02.md` (v3, `scutemob-182`),
> `seed-rerank-2026-07-27.md` (v2, `scutemob-159`), `oos-retriage-plan-2026-07-18.md`
> (`scutemob-115`), `rider-seed-triage-2026-07-19.md` (`scutemob-142`). Same shape:
> headline → full census → chain-verification notes → ranked queue → parked → source-doc edits.
>
> **Method (binding, per `feedback_retriage_verification` / `feedback_verify_full_chain` /
> `feedback_pb_yield_calibration`, and per v3 §2's own hard-won additions)**: a closure is believed
> only when the *shipped code* says so, never a banner or a status column; a seed premise is
> re-derived from source and CR (MCP) rather than copied from the filing row; card scope comes from
> the compiled corpus enumerated by file, not from a seed's estimate; yields are discounted 2-3×.
> **Four rules this triage adds to that list, each paid for by a shipped batch:**
> 1. **Every published count carries its derivation rule**, so the next reader re-derives rather
>    than trusts. (PB-DX8's `t_reconciliation_report` lesson: *publish the figure, do not
>    transcribe it.*)
> 2. **A count is a SNAPSHOT, not a floor.** PB-DX27's "67 machine-checkable notes" reproduced as
>    **49** at HEAD by its own literal method, and *every* variant reproduction was smaller.
> 3. **A yield estimate must say what it measures.** PB-DX26's "~4-6 flips" was wrong in both
>    directions at once because repairing an already-`Complete` def flips nothing while reviewing
>    that same def's unexamined marker can flip it *down*.
> 4. **A wire cell is a prediction to be gate-checked at dispatch, and it carries a confidence.**
>    PB-DX27's brief predicted "expected wire impact NONE" and `protocol_schema_fingerprint_is_pinned`
>    refuted it — `ContinuousEffectDef.modification` is a sibling of fields already in the closure,
>    so *reachability*, not novelty, decides. Every wire cell in §4 is marked HIGH / MEDIUM / LOW.
>
> **Zero engine/simulator/tool/card-def code changed by this task.** Docs + triage only; the
> `git diff --numstat` over `crates/` and `tools/` is **empty** (§6). Tests, coverage, PROTOCOL and
> HASH are untouched **by construction**, not by measurement.
>
> **Engine baseline this triage was verified against**: HEAD `96a1fed3` (the `scutemob-211`
> collect), PROTOCOL **37** / HASH **76** (read from
> `crates/engine/src/rules/protocol.rs:401` and `crates/engine/src/state/hash.rs:839`), coverage
> **1,136/1,803 = 63.0%**, tests **4,721 / 0 / 5**.

---

## 0. Headline

Six things. Five are verification findings; one is a ranking finding.

1. **The census is 2.6× v3's and ~6× the brief's, and the cause is a document cutoff, exactly as
   it was last time.** The brief scoped "~35+ seeds". The population is **208**. v3 recorded this
   same failure about v2 — *"v2's census closed 2026-07-31; every PB-DX batch shipped
   2026-08-01"* — and then reproduced it: v3's census closed **2026-08-02**, and the recursion
   adjudication plus the entire triage-2 successor run (`scutemob-186..194`) shipped **that same
   day**. A census cutoff is a date on a document, and work does not respect it. **The fix is not
   a better cutoff; it is to derive the population by set difference against the previous census's
   own table** (§1a), which is what this triage did and what makes the number reproducible.

2. **61 of 208 seeds — 29% — have no registry row, and `dispatch hygiene 5` names the registry as
   ground truth.** v3 warned its pass C missed 10. It now misses 61, and the missing set is not
   random: it is almost exactly one era of work (SIM-4/5/6, ENG-1/2, UI-4/5/6, plus the
   adjudication) which filed into `memory/workstream-state.md` handoff prose. The cause is a
   convention nobody wrote down as a rule — `OOS-G1-1`'s note says outright that a seed closed in
   its own batch gets no row, *"the gate is the durable artefact"* — which is defensible for the
   nine such seeds and does not cover the ~50 that are **open**. Two seeds (`OOS-CARDS2-3`,
   `OOS-CARDS2-4`) are recorded CLOSED in CLAUDE.md and appear in the registry **neither open nor
   closed**, and PB-DX32's own `/review` caught that and it was never fixed
   (`pb-review-DX32.md:336`).

3. **The top of the queue is a card nobody has ever been able to play, and its seed says
   "latent".** `OOS-DX27-1` — no CR 305.6 intrinsic-mana-ability derivation — is live on **three
   deck-legal `Complete` format staples**: `urborg_tomb_of_yawgmoth`, `yavimaya_cradle_of_growth`
   and `dryad_of_the_ilysian_grove` each grant a basic land type through `AddSubtypes` and no mana
   ability ever follows, because the engine's `AddSubtypes` arm is three lines that touch
   `chars.subtypes` and nothing else. The basic lands prove it by construction: `swamp.rs:11-27`
   hand-authors `{T}: Add {B}`, which CR 305.6 says it should not need to. **And `OOS-DX27-10`
   closes for free inside it** — the double `{T}: Add {R}` under two Moons exists only because
   Blood Moon and Magus hand-author the grant no derivation supplies. §2.1.

4. **`OOS-DX27-9`'s headline — "PB-DX42b's rank premise is false" — does not hold on the axis the
   rank was computed from.** The layer-querying-condition population did move 1 → 2, and the
   second member (`the_world_tree`) is **`Completeness::partial`**. The adjudication ranked
   PB-DX42b on **7 deck-legal `Complete` pairs** under a convention it states two lines above its
   own table. The deck-legal population moved **1 → 1**. So the premise stands and the row is
   re-scoped rather than acted on (§2.2, §3.1). **The seed's durable half survives**: The World
   Tree's filter reads `Land` where the Archangel's reads `Artifact`, so the supply census does
   not carry over — a cost that lands the day `Effect::SearchLibrary` gains a count field, which
   is v3 rank 15 / **PB-DX9's own scope**. That coupling is now written down.

5. **Two independent verifications of this task reached opposite verdicts on the same gate, and
   reconciling them is worth more than either.** Asked whether `OOS-ADJ-2` ("nothing gates the size
   of the layer-querying population") is discharged by the shipped PB-DX42a rider, one pass said
   **yes** — the gate pins the population by name, states both legal exits in its own failure text,
   and **fired on its first real event**. The other said **no** — it is blind to seven of the eleven
   layer-querying `Condition` variants. Both are right about what they measured. The gate covers
   **the population as it exists** and is blind to **7 of the 11 ways it can grow**, because axis 1
   filters on one literal variant name and axis 2 needs a `TargetFilter` payload that eight of the
   eleven do not carry. *A gate written for one variant measures that variant* — this project's
   own thesis, arriving at the gate written to close the seed that predicted it. Verdict:
   **partially discharged**, re-scoped, with an ~8-line widening carried as a rider (§2.3).

6. **The ranking finding: the first band is no longer led by conjunctions.** v3's band 1 was
   dominated by defects needing two cards on the battlefield at once. This triage found four
   entries that are wrong **on their own** — three format-staple lands that produce no mana
   (finding 3), eleven `Complete` defs that auto-take a printed "you may" (`OOS-DX24-9` ≡
   `OOS-DX27-5`, the same defect filed twice and independently re-measured at 11 by two passes),
   eighteen `Complete` defs whose combat-damage trigger may be pushed twice (`OOS-DX24-4`, upgraded
   from "exposure UNMEASURED"), and seven `Complete` defs that **cannot be cast as printed at all**
   (the `OOS-DX29-3`/`-9`/`-12`/`-14` cluster, including one Spree card that is uncastable from
   both the browser *and* the bot path). PB-DX42b, whose seven pairs each need a conjunction,
   stays in band 1 and is no longer at the top of it.

**The ranking convention is unchanged** and is quoted verbatim in §4. **Honest discounted yield
across the new entries**: ~3-6 clean completeness flips (`OOS-RR4-2`, `OOS-DX26-6`+`OOS-DX27-4`
and `OOS-DX8-1`+`-3` carry nearly all of them, and two of those three are *behind an engine fix*,
not behind authoring), correctness repairs on **53 already-`Complete` deck-legal cards** — and that figure is a **sum of
the band-1 rows' measured populations with overlaps NOT deduplicated**, not a distinct-def count:
3 (rank 1) + 7 (2) + 15 (3) + 11 (4) + 3 (6) + 1 (7) + 6 (8) + 1 (9) + 1 (12) + 1 (13) + 1 (14) +
1 (15) + 1 (16) + 2 (17). **PB-DX47's 18 were excluded** because they were unconfirmed until its
probe ran; including them takes it past 70. **↻ 2026-09-02: the probe ran and CONFIRMED them** (`scutemob-218`), and the 18 re-derived at HEAD exactly, so the honest band-1 figure is now **71**. And three instrument repairs whose value is that
they stop a green suite from lying.

---

## 1. Full seed census (AC 6490)

### 1a. Scope, method, and the derivation rule for every number below

**Scope**: every `OOS-*` seed filed after v3's census closed (**2026-08-02**, task `scutemob-182`).
That cutoff is a *date on a document*, not a date on the work: v3's census closed the same day the
adjudication (`scutemob-186`) and the whole triage-2 successor run (`scutemob-187..194`) shipped, so
"after v3" and "after 2026-08-02" are the same set only because v3 never saw its same-day siblings.
This is the identical failure mode v3 recorded about v2 ("v2's census closed 2026-07-31; every
PB-DX batch shipped 2026-08-01"), **recurring one triage later** — see finding 1 in §0.

**The derivation rule, stated so it can be re-run rather than trusted.** The census population is

```
S  =  ALL  −  V3  −  LEGACY
```

where

| symbol | definition | value at HEAD `96a1fed3` |
|---|---|---|
| `ALL` | distinct IDs matching `OOS-[A-Za-z0-9]+-[0-9]+[a-zA-Z]?` over `*.md`, `*.rs`, `*.py`, `*.js`, `*.svelte` in the whole tree | **488** |
| `V3` | the distinct IDs of v3's own census table (`seed-rerank-2026-08-02.md` §1a "Totals"): `DX1-1..6`, `DX2-1..7`, `DX3-1`, `DX3b-1`, `DX4-1..6`, `DX5-1..8`, `DX6-1..5`, `M11-5..10`, `CARDS1-1..3`, `SIM1-1..4`, `CARDS2-1..11`, `SIM2-1..7`, `UI2-1..5`, `SIM3-1..5`, `UI3-1..4` | **79** (80 rows; `OOS-M11-10` was two seeds under one ID) |
| `LEGACY` | the pre-v2 families — `DP1..DP10`, `AC6/7/8`, `EAT`, `EF*`, `EWC`, `EWCD`, `LKI`, `M11-1..4`, `OS4/6/7/8/9/10`, `RS*`, `TS`, `XA`, `XA2`, `XS` | **196** |
| `S` | the v4 candidate population | **213** |

The exact command is in §6 so it can be replayed. **`ALL` is a snapshot of the tree, not of
reality**: an ID that was proposed in a plan and then dropped still matches the regex, which is why
`S` has to be filtered once more (the table immediately below) before it is a seed count.

**Five of the 213 are not seeds**, and each is a different way for a number to look like a filing:

| ID | why it is not a seed |
|---|---|
| `OOS-DX1-7` | proposed in `pb-plan-DX1.md:812`, resolved **closed-on-arrival** in `pb-review-DX1.md:185`. v3 §1a already dispositioned it; recorded again only because the grep still finds it. |
| `OOS-DX23-5` | a **conditional** filing in `pb-plan-DX23.md:827` — "file only if a fuzz/playthrough ratchet moves". The condition never fired. No filing exists. |
| `OOS-DX23-9` | named and **rejected** in `pb-DX23-execution-notes.md:733` — "Chose: extend, not file OOS-DX23-9". |
| `OOS-ENG1-5` | a deliberately skipped number, disclosed at `pb-review-ENG1.md:72`. CLAUDE.md already says "no `-5`, deliberately unused". |
| `OOS-M11-10E` | not a new seed — the **renumbered** closed equip half of `OOS-M11-10`, done by `scutemob-211` per that note's own instruction. It is a `V3` seed wearing a new ID. |

**So the census is 208 post-v3 seed IDs** — **2.6× v3's own 80** and roughly **6×** the task
brief's "~35+". The brief was not careless; the same thing happened to v3 (brief said ~40, census
found 80) and for the same structural reason, which is finding 1.

**Registry coverage of those 208 — the headline census finding.**

| | count | derivation |
|---|---|---|
| rowed in `docs/audits/decision-point-audit.md` §8.1 | **147** | `S ∩ {IDs matching `^\| \*\*(OOS-…)` in that file}`, minus `OOS-M11-10E` |
| **NOT rowed anywhere in the registry** | **61** | the complement |

**61 of 208 — 29% — are invisible to a registry-driven re-rank**, and the registry is the document
that `dispatch hygiene 5` names as ground truth. v3 warned that its pass C "misses 10 rows"
(the CARDS-2 family). It now misses **61**. The unrowed set is not random: it is almost exactly
**one era of work**, the triage-2 successor run (`scutemob-187..194` — SIM-4/5/6, ENG-1/2,
UI-4/5/6) plus the recursion adjudication (`scutemob-186`), which filed their seeds into
`memory/workstream-state.md` handoff sections and `docs/audits/mtg-characteristics-recursion-adjudication.md`
§6 and never rowed them.

**The 61 unrowed seeds, by filing home** (each verified by grepping for the ID and reading the
document that *states* the finding, not the ones that merely cite it):

| filing home | IDs | n |
|---|---|---|
| `docs/audits/mtg-characteristics-recursion-adjudication.md` §6 | `OOS-ADJ-1..6` (`-7` **is** rowed — PB-DX27 wrote it when it closed it) | 6 |
| `memory/playtest-triage-2026-08-02b.md` (the G-rows) | `OOS-G1-1`, `G2-1..3`, `G3-1`, `G3-2`, `G4-1`, `G4-2`, `G5-1..3`, `G6-1`, `G7-1`, `G8-1`, `G10-1` | 15 |
| `memory/workstream-state.md` handoffs + `pb-plan-ENG1.md` / `pb-review-ENG1.md` | `OOS-ENG1-6,-7,-8,-10` | 4 |
| `memory/primitives/pb-plan-ENG2.md` + the ENG-2 handoff | `OOS-ENG2-1..9` | 9 |
| the SIM-4 / SIM-5 / SIM-6 handoffs (+ `docs/mtg-engine-feedback-engineering.md`, `tools/play-server/README.md`) | `OOS-SIM4-1..3`, `OOS-SIM5-1..5`, `OOS-SIM6-1..6` | 14 |
| the UI-5 / UI-6 handoffs (+ feedback-engineering) | `OOS-UI5-1..4`, `OOS-UI6-1..6` | 10 |
| `memory/primitives/pb-DX28-execution-notes.md` | `OOS-DX28-9`, `OOS-DX28-10` | 2 |
| `memory/primitives/seed-rerank-2026-08-02.md` §1f | `OOS-RR3-1` | 1 |
| **total** | | **61** |

**Why it happened, and it is not sloppiness — it is a convention nobody wrote down as a rule.**
`OOS-G1-1`'s own filing note says it plainly: *"Not filed as open in
`docs/audits/decision-point-audit.md` §8.1 for that reason [it was closed in the same task]; the
gate is the durable artefact."* That convention is defensible for a found-and-fixed-same-batch
seed — and nine of the unrowed 61 are exactly that (`G1-1`, `G2-1`, `G3-1`, `G4-1`, `G4-2`,
`G5-1`, `G5-2`, `G7-1`, plus `ENG2-4`/`-5`). **It does not cover the other ~50, which are OPEN.**
The convention leaked from "closed-in-batch seeds need no row" to "handoff prose is a filing
venue", and an entire successor run went by under it.

**Two consequences a future dispatcher must know:**

1. **Grepping the registry undercounts closures as well as openings.** A reader auditing "how many
   post-v3 seeds are closed?" from §8.1 alone will miss every closed-in-batch seed, because the
   convention deletes exactly those.
2. **`OOS-CARDS2-3` and `OOS-CARDS2-4` are recorded CLOSED in CLAUDE.md and in handoffs and have
   no registry row at all** — neither open nor closed. `OOS-CARDS2-3`'s case is the sharpest,
   because **PB-DX32's own `/review` caught it and it was never fixed**:
   `memory/primitives/pb-review-DX32.md:336` says "Seed dispositions in
   `docs/audits/decision-point-audit.md` §8.1 — not done — no PB-DX32 disposition anywhere in that
   doc." A review finding about registry hygiene, taken, and then not carried into the registry.

**Two range-notation errors found while reconciling, both in *summary* prose rather than in a
filing** — recorded because a reader expanding a range gets the wrong set:

- **CLAUDE.md's SIM-6 bullet says "Seeds `OOS-SIM6-1..5`". There are six.** `OOS-SIM6-6` was filed
  by that batch's `/review` cycle (workstream-state, the SIM-6 handoff) and is outside every
  summary range in the tree.
- **The `memory/workstream-state.md` W6 row says PB-DX29 "filed `OOS-DX29-1..14`". It filed 17**
  (`OOS-DX29-1..17`, all rowed). CLAUDE.md has the right number; the W6 row does not. Corrected by
  this task's pointer sync (§6).

This is v3's own wildcard lesson recurring: *neither narrative source enumerates concretely; both
write ranges.* v3 expanded every range against the registry and caught `OOS-DX5-8` that way. The
same method here catches `OOS-SIM6-6` — and this time the registry **cannot** be the arbiter,
because `OOS-SIM6-6` is not in it either. **When 29% of the population is unrowed, range expansion
has no authority to appeal to and every range must be expanded against its own filing document.**

### 1b. Verdict distribution — how 208 seeds resolve

Every one of the 208 carries exactly one verdict. **Tiering, stated so the depth of the evidence
behind each verdict is legible rather than implied:**

- **Tier A — full HEAD code read**, cite re-resolved, population re-derived. Applied to every seed
  whose class is `correctness` / `capability` / `card yield` / `gate integrity` **and** whose
  recorded status is open. These are the rank candidates and they got the effort.
- **Tier B — status and class verified, one cite each.** Applied to seeds recorded CLOSED (the
  closure confirmed in code, per the binding method) and to seeds whose class is
  `method` / `documentation` / `evidence-integrity lesson` / `design record` — none of which can
  enter a queue.

| verdict | n | meaning |
|---|---|---|
| `CLOSED` | **25** | closure verified in shipped code, not from a status column (§1c has the four nobody recorded) |
| `QUEUE-CANDIDATE` | **45** | open, real, rankable — every one appears in §4, merged into 24 dispatchable entries per §1d |
| `RIDER` | **32** | too small to hold a dispatch slot; each names its host batch in §4 |
| `PARKED` | **63** | real, do not queue — §5 gives the reason per item |
| `DESIGN-RECORD` | **43** | a decision already taken and written down; ranking one wastes a slot |
| **total** | **208** | |

**Derivation**: one verdict per ID, assigned family-by-family from the Tier-A/Tier-B passes, then
summed. The per-family ledger below is the check — read it as the arithmetic, not as decoration,
because a distribution table with no ledger under it is exactly the kind of number this document
tells you not to trust.

| family | n | closed | queue | rider | parked | design |
|---|---:|---:|---:|---:|---:|---:|
| `DX19` | 4 | 1 | 1 | 0 | 1 | 1 |
| `DX20` | 10 | 1 | 1 | 4 | 4 | 0 |
| `DX21` | 7 | 0 | 4 | 1 | 0 | 2 |
| `DX22` | 13 | 0 | 1 | 0 | 3 | 9 |
| `DX23` | 7 | 0 | 0 | 2 | 3 | 2 |
| `DX24` | 9 | 0 | 2 | 2 | 3 | 2 |
| `DX25` | 6 | 1 | 1 | 2 | 1 | 1 |
| `DX25b` | 5 | 1 | 1 | 2 | 1 | 0 |
| `DX25c` | 6 | 1 | 1 | 0 | 4 | 0 |
| `DX26` | 8 | 1 | 3 | 1 | 2 | 1 |
| `DX27` | 10 | 0 | 4 | 4 | 1 | 1 |
| `DX28` | 10 | 0 | 3 | 0 | 4 | 3 |
| `DX29` | 17 | 0 | 6 | 8 | 2 | 1 |
| `DX32` | 10 | 1 | 1 | 0 | 0 | 8 |
| `DX7` | 3 | 1 | 0 | 0 | 2 | 0 |
| `DX8` | 8 | 2 | 2 | 0 | 3 | 1 |
| `ADJ` | 7 | 1 | 2 | 0 | 0 | 4 |
| `ENG1` | 9 | 0 | 1 | 1 | 6 | 1 |
| `ENG2` | 9 | 2 | 3 | 1 | 3 | 0 |
| `FB1` | 9 | 1 | 1 | 0 | 3 | 4 |
| `G*` | 15 | 9 | 1 | 0 | 4 | 1 |
| `RR3` | 2 | 1 | 1 | 0 | 0 | 0 |
| `SIM4` | 3 | 1 | 0 | 0 | 2 | 0 |
| `SIM5` | 5 | 0 | 1 | 2 | 2 | 0 |
| `SIM6` | 6 | 0 | 1 | 2 | 3 | 0 |
| `UI5` | 4 | 0 | 0 | 0 | 3 | 1 |
| `UI6` | 6 | 0 | 3 | 0 | 3 | 0 |
| **total** | **208** | **25** | **45** | **32** | **63** | **43** |

**The 45 queue candidates, listed so §4 can be checked against them** — every one is placed in §4, either as a ranked row, as a named rider, or in the "Deliberately NOT ranked" list with its reason.
 — `OOS-` prefixes dropped:
`DX19-2`, `DX20-10`, `DX21-1`, `DX21-2`, `DX21-4`, `DX21-7`, `DX22-8`, `DX24-4`, `DX24-9`,
`DX25-1`, `DX25b-1`, `DX25c-6`, `DX26-3`, `DX26-6`, `DX26-8`, `DX27-1`, `DX27-2`, `DX27-4`,
`DX27-5`, `DX28-1`, `DX28-5`, `DX28-6`, `DX29-1`, `DX29-2`, `DX29-3`, `DX29-9`, `DX29-12`,
`DX29-14`, `DX32-1`, `DX8-1`, `DX8-3`, `ADJ-1`, `ADJ-2`, `ENG1-9`, `ENG2-1`,
`ENG2-2`, `ENG2-3`, `FB1-1`, `G2-2`, `RR3-1`, `SIM5-1`, `SIM6-3`, `UI6-2`, `UI6-5`, `UI6-6`.
Plus the **three this task filed**: `RR4-1`, `RR4-2`, `RR4-3` (not part of the 208 — they did not
exist when the census was taken, which is itself worth noticing: **a census is a snapshot of the
moment before the triage that reads it**).

### 1c. Silently closed — found by reading code, not status text (AC 6490)

Four seeds are closed at HEAD and **no document says so**. Each was found because the method
forbids believing a status column.

| seed | recorded status | actual state at HEAD |
|---|---|---|
| **`OOS-SIM4-2`** | "narrowed" (workstream-state) | **CLOSED.** PB-DX20's `casting::aura_spell_target_requirements` is consumed by `queries.rs:71-142` `spell_target_requirements`, the single function both cast validation and the bot's `targeting.rs::plan_targets` call. No offer≠cast Aura gap remains **for either client** — more than the row claims. |
| **`OOS-DX20-7`** | open, "out of scope" | **CLOSED silently by PB-DX26.** The row asks for a roster gate over `all_cards()` pinning *Activated + `AttachEquipment` ⇒ non-empty `targets`*. That gate is in the tree: `cards1_equip_target_roster.rs`'s `roster_r1` **is** that walk (made recursive over all ten `Effect` nesting sites by PB-DX26), R1 pins 38 members, R2 asserts every member declares exactly one `TargetRequirement` with the CR 702.6a filter (`:274-330`), R3 is a non-vacuity floor. The permissive `targets.first()` guard still stands at `abilities.rs:559-563` — deliberately, per its own comment — but the class it could hide is now machine-covered. |
| **`OOS-DX26-7`** (class half) | "the CLASS is open and unmeasured" | **CLOSED by PB-DX8**, which the row itself predicted would close it and then nobody checked. `pb_dx8_oracle_decision_cross_check.rs:54` maps the printed `up to` phrase to `{UpToN}`, `:154` registers it as a ratcheted channel, `:1001-1002` asserts the mapping, members enumerated at `:585-645`. Measured 70 oracle-positive / 10 expressible, with a stated recall bound. |
| **`OOS-DX7-3`** | open | **CLOSED IN EFFECT.** The seed's complaint is "no stated exclusion list". `GAMESTATE_NOT_IN_PUBLIC_HASH` is a declared 3-entry list (`hash_schema.rs:4252`), `PARTIALLY_HASHED_GAMESTATE` is empty, and both gates have held green across **three HASH-moving batches** since. (`GameState`'s "45 fields" is **UNMEASURED** and the gate does not need it.) |

**And one seed is discharged in the direction nobody looked**: `OOS-SIM5-4` deferred an
offer-suppression filter worth **1 of 166** refusals at filing; at HEAD it is worth **0 of 105**
(§2.6), *and* its recorded blocker ("needs a new engine query") was already refuted in-source by
`targeting.rs:83-91`. It stays parked — with a better justification than it had when filed.

### 1d. Merged — one fix closes both, so they are one queue entry

| pair | why | evidence |
|---|---|---|
| **`OOS-DX24-9` ≡ `OOS-DX27-5`** | **The same defect, filed twice by two batches five days apart, and neither row cites the other.** Both are `Effect::MayPayThenEffect` being pay-when-able: DX24-9 frames it as CR 118.12 optional cost engine-chosen on `nether_traitor`; DX27-5 frames it as a DSL/corpus consistency question. ~~**Two independent measurements of this task both returned 11 deck-legal `Complete` defs** for the same population, which is what proves they are one thing.~~ **↻ CORRECTED by PB-DX45 (`scutemob-217`, 2026-09-02): the number is 10, and the identity does not rest on it.** Re-derived at HEAD by two independent routes (the `all_cards()` walk and `decision_gate.rs`'s frozen `BASELINE`, which carried exactly ten `may_pay_then_effect` entries), with no member's marker having moved since **before** this memo's census closed. The two rows ARE one defect — they name the same `Effect` variant and the same `execute_effect_inner` arm — but the evidence offered here was two agreeing wrong numbers. Filed as `OOS-DX45-2`. This memo's own §4 also proved short in a second way PB-DX45 had to repair: the SITE is `try_pay_optional_cost`, which has **two** callers, and the second (`Effect::LookAtTopThenPlace`'s `place_cost`) is live on a deck-legal `Complete` def and named nowhere. | `effects/mod.rs:4657-4702` (unconditional `try_pay_optional_cost`, no suspension); DSL doc `card_definition.rs:1786-1791` |
| **`OOS-DX29-9` + `OOS-DX29-12`** | One defect surface. DX29-9 says the right half of a split card is uncastable; DX29-12 says a fused cast cannot announce the right half's targets and PB-DX29 **gated** the fuse offer rather than shipping it. Verified: the gate (`legal_actions.rs:2752`, predicate `:2901-2906`) fires on **100% of members** because both deck-legal fuse defs declare non-empty right-half targets (`turn.rs:108`, `wear_tear.rs:52`). **So DX29-9's own text — "a human can cast Turn, or Turn+Burn, but never Burn" — is falsified by DX29-12's gate**: at HEAD only the *left half of either* is castable. Closing DX29-12 alone restores fusing and still leaves the right half alone uncastable. | |
| **`OOS-DX27-1` + `OOS-DX27-10`** | DX27-10 is a strict **sub-case**: the double `{T}: Add {R}` exists only because Blood Moon and Magus hand-author a grant that no CR 305.6 derivation supplies, and `AddManaAbility` is append-only. A derivation is idempotent by construction and lets both defs delete the grant. Ranking DX27-10 separately buys a `push_back` guard that the real fix deletes. | §2.1 |
| **`OOS-DX26-6` + `OOS-DX27-4`** | DX26-6's remaining card yield is **not** DSL work: two of its three claimed flips (`blackblade_reforged`, and `empyrial_plate` raising the analogue) are blocked on **one engine bug** — `resolve_cda_amount` resolving the controller from the *equipped creature* rather than the effect's controller (CR 108.5/611.2c), which is exactly `OOS-DX27-4`. `crown_of_skemfar` is the second def naming it. One fix, two rows. | `layers.rs:1862-1867`, repeated `:1881-1886`, `:1896-1901` |
| **`OOS-DX8-1` + `OOS-DX8-3`** | DX8-3's "72 effectively-`Complete` defs print a you-may nothing expresses" **is** DX8-1's `may` channel, exactly — re-derived from the `BASELINE` const body as `may` 72 / `up_to` 10 / `choose` 2. Separately they build the same worklist twice. | §2.7 |
| **`OOS-ENG2-1` + `OOS-ENG2-2`** | One mechanism (`PermanentTargeted` emitted at 3 of 12 announcement sites), one population (3 deck-legal `Complete` Ward defs). ENG2-2 is ENG2-1's site census, not a second finding. | §2.5 |
| **`OOS-DX20-10` + `OOS-DX20-5`** | Both need the same new `EnchantFilter` field. `kayas_ghostform` additionally needs `controller: You`, which the same edit supplies. | §4 rank 9, which carries the three-site fix census |
| **`OOS-DX32-1` + `OOS-DX22-8` + `OOS-FB1-1`** | The two live fuzz violations cannot be *diagnosed* without the crash artefact, and `bin/fuzzer.rs:328` is literally `command_history: Vec::new()`. FB1-1 is their **prerequisite**, not a separate row. | §2.8 |

### 1e. NOT queue work — design records a class column will mislead you into ranking

Forty-two seeds carry a `correctness` or `capability` class and describe a **decision already taken
and written down**. v3 flagged three of these; the population has grown by an order of magnitude
because the last twelve batches each recorded their own residuals as seeds. The largest groups:

- **`OOS-DX22-1/-2/-3/-7/-9/-10/-11/-12/-13`, `OOS-DX32-2..5/-8/-10`, `OOS-DX23-7`, `OOS-DX21-6`** —
  calibration notes and method lessons ("a plan's evidence must be re-measured", "a random bot's
  seed choice is not evidence"). Real, durable, **not work**.
- **`OOS-ADJ-3/-4/-5/-6`** — three are pre-existing architecture notes; `OOS-ADJ-3` is a
  **dispatch-time precondition** for PB-DX42b (re-word `OOS-DX19-2`, which frames a cross-layer
  bounding problem as a CR 613.8b fixpoint) and is carried on that row in §4 rather than ranked.
- **`OOS-DX25-5`, `OOS-DX24-2/-6`, `OOS-DX28-2/-7/-8`, `OOS-ENG1-4`, `OOS-G3-2`** — API-surface and
  design-residual notes, each documented at its own declaration.

**One correction to a design record, because it is the kind that rots**: `OOS-DX28-8`'s row states
its mechanism as "the cancelling `\"target\"` must be in a *different* ability". PB-DX28's own
`/review` refuted that **by execution** — one planted sentence in the **same** ability cancels
(`pb_dx28_chosen_object_roster.rs:414-417`). The in-source doc is right; the registry row is not,
and it is corrected in §6.

### 1f. Cite rot is systemic in this cohort, and that is a finding about the registry, not about these seeds

**Fourteen** of the 208 carry a line cite that no longer resolves (§2.10 tables the eight most
load-bearing; the DX21/DX23/DX24/DX25 cohort adds `OOS-DX21-1` `combat.rs:759`→`:805`,
`OOS-DX21-2` three cites moved 250-300 lines, `OOS-DX23-1` `:508-510`→`legal_actions.rs:722-724`
— now landing inside an unrelated doc comment — `OOS-DX24-1` ~170 lines, `OOS-DX25c-5` 12 lines).
**Every premise survived; only the addresses rotted.** Every symbol name still resolves, which is
`OOS-DX2-2`'s "cite by symbol, not by line" discipline earning its keep for the third triage
running — and the reason this document cites `file.rs::symbol` wherever the target is a function.

A rotted cite reads as **"not found"**, and "not found" reads as **closed**. v3 recorded that
hazard about `OOS-DX6-5` and it has now recurred fourteen times. The corrections are applied to
the rows themselves (§6), not merely listed here.

### 1g. The user-directed coordinator flag — Blood Moon + Urza's Saga (AC 6492)

`memory/workstream-state.md` → "Coordinator flags for the next re-rank" carries a **user-directed**
2026-08-13 item: rank corner case **#36** this pass. It is now **DISCHARGED**: three registry rows
filed (**`OOS-RR4-1`**, **`OOS-RR4-2`**, **`OOS-RR4-3`** — grep-confirmed absent from the registry
first, per dispatch hygiene 5, with needles `Saga`/`saga`/`RemoveAllAbilities`/`714`/`lore`, whose
only three hits are library-search rosters and one gate-walk enumeration), both work pieces ranked
in §4, and the flag annotated in place.

**The flag was substantially right and wrong in four particulars.** A coordinator's message is a
claim like any other — PB-DX7's lesson, applied to the coordinator who wrote that lesson down.

| flag clause | verdict at HEAD |
|---|---|
| CC #36 is the interaction; marked **GAP** | **HOLDS** (`corner-cases.md:462`, `corner-case-audit.md:73`) |
| "one of **4** remaining" gaps | **REFUTED — it is the ONLY one.** Census of the audit table: **35 `COVERED` / 1 `GAP`**. CLAUDE.md's "32 COVERED, 4 GAP, 0 DEFERRED" is stale by three closures. This *raises* the flag's value. |
| `urzas_saga.rs` is `partial`, chapters I/II placeholder `GainLife(0)`, no test references it | **HOLDS** (`:69`, `:29-32`, `:39-42`; 0 executable references tree-wide) |
| behind a TODO naming a **missing** "Saga gains an activated ability" primitive | **REFUTED. The primitive exists** — `LayerModification::AddManaAbility` / `AddActivatedAbility` with **four** shipped corpus users, plus `EffectFilter::Source`, `EffectDuration::WhileSourceOnBattlefield` and `Effect::ApplyContinuousEffect` (precedent: `vraska_betrayals_sting.rs:88-115` registers a Layer-6 effect from a *resolving* ability). Chapter I is authorable today with **zero engine lines**. |
| "**Both** Saga engine sites read the printed def" | **REFUTED — five behavioural sites, not two.** The flag's list was a floor (dispatch hygiene 6, third consecutive instance). |
| `blood_moon.rs` carries `RemoveAllAbilities` at Layer 6 | **HOLDS** (`:43-44`) |
| the (b) half is a live latent engine defect independent of Urza's Saga | **HOLDS, and is stronger than filed** — two *deck-legal* pairs, and a second blanking channel the flag never names |
| "route through layer-resolved abilities" | **REFUTED as a fix description.** `AbilityDefinition::SagaChapter` is never lowered into `Characteristics`, so `calculate_characteristics` cannot answer the question and PB-DX19's `characteristics_for_condition` is the wrong tool (it guards a recursion; there is none here). |
| "same neighbourhood as PB-DX42b; weigh ordering against it" | **Adjacency, not dependency.** Different subject, different mechanism, different population (`OOS-DX27-9` puts PB-DX42b's at 2, neither member a Saga). No ordering constraint either way. |
| PB-DX27's `OOS-ADJ-7` "is adjacent but does NOT touch this" | **True, and it understates in the useful direction** — `OOS-ADJ-7`'s `SetLandTypes` fix is *why* a Blood-Mooned Urza's Saga is still a Saga at all (`ALL_LAND_TYPES` holds `"Urza's"` but not `"Saga"`, CR 205.3h), i.e. still CR 714.4's exempted object. It did half the setup. |

**The measurement that decides the ranking.** Derivation rule: source grep over
`crates/card-defs/src/defs/*.rs` (1,803 defs), `Complete` == declares `Completeness::Complete`
**or** declares no `completeness` field. Stated as a **floor**, because it is a grep and not an
`all_cards()` enumeration (SR-36 prefers the latter and this triage could not run it read-only).

- Saga side: **4** defs carry `SagaChapter`; exactly **1 is deck-legal** — `binding_the_old_gods`
  (`Complete` by derive, **zero** test-tree references).
- Blanker side: **13** defs carry `LayerModification::RemoveAllAbilities`; **8** deck-legal.
- **Deck-legal pairs: two.** **Pair A** = `imprisoned_in_the_moon` × `binding_the_old_gods` — but
  it exists *only because of `OOS-DX20-10`* (that Aura declares `EnchantTarget::Permanent` for a
  printed "creature, land, or planeswalker"), so fixing that seed kills this pair. **Pair B** =
  `reality_shift` × `binding_the_old_gods` — **unconditional, no card-def defect required**: both
  `Complete` by derive, Reality Shift manifests, and CR 708.2a gives a face-down permanent no
  abilities, which neither Saga site checks while `queue_carddef_etb_triggers` in the same
  subsystem does.
- **The famous pair is NOT deck-legal** (`urzas_saga` is `partial`; `validate_deck` rejects it).

**That last line inverts the intuitive ordering, and it is the whole ranking argument**: the
*engine* piece is live today **without** the card piece, and the *card* piece is what makes the
famous case reachable and testable. So they are ranked separately, engine first — see §4.

---

## 2. Chain-verification notes (AC 6490)

Only the seeds whose verification **changed something** get a note. Everything else is verified in
§4's queue and §5's parked table, each row carrying its own `file:line`. Every population below
states the derivation rule that produced it, per the method note.

### 2.1 `OOS-DX27-1` — filed "latent", live-wrong on three format staples. The largest severity change this pass found.

The row's class column says *"correctness, class-level (**latent per-card**)"*. It is not latent.

**CR 305.6, verbatim (MCP `get_rule`)**: *"An object with the land card type and a basic land type
has the **intrinsic ability** '{T}: Add [mana symbol]', even if the text box doesn't actually
contain that text or the object has no text box."*

**The engine has no such derivation.** `Characteristics.mana_abilities` is written from exactly
four kinds of site — the def's own abilities (`rules/face.rs:110-115`, `rules/resolution.rs:887-891`),
wholesale copy (`rules/copy.rs:86`, `rules/layers.rs:1624`), wipe (`layers.rs:342`, `:1767`) and
explicit grant (`layers.rs:1783`). **None reads `chars.subtypes`.** The `AddSubtypes` arm
(`layers.rs:1661-1665`) is three lines: `for s in subtypes { chars.subtypes.insert(s.clone()); }`.
The basic lands prove it by construction — `swamp.rs:11-27` hand-authors `{T}: Add {B}` as an
`AbilityDefinition::Activated`, which CR 305.6 says it should not need to.

**Measured population — derivation rule stated.** Scan `crates/card-defs/src/defs/*.rs`
comment-stripped for a land-type-conferring `LayerModification` (`SetTypeLine` / `AddSubtypes` /
`SetLandTypes` / `SetCardTypes`) whose payload names a basic land subtype → 6 hits, minus
`awaken_the_ancient` (Mountain appears only in its `EnchantFilter`, not as a grant) = **5
conferring defs, all 5 deck-legal `Complete` by derive.** Two hand-author the mana grant; **three
do not**:

| def | confers | authors the mana grant? | `Complete`? |
|---|---|---|---|
| `blood_moon.rs` | `SetLandTypes({Mountain})` over `AllNonbasicLands` | **yes** (`:58`, `AddManaAbility`) | derive |
| `magus_of_the_moon.rs` | same | **yes** (`:57`) | derive |
| **`urborg_tomb_of_yawgmoth.rs`** | `AddSubtypes({Swamp})` over `EffectFilter::AllLands` (`:17`, `:20`) | **NO** | derive |
| **`yavimaya_cradle_of_growth.rs`** | `AddSubtypes({Forest})` over `AllLands` (`:17`, `:20`) | **NO** | derive |
| **`dryad_of_the_ilysian_grove.rs`** | `AddSubtypes(all five basics)` over `LandsYouControl` (`:34`, `:45`) | **NO** | derive |

Every land under Urborg should tap for `{B}`, every land under Yavimaya for `{G}`, every land you
control under the Dryad for any colour. **None of them can.** Three deck-legal `Complete`
format-staple defs, silently under-delivering their entire printed text, today, in the shipped
browser game.

**And `OOS-DX27-10` is a strict sub-case that closes for free.** The double `{T}: Add {R}` under two
moons exists *only* because Blood Moon and Magus hand-author the grant no derivation supplies, and
`AddManaAbility` is append-only (`layers.rs:1782-1783`, its own comment says so). A CR 305.6/305.7
derivation is idempotent by construction and lets **both** defs delete their explicit grant. Rank
the two together; do not rank `OOS-DX27-10` separately.

**Two constraints for whoever takes it.** (i) The derivation must run *after* Layer 4 resolves and
consume the **resolved** subtype set, or it re-opens the CR 613.8 Blood Moon × Urborg dependency
arm (`layers.rs:2093-2103`, re-derived and pinned by
`pb_dx27_blood_moon_type_scope.rs:508-568`). (ii) The existing gate is blind to the sub-case:
`pb_dx27_blood_moon_type_scope.rs:448` `t6_ancient_den_gains_exactly_the_granted_tap_add_red_ability`
builds a **one**-Blood-Moon fixture, so it asserts "exactly one granted ability" under the only
configuration where one is possible. A two-moon fixture is the minimum new evidence.

### 2.2 `OOS-DX27-9` — the seed that says PB-DX42b's rank premise is false. It is half-true, and as stated it is misleading.

This is the measurement the whole task was dispatched to settle, so the derivation is published in
full rather than summarised.

**Step 1 — which `Condition` variants are layer-querying.** `ContinuousEffectDef.condition` is
dispatched from exactly one place: `rules/layers.rs:685`, inside `is_effect_active` (`:628`),
calling `effects::check_static_condition` (`effects/mod.rs:10759`). That function has five explicit
arms and a `_ =>` fallback (`:10840+`) into `check_condition` (`:10208`). Of the explicit arms only
**`YouControlNOrMoreWithFilter`** calls `characteristics_for_condition` (`:10822`). Mapping every
remaining `characteristics_for_condition` call site inside `check_condition`'s range to its
enclosing `Condition::` arm adds **ten** more reachable-by-fallback variants: `YouControlPermanent`
(`:10228`), `OpponentControlsPermanent` (`:10239`), `ControlLandWithSubtypes` (`:10337`),
`ControlAtMostNOtherLands` (`:10353`), `ControlBasicLandsAtLeast` (`:10393`),
`ControlAtLeastNOtherLands` (`:10414`), `ControlAtLeastNOtherLandsWithSubtype` (`:10432`),
`ControlLegendaryCreature` (`:10445`), `ControlCreatureWithSubtype` (`:10456`),
`OpponentControlsMoreLandsThanYou` (`:10624`). **The layer-querying variant set is 11, not 1.**

**Step 2 — which `ContinuousEffectDef` literals carry `condition: Some(..)`.** Executed the shipped
gate's own whole-corpus serde walk (`cargo test -p mtg-engine --test core pb_dx42a -- --nocapture`),
which serializes every `all_cards()` def and collects every five-key
`{condition,duration,filter,layer,modification}` node at any depth, descending `And`/`Or`/`Not` to
leaves. Live output: **386** `ContinuousEffectDef` nodes (the floor pin is 382), **18** conditioned
instances across **16** distinct cards, **9** distinct leaf variants.

**Step 3 — intersect.** Of those 9 corpus variants, exactly **one** is in the 11-member
layer-querying set. Population = **2 instances, 2 cards**: `indomitable_archangel` and
`the_world_tree`, both via `YouControlNOrMoreWithFilter`. The gate pins exactly this —
`pb_dx42a_continuous_condition_roster.rs:514-537` `t5_layer_querying_set_is_pinned` asserts
set-equality against those two, and all 10 tests in the file pass.

**The correction.** `build_roster` (`:292-310`) walks `all_cards()` with **no completeness filter**.
Applying the deck-legal rule to the two members:

- `indomitable_archangel.rs` — no `completeness` field → derive-`Complete` → **deck-legal**;
- `the_world_tree.rs:73` — **`Completeness::partial(..)`**, blocked on `Effect::SearchLibrary`'s
  missing count field → **NOT deck-legal**.

So the **total corpus population** moved 1 → 2, exactly as `OOS-DX27-9` says. The **deck-legal
`Complete` population** moved 1 → **1**. And the deck-legal axis is the one the rank was argued
on: the adjudication's severity table
(`docs/audits/mtg-characteristics-recursion-adjudication.md:717-730`) scores PB-DX42b on
*"measured live-wrong population — **7 pairs**"* under a convention it states two lines earlier as
*"live-wrong on a deck-legal `Complete` path first"*. **A `partial` def cannot make a deck-legal
pair, so the 7 is unmoved.**

**Verdict: `OOS-DX27-9`'s headline — "the rank premise is false" — does not hold on the axis the
rank used.** Its **durable half survives and is the part to keep**: the adjudication §2.3 supply
census does not carry over to The World Tree's `Land` filter (it was measured for the Archangel's
**Artifact** filter, and §7 of that document already flags the 7 as "a floor *for its own
filter*"). That becomes a live cost on the day The World Tree is promoted — i.e. the day
`Effect::SearchLibrary` grows a count field, which is **v3 rank 15 / PB-DX9's own scope**. Re-scoped
accordingly in §3; the row itself is corrected rather than deleted.

**A second finding the row does not carry, and it is the sharper one**: the gate that guards
PB-DX42b's rank premise **measures a different population from the one the rank used** — all defs
vs deck-legal `Complete`. If the premise is worth gating, the gate must report both. That is a
two-line change to `t4`'s report.

### 2.3 `OOS-DX28-1` — the rider that was supposed to protect that premise is half-blind, and the half that works is an accident

`OOS-DX28-1` records `pb_dx42a_continuous_condition_roster.rs`'s hand-maintained
`TARGET_FILTER_FIELDS` fingerprint going blind corpus-wide on a routine field addition, and says
the repair pins the fingerprint "against the struct declaration read from source". **Read at HEAD,
the repair covers `ContinuousEffectDef` and not `TargetFilter`** — i.e. not the constant that went
blind. `t9_fingerprints_match_their_structs_and_cannot_collide` (`:677-739`) builds both sets
(`:678-679`), asserts they differ (`:684`), then calls the source-reading closure **exactly once**,
`declared("ContinuousEffectDef")` at `:730`, and asserts only `ce_pinned == ce_declared` at `:732`.
`grep -n "declared("` over the file returns **one** hit. The two field sets are in sync today (33
entries vs 33 `pub` fields), so nothing is currently wrong — but nothing pins it either.

**What catches a desync today, and why that is fragile.** If `TargetFilter` grows a 34th field,
`object_field_set_equals` short-circuits at `:143`, so `is_target_filter_node` (`:158`) is false
everywhere, so `subtree_contains_target_filter` (`:205`) is false everywhere, so axis 2
(`axis2_layer_querying_structural_set`, `:570`) collapses to `{}`. The only test that reddens is
`t6_two_axes_agree_on_the_conditioned_population` (`:591`), and it reddens **because axis 1 is
non-empty** — i.e. detection rests entirely on the layer-querying population being non-zero, which
is the exact quantity PB-DX42b is about. **Were the population ever to reach 0, the fingerprint
could desync with the whole file green.** Fix: three lines beside `:732`, reusing the same closure.

**The bigger hole is not the fingerprint.** Axis 1 (`:463-469`) filters on the literal string
`"YouControlNOrMoreWithFilter"`. Of the 11 layer-querying variants derived in §2.2, **eight carry no
`TargetFilter` payload** — `ControlLandWithSubtypes`, `ControlAtMostNOtherLands`,
`ControlBasicLandsAtLeast`, `ControlAtLeastNOtherLands`, `ControlAtLeastNOtherLandsWithSubtype`,
`ControlLegendaryCreature`, `ControlCreatureWithSubtype`, `OpponentControlsMoreLandsThanYou`. If a
def ever routes one of those through a `ContinuousEffectDef.condition`, **axis 1 misses it (wrong
variant name) and axis 2 misses it (no `TargetFilter`), so `t5` and `t6` are both GREEN while the
population has grown.** The module doc (`:65-79`) and `t7` (`:618-633`) pin **one** of the eight
absent. Seven are unpinned.

**So the honest statement of what the PB-DX42a rider buys**: it protects the premise against the
two layer-querying variants that carry a `TargetFilter`, and is silently blind to seven that do
not. Widening `t7`'s single-variant pin to the full eight-member set is ~5 lines and is the
cheapest way to make PB-DX42b's premise actually gated. **This is `OOS-ADJ-2` — "nothing gates the
size of the corpus population" — surviving the gate written to close it**, and it is why §4 carries
the widening as a named rider rather than assuming the rider already did its job.

### 2.4 `OOS-DX29-14` — a deck-legal `Complete` card that cannot be cast at all, and its row states no population

`insatiable_avarice.rs` declares **no `completeness` field** → deck-legal `Complete` by derive. It
carries `KeywordAbility::Spree` (`:20`) and `ModeSelection.mode_costs: Some(vec![{2}, {B}{B}])`
(`:33-42`) with `min_modes: 1`, over a base cost of `{B}`. **`mode_costs` appears zero times in all
of `crates/simulator/src`** (`grep -rn mode_costs crates/simulator/src | wc -l` → `0`), so
`effective_cast_cost_with_additional` → `auto_tap_commands_for` taps `{B}` while `casting.rs:2961-2981`
charges `{2}{B}`. Result: `InsufficientMana` on every attempt.

**And it is not browser-only.** `params.rs:333-336` falls back to
`legal_actions::spell_default_modes` (`:3541-3559`), which returns `[0]` — whose mode cost is the
`+{2}`. **The bot path is equally stuck.** Population, derivation stated: three defs carry
`KeywordAbility::Spree`; `final_showdown` and `smugglers_surprise` are `Completeness::partial`, so
the deck-legal population is **exactly 1** — but that one is *totally* uncastable, not degraded.

This is **PB-DX29's own subject matter, one variant over**: that batch's headline was
`effective_cast_cost_with_additional` reading Squad and nothing else, and it extended the function
for seven riders. `mode_costs` is the eighth site and was outside the seven. Fold it into the same
function; wire **none**, HIGH confidence.

### 2.5 `OOS-ENG2-1` + `OOS-ENG2-2` — Ward never fires on a triggered ability, and for once the filed site census is exact

CLAUDE.md has carried this pair as "successor candidate" since ENG-2 shipped. Both premises hold.
`GameEvent::PermanentTargeted` — the only event that drives Ward — is emitted at **three** sites
(`casting.rs:4793`, `abilities.rs:1450`, `abilities.rs:2043`). Of the twelve
`push_target_announcement` sites that emit the *display* event `TargetsAnnounced`, **five** emit no
Ward dispatch: `flush_sorted`'s two arms (`abilities.rs:8929`, `:9634`),
`handle_activate_forecast` (`:1817`), `handle_scavenge_card` (`:11075`) and
`handle_activate_loyalty_ability` (`engine.rs:3772`).

**The seed's five-site census is correct and complete** — worth saying out loud, because "the site
list is a floor" has held for three consecutive batches and this is the counterexample.

**Population, derivation stated**: regex `KeywordAbility::Ward\b` ∪
`WhenBecomesTarget(ByOpponent)?` over all 1,803 defs, intersected with the deck-legal rule →
**3 deck-legal `Complete` defs**: `adrix_and_nev_twincasters` (Ward 2),
`miirym_sentinel_wyrm` (Ward 2, derive), `tyrranax_rex` (Ward 4). **Zero** deck-legal defs carry
`WhenBecomesTarget`/`WhenBecomesTargetByOpponent` (the six that do are 5 `partial` + 1 `inert`),
and the Disguise/Cloak engine grant at `layers.rs:348` adds **0** deck-legal members. Narrow
population, real game outcome, cheap fix, existing deviation pin at
`pb_eng2_targets_announced.rs:384-392` that instructs the successor to **invert** rather than
delete. Budget for golden-script / fuzz parity movement, as the ENG-2 handoff warns.

> **↻ CORRECTED 2026-09-02 by `PB-DX48` (`scutemob-219`), which shipped this row. Three of the
> figures above are grep artefacts; the headline one is not.**
>
> * **The 3 deck-legal `Complete` Ward defs REPRODUCE exactly** — `adrix_and_nev_twincasters`,
>   `miirym_sentinel_wyrm`, `tyrranax_rex`, with costs 2 / 2 / 4. That is the number the rank rested
>   on and it held.
> * **"regex `KeywordAbility::Ward\b` … over all 1,803 defs" counts 5 and the truth is 4.**
>   `vein_ripper` does not declare the variant; the name appears only inside a `// TODO` explaining
>   why it *cannot*. This memo's stated derivation is a **source regex**, and SR-36's rule is to
>   enumerate `all_cards()` — the same failure `OOS-DX47-2` filed one batch earlier.
> * **"the six that do are 5 `partial` + 1 `inert`" is a source-grep count of a MIXED set.** Exactly
>   **one** def structurally declares `WhenBecomesTarget`/`-ByOpponent` (`goldspan_dragon`,
>   `partial`); the other five only MENTION the condition in a blocker comment. The conclusion —
>   **0 deck-legal** — is unaffected.
> * **"the Disguise/Cloak engine grant adds 0 deck-legal members" is TRUE about declarations and
>   FALSE about reach, and the difference is a live defect.** `KeywordAbility::Cloak` does not
>   exist — Cloak is `Effect::Cloak` — so a keyword grep measures zero and reads like a
>   measurement. `cryptic_coat` is `Complete` and deck-legal, and its ETB Cloak puts a face-down
>   permanent on the battlefield that the layer walk gives ward {2} **and no Ward triggered
>   ability**, because Ward is lowered into a `TriggeredAbilityDef` only in `state/builder.rs`.
>   Filed as **`OOS-DX48-4`**, LIVE rather than latent.
> * A second live find the census could not see at all: **`brutal_cathar`** is `Complete` and
>   deck-legal while its back face prints *"Ward—Pay 3 life"* with no Ward mechanism authored
>   (**`OOS-DX48-7`**). Only an INVERSE oracle-text axis finds it; every structural roster walks
>   `KeywordAbility::Ward` and is blind to a card whose Ward is unauthored.
>
> All five re-derived by `core::pb_dx48_announcement_site_roster`, which PRINTS them
> (`t_census_report`) rather than restating them.

### 2.6 The bot refusal surface has collapsed to exactly three seeds, one of which is 72% of it

Re-executed at HEAD (`cargo test -p mtg-simulator --test sim5_bot_cast_discipline`; seeds 0/7/42,
26 turns, 4 heuristic bots): **105 total refusals**, reproducing PB-DX29's recorded 105 exactly —
which is what makes the class split trustworthy rather than merely new.

| class | seed | count | share |
|---|---|---|---|
| `InsufficientMana` on `activate` — auto-tap covers `CastSpell` alone | **`OOS-SIM6-3`** | **76** | **72.4%** |
| blocker refusals (14 `CrossPlayerBlock` + 13 attacker-declaring-blockers) | `OOS-SIM5-3` | 27 | 25.7% |
| modal per-mode target slices unqueryable | `OOS-SIM5-5` | 2 | 1.9% |
| **residue** | — | **0** | — |

**Two things changed against the record.** The 40-refusal `activation_condition` family SIM-6
recorded is now **zero** (SIM-6 closed it). And **cast-side refusals are zero of any kind**, which
is what retires `OOS-SIM5-4`: it deferred an offer-suppression filter worth 1 of 166 refusals at
filing, and at HEAD it is worth **0 of 105**. Its recorded blocker is *also* stale, and the code
says so before any reader does — `targeting.rs:83-91` records that PB-DX20 made `Unsatisfiable`
reachable for Auras, so "needs a new engine query" no longer holds. **Parked, with a better
justification than it had when filed.**

Cite correction: `OOS-SIM6-3` cites `local_game.rs:738`; at HEAD the guard is `:1111-1114`
(`let Command::CastSpell(cast) = command else { return None }`). The filed figure "62 of 113" is
now **76 of 105** — larger in share and in count.

### 2.7 `OOS-DX8-3` — the DSL's only optionality flag is not under-used, it is INERT

The row says 5 defs in 1,803 carry an `optional` flag while 72 effectively-`Complete` defs print a
"you may" nothing expresses. Both halves reproduce. The part the row does not say is worse:
**`optional: bool` exists on exactly one `Effect` variant** (`card_definition.rs:2060`,
`LookAtTopThenPlace`), its five users all set it `true`, and the engine's **sole** consuming arm
destructures it **`optional: _`** (`effects/mod.rs:6331`). Nothing reads it.

So the ratio is not 5 : 72. For anything the engine acts on it is **0 : 72**. And `OOS-DX8-1`'s
`BASELINE` (80 entries, `pb_dx8_oracle_decision_cross_check.rs:577`) carries **`None` in the reason
slot for all 80** — zero adjudicated since the freeze — with a channel histogram re-derived from
the const body of `may` **72** / `up_to` **10** / `choose` **2**. DX8-3's 72 *is* DX8-1's `may`
channel, exactly. **Dispatch them as one batch**; separately they build the same worklist twice.

### 2.8 Two fuzz seeds are live and neither filed number reproduces — in opposite directions

One fresh run of the exact filed invocation
(`cargo run --profile fuzz --bin mtg-fuzzer -- --games 20 --seed 1 --max-turns 200`): 20/20 games
completed, 0 crashes. HARD raw **106** across **7/20** games; TRANSIENT `no_orphaned_tokens` **226**
across 12, correctly split off by PB-DX32's own classification.

| seed | filed | at HEAD | direction |
|---|---|---|---|
| `OOS-DX32-1` (`player_consistency`, undiagnosed) | 114 across 5/20 | **84** across 4/20 [12,13,14,19] | down |
| `OOS-DX22-8` (`attachment_validity`) | 11 across 3/20 | **22** across 3/20 [2,5,10] | **up, doubled** |

**They moved in opposite directions**, which is exactly what five PB-DX batches perturbing bot play
on the same seeds should do, and is exactly why neither number should have been carried forward.
`OOS-DX32-1`'s headline "26.8% of a run" reproduces against the HARD+TRANSIENT denominator (25.3%)
and is **79.2% of the HARD bucket** — the bucket that decides `--stop-on-error`.

**Both are blocked on the same missing tool, and it should be ranked as their prerequisite rather
than as a separate row**: `OOS-FB1-1` — `bin/fuzzer.rs:328` is literally
`command_history: Vec::new()`, so the crash artefact reproduces nothing and "is this a real bug or
an SBA-timing false positive" cannot be answered from it. Cite correction: `OOS-DX22-8` cites
`invariants.rs:386`; at HEAD `check_attachment_validity` is `:472`.

### 2.9 Two of the four silent closures, in detail

§1c tables all **four** seeds that are closed at HEAD with no document saying so. Two need more
than a line, because in each case the closure is *wider* than any row claims:

- **`OOS-SIM4-2` is CLOSED, not "narrowed".** Its workstream-state row says narrowed. PB-DX20's
  `casting::aura_spell_target_requirements` is consumed by `queries.rs:71-142`
  `spell_target_requirements`, which is the single function that both cast validation and the
  bot's `targeting.rs::plan_targets` call. There is no offer≠cast Aura gap left **for either
  client**, which is more than the row claims.
- **`OOS-DX7-3` is closed in effect.** `GAMESTATE_NOT_IN_PUBLIC_HASH` is a 3-entry declared
  exclusion list (`hash_schema.rs:4252`), `PARTIALLY_HASHED_GAMESTATE` is empty, and both gates
  have held green across **three HASH-moving batches** since. The seed's own complaint — "no
  stated exclusion list" — no longer describes HEAD. (`GameState`'s 45-field count is
  **UNMEASURED**; the gate does not need it.)

### 2.10 Cite drift found this pass — corrected in the rows themselves (§6)

A seed whose cite has rotted reads as "not found" and risks a false closure — v3 recorded that
about `OOS-DX6-5` and it recurred five more times.

| seed | filed cite | cite at HEAD |
|---|---|---|
| `OOS-SIM6-3` | `local_game.rs:738` | **`:1111-1114`** |
| `OOS-DX22-8` | `invariants.rs:386` | **`:472`** |
| `OOS-ENG1-1` | `effects/mod.rs:9276` | **`:9779-9781`** |
| `OOS-ENG1-2` | `effects/mod.rs:5518-5560` | **`:5998-6060`** |
| `OOS-ENG1-9` | `resolution.rs:120-142` | **`:143`** (`*state = restart_point;`) |
| `OOS-DX27-3` | `abilities.rs:7197`, call sites `:3754/:3760` | **`:7219`**, `:3755`/`:3761` |
| `OOS-DX27-4` | `layers.rs:1861-1867` | **`:1862-1867`** (repeats at `:1881-1886`, `:1896-1901`) |
| `OOS-DX29-13` | (no line given) | **`legal_actions.rs:2570-2581`** |

### 2.11 `OOS-RR3-1` — re-measured, and the memo's own method is wrong by one

v3 published **965 of 1,803** defs that never declare a `completeness` marker, with the standing
instruction "re-measure rather than cite it". Paid twice now.

- **963** at HEAD by the memo's literal method (the file "never mentions `completeness` at all").
- **964** at HEAD by the semantically correct field-position method (`^\s*completeness:\s*Completeness::`).

The gap is one def — `misdirection.rs`, whose only occurrence of the word is in a **comment**. So
the memo's method **under-counts the derive population by one**, and a successor re-running it gets
a number that is right for the wrong reason. Trend 966 (2026-08-01) → 965 (2026-08-02) → **963**:
monotone down, so it behaves as a **ceiling**, not a floor. The field-position rule is the one to
inherit; it is also the rule that reproduces `1,136 / 1,803 = 63.0%` byte-for-byte against
`tools/authoring-report.py`, which is what validates it rather than merely asserting it.

---

## 3. Disposition of v3's standing rows (AC 6491)

v3's §4 ranks **1 through 13 are all shipped** (PB-DX19, DX20, DX21, DX22, DX23, DX24, DX25,
DX25b, DX25c, DX26, DX7, DX8, DX27, DX28, DX29 — fifteen batches across thirteen rank slots,
because 7b/7c were inserted). Everything below re-verifies the **unshipped** rows against HEAD.

### 3.1 PB-DX42b — explicitly re-decided, not carried (AC 6491)

The task brief requires this row be re-decided rather than carried forward, because `OOS-DX27-9`
records its rank premise as false and `OOS-DX28-1` records the gate that was supposed to protect
that premise going blind. Both were re-measured (§2.2, §2.3). **The decision is: PB-DX42b keeps its
scope and its band, on a corrected premise, with the gate widening attached as a named rider.**
The reasoning, in the order the evidence forced it:

1. **The falsification does not hold on the axis the rank used.** `OOS-DX27-9` is right that the
   *total corpus* layer-querying-condition population moved 1 → 2. It is silent on the fact that
   the second member (`the_world_tree`) is **`Completeness::partial`**, and the adjudication ranked
   PB-DX42b on **deck-legal `Complete` pairs** — 7 of them, `indomitable_archangel` × seven
   Artifact-moving supply cards — under a convention it states two lines above the table. The
   deck-legal population is **1 → 1**. **The 7 is unmoved, so the rank computed from it stands.**
2. **What the seed got right is worth keeping and is re-scoped rather than deleted.** The
   adjudication §2.3 supply census was measured for the Archangel's **Artifact** filter and does
   **not** carry over to The World Tree's **Land** filter — §7 of that document already calls the 7
   "a floor *for its own filter*, and the filter is narrow". That cost lands on the day The World
   Tree is promoted, which is the day `Effect::SearchLibrary` gains a count field — **v3 rank 15 /
   PB-DX9's own scope**. So the two rows are coupled, and the coupling is now written down:
   **if PB-DX9 ships before PB-DX42b, re-measure PB-DX42b's supply census before dispatching it.**
3. **A rank premise that has to be re-litigated by hand every triage is an ungated premise —
   and `OOS-ADJ-2` is PARTIALLY discharged, which two independent verifications of this task
   read in opposite directions.** One pass concluded ADJ-2 **discharged** (the gate pins the
   population *by name*, states both legal exits in its own failure text, and **fired on its
   first real event** — PB-DX27's The World Tree forced exit (b) rather than joining silently,
   which is precisely the hazard the seed predicted). The other concluded it **not discharged**
   (the gate is blind to seven of the eleven layer-querying `Condition` variants). **Both are
   right about what they measured, and reconciling them is the finding**: `t5`
   (`pb_dx42a_continuous_condition_roster.rs:514-537`) filters axis 1 on the literal string
   `"YouControlNOrMoreWithFilter"` (`:463-469`); axis 2 catches any variant carrying a
   `TargetFilter`; and `t7` (`:618-633`) pins exactly **one** of the eight non-`TargetFilter`
   layer-querying variants absent — with a failure message that says, in-source, why the other
   structural signal cannot see it. So the gate covers **the population as it exists** and is
   blind to **7 of the 11 ways it can grow**. This is the batch-level thesis recurring at the
   gate level: *a gate written for one variant measures that variant.* Verdict: **ADJ-2 stays
   OPEN, re-scoped to the seven unpinned variants**, and the widening (`t7` from a one-variant
   pin to the eight-member set; three lines beside `t9:732` for the `TargetFilter` half, which
   §2.3 shows is the constant that actually went blind) is **~8 lines total**, carried in §4 as a
   **rider that may ride any batch touching the engine tests** — not as a reason to move
   PB-DX42b.
4. **What does move it, slightly, is the conjunction argument that was always in the row.** Every
   one of the 7 pairs needs both cards on the battlefield, same controller, with the mis-typed
   permanent pivotal to a count of three, and four of the seven are Auras needing a host of the
   right printed type as well. Measured against the new band-1 members this triage found — three
   format-staple lands that are wrong **on their own, unconditionally** (§2.1), and a Spree card
   that cannot be cast **at all** (§2.4) — PB-DX42b sits below them. It is still band 1; it is no
   longer the top of it.

**Three corrections applied to the source rows themselves rather than only recorded here** (§6):
`OOS-DX27-9`'s registry row gains the deck-legal-vs-total distinction and loses its "the rank
premise is false" framing; `OOS-DX28-1`'s row gains the `t9`-covers-the-wrong-struct finding; and
`OOS-ADJ-2` is recorded **PARTIALLY DISCHARGED and re-scoped** in the adjudication's §6 rather than left to read
as closed by the rider that cites it.

**Not re-decided, and stated so it is not mistaken for an omission**: the adjudication's
`OOS-ADJ-3` instruction — *re-word `OOS-DX19-2`'s "CR 613.8b dependency-aware fixpoint" framing
before any PB-DX42b dispatch, because CR 613.8a(a) confines dependency to a single layer and the
live case is cross-layer* — is **still outstanding** and is still a dispatch-time precondition. It
is a wording fix to a registry row, not queue work, and it is carried on PB-DX42b's row in §4.

### 3.2 The other twenty standing rows, re-verified (AC 6491)

**Nothing is fully closed. All 21 standing premises reproduce at HEAD.** That is the first result
and it is worth stating plainly: twelve batches shipped between v3 and this triage and none of them
incidentally closed a queued row. What they did instead is make **three rows cheaper and three rows
more expensive**, refute **two populations**, and falsify **two wire predictions in opposite
directions**.

> Rank column = v3's. `pop` re-derived by this task with the deck-legal rule (declares
> `Completeness::Complete` or declares no `completeness` field), corpus 1,803.

| v3 rank | batch | premise at HEAD | population re-measured | wire re-checked (conf.) | pressure |
|---|---|---|---|---|---|
| 14 | PB-DX18 | **all 6 seeds LIVE** | `ShuffleIntoOwnerLibrary` **1 def**, `known_wrong`, 0 deck-legal. Miracle 3 defs / 2 deck-legal, but `grep ChooseMiracle` in simulator+tools = **0**. **`OOS-M11-5`'s blast radius is bigger than filed** — `casting.rs:4788-4798` fires `PermanentTargeted` → Ward, **3 deck-legal `Complete`** Ward defs | **HASH only** (HIGH) — `protocol_schema.rs:116-117` `CLOSURE_MUST_NOT_CONTAIN` blocks `GameState`; the PB-DX21 precedent | **UP** |
| 15 | PB-DX9 | all 5 LIVE | `OOS-DP9-3`: 7 defs, **2** with the missing count as sole blocker (`tooth_and_nail`, `buried_alive`, both `partial`). `OOS-DX4-5`: **5**, all deck-legal — so **0 flips**, in-place repair. `OOS-DP9-2` floor 2 → **4** | DP9-3 **PROTOCOL+HASH** (HIGH); **DX4-5 now none** (HIGH) — PB-DX28's `EffectChoiceQuestion::ChooseObject { up_to: true }` already expresses it | **UP** |
| 16 | PB-DX10 | LIVE — `abilities.rs:9613`/`:9616` are still two identical branches | **REFUTED: 2 deck-legal `Complete`, not 4** (`felidar_retreat`, `retreat_to_kazandu`) of 7 modal triggered defs. The headline `hullbreaker_horror` is *gated*, so `min_modes: 0` has **0** deck-legal members, and **`OOS-DP8-3` has 0 corpus members at all** (`grep Modular` in card-defs = 0) | PROTOCOL+HASH (HIGH) | **DOWN** |
| 17 | PB-DX30 | LIVE — zero SBA check on `handle_pass_priority` (`engine.rs:2325-2344`); 11 existing check sites, not 4 | **the row's "22 `Complete` sac-for-mana defs" is a token-creator census wearing a sac-for-mana label.** Literal reading of the row's own words = **1**. Honest floors: **82** deck-legal with an activation-time sacrifice cost, **13** with `spell_additional_costs` | PROTOCOL none (HIGH); **HASH likely MOVES** (MED) via a new hashed resume site (`hash.rs:3442`) — the row says none | **DOWN** |
| 18 | PB-DX31 | all 4 LIVE | the ratchet figures 36/20/9 are **`(def × ability)` rows over all 1,803 with no completeness filter** (`sim2_mana_source_roster.rs:37-62`). Deck-legal: **12** mana-component, **9** scaled | none (HIGH) | STAY, mild UP |
| 20 | PB-DX33 | LIVE and **widened** | **9 hand-built sites, not 5** (`input.rs:54,82,117,136,180,228,631,649,687`); 1 routed. **17 of 26 `LegalAction` variants unreachable from the TUI**; PB-DX29 widened it by 2 | none (HIGH) | STAY (scope UP, urgency DOWN) |
| 21 | PB-DX34 | all 3 LIVE | **340 occurrences / ~335 constructions, 330 of them in tests** — not "337 sites". `propaganda` + `ghostly_prison` are both deck-legal `Complete` but carry `x_count: 0`, so the tax path has **0 live reach** | PROTOCOL (HIGH) **+ a HASH bump the row omits** for `GameRestriction` (`hash.rs:2959`) | **DOWN + SPLIT** |
| 22 | PB-DX35 | both LIVE; `OOS-DX4-5` **cheapened** | **2 real flips** (`shambling_ghast`, `hullbreaker_horror`) **+ 1 live-wrong deck-legal `Complete`** (`retreat_to_kazandu`). DX4-5's 5 are all already `Complete` → 0 flips there | DX4-2 none-if-registry / both-if-lowered (MED); **DX4-5 now none** (HIGH) | **UP** |
| 23 | PB-DX36 | **LIVE and worse than filed**: `combat_only` is read in **exactly one place and that place is the hasher** (`hash.rs:6566`). No dispatcher reads it, so `true` and `false` are behaviourally identical | **1 deck-legal `Complete`** (`sigil_of_sleep`, marker-less) of 3 Aura-trigger defs. `WhenDealsDamage` is named by **exactly 1 def** (`exalted_angel`, `partial`) → **1 flip**, not "1-2 + family" | **both** (HIGH), but only via `EffectAmount` — `TriggerCondition` is **off-wire** (`protocol.rs:258-260`), which the row gets backwards | **UP** |
| 24 | PB-DX11 | all 3 LIVE; PB-DX23 changed nothing | **5** draw-replacement-blocked defs, not 3 — all `inert`, **none deck-legal**, and only **2** reachable by the stated minimum widening. **0 `WouldDraw` defs corpus-wide** | PROTOCOL+HASH (HIGH) | STAY / slight DOWN |
| 25 | PB-DX12 | LIVE; enforcement surface grew from one type to **~11 files** since filing | of 4 named, **2 have no def file** (`westvale_abbey`, `ormendahl_profane_prince`) → **3 in-place flips + 1 new authoring** | PROTOCOL+HASH (HIGH) | **DOWN** |
| 26 | PB-DX13 | both LIVE | 3 named + `naya_charm`; **all non-`Complete`**, so all in-place flips; discount to 2 defensible | PROTOCOL+HASH (HIGH) | STAY (mild UP over 25) |
| 27 | PB-DX37 | all 4 read sites LIVE verbatim | **premise dormant, measured**: 23 production creation sites, **byte-identical per-file to PB-DX5's collect `f20823b1`** — PB-DX27's `SetLandTypes` and PB-DX28's `TargetOwner`/`ChoiceZone` moved it by **zero**. The row's "13" is the `resolution.rs` subset; the exempt population is **22 across 4 files** | none (HIGH) | **DOWN → fold as a rider** |
| 28 | PB-DX14 | LIVE, **+ a second read site nobody named** (`replay_harness.rs:4016`) | **32** defs carry `starting_loyalty`, **7** deck-legal (independently reproducing PB-DX29's "7 not 6"). **15 defs have a `back_face` and 0 of them are planeswalkers → the affected population is ZERO** | **the row's "PROTOCOL+HASH" is WRONG → predict NONE** (HIGH): `CardFace` is off-wire via `CLOSURE_MUST_NOT_CONTAIN` and has **no `HashInto`** | **DOWN, hard** |
| 29 | PB-DX15 | **three different verdicts in one row** | `OOS-DP9-11` **live-wrong on 5 deck-legal `Complete`** (`birthing_ritual`, `chaos_warp`, `goblin_ringleader`, `growing_rites_of_itlimoc`, `sylvan_messenger`); `OOS-DP9-8` **live-wrong on 10 deck-legal `Complete`** (the Fleshbag / Grave Pact family); **`OOS-DP9-16` is unreachable by construction** — both delayed-trigger producers mint fresh `ObjectId`s | none (HIGH) — but golden scripts and SR-9b fingerprints move | **UP, and SPLIT** |
| 30 | PB-DX38 | LIVE and **bigger** | **10 wrong cites across 11 lines** in `events.rs`, not 9 (new: `:817` `PermanentGoaded` cites CR 701.38 = *Vote*; should be 701.15). CR 726 = **76 across 27 files** (41 in source), not 74/25. A new mechanical derivation finds **206 candidate mismatches across 97 files** | none (HIGH) | **UP** |
| 31 | PB-DX16 | LIVE | **`edgar_charmed_groom.rs` does not exist** — the "1 flip" is a **new authoring**, not a flip | PROTOCOL+HASH (HIGH) | **DOWN** |
| 32 | PB-DX17 | LIVE (`abilities.rs:4447` fires once per combat) | **1 def** mentions "attacks a player" (`shiny_impetus`, gated); 0 mention the opponent-attacks-another case; `karazikar` unauthored | **the row's "none" is UNSAFE** (MED) — the runtime `TriggerEvent` is reachable from `Characteristics` (`game_object.rs:988`), a `CLOSURE_MUST_CONTAIN` root; the DSL `TriggerCondition` the row is thinking of is the off-wire one | **DOWN** |
| 33 | PB-DX39 | both LIVE (`layers.rs:842-859`, `:897-907`) | reproduces exactly — `umezawas_jitte` deck-legal `Complete` and live-wrong, `mardu_ascendancy` `partial` | none achievable (LOW-MED) | **UP** (modest) |
| 34 | PB-DX40 | both LIVE; `wastes.rs` absent, **0 defs carry Decayed** | "+2 defs" = **2 new authorings**, not flips. The row says the corpus is 1,804; it is **1,803** | none (HIGH); every def-count pin moves — budget **two** reconciliation passes (PB-DX27's lesson) | STAY / slight UP |
| 35 | PB-DX41 | both LIVE; **PB-DX29 did NOT close `OOS-SIM1-1`** (`params.rs:346-354` byte-unchanged) | 5 of 7 cast-relevant `GameRestriction` variants mirrored; 2 unmirrored + split second (0 references in the simulator) | **the row's "PROTOCOL" is FALSE → NONE** (HIGH): `CastSpellData` already carries `hybrid_choices` (`command.rs:810`) and `phyrexian_life_payments` (`:816`); the missing field is on the **simulator-local** `LegalAction`. Its "split it out unless it rides PB-DX34's bump" dependency is **void** | **UP, by more than one notch** |

**Three cheapenings, each caused by a shipped batch answering a seed's own open question.**
(i) `OOS-M11-5`'s in-source justification (`casting.rs:6072-6073`: *"used by auras/bestow which
validate via a separate enchant path"*) is **stale** — PB-DX20 synthesises the requirement at the
cast site (`:3627-3637`), so Auras now reach `validate_targets_inner` with a non-empty list.
(ii) `OOS-DX4-5`'s wire cell said "depends on whether a costless *may* gets a real channel"; PB-DX28
built it (`stubs.rs:976-984`, `ChooseObject { up_to: true }`). (iii) `OOS-SIM2-3`: PB-DX29 already
shares the cost arithmetic through `effective_cast_cost_with_additional` (`local_game.rs:1125`);
only the command filter at `:1112` is unwidened — which is `OOS-SIM6-3`.

**Three cost increases.** `OOS-DP2-4`'s PRNG pin now re-deals **18+ committed seeded fixtures**
(PB-DX22 multiplied them), and PB-DX26 needed **two** reconciliation passes for a single marker
flip. Rank 20's site count went 5 → 9. Rank 25's enforcement surface went from one type to ~11
files.

**Two wire predictions are wrong in opposite directions, and one "none" is unsafe.** Rank 28
predicts PROTOCOL+HASH and measures **none**; rank 35 predicts PROTOCOL and measures **none**;
rank 32's "none" is unsafe because it names the off-wire DSL type while the reachable one is the
runtime `TriggerEvent`. Rank 21 omits a HASH bump and rank 17 probably does too. **Five of
twenty-one wire cells were wrong**, which is why every cell in §4 now carries a confidence.

**Three yield labels measure the wrong thing** — PB-DX26's lesson, recurring in the standing rows.
Rank 31's "1 flip" and rank 28's "2 flips" are **new authorings** (no def file exists at all);
rank 34 labels its two correctly. `rider-seed-triage-2026-07-19.md:321` made exactly this
correction for `karazikar` and left `edgar` wrong **one line above it**.

**A gate-integrity class spanning ranks 15, 16 and 22, found only because three rows were checked
together.** `pb_dx8_oracle_decision_cross_check.rs` counts a field's **presence** as proof the code
**reads** it, and in both measured cases it does not. `optional: true` (`:226-231`) is structural
evidence for the `may` channel — and the corpus's only five `optional` keys are **exactly
`OOS-DX4-5`'s inert-field members**, so *the gate exempts precisely the defs that disprove it*.
`modes` non-null (`:234-239`) is evidence for the `choose` channel — true on the cast path, false on
the triggered path (`abilities.rs:9613`). PB-DX8's published `may` figures (287 / 72) are therefore
**understated by up to 5**. This is §2.7's inert-`optional` finding arriving from a second
direction, and it argues for merging the evidence-integrity halves of ranks 15/16/22 into one
dispatch.

**Two registry rows are WRONG rather than stale, and one of them would make a batch ship a defect.**
`OOS-UI2-5` (registry `:1276`) claims the TUI routes casts through `params.rs` and gets a silent
`eligible[0]` default. **It has never routed a cast** — a TUI human gets an outright refusal
(`casting.rs:3315`). v3 recorded this correction at its own §1c and **the registry was never
updated**. The consequence is the reverse of what PB-DX33's row implies: **routing the `CastSpell`
site through `params.rs` is what would *create* the silent-default defect**, on 13 deck-legal
`Complete` defs. `OOS-DX23-3` (`:1348`) says the TUI "never" routes through `params.rs`; false
since SIM-6 (`a878ca26`). Both corrected in §6.

**Seven standing-row seeds have no registry row** — `OOS-CARDS2-6`, `OOS-OS6-1`, `OOS-OS7-1`,
`OOS-RS-5`, `OOS-OS4-1`, `OOS-RS4-3`, `OOS-OS4-3` — affecting ranks 23, 25, 26, 28, 31, 32. These
are **additive to §1a's 61**, which counted post-v3 seeds only, so the registry's true blind spot is
**68**. Whoever takes those ranks files rows first (dispatch hygiene 5, sixth consecutive instance).
Note that `OOS-RS-5` and `OOS-RS4-3`'s only homes include `rider-seed-triage-2026-07-19.md`, which
CLAUDE.md forbids claiming from (§3/§5) — use `pb-review-OS7.md:118-125` and `pb-plan-RS4.md`.

**Two numbering hazards worth carrying**: `OOS-OS7-3` is a **burned ID** and must not be reused;
and rank 32's scope "`OOS-OS7-1` **R2+R3**" is **not reproducible** — `pb-plan-OS7.md:270-295`
defines only R1 and R2. R3 was invented by an earlier triage and has been copied forward three
times.

**§5's `proliferate` entry reproduces exactly at 23**, by four independent derivations
(`Effect::Proliferate` ∩ deck-legal = 23; case-insensitive text ∩ deck-legal = 24, the extra being
`rift_bolt`, whose first two lines are a stray header describing *Inexorable Tide* — itself a
rank-30 hygiene item; and the executed `decision_site_reconciliation_report` printing
`proliferate: 23 Complete` against `1136 Complete of 1803`). **`OOS-DP10-6`'s "25" (registry
`:1209`) is a stale 2026-07-27 snapshot** that drifted *down* through CARDS-2's and PB-DX27's honest
demotions. The still-auto union is **80**, unmoved by PB-DX8, DX28 and DX29; the only parked row to
leave the set is `discard_cards` (13 → 12), closed by ENG-1.

---

## 4. The authoritative queue — 41 ranked entries, **PB-DX9 .. PB-DX61** (AC 6493)

> ### 🚦 READ THIS BEFORE CLAIMING ANYTHING
>
> **The PB-DX number is a stable label, NOT a rank.** v3's ranks 1-13 are all shipped. Every
> standing v3 entry **keeps its number and its scope** unless a re-verification in §3.2 corrected
> it, so that every existing cite still resolves; new batches are numbered **PB-DX43 onward**
> (PB-DX42a shipped as a rider; PB-DX42b is standing and re-decided in §3.1). The table is ordered
> by **rank**, and the rank column is the only thing a dispatcher should read. Renumbering the
> survivors was considered and rejected for the third triage running: this queue exists because of
> the N4 re-dispatch hazard, and silently re-pointing a PB number at different work *is* that
> hazard.
>
> **↻ 2026-09-04: `PB-DX36` SHIPPED (`scutemob-228`). The next dispatch is `PB-DX52`** (rank 14 —
> Bolt Bend's printed *"or ability"* half is unreachable, `OOS-DX25b-1` + rider `OOS-DX25b-5`).
> Ranks **1-13 are all shipped**.
>
> **Read this before dispatching from row 13's cell — every one of its cells held, and the two
> things that did NOT come from the memo are the ones worth carrying.** The wire cell was right on
> both halves (`TriggerCondition` off-wire, `EffectAmount` on-wire — PROBED at stage 0 by extending
> each gate's own `CLOSURE_MUST_NOT_CONTAIN`, not inherited), the yield cell's **1 flip
> (`exalted_angel`)** delivered exactly one, and the correctness framing was accurate.
>
> **(a) The obvious fix was wrong, and only an INVERSE oracle-text census shows it.** `combat_only`
> is read solely by the hasher, so the tempting move is to delete it. Declared users of
> `combat_only: true`: **0**. *Printed* users: **1** — `breath_of_fury`, whose blocker is Aura
> re-attachment, not the trigger. A census over the DECLARED axis alone says "delete the flag" and
> would have made that card permanently over-fire. **The declared axis and the printed axis do not
> nest, and this is the third batch in this queue to be saved by running both** (PB-DX26, PB-DX43,
> now here).
>
> **(b) The member list was a FLOOR by THREE, and the extra member is a GRANTED ability.**
> The task brief names exactly ONE self-family def (`exalted_angel`) — queried, not assumed;
> `goblin_lackey`, `warren_instigator` and `tandem_lookout` all came from this batch's own
> stage-0 inverse oracle scan, and the `all_cards()` roster then corrected THAT to ten still
> blocked. `tandem_lookout` grants
> *"Whenever this creature deals damage to an opponent, draw a card"* through Soulbond and is a
> fourth (`OOS-DX36-1`). A structural census keyed on `AbilityDefinition::Triggered` cannot see an
> ability a card GRANTS rather than has — the same shape as PB-DX43's token-conferred abilities.
>
> **(c) The task brief's CR cite was wrong and was not obeyed.** It cites CR 603.10a for
> *"that much"*; CR 603.10a is look-back-in-time **zone-change** triggers. Shipped against
> CR 603.2c and CR 608.2h / CR 113.7a. **A brief is a claim like any other** — and this one had
> been copied into the acceptance criteria, so obeying it would have put 13 wrong cites in the
> tree under an AC that read as satisfied.
>
> **Read this before dispatching from row 12's cell — its wire cell was right on BOTH halves and
> its yield cell was wrong by one flip, for a reason the seed row itself predicted.** *"2 real
> flips (`shambling_ghast`, `hullbreaker_horror`)"* delivered ONE. `hullbreaker_horror` could not
> be re-shaped, because 3 of the 7 corpus modal TRIGGERED abilities look their `ModeSelection` up
> in the REGISTRY index space while carrying a RUNTIME `ability_index` — a defect no document
> named — so moving its targets into `mode_targets` would DROP the requirement, which is exactly
> the trap `OOS-DX4-2`'s own row warns about. It is re-adjudicated with the surviving blocker
> named rather than re-shaped, and the class is filed as `OOS-DX35-1` with zero deck-legal blast
> radius measured rather than assumed. **The durable half for the next dispatcher**: a yield cell
> that names members is a FLOOR on the CENSUS and a CEILING on the FLIPS — `OOS-DX4-2`'s member
> list was short by more than double (5 of 7, and the only deck-legal member,
> `retreat_to_kazandu`, is named by neither the seed nor this row), while its two named flips
> delivered one.
>
> **Read this before dispatching from row 11's cell — its wire cell was RIGHT, its yield cell was
> right, and BOTH the row and its seed under-stated the defect's reach.** *"HASH (MED) — needs a
> third combat-state field"* held on the fingerprint (HASH 81 → 82, PROTOCOL 41 unmoved, closure
> type count unmoved at 132) and was **wrong on the count**: CR 508.8 ORs its two facts in one
> sentence, so ONE monotone `bool` set by ONE new mutator is the whole predicate, and two fields
> would have been two things to drift. *"0 flips"* held exactly. What neither the row nor
> `OOS-DX21-4` says is **how the defect is actually reached**: the seed's recipe (*"kill it / phase
> it out / stop it being a creature"*) reproduces NOTHING, because the engine implements only two
> of CR 506.4's six removal causes (`OOS-DX51-2`). The route that does reproduce makes the seed
> **worse** than filed — `reconnaissance` is `Complete`, deck-legal, `{0}`, instant-speed and
> repeatable, so this was **live on 2 deck-legal `Complete` defs**. Also: **a SECOND SR-38 hole
> sits on the very `if` statement row 11 sends you to** (`OOS-DX51-3`), and **this batch's own
> `r1` source gate was defeated by execution and re-keyed** (`OOS-DX51-6`).
>
> **Read this before dispatching from row 10's cell — its wire cell was RIGHT, its coverage cell
> was right, and its scope was short by a whole mechanism.** *"HASH only (HIGH) — `GameState` is in
> `CLOSURE_MUST_NOT_CONTAIN`"* held exactly (HASH 80 → 81, PROTOCOL 41 unmoved), and it held even
> after `AbilityDefinition::Splice` gained a field mid-batch. *"0 flips"* held. What the row does
> not contain is **splice**: CR 702.47a copies the spliced card's text box onto the spell, so a
> spliced spell requires that card's targets, and `AbilityDefinition::Splice` had **no `targets`
> field at all** — so closing `OOS-M11-5` REQUIRED shipping it, and a batch that only added the
> CR 601.2c rejection would have broken the corpus's one splice card. And the row's cost cell
> (*"the PRNG pin now re-deals **18+** committed seeded fixtures"*) is **refuted by measurement**:
> exactly **ONE** pin moved, because the simulator's opening deal uses `SliceRandom` with its own
> `StdRng` and is not one of the four sites the seed names. The `*_SEED` axis is the wrong axis.
>
> **Read this before dispatching from row 9's cell — its wire cell was RIGHT, its site cell was
> right about the NUMBER and wrong about the SHAPE, and its coverage cell was wrong.** The wire
> cell said *"PROTOCOL + HASH (HIGH)"* and *"cheaper than the row implies: `TargetFilter.has_card_types`
> already exists and `enchant_target_to_requirement` already uses it"* — both held exactly
> (PROTOCOL 40 → 41, HASH 79 → 80, one bump each, type counts unchanged at 98 / 131, all predicted
> in writing before any code). *"Three sites"* is the right count of PLACES and the wrong picture:
> there were **two arithmetics and three consumers**, the consumers already shared, so a batch
> patching "three sites" one at a time carries the new field in two copies. And *"+1 `partial`
> unblocked (`kayas_ghostform`)"* is **false** — that def's own `Completeness::partial` note
> already said the Enchant line was not its blocker; coverage is **unmoved at 63.1% with 0 flips**.
> The population was also short by one: **`breath_of_fury`**, which no seed row and no cell names.
>
> **Read this before dispatching from row 8's cell — its wire cell was RIGHT and its scope cell
> was short by one site.** The wire cell said *"PROTOCOL + HASH (MED) — the timing half needs an
> `EffectChoiceQuestion` variant; the legality half may be routing only"*. Both halves reproduce
> exactly: the legality half moved **neither** fingerprint and the timing half moved each **once**
> (PROTOCOL 39 → 40, HASH 78 → 79), both predicted in writing before any code and both gate-computed.
> The census cell reproduces too — **6** deck-legal `Complete` mutate defs, re-derived by walking
> `all_cards()` and printed by a test. After a queue in which almost every published population has
> been a floor, row 8's numbers held; that is worth recording precisely because it is rare.
>
> **What the row did NOT have, and no dispatcher could have inferred from it**: both seeds describe
> **two** enforcement sites and there are **three** — `legal_actions.rs`'s offer enumeration is the
> third, and leaving it alone while tightening cast-time legality ships a clean offer followed by a
> guaranteed refusal (the SR-38 shape PB-DX29 gated Fuse to avoid, PB-DX44 recreated and PB-DX45
> shipped). And **`OOS-DX25-1`'s own prescription would have made the fix CR-wrong**: CR 702.140b is
> an explicit EXCEPTION to CR 608.2b, so routing the mutate host into `spell_targets` and stopping
> there hands it to a fizzle gate that must never see it. Neither seed, nor this row, says so — only
> the rule does. **A site list is a floor and a prescription is a claim.**
>
> **Read this before dispatching from row 7's cell — it is right about the wire and wrong about the
> populations, in a way row 7 could not have known.** The wire cell holds exactly: the
> continuous-effect-scan design moved **neither** fingerprint (PROTOCOL 39 / HASH 78, gate-executed,
> predicted in writing before any code). The census cells do **not**: the Saga population is **3**,
> not 4 (`song_of_freyalise` names `SagaChapter` only in `// TODO`s and its `inert` note — SR-36
> again), and the blanker population is **11 / 8**, not 13 / 8 and not `OOS-RR4-1`'s own corrected
> 9. **Every figure in that chain grepped the string `RemoveAllAbilities`, which is the wrong
> question**: PB-DX43 moved CR 305.7's ability loss into `SetLandTypes`, so both moons are blankers
> again through a variant no such grep can see. Only deciding by **calling**
> `layers::modification_blanks_abilities` counts a blanker as a blanker. And the deck-legal 8 agrees
> with the row **by coincidence of totals, not of membership** — the row's 8 was 8-of-13
> `RemoveAllAbilities` defs; the true 8 is six of those **plus the two moons**. A batch that checked
> only the total would have recorded the row as confirmed.
>
> **The design mandate in row 7's cell also understated one thing and it is worth carrying
> forward**: CR 714.3a has **no** *"with one or more chapter abilities"* clause (714.3b and 714.4
> both do), so a Layer-6-blanked Saga is still a Saga and still takes its ETB lore counter, and the
> five sites do **not** all ask the same question. Two of the query's fields, not one.
>
> **Read this before dispatching from row 6's cell.** Row 6's wire cell said *"none (HIGH) — reuses
> `GameEvent::PermanentTargeted` at more sites"*, and it was right: PROTOCOL **39** / HASH **78**
> both gate-executed and UNMOVED. Its OUTCOME cell was **wrong in the direction that matters**.
> *"Reuses `PermanentTargeted` at more sites"* describes an EMISSION, and emission alone changes
> nothing at the two sites the seed is named after: `check_and_flush_triggers` scanned the command's
> events and only THEN flushed, so the events a flush itself produced were scanned by nothing, and
> the Ward trigger sat queued until after priority (CR 603.3b). The fix is a bounded fixpoint with
> exactly-once event scanning. **A hook inside `flush_sorted` was tried first and defeated by
> execution — Ward fired TWICE.** Row 6's *"budget fuzz/golden parity movement"* did **not** come
> due: **zero** moved pins, with the reason measured (§4 row 6's strike). And its *"invert the
> deviation pin"* instruction could not be obeyed literally — the pin's fixture targets a PLAYER, so
> no correct engine emits `PermanentTargeted` there and the boolean could never flip (`OOS-DX48-5`).
> *(prior: `PB-DX47` SHIPPED `scutemob-218` 2026-09-02; `PB-DX45` SHIPPED `scutemob-217` 2026-09-02;
> `PB-DX15a` SHIPPED `scutemob-216` 2026-08-23; `PB-DX44` SHIPPED `scutemob-215` 2026-08-15;
> `PB-DX43` SHIPPED `scutemob-213` 2026-08-14.)*
>
> *(`PB-DX47`'s probe-first outcome: the double-push was **REAL**, not a dedup — measured on a
> game built through the production pregame path, `{CardDefETB: 1, Normal: 1}` for one event and
> **two** `+1/+1` counters from a card printing one. The registry scan is deleted and the
> layer-resolved runtime lowering is the single dispatcher; `OOS-DX24-4` CLOSED. This row's own
> "18 deck-legal `Complete` defs if real" **reproduces exactly** at HEAD, re-derived not trusted.)*
>
> **PB-DX45's row said 11 deck-legal `Complete` defs and the number is 10** — see §1d's own
> correction below and `OOS-DX45-2`. This memo's §1d offered "two independent measurements both
> returned 11" as the PROOF that `OOS-DX24-9` and `OOS-DX27-5` are one defect. They are one
> defect; the proof was two agreeing wrong numbers, and a future dispatcher should treat every
> population figure in §4 as an ESTIMATE IN BOTH DIRECTIONS, not merely as a floor.
>
> ~~**The next dispatch is `PB-DX43`.**~~ *(superseded — `PB-DX43` shipped `scutemob-213` and `PB-DX44` shipped `scutemob-215`; see the 2026-08-15 banner above. Left struck rather than deleted because the sentence after it explains WHY this memo names a dispatch at all.)* CLAUDE.md said "next dispatch: coordinator's call" until this
> task; it is repointed in §6. If you are reading a stale pointer, this banner is the correction.
>
> **Numbering, stated so nobody hunts for a gap.** New labels are **PB-DX43..PB-DX61**, and
> **PB-DX46 is deliberately unused** — v3's rank 29 splits, and its live half is labelled
> **PB-DX15a** rather than given a fresh number, because "PB-DX15" is what every existing cite to
> that work says and the split keeps the association legible. `OOS-DP9-16`, the third seed in that
> row, is parked (unreachable by construction: both delayed-trigger producers mint fresh
> `ObjectId`s), so there is no PB-DX15b.
>
> **Three hard sequencing constraints, derived rather than asserted:**
> 1. **`OOS-FB1-1` precedes PB-DX56.** The two live fuzz violations cannot be *diagnosed* without a
>    crash artefact that reproduces, and `bin/fuzzer.rs:328` is literally `command_history:
>    Vec::new()`. It is a prerequisite, not a sibling row.
> 2. **PB-DX9 before PB-DX42b, or re-measure.** PB-DX9 gives `Effect::SearchLibrary` a count field,
>    which is what promotes `the_world_tree` to `Complete` — and that is the day PB-DX42b's supply
>    census stops covering its own population (§2.2, `OOS-DX27-9`'s durable half).
> 3. **Any card-def batch re-deals every seeded fixture** (`OOS-CARDS2-3`'s corpus→seed coupling,
>    now gated). Batch PB-DX58 and PB-DX60 so the re-deal is paid once, and **budget two
>    reconciliation passes, not one** — PB-DX27 needed two for a single marker flip.

### Ordering rule — inherited **verbatim** from v2 via v3, unchanged and still binding

> **Ordering rule** (unchanged from both prior triages): (1) live-wrong on a `Complete`/deck-legal
> path; (2) gate integrity — a gate that reports success while checking nothing; (3) cheap
> high-yield riders; (4) agency / CR completion. Within a tier, cheaper first. "Discounted ship"
> is the expected clean-`Complete` count after the batch, at the historical 2-3× overcount
> discount. **Every wire prediction below is a prediction, not a licence** — the implementer
> gate-computes `PROTOCOL_SCHEMA_FINGERPRINT` / `HASH_SCHEMA_VERSION` and treats a mismatch with
> the prediction as a signal to stop and re-scope (the PB-DP2/DP3 precedent, where two predicted
> bumps were falsified).

And v3's own addendum, also verbatim:

> Compute both fingerprints from the gate's own output; never predict them. A prediction that
> disagrees with the gate is a signal to **stop and re-read**, not to edit the pin. One wire bump
> per PB. Any row above predicting a HASH bump on a type reachable from `Characteristics` should be
> assumed to be a PROTOCOL bump too.

**v4 adds one clause, because §3.2 measured five of twenty-one standing wire cells wrong**: every
`wire` cell below carries a **confidence** — HIGH means the reachability was traced to a
`CLOSURE_MUST_CONTAIN` / `CLOSURE_MUST_NOT_CONTAIN` root or to a `HashInto` impl this task read;
MEDIUM means the type was identified but its reachability was inferred; LOW means the cell is a
guess and the dispatcher should treat gate-computing it as part of stage 0.

| rank | batch | scope | seeds | class | discounted yield (what it measures) | wire (conf.) |
|---|---|---|---|---|---|---|
| ~~**1**~~ | ~~**PB-DX43**~~ **✅ SHIPPED** (`scutemob-213`, 2026-08-14) | CR 305.6/305.7 intrinsic mana abilities from land subtypes | **OOS-DX27-1** + **OOS-DX27-10** (sub-case, closes free) | **CORRECTNESS — 3 deck-legal `Complete` format staples produce no mana at all** | 0 flips (all three already `Complete`); repairs `urborg_tomb_of_yawgmoth`, `yavimaya_cradle_of_growth`, `dryad_of_the_ilysian_grove` in place and lets `blood_moon`/`magus_of_the_moon` delete a hand-authored grant | **HASH** LOW / PROTOCOL none — the derivation writes `Characteristics.mana_abilities`, which is computed not stored, **but `Characteristics` is a PROTOCOL closure root**, so predict-then-gate **SHIPPED as predicted on wire (both UNMOVED, gate-executed) and on coverage (0 flips). The census was a floor short by THREE** — `awaken_the_woods` (a 4th live-wrong `Complete` def, its Forest token produced nothing), `overlord_of_the_hauntwoods` (a 3rd double-grant risk) and `leyline_of_the_guildpact` (`Inert`) — all found by an inverse axis over printed text, because this row's derivation rule reads `LayerModification` payloads and a token confers through a `TokenSpec`. **The scope was also short by half a fix**: deleting only the moons' mana grant would have left their own layer-6 `RemoveAllAbilities` wiping the layer-4 derived ability and broken Blood Moon entirely, so CR 305.7's ability-LOSS moved into `SetLandTypes` and each moon dropped TWO statics. See `memory/primitives/pb-DX43-execution-notes.md`. |
| ~~**2**~~ | ~~**PB-DX44**~~ **✅ SHIPPED** (`scutemob-215`, 2026-08-15) | the casts you cannot make — pitch, split-card halves, fuse targets, Spree mode costs | **OOS-DX29-3** + **OOS-DX29-14** + **OOS-DX29-9** ≡ **OOS-DX29-12** | **CORRECTNESS/AGENCY — 7 deck-legal `Complete` defs cannot be cast as printed, one of them not at all** | 0 flips; `insatiable_avarice` becomes castable from **both** the browser and the bot path; 4 pitch defs gain their printed alternative cost; both fuse defs gain a fused cast | mixed: Spree **none** (HIGH, `effective_cast_cost_with_additional` is the eighth site of PB-DX29's own seven); fuse targets **none** (HIGH, the list grows in content not type); pitch **none** (HIGH, `AltCostKind` exists); **half-selector PROTOCOL** (HIGH) |  *(**OOS-DX29-3** pitch half CLOSED / graveyard half DEFERRED-and-measured, **OOS-DX29-9**, **OOS-DX29-12**, **OOS-DX29-14** all CLOSED. Wire: **PROTOCOL 37 → 38 / HASH 76 → 77** — the memo's cell predicted PROTOCOL only and was short by the HASH half, which the batch predicted in writing before any code. Populations: pitch **4** (row exact), right half **3** not 2, Spree **1**. Filed **OOS-DX44-1..5**.)*
| ~~**3**~~ | ~~**PB-DX15a**~~ **✅ SHIPPED** (`scutemob-216`, 2026-08-23) | the two live CR sweeps | **OOS-DP9-8** + **OOS-DP9-11** — **both CLOSED** | **CORRECTNESS** | — | — | *(**Both wire cells confirmed: PROTOCOL 38 / HASH 77 gate-executed and UNMOVED**, predicted in writing before any code. **The "golden scripts and SR-9b fingerprints move; budget the re-pin" half was WRONG and the reason is the batch's headline**: nothing moved, because every fixture in the tree has `active_player` = the LOWEST `PlayerId`, and APNAP starting from the lowest id IS ascending `PlayerId`. That is also why `OOS-DP9-8` survived — **its own pin was vacuous**, asserting `vec![p(1), p(2)]` on a 2-seat fixture that cannot express the deviation. Now inverted onto a 3-seat active-`p(2)` fixture, with a companion gate stating the vacuity structurally over 2..=6 seats. **This row's "15 deck-legal `Complete` defs between them" is short and mis-framed on both halves.** `OOS-DP9-11`: **17**, not 5 — the named 5 are ONE of FOUR mechanisms (the others: every `SearchLibrary`-to-library tutor (8), Hideaway (1), PartnerWith (3), the last renumbering the WHOLE library, ~99 ids per ETB); and `chaos_warp`, one of the row's own five, reaches the `Library{Top}` branch, not the bottom helper the seed is filed against. `OOS-DP9-8`: the "Fleshbag/Grave Pact family (10 defs)" **makes no per-player choice at all** — `sacrifice_permanents_for_player` auto-picks — so what is repaired there is event ORDER, not agency; only **2** deck-legal `Complete` defs exercise the literal question-order claim. Fix is structural at the two `GameState` move helpers, not the per-caller sweep the seed asks for, because `Effect::MoveZone`/`PutOnLibrary` resolve their destination at RUNTIME. Riders (FINAL, after the `/review` fix cycle inverted both first-draft verdicts): **`OOS-DX24-1` CLOSED** (its deferral reason was refuted — the triggering EVENT is a wire-neutral discriminator already passed to the doubler) and **`OOS-DX24-7` RE-OPENED** (the `EventBatchTiming` fix regressed CR 603.10a on 21 board wipes — `DestroyAll` kills in ONE loop — and is reverted; the correct unit is a simultaneous GROUP, needing group boundaries in the event stream) — and **both riders' prescribed fixes were wrong as written**, each refuted by executing it. `OOS-DP9-16` NOT taken, parked as directed. Filed **OOS-DX15a-1..7**. See `memory/primitives/pb-DX15a-execution-notes.md`.)*
| ~~**4**~~ | ~~**PB-DX45**~~ **✅ SHIPPED** (`scutemob-217`, 2026-09-02) | `Effect::MayPayThenEffect` is pay-when-able, so CR 118.12's player decision is engine-made | **OOS-DX24-9** ≡ **OOS-DX27-5** — **both CLOSED**, cross-cited | **CORRECTNESS/AGENCY** | — | — | *(**Wire confirmed EXACTLY as this cell predicts: PROTOCOL 38 → 39 / HASH 77 → 78**, one bump each, both taken from the failing gates' own output and both predicted in writing before any code — including the prediction that neither closure's type count would move (confirmed 98 and 131). The design cell is right too: one `EffectChoiceQuestion::PayOptionalCost` on PB-DX28's channel, no new mechanism. **Three things this row got wrong.** (1) **The population is 10, not 11** — re-derived twice at HEAD, `OOS-DX45-2`; §1d's correction above has the detail. (2) **The SITE list is short by one**: `effects::try_pay_optional_cost` has TWO callers and this row names only `MayPayThenEffect`; the other is `Effect::LookAtTopThenPlace`'s `place_cost`, the identical CR 118.12 decision one function over, live on a deck-legal `Complete` def (`birthing_ritual`). Both repaired; the scope line shipped is *every caller of `try_pay_optional_cost`*, which is also what puts three never-charged `Complete` defs (`teneb_the_harvester` + two Extort carriers, `OOS-DX45-3`) explicitly OUT. (3) **"0 flips" is wrong: there is exactly ONE**, `vampire_gourmand` `partial` → `Complete`, from the `OOS-DX27-5` policy re-adjudication the row's own seed demanded — predicted and NAMED before regeneration; coverage **1,136 → 1,137 = 63.1%**. Also: `OOS-DX27-5`'s claim that TWO defs were left `partial` on the same shape is refuted — `ruthless_technomancer`'s marker names its activated ability. Ruling in `memory/decisions.md`. Filed **OOS-DX45-1..8**. See `memory/primitives/pb-DX45-execution-notes.md`.)*
| ~~**5**~~ | ~~**PB-DX47**~~ **SHIPPED** (`scutemob-218`, 2026-09-02) | ~~probe first: does a `WhenDealsCombatDamageToPlayer` trigger get pushed twice?~~ **YES — the double-push was REAL.** The registry scan in `abilities.rs`'s `CombatDamageDealt` arm is DELETED; the layer-resolved runtime lowering is the single authoritative dispatcher | **OOS-DX24-4 CLOSED** (four corrections to its own claims recorded in the row); filed `OOS-DX47-1..7` | **CONFIRMED — 18 deck-legal `Complete` defs**, and this cell's conditional figure **reproduces exactly** at HEAD, re-derived by an `all_cards()` walk rather than trusted (dispatch hygiene 6). The batch's own first-draft roster said 30, typed from a `grep -l` that counts `// TODO` comments — SR-36 broken inside the batch, caught by its own gate, filed `OOS-DX47-2` | **none, as predicted**: PROTOCOL 39 / HASH 78 both gate-executed and UNMOVED. This cell's claim that the justifying comment is false at HEAD was **correct, and short by a half** — it is false TWICE (the conversion DOES happen in `build_face_ability_vectors`, *and* `enrich_spec_from_def` is production). Both copies of the sentence corrected; the second had been cited as precedent (`OOS-DX47-6`) |
| ~~**6**~~ | ~~**PB-DX48**~~ **✅ SHIPPED** (`scutemob-219`, 2026-09-02) | Ward never fires on a triggered ability | **OOS-ENG2-1** ≡ **OOS-ENG2-2** (+ **OOS-ENG2-3**) | **CORRECTNESS — 3 deck-legal `Complete` Ward defs** | 0 flips; the seed's 5-site census is **exact and complete**, which is rare enough to state | **none** (HIGH) — reuses `GameEvent::PermanentTargeted` at more sites. Invert the deviation pin at `pb_eng2_targets_announced.rs:384-392`; budget fuzz/golden parity movement |  *(**OOS-ENG2-1** ≡ **OOS-ENG2-2** FILED and CLOSED — neither had a registry row, the 61-of-208 blind spot this memo measured; **OOS-ENG2-3** ROWED and NARROWED. The 5-site census reproduces **EXACTLY**, so this row's "exact and complete" claim is the one that held. Wire **PROTOCOL 39 / HASH 78 UNMOVED**, predicted in writing before any code. **The parity movement this row budgeted came due on the FUZZ side and not in the test suite, and the split is measured.** The full workspace shows **0** regressions, because no fixture in the tree puts a triggered / forecast / scavenge / loyalty ability's target on an opponent-controlled permanent carrying a becomes-target trigger, and the SR-9b per-step fingerprint hashes GAME STATE rather than the event stream. The fuzzer is a separate measurement and it MOVED: identical invocation on the merge base vs this branch gives HARD **185 / 13 distinct UNMOVED** (both sub-checks, both game lists, and avg turns 122.3 all identical) but TRANSIENT `no_orphaned_tokens` **273 → 275** and **+20 rejections, all twenty inside ONE game of twenty** (seed 12) — one divergence with everything else downstream of it. *(The first draft of this strike said the budget "did NOT come due — 0 moved pins"; it was written at `2ce70e35`, before the fuzz A/B ran at `1eab7cf3`, and was never re-taken. That is PB-DX45's "measured table never re-taken" defect, caught by this batch's own `/review`, and it matters because this memo is the document the next dispatcher reads.)* Two corrections to the row: the fix is NOT an emission (see the banner), and the deviation pin could not be inverted literally. Filed **OOS-DX48-1..7**.)*
| ~~**7**~~ | ~~**PB-DX49**~~ **✅ SHIPPED** (`scutemob-220`, 2026-09-03) | every Saga site reads the printed def, so a blanked Saga is still sacrificed | **OOS-RR4-1** *(filed by this task)* + **OOS-RR4-3** (doc rot, rider) — **both CLOSED** | **CORRECTNESS — 2 deck-legal `Complete` pairs, one unconditional; closes the engine half of the corner-case audit's LAST open GAP** | 0 flips **as predicted, and 0 card-def edits**; 5 behavioural sites unified behind one read-only query; folds in the CR 708.2a face-down conjunct `queue_carddef_etb_triggers` already performs | **none** (HIGH) **for the continuous-effect-scan design only** — lowering `SagaChapter` into `Characteristics` instead moves **both**. The brief must name the design (§1g) | *(**The wire cell is EXACTLY right** — PROTOCOL 39 / HASH 78 both gate-executed and UNMOVED, predicted in writing before any code at `57d1dc42`. **Both census cells are wrong**: Saga side **3**, not 4; blanker side **11 / 8**, not 13 / 8 nor the row's own corrected 9 — see the banner for why every figure in that chain measured the wrong thing, and why the deck-legal 8 agrees by coincidence of totals rather than of membership. **The CR reading behind the site list needed correcting too**: CR 714.3a carries no chapter-ability clause, so a Layer-6-blanked Saga still takes its ETB counter and site 4 asks a different question from sites 1/2/3/5. Shipped shape: `layers::abilities_are_blanked` as the single blanking predicate (IG-1 refactored to consume it) plus `rules::saga::saga_view` as the single CR 714 query, with `resolution.rs`'s two CR 113.7a sites excluded and saying so in source. Corner case **#36 GAP → PARTIAL** (engine half covered; the card half stays gated on `OOS-RR4-2`, explicitly not taken). Tests **4,941 / 0 / 5** (+41, 58 targets). Filed **OOS-DX49-1..9**, of which **`OOS-DX49-1` is LIVE on a deck-legal `Complete` def** — `binding_the_old_gods`' chapter I destroys nothing, because `SagaChapter` is never lowered into `chars.triggered_abilities`; found by execution, deliberately left unpinned. See `memory/primitives/pb-DX49-execution-notes.md`.)*
| **8** | ~~**PB-DX50**~~ **✅ SHIPPED** (`scutemob-221`, 2026-09-03) | the mutate surface: target legality and CR 702.140c timing | **OOS-DX25-1** (re-classified latent → partly live) + **OOS-DX29-2** | **CORRECTNESS — 6 deck-legal `Complete` mutate defs, newly human-reachable** | 0 flips; mutate target validation (`casting.rs:1306-1364`) checks zone, creature-ness, non-Human and owner **and nothing else** — no shroud, no protection — while PB-DX29 just made mutate fully reachable from the browser | **PROTOCOL + HASH** (MED) — the timing half needs an `EffectChoiceQuestion` variant; the legality half may be routing only — **HELD on both halves**: legality moved neither fingerprint, timing moved each once (PROTOCOL 39 → 40, HASH 78 → 79). Both predicted in writing before any code and gate-computed; type counts unchanged at 98 / 131. **`OOS-DX25-1` and `OOS-DX29-2` both CLOSED**, each row corrected against its own claims; `OOS-DX50-1..11` filed, of which `-1` (every trigger from a replayed CR 608.2d resolution queued TWICE — pre-existing and engine-wide), `-2` and `-10` are also closed here. **`-11` was filed by the `/review` fix cycle, after this row's first draft said `-1..10`** — dispatch hygiene 8's exact case, caught by re-checking this cell against the registry after the fix cycle rather than before it. |
| ~~**9**~~ | ~~**PB-DX20b**~~ **✅ SHIPPED** (`scutemob-222`, 2026-09-03) | `EnchantFilter` has no OR over card types | **OOS-DX20-10** + **OOS-DX20-5** — **both CLOSED as ONE defect**, cross-cited | **CORRECTNESS — 1 deck-legal `Complete` (`imprisoned_in_the_moon`), human-reachable since PB-DX20** | **0 flips — and the "+1 `partial` unblocked" half of this cell is REFUTED**; `kayas_ghostform`'s own marker note already said the Enchant line was not its blocker, so it stays `partial` with the surviving blocker named. Coverage unmoved **1,137/1,803 = 63.1%** | **PROTOCOL 40 → 41 / HASH 79 → 80, ONE bump each — this cell is EXACTLY right**, including *"cheaper than the row implies"*: the fix lowers onto the existing `TargetFilter.has_card_types` and builds no parallel OR mechanism. Both predicted in writing before any code, type counts predicted unchanged and confirmed at 98 / 131 *(**"Three sites" is the right NUMBER and the wrong SHAPE**: two ARITHMETICS — `casting::enchant_target_to_requirement` and `sba::enchant_filter_matches` — and three CONSUMERS that were already shared. Shipped as ONE arithmetic: `casting::enchant_filter_to_target_filter` is the single lowering and `sba.rs`'s hand-rolled six-field predicate is DELETED in favour of calling it and handing off to `effects::matches_filter`. **The population is THREE, not two** — `breath_of_fury` prints "Enchant creature you control" and dropped the controller clause, named by neither seed nor this cell, and needing no new expressiveness at all. **A neighbouring batch's row died here and it was designed to**: PB-DX49's Pair A existed only because of `OOS-DX20-10`, so `r4a` went red and is re-adjudicated with the death COMPUTED, not deleted. Tests **5,015 / 0 / 5** (+24, 60 targets, itemised by NAME as 25 additions / 1 disclosed rename / 0 removals). Filed **OOS-DX20b-1..7**, of which `-1` is a pre-existing SR-38 offer defect found by execution (`legal_actions.rs` reads RAW characteristics for `DeclareAttackers` eligibility) and `-5` is a close-out methodology hazard that fabricates a removal. See `memory/primitives/pb-DX20b-execution-notes.md`.)*|
| ~~**10**~~ | ~~**PB-DX18**~~ **✅ SHIPPED** (`scutemob-225`, 2026-09-04) *(standing, v3 rank 14)* | the trust boundary on ungated commands | **OOS-DP2-7** + **-4** + **-8** + **OOS-DX2-4** + **OOS-DX2-1** + **OOS-M11-5** — **all six CLOSED** | correctness (gated) + hygiene | **0 flips as predicted** (coverage unmoved 1,137/1,803 = 63.1%, proven by regeneration); 3 card-def edits, of which 2 are comment-only and 1 authors `glacial_ray`'s CR 702.47a splice targets — **no `Completeness` marker moved** | **HASH 80 → 81 / PROTOCOL 41 UNMOVED — this cell is EXACTLY right**, and it survived a mid-batch addition it did not anticipate (`AbilityDefinition::Splice` gained a field; `AbilityDefinition` is reachable only through `CardDefinition`, which the same list excludes). *(**The scope was short by a whole mechanism: SPLICE.** See the banner. **The cost cell is refuted**: ONE pin moved, not 18+, and it moved in the improving direction — the fuzz decision partition gained `surveil`, attributed by an EXECUTED A/B. **Two further corrections**: `OOS-DP2-7`'s prescription (*"a seeded `Zone::shuffle` at both sites"*) cannot work as written, because one site holds `&GameState` and a shuffle before the move does not include the redirected card; and `OOS-DP2-4`'s addendum names ONE re-permutation channel where there are TWO — `Rng::random_range`'s sampling is as unpinned as `StdRng` itself, so pinning only the generator leaves the identical defect one layer down. Filed **OOS-DX18-1..6** (`-6` by the `/review` fix cycle). Benches: a REAL uniform ~2.5-4.5% regression, four runs with the same-code band measured first. See `memory/primitives/pb-DX18-execution-notes.md`.)* |
| ~~**11**~~ | ~~**PB-DX51**~~ **✅ SHIPPED** (`scutemob-226`, 2026-09-04) | CR 508.8 / 506.4 — the step-end skip, the blocker offer, and the `CombatState` install order | **OOS-DX21-4** + **OOS-DX21-5** (rider) + **OOS-DX21-2** — **all three CLOSED**, each row corrected against its own drifted cites and claims | **CORRECTNESS — live on 2 deck-legal `Complete` defs** (`reconnaissance`, `thaumatic_compass`), which is MORE than this cell claimed | **0 flips as predicted** (coverage unmoved 1,137/1,803 = 63.1%, proven by regeneration); **0 card-def edits**, so the shortcut was available and the regeneration was run anyway | **HASH 81 → 82 / PROTOCOL 41 UNMOVED — this cell is right on the fingerprint and WRONG ON THE COUNT.** *(**"a third combat-state field"** is one field too many: CR 508.8 ORs its two facts in a single sentence, so ONE monotone `bool` — `had_attackers`, set by ONE new mutator `CombatState::add_attacker` on both of CR 508.8's routes — IS the predicate. Closure type count UNMOVED at 132, measured at the merge base rather than assumed. **All three cites in this cell drifted again** and are corrected in the rows. **The CR 508.4 entrant census reproduces at FOUR**; the dispatch brief's two extras are refuted. **`OOS-DX21-4`'s own reproduction recipe is wrong in all three of its named routes** — the engine implements 2 of CR 506.4's 6 removal causes (`OOS-DX51-2`), and the route that does reproduce is `reconnaissance`'s free instant-speed `{0}`. Fuzz: the PB-DX32 gate config is byte-identical before and after, attributed by an **executed ablation**; the 20-game run's movement is ALL `OOS-DX21-6` reindexing, proven by a third run in which the full engine change with only the offer conjunct ablated reproduces the merge base byte-identically — so the engine half is fuzz-neutral by measurement. `AlreadyDeclaredBlockers` **9 → 0**. Tests **5,058 / 0 / 5** (+14, 61 targets, byte-exact set diff, 0 leavers / 0 removals / 0 renames). Filed **OOS-DX51-1..7** — the first draft of this cell said `-1..6`, dispatch hygiene 8's exact case, corrected by re-checking against the registry AFTER the `/review` fix cycle. `-6` is this batch's own `r1` gate defeated by execution — **six successful bypasses across three successive drafts, two of them found by the `/review` AFTER the first re-key** — and `-7` is the residual that follows: no textual gate over a `pub` field is closable, and the compile-enforced version is measured at ~160 sites and deliberately not taken. `-5` records that PB-DX18's published close pin of 5,041 does not reproduce (5,044 at a byte-identical `.rs` tree). See `memory/primitives/pb-DX51-execution-notes.md`.)*|
| ~~**12**~~ | ~~**PB-DX35**~~ **✅ SHIPPED** (`scutemob-227`, 2026-09-04) *(standing, v3 rank 22)* | modal trigger targets + the inert `optional` | **OOS-DX4-2** + **OOS-DX4-5** — **both CLOSED**, plus **OOS-DP10-5** CLOSED and **OOS-DX8-3** updated | **CORRECTNESS — 1 deck-legal `Complete` def live-wrong on Half A (`retreat_to_kazandu`) and 5 on Half B** | **1 flip, NOT 2 — this cell is REFUTED and the seed row's own trap is why.** `shambling_ghast` `partial` → `Complete`. `hullbreaker_horror` is re-adjudicated and NOT re-shaped: its modal ability sits at REGISTRY index 1 behind `Keyword(Flash)` while a `Normal`-kind trigger carries a RUNTIME `ability_index` of 0, so the `ModeSelection` lookup returns a `Keyword` and moving its targets into `mode_targets` would DROP the requirement exactly as `OOS-DX4-2` warns. Coverage **1,137 → 1,138 / 1,803 = 63.1%**, the single flip named in writing before any code. DX4-5's 5 are indeed already `Complete` → **0 flips there**, in-place comment-only repair, as this cell says | **BOTH UNMOVED — this cell's Half-B half is EXACTLY right and its Half-A half resolves to the "none" branch.** *(**"none-if-registry / both-if-lowered"** was the right question: `TriggeredAbilityDef` carries no `modes` field at all, so the REGISTRY is already the incumbent source at both existing read sites and reading `mode_targets` there adds no type, variant or field. The lowering branch is COSTED rather than waved away — 190 exhaustive struct literals across 44 files, plus both bumps — and named as the branch not taken. **"DX4-5 now none — PB-DX28 built the channel"** is exactly what shipped: `EffectChoiceQuestion::ChooseObject { count: 1, up_to: true }`, which is what all five printed cards literally say. HASH 82 / PROTOCOL 41, gate-executed, **ZERO bumps for the whole PB**. **THE FINDING**: 3 of the 7 corpus modal triggered abilities look their modes up in the wrong index space (`hullbreaker_horror`, `glissa_sunslayer`, `junji_the_midnight_sky` — two resolve `Effect::Nothing`, junji executes mode 0 forever); zero deck-legal blast radius, filed as **OOS-DX35-1**. **`OOS-DP10-5`'s standing "sweep for others not yet found", inherited unrun by nine batches, was EXECUTED**: one new real member — `Effect::CounterUnlessPays` discards its `cost`, so CR 118.12a's "unless its controller pays" is never offered, live on **7 deck-legal `Complete` defs** (**OOS-DX35-3**) — and one checked-and-CLEAN, recorded because it proves the sweep read each discard rather than counting them. Tests **5,058 → 5,097 / 0 / 5** (+39, 63 targets, byte-exact set difference, 0 leavers / 0 removals / 0 renames; this cell said +33 for one commit, superseded by the close-out's own re-take). Filed **OOS-DX35-1..10**; `-1`/`-2` collided because the batch's two delegated halves both claimed `-1`, which is recorded in `-2`'s own row. See `memory/primitives/pb-DX35-execution-notes.md`.)*|
| ~~**13**~~ | ~~**PB-DX36**~~ **✅ SHIPPED** (`scutemob-228`, 2026-09-04) *(standing, v3 rank 23)* | `WhenDealsDamage` + the dead `combat_only` arm | **OOS-CARDS2-6** — **FILED** (it had no registry row) **and CLOSED**, both halves | **CORRECTNESS — worse than filed**: `combat_only` is read **only by the hasher**, so `true` and `false` are behaviourally identical | **1 flip** (`exalted_angel`), not "1-2 + family"; repairs `sigil_of_sleep` (deck-legal `Complete`, marker-less, silently drops a printed trigger) | **both** (HIGH) — via `EffectAmount`; `TriggerCondition` is **off-wire**, which the v3 row had backwards  *(**Every cell held.** Wire **PROTOCOL 41 → 42 / HASH 82 → 83**, ONE bump each, both PROBED at stage 0 and predicted in writing before any production line, with both closure type counts predicted and confirmed UNCHANGED at 98 / 132. Coverage **1,138 → 1,139 = 63.2%**, the one flip NAMED before regeneration. **Three things the memo did not have.** (1) The flag is ANIMATED, not deleted: an inverse oracle census finds `breath_of_fury` printing the combat-only shape, so 0 declared users but 1 printed one. (2) The recipient axis — `DamageRecipient { Any, Player, Opponent }` — is the half no document names, and it is what repairs `curiosity`/`ophidian_eye`'s *"an opponent"* approximation; it went on `TriggerEvent` rather than `TriggeredAbilityDef` because that struct has **190 exhaustive literals across 44 files**, `OOS-DX35-1`'s figure reproduced exactly. (3) The brief's CR 603.10a cite for *"that much"* is **wrong** (that rule is look-back zone-change triggers); shipped against CR 603.2c + CR 608.2h/113.7a. Filed **OOS-DX36-1..9**.)* |
| **14** | **PB-DX52** | Bolt Bend's printed "or ability" half is unreachable | **OOS-DX25b-1** + **OOS-DX25b-5** (rider) | correctness — 1 deck-legal `Complete` | 0 flips | **PROTOCOL + HASH** (HIGH) — needs a target id space for ability stack entries that does not exist |
| **15** | **PB-DX39** *(standing, v3 rank 33)* | source-relative filters through LKI | **OOS-DX5-3** + **OOS-DX5-7** residual | correctness, narrow | 0 flips; `umezawas_jitte` (deck-legal `Complete`, live-wrong) + `mardu_ascendancy` (`partial`) — reproduces exactly | **HASH if a snapshot must be stored; none if derivable at resolution** (LOW-MED) |
| **16** | **PB-DX53** | CR 508.6 raid gate clobbered by re-declaration | **OOS-DX21-1** (re-scoped by its own review to `windbrisk_heights` alone) | correctness — 1 deck-legal `Complete` | 0 flips | **HASH** (MED) |
| **17** | **PB-DX54** | a resolving spell cannot be its own redirect victim | **OOS-DX25c-6** | correctness (under-permission) — 2 deck-legal `Complete` | 0 flips | **HASH** (MED) |
| **18** | **PB-DX42b** *(standing, re-decided in §3.1 — NOT carried)* | CR 613.1d layer-bounded condition queries | **OOS-ADJ-1** ≡ **OOS-DX19-2** (+ **OOS-DX19-1** residue) | **CORRECTNESS — 7 deck-legal `Complete` pairs, every one needing a conjunction** | 0 flips | **none** (HIGH) expected — gate-execute both. **Preconditions**: re-word `OOS-DX19-2` per `OOS-ADJ-3` (a cross-layer bounding problem framed as a CR 613.8b fixpoint — a worker taking it literally builds the wrong thing); and if PB-DX9 has shipped, re-measure the supply census first |
| **19** | **PB-DX55** | the whole bot/human refusal surface, which is now exactly three seeds | **OOS-SIM6-3** (76 of 105) + **OOS-SIM5-3** (27) + **OOS-SIM5-5** (2) | **GATE INTEGRITY / AGENCY — and it is the shipped alpha's own 422** | 0 flips; **100% of the measured refusal surface**, residue zero. A human's mana-cost activation in the browser still 422s | **none** (HIGH) — simulator-internal. Cite correction: `local_game.rs:738` → **`:1111-1114`** |
| **20** | **PB-DX56** | make the two live fuzz violations diagnosable, then diagnose them | **OOS-FB1-1** *(prerequisite)* → **OOS-DX32-1** (84, 79.2% of HARD) + **OOS-DX22-8** (22, doubled) | **GATE INTEGRITY** — `--stop-on-error` halts on an undiagnosed class | 0 flips | **none** (HIGH) |
| **21** | **PB-DX57** | the gate-widening cluster — six gates that report success while checking less than they claim | **OOS-DX28-1** + **OOS-DX28-5** + **OOS-ADJ-2** *(the PB-DX42a widening: `t7` 1→8 variants, `t9`'s missing `TargetFilter` half — §2.3)* + **OOS-DX26-3** + **OOS-DX21-7** + **OOS-DX28-6** *(census first: no gate exists for the "an in-def comment asserts a resolution mechanism the code does not use" shape, and PB-DX27's blocker-note sweep does not cover it; population UNMEASURED, so stage 0 is the census)* | **GATE INTEGRITY** | 0 flips; ~8 lines for the DX42a half alone, which is what makes PB-DX42b's premise actually gated (§2.3) | **none** (HIGH) — test-only |
| **22** | **PB-DX9** *(standing, v3 rank 15)* | multi-card search + the inert-field family | **OOS-DP9-3** (+DP9-2/-4/~~-9~~, ~~**DP10-5 ≡ OOS-DX4-5**~~ — **↻ BOTH CLOSED by PB-DX35 (`scutemob-227`, 2026-09-04); the v3 memo's constraint that `OOS-DX4-5` ride THIS row or PB-DX35 but not both is DISCHARGED, and it rode PB-DX35.** `Effect::LookAtTopThenPlace.optional` is no longer inert, so this row's "inert-field family" half is down to **`Effect::SearchLibrary.reveal` (`OOS-DP9-9`) alone** — PB-DX35 executed `OOS-DP10-5`'s own sweep and confirmed that is the only surviving member of the filed family, while adding ONE new one the family never named: `Effect::CounterUnlessPays.cost` (**OOS-DX35-3**, 7 deck-legal `Complete` defs). **Re-scope this row before dispatching it**: its multi-card-search half (`OOS-DP9-3`, 2 flips) is untouched and is now the whole of it) | capability / card yield | **2 flips** (`tooth_and_nail`, `buried_alive`) — and see sequencing constraint 2: this row is what promotes `the_world_tree` and moves PB-DX42b's premise | **PROTOCOL + HASH** (HIGH) |
| **23** | **PB-DX38** *(standing, v3 rank 30 — promoted)* | the CR-citation rot sweep | **OOS-UI3-1** + **OOS-DX2-6** + **OOS-DX25-6** | doc hygiene (Architecture Invariant 8) | 0 flips; **10** wrong cites across 11 lines in `events.rs` (not 9), CR 726 **76 across 27 files**, CR 701.5-for-*counter* **333** occurrences of which 4 name a nonexistent `701.5g`, and a new mechanical derivation finds **206 candidate mismatches across 97 files** | **none** (HIGH) |
| **24** | **PB-DX58** | one engine bug, three equip promotions | **OOS-DX27-4** + **OOS-DX26-6** *(merged — §1d)* | correctness + **card yield** | **the v3-era "3 flips with no new DSL" is refuted**: 0 flips with no new work, **2 flips behind one engine fix** (`resolve_cda_amount` resolving the controller from the *equipped creature*, CR 108.5/611.2c-wrong), 1 behind a new enum variant. The population is **11** non-`Complete` equip defs, not 10 | **none** (HIGH) — the fix reads the effect's controller, already available |
| **25** | **PB-DX59** | adjudicate the 80-def `BASELINE` and give the DSL a working optionality flag | **OOS-DX8-1** + **OOS-DX8-3** *(merged)* | **card yield + agency** | 80 entries, **all 80 carrying `None` in the reason slot** — zero adjudicated since the freeze. Channel split `may` 72 / `up_to` 10 / `choose` 2. **The DSL's only optionality flag is inert**: `optional: bool` exists on one `Effect` variant and its sole consuming arm destructures it `optional: _`, so the ratio is 0:72, not 5:72 | **PROTOCOL + HASH** (MED) if a real channel ships |
| **26** | **PB-DX60** | Urza's Saga, and the two DSL gaps its TODO was hiding | **OOS-RR4-2** *(filed by this task)* | card yield (1) + **DSL-gap discovery** | **1 flip** (`urzas_saga` `partial → Complete`, 1,136 → 1,137, no rounding change) and **0** currently-wrong deck-legal defs repaired — its value is **enabling**: it makes corner case #36 constructible and gives PB-DX49 its headline fixture. Chapter I is authorable **today with zero engine lines**; chapters II/III are a `TokenSpec` CDA and a printed-mana-cost predicate, both general DSL gaps that should be split out and ranked on their own populations | chapter I **none** (HIGH); chapters II/III **PROTOCOL likely** (MED) — the `OOS-DX28` `TargetFilter.owner` precedent |
| **27** | **PB-DX33** *(standing, v3 rank 20)* | route the TUI through `params.rs` | **OOS-SIM1-2 ≡ OOS-SIM2-7** + **OOS-UI2-5** + **OOS-DX6-5** + **OOS-DX23-3** | correctness (TUI-only, latent) | 0 flips; **9** hand-built sites, not 5; 17 of 26 `LegalAction` variants unreachable from the TUI | **none** (HIGH) — routing. **⚠ `OOS-UI2-5`'s registry row is WRONG, not stale**: the TUI has never routed a cast, so a human gets a refusal, not a silent default — **routing the `CastSpell` site is what would CREATE the defect** on 13 deck-legal `Complete` defs. Fix the row before dispatch (§6) |
| **28** | **PB-DX31** *(standing, v3 rank 18)* | the mana solver's model | **OOS-SIM2-1** + **-2** + **-3** + **-4** | capability — bot play strength | 0 flips; deck-legal figures are **12** mana-component and **9** scaled (the v3 row's 36/20/9 are `(def × ability)` rows with no completeness filter) | **none** (HIGH) |
| **29** | **PB-DX13** *(standing, v3 rank 26)* | target-scoped filters | **OOS-OS7-1 R1** + **OOS-RS-5** *(neither has a registry row — file first)* | correctness + capability | **2 flips** of 4 named, all in-place | **PROTOCOL + HASH** (HIGH) |
| **30** | **PB-DX34** *(standing, v3 rank 21 — SPLIT)* | `Command::DeclareAttackers` — the X channel, then the boxing | **OOS-DX6-1** + **OOS-DX6-2** *(the boxing, `OOS-DX6-4`, splits out)* | correctness (latent) + refactor debt | 0 flips; `propaganda`/`ghostly_prison` are deck-legal `Complete` but carry `x_count: 0`, so live reach is **0**. **340** occurrences / ~335 constructions, **330 of them in tests** | **PROTOCOL** (HIGH) **+ a HASH bump the v3 row omits** for `GameRestriction` |
| **31** | **PB-DX12** *(standing, v3 rank 25)* | multi-count sacrifice cost | **OOS-OS6-1** *(no registry row)* | capability | **3 in-place flips + 1 new authoring** — two of the four named defs have no def file | **PROTOCOL + HASH** (HIGH). Enforcement surface grew from one type to ~11 files |
| **32** | **PB-DX30** *(standing, v3 rank 17)* | CR 704.3 — SBAs are not checked on a priority pass | **OOS-M11-7** | correctness (self-healing window) | 0 flips. **The row's "22 `Complete` sac-for-mana defs" is a token-creator census wearing the wrong label** — literal reading = 1; honest floors are 82 deck-legal with an activation-time sacrifice cost and 13 with `spell_additional_costs` | PROTOCOL none (HIGH); **HASH likely MOVES** (MED) — the row says none |
| **33** | **PB-DX11** *(standing, v3 rank 24)* | `WouldDraw` widening | **OOS-DP5-6** (+DP5-8, -9) | capability | **5** blocked defs, not 3; **all `inert`, none deck-legal**; only **2** reachable by the stated minimum widening; **0 `WouldDraw` defs corpus-wide** | **PROTOCOL + HASH** (HIGH) |
| **34** | **PB-DX10** *(standing, v3 rank 16)* | PB-DP8b — modal triggered abilities | **OOS-DP3-4** + **OOS-DP8-7** | agency / CR 700.2b | **2** deck-legal `Complete`, not 4; the headline `min_modes: 0` case has **0** deck-legal members and rider **`OOS-DP8-3` has 0 corpus members at all** | **PROTOCOL + HASH** (HIGH) |
| **35** | **PB-DX40** *(standing, v3 rank 34)* | the two micro card-authoring items | **OOS-DX4-3** + **OOS-DX4-4** | capability, micro | **2 new authorings**, not flips; 0 defs carry Decayed and `wastes.rs` is absent. Corpus is **1,803**, not the row's 1,804 | **none** (HIGH); every def-count pin moves — budget two reconciliation passes |
| **36** | **PB-DX41** *(standing, v3 rank 35 — promoted)* | the SR-38 residue the enumeration missed | **OOS-SIM1-3** + **OOS-SIM1-1** | correctness (narrow, safe-failing) | 0 flips; 2 unmirrored `GameRestriction` variants + split second (0 simulator references) | **the v3 row's "PROTOCOL" is FALSE → NONE** (HIGH). `CastSpellData` already carries both payment fields; the missing one is on the simulator-local `LegalAction`. **Its "split it out unless it rides PB-DX34's bump" dependency is void** |
| **37** | **PB-DX61** | the corpus's unexamined-marker population | **OOS-DX26-8** + **OOS-RR3-1** *(merged — same population, filed twice)* | structural / marker integrity | **964 of 1,803 (53.5%)** defs declare no marker at all and have never been reviewed. Trend 966 → 965 → **964**: monotone down, so it is a **ceiling**. The memo's own literal method returns **963** and is wrong by one (`misdirection.rs`'s comment) | **none** for the review; **PROTOCOL + HASH** (HIGH) for `the_reaver_cleaver`'s missing `…ToPlayerOrPlaneswalker` variant |
| **38** | **PB-DX14** *(standing, v3 rank 28)* | back-face starting loyalty | **OOS-OS4-1** (+**OOS-RS4-3**) *(neither rowed)* | capability | **the affected population is ZERO** — 15 defs have a `back_face` and none is a planeswalker. The "2 flips" are **new authorings**. A second read site nobody named: `replay_harness.rs:4016` | **the v3 row's "PROTOCOL + HASH" is WRONG → NONE** (HIGH): `CardFace` is off-wire and has no `HashInto` |
| **39** | **PB-DX16** *(standing, v3 rank 31)* | edgar return-transformed | **OOS-OS4-3** *(not rowed)* | capability, micro | **1 new authoring** — `edgar_charmed_groom.rs` does not exist — not "1 flip" | **PROTOCOL + HASH** (HIGH) |
| **40** | **PB-DX17** *(standing, v3 rank 32)* | attacked-player trigger family | **OOS-OS7-1 R2** *(the row says "R2+R3"; `pb-plan-OS7.md:270-295` defines only R1 and R2 — **R3 was invented by an earlier triage and copied forward three times**)* | capability | **1 new card** (`karazikar`, unauthored); 1 gated def mentions the printed phrase | **the v3 row's "none" is UNSAFE** (MED) — the runtime `TriggerEvent` is reachable from `Characteristics`, a closure root |
| **41** | **PB-DX37** *(standing, v3 rank 27 — DEMOTED to rider)* | the `affected_set` discriminator | **OOS-DX5-1** + **OOS-DX5-8** | gate integrity (latent) | **premise dormant, measured**: 23 production creation sites, **byte-identical per-file to PB-DX5's collect `f20823b1`** — three subsequent batches moved it by zero | **none** (HIGH). **Fold into any batch touching `layers.rs`**; it does not warrant a slot |

**Riders with no slot of their own** — each names its host, and none should be dispatched alone:

| rider | host | why |
|---|---|---|
| **OOS-DX29-4** + **OOS-DX29-10** | PB-DX44 | hybrid/Phyrexian pips charged free in **nine** additive arms (the row named six and omitted Kicker, Buyback and the Spree arm that is PB-DX44's own subject). 0 members today; the gate's recall bound is stated at `pb_dx29_additional_cost_roster.rs:148-167` and covers only 6 of 10 cost kinds |
| **OOS-DX29-11** + **OOS-DX29-17** | PB-DX44 | escalate vs mode selection answering the same question twice; over-announced escalate charged in full and silently clamped. 0 deck-legal members |
| **OOS-DX29-6**, **-13**, **-15** | PB-DX44 / PB-DX57 | four mechanics sharing one `AdditionalCost::Sacrifice` with no arbitration (**five** consumers, not four); a wrong `CardId` producing a silently rider-less offer with no gate; the entwine decision made twice from two sources |
| **OOS-DX29-7** | PB-DX60 | `dawns_truce` is authorable; +1 flip |
| **OOS-DX27-3**, **-6**, **-7** | any card-def batch | the emblem combat-damage dispatch gap (0 corpus reach); the 357-ceiling opaque-note ratchet; **two of `OOS-DX27-7`'s four claimed cite corrections were recorded and never applied to the source** (`fell_stinger.rs:33`, `crucible_of_the_spirit_dragon.rs:37`) |
| **OOS-DX24-1**, **-7** | PB-DX15a | one conjunct at `abilities.rs:10196`; the CR 603.10a look-back set being coarser than one batch at its caller |
| **OOS-DX25-4**, **OOS-DX25b-4** | PB-DX52 / PB-DX54 | `SpellCountered` emitted for 2 of 25 ability kinds on both paths; `deflecting_swat`'s `must_change: false` deterministic no-op |
| **OOS-DX20-3/-4/-6**, **OOS-DX26-2** | any card-def batch | aspirationally-wrong TODOs (two in `polymorphists_jest.rs`, not one) and `commanders_plate`'s genuinely-absent "is your commander" predicate — **`blackblade_reforged` is no longer a member**, PB-DX27 authored its CR 702.6c line |
| **OOS-ENG1-6** | any `effects/mod.rs` batch | `Effect::MillCards` resolves its count `as usize` with no `.max(0)`, **nine lines above** `CreateToken`'s `raw_count.max(0)`. One line |
| **OOS-ENG1-9** *(take fix (b))*, **OOS-G2-2**, **OOS-UI6-2**, **OOS-UI6-5**, **OOS-UI6-6** | any play-server batch | a name captured at ask time (wire-neutral; **6** deck-legal defs, not 4); the mulligan seed that makes every mulliganed game's bug report unreproducible; `all_cards` populated at one construction site with no roster gate; **the Invariant-7 raw-read gate is still a 7-needle enumerated set** (`main.rs:5088-5145`) rather than type-level, so it can only see the channels someone thought of — `OOS-UI6-5`, and no PB-DX batch has touched it since UI-6; `library_look_cards` open-coding CR 121.1 |
| **OOS-SIM6-1**, **OOS-SIM6-2**, **OOS-ENG2-9**, **OOS-DX23-3**, **OOS-DX23-6** | PB-DX55 / PB-DX33 | narrowing `flatten_cost_into`; two note-vs-code shapes; a superseded prose arm; the TUI dredge parity gap; `--all-targets` appearing only in CLAUDE.md narrative and in neither the runner agent's step nor the Milestone Checklist |

**Deliberately NOT ranked, and the reason stated rather than left as an omission:**

- **`OOS-DX29-1` (Assist)** — the row frames it as unilaterally spending an opponent's mana. At HEAD
  no in-tree client can announce it: `params.rs` never emits it and `api.rs:1065` refuses it at the
  400 boundary. The CR 702.132a consent violation is reachable only from a hand-built `Command`.
  What is live is the **agency** loss on one deck-legal `Complete` def (`huddle_up`). Band 4.
- **`OOS-DX27-2` (Exploit)** — real, but **0 deck-legal members**; all three Exploit defs are
  `partial`. Band 4, and it needs a wire bump for the choice half.
- **`OOS-SIM5-1`** — bots target the lowest `ObjectId`, which is seat 1, in every player-eligible
  slot of every game. Game *character*, not correctness. Band 4, above `proliferate`.
- **`proliferate`** — **23** deck-legal `Complete` defs, confirmed four independent ways
  (`OOS-DP10-6`'s registry figure of 25 is a stale 2026-07-27 snapshot that drifted *down* through
  honest demotions). Still the highest-count agency row and still unranked, for v2's original
  reason: it is agency restoration on otherwise-correct cards, and eighteen live-wrong entries sit
  above it. Carried forward for the third triage running.
- **`OOS-G6-1`** — ~70 defs carry an unofferable alternative cast mode; `AltCastAbility` measures
  **36 defs / 27 `Complete`** and all **4** `CommanderFreeCast` defs are `Complete` and
  un-free-castable. **Milestone-scale**, as its own filing says. Parked, not ranked.

---

## 5. Parked — real, do not queue

**Sixty-three seeds, and all sixty-three are named below** — because a verdict distribution whose
PARKED bucket is not enumerable is not auditable, which is the standard §1b sets for itself.
Grouped by the reason, since the reason is what a future reader needs.

| group | items | n | why parked |
|---|---|---:|---|
| **latent with a measured population of ZERO** | `OOS-DX20-1`, `-2`, `-8`, `-9`; `OOS-DX22-4`, `-5`, `-6`; `OOS-DX23-2`, `-8`; `OOS-DX24-3`, `-8`; `OOS-DX25-2`; `OOS-DX25b-2`; `OOS-DX25c-1`, `-2`, `-3`, `-4`; `OOS-DX26-1`; `OOS-DX28-3`, `-4`; `OOS-DX29-5`, `-16`; `OOS-SIM4-1`, `-3` | 24 | each **re-measured**, not assumed. Re-rank the day the first member is authored. `OOS-DX22-4`'s population is 0 **by construction** (`random_deck`, its only caller, draws from the same `all_cards()` map); `OOS-SIM4-1`'s is 0 because the TUI has **no redeal call site at all** (`grep -rn redeal tools/tui/src/` finds only a pointer comment) |
| **structural residuals, disclosed at their own declaration** | `OOS-DX7-1`, `-2`; `OOS-DX8-2`, `-4`, `-8`; `OOS-DX26-4`; `OOS-ENG1-1`, `-2`, `-3`, `-7`, `-8`, `-10` | 12 | each is a **stated bound**, not a defect. `OOS-DX7-1` is the sharpest: 9 `Effect` discriminant collisions across 18 variants, both ratchets green, and **no 10th pair has appeared despite PROTOCOL moving 96 → 98** |
| **M10a-shaped — need an engine-side notion that does not exist yet** | `OOS-ENG2-6`, `-7`, `-8`; `OOS-UI6-1`, `-3`, `-4`; `OOS-UI5-1`, `-2` | 8 | a "publicly revealed / revealed to whom" notion, or TUI-surface items whose home is the M10 client work. (`OOS-UI3-2`/`-4` are the same class but are v3-census seeds, inherited below rather than counted here) |
| **instrument, policy and coverage items with no live yield** | `OOS-FB1-4`, `-7`, `-8`; `OOS-SIM6-4`, `-5`, `-6`; `OOS-SIM5-2`; `OOS-DX23-4`; `OOS-DX24-5` | 9 | forage/sacrifice_self unmirrored, the TUI's missing picker, `x_value: 0` at offer time, `UpToN` slots announced empty by design, a bot **policy** preference that is not a correctness claim (`DX23-4`), and a probe that does not discriminate what its name says (`DX24-5`) — all latent with **0 measured traffic** |
| **milestone-scale, as their own filings say** | `OOS-G6-1`, `OOS-G2-3`, `OOS-UI5-4`, `OOS-G10-1` | 4 | `G6-1`: 36 `AltCastAbility` defs / 27 `Complete`, and all 4 `CommanderFreeCast` defs are `Complete` and un-free-castable. `G2-3`: `Command::TakeMulligan`/`KeepHand` are unreachable — `builder.rs:62` defaults `turn_number: 1` and `.turn_number(0)` has **0 hits workspace-wide**. `UI5-4`: the R7 frontend harness, designed twice and built zero times. `G10-1`: the HTTP-fuzz instrument, proposed and never built |
| **discharged by measurement rather than by a fix** | `OOS-SIM5-4`, `OOS-G5-3` *(the same deferral, filed under two IDs)* | 2 | it deferred an offer-suppression filter worth **1 of 166** refusals at filing; at HEAD it is worth **0 of 105** (§2.6), and its recorded blocker was already refuted in-source by `targeting.rs:83-91`. Parked with a **better** justification than it had when filed |
| **re-scoped rather than acted on** | `OOS-DX27-9` | 1 | its "PB-DX42b's rank premise is false" headline does not hold on the deck-legal axis the rank used (§2.2); the row is corrected in place and the durable half is carried as a sequencing constraint on PB-DX9 |
| **diagnosability only, no state effect** | `OOS-DX19-4` | 1 | the depth tripwire and saturation assert are still absent; the crash they would have diagnosed is closed |
| **filed in execution notes as deferred LOWs** | `OOS-DX28-9`, `-10` | 2 | `ChooseObject` produces no decision-coverage row; the CR 608.2b fizzle direction is proven structurally rather than end-to-end |
| **inherited from v3 §5 unchanged (not part of the 208)** | `OOS-DX1-1`+`-2`, `OOS-DX5-4`/`-5`, `OOS-CARDS2-1`/`-2`/`-5`, `OOS-UI3-2`/`-3`/`-4`, and v3's own inheritance of v2's §5 in full | — | nothing in this census closes or re-activates any of them. Read v3 §5 for the per-item reason |
| **the highest-count agency row, still unranked (not part of the 208)** | **`proliferate`** | — | **23** deck-legal `Complete` defs on PB-DP9's `AnswerEffectChoice` channel, confirmed four independent ways. Agency restoration on otherwise-correct cards, with eighteen live-wrong entries above it; carried forward for the third triage running. **The registry's "25" (`:1209`) is a stale 2026-07-27 snapshot** that drifted *down* through CARDS-2's and PB-DX27's honest demotions — corrected in the row (§6) |
| **total, from the 208** | | **63** | matches §1b's PARKED column exactly |

**One count worth carrying forward for whoever next reads §5**: the still-auto-chosen union is **80**
and was unmoved by PB-DX8, PB-DX28 and PB-DX29. The only parked row to leave the set since v3 is
`discard_cards` (13 → 12), closed by ENG-1.

---

## 6. Source-doc updates applied by this task

**Zero engine / simulator / tool / card-def code changed.** `git diff --numstat` over `crates/` and
`tools/` is **empty** — executed, not asserted. Tests (**4,721 / 0 / 5**), coverage
(**1,136/1,803 = 63.0%**), PROTOCOL (**37**) and HASH (**76**) are untouched **by construction**:
this task edited only `memory/`, `docs/` and `CLAUDE.md`, so there is nothing for a gate to
recompute. That is a stronger claim than "the gates were re-run and were green", and it is the
honest one for a doc-only task.

1. **`memory/primitives/seed-rerank-2026-08-14.md`** — this file (new). The authoritative queue.
2. **`memory/primitives/seed-rerank-2026-08-02.md`** — **§4 banner'd SUPERSEDED** with a pointer
   here; the header's "This document is the authoritative primitive queue" claim scoped to
   §1-§3, which remain canonical. No shipped row edited, no history rewritten — the v2→v3
   precedent exactly.
3. **`docs/audits/decision-point-audit.md`** — three new rows filed (**`OOS-RR4-1`**,
   **`OOS-RR4-2`**, **`OOS-RR4-3`**, §1g), and corrections applied **to the rows themselves**
   rather than only recorded here:
   - **`OOS-DX27-9`** — the deck-legal-vs-total distinction added; the "the rank premise is false"
     framing replaced with what is actually true (§2.2).
   - **`OOS-DX28-1`** — records that `t9` pins `ContinuousEffectDef` and **not** the `TargetFilter`
     fingerprint that went blind (§2.3).
   - **`OOS-DX28-8`** — its stated mechanism ("the cancelling *target* must be in a different
     ability") corrected; PB-DX28's own `/review` refuted it by execution and only the in-source
     doc was updated.
   - **`OOS-UI2-5`** — the row is **wrong, not stale**: the TUI has never routed a cast, so a human
     gets a refusal rather than a silent default, and **routing the `CastSpell` site is what would
     create the defect**. v3 recorded this at its §1c and the registry was never updated.
   - **`OOS-DX23-3`** — "the TUI never routes through `params.rs`" is false since SIM-6
     (`a878ca26`).
   - **`OOS-DP10-6`** — its `proliferate` figure of 25 is a stale 2026-07-27 snapshot; the measured
     value is **23**.
   - **`OOS-DX22-8`**, **`OOS-DX32-1`**, **`OOS-ENG1-1`**, **`OOS-ENG1-2`**, **`OOS-ENG1-9`**,
     **`OOS-DX27-3`**, **`OOS-DX27-4`**, **`OOS-DX29-13`** — rotted line cites corrected (§2.10);
     `OOS-DX22-8` and `OOS-DX32-1` additionally carry their **re-measured** counts (22 and 84), a
     note that the two moved in *opposite* directions, and the `OOS-FB1-1` prerequisite.
   - **`OOS-SIM6-3`'s cite correction could NOT be applied to a row, because it has no row.**
     Its filed cite `local_game.rs:738` is `:1111-1114` at HEAD and its filed figure "62 of 113"
     is **76 of 105**, and both corrections live only in §2.6 and on §4 rank 19. **This is §1a's
     29%-unrowed finding biting inside this task's own fix list**: a dispatcher who consults the
     registry — the document `dispatch hygiene 5` names as ground truth — will read a dead cite
     and a stale figure for the seed that is **72% of the entire measured refusal surface**.
     Recorded here rather than silently dropped; filing the row is one line of PB-DX55's stage 0.
   - **Markdown render**: the two rows this task edited that carried **unescaped `|` inside code
     spans** (`OOS-DX32-1`'s `has_lost || has_conceded`, `OOS-DX29-13`'s `and_then(|cid| …)`) are
     escaped, so they render as 4-cell rows again. **Ten further pre-existing rows have the same
     defect and were NOT touched**, because this task has no mandate to edit rows it is not
     correcting: registry lines **1140** (`OOS-DP5-1`), **1194** (`OOS-DP9-10`), **1198**
     (`OOS-DP9-14`), **1230** (`OOS-DX4-1`), **1235** (`OOS-DX4-6`), **1236** (`OOS-M11-5`),
     **1248** (`OOS-M11-6`), **1257** (`OOS-CARDS1-2`), **1357** (`OOS-DX24-5`), **1423**
     (`OOS-DX29-2`). GFM does not protect pipes inside backticks; each renders as a 6- or 7-cell
     row in a 4-column table. Recorded as a residual rather than silently swept — it is a
     one-line-each rider for any batch that opens the registry.
4. **`docs/audits/mtg-characteristics-recursion-adjudication.md`** — `OOS-ADJ-2` recorded
   **partially discharged and re-scoped to the seven unpinned variants** in §6, rather than being
   left to read as closed by the rider that cites it.
5. **`CLAUDE.md`** — Current State's queue pointer repointed here; the "next dispatch: coordinator's
   call" banner cleared and replaced with **PB-DX43**; the stale "36 corner cases: 32 COVERED,
   4 GAP" corrected to the measured **35 COVERED / 1 GAP**.
6. **`memory/workstream-state.md`** — the W6 row repointed and its stale "filed `OOS-DX29-1..14`"
   corrected to **`1..17`**; the user-directed Blood Moon coordinator flag annotated **DISCHARGED**
   with the four particulars in which reading the code refuted it.

**Three seeds filed by this task** (§1g): `OOS-RR4-1`, `OOS-RR4-2`, `OOS-RR4-3`. All three were
grep-confirmed absent from the registry before filing (dispatch hygiene 5). They are rowed in
`docs/audits/decision-point-audit.md` §8.1 rather than only in this memo — the `OOS-RR3-2`
precedent, and the correction to v3's choice, since RR3-2 had to be retro-rowed by PB-DX27 anyway.

### `/review` cycle — 0 HIGH / 3 MEDIUM / 5 LOW, all 8 taken

The reviewer had a shell and used it: it re-ran the census command and reproduced
**488 / 79 / 196 / 213 / 208** exactly, checked §1b's ledger in **both** dimensions (27 family rows
against their `n`, five verdict columns against 25/45/32/63/43), byte-diffed the §4 ordering rule
against v2 `:876-883` and v3 `:782-786` and confirmed it **verbatim**, executed
`pb_dx42a_continuous_condition_roster` (10/10, printing 386 / 18 / 16 / 9 — every §2.2 figure
reproducing), and confirmed `git diff --numstat main -- crates/ tools/` empty. All six criteria
PASS. The eight defects it found are the item-level trace under the arithmetic, and every one is
this document failing its own standard:

| # | sev | finding | taken |
|---|---|---|---|
| 1 | MED | **§6 promised a registry correction to `OOS-SIM6-3`, which has no registry row** — contradicting §1a's own 29%-unrowed finding four hundred lines earlier | §6 rewritten to say the correction **could not** be applied and lives only in §2.6 and §4 rank 19, with the consequence stated: a dispatcher consulting the registry reads a dead cite and a stale figure for the seed that is **72% of the whole refusal surface** |
| 2 | MED | **§1b claimed "every one appears in §4" and three of the 45 appeared nowhere** (`OOS-DX28-6`, `OOS-UI6-5`, `OOS-DX21-2`); `OOS-ADJ-2` was carried substantively but never named by ID | all four placed — `DX21-2` into rank 11 (same combat offer surface), `DX28-6` and `ADJ-2` into rank 21, `UI6-5` into the play-server rider row — and §1b's claim reworded to say *how* each is placed |
| 3 | MED | **§5 said "sixty-three seeds" and enumerated 47** — sixteen PARKED seeds carried a ledger verdict and were named nowhere, so "every one of the 208 carries exactly one verdict" was not auditable for that bucket | §5 rewritten as a grouped table with an `n` column; **all 63 enumerated**, the column sums to 63, and the enumeration now matches §1b's PARKED column by construction |
| 4 | LOW | §1d cited **`§2.12`, a section that does not exist** — the evidence cell for the `OOS-DX20-10` merge | repointed at §4 rank 9, which carries the three-site fix census |
| 5 | LOW | §6 presented itself as the complete source-doc ledger and **omitted the `OOS-DX32-1` edit** this branch made | added, with its re-measured 84 / 79.2% figures |
| 6 | LOW | two registry rows this task edited render as 6-cell rows — **unescaped `\|` inside code spans** | both escaped; **ten further pre-existing rows have the same defect and were deliberately NOT touched**, each named by line in §6 as a rider rather than swept into a re-rank's diff |
| 7 | LOW | **three different verdict words for the same edit** — §3.1 said "stays OPEN", then "not discharged", while the shipped adjudication note says "PARTIALLY DISCHARGED, re-scoped" | unified on the shipped wording, so the memo and the source of record agree |
| 8 | LOW | `seed-rerank-2026-08-02.md:17` still read **"This document is the authoritative primitive queue"**, scoped only by an adjacent banner | struck in place with the reason for striking rather than deleting: it is what the document said while PB-DX19..PB-DX29 were dispatched from it |

**And one the reviewer did not raise, found while checking its arithmetic**: §0 published
"correctness repairs on **~45** already-`Complete` deck-legal cards" with no derivation — the exact
shape rule 1 of this document's own method forbids. It is **53**, and it is a *sum of band-1 row
populations with overlaps not deduplicated*, which is now stated along with the per-rank addends
and the fact that PB-DX47's then-unconfirmed 18 were excluded. **↻ 2026-09-02: PB-DX47's probe CONFIRMED them and they re-derived at HEAD exactly, so the honest figure is 71** (`scutemob-218`).

**The pattern across 1, 2, 3 and 5 is one thing**: every published *aggregate* in this document
reproduced, and four of its *item-level traces* were short. An arithmetic that checks out is not
an enumeration that checks out, and only the enumeration survives being handed to a dispatcher.

### The census-integrity instruction for the next re-rank

v3 left one and it was right; this triage adds three, each earned.

1. **Derive the population by set difference against the previous census's own table**, not by a
   date. `S = ALL − V3 − LEGACY`, published in §1a with its exact command below. A date cutoff
   failed v2 and then failed v3 for the same reason.
2. **Do not treat the registry as the population.** It is the *filing record for one kind of work*.
   **68** seeds relevant to this queue have no row in it (61 post-v3 in §1a, plus the 7 standing-row
   seeds in §3.2). Run all three passes — the registry, the handoff prose in
   `memory/workstream-state.md`, and the per-batch execution notes in `memory/primitives/` — and
   reconcile.
3. **Expand every range against its own filing document, not against the registry.** With 29% of
   the population unrowed, the registry has no authority to arbitrate a range. That is how
   `OOS-SIM6-6` was found outside CLAUDE.md's "`OOS-SIM6-1..5`".
4. **Publish the command.** The one this census used:

```sh
grep -rhoE 'OOS-[A-Za-z0-9]+-[0-9]+[a-zA-Z]?' \
  --include='*.md' --include='*.rs' --include='*.py' --include='*.js' --include='*.svelte' . \
  | sort -u
```

and, for the registry side:

```sh
grep -oE '^\| \*\*OOS-[A-Za-z0-9-]+' docs/audits/decision-point-audit.md | sort -u
```
