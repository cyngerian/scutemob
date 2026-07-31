# Seed Re-rank — RS5..RS11 vs the PB-DP suite (2026-07-27, task `scutemob-159`)

<!-- last_updated: 2026-07-27 -->

> **This document is the authoritative primitive queue.** It supersedes
> `memory/primitives/rider-seed-triage-2026-07-19.md` §3/§5 (the paused PB-RS queue) as the
> thing a dispatcher reads to pick the next batch. The RS doc remains the canonical *filing
> record* for the OS/RS-wave seeds — its §1c seed rows are still the source of truth for what
> those seeds say — but **its queue is retired**; every one of RS5..RS11 is dispositioned in §3
> below and none of them may be claimed by rank any more.
>
> **Precedents / structural models**: `memory/primitives/oos-retriage-plan-2026-07-18.md`
> (`scutemob-115`) and `rider-seed-triage-2026-07-19.md` (`scutemob-142`). Same shape:
> headline → full census → chain-verification notes → ranked queue → dormant → source-doc edits.
>
> **Method (binding, per `feedback_retriage_verification` / `feedback_verify_full_chain` /
> `feedback_pb_yield_calibration`)**: a closure is believed only when the *shipped code* says so,
> never a banner; a seed premise is re-derived from source and CR (MCP) rather than copied from
> the filing doc; card scope comes from the compiled corpus, not from a seed's estimate; yields
> are discounted 2-3×.
>
> **Zero engine/card-def code changed by this task.** Docs + triage only. `cargo test --all`
> re-run on this branch: **3,928 passed / 0 failed** (4 ignored) across 30 test binaries —
> the pin holds unchanged.
>
> **Engine baseline this triage was verified against**: PROTOCOL 31 / HASH 68, coverage
> 1,139/1,804 = 63.1% (live figures from `decision_gate::decision_site_reconciliation_report`,
> re-run 2026-07-27 — see §2.9).

---

## 0. Headline

The brief asked for RS5..RS11 to be re-ranked against ~90 new seeds. The re-rank happened, but
the four things worth reading first are all *census* findings, not ranking ones:

1. **One RS-queue seed was silently closed by the DP suite and nobody noticed.** **OOS-RS3-1**
   (the five `CardDefETB` sweeps never check intervening-if at queue time, CR 603.4) was
   explicitly marked **rankable** in `rider-seed-triage` §1c and named in the §5 banner as an
   insert candidate. **PB-DP6 (`scutemob-154`) closed it** — all five sweep sites now call
   `carddef_intervening_if_holds_at_queue_time` (`turn_actions.rs:310`, `:483`, `:561`, `:781`,
   `:1945`), one of **14** gated sites. Verified at source, not from the DP6 banner. Had the RS
   queue simply resumed at its own §5 instruction, a worker would have been dispatched to
   implement a fix that shipped a week earlier. This is the `N4 re-dispatch hazard` the task
   brief warned about, live.

2. **A phantom seed ID is loose in the record.** **OOS-RS1-2** appears twice in
   `pb-review-RS1.md` (`:161`, `:251`) as a *conditional* filing — "fix `RestrictSearchTopN` …
   **or** file it as OOS-RS1-2". The fix was applied (`effects/mod.rs:3568-3577` reads
   `Zone::top_n`, with the CR 701.23/121.1 comment the review asked for), so the seed was never
   filed and has no row anywhere. Same class as OOS-OS10-1 and OOS-OS7-3, both struck by
   `scutemob-142`. **Strike it from all carry-forward lists.**

3. **The cheapest item in the entire 204-seed inventory is a card-authoring micro with zero
   engine change, and it exists only because PB-DP6 landed.** **OOS-DP6-3** says
   `garruks_uprising` and `inventors_fair` carry stale blocker notes naming the wrong DSL type.
   Chain-verified this task and it is now *more* true than when filed: `Condition::
   YouControlNOrMoreWithFilter` exists (`card_definition.rs:3834`), is used by **21** shipped
   defs (the seed said six), **and is in `condition_is_queue_time_evaluable`'s true set**
   (`effects/mod.rs:10139`) — so PB-DP6 made the *trigger-time* half both defs call
   "inexpressible" expressible. Both are authorable today. **2 flips, 0 engine lines.**

4. **The RS queue's own next pick (R5) is the weakest item in this document.** **OOS-RS-4**
   (Anim Pakal LKI counters) is a **LOW** review finding worth 0 flips on 1 already-`Complete`
   card, reachable only if Anim Pakal leaves the battlefield in response to its own attack
   trigger — *and* the obvious one-line fix is a trap: swapping `EffectAmount::CounterCount` for
   `CounterCountAtLastKnownInformation` would make it produce **zero** Gnomes always, because
   that variant reads `ctx.lki_counters`, which is only populated for leave-the-battlefield
   triggers (`effects/mod.rs:8429-8434`) and Anim Pakal's is `WheneverYouAttack`. It is demoted
   out of the queue entirely (§5).

