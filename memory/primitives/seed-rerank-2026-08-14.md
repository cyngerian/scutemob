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

**The 45 queue candidates, listed so §4 can be checked against them** — `OOS-` prefixes dropped:
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
| **`OOS-DX24-9` ≡ `OOS-DX27-5`** | **The same defect, filed twice by two batches five days apart, and neither row cites the other.** Both are `Effect::MayPayThenEffect` being pay-when-able: DX24-9 frames it as CR 118.12 optional cost engine-chosen on `nether_traitor`; DX27-5 frames it as a DSL/corpus consistency question. **Two independent measurements of this task both returned 11 deck-legal `Complete` defs** for the same population, which is what proves they are one thing. | `effects/mod.rs:4657-4702` (unconditional `try_pay_optional_cost`, no suspension); DSL doc `card_definition.rs:1786-1791` |
| **`OOS-DX29-9` + `OOS-DX29-12`** | One defect surface. DX29-9 says the right half of a split card is uncastable; DX29-12 says a fused cast cannot announce the right half's targets and PB-DX29 **gated** the fuse offer rather than shipping it. Verified: the gate (`legal_actions.rs:2752`, predicate `:2901-2906`) fires on **100% of members** because both deck-legal fuse defs declare non-empty right-half targets (`turn.rs:108`, `wear_tear.rs:52`). **So DX29-9's own text — "a human can cast Turn, or Turn+Burn, but never Burn" — is falsified by DX29-12's gate**: at HEAD only the *left half of either* is castable. Closing DX29-12 alone restores fusing and still leaves the right half alone uncastable. | |
| **`OOS-DX27-1` + `OOS-DX27-10`** | DX27-10 is a strict **sub-case**: the double `{T}: Add {R}` exists only because Blood Moon and Magus hand-author a grant that no CR 305.6 derivation supplies, and `AddManaAbility` is append-only. A derivation is idempotent by construction and lets both defs delete the grant. Ranking DX27-10 separately buys a `push_back` guard that the real fix deletes. | §2.1 |
| **`OOS-DX26-6` + `OOS-DX27-4`** | DX26-6's remaining card yield is **not** DSL work: two of its three claimed flips (`blackblade_reforged`, and `empyrial_plate` raising the analogue) are blocked on **one engine bug** — `resolve_cda_amount` resolving the controller from the *equipped creature* rather than the effect's controller (CR 108.5/611.2c), which is exactly `OOS-DX27-4`. `crown_of_skemfar` is the second def naming it. One fix, two rows. | `layers.rs:1862-1867`, repeated `:1881-1886`, `:1896-1901` |
| **`OOS-DX8-1` + `OOS-DX8-3`** | DX8-3's "72 effectively-`Complete` defs print a you-may nothing expresses" **is** DX8-1's `may` channel, exactly — re-derived from the `BASELINE` const body as `may` 72 / `up_to` 10 / `choose` 2. Separately they build the same worklist twice. | §2.7 |
| **`OOS-ENG2-1` + `OOS-ENG2-2`** | One mechanism (`PermanentTargeted` emitted at 3 of 12 announcement sites), one population (3 deck-legal `Complete` Ward defs). ENG2-2 is ENG2-1's site census, not a second finding. | §2.5 |
| **`OOS-DX20-10` + `OOS-DX20-5`** | Both need the same new `EnchantFilter` field. `kayas_ghostform` additionally needs `controller: You`, which the same edit supplies. | prose in §2.12 |
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

### 2.9 Silent closures found — two, neither recorded anywhere

The census AC requires silently-closed seeds be dispositioned. Two were found by reading code
rather than status text:

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
`OOS-ADJ-2` is recorded **not discharged** in the adjudication's §6 rather than being left to read
as closed by the rider that cites it.

**Not re-decided, and stated so it is not mistaken for an omission**: the adjudication's
`OOS-ADJ-3` instruction — *re-word `OOS-DX19-2`'s "CR 613.8b dependency-aware fixpoint" framing
before any PB-DX42b dispatch, because CR 613.8a(a) confines dependency to a single layer and the
live case is cross-layer* — is **still outstanding** and is still a dispatch-time precondition. It
is a wording fix to a registry row, not queue work, and it is carried on PB-DX42b's row in §4.
