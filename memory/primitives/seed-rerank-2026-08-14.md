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
`S` has to be filtered once more (§1e-bis) before it is a seed count.

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