**The ranking convention is unchanged** (inherited from both prior triages): live-wrong on a
`Complete`/deck-legal path first; then gate integrity; then cheap high-yield riders; then
agency/quality. Applied to the merged inventory, the RS queue's remaining seven items do not
hold the top of the list: **the top five slots are all DP-suite seeds or an RS seed re-scoped
upward** (OOS-OS7-2, ex-RS6, is the one RS item that gains rank — see §2.3, it is live-wrong on
**7** `Complete` defs in ordinary play, which its own filing understated as "0 flips; repairs
golgari_charm + siblings").

**Honest discounted yield across the whole successor queue: ~13-15 clean flips**, plus integrity
repairs on ~15 already-`Complete` cards and three gate-integrity fixes whose value is that they
stop a green suite from lying.

---

## 1. Full seed census

**209 distinct `OOS-*` tokens** across `memory/`, `docs/`, `crates/`, `tools/` and `CLAUDE.md`
after removing template placeholders (`OOS-seed-name`, `OOS-confirmed`, `OOS-flagged`,
`OOS-retriage`), bare umbrella/heading tokens (`OOS-LKI`, `OOS-XA`, `OOS-XA2`, `OOS-EAT`,
`OOS-EWC`, `OOS-EF5`, `OOS-EF3b`, `OOS-EF4`, `OOS-EF10`, `OOS-AC7`, `OOS-DP3/4/5/6/10`,
`OOS-M11`, `OOS-RS2`) and the `-N` naming-convention tokens. Minus the two known alias pairs
(`OOS-XA-3`≡`OOS-XA2-3`, `OOS-AC9-MULTINAME`≡`OOS-AC9-SEARCHNAME`) → **207 distinct seeds**;
minus **3 phantoms** (never filed: `OOS-OS10-1`, `OOS-OS7-3`, and **`OOS-RS1-2`, newly found by
this task**) → **204 real filed seeds**, plus the two unnumbered items `EF-EF1-A` (shipped as
PB-OS2) and *hidden_strings optionality* (dormant).

Per inventory:

| inventory | rows | note |
|---|---|---|
| `docs/audits/decision-point-audit.md` §8.1 (PB-DP suite) | **109** | machine-counted from the table: DP1 4 / DP2 8 / DP3 9 / DP4 13 / DP5 9 / DP6 10 / DP7 12 / DP8 14 / DP9 19 / DP10 11. No duplicate IDs. |
| `memory/primitives/rider-seed-triage-2026-07-19.md` §1a-§1c (OS/RS waves) | **28** | 11 OS-series (2 phantom) + 17 RS-series (1 phantom) |
| `memory/m11-session-plan.md` + audit §7 (M11) | **4** | OOS-M11-1..4 |
| `memory/primitives/oos-retriage-plan-2026-07-18.md` §1a-§1d (legacy) | **65** | 70 enumerated rows, deduped by the two alias pairs |

### 1a. CLOSED — verified against shipped code, not banners (AC 5855)

Every closure the task brief listed as "already closed, do not re-rank" was re-verified. **All
six hold.** One further closure was found that no document records.

| seed | claimed closer | verification performed this task | verdict |
|---|---|---|---|
| **OOS-M11-1** | PB-DP2 (`scutemob-150`) | `rules/commander.rs:844` — `library.shuffle(&mut rng)` runs *before* `:846`'s `LibraryShuffled`, with the doc block at `:794-798` naming `timestamp_counter` as the seed; `:914` — `handle_keep_hand` calls `state.move_object_to_bottom_of_zone(*obj_id, lib_zone_id)`. Both halves (shuffle + bottoming) present. | **CLOSED** |
| **OOS-M11-4** | PB-DP8 (`scutemob-156`) | `Command::ChooseTriggerTargets` dispatched at `rules/engine.rs:606` and admitted through the blocking gate at `:297`; `GameState.pending_trigger_targets` at `state/mod.rs:159` with the read accessor at `:519`. | **CLOSED** |
| **OOS-DP1-1** | PB-DP4 (`scutemob-152`) | `rules/engine.rs:1095`, `:1337`, `:1488` — all three sites now carry a *comment* recording the deleted write; `grep 'priority_holder = Some(active_player)'` over `engine.rs` returns **only those three comments**, no live assignment. Closed by deletion, as the row claims. | **CLOSED** |
| **OOS-RS3-4** | PB-DP4 (`scutemob-152`) | `rules/combat.rs:797` defines `has_uncosted_attack_target`; it is called from **both** must-attack tests — the goad block (`:389`) and the `MustAttackEachCombat` block (`:478`). The seed's own "check whether goad has the identical hole" rider is correctly carried forward as OOS-DP4-3 (the *directional* half, still open — see §1c). | **CLOSED** |
| **OOS-RS-3** | PB-RS4 (`scutemob-146`) | `rules/replacement.rs:1545-1560` (`apply_self_etb_from_definition`) and `:2335-2339` (non-self permanent replacements) both read `def.effective_abilities(entering_is_transformed)`; `rules/face.rs::deregister_face_statics` is now a generic loop over `remove_one_registration`, not a `Static`-only match; `crates/engine/tests/core/face_dereg_parity.rs` (185 lines) is the parity gate. All three §2.4 deviations gone. | **CLOSED** (and with it OOS-OS4-2) |
| **OOS-DP7-7** | PB-DP10 (`scutemob-158`) | `crates/engine/tests/core/decision_gate.rs:925` — `decision_site_reconciliation_report` exists and was executed this task; it prints the all-rows union **267** and the still-auto union **97** (full output in §2.9). The 277-def re-derivation is real and machine-computed. | **CLOSED** |
| **OOS-RS3-1** | *(nothing — nobody claimed this)* | **NEW CLOSURE, found by this task.** All five `CardDefETB` sweep sites gate at queue time: `turn_actions.rs:310` (upkeep), `:483`, `:561`, `:781`, `:1945` — each `if !super::abilities::carddef_intervening_if_holds_at_queue_time(...) { return None; }` immediately before the matching `PendingTrigger::blank(..., CardDefETB)` push. Fourteen gated sites workspace-wide. Audit §4.8's DP-15 row records the closure; **the RS doc does not**, and its §5 banner still advertises the seed as an insert candidate. | **CLOSED by PB-DP6 (`scutemob-154`)** |

Also closed within the DP inventory itself (recorded here so the census is complete, no
re-verification claimed beyond reading the rows): **OOS-DP8-9** and **OOS-DP8-10** (closed in
the PB-DP8 fix cycle). **Partial**: OOS-DP1-2 (3 of 5 entry guards), OOS-DP8-4 (narrowed twice,
default-quality residual), OOS-DP8-13 (mostly closed, two residuals), OOS-DP9-5 (axis 2 half
fixed), OOS-DP9-10 (5 sites fixed, no gate), OOS-DP9-19 (4 sites routed, (b) and (d) open).

### 1b. STALE PREMISES — the seed is open but says something no longer true

Filed here because a stale premise mis-ranks a seed in both directions.

| seed | what the seed says | what is true at PROTOCOL 31 / HASH 68 |
|---|---|---|
| **OOS-DP8-6** | "`GameEvent::private_to()` does not exist. `rg 'private_to' crates/` returns zero." | **It exists** — `rules/events.rs:1503`, added by PB-DP9 as the declaration half. The residual is real and unchanged (nothing *enforces* it; it is a declaration, not an enforcement point) but the seed's headline is now false. Re-scope to "private_to has no consumer", M10-gated. |
| **OOS-DP6-3** | "…`Condition::YouControlNOrMoreWithFilter` … already used by six shipped defs. Both cards are plausibly authorable today." | **21** shipped defs use it, and it is queue-time evaluable (`effects/mod.rs:10139`), so PB-DP6 also unblocked the trigger-time half both defs' notes call inexpressible. Not "plausibly" — verified authorable. See §2.4. |
| **OOS-DP9-3** | "~7 `partial` defs say so in their own source" | 6 are `partial`, **`protean_hulk` is `inert`**; and 5 of the 7 carry a *second, independent* blocker so a count-only fix flips **2**, not 7. See §2.5. |
| **OOS-DP7-11** | "5 structs are currently outside it … a further 9 path-qualified impls are enums" | 5 structs is **exact** (`MergedComponent`, `FlashGrant`, `PlayFromTopPermission`, `PlayFromGraveyardPermission`, `SacrificedCreatureLki`); the enum count is now **10**, not 9. Gate hole verified live at `tests/core/hash_schema.rs:1540-1541`. |
| **OOS-DP2-7** | "reachable in ordinary play (unlike the mulligan)" — repeated verbatim in this task's brief | The two *engine sites* are on an ordinary-play path, but the only def that can reach them, `darksteel_colossus.rs`, is `Completeness::known_wrong` and therefore rejected by `validate_deck`. **Not live-wrong on a `Complete` path** — ranks below the items that are. See §2.6. |
| **OOS-DP10-3** / **OOS-AC9-FILTERMANA** | filed independently, ~9 months apart | **Same gap, two IDs.** `Effect::AddManaFilterChoice` (`effects/mod.rs:2823-2845`) adds *one of each* colour, so a filter land makes two mana instead of one chosen one. Treat `OOS-AC9-FILTERMANA` as an alias of `OOS-DP10-3`; the legacy dormant row should point here. |
| **OOS-RS-4** | rider-triage ranks it **R5**, "correctness, repairs 1 `Complete` card" | Accurate as a *class*, but it is `pb-review-OS11.md`'s Finding **2, severity LOW**, and the natural fix is unavailable (§0 point 4 / §2.7). Demoted. |

### 1c. ACTIVE candidates — ranked into §4

**Correctness, live-wrong on a `Complete`/deck-legal path (5)**: `OOS-DP6-1`, `OOS-DP5-7`,
`OOS-OS7-2`, `OOS-DP10-8`, `OOS-RS2-1`.

**Gate integrity (5)**: `OOS-DP7-11`, `OOS-DP9-13`, `OOS-DP10-9`, `OOS-DP10-1`,
`OOS-DP9-10` (residual).

**Cheap card yield (7)**: `OOS-DP6-3`, `OOS-DP9-3`, `OOS-DP5-6`, `OOS-OS6-1`, `OOS-OS7-1 R1`
+ `OOS-RS-5`, `OOS-OS4-1` (+`OOS-RS4-3`), `OOS-OS4-3`.

**Agency / CR completion (6)**: `OOS-DP3-4` + `OOS-DP8-7` (PB-DP8b), `OOS-DP8-3`,
`OOS-DP7-1`, `OOS-DP10-2`, `OOS-DP10-10`, `OOS-OS7-1 R2+R3`.

**Correctness sweeps / narrow (12, ride a batch)**: `OOS-DP1-2`, `OOS-DP2-1`, `OOS-DP2-7`,
`OOS-DP2-8`, `OOS-DP3-1`+`OOS-DP3-6`, `OOS-DP4-3`, `OOS-DP4-10`, `OOS-DP6-2`, `OOS-DP6-5`,
`OOS-DP6-6`+`OOS-DP6-10`, `OOS-DP6-9`, `OOS-DP7-9`, `OOS-DP7-10`, `OOS-DP8-11`, `OOS-DP8-13`,
`OOS-DP9-8`, `OOS-DP9-9`, `OOS-DP9-11`, `OOS-DP9-14`, `OOS-DP9-16`, `OOS-DP9-19(b)(d)`,
`OOS-DP10-5`, `OOS-DP4-1`.

**Gate/bookkeeping mechanization (3)**: `OOS-DP10-4`, `OOS-DP10-7`, `OOS-DP10-11`.

### 1d. NOT the primitive queue — owned elsewhere

The primitive queue is for **engine correctness**. These are real and open, and they belong to
another track. Recorded so nobody re-ranks them into a PB.

**Simulator / M11-local (13)** — `crates/simulator` or `tools/`, cannot produce a wrong game
state: `OOS-M11-2` (see §2.8 — audit §7's exclusion **confirmed**), `OOS-DP2-5`, `OOS-DP4-8`,
`OOS-DP4-11`, `OOS-DP5-1`, `OOS-DP5-2`, `OOS-DP7-5`, `OOS-DP7-6`, `OOS-DP7-12`, `OOS-DP8-1`,
`OOS-DP8-2`, `OOS-DP9-1`, `OOS-DP9-7`, `OOS-DP9-17` (engine-side note only), `OOS-DP8-12`
(replay-viewer/TUI display).

**M10 / hidden-information-gated (5)**: `OOS-DP2-3`, `OOS-DP7-3`, `OOS-DP8-6`, `OOS-DP9-6`,
plus Architecture-Invariant-7 wording.

**Tooling / fuzzer (2)**: `OOS-DP3-9` + `OOS-M11-3` — the same 150-200-turn regime, plausibly
one root cause; a tooling batch, not a primitive.

**Cosmetic / stale-cite doc pass (8)**: `OOS-DP1-3`, `OOS-DP4-6`, `OOS-DP6-8`, `OOS-DP2-4`
(refactor + the `StdRng`-stability addendum, which should ride §4 PB-DX18), `OOS-DP4-7`,
`OOS-DP3-8`, `OOS-DP7-8`, `OOS-DP9-15`.

**Documented deviations / bookkeeping-answered (10)**: `OOS-DP1-4`, `OOS-DP2-6`, `OOS-DP3-3`,
`OOS-DP4-5`, `OOS-DP4-12`, `OOS-DP5-3`, `OOS-DP5-5` (**but see the note below**), `OOS-DP6-4`,
`OOS-DP6-7`, `OOS-DP7-4`, `OOS-DP8-8`, `OOS-DP8-14`, `OOS-DP9-12`.

> **OOS-DP5-5 is cheaper than it says it is.** Its filing reads "the alternative — suspending
> mid-resolution — needs a suspendable effect resolver, i.e. exactly the pending-decision
> machinery §8's sequencing note calls for in PB-DP7..DP9." **PB-DP9 built it** (the
> abort-and-replay: clone at entry, restore wholesale, bank the answer, re-run the resolution).
> A deferred draw could now stop the whole resolution the same way a search does. Re-scope the
> seed before ranking it — it is no longer blocked on machinery that does not exist.

**Deferred-on-wire, own PB when someone wants the shape (6)**: `OOS-DP2-2`, `OOS-DP3-2`,
`OOS-DP4-2`, `OOS-DP5-4`, `OOS-DP8-5` + `OOS-DP9-18` (the CR 800.4a object sweep — these two
should ship together and are a batch of their own, not a rider).

### 1e. Carried forward unchanged from the 2026-07-18 legacy inventory

**31 seeds stay parked**, re-checked only for whether a later wave closed them (none did):
7 deferred (`OOS-EF5-1` Battle subsystem, `OOS-EF5-2` Super Nova, `OOS-XS-4` protection-from-
colour, `OOS-AC7-1/2/3`, `OOS-AC6-2`) and 24 dormant-0-yield (`OOS-EAT-1/2/3`,
`OOS-LKI-Power-1/4/5`, `OOS-LKI-4`, `OOS-XS-E-1/2`, `OOS-AC8-2`, `OOS-AC9-SEARCHNAME`,
`OOS-AC9-FILTERMANA` → **now aliased to OOS-DP10-3**, `OOS-AC9-ELSPETH`, `OOS-AC9-AMASSCHOICE`,
`OOS-EF3b-2`, `OOS-XA2-1/2/4/5`, `OOS-EWCD-1/2/3`, `OOS-AC7-4`, `OOS-TFS`). The other 34 of
that inventory's 65 are closed (23 verified stale in `scutemob-115`; the 16 candidates all
shipped as PB-OS1..OS11, `scutemob-116..141`).

From the OS/RS wave, also parked: **`OOS-RS-6`** (crucible dynamic-X, 1 card, 3 coupled gaps),
**hidden_strings optionality** (needs an interactive channel — note that PB-DP7/DP8/DP9 have now
*built* one, so this is worth a re-scope pass before it is next dismissed), **`OOS-RS3-2`**
(8 textually-admitted gaps on effectively-`Complete` defs — a re-marking sweep; its one
live-wrong member `emeria_the_sky_ruin` should be pulled forward on its own),
**`OOS-RS3-3`** (LOW, `TargetCreatureYouControl` announce-legality only), **`OOS-RS4-1`**,
**`OOS-RS4-2`**, **`OOS-RS4-4`** (all latent, 0 reachable yield today), **`OOS-RS1-1`**
(`ZoneTarget::Library { position }` inert — verified still true at `effects/mod.rs:8594`, which
destructures `ZoneTarget::Library { owner, .. }`; this is what keeps **OOS-OS8-2**/muxus gated),
**`OOS-OS8-2`** (card-gated behind OOS-RS1-1).

### 1f. Census totals

Two figures are computed and exact; the rest are stated per inventory rather than rolled into
one grand total, because the four inventories were counted by different methods at different
times and a single reconciled number would be false precision. This suite has published a
plausible-sounding roster and been wrong three times (PB-DP6's 3-vs-14, PB-DP8's 84-vs-77,
PB-DP9's 74/16/8-vs-69/16/7); the honest move is to say which numbers are machine-derived.

**Machine-derived (reproduce with the grep in §1's preamble + the `awk` row count over audit
§8.1):**

| figure | value | method |
|---|---|---|
| Distinct `OOS-*` tokens across the repo | **209** | `grep -rhoE 'OOS-[A-Za-z0-9-]+' memory/ docs/ crates/ tools/ CLAUDE.md \| sort -u`, minus placeholders / umbrella tokens / `-N` naming tokens |
| — the two known alias pairs | −2 | `OOS-XA-3`≡`OOS-XA2-3`, `OOS-AC9-MULTINAME`≡`OOS-AC9-SEARCHNAME` |
| **Distinct seeds** | **207** | |
| — phantoms, never filed | −3 | `OOS-OS10-1`, `OOS-OS7-3` (both struck by `scutemob-142`), **`OOS-RS1-2` (struck by this task)** |
| **Real filed seeds** | **204** | |
| Audit §8.1 rows | **109** | `awk` over the table, no duplicate IDs: DP1 4 / DP2 8 / DP3 9 / DP4 13 / DP5 9 / DP6 10 / DP7 12 / DP8 14 / DP9 19 / DP10 11 |

**Verdict buckets (per inventory, as classified above):**

| bucket | DP §8.1 | OS/RS | M11 | legacy | note |
|---|---|---|---|---|---|
| **CLOSED, verified this task** | OOS-DP1-1, OOS-DP7-7 | OOS-RS-3, OOS-RS3-4, **OOS-RS3-1 (new)** | OOS-M11-1, OOS-M11-4 | — | §1a; all six brief-listed claims hold, plus one nobody had recorded |
| **CLOSED in batch** (row-recorded, not re-verified) | OOS-DP8-9, OOS-DP8-10 | OOS-OS4-2, OOS-OS8-1, OOS-OS9-1, OOS-RS-1, OOS-RS-2 | — | 23 resolved/stale + 16 shipped as PB-OS1..OS11 | figures as stated by the owning doc |
| **PARTIAL** | OOS-DP1-2, DP8-4, DP8-13, DP9-5, DP9-10, DP9-19 | OOS-RS4-3 (folded into OOS-OS4-1) | — | — | open residual, ranked or parked individually |
| **STALE PREMISE, open** | 5 | 1 | — | 1 | §1b — 7 total |
| **ACTIVE, ranked in §4** | 31 | 6 | — | — | §1c |
| **NOT the primitive queue** | ~40 | — | OOS-M11-2, OOS-M11-3 | — | §1d — simulator/M11 15, M10 5, tooling 2, cosmetic 8, documented-deviation 13, wire-deferred 6 |
| **PARKED** | ~18 latent | 8 | — | 31 (7 deferred + 24 dormant) | §1e / §5 |

*(Buckets overlap where one seed rides another — e.g. a latent DP seed listed as a rider in §4
also appears under PARKED. The queue in §4 is the non-overlapping, dispatchable view.)*

---

## 2. Chain-verification notes

Each of these was walked filter → effect → layer → cost (or the equivalent chain for the
subsystem) against source at PROTOCOL 31 / HASH 68, and against CR via `mcp__mtg-rules`.

### 2.1 OOS-DP6-1 — the dropped intervening-if **(the highest-severity open seed in the inventory)**

`aurelia_the_warleader.rs:32-33` declares `trigger_condition: TriggerCondition::WhenAttacks`
with `intervening_if: Some(Condition::IsFirstCombatPhase)`, and the file carries **no
`completeness` field**, so it is `Complete` by `#[default]` and `validate_deck` admits it.
`testing::replay_harness::build_face_ability_vectors` (`:2382`) lowers that ability into a
runtime `TriggeredAbilityDef` and hardcodes `intervening_if: None` — **34** such literals in
that file. The lowering is **not test-only**: it is called from `rules/face.rs:104` and
`rules/resolution.rs:864`, both live permanent-creation paths. CR 603.4 (verified via MCP)
requires the check at *both* trigger time and resolution; here it happens at **neither**,
because both ends read the runtime field, which is `None`.

PB-DP6 confirmed by execution at close-out that this makes Aurelia an unbounded extra-combat
engine on a deck-legal card. **This is the only open seed that is simultaneously (a) live-wrong,
(b) on a `Complete` def, (c) unbounded rather than merely incorrect.** Under the convention both
prior triages used, it takes rank 1.

Cost is real: the clean fix is an `Option<Condition>` on `TriggeredAbilityDef`, which lives
inside `Characteristics` and is hashed (`state/hash.rs:3337`) ⇒ **HASH bump**. The seed's
alternative (c) — index-correspondence between card-def and runtime ability lists — is
explicitly fragile and must be mirrored at resolution and re-derived on every face change; the
adjacent `OOS-DP6-2` shows what happens when an index space is assumed rather than proven.

### 2.2 OOS-DP5-7 — `Command::ChooseDredge` has no pending-state gate **(live today, needs no card)**

Verified end to end. `rules/engine.rs:534-544` validates only `validate_player_exists`, resets
loop detection, and calls `replacement::handle_choose_dredge`. That handler
(`rules/replacement.rs:2925`) has a `None` arm that goes straight to
`draw_card_skipping_dredge(state, player)` — **unconditionally, with no check that a draw is
pending** — and a `Some(card)` arm that validates the *card* (in graveyard, has `Dredge(n)`,
library ≥ n) and never the pending state either. So any player, at any time, can send
`ChooseDredge { card: None }` and take a free card.

The `None` arm is the sharper half: it requires no dredge card, no graveyard contents, and no
game state at all beyond the player existing. It is the only item in this document that is
wrong in a game you could sit down and play right now, with no adversary needed beyond a
client that sends the command.

Adjacent and the same subject: **OOS-DP7-2** — `rules/events.rs` and `rules/replacement.rs`
both carry doc comments asserting the engine *pauses* for a `ChooseDredge`. It does not. A doc
comment is currently the only thing asserting the property (`memory/conventions.md`'s
aspirationally-wrong-comment rule). Fix them together.

### 2.3 OOS-OS7-2 (ex-RS6) — CR 611.2c is unimplemented, and it is worse than its filing says

CR **611.2c** (verified via MCP): *"If a continuous effect generated by the resolution of a
spell or ability modifies the characteristics or changes the controller of any objects, the set
of objects it affects is determined when that continuous effect begins. After that point, the
set won't change."*

`struct ContinuousEffect` (`card-types/src/state/continuous_effect.rs:531-561`) has `filter:
EffectFilter` and **no affected-object set**. `rules/layers.rs:613` evaluates
`EffectFilter::AllCreatures` live against the current board on every characteristics
calculation. There is no snapshot anywhere.

**The filing understates this as "0 flips; repairs `golgari_charm` + siblings".** Corpus sweep
from the defs: **9** defs pair `Effect::ApplyContinuousEffect` with an `EffectFilter::All*`
mass filter, and **7 of them are `Complete`** — `bladewing_the_risen`, `goblin_lookout`,
`crippling_fear`, `eyeblight_massacre`, `golgari_charm`, `olivias_wrath`,
`the_meathook_massacre` (the other two, `elvish_dreadlord` and `final_showdown`, are `partial`).
Reachability is trivial and requires no unusual play: resolve Golgari Charm's "all creatures get
-1/-1 until end of turn", then play a creature that same turn — the engine gives the newcomer
-1/-1, and real Magic does not. That is live-wrong on a `Complete` path in an ordinary game,
which is precisely the criterion that put OOS-RS-1 at the head of the RS queue.

Cost is the reason it is rank 5 and not rank 1: the fix adds hashed state to `ContinuousEffect`
(**HASH**, and **PROTOCOL** if the type is inside the wire closure — the implementer must
gate-compute both rather than predict), and it changes behaviour for every resolution-generated
mass effect in the corpus at once.

### 2.4 OOS-DP6-3 — two stale blocker notes, and PB-DP6 already removed the blocker

`garruks_uprising.rs:67-74` and `inventors_fair.rs:73-80` both say they are blocked because
`InterveningIf` offers only `ControllerLifeAtLeast` / `SourceHadNoCounterOfType`. That names the
**runtime** 2-variant enum; the def-level field is `Option<Condition>`, and
`Condition::YouControlNOrMoreWithFilter { count, filter }` exists at
`card_definition.rs:3834` and is used by **21** shipped defs. Garruk's Uprising needs
"if you control a creature with power 4 or greater"; Inventors' Fair needs "if you control three
or more artifacts" — both are exactly that variant.

The note's own hedge ("`Effect::Conditional + Condition::YouControlNOrMoreWithFilter` would fix
the resolution-time half; the trigger-time half remains blocked") **is now false**:
`condition_is_queue_time_evaluable` lists `Condition::YouControlNOrMoreWithFilter` in its
true-set (`effects/mod.rs:10139`), and PB-DP6 wired the card-def `intervening_if` into all 14
queue sites. Both halves of CR 603.4 are available today.

Inventors' Fair additionally records that its search ability "currently permits illegal
activation" and that `activation_condition: Some(Condition::YouControlNOrMoreWithFilter)` is the
fix — i.e. a second, already-expressible correction on the same file.

**2 flips, 0 engine lines, no wire.** Nothing in the inventory is cheaper.

### 2.5 OOS-DP9-3 — the seed's yield is 7, the corpus says 2

CR **701.23d** (verified via MCP) is the right cite, not the bare `701.23`: *"If a player is
searching a hidden zone simply for a quantity of cards, such as 'a card' or 'three cards,' that
player must find that many cards."* CR 701.23h adds the multi-search-is-one-search rule
(`OOS-DP9-9`'s subject). Per-def audit of the seven named defs:

| def | marker | second blocker beyond the count |
|---|---|---|
| `tooth_and_nail` | `partial` | **none** — "'up to two' search — SearchLibrary finds one card" is the whole note |
| `buried_alive` | `partial` | **none** — "'up to three' … using one creature as approximation" |
| `myriad_landscape` | `partial` | needs a "**share a land type**" cross-card filter constraint |
| `goblin_recruiter` | `partial` | needs "put them on top **in any order**" — that is `OOS-DP9-2`'s DSL gap |
| `sarkhan_unbroken` | `partial` | needs "any number" *and* an interactive add-one-mana-of-any-colour |
| `tiamat` | `partial` | needs "each have **different names**" uniqueness *and* a name-**exclusion** filter |
| `protean_hulk` | **`inert`**, not `partial` | needs a **total-mana-value budget** across the found set |

**Discounted yield: 2** (`tooth_and_nail`, `buried_alive`) for a count-only fix; a further 1-2
if the same batch also adds unbounded ("any number") counts. The seed's "largest card-yield item
adjacent to this batch" claim does not survive the corpus check — but its *machinery* claim does
(one `count: EffectAmount` on the effect, `found: Vec<ObjectId>` on the answer, zero new
plumbing), so it stays a cheap batch, just not a high-yield one.

### 2.6 OOS-DP2-7 — real, but gated

`ReplacementModification::ShuffleIntoOwnerLibrary` is handled at
`rules/replacement.rs:1084-1092` and `:1181-1200` (the seed's `:854`/`:965` cites are stale by
~220 lines — the OOS-DP6-8 line-drift class again). Both sites push
`GameEvent::LibraryShuffled { player: owner }` and **neither calls `Zone::shuffle`**; the
comment at the first site says "Redirect to library AND shuffle the library" and only the
redirect happens. Architecture Invariant 4 violation, verbatim as filed.

**But**: `darksteel_colossus.rs` is the only def in the corpus using the variant, and it is
`Completeness::known_wrong` ⇒ `validate_deck` rejects it. The brief's "reachable in normal play"
is true of the *engine site* and false of the *game*: no legal deck can reach it. Ranked as a
micro-batch with the other shuffle-integrity items (§4 PB-DX18), not as a Tier-1 correctness
item.

### 2.7 OOS-RS-4 (ex-RS5) — verified, and the obvious fix is a trap

`anim_pakal_thousandth_moon.rs:84-87` uses `EffectAmount::CounterCount { target:
EffectTarget::Source, counter: PlusOnePlusOne }`, which `effects/mod.rs:8288-8300` resolves by
reading `state.objects.get(&id).counters` — a **live** read, so if Anim Pakal leaves the
battlefield in response to its own attack trigger the source is a dead `ObjectId` (CR 400.7) and
the count is 0 instead of the last known value.

The LKI-aware sibling exists — `EffectAmount::CounterCountAtLastKnownInformation { counter }` —
but it reads `ctx.lki_counters` (`effects/mod.rs:8429-8434`), which is populated **only at
leave-the-battlefield trigger fire time**, and Anim Pakal's trigger is `WheneverYouAttack`. A
straight swap would return 0 every time and make the card strictly worse. A real fix needs LKI
capture for a non-LBA trigger whose source dies mid-resolution (CR 608.2h / 113.7a).

Severity as filed by its own review: **LOW**. Yield: 0 flips, 1 already-`Complete` card, one
narrow interaction. **Demoted out of the queue** (§5).

### 2.8 OOS-M11-2 — audit §7's exclusion CONFIRMED (brief item 3)

Re-verified both halves. `crates/simulator/src/mana_solver.rs` contains **zero** occurrences of
`mana_pool`, and reads `obj.characteristics.mana_abilities` directly at `:35` rather than
`calculate_characteristics`. `memory/m11-session-plan.md:914` (risk R3) already assigns the pool
half to **Session 3 item 7** and files the layer half as this seed.

**Verdict: do NOT enter it into the primitive queue.** It is `crates/simulator`-only; the
engine's own payment paths are layer-correct and authoritative, and `solve_mana_payment` returns
a `Vec<Command>` that `process_command` then judges — it cannot produce a wrong game state, only
a wrong suggestion. PB-DP4's rider (the attack-tax debit makes pool-blindness cost real mana) is
noted and does not change the ownership. **M11-local Session 3 owns it.**

### 2.9 OOS-DP10-6 — the measured ranking input, re-read as instructed

`decision_gate::decision_site_reconciliation_report` executed on this branch, 2026-07-27
(`cargo test -p mtg-engine --test core decision_site_reconciliation_report -- --nocapture`):

```
triggered_targets   77 Complete (+21)  SERVED by PB-DP8
search_library      73 Complete (+25)  SERVED by PB-DP9; residual: OOS-DP9-9, OOS-DP9-3
proliferate         25 Complete (+4)   AUTO-CHOSEN  (CR 701.34a)
scry                16 Complete (+3)   SERVED by PB-DP9
discard_cards       13 Complete (+4)   AUTO-CHOSEN  (CR 701.9b)
sacrifice_permanents 11 Complete (+7)  AUTO-CHOSEN  (CR 701.21a)
may_pay_then_effect 10 Complete (+2)   AUTO-CHOSEN  (CR 118.12)
choose_color_or_type 10 Complete (+6)  AUTO-CHOSEN
look_at_top_or_route 10 Complete (+4)  AUTO-CHOSEN  (UPPER BOUND — see caveat)
wheel_hand          10 Complete (+0)   NO-DECISION  (CR 404.3 choice is OOS-DP10-10)
surveil              8 Complete (+1)   SERVED by PB-DP9
counter_unless_pays  7 Complete (+0)   AUTO-CHOSEN
modal_trigger        4 Complete (+3)   AUTO-CHOSEN  (CR 700.2b — PB-DP8b's roster)
change_targets       3 Complete (+1)   AUTO-CHOSEN  (CR 115.7d)
bolster_amass        3 Complete (+2)   AUTO-CHOSEN  (CR 701.39a/701.47a)
connive / discover / put_on_library   1 Complete each
may_pay_or_else / choose_stub         0 Complete  (GATED)
add_mana_filter_choice 0 Complete (+7) AUTO-CHOSEN (nothing held this until PB-DP10's T7)
the_ring_tempts_you  0 Complete (+1)   AUTO-CHOSEN (ditto)
ALL-ROWS UNION 267   STILL-AUTO UNION 97   live denominator 1139/1804 = 63.1%
```

**How this document uses it.** The row counts are a *decision-surface* ranking, not a card-yield
ranking, and they are `>=`-floor-checked rather than pinned. Three consequences for §4:

- **`proliferate` 25 is the largest single auto-chosen class**, but PB-DP9's `AnswerEffectChoice`
  channel generalises to it "for the cost of one `EffectChoiceQuestion`/`Answer` variant pair and
  zero new plumbing". It is *not* ranked into §4's top tier because it is an **agency**
  restoration on cards that are otherwise correct, not a live-wrong path — but it is the single
  best candidate for the batch *after* this queue's correctness block, and whoever picks it up
  should read the PB-DP9 §4.9 note first. Recorded as the queue's designated successor.
- **`modal_trigger` 4** is PB-DP8b's real roster, correcting OOS-DP8-7's "~5".
- **`look_at_top_or_route` 10 is explicitly an upper bound** (the report says so itself);
  anybody ranking that row must re-derive the genuine-choice subset first. Not ranked here.

### 2.10 Gate integrity — three holes in one instrument

Verified together because they are the same failure mode at three depths, and PB-DP10's own
lesson (*"a gate cited as covering something is a claim like any other"*) is the reason they
rank above card yield:

- **OOS-DP7-11** — `tests/core/hash_schema.rs:1540-1541`: `let Some(body) = bodies.get(ty) else
  { continue; }`. `hashinto_impl_bodies()` keys on the exact type token, so any impl written
  `impl HashInto for crate::state::stubs::Foo` is looked up as `Foo`, missed, and silently
  skipped. **5 structs are outside the gate right now** — `MergedComponent`, `FlashGrant`,
  `PlayFromTopPermission`, `PlayFromGraveyardPermission`, `SacrificedCreatureLki` (verified by
  classifying all 15 path-qualified impls; the other 10 are enums, which the gate cannot see
  either — see the next row).
- **OOS-DP9-13** — the same gate iterates `named_field_structs()`, so a hashed **enum variant**
  can silently drop a field; PB-DP9 demonstrated it by deleting a feed and watching the suite
  stay green.
- **OOS-DP10-1** — `pb_dp9_effect_choice.rs::roster`'s serde walk matches object keys only and
  is blind to a unit variant; documented and cross-checked by value against the canonical walk,
  but the copy remains.

One test-only batch closes all three, plus **OOS-DP9-10**'s residual (no gate for
unordered-iteration-to-outcome). Normalising the scanner key on the bare name is the fix for the
first; extending the scan to enum variants is the fix for the second; both are the same file.

---

## 3. RS5..RS11 dispositions (AC 5856)

Every item of the paused queue, with a verdict. **None of RS5..RS11 may be claimed by its old
rank.** `rider-seed-triage-2026-07-19.md` §5's banner is updated to say so.

| old rank | seed(s) | premise re-verified? | disposition |
|---|---|---|---|
| **R5** | **OOS-RS-4** — Anim Pakal live-counter-vs-LKI | **Yes, and it is weaker than filed** (§2.7): `pb-review-OS11.md` Finding 2, severity **LOW**; 0 flips; the natural one-line fix returns 0 Gnomes because `CounterCountAtLastKnownInformation` is LBA-only | **RETIRED from the queue → PARKED (§5).** Re-rank only if a second card needs non-LBA LKI capture, or alongside a CR 608.2h/113.7a LKI batch. |
| **R6** | **OOS-OS7-2** — CR 611.2c affected-set snapshot | **Yes, and it is stronger than filed** (§2.3): not "0 flips", but **7 `Complete` defs live-wrong in ordinary play** | **RE-RANKED UP → PB-DX5** (highest-ranked ex-RS item) |
| **R7** | **OOS-OS7-1 R1** + **OOS-RS-5** — target-scoped filters | Yes — `EffectFilter::CreaturesControlledByDefendingPlayer` shipped in PB-OS7; `CreaturesControlledByTargetPlayer` still absent; `kogla_the_titan_ape` is `known_wrong` with the note naming the gap | **RE-RANKED → PB-DX13** (3 flips, capability, PROTOCOL+HASH) |
| **R8** | **OOS-OS6-1** — multi-count sacrifice cost | Yes — `Cost::Sacrifice(TargetFilter)` (`card_definition.rs:1257`) has no count field; no sibling variant exists | **RE-RANKED → PB-DX12** (4 flips discounted to 3, capability, PROTOCOL+HASH). Its DFC oracle-sourcing hazard still applies — re-source Westvale/Ormendahl from `cards.sqlite`, `lookup_card` does not flatten `card_faces`. |
| **R9** | **OOS-OS4-3** — edgar return-transformed | Yes — `Effect::ExileSourceAndReturnTransformed` exists (`card_definition.rs:2164`); the *from-graveyard* sibling `ReturnSourceToBattlefieldTransformed` does not | **RE-RANKED DOWN → PB-DX16** (1 flip, own wire bump; the wire numbers in the seed are stale — live is PROTOCOL 31 / HASH 68, not "19→20 / 56→57"). Same DFC oracle-sourcing hazard. |
| **R10** | **OOS-OS4-1** (+ **OOS-RS4-3**) — back-face starting loyalty | Yes — `struct CardFace` (`card_definition.rs:30-44`) still has no `starting_loyalty`; only `CardDefinition` does | **RE-RANKED → PB-DX14** (2 flips: `nicol_bolas_the_ravager`, `grist_voracious_larva`) |
| **R11** | **OOS-OS7-1 R2+R3** — attacked-player trigger family | Yes — `WheneverYouAttack` fires once per combat (`abilities.rs:4314`) with no per-defending-player fan-out; **and there is no `karazikar` def in the corpus at all**, so the "1 flip" is a *new authoring*, not a marker flip | **RE-RANKED DOWN → PB-DX17** (last in the queue) |
| *(insert candidate named in §5's banner)* | **OOS-RS3-1** — CardDefETB queue-time intervening-if | Yes — **already fixed** | **SUPERSEDED / CLOSED by PB-DP6** (§1a). Do not dispatch. |
| *(rider named in §5's banner)* | **OOS-RS2-1** — `TurnFaceUp` unflattened cost | **Yes, still live** — `rules/engine.rs::handle_turn_face_up` calls `can_spend`/`spend` on the raw cost (~`:2137`); `ManaPool::can_spend` (`player.rs`) reads only the six colours + generic and guards residue with `debug_assert_flattened`, i.e. **debug-only**. `kitchen_finks` is `Complete` with two `{G/W}` pips, so flipping it face-up for its mana cost charges `{1}` and both pips are free in release | **RE-RANKED UP → PB-DX6**, bundled with **OOS-DP4-1** (the attack-tax pip site, same class) |

Also carried out of the RS doc: **OOS-OS8-2** (muxus) stays card-gated behind **OOS-RS1-1**
(verified still inert, §1e) and is not queued; **OOS-RS-6** (crucible) and *hidden_strings* stay
dormant, with the note that PB-DP7..DP9 have since built the interactive channel hidden_strings
was waiting on, so its dismissal deserves a fresh look rather than another copy-forward.

---

## 4. The successor queue — **PB-DX1 .. PB-DX18** (AC 5857)

**Prefix**: `PB-DX` ("decision-suite eXtension"). Verified unclaimed — zero occurrences of
`PB-DX` anywhere in `memory/`, `docs/` or `CLAUDE.md` before this document. `PB-SR*`, `PB-RS*`,
`PB-OS*`, `PB-EF*`, `PB-AC*`, `PB-DP*` are all taken and `PB-Q2/Q3/Q5` are reserved.

**Ordering rule** (unchanged from both prior triages): (1) live-wrong on a `Complete`/deck-legal
path; (2) gate integrity — a gate that reports success while checking nothing; (3) cheap
high-yield riders; (4) agency / CR completion. Within a tier, cheaper first. "Discounted ship"
is the expected clean-`Complete` count after the batch, at the historical 2-3× overcount
discount. **Every wire prediction below is a prediction, not a licence** — the implementer
gate-computes `PROTOCOL_SCHEMA_FINGERPRINT` / `HASH_SCHEMA_VERSION` and treats a mismatch with
the prediction as a signal to stop and re-scope (the PB-DP2/DP3 precedent, where two predicted
bumps were falsified).

| rank | batch | seed(s) | class | discounted ship | wire prediction |
|---|---|---|---|---|---|
| **PB-DX1** | the dropped intervening-if | **OOS-DP6-1** (+riders DP6-5, DP6-9) | **CORRECTNESS — live-wrong `Complete`, unbounded** | 0 flips; repairs `aurelia_the_warleader`, unblocks `karlach` + `tatyova` | **HASH** (field on `TriggeredAbilityDef`, inside `Characteristics`) |
| **PB-DX2** | unguarded resolution-time commands | **OOS-DP5-7** + **OOS-DP7-2** (+riders DP2-1, DP9-14) | **CORRECTNESS — live exploit, trust boundary** | 0 flips; closes a free-card exploit + 2 lying doc comments | **none** |
| **PB-DX3** | two stale blocker notes | **OOS-DP6-3** | **card yield, zero engine** | **2 flips** (`garruks_uprising`, `inventors_fair`) | **none** |
| **PB-DX4** | the `BASELINE` triage sweep | **OOS-DP10-8** | **CORRECTNESS — marker integrity** | 0 flips; ≥2 known live-wrong `Complete` defs corrected, 95 entries triaged | **none** (test + card-def markers) |
| **PB-DX5** | CR 611.2c affected-set snapshot | **OOS-OS7-2** *(ex-R6)* | **CORRECTNESS — engine-wide, 7 `Complete` defs** | 0 flips; repairs 7 `Complete` + 2 `partial` defs | **HASH**, and **PROTOCOL** if `ContinuousEffect` is in the wire closure — compute, do not assume |
| **PB-DX6** | the last unflattened pip sites | **OOS-RS2-1** + **OOS-DP4-1** | **CORRECTNESS — live undercharge (narrow)** | 0 flips; closes the OOS-RS-2 class at its 4th and 5th sites | **PROTOCOL** (`DeclareAttackers` gains the two payment-choice fields `ActivateAbility`/`TapForMana` already have) |
| **PB-DX7** | SR-19 gate holes | **OOS-DP7-11** + **OOS-DP9-13** (+DP10-1, DP9-10 residual) | **gate integrity** | 0 flips; 5 structs + all hashed enums re-enter the gate | **none** (test-only) |
| **PB-DX8** | oracle-text-vs-DSL cross-check | **OOS-DP10-9** | **gate integrity — the worst blind spot** | 0 flips; makes dropped "may"/"choose" clauses visible for the first time | **none** (test-only) |
| **PB-DX9** | multi-card search + the inert-field family | **OOS-DP9-3** (+DP9-2, DP9-4, DP9-9, DP10-5) | capability / card yield | **2 flips** (`tooth_and_nail`, `buried_alive`) — **not 7**, see §2.5 | **PROTOCOL + HASH** (`count` on `Effect::SearchLibrary`, `found: Vec<ObjectId>` on the answer) |
| **PB-DX10** | PB-DP8b — modal triggered abilities | **OOS-DP3-4** + **OOS-DP8-7** (+rider DP8-3) | agency / CR 700.2b | 0-1 flips; restores the choice on **4** `Complete` defs (measured, §2.9) | **PROTOCOL + HASH** (one `BlockingDecision` variant, one `Command`, one `LegalAction`) |
| **PB-DX11** | `WouldDraw` widening | **OOS-DP5-6** (+DP5-8, DP5-9) | capability — *the batch that gives PB-DP5's machinery any yield* | **2 flips** (of 3 inert defs: `laboratory_maniac`, `teferis_ageless_insight`, `out_of_the_tombs`) | **PROTOCOL + HASH** (new `ReplacementModification` variants) |
| **PB-DX12** | multi-count sacrifice cost | **OOS-OS6-1** *(ex-R8)* | capability | **3 flips** (of 4 named) | **PROTOCOL + HASH** |
| **PB-DX13** | target-scoped filters | **OOS-OS7-1 R1** + **OOS-RS-5** *(ex-R7)* | correctness + capability | **2 flips** (of 3 named) | **PROTOCOL + HASH** (one shared bump) |
| **PB-DX14** | back-face starting loyalty | **OOS-OS4-1** (+OOS-RS4-3) *(ex-R10)* | capability | **2 flips** | **PROTOCOL + HASH** |
| **PB-DX15** | CR 400.7 / APNAP / delayed-trigger sweeps | **OOS-DP9-11** + **OOS-DP9-16** + **OOS-DP9-8** | correctness, sweeps | 0 flips; repairs 3 engine-wide classes | **none** expected |
| **PB-DX16** | edgar return-transformed | **OOS-OS4-3** *(ex-R9)* | capability, micro | **1 flip**, oracle-gated | **PROTOCOL + HASH** (new `Effect` variant) |
| **PB-DX17** | attacked-player trigger family | **OOS-OS7-1 R2+R3** *(ex-R11)* | capability | **1 new card** (`karazikar` is unauthored) | none (`TriggerCondition` is outside the closure) |
| **PB-DX18** | shuffle integrity micro-batch | **OOS-DP2-7** + **OOS-DP2-4** + **OOS-DP2-8** | correctness (gated) + hygiene | 0 flips; 2 phantom `LibraryShuffled` sites fixed, PRNG pinned, mulligan capped | **none** |

**Designated successor to this queue**: `proliferate` (25 `Complete` defs, the largest
auto-chosen class in §2.9) on PB-DP9's `AnswerEffectChoice` channel. Not ranked here because it
is agency restoration on otherwise-correct cards, but it is the highest-count remaining row and
the machinery is built.

**Sequencing notes.** PB-DX1, DX5, DX6, DX9, DX10, DX11, DX12, DX13, DX14 and DX16 each force a
wire bump; one bump per PB (AC-5040 discipline) and do not batch adjacent bumping capability PBs
just to save churn. **PB-DX3 can be dispatched at any time and blocks nothing** — it is pure
authoring and could ride any batch as a rider. **PB-DX2 and PB-DX1 are independent** and may run
concurrently with each other and with M11-local (which touches `crates/simulator` / `tools/`).

### Dispatch briefs

**PB-DX1 — `PB-DX1: the intervening-if dropped in the runtime lowering (OOS-DP6-1)` · CORRECTNESS**
`build_face_ability_vectors` (`crates/engine/src/testing/replay_harness.rs:2382`) lowers card-def
triggered abilities into runtime `TriggeredAbilityDef`s and hardcodes `intervening_if: None` at
all 34 push sites, because the card-def field is `Option<Condition>` and the runtime field is
`Option<InterveningIf>`. Both the queue site (`abilities::collect_triggers_for_event`) and the
resolution site (`resolution.rs`) read the *runtime* field, so CR 603.4's condition is checked in
**neither** place — and the lowering is reached on live paths (`rules/face.rs:104`,
`rules/resolution.rs:864`), not only in tests. `aurelia_the_warleader.rs` carries no
`completeness` field, is therefore `Complete`, is deck-legal, and PB-DP6 confirmed **by
execution** that she grants herself unbounded extra combats. The plan must choose between the
clean fix (an `Option<Condition>` on `TriggeredAbilityDef` ⇒ **HASH bump**, since the type sits
inside `Characteristics` at `state/hash.rs:3337`) and the seed's alternative (b) — re-routing
these conditions onto a `CardDefETB`-style dispatch that re-reads the registry, which is what the
adjacent `WhenExertedAsAttacks` block already does and is why *it* reaches PB-DP6's gate. **Do
not take alternative (c)** (index correspondence between the two ability lists) without reading
OOS-DP6-2 first: that seed is a live example of an index space assumed and wrong. Mandatory
fail-before probe: drive the engine (no state pokes) through Aurelia's first and extra combats
and assert the trigger fires **once**. Riders, both cheap and both the same CR sentence's other
half: **OOS-DP6-5** (`PendingTriggerKind::TurnFaceUp` has no resolution-time re-check) and
**OOS-DP6-9** (haunt reads the card-def ability and ignores `intervening_if` at both ends).

**PB-DX2 — `PB-DX2: gate the resolution-time commands nothing gates (OOS-DP5-7 + OOS-DP7-2)` · CORRECTNESS**
`Command::ChooseDredge` has no pending-state gate anywhere: `rules/engine.rs:534-544` checks only
that the player exists, and `replacement::handle_choose_dredge` (`replacement.rs:2925`) validates
the *card* but never that a draw is outstanding — so `ChooseDredge { card: None }` is a free card
for any player at any time, and `card: Some(x)` dredges at will. The fix reuses PB-DP5's
machinery almost verbatim: give the `DredgeAvailable` pause its own `PendingDraw`-style entry and
require-and-consume it in the handler; existing dredge tests and golden script `replacement/014`
reach `DredgeChoiceRequired` first and stay green. Fold in **OOS-DP7-2** in the same batch,
because it is the same subject seen from the documentation side: `rules/events.rs` and
`rules/replacement.rs` both assert in doc comments that the engine *pauses* for the answer, and
`DrawStepOutcome`'s own doc says outright that the caller does not — either wire them onto
`blocking_decision` or correct the comments, but do not leave a comment asserting a guarantee the
code does not make. Cheap riders on the same trust-boundary theme: **OOS-DP2-1** (`handle_keep_hand`
checks only the *count* of `cards_to_bottom` and will happily bottom a card from another player's
hand — add a per-entry `obj.zone == ZoneId::Hand(player)` guard plus a duplicate-id check) and
**OOS-DP9-14** (reap `pending_effect_choice` at the top of `resolve_top_of_stack`, mirroring
`drop_departed_trigger_flush`). Wire-neutral throughout; every fix is a guard, not a type.

**PB-DX3 — `PB-DX3: two stale blocker notes — garruks_uprising + inventors_fair (OOS-DP6-3)` · CARD YIELD, ZERO ENGINE**
Both defs are `partial` on a blocker that no longer exists, and both notes name the wrong DSL
type: they describe the **runtime** `InterveningIf` (2 variants) when the def-level field is
`Option<Condition>`, and `Condition::YouControlNOrMoreWithFilter { count, filter }`
(`card_definition.rs:3834`) is exactly what each card needs — "a creature with power 4 or
greater" and "three or more artifacts" respectively. 21 shipped defs already use it. The notes'
hedge that "the trigger-time half remains blocked" is also stale: the variant is in
`condition_is_queue_time_evaluable`'s true set (`effects/mod.rs:10139`) and PB-DP6 wired the
card-def `intervening_if` into all 14 queue sites, so both halves of CR 603.4 hold. Inventors'
Fair needs one extra already-expressible correction its own note records — `activation_condition:
Some(Condition::YouControlNOrMoreWithFilter)` on the search ability, which currently permits an
illegal activation. **No engine change, no wire change, 2 flips.** Verify each against oracle
text via MCP before flipping the marker (both are `partial`, so a wrong flip ships a legal-but-
wrong card), and add a fail-before probe per card that the trigger does *not* fire when the
condition is false at queue time.

**PB-DX4 — `PB-DX4: triage the 97-entry decision BASELINE against oracle text (OOS-DP10-8)` · CORRECTNESS**
PB-DP10 froze 97 `Complete` defs that still carry an engine-made choice into a name-keyed
`BASELINE`, but populated it **mechanically**; the plan's §5.3 class-B (engine picks among legal
options) vs class-D (the def is simply wrong) triage was never performed. A closing-review
spot-check of **5 of the 97 found 2 class-D members**, both verified against oracle text by MCP
this task: **Smuggler's Copter** — printed "you **may** draw a card. If you do, discard a card",
authored as an unconditional `Effect::Sequence(vec![DrawCards, DiscardCards])` and left
`Complete`, so the controller is forced to loot on every attack and block (the 20th instance of
DP-12's class, where the other 19 are `known_wrong` — the fix is a one-line marker change); and
**Shambling Ghast** — printed "-1/-1 **until end of turn**" authored as a *permanent*
`CounterType::MinusOneMinusOne`, a stored `oracle_text` saying "enters" against a `WhenDies`
trigger, and a `KeywordAbility::Decayed` the printed card does not have (MCP confirms: keywords
are `["Treasure"]` only). Both are deck-legal today. The batch is the remaining **95**: read each
def against its oracle text, classify B or D, fix or `known_wrong` the D's, and record the split.
Two extrapolation cautions — 2-of-5 is a very noisy sample and this suite has published a
plausible roster and been wrong three times (PB-DP6's 3-vs-14, PB-DP8's 84-vs-77, PB-DP9's
74/16/8-vs-69/16/7); and per plan §5.3 the default is **file, do not demote**, so a marker flip
needs an oracle citation in the commit. Test-only plus card-def markers; no engine lines, no wire.

**PB-DX5 — `PB-DX5: CR 611.2c — lock the affected set of a resolution-generated continuous effect (OOS-OS7-2)` · CORRECTNESS**
CR 611.2c says the set of objects a resolution-generated continuous effect affects is fixed when
the effect begins and *never changes*. `struct ContinuousEffect`
(`card-types/src/state/continuous_effect.rs:531-561`) has a `filter` and no set, and
`rules/layers.rs:613` re-evaluates `EffectFilter::AllCreatures` live on every characteristics
calculation — so a creature that enters *after* Golgari Charm resolves wrongly gets -1/-1, and a
debuffed creature that changes controller wrongly loses it. **9 defs pair
`Effect::ApplyContinuousEffect` with a mass `EffectFilter::All*`, 7 of them `Complete`**
(`bladewing_the_risen`, `goblin_lookout`, `crippling_fear`, `eyeblight_massacre`,
`golgari_charm`, `olivias_wrath`, `the_meathook_massacre`). This is the item the RS triage filed
as "0 flips; repairs `golgari_charm` + siblings" and it is materially bigger than that. Scope:
add the snapshot to `ContinuousEffect` (an `Option<OrdSet<ObjectId>>`, `None` meaning "static
ability, re-evaluate" so static effects keep their CR 611.2c-correct live behaviour), populate it
at `Effect::ApplyContinuousEffect` execution only, and read it in `is_effect_active` /
`effect_applies_to`. **Compute both fingerprints** — the field is hashed for certain, and
PROTOCOL depends on whether `ContinuousEffect` is inside the wire closure. Mandatory
discriminating probe: mass -1/-1, then a creature enters, then assert the newcomer is unmodified,
citing CR 611.2c. Expect a large test-repair surface; every repair must be CR-justified, never by
weakening an assertion (the PB-DP9 precedent).

**PB-DX6 — `PB-DX6: the last two unflattened mana-cost payment sites (OOS-RS2-1 + OOS-DP4-1)` · CORRECTNESS**
PB-RS2 routed three of the engine's payment sites (`ActivateAbility`, `TapForMana`,
`CastSpellData`) through `ManaCost::flatten_hybrid_phyrexian`. Two were left standing.
(1) `handle_turn_face_up` (`rules/engine.rs`, the `can_spend`/`spend` pair ~`:2137`) pays a raw,
unflattened `def.mana_cost`; `ManaPool::can_spend` reads only the six colours and generic, and
its `debug_assert_flattened` residue guard is **debug-only**, so in release every hybrid and
Phyrexian pip in a `TurnFaceUpMethod::ManaCost` flip is free. `kitchen_finks` is `Complete` with
two `{G/W}` pips: manifest or cloak it and flip it for `{1}`. (2) `Command::DeclareAttackers` has
no `hybrid_choices` / `phyrexian_life_payments` fields at all, so a hybrid or Phyrexian attack tax
cannot be paid — PB-DP4 rescued this from a silent free attack into a hard rejection, which is
better but still wrong. Both need the same two `Command` fields PB-RS2 added elsewhere ⇒ one
**PROTOCOL** bump for the batch. Also make `can_spend`/`spend` fail *loud* (not `debug_assert`) on
a non-empty hybrid/Phyrexian residue — an SR-4-shaped guard there would have caught the seven
filter lands years ago. Latent on the attack-tax half (`propaganda.rs` / `ghostly_prison.rs` are
pure generic); live on the `TurnFaceUp` half.

**PB-DX7 — `PB-DX7: the SR-19 gate reports success while checking nothing (OOS-DP7-11 + OOS-DP9-13)` · GATE INTEGRITY**
`crates/engine/tests/core/hash_schema.rs::every_hashed_struct_field_is_hashed_or_allowlisted`
looks impl bodies up by the **bare** struct name (`:1540-1541`, `let Some(body) = bodies.get(ty)
else { continue; }`) while `hashinto_impl_bodies()` keys them by the exact type token as written,
so a path-qualified `impl HashInto for crate::state::stubs::Foo` silently leaves the struct out of
scope with no diagnostic. Five structs are outside it today — `MergedComponent`, `FlashGrant`,
`PlayFromTopPermission`, `PlayFromGraveyardPermission`, `SacrificedCreatureLki` (all 15
path-qualified impls were classified for this triage; the remaining 10 are enums). The same gate
iterates `named_field_structs()`, so **enum variants are outside it by construction**
(OOS-DP9-13, demonstrated by deleting a `hash_into` feed from an `EffectChoiceQuestion` variant
and watching every gate stay green). Fix both in one test-only batch: normalise the scanner's key
on the bare name — do **not** rename the five call sites, which leaves the hole able to reopen —
and extend the scan to enum variants. Fold in **OOS-DP10-1** (the copied serde walk that matches
object keys only) and file the outcome against **OOS-DP9-10**'s residual (there is still no gate
for unordered-iteration-to-outcome). Mandatory demonstration in both directions: delete a field's
feed from one struct impl and one enum variant impl and show the gate reddening by name, then
restore.

**PB-DX8 — `PB-DX8: the decisions the DSL never encoded (OOS-DP10-9)` · GATE INTEGRITY**
PB-DP10's gate classifies 22 decision sites by walking for DSL **variant names**, so a choice
dropped at authoring time — a "you may X. If you do, Y" written as a bare `Sequence`, a "choose
one" flattened to its first mode — hits zero rows and passes forever. That class is *strictly
worse* than the one the gate records: a recorded auto-choice is at least a legal outcome, while a
dropped "may" is not. Smuggler's Copter is the live example and appears in `BASELINE` only by the
incidental `DiscardCards`. The instrument is different from a variant walk: cross-check
`oracle_text` (already on the def, already parsed by `effect_choose_gate.rs`) for `may` /
`choose` / `up to` against the presence of any decision-bearing variant anywhere in the effect
tree, with a curated exception list for the phrasings that are not choices ("you may cast",
reminder text in parentheses, keyword reminder blocks). Test-only and feasible today. Expect a
noisy first run — the deliverable is the triaged list, not a green gate on day one, and the
exception list must be argued per entry rather than tuned until quiet.

*(PB-DX9 .. PB-DX18: the table above carries scope, class, discounted yield and wire prediction
for each. Their seeds are fully specified in their own filing rows —
`docs/audits/decision-point-audit.md` §8.1 for the DP-family items, and
`memory/primitives/rider-seed-triage-2026-07-19.md` §1a-§1c for the ex-RS items — and each
re-verification performed by this task is in §2 / §3 above. A dispatcher should write the
one-page brief at dispatch time from those two sources plus this document's corrections.)*

---

## 5. Parked — real, do not queue

| item | why parked |
|---|---|
| **OOS-RS-4** (ex-R5) | LOW severity, 0 flips, 1 `Complete` card, narrow interaction, and the obvious fix returns 0 (§2.7). Re-rank only with a CR 608.2h/113.7a LKI-capture batch. |
| **OOS-RS1-1** / **OOS-OS8-2** | `ZoneTarget::Library { position }` still discarded (`effects/mod.rs:8594`); muxus stays inexpressible. Capability, 1 card. |
| **OOS-RS-6** (crucible dynamic-X) | 1 card, 3 coupled gaps, touches the PB-OS11 mana-ability lowering path. |
| **hidden_strings optionality** | Was blocked on "the missing M10+ interactive-decision channel". **That channel now exists** (PB-DP7/DP8/DP9). Re-scope before dismissing it again. |
| **OOS-RS3-2** | A re-marking sweep over 8 textually-admitted gaps, not a primitive. Its one live-wrong member, `emeria_the_sky_ruin` (`intervening_if: None, // TODO DSL gap`), is worth pulling forward alone — and note PB-DX3's finding that this class of blocker note is often stale. |
| **OOS-RS3-3**, **OOS-RS4-1/2/4**, **OOS-DP3-5**, **OOS-DP6-5/6/9/10**, **OOS-DP7-9/10**, **OOS-DP8-4/5/11/13**, **OOS-DP9-5/14/16/19** | Latent — real, 0 reachable yield on today's roster. Most are named as riders in §4; the rest activate when a specific card lands. |
| **OOS-DP8-5 + OOS-DP9-18** | The CR 800.4a object sweep (a player who leaves takes their objects with them). Wide, its own batch, not a rider. |
| The 31 legacy deferred/dormant seeds (§1e) | Unchanged since 2026-07-18; none closed by a later wave. Build only when a card demands it. |

---

## 6. Source-doc updates applied by this task

**Zero engine/card-def code changed.** Doc-only edits:

1. **`memory/primitives/seed-rerank-2026-07-27.md`** — this file (new). The authoritative queue.
2. **`memory/primitives/rider-seed-triage-2026-07-19.md`** — §5's banner rewritten: the PB-RS
   queue is **RETIRED**, RS5..RS11 dispositions recorded inline, and the two insert candidates
   the old banner named are resolved (OOS-RS3-1 **closed by PB-DP6**; OOS-RS2-1 **re-ranked to
   PB-DX6**). §3's rank table marked superseded. §1c's OOS-RS3-1 row marked CLOSED.
3. **`docs/audits/decision-point-audit.md`** — §8.1 rows updated with this task's verdicts
   (OOS-DP8-6 premise stale; OOS-DP6-3 scope widened and verified; OOS-DP9-3 yield corrected
   7 → 2; OOS-DP7-11 enum count 9 → 10; OOS-DP2-7 gated-not-live; OOS-DP10-3 aliased to
   OOS-AC9-FILTERMANA; OOS-DP5-5 machinery now exists), and §8 given a pointer to this queue.
4. **`memory/primitives/oos-retriage-plan-2026-07-18.md`** — §1d's `OOS-AC9-FILTERMANA` row
   cross-referenced to OOS-DP10-3 as the same gap.
5. **`memory/primitives/pb-review-RS1.md`** — the conditional **OOS-RS1-2** marked
   **NEVER-FILED (phantom)**: the review's Finding 1 was fixed, not deferred.
6. **`CLAUDE.md`** — Current State's queue line repointed at this document.
7. **`memory/workstream-state.md`** — queue pointer updated.

Nothing else needed a per-doc status change. **One seed was found resolved-stale** (OOS-RS3-1)
and one phantom struck (OOS-RS1-2) — which, set against `scutemob-142`'s finding that *no* rider
seed had gone stale in the week after filing, is the expected shape: the DP suite ran for ten
batches across the same subsystems the RS queue was waiting on, and it closed one of its items
in passing.
