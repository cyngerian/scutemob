# Seed Re-rank v3 — the post-2026-07-27 seeds folded into the PB-DX queue (2026-08-02, task `scutemob-182`)

<!-- last_updated: 2026-08-02 -->

> **This document is the authoritative primitive queue.** It supersedes
> `memory/primitives/seed-rerank-2026-07-27.md` §4 (the PB-DX1..DX18 queue) as the thing a
> dispatcher reads to pick the next batch. The v2 document remains the canonical record of *why*
> PB-DX1..DX18 exist and the canonical filing record for the seeds it triaged — its §1/§2/§3 are
> still true and still worth reading. **Its §4 queue is superseded**: PB-DX1..DX6 shipped, and the
> ranking of what remains has changed because 80 seed rows were filed after its census closed.
>
> **Precedents / structural models**: `seed-rerank-2026-07-27.md` (`scutemob-159`),
> `oos-retriage-plan-2026-07-18.md` (`scutemob-115`), `rider-seed-triage-2026-07-19.md`
> (`scutemob-142`). Same shape: headline → full census → chain-verification notes → ranked queue →
> parked → source-doc edits.
>
> **Method (binding, per `feedback_retriage_verification` / `feedback_verify_full_chain` /
> `feedback_pb_yield_calibration`)**: a closure is believed only when the *shipped code* says so,
> never a banner or a status column; a seed premise is re-derived from source and CR (MCP) rather
> than copied from the filing row; card scope comes from the compiled corpus enumerated by file,
> not from a seed's estimate; yields are discounted 2-3×.
>
> **Zero engine/simulator/tool/card-def code changed by this task.** Docs + triage only. The
> `git diff` is confined to `memory/`, `docs/` and `CLAUDE.md`.
>
> **Engine baseline this triage was verified against**: HEAD `8195b109` (contains `origin/main`),
> PROTOCOL **33** / HASH **70**, coverage 1,133/1,803 = 62.8%, tests 4,263 at `b76b1df4`.

---

## 0. Headline

Five things, all census or verification findings rather than ranking ones.

1. **The census is twice the size the brief expected, and the reason is a cutoff, not an
   oversight.** The task brief scoped "~40 seeds (DX6 + the 174-181 run)". The real post-2026-07-27
   population is **80 rows / 79 distinct IDs**. The extra 36 rows are **PB-DX1..DX5's 29**
   (`OOS-DX1-1..6` 6, `OOS-DX2-1..7` 7, `OOS-DX3-1` 1, `OOS-DX3b-1` 1, `OOS-DX4-1..6` 6,
   `OOS-DX5-1..8` 8 = 29) plus the **seven M11-local rows** (`OOS-M11-5..10` — six IDs, seven rows,
   because `OOS-M11-10` is two seeds; see finding 4). 29 + 7 = 36.
   v2's own census was taken on **2026-07-31**; every PB-DX batch shipped on **2026-08-01**. So the
   queue document that ranks PB-DX1..DX18 has never seen the seeds PB-DX1..DX5 filed. Four of those
   29 are live-wrong on deck-legal `Complete` cards today.

2. **The registry's only self-declared HIGH is also the cheapest item in the entire inventory to
   close, and it is a hard process abort.** `OOS-SIM2-6` — `calculate_characteristics` recurses
   without bound — was verified hop-for-hop against HEAD (§2.1). The class is **exactly one card**
   (`indomitable_archangel`, `Complete` by the `#[default]` derive), the fix is **one line**
   (`effects/mod.rs:10259`, read `&obj.characteristics` instead of `expect_characteristics`), and
   the *sibling* counter 1,800 lines away in the same subsystem already made that exact choice for
   this exact reason (`layers.rs:2304-2310`). It has been live for **four and a half months**
   (def `d83ac94d` 2026-03-12, recursive call site `aa23d26c` 2026-03-23) behind a comment that
   argues termination from the wrong invariant and a test that names the card while hand-building
   the effect with `condition: None`. **It is PB-DX19, the first dispatch.**

3. **Four seeds filed as "latent" are live-wrong on deck-legal `Complete` cards, and in three of
   eight cases a one-line grep would have caught it.** `OOS-DX1-3` (`nether_traitor`),
   `OOS-DX2-5`/`-2`/`-7` (`golgari_grave_troll`), `OOS-DX4-2`
   (`retreat_to_kazandu`), `OOS-DX4-6` (all ten Karoo bounce lands + two more). Each row's
   "no corpus def pairs the two" clause was checked against the corpus and is false. The
   `#[default] Completeness::Complete` derive explains **five** of the eight defs (§2.5) — this is
   its third through seventh recorded instance (`aurelia_the_warleader`
   PB-DX1, `emeria_the_sky_ruin` PB-DX3b, and now these). v2 already called it "a twice-demonstrated
   silent-defect generator"; it is now seven-times-demonstrated. But `nether_traitor`,
   `qarsi_sadist` and `voldaren_epicure` declare `Completeness::Complete` **explicitly**, so the
   derive is not the whole story — **the actual shared mechanism is that nobody looked**, and
   **no seed owns the corpus-wide question** either way. Filed here as **`OOS-RR3-1`**.

4. **`OOS-M11-10` is still one ID for two seeds, and one half is closed.** The equip half is CLOSED
   (CARDS-1, `scutemob-179`); the loyalty-ability targeting half is OPEN and is what every cite
   outside `docs/audits/decision-point-audit.md`'s own table means. Any automated "is OOS-M11-10
   closed?" query returns the wrong answer for whichever half it did not mean. This document writes
   them as **`OOS-M11-10(equip)`** and **`OOS-M11-10(loyalty)`** and §6 renumbers the closed one.

5. **The project's main correctness instrument has never exercised the code paths its acceptance
   evidence claims.** `OOS-UI2-1` (the fuzzer never shuffles; basics are appended last; `Zone::top`
   is the last index) is confirmed link-for-link at HEAD, and `OOS-SIM3-1`'s qualification is
   arithmetically consistent with it: the first non-land is personal draw ~35-40, i.e. game turn
   ≈136-156, and SIM-3 measured its earliest cast at turn 143. **Both are right at different
   depths.** Every "fuzz parity" acceptance claim taken at `--max-turns` below ~140 is a claim
   about a land-only game, and `OOS-SIM1-4` adds that the fuzzer registers no commander at all, so
   CR 903.8/903.9a/903.10a have never been fuzzed either. The three close together as **PB-DX22**,
   because each of them re-rolls every recorded seed and paying that once is the whole point.

**The ranking convention is unchanged** (inherited from all three prior triages): live-wrong on a
`Complete`/deck-legal path first; then gate/evidence integrity; then cheap high-yield riders; then
agency/quality. Applied to the merged inventory, **none of the standing PB-DX7..DX18 items holds
the top of the list** — the first eight ranks are all new. PB-DX7 (the SR-19 gate holes), which
CLAUDE.md currently advertises as "next dispatch", ranks **ninth**.

**Honest discounted yield across the new entries**: ~6-10 clean completeness flips (PB-DX26 and
PB-DX27 carry nearly all of them), plus correctness repairs on ~55 already-`Complete` deck-legal
cards, one hard-crash closure, and three instrument repairs whose value is that they stop a green
suite from lying.

---

## 1. Full seed census (AC 6031)

### 1a. Scope, method and totals

**Scope**: every `OOS-*` seed filed after 2026-07-27 — which, because v2's census closed on
2026-07-31 and PB-DX1..DX5 shipped on 2026-08-01, means everything v2 did not see.

**Method**: the union of three independent passes, cross-checked against each other.

| pass | source | what it contributes |
|---|---|---|
| A | `memory/workstream-state.md`, every post-cutoff Worker Handoff section, read in full | per-batch narrative, "not done / deliberate" lists, the closures a later batch performed |
| B | `memory/archive/claude-md-changelog-2026-08.md`, read in full | the rotated per-batch entries; **cites concrete IDs only sparsely** — most filings are recorded as ranges |
| C | `docs/audits/decision-point-audit.md` §8.1 (which starts at L823; its **post-cutoff rows** are L1013-1084) + `memory/card-authoring/cards2-field-fidelity-2026-08-02.md` §5 — the section is `## 5. Cross-references and seeds`, **not** §7; that doc has five sections — + a repo-wide `OOS-` grep | the **authoritative** row-per-seed registry; the only place every concrete ID exists |

**Wildcards resolved.** Neither narrative source enumerates concretely; both write ranges. Every
one was expanded against pass C and cross-checked:

| token, as written | where | concrete expansion | resolved against |
|---|---|---|---|
| `OOS-DX1-1..6` | archive (rotated PB-DX1 entry) | DX1-1..DX1-6 | audit §8.1 L1013-1018 |
| `OOS-DX2-1..7` | archive | DX2-1..DX2-7 | audit §8.1 L1019-1025 |
| `OOS-DX4-1..6` | workstream-state's **W6 row** in Active Claims | DX4-1..DX4-6 (only -1/-2/-3 described in prose) | audit §8.1 L1028-1033 |
| `OOS-DX5-1..5` + `-6` (W6 row) / `OOS-DX5-1..7` (the PB-DX5 **Seeds** line) | workstream-state | **DX5-1..DX5-8** — both written ranges are stale; `OOS-DX5-8` exists and neither narrative mentions it | audit §8.1 L1035-1042 |
| `OOS-DX6-1..5` | workstream-state's **W6 row** in Active Claims | DX6-1..DX6-5 (only -1/-3/-5 described) | audit §8.1 L1049-1053 |
| `OOS-UI2-1..5` | the UI-2 handoff's **Seeds filed** line | UI2-1..UI2-5 | audit §8.1 L1070-1074 |
| `OOS-SIM3-*` | the **wildcard token** is CLAUDE.md's wording only. Pass A resolves it in full — the SIM-3 handoff's **Seeds filed** block names all five individually. Pass B does **not**: the archive writes the range `OOS-SIM3-1..5` and names -1, -2, -3 and -5 in prose but never -4, so it resolves 4 of 5 | **SIM3-1..SIM3-5** | audit §8.1 L1075-1079 |
| `OOS-CARDS2-1..11` | the CARDS-2 handoff's **Full evidence record** line | CARDS2-1..CARDS2-11 (**only -9 is in audit §8.1**, at L1062; the other ten live in the CARDS-2 evidence record) | `cards2-field-fidelity-2026-08-02.md` §5, L286-365 |
| `OOS-CARDS2-5/6` (slashed pair) | the CARDS-2 handoff's demotion narrative | CARDS2-**5** (`cyber_conversion`) and CARDS2-**6** (`exalted_angel`), two seeds | evidence record L157, L161 |

**Totals.**

| family | rows | filed by | registry cite |
|---|---|---|---|
| `OOS-DX1-1..6` | 6 | PB-DX1 (`scutemob-160`) | audit §8.1 L1013-1018 |
| `OOS-DX2-1..7` | 7 | PB-DX2 (`scutemob-162`) | audit §8.1 L1019-1025 |
| `OOS-DX3-1` | 1 | PB-DX3 (`scutemob-164`) | audit §8.1 L1026 |
| `OOS-DX3b-1` | 1 | PB-DX3b (`scutemob-166`) | audit §8.1 L1027 |
| `OOS-DX4-1..6` | 6 | PB-DX4 (`scutemob-168`) | audit §8.1 L1028-1033 |
| `OOS-DX5-1..8` | 8 | PB-DX5 (`scutemob-170`) | audit §8.1 L1035-1042 |
| `OOS-DX6-1..5` | 5 | PB-DX6 (`scutemob-172`) | audit §8.1 L1049-1053 |
| `OOS-M11-5..10` | **7 rows / 6 IDs** | M11-local S5/S4/S8 + the playtest triage | audit §8.1 L1034 (M11-5) + L1043-1048 (M11-7/-9/-10 loyalty/-6/-8/-10 equip) |
| `OOS-CARDS1-1..3` | 3 | CARDS-1 (`scutemob-179`) | audit §8.1 L1054-1056 |
| `OOS-SIM1-1..4` | 4 | SIM-1 (`scutemob-175`) | audit §8.1 L1057-1060 |
| `OOS-CARDS2-9` | 1 | retro-filed by SIM-2 into the CARDS-2 family | audit §8.1 L1062 |
| `OOS-SIM2-1..7` | 7 | SIM-2 (`scutemob-176`) | audit §8.1 L1063-1069 |
| `OOS-UI2-1..5` | 5 | UI-2 (`scutemob-178`) | audit §8.1 L1070-1074 |
| `OOS-SIM3-1..5` | 5 | SIM-3 (`scutemob-177`) | audit §8.1 L1075-1079 |
| `OOS-UI3-1..4` | 4 | UI-3 (`scutemob-180`) | audit §8.1 L1081-1084 |
| `OOS-CARDS2-1..8,10,11` | 10 | CARDS-2 (`scutemob-181`) | `cards2-field-fidelity-2026-08-02.md` §5 |
| **total** | **80 rows / 79 distinct IDs** | | |

**Completeness checks performed, and what each found:**

- **UI-1 (`scutemob-174`) filed zero seeds.** There is no `OOS-UI1-*` family anywhere in the repo.
  Its handoff's "not done" list cites only pre-existing rows (`OOS-DP7-6`, `OOS-DP8-2`,
  `OOS-DP9-7`, `OOS-DP9-1`). Recorded here so a future reader does not go looking.
- **`OOS-CARDS2-9` was filed by SIM-2, not CARDS-2**, and is the only member of its family in
  audit §8.1. The other ten CARDS-2 seeds are in the evidence record and would be invisible to a
  re-rank driven off §8.1 alone. A repo-wide `OOS-CARDS2` grep found no filing outside those two
  documents (every source hit is a *citation*, not a filing).
- **`OOS-M11-5` is never mentioned in the 2026-08 archive**, only in workstream-state (the M11-local S5 narrative)
  and audit §8.1 (L1034). Pass B alone would have missed it.
- **PB-DX1..DX4's seeds are absent from workstream-state's per-batch handoffs** — those sections
  are rotated out; only the 30k-character W6 table row at L18 survives, and it names DX1's seeds
  not at all and DX4's/DX5's/DX6's only partially. Pass A alone would have missed 20 rows.
- **The `OOS-DX5-1..7` range in workstream-state is stale by one**: `OOS-DX5-8` is in the registry
  and in neither narrative.

**Three IDs that exist outside the registry, and what they mean for a registry-driven re-rank:**

- **`OOS-DX1-7` was filed in a plan and never registered.** `pb-plan-DX1.md:812` proposes it
  ("*only if §7.4 confirms*" — `rules/protocol.rs`'s `- 25:`/`- 26:` History parentheticals
  mis-state the wire closure) and `pb-review-DX1.md:185` resolves it: "File OOS-DX1-7 as
  **closed-on-arrival** (§7.4's prose corrections did land)". It has **no row in audit §8.1** and
  never will. **Dispositioned CLOSED-ON-ARRIVAL**, doc-only, nothing to do. Recorded because pass C
  claims a repo-wide `OOS-` grep and a reader checking that claim will find this ID and wonder.
- **`OOS-DX1-6` names two different things.** The plan file's `OOS-DX1-6` (above) and the
  registry's `OOS-DX1-6` (Tatyova's `(6)`-for-"seven", §1b) are unrelated — the number was reused
  when the plan's speculative rows were dropped and the shipped ones renumbered. Cite the registry
  row, never the plan row, and treat any pre-ship `pb-plan-*.md` seed number as provisional.
- **`OOS-RR3-1` / `OOS-RR3-2`** are filed by *this* task (§1f) and are likewise not in §8.1, for
  the stated reason that §8.1 is "seeds filed by shipped PB-DP work".

> **⚠️ A line cite into a document you are also editing is self-invalidating — and this task
> proved it twice.** The `§8.1 L###` cites above are stated **as of this commit**, which itself
> inserted **two** banners above the §8.1 table (8 lines at the §8 header, 11 more at the
> re-rank note) — a **+19** shift. The cites were computed before those edits, shipped wrong, and
> were corrected only after a review caught them. Then the *fix cycle* reproduced the same defect
> in the other direction: it re-derived the audit cites but left this memo's
> `workstream-state.md` line cites stale by **+96**, because this task had inserted a 96-line
> handoff section into that file. Those are now written as **section names, not line numbers**,
> which is the only form that survives the edit. **Re-derive §8.1 cites by symbol**
> (`grep -n '\*\*OOS-'`) rather than trusting the numbers above — the convention `OOS-DX2-2`'s row
> adopted after its own cites drifted three times inside one batch. The rule generalises: *cite a
> file you are editing by symbol; cite a file you are not editing by line if you must.*

**The census-integrity lesson, stated plainly**: none of the three passes alone is complete.
Pass A misses 20 rows (rotated handoffs), pass B misses at least `OOS-M11-5` and records almost
everything as an unresolvable range, and pass C misses 10 rows (the CARDS-2 family). A future
re-rank must run all three and reconcile, and the audit's §8.1 must be treated as the *registry*
while the narratives are treated as *provenance*.

### 1b. CLOSED — verified against shipped code, not banners (AC 6032)

| seed | claimed closer | verification performed this task | verdict |
|---|---|---|---|
| **OOS-DX1-5** (Aurelia's `IsFirstCombatPhase` proxy) | PB-DX1 review F1 | `aurelia_the_warleader.rs:52-54` — `once_per_turn: true, intervening_if: None`, with the 24-line justification at `:28-51`; `replay_harness.rs:2559` propagates `once_per_turn` into `TriggeredAbilityDef`; `karlach_fury_of_avernus.rs:48` correctly *retains* its `IsFirstCombatPhase` | **CLOSED** |
| **OOS-DX1-6** (Tatyova's `(6)` for "seven") | PB-DX1 review F5 | `tatyova_steward_of_tides.rs:92` — `ControlAtLeastNOtherLands(7)`. The seed's **sweep** half was *executed* this task, two ways (all 31 `intervening_if: Some` defs' count arguments vs their oracle numbers; all **14** `ControlAtLeastNOtherLands` files) — **0 further approximations** | **CLOSED (fix); sweep executed, 0 yield** |
| **OOS-DX3-1** (the stale-blocker bucket) | PB-DX3b (`scutemob-166`) | all four claimed repairs read at source: `jadar_ghoulcaller_of_nephalia.rs:61-66` + `:89`, `ophiomancer.rs:61-65` + `:76`, `dwynen_s_elite.rs:59-62` + `:73`, `emeria_the_sky_ruin.rs:73-76` + `:113` (explicit `partial`, spurious `Legendary` removed) | **CLOSED** — but its *generalisation* has no ID; see `OOS-RR3-2` in §1f |
| **OOS-DX5-6** (the "checked non-finding" that was a finding) | PB-DX5 fix cycle | the corrected account is in-source at `layers.rs:1151-1156`; the pin is `pb_dx5_affected_set_snapshot.rs:1086` (T15); both principals (`mirror_entity`, `inkmoth_nexus`) confirmed `Complete` by derive | **CLOSED — behaviour is CR-correct as shipped** |
| **OOS-DX5-7** (source-relative filters applied to nobody) | PB-DX5 fix cycle | the mechanism is `layers.rs:658-660` — the `affected_set` membership return sits **above** the whole `match &effect.filter` block at `:661`, so a locked effect never re-consults the source. Pin T12 at `pb_dx5_affected_set_snapshot.rs:817-853` | **CLOSED — residual open, see §1c** |
| **OOS-M11-6** (colourless commander → Forest padding) | PB-DX4, incidentally | `deck.rs:107-140` pads from the identity-filtered `eligible` pool; `deck.rs:160-178` — the `CardId("forest")` fallback arm is **deleted**; the signature is now `-> Option<DeckConfig>`, i.e. it refuses rather than building an illegal deck | **CLOSED** |
| **OOS-M11-8** (announced `{X}` unpayable) | S8 (human half) + SIM-2 (bot half) | one `auto_tap_commands_for` at `local_game.rs:652-691`, using `effective_cast_cost_with_additional` (`:666`) and `x_value.saturating_mul(cost.x_count)` (`:673-675`); **both** callers reach it — human `submit` at `:542`, bot `advance()` at `:462-467`. Pin `sim2_mana_intelligence.rs:1184` (t21) | **CLOSED (both halves)** |
| **OOS-M11-10(equip)** | CARDS-1 (`scutemob-179`) | measured from the corpus: **22** defs mention `AttachEquipment`, **17** now carry `TargetCreatureWithFilter` — matching the closure's roster exactly. Gates `cards1_equip_target_roster.rs:66` (R1, exact-17 with a non-vacuity floor) and `cards1_equip_target_repair.rs` | **CLOSED — see the warning in §1f** |
| **OOS-CARDS2-9** (provider counted mana it could not activate) | SIM-2 (`scutemob-176`) | `mana_solver.rs:295-360` `tap_ability_is_activatable`, five arms each with its CR cite; both call sites live (`legal_actions.rs:727-731` offer loop, `mana_solver.rs:395` solver gather); four discriminators at `sim2_mana_intelligence.rs:645/774/958/1301`; the two `KNOWN_FALSE_OFFERS` strings are gone | **CLOSED** |
| **OOS-UI2-2** (`TapForMana` scored above `PassPriority`) | SIM-2 | `heuristic_bot.rs:244` — `LegalAction::TapForMana { .. } => 0`, against `:249` `PassPriority => 1`; header comment at `:9` and rationale at `:225-243` | **CLOSED** |
| **OOS-UI2-3** (`squad_max_count` under-report) | UI-2 (cause 1) + SIM-2 (causes 2 **and 3**) | cause 1 `legal_actions.rs:1975-1987` sums real `produces`; cause 2 the pin flipped as its own instruction said — `legal_actions.rs:3677` `squad_max_count_counts_true_production_now_that_f4_is_closed`, asserting `1`; **cause 3 was closed without the row knowing**: `can_afford` at `:1752-1757` is now a single `solve_mana_payment_with_pool` call | **CLOSED (all three causes)** |

**Ten of eleven claimed closures hold. The eleventh, `OOS-UI2-3`, was closed *further* than
recorded** — its third cause was `OOS-M11-2`'s `can_afford` pool-OR-sources split, and SIM-2 closed
that too. `OOS-M11-2`'s surviving residue is therefore smaller than CLAUDE.md currently states:
cost **modifiers** and CR 106.12 restricted mana only, on the layer-resolution path.

### 1c. STALE PREMISES and CORRECTED SCOPE — the seed is open but says something no longer true

Every one of these was ranked on the corrected reading, not the filed one.

| seed | what the row says | what HEAD says | effect on rank |
|---|---|---|---|
| **OOS-DX1-3** | "latent — no corpus def pairs a lowered trigger with `trigger_zone: Some(Graveyard)`" | **False.** `nether_traitor.rs:35/:57/:60` pairs `WheneverCreatureDies` with `trigger_zone: Some(Graveyard)` and is `Completeness::Complete`. Also the loss is **not uniform**: the `WheneverPermanentEntersBattlefield` lowering arm *does* skip (`replay_harness.rs:3034`), the `WheneverCreatureDies` arm (`:3249-3288`) does not | **latent → live-wrong.** Ranked 6th (PB-DX24); the narrow fix is one line and wire-neutral, not the HASH bump the row implies |
| **OOS-DX2-2 / -5 / -7** | "latent"; `-5` scoped to "the bots never dredge" | **False on both counts.** `golgari_grave_troll` declares no `completeness` field → `Complete`, deck-legal. And there is no `LegalAction::ChooseDredge` variant **at all** (`grep` over `crates/simulator` and `tools`: zero) — the human seat in the shipped browser is as mute as the bots | **latent → live-wrong, and one client wider.** Ranked 5th (PB-DX23) |
| **OOS-DX2-6** | "three locations cite CR 726 for mandatory loops" | **74 occurrences across 25 files** (measured at `main`; this memo itself adds one). Most are the paired `CR 104.4b / CR 726` form, which self-consistently reads as correct. One hit (`walk_in_closet.rs:20`) is a different citation and must be excluded from a mechanical sweep | scope ×25; still LOW |
| **OOS-DX4-2** | premise: "every consumer of `mode_targets` lives in `rules/casting.rs`" | **False, and was false when filed.** `abilities.rs:426-484` (`mode_targets_active`) has honoured it on the *activated* path since `96cbbd12` / `scutemob-108`, 60 tasks earlier. The conclusion survives (the *triggered* path ignores it) but there are **two** precedents to copy, one in the same file | fix cost down; and see next row |
| **OOS-DX4-2** (severity) | both named members are `partial`, so it reads latent | `retreat_to_kazandu.rs` declares **no `completeness` field** → `Complete`; its flat `targets: [TargetCreature]` makes a creature mandatory for its "You gain 2 life" mode, so CR 603.3d removes the whole landfall trigger on an empty board | **latent → live-wrong** |
| **OOS-DX4-6** | "two `Complete` defs" | **≥14.** All ten Karoo bounce lands (`azorius_chancery.rs:39` and nine siblings) plus `whitemane_lion` and `shrieking_drake` declare a real `TargetRequirement` for a printed *untargeted* choice. None carries a `completeness` field. The deviation is **exploitable in the controller's favour** (respond by moving the chosen land, the ETB fizzles under CR 608.2b, keep both lands) | scope ×7; the single largest ranking error in the inventory |
| **OOS-DX5-1** | "12 exempt creation sites, all `SingleObject`" | **13**, and the 13th (`resolution.rs:7809`, `ClassLevelAbility`) is neither a keyword grant nor `SingleObject` — it forwards `continuous_effect.filter` verbatim. Exposure is still nil (0 `ClassLevel` defs register a static) | scope +1; rank unchanged |
| **OOS-DX6-4** | "~320 construction sites" | **337** (`grep -rn "Command::DeclareAttackers {"`). The number grows with the codebase | strengthens the "batch it with DX6-1" argument |
| **OOS-DX6-5** | cites `input.rs:616/:632/:666` | sites are now `:624/:642/:680` — shifted by the fix cycle's own comments. Verifying by line number alone reads as "not found" and risks a false closure | cite drift only |
| **OOS-M11-10(loyalty)** | "the six-arm `params.rs` allowlist" | **nine arms** now (`params.rs:234-244`); UI-1 added three. `ActivateLoyaltyAbility` is still outside it, so the conclusion holds | fix cost re-based |
| **OOS-SIM1-3** | frames itself as complete by enumerating `GameRestriction` | its own later correction is the right one: **`GameRestriction` is not the only cast gate**. `has_split_second_on_stack` (`casting.rs:6997`) is unmirrored too — `grep 'split_second' crates/simulator/src/` returns nothing | severity unchanged; the *lesson* is promoted to §2.6 |
| **OOS-UI2-4** | "fourteen of sixteen `AdditionalCost` variants" | **15 variants, not 16**, and "Kicker" is not one of them (it is `CastSpellData.kicker_times`). The true residue is **13** | arithmetic corrected |
| **OOS-UI2-5** | "`params.rs` appends `eligible[0]` for any unparameterised submission, **including the TUI's**" | **Premise false — the TUI never calls `params.rs`.** `tools/tui/src/play/input.rs:180-201` hand-builds `CastSpellData { additional_costs: vec![], .. }`; repo-wide grep for `action_to_command_with_params` under `tools/` finds only comments. The TUI gap is real but is a *refusal*, not a silent sacrifice | severity down; **merged into PB-DX33 with `OOS-DX6-5`**, which is the identical defect in the same file |
| **OOS-SIM2-5** | "the four P/T arms in `layers.rs`" | **10 unchecked sites**, including the ±1/+1 counter path at `:394/:397` that every game exercises. Also: `Cargo.toml:51-54`'s `[profile.fuzz]` sets `overflow-checks = true`, so a fuzz-profile run **panics** where a plain `--release` run **wraps silently** — two supported invocations, two failure modes | scope ×2.5; folded into PB-DX19 |
| **OOS-SIM3-5** | states (b) "countering a copy moves the original's card" as a flat fact | **(b) is unreachable at HEAD**: the lookup is `position()` (first match = lowest index = bottom of stack), so with the original still on the stack a counter always lands on the original; with it gone, `TargetSpell` validation refuses. Meanwhile the row's **(c) rider** — a plain Counterspell on a mutate spell is a silent no-op — is the one that is live, on 6 `Complete` mutate defs × 24 counter defs | **rank on (c), fix all three in one edit** |
| **OOS-CARDS2-10** | "Delighted Halfling also moves decks" | **Stale.** `delighted_halfling.rs:47` is `Completeness::partial` now, and `random_deck` filters on `is_complete()` before computing identity. But the row *understates* the rest: `qarsi_sadist` and `voldaren_epicure` are **`Complete`** and silently under-deliver their printed text | 2 live-wrong cards, not zero |
| **OOS-CARDS2-6** | framed as a pure capability gap blocking one `partial` def | **There is a live rider the row does not state.** `TriggerCondition::WhenEnchantedCreatureDealsDamageToPlayer { combat_only: bool }` already exists; `abilities.rs:5604-5612` is an engine `TODO(PB-37)` saying the `false` arm never fires and naming `curiosity`, `ophidian_eye`, `sigil_of_sleep`. `sigil_of_sleep` sets `combat_only: false` and is `Complete` by derive | **capability → correctness**; ranked as PB-DX36 |
| **OOS-CARDS1-1 vs -2** | written as though `-1` outranks `-2` | the completeness markers **invert** it: `darksteel_garrison` (CARDS1-1) is `partial`; `lizard_blades` (CARDS1-2) is **`Completeness::Complete`** at `:86` and is the corpus's only Reconfigure def | CARDS1-2 promoted into PB-DX20; CARDS1-1 stays with the card batch |

### 1d. ACTIVE candidates — ranked into §4

**80 rows** (79 distinct IDs; `OOS-M11-10` is two seeds). Minus **11** verified CLOSED (§1b),
minus **2** further rows that are design records rather than work (§1e lists three, but
`OOS-DX5-6` is *also* one of the 11 closures — count it once, under CLOSED), minus the **4** rows
the merges in §1f actually remove from the active population → **63 active rows**, every one of
which appears in §4's queue or §5's parked table.

Two of §1f's six merge pairs do **not** reduce the count, which is worth stating rather than
hiding in the arithmetic: `OOS-DX5-7` is already subtracted under CLOSED (only its *residual*
rides PB-DX39), and `OOS-DP10-5` is a **pre-cutoff** seed that was never among the 80 in the first
place. The merges collapse queue *entries*, not IDs: all 79 IDs remain individually traceable
through §1b/§1c/§1e/§1f/§4/§5 — so a raw extraction of §4 ∪ §5 returns
**more** than 63 IDs (a merged seed still names both members in its entry), and is not a way to
check the 63. Check it the other way: every one of the 79 is dispositioned exactly once in
§1b/§1c/§1e/§1f/§4/§5, and 80 − 11 − 2 − 4 = 63. The full per-seed verdict, class, severity, live-wrongness, measured scope and wire
prediction is carried in the queue and parked tables rather than repeated here; the ones whose
verification *changed* their rank get a note in §2.

**Live-wrong on a deck-legal `Complete` card today — the top of the ranking, measured:**

| seed | the card(s) | measured population |
|---|---|---|
| `OOS-SIM2-6` | `indomitable_archangel` | 1 def (2 independent measurements, §2.1) |
| `OOS-CARDS2-4` | `hyena_umbra` + 12 more | **13** deck-legal `Complete` Aura defs |
| `OOS-M11-9` | any vigilant attacker | **14** deck-legal `Complete` vigilant creatures |
| `OOS-DX2-5`/`-2`/`-7` | `golgari_grave_troll` | 1 def, permanent draw-cadence corruption |
| `OOS-DX1-3` | `nether_traitor` | 1 def of 3 with `trigger_zone: Some(Graveyard)` |
| `OOS-SIM3-5(c)` | 6 mutate defs × 24 counter defs | 6 `Complete` mutate defs |
| `OOS-CARDS1-3` | `umezawas_jitte`, `sword_of_feast_and_famine`, +8 | **21** Equipment defs (18 under a naive set difference — see §2.7), **10** deck-legal — *exact under every method* |
| `OOS-DX4-6` | the ten Karoo bounce lands, +4 | **≥14** `Complete` defs |
| `OOS-CARDS1-2` | `lizard_blades` | 1 def (the corpus's only Reconfigure) |
| `OOS-M11-10(loyalty)` | `sarkhan_vol`, `teferi_time_raveler`, +2 | **4** of 6 `Complete` planeswalkers |
| `OOS-DX4-5` | `birthing_ritual`, `satyr_wayfinder`, +3 | **5** `Complete` defs (`optional` inert) |
| `OOS-DX4-2` | `retreat_to_kazandu` | 1 of 8 modal-trigger defs |
| `OOS-CARDS2-10` | `qarsi_sadist`, `voldaren_epicure` | **2** `Complete` defs |
| `OOS-CARDS2-6` (rider) | `sigil_of_sleep` | 1 `Complete` def; 12 in the oracle family |
| `OOS-CARDS2-7` | `archetype_of_imagination`, `gingerbrute`, `xenagos_the_reveler`, … | **35** `Complete` defs invisible to the deviation scan |
| `OOS-M11-7` | any sac-for-mana permanent | **22** `Complete` defs (self-healing window) |

### 1e. NOT queue work — design records that a class column will mislead you into ranking

Three rows carry a `correctness` or `design-record` class but describe **decisions already taken
and written down**, not work. Ranking them wastes a dispatch slot.

| seed | why it is not work |
|---|---|
| **OOS-DX5-2** | CR 613.6, a static effect's affected set changing between layers inside one `calculate_characteristics` call. `layers.rs:590-620`'s own doc block says this is the *intended* design (it is what makes Opalescence-before-Humility work). The row itself says "not actionable without redesigning how `chars` flows". No misbehaving card exists. |
| **OOS-DX5-6** | the behaviour it describes is **shipped and CR-correct**, pinned by T15. Its value is the corrected *methodology* ("check which effects of ANY filter write it", not "which mass-filter defs write it"). Keep as a record. |
| **OOS-DX6-3** | a written "we considered `Result` and rejected it, here is why" note. `player.rs:148-215` is exactly what the row describes, rationale in-source. |

### 1f. Duplicates, merges, and the seeds nobody owns

**Merged — one fix closes both, so they are one queue entry:**

| pair | why |
|---|---|
| `OOS-SIM1-2` ≡ `OOS-SIM2-7` | **literally the same two lines** — `tools/tui/src/play/input.rs:168` and `app.rs:283`, the only non-test callers of the pool-blind `solve_mana_payment`. Filed twice, from the tax side and the residual side. |
| `OOS-UI2-5` + `OOS-DX6-5` | the same "the TUI hand-builds a `Command` instead of routing through `params.rs`" defect in the same file, on `CastSpell` and `DeclareAttackers` respectively. `input.rs:616-623` already carries the in-source comment prescribing the remedy. |
| `OOS-UI2-1` + `OOS-SIM3-1` | one defect, one corrected horizon. SIM3-1 does not contradict UI2-1; it dates it. |
| `OOS-CARDS2-11` ⊂ `OOS-CARDS2-8` | CARDS2-11's three headline items (`chord_of_calling`, `green_suns_zenith`, `the_world_tree`) are *stale-blocker-note instances*, which is exactly what CARDS2-8 is. Keep -11 as a pointer row so `cards2-oracle-sweep-2026-08-02.md` stays reachable; do the work under -8. |
| `OOS-DX5-3` + `OOS-DX5-7`'s residual | one mechanism — a source-relative filter answering `false` because `state.objects.get(&source_id)` is `None` — differing only in *when* the source left (before resolution vs sacrificed as a cost). |
| `OOS-DX4-5` + `OOS-DP10-5` | the card-side population and the engine-side gap of one defect. Two IDs, one thing; must close together and must not be counted twice. |

**One seed that is neither closed nor parked, recorded here so it cannot fall through again:**
**`OOS-DX2-3`** is **ACTIVE (reopened)** — PB-DX2 closed it on a structural proof that a re-review
falsified, which is the single most instructive failure in this census's provenance. It is LOW with
**0** corpus reach today, but it lives on the discharge **PB-DX23 edits**, so it rides that batch as
a named watch item rather than being parked. See PB-DX23's brief.

**Two seeds filed by this task, because the finding exists and nothing owns it:**

- **`OOS-RR3-1` — nobody has ever reviewed the population of defs that never declare a
  `completeness` marker, and the `#[default] Completeness::Complete` derive that covers them is
  now a seven-times-demonstrated silent-defect generator.** Measured this task:
  **965 of 1,803 def files never mention `completeness` at all** (`mod.rs` excluded; 1,804 `.rs`
  files in `defs/` including it) — a clear majority of the 1,133-strong `Complete` population.
  PB-DX4 measured 966 on 2026-08-01; one def has gained an explicit marker since, so the ratchet
  is holding but the number is a *snapshot*: re-measure rather than cite it.
  Of the eight live-wrong defs §1d found behind a "latent"-or-stale-premise row (`qarsi_sadist` and
  `voldaren_epicure` come from `OOS-CARDS2-10`, which is stale-premise, not latent), **five** came
  through this door
  (`golgari_grave_troll`, `retreat_to_kazandu`, the ten Karoos, `sigil_of_sleep`,
  `indomitable_archangel`); the other three (`nether_traitor`, `qarsi_sadist`,
  `voldaren_epicure`) declare `Complete` **explicitly** and were simply never checked — so the
  seed is *two* findings, and §2.5 keeps them apart. PB-DX4 already ratcheted the marker count in
  the growth direction; what does not exist is a *review* of the population. Not queued as its own
  batch — it is the standing reason every batch below must enumerate the corpus before calling
  anything latent.
- **`OOS-RR3-2` — the corpus-wide re-check of dated blocker notes has no ID.** `OOS-DX3-1`'s
  closure explicitly names it "a cheap standing sweep" and then closes without filing it. Measured
  surface: **251** defs mention "DSL gap", **77** mention "Blocker", **44** assert a variant "does
  not exist"; the machine-checkable subset (a note naming a concrete `Effect::`/`Condition::`/
  `Cost::`/`TriggerCondition::`/`TargetFilter` identifier on the same line as the gap phrase) is
  **67** defs. Overlaps `OOS-CARDS2-8` heavily; queued together as **PB-DX27**.

**A warning that belongs with a closure, not a seed**: `cards1_equip_target_roster.rs` R1 pins the
equip surface at **exactly 17** defs. That reads as a clean sweep. It is not — `OOS-CARDS1-3`'s
**the further Equipment defs with no equip ability at all** are one link earlier in the same chain
and are a *larger* population. Anyone auditing "is equip done?" by reading the gate gets the wrong
answer. Same shape for `OOS-M11-8`'s closure, whose row names `OOS-M11-2` as "same family" while
`OOS-M11-2`'s layer-resolution half is open.

---

## 2. Chain-verification notes (AC 6032)

Only the seeds whose verification *changed* something get a note. Everything else is verified in
the queue and parked tables, each row of which carries its own `file:line`.

### 2.1 OOS-SIM2-6 — the recursion, walked hop by hop, and the severity argument (AC 6033)

**The cycle. Four hops, all present at HEAD, verified by reading each site rather than by trusting
the filing:**

1. `crates/engine/src/rules/layers.rs:35` `calculate_characteristics` → `:44-46`
   `state.continuous_effects.iter().filter(|e| is_effect_active(state, e))`
2. `crates/engine/src/rules/layers.rs:508` `is_effect_active` → `:565`
   `if !crate::effects::check_static_condition(state, condition, source_id, controller)`
3. `crates/engine/src/effects/mod.rs:10212` `check_static_condition`, the
   `Condition::YouControlNOrMoreWithFilter` arm → `:10259`
   `let chars = crate::rules::layers::expect_characteristics(state, obj.id);`
4. `crates/engine/src/rules/layers.rs:477` `expect_characteristics` → `:478`
   `calculate_characteristics(state, object_id)` → **back to hop 1**.

**Why it is unconditional rather than probabilistic.** Hop 3 iterates the controller's battlefield
permanents and calls `expect_characteristics` on each **before** the `exclude_self` test (`:10259`
vs `:10266`), and the Archangel is itself one of its controller's battlefield permanents. So the
recursion needs no artifact and no second permanent: one Archangel on the battlefield is enough,
because the effect is re-collected at `layers.rs:46` on **every** nested call regardless of which
object is being queried.

**The card.** `crates/card-defs/src/defs/indomitable_archangel.rs:29-43` registers
`AbilityDefinition::Static` with `condition: Some(Condition::YouControlNOrMoreWithFilter { count: 3,
filter: { has_card_type: Some(CardType::Artifact), .. } })`. `grep -c completeness` on the file
returns **0** — `Complete` by the `#[default]` derive. `validate_deck` (`commander.rs:233`) rejects
only on `!def.completeness.is_complete()`, so it accepts. `replacement.rs:2815-2837` copies
`condition` verbatim onto the registered `ContinuousEffect` at ETB.

**No guard exists.** Grep over `layers.rs` + `effects/mod.rs` for `thread_local`, `RefCell`,
`Cell<`, `depth`, `MAX_DEPTH`, `recursion_depth`, `memo`, `cache`: zero hits outside comments.
Workspace-wide `recursion_depth|MAX_RECURSION|MAX_DEPTH` in `crates/engine/src/`: zero. CR 104.4b
loop detection (`state/mod.rs:287`) is a *game-state hash* detector operating **between** commands
and is structurally incapable of interrupting an in-call stack recursion.

**The finding worth carrying up.** A sibling counter 1,800 lines away made the opposite, correct
choice for the same hazard: `layers.rs:2291` `EffectAmount::PermanentCount` uses base
characteristics, with the comment at `:2304-2310` — *"We deliberately use base characteristics here
(not calculate_characteristics) to avoid recursive CDA evaluation."* Meanwhile
`effects/mod.rs:10245-10256` asserts the opposite: *"This is re-entrant but safe… Termination is
guaranteed because we are checking the types of other battlefield objects, not the object currently
being calculated — there is no direct self-referential cycle."* **That comment is the defect.** The
recursion is not on the same *object*; it is on the same *effect*. And the comment even proposes
the fix — *"If performance becomes an issue, consider using base characteristics for the filter
check"* — as a performance note, having mis-diagnosed a crash as a cost. Generalisable:
**a safety argument written next to the code it excuses is not evidence.**

**Why no gate ever fired.** The corpus's one Archangel test,
`crates/engine/tests/rules/static_grants.rs:711-760`, hand-builds the `ContinuousEffect` with
`condition: None` at `:736` instead of registering the def through ETB. It exercises the filter and
never the condition. **A test that names the card while dodging the field is worse than no test —
it reads as coverage.**

**The class is exactly one, measured two ways.** (i) Only `ContinuousEffect.condition` reaches
`is_effect_active`; of **380** `ContinuousEffectDef` literals, exactly
**17** carry `condition: Some(..)`, and exactly **one** of those uses a recursion-capable variant.
The other 16 use `SourceHasCounters`, `DevotionToColorsLessThan`, `OpponentLifeAtMost`,
`SourceIsUntapped`, `CompletedADungeon`, `IsYourTurn`, `ControllerLifeAtLeast`,
`YouControlYourCommander` — all base-state reads. All four engine-synthesised `ContinuousEffectDef`s
in `state/builder.rs` carry `condition: None`. (ii) **11** `Condition` variants are recursion-capable
(their `check_static_condition` / `check_condition` arm calls `expect_/calculate_characteristics`);
74 further def files mention one, but every one is in an `AbilityDefinition::Replacement`, an
`activation_condition`, an `intervening_if` or an `Effect::Conditional` — none of which route
through `is_effect_active`.

**The landmine.** `crates/card-defs/src/defs/greymond_avacyns_stalwart.rs:38-43` is
`Completeness::inert` with a note **instructing a future author to build exactly this**: *"The +2/+2
conditional static IS now expressible (`Condition::YouControlNOrMoreWithFilter` +
`ContinuousEffectDef.condition`) and should be wired."* Wiring it as written adds a second crash
card. The class is 1 today and the corpus contains written instructions to make it 2.

**Severity argument (AC 6033) — and it IS the first dispatch.**

- **Crash, not silent-wrong.** `fatal runtime error: stack overflow` → SIGABRT. Not
  `catch_unwind`-able, so the play-server's request boundary cannot contain it: the *process* dies,
  taking every seat's game with it. Still overflows at `ulimit -s 524288`.
- **Fires immediately.** `rules/sba.rs:188` calls `calculate_characteristics` on every SBA sweep,
  and `check_and_apply_sbas` is invoked from ≥9 sites in `rules/engine.rs`;
  `crates/view-model/src/lib.rs:452` calls it for every object in every seat view. The abort lands
  on the first SBA check after the Archangel resolves, before any view is returned.
- **Reachable from the shipped browser game.** `setup.rs:230` builds each seat's deck via
  `deck::random_deck` over `all_cards()` and `:250` admits it through the real `validate_deck`.
  The Archangel is mono-white and `Complete`, so it is in the pool for any W-identity commander.
- **Not a regression — 4.5 months old.** Def `d83ac94d` (2026-03-12); recursive call site
  `aa23d26c` (2026-03-23). Nothing in the 174-181 run caused it; SIM-2 found it.
- **Cheapest HIGH in the inventory.** One line, one card, zero wire movement, and the correct
  precedent is already in the tree.

**Ranked first.** Maximum severity, minimum blast radius, minimum fix cost — there is no
argument for anything preceding it. **And it is a hard sequencing prerequisite for PB-DX22**: the
moment the fuzzer shuffles (and therefore starts casting spells before turn ~140), the Archangel
becomes reachable in fuzz runs at ordinary depths. Shipping PB-DX22 first would turn a rare
turn-191 abort into a common one.

**On whether it is the mechanism behind `OOS-M11-3` / `OOS-DP3-9`'s stack-overflow half:
plausible, and the depth-dependence is evidence for it — but it is not proven, and this document
does not claim it is.** The apparent contradiction (`OOS-UI2-1` says the fuzzer never casts a
spell, so how does the Archangel resolve?) reconciles arithmetically: UI-2's measurements are all
at `--max-turns 80`, the deck's fixed order puts the first non-land at personal draw ~35-40, and
both `OOS-DP3-9`'s overflow repro and `OOS-SIM2-6`'s are in the `--max-turns 200` / turn-191
regime. **The decisive experiment is cheap and belongs in PB-DX19's plan**: run
`mtg-fuzzer --games 15 --seed 1` twice, once as-is and once with the Archangel's static commented
out. If the overflow disappears, `OOS-DP3-9`'s stack-overflow half merges into `OOS-SIM2-6`.
Until that A/B runs, "very likely the mechanism" is the correct strength.

### 2.2 OOS-CARDS2-4 — every Aura is unplayable in the browser, and the offer layer cannot see why

> **CLOSED by PB-DX20 (`scutemob-198`, 2026-08-04).** This section's three-end diagnosis was
> correct and its **prescription was one layer off**: it proposed synthesising the requirement in
> `crates/simulator/src/legal_actions.rs`, but that file is not on the browser's path — the offer
> reads `mtg_engine::spell_target_requirements` (`rules/queries.rs`), which is where the synthesis
> landed. `legal_actions.rs` took **zero** lines and `tools/play-server` took zero production
> lines. Kept unedited below as the record of the diagnosis.

Verified both ends personally. **Engine end**: `crates/engine/src/rules/casting.rs:3723-3733` — if
the card is an Aura enchantment and `sba::get_enchant_target(&chars.keywords)` yields a restriction,
then `if spell_targets.is_empty() { return Err(... "Aura spells require exactly one target
(CR 303.4a)") }`. **Provider end**: `crates/simulator/src/legal_actions.rs` contains **zero**
occurrences of `Enchant(`, `get_enchant_target` or `target_min`. **Offer end**:
`tools/play-server/src/view.rs:1921-1922` computes `(target_min, target_max)` from
`mtg_engine::target_count_range(&requirements)`, and `casting.rs:5898-5911` shows that function
iterating `&[TargetRequirement]` only — **a keyword is not a `TargetRequirement`, so the sum is 0**.
The client then renders a zero-target action (`ActionBar.svelte:266`), the human clicks, and the
engine 422s.

The workaround is in the *test driver*, not the product: `tools/play-server/src/main.rs:1735-1740`
`KNOWN_FALSE_OFFERS = ["Aura spells require exactly one target"]`, with the seed cited in the
comment above it. That is an honest record and the right thing to have written — but it is a
suppression, and it is the only reason the play-server driver is green.

**Measured population: 13 deck-legal `Complete` Aura defs** — intersect (defs declaring the `"Aura"`
subtype: 27) ∩ (defs with `KeywordAbility::Enchant(`: 23) = 23, ∩ `Complete` = 13:
`awaken_the_ancient`, `chained_to_the_rocks`, `darksteel_mutation`, `dimensional_exile`,
`eaten_by_piranhas`, `hyena_umbra`, `imprisoned_in_the_moon`, `kasminas_transmutation`,
`kenriths_transformation`, `ossification`, `rancor`, `sigil_of_sleep`, `wild_growth`.

**Adjacent, previously unfiled**: 4 Aura defs declare *no* `Enchant` keyword at all
(`animate_dead`, `curse_of_opulence`, `open_the_armory`, `sram_senior_edificer`). Two are `inert`,
so they are deck-excluded — but the shape means `casting.rs`'s Aura gate would be **skipped
entirely** for such a def, which is a second failure mode of the same seam and is worth a probe in
PB-DX20's plan.

**This is the same shape as `OOS-M11-10(equip)` (closed) and `OOS-CARDS1-2` (open), one link
earlier**: not "the picker never asks" but "the requirement is written somewhere the picker does
not look." Generalisable, and worth stating once for the queue: **an engine special-case that
derives a requirement from anything other than the `TargetRequirement` list is invisible to the
offer layer by construction, and SR-38 cannot see it either.**

### 2.3 OOS-M11-9 — the guard is absent, and the consequences are three, not one

Verified personally. `crates/engine/src/rules/combat.rs:41-75` — `handle_declare_attackers`'s guard
set is exactly four checks (step, active player, priority holder, then per-attacker legality), and
nothing consults prior declaration state. `:69-71` initialises `CombatState` only
`if state.combat.is_none()`, so a second declaration reuses the existing combat. **The blocker side
has the guard the attacker side lacks**: `combat.rs:1103` — `if combat.defenders_declared.contains(
&player) { return Err(GameStateError::AlreadyDeclaredBlockers(player)) }`. So the seed's own
"verify whether CR 509.1 is already covered" resolves **covered**; only the attacker half is open.

**The row says a re-declaration "overwrites `combat.attackers`". It does not, and the three real
consequences are each worse than that:**

1. `combat.rs:743-745` does `combat.attackers.insert(*attacker_id, target.clone())` — an insert
   into a map. Declarations **accumulate**; only a repeated same-id entry overwrites, and it
   overwrites that creature's **attack target mid-combat**.
2. `combat.rs:795-805` pushes a fresh `GameEvent::AttackersDeclared` and immediately runs
   `abilities::check_triggers` + `flush_pending_triggers`, so **every "whenever this creature
   attacks" trigger re-fires on each re-declaration**.
3. `combat.rs:759` assigns `ps.attackers_declared_this_turn = attackers.len() as u32`, so a
   1-attacker re-declaration **clobbers the raid count** that `effects/mod.rs:10032` reads for
   `Condition::YouAttackedWithNOrMore`.

**Reachability is higher than at filing.** SIM-1 made a vigilant commander castable from the
command zone; there are **14** deck-legal `Complete` vigilant creatures in the corpus. A human in
the browser reaches all three consequences by clicking attack twice. The existing mitigations are
**client-side only** and both say so: `heuristic_bot.rs:49` (`RepeatKey::DeclareAttackers => cap
1`, explicitly "a preference cap, not a legality cap") and
`crates/simulator/tests/local_game_playthrough.rs:127/138/151/276/405`. There is no cap on the
play-server path. **Ranked HIGH**, against the registry's implicit medium.

### 2.4 The fuzzer trio — one defect, one horizon, and an open measurement

`OOS-UI2-1`'s chain at HEAD, five links: `deck.rs:90-148` builds `main_deck` in fixed structural
order (60 non-lands, ≤5 non-basic lands, then basics to 99 — **basics last**) →
`bin/fuzzer.rs:331-339` loads them straight into `ZoneId::Library` with **no `shuffle` anywhere in
the file** → `state/builder.rs:380`/`state/mod.rs:1129-1155` preserve insertion order →
`zone.rs:109` `Zone::Ordered(v) => v.push_back(id)` → `zone.rs:159-161` `top()` is `v.last()`.
Contrast `setup.rs:280`, which *does* shuffle — the `LocalGame` path is fine; only the fuzzer is
blind.

`OOS-SIM3-1` does not contradict it; it dates it. `engine.rs:3485-3500` deals **no opening hand**,
so with 34 basics + ≤5 non-basic lands on top, the first non-land is personal draw ~35-40 = game
turn ≈136-156 in a 4-player game. SIM-3's measured earliest cast at **turn 143** sits inside that
band. **"Never" is `--max-turns 80`; "from ~turn 143" is the default cap.** Every fuzz-parity claim
in this project's history must now be read against the `--max-turns` it ran at.

`OOS-SIM1-4` is the third leg: `fuzzer.rs:322-327` places the commander card
`.in_zone(ZoneId::Command(*pid))` but **never calls `builder.player_commander`** (the only
production registrar is `setup.rs:276`), so `commander_ids` is empty and CR 903.8 tax, CR 903.9a
zone return and CR 903.10a commander damage have **never been fuzzed**.
`crates/simulator/tests/local_game.rs:78` repeats the defect.

> **SETTLED by PB-DX22 (`scutemob-196`, merge `95f53b78`, 2026-08-03)**: the offer was
> SUPPRESSED — `commander_ids` empty in every fuzzer game (0/4, ~57k commands, zero
> command-zone casts), so OOS-SIM1-4 was the cause, not an independent gap.

**One open measurement this task could not settle read-only, and it changes PB-DX22's sizing.**
SIM-1 added a command-zone cast loop (`legal_actions.rs:675-693`) and a commander is *not* in the
library, so the no-shuffle defect does not gate it — a bot should be able to cast its commander
around game turn 12-24, more than a hundred turns before SIM-3's measured 143. Either SIM-3
measured a pre-SIM-1 build, or something suppresses that offer for bots. **PB-DX22's plan must
re-measure the first `ZoneId::Stack` arrival at HEAD before ranking its own scope.** One
instrumented `mtg-fuzzer --games 5 --seed 1` settles it.

### 2.5 Why "latent" is not a verdict this project can trust — and the derive is only half the story

Four rows in this census are filed "latent" on the stated ground that no corpus def pairs the two
conditions the seed names. **All four are false.** The defs are:

- `OOS-DX1-3` → `nether_traitor` (`trigger_zone: Some(Graveyard)` + `WheneverCreatureDies`)
- `OOS-DX2-2`/`-5`/`-7` → `golgari_grave_troll` (`KeywordAbility::Dredge`)
- `OOS-DX4-2` → `retreat_to_kazandu` (modal triggered ability with flat `targets`)
- `OOS-DX4-6` → the ten Karoo bounce lands (untargeted printed choice as a `TargetRequirement`)

**The obvious explanation is wrong, and the true one is worse.** The tempting story — inherited
from PB-DX1's `aurelia_the_warleader` and PB-DX3b's `emeria_the_sky_ruin` — is that the def is
`Complete` by the `#[default]` derive and therefore invisible to a grep for
`Completeness::Complete`. That holds for **five** of the eight defs this census caught
(`golgari_grave_troll`, `retreat_to_kazandu`, the ten Karoos, `sigil_of_sleep`,
`indomitable_archangel` — each verified by `grep -c completeness` returning 0). It does **not**
hold for the other three: `nether_traitor.rs:60`, `qarsi_sadist` and `voldaren_epicure` each
declare `completeness: Completeness::Complete` **explicitly**. A one-line grep would have found
them.

So the shared mechanism is not the derive. It is that **the latency claim was never checked against
the corpus at all** — in three of eight cases not even by the cheapest possible check. The derive
is the most common *reason a check would have failed had it been run*; it is not the reason the
check was skipped. Stating it as "the derive did it" would let a future triage think that
grepping for the explicit marker is sufficient diligence, and for three of these eight defs it
would have been.

**965 of 1,803 def files never mention `completeness`** — re-measured this task; PB-DX4 measured
966 the day before, so the ratchet holds but the figure is a snapshot. Filed as `OOS-RR3-1`.

**Binding instruction for every batch in §4**: *a latency claim is not verified until the corpus has
been enumerated — over `all_cards()` where possible (SR-36), and with a missing `completeness`
field treated as `Complete` — and "no def does X" is not a finding until someone has actually
looked.*

### 2.6 "An enumeration is only as complete as the category it names" — now three instances

SIM-1 recorded this lesson (`OOS-SIM1-3`: `GameRestriction` is not the only cast gate — split
second is unmirrored). This census found two more of the identical shape, which promotes it from
an anecdote to a standing check:

- **`OOS-CARDS2-7`** — `completeness_deviation_scan`'s needle set
  (`crates/engine/tests/core/completeness_deviation_scan.rs:48-54`) is five phrases the *gate
  author* thought of, not the phrases the *corpus uses*. Measured: **35 `Complete` defs** contain
  `DSL gap` / `deferred` / `not expressible` / `TODO` / `cannot be expressed` / `unsupported` and
  redden nothing (`DSL gap` 12, `deferred` 12, `TODO` 11, …). The gate whose entire job is catching
  this is measurably missing a third of the vocabulary.
- **`OOS-SIM3-2`** — `crates/simulator/src/invariants.rs:26-43` `check_all` calls **10** functions,
  one of which (`check_mana_non_negative`, `:100-103`) is a no-op with `_`-prefixed params, so
  **9 can fire**. `docs/mtg-engine-simulator.md:221-256` documents 12; `runtime-integrity.md:55-84`
  documents 9 of which four exist nowhere. The two never written are **legal-action soundness**
  (which *is* SR-38, the property F4/F7/F9 and every play-server 422 are instances of) and **SBA
  idempotency**.

Both are ranked as gate integrity in §4. The generalisation belongs in `memory/conventions.md`:
**a gate that enumerates its own targets is only as complete as the category its author named;
derive the category from the thing being checked, not from the checker.**

### 2.7 `OOS-CARDS1-3`'s roster is method-sensitive — measure it at dispatch, not from this memo

The seed says **21** Equipment defs print `Equip {N}` and have no equip ability. Re-measured, the
total is **18 or 21** depending on one rule you have to choose before you count, and the choice is
not obvious.

Raw greps over `crates/card-defs/src/defs/`: **21** files contain `KeywordAbility::Equip`, **22**
contain `AttachEquipment`, and the two sets overlap in **3** files — `blackblade_reforged`,
`blade_of_the_bloodchief`, `sword_of_body_and_mind`. So a naive set difference gives 21 − 3 =
**18**. But in **all three** overlap files the only `AttachEquipment` match is inside the def's own
`Completeness::partial("…")` blocker string, describing the ability it *lacks* — e.g.
`sword_of_body_and_mind.rs:97-99`, `"… with no AbilityDefinition::Activated { cost: Mana({2}),
effect: AttachEquipment …"`. Discount `Completeness::` strings and the overlap is **0**, giving
**21** — the seed's figure. The two numbers are the same measurement under two readings of "has an
equip ability", and the seed's reading is the correct one.

**The deck-legal `Complete` subset is 10 under every method** — `bone_saw`, `kite_shield`,
`paradise_mantle`, `sword_of_feast_and_famine`, `sword_of_light_and_shadow`,
`sword_of_sinew_and_steel`, `sword_of_truth_and_justice`, `sword_of_war_and_peace`,
`the_reaver_cleaver`, `umezawas_jitte` — and that is the number the rank rests on. **Rank on the
10; re-derive the total from `all_cards()` in the plan**, not by grep, and note in passing that
`cards1_equip_target_roster.rs` / `cards1_equip_target_repair.rs:541` match
`Effect::AttachEquipment` **non-recursively**, so a def nesting the attach inside an
`Effect::Sequence` drops out of the exact-17 pin silently.

Generalisable, and it is the third time this census has hit it: **a roster derived by grep is a
measurement of the grep, not of the corpus.** SR-36 already says "enumerate `all_cards()` for
rosters, never grep source"; this memo's own counts are greps and are labelled as such throughout.

---

## 3. Disposition of the v2 queue, PB-DX7..PB-DX18 (AC 6034)

All twelve standing entries were re-checked against this census. **Ten survive unchanged, one is
widened, one is widened and promoted.** None is closed or retired.

> The `rank` values below are **copies of §4's rank column**, which is authoritative. If they ever
> disagree, §4 wins — and the disagreement is a bug in this table, not a second opinion.

| v2 entry | v2 seeds | disposition | why |
|---|---|---|---|
| **PB-DX7** | `OOS-DP7-11` + `OOS-DP9-13` (+DP10-1, DP9-10 residual) | **SURVIVES, rank 9** (was "next dispatch") | Still real, still test-only, still 0 flips. It is displaced by eight items that are live-wrong on deck-legal `Complete` cards or are a hard crash. Gate integrity ranks below live correctness by the standing convention — v2 applied the same rule. |
| **PB-DX8** | `OOS-DP10-9` | **SURVIVES and is WIDENED** — fold in `OOS-CARDS2-7`, rank 10 | Same instrument, same failure mode, one link apart: DP10-9 is "the DSL never encoded the choice"; CARDS2-7 is "the deviation scan's needles miss the corpus's own vocabulary". Both are `oracle_text`-vs-source cross-checks over `all_cards()`. Doing them separately builds the same scanner twice. CARDS2-7 brings a measured 35-def worklist. |
| **PB-DX9** | `OOS-DP9-3` (+DP9-2/-4/-9, DP10-5) | SURVIVES unchanged, rank 15 | Note: **`OOS-DX4-5` is `OOS-DP10-5`'s card-side population** (5 `Complete` defs with an inert `optional`). It must ride this batch or **PB-DX35**, not both. |
| **PB-DX10** | `OOS-DP3-4` + `OOS-DP8-7` (+DP8-3) | SURVIVES unchanged, rank 16 | Untouched by the new census. |
| **PB-DX11** | `OOS-DP5-6` (+DP5-8, DP5-9) | SURVIVES unchanged, rank 24 | Untouched. Note that `grep ReplacementTrigger::WouldDraw crates/card-defs/src/defs/` returns **0** — the yield claim (2 flips of 3 inert defs) is a *capability* claim, not a repair claim. |
| **PB-DX12** | `OOS-OS6-1` | SURVIVES unchanged, rank 25 | Untouched. |
| **PB-DX13** | `OOS-OS7-1 R1` + `OOS-RS-5` | SURVIVES unchanged, rank 26 | Untouched. |
| **PB-DX14** | `OOS-OS4-1` (+OOS-RS4-3) | SURVIVES unchanged, rank 28 | Untouched. |
| **PB-DX15** | `OOS-DP9-11` + `-16` + `-8` | SURVIVES unchanged, rank 29 | Untouched. |
| **PB-DX16** | `OOS-OS4-3` | SURVIVES unchanged, rank 31 | Untouched. |
| **PB-DX17** | `OOS-OS7-1 R2+R3` | SURVIVES unchanged, rank 32 | Untouched. |
| **PB-DX18** | `OOS-DP2-7` + `-4` + `-8` | SURVIVES, rank 14 — **promoted** | `OOS-DX2-4` (mulligan commands have no pregame gate) and `OOS-DX2-1` (miracle is not gated on the offer) are the same trust-boundary class on the same command surface. Merged into PB-DX18's scope with `OOS-M11-5`; see the queue row. |

**v2's "designated successor" (`proliferate`, **23** `Complete` defs — v2 said 25 — on PB-DP9's `AnswerEffectChoice`
channel) is unchanged and still unranked** — it is agency restoration on otherwise-correct cards,
and eight live-wrong items now sit above it. Carried forward verbatim.

**v2's §4 "Ordering rule" and "Sequencing notes" paragraphs are inherited unchanged**: one wire
bump per PB (AC-5040 discipline), gate-compute both fingerprints rather than predicting them, and
treat a mismatch as a signal to stop.

---

## 4. The merged queue — **PB-DX7 .. PB-DX41** (AC 6034)

> ### 🚦 READ THIS BEFORE CLAIMING ANYTHING
>
> **The PB-DX number is a stable label, NOT a rank.** PB-DX1..DX6 shipped. PB-DX7..DX18 keep their
> numbers **and their scopes** from v2 so that every existing cite still resolves; new batches are
> numbered **PB-DX19..PB-DX41**. The table below is ordered by **rank**, and the rank column is the
> only thing a dispatcher should read. Renumbering the survivors was considered and rejected: this
> queue's whole reason for existing is the N4 re-dispatch hazard, and silently re-pointing
> "PB-DX7" at different work is exactly that hazard with a fresh coat of paint.
>
> **The next dispatch is `PB-DX19`, not `PB-DX7`.** CLAUDE.md's Current State said "Next dispatch:
> PB-DX7" until this task; it is repointed in §6. If you are reading a stale pointer, this banner
> is the correction.
>
> **Two hard sequencing constraints, both derived rather than asserted:**
> 1. **PB-DX19 must precede PB-DX22.** Shuffling the fuzzer makes spells castable at ordinary
>    depths, which makes `indomitable_archangel`'s stack overflow common rather than rare (§2.1).
> 2. **PB-DX22 re-rolls every recorded seed** (`OOS-UI2-1` and `OOS-SIM1-4` both do), and so does
>    **any card-def batch** via `OOS-CARDS2-3`'s corpus→seed coupling. Batch the card-def work
>    (PB-DX26, PB-DX27) so the re-deal is paid once, and land `OOS-CARDS2-3`'s pool-size gate
>    *before* them so the re-deal announces itself.

| rank | batch | scope | seeds | class | discounted yield | wire |
|---|---|---|---|---|---|---|
| **1** | ~~**PB-DX19**~~ ✅ SHIPPED (`scutemob-184`, `451e3517`, 2026-08-02) | the unbounded characteristics recursion + unchecked P/T arithmetic | **OOS-SIM2-6** (HIGH) + **OOS-SIM2-5** | **CORRECTNESS — hard process abort, deck-legal** | 0 flips; closes the only HIGH in the registry; 10 arithmetic sites hardened | **none** (both fixes are arithmetic/read-site) |
| **2** | **PB-DX20** | the offer layer cannot see a keyword-carried target requirement | **OOS-CARDS2-4** (HIGH) + **OOS-CARDS1-2** | **CORRECTNESS — live in the browser on first contact** | 0 flips; repairs 13 `Complete` Auras + 1 `Complete` Reconfigure | **none** (provider + one synth site) |
| **3** | **PB-DX21** | CR 508.1 — attackers may be declared without limit | **OOS-M11-9** | **CORRECTNESS — silent state corruption by a normal client action** | 0 flips; 14 `Complete` vigilant creatures; deletes 2 client-side mitigations | **none** if the guard reads `combat.attackers`; **HASH** if it mirrors `defenders_declared` |
| **4** | ~~**PB-DX22**~~ ✅ SHIPPED (`scutemob-196`, `95f53b78`, 2026-08-03) | make the fuzzer a real instrument | **OOS-UI2-1** + **OOS-SIM3-1** + **OOS-SIM1-4** | **EVIDENCE INTEGRITY — every historical fuzz-parity claim depends on it** | 0 flips; re-rolls every recorded seed **once** | **none** (`crates/simulator` only) |
| **5** | **PB-DX23** | dredge has no answer channel for anyone | **OOS-DX2-5** + **OOS-DX2-2** + **OOS-DX2-7** + **OOS-DX2-3** *(watch item)* | **CORRECTNESS — permanent draw-cadence corruption, deck-legal** | 0 flips; 1 def (`golgari_grave_troll`); adds a `LegalAction` variant | **none** (`Command::ChooseDredge` and the event already exist) |
| **6** | **PB-DX24** | the lowering drops `trigger_zone`; the two index spaces disagree | **OOS-DX1-3** + **OOS-DX1-4** | **CORRECTNESS — live-wrong on `nether_traitor`** | 0 flips; 1 live def + 6 latent queue sites aligned | **none** for the narrow fix; **HASH** only if `TriggeredAbilityDef` grows the field |
| **7** | **PB-DX25** | `Effect::CounterSpell`'s three stack-object shapes | **OOS-SIM3-5** | **CORRECTNESS — a countered spell resolves anyway, silently** | 0 flips; 6 `Complete` mutate defs × 24 counter defs | **none** (one arm's internals) |
| **8** | **PB-DX26** | the equip surface, one link earlier | **OOS-CARDS1-3** + **OOS-CARDS1-1** + **OOS-DX3b-1** | **CORRECTNESS + CARD YIELD** | **~4-6 flips** (**10** deck-legal defs gain their printed ability — exact; the batch's *total* is 21, or 18 under a naive set difference — re-measure from `all_cards()` at dispatch, §2.7) | **none** (card-def; or a `keyword_registry` promotion) |
| **9** | **PB-DX7** *(standing)* | SR-19 gate holes | **OOS-DP7-11** + **OOS-DP9-13** (+DP10-1, DP9-10 residual) | gate integrity | 0 flips; 5 structs + all hashed enums re-enter the gate | **none** (test-only) |
| **10** | **PB-DX8** *(standing, widened)* | oracle-text-vs-DSL cross-check | **OOS-DP10-9** + **OOS-CARDS2-7** | **gate integrity — the worst blind spot, now measured** | 0 flips; makes dropped "may"/"choose" clauses visible, and 35 `Complete` gap-noted defs | **none** (test-only) |
| **11** | **PB-DX27** | the stale-blocker-note sweep + the wrong-oracle register | **OOS-CARDS2-8** + **OOS-CARDS2-11** + **OOS-CARDS2-10** + **OOS-RR3-2** | **CARD YIELD + CORRECTNESS** | **~4-8 flips** (67 machine-checkable notes; 2 live-wrong `Complete` defs repaired) | **none** (card-def) |
| **12** | **PB-DX28** | the untargeted-choice class + the owner axis | **OOS-DX4-6** + **OOS-DX4-1** | **CORRECTNESS — exploitable, ≥14 `Complete` defs incl. 10 Karoos** | 0 flips; repairs ≥14 already-`Complete` cards | **both** (a new `EffectTarget` variant is inside `Effect`; `TargetFilter` gaining a field has PROTOCOL v23 / HASH v60 precedent) |
| **13** | **PB-DX29** | the `params.rs` allowlist and the cost-kind surface | **OOS-M11-10(loyalty)** + **OOS-UI2-4** | AGENCY — 4 `Complete` planeswalkers and 13 of 15 cost kinds unusable by a human | 0 flips | **none** (the `Command` fields already exist) |
| **14** | **PB-DX18** *(standing, widened)* | the trust boundary on ungated commands | **OOS-DP2-7** + **OOS-DP2-4** + **OOS-DP2-8** + **OOS-DX2-4** + **OOS-DX2-1** + **OOS-M11-5** | correctness (gated) + hygiene | 0 flips; 3 un-gated commands gated, 2 phantom `LibraryShuffled` sites fixed, PRNG pinned | **HASH** (a pregame-phase flag and a just-drawn-object record are stored state) |
| **15** | **PB-DX9** *(standing)* | multi-card search + the inert-field family | **OOS-DP9-3** (+DP9-2/-4/-9, **DP10-5 ≡ OOS-DX4-5**) | capability / card yield | **2 flips** (`tooth_and_nail`, `buried_alive`) — not 7, per v2 §2.5 | **PROTOCOL + HASH** |
| **16** | **PB-DX10** *(standing)* | PB-DP8b — modal triggered abilities | **OOS-DP3-4** + **OOS-DP8-7** (+DP8-3) | agency / CR 700.2b | 0-1 flips; restores the choice on 4 `Complete` defs | **PROTOCOL + HASH** |
| **17** | **PB-DX30** | CR 704.3 — SBAs are not checked on a priority pass | **OOS-M11-7** | correctness (self-healing window) | 0 flips; 22 `Complete` sac-for-mana defs | **none** (existing events at new times; golden scripts and SR-9b fingerprints will move) |
| **18** | **PB-DX31** | the mana solver's model | **OOS-SIM2-1** + **OOS-SIM2-2** + **OOS-SIM2-3** + **OOS-SIM2-4** | capability — bot play strength on the shipped surface | 0 flips; 36 multi-mana + 20 mana-component + 9 scaled sources become plannable | **none** |
| **19** | ~~**PB-DX32**~~ ✅ SHIPPED (`scutemob-197`, `685aa1c4`, 2026-08-03; promoted + dispatched per feedback doc §2.3) | make the fuzzer's *output* mean something | **OOS-SIM3-2** + **OOS-SIM3-3** + **OOS-SIM3-4** + **OOS-CARDS2-3** | **gate integrity** — dedupe the checkpoint weighting, write SR-38's own invariant, gate the corpus→seed coupling | 0 flips | **none** |
| **20** | **PB-DX33** | route the TUI through `params.rs` | **OOS-SIM1-2 ≡ OOS-SIM2-7** + **OOS-UI2-5** + **OOS-DX6-5** | correctness (TUI-only, latent) | 0 flips; 5 hand-built command sites migrated | **none** (routing) |
| **21** | **PB-DX34** | `Command::DeclareAttackers` — the X channel and the box | **OOS-DX6-1** + **OOS-DX6-4** + **OOS-DX6-2** | correctness (latent) + refactor debt | 0 flips; unblocks Norn's Annex authoring | **PROTOCOL** (one bump pays for the field *and* the boxing; 337 sites) |
| **22** | **PB-DX35** | modal trigger targets + the inert `optional` | **OOS-DX4-2** + **OOS-DX4-5** | correctness + agency | 0 flips; 1 live-wrong `Complete` + 5 `Complete` defs regain a printed choice | **none** for DX4-2; DX4-5 depends on whether a costless "may" gets a real channel (the DP-12 gap) |
| **23** | **PB-DX36** | `WhenDealsDamage` + the dead `combat_only: false` arm | **OOS-CARDS2-6** (both halves) | **CORRECTNESS** — `sigil_of_sleep` is `Complete` and silently drops its trigger; closes `TODO(PB-37)` | **1-2 flips** (`exalted_angel` + family) | **both** (new `TriggerCondition` + `EffectAmount` variants) |
| **24** | **PB-DX11** *(standing)* | `WouldDraw` widening | **OOS-DP5-6** (+DP5-8, DP5-9) | capability | **2 flips** of 3 inert defs | **PROTOCOL + HASH** |
| **25** | **PB-DX12** *(standing)* | multi-count sacrifice cost | **OOS-OS6-1** | capability | **3 flips** of 4 named | **PROTOCOL + HASH** |
| **26** | **PB-DX13** *(standing)* | target-scoped filters | **OOS-OS7-1 R1** + **OOS-RS-5** | correctness + capability | **2 flips** of 3 named | **PROTOCOL + HASH** |
| **27** | **PB-DX37** | the `affected_set` discriminator, all four sites | **OOS-DX5-1** + **OOS-DX5-8** | gate integrity (latent) | 0 flips; 13 creation sites behind one constructor, 4 read sites taught | **none** |
| **28** | **PB-DX14** *(standing)* | back-face starting loyalty | **OOS-OS4-1** (+OOS-RS4-3) | capability | **2 flips** | **PROTOCOL + HASH** |
| **29** | **PB-DX15** *(standing)* | CR 400.7 / APNAP / delayed-trigger sweeps | **OOS-DP9-11** + **OOS-DP9-16** + **OOS-DP9-8** | correctness, sweeps | 0 flips; 3 engine-wide classes | **none** expected |
| **30** | **PB-DX38** | the CR-citation rot sweep | **OOS-UI3-1** + **OOS-DX2-6** | doc hygiene (Architecture Invariant 8) | 0 flips; 9 wrong cites + 74 CR-726 occurrences across 25 files | **none** |
| **31** | **PB-DX16** *(standing)* | edgar return-transformed | **OOS-OS4-3** | capability, micro | **1 flip**, oracle-gated | **PROTOCOL + HASH** |
| **32** | **PB-DX17** *(standing)* | attacked-player trigger family | **OOS-OS7-1 R2+R3** | capability | **1 new card** (`karazikar`) | none |
| **33** | **PB-DX39** | source-relative filters through LKI | **OOS-DX5-3** + **OOS-DX5-7**'s residual | correctness, narrow | 0 flips; `umezawas_jitte` (`Complete`) + `mardu_ascendancy` (`partial`) | **HASH** if a snapshot must be stored; none if derivable at resolution |
| **34** | **PB-DX40** | the two micro card-authoring items | **OOS-DX4-3** + **OOS-DX4-4** | capability, micro | **+2 defs** (one Decayed creature, `wastes.rs`); deletes a simulator special case | **none** (but both move every def-count pin — batch them) |
| **35** | **PB-DX41** | the SR-38 residue the enumeration missed | **OOS-SIM1-3** + **OOS-SIM1-1** | correctness (narrow, safe-failing) | 0 flips; 2 `GameRestriction` variants + split second mirrored | **PROTOCOL** for SIM1-1 (a payment channel on `CastSpell`) — **split it out** unless it rides PB-DX34's bump |

### Ordering rule (inherited from v2, unchanged and still binding)

Compute both fingerprints from the gate's own output; never predict them. A prediction that
disagrees with the gate is a signal to **stop and re-read**, not to edit the pin. One wire bump per
PB. Any row above predicting a HASH bump on a type reachable from `Characteristics` should be
assumed to be a PROTOCOL bump too.

### Dispatch briefs — **full text for PB-DX19..DX23; the rest are table-only, deliberately**

Same reasoning as v2: a brief written now for a batch dispatched four PBs from now is a stale
premise waiting to happen, which is the failure this document exists to catch. Each later rank
carries scope, class, discounted yield and a wire prediction in the table, and its seeds are fully
specified in their filing rows — `docs/audits/decision-point-audit.md` §8.1 for everything except
the CARDS-2 family, which is in `memory/card-authoring/cards2-field-fidelity-2026-08-02.md` §5 —
with this task's corrections in §1c and §2. **Write the brief at dispatch time from those sources,
and re-verify the premise first.**

---

**PB-DX19 — `PB-DX19: the unbounded characteristics recursion (OOS-SIM2-6 + OOS-SIM2-5)` · CORRECTNESS**

`calculate_characteristics` (`crates/engine/src/rules/layers.rs:35`) collects active effects at
`:46` through `is_effect_active` (`:508`), which at `:565` calls `check_static_condition`
(`crates/engine/src/effects/mod.rs:10212`), whose `Condition::YouControlNOrMoreWithFilter` arm at
`:10259` calls `expect_characteristics` (`layers.rs:477`), which calls `calculate_characteristics`.
**Four hops, no guard, and the recursion is unconditional** because the arm evaluates
`expect_characteristics` for every candidate permanent *before* the `exclude_self` test at `:10266`
and the source is itself a candidate. `indomitable_archangel.rs:29-43` registers exactly this
condition and declares **no `completeness` field**, so it is `Complete`, `validate_deck` accepts it,
and `random_deck` puts it in any W-identity seat's pool. The result is `fatal runtime error: stack
overflow` → SIGABRT — not `catch_unwind`-able, so the play-server's request boundary cannot contain
it and the whole 4-player game dies.

**The fix is one line**: read `&obj.characteristics` at `effects/mod.rs:10259` instead of calling
`expect_characteristics` — `matches_filter` already takes `&Characteristics`
(`effects/mod.rs:9533`), so it is type-compatible. **The precedent is in the tree and made the
opposite choice for the same hazard**: `layers.rs:2291` `EffectAmount::PermanentCount` uses base
characteristics with the comment at `:2304-2310` naming recursive CDA evaluation as the reason. The
plan must decide explicitly between that cheap fix and the CR-honest one — CR 613.8b already
supplies the termination rule ("if several dependent effects form a dependency loop, then this rule
is ignored and the effects in the dependency loop are applied in timestamp order") and the engine
already has 613.8 machinery (`layers.rs:1747` `resolve_layer_order` → `:1764`
`toposort_with_timestamp_fallback`). **Take the base-characteristics fix**; a dependency-aware
fixpoint is a PB of its own and this one should land today.

Three things the plan must also do. (1) **Fix the comment, not just the code.**
`effects/mod.rs:10245-10256` argues termination from the wrong invariant ("we are checking *other*
objects") and proposes the correct fix as a *performance* note — it is the reason this survived
4.5 months. (2) **Fix the test that dodged it**: `crates/engine/tests/rules/static_grants.rs:711-760`
names Indomitable Archangel and hand-builds the effect with `condition: None` at `:736`, exercising
the filter and never the condition. Drive the def through ETB registration instead, and prove the
new probe fails before the fix by executing the revert. (3) **Disposition the landmine**:
`greymond_avacyns_stalwart.rs:38-43` instructs a future author to build a second instance of this
exact shape; either the note is corrected or the class grows the moment somebody follows it.

**Fold in `OOS-SIM2-5`** — same file, same subsystem, no wire. `layers.rs` applies P/T
modifications with bare `+=`/`-=` at **ten** sites, not the four the seed names: `:394`, `:397`
(the ±1/+1 counter path, exercised by every game, itself an unchecked subtraction over `as i32`
widened counts at `:385`/`:389`), `:1658`, `:1663`, `:1668`, `:1671` (the named `ModifyPower`/
`ModifyToughness`/`ModifyBoth` arms), and `:1698`, `:1701`, `:1715`, `:1729` (the `*Dynamic`
arms). `devilish_valet` is `Complete` by derive and genuinely doubles — `effects/mod.rs:3898-3902`
substitutes `ModifyPowerDynamic` to a concrete `ModifyPower(v)` at resolution (CR 608.2h) where `v`
resolves through `expect_characteristics(..).power`, so each trigger adds the creature's *current*
power. Use `saturating_add`/`saturating_sub` at all ten and record the ceiling as a documented
deviation. Note for the acceptance evidence: `Cargo.toml:51-54`'s `[profile.fuzz]` sets
`overflow-checks = true`, so a fuzz-profile run **panics** here while a plain `--release` run
**wraps silently to negative power** — say which profile any artefact came from.

**Mandatory experiment, and it is cheap**: run `mtg-fuzzer --games 15 --seed 1` twice, once as-is
and once with the Archangel's static commented out. If the overflow disappears, `OOS-DP3-9` /
`OOS-M11-3`'s stack-overflow half is closed by this batch and should be merged into it — record
the result either way. **Wire: none.** Expect PROTOCOL 33 / HASH 70 unmoved; gate-execute both.

---

**PB-DX20 — `PB-DX20: the offer layer cannot see a keyword-carried target requirement (OOS-CARDS2-4 + OOS-CARDS1-2)` · CORRECTNESS**

`crates/engine/src/rules/casting.rs:3723-3733` rejects an Aura spell cast with no targets, deriving
the requirement from `sba::get_enchant_target(&chars.keywords)` (CR 303.4a) rather than from a
`TargetRequirement`. The offer side cannot see it: `crates/simulator/src/legal_actions.rs` contains
**zero** occurrences of `Enchant(`, `get_enchant_target` or `target_min`, and
`tools/play-server/src/view.rs:1921-1922` derives `(target_min, target_max)` from
`mtg_engine::target_count_range(&requirements)`, which iterates `&[TargetRequirement]` only
(`casting.rs:5898-5911`) — so the sum is **0**, the browser renders a zero-target action, the human
clicks, and the engine 422s. **13 deck-legal `Complete` Aura defs** are affected (measured:
27 defs with the `"Aura"` subtype ∩ 23 with `KeywordAbility::Enchant(` = 23, ∩ `Complete` = 13,
including `rancor`, `umezawa`-adjacent staples and `hyena_umbra`, the card that stopped CARDS-2's
driver). The only reason the play-server suite is green is the suppression at
`tools/play-server/src/main.rs:1735-1740` (`KNOWN_FALSE_OFFERS`), which names this seed.

Scope: synthesise a `TargetRequirement` from `get_enchant_target` in `legal_actions.rs` so the
offer and `casting.rs` agree on **one** requirement list. Do not duplicate the derivation — SR-38's
"only offer what the engine accepts" is only true if the two arithmetics are literally the same
function, which is the lesson SIM-1 recorded when it made `effective_cast_cost` consume
`apply_commander_tax` rather than re-derive it.

**Fold in `OOS-CARDS1-2`** (Reconfigure), which is the same shape written in engine source:
`crates/engine/src/testing/replay_harness.rs:3982-3993` synthesises the Reconfigure ability with
`targets: vec![]` at `:3986` while its effect is `AttachEquipment { target: DeclaredTarget { index:
0 } }`. `lizard_blades.rs:86` is **`Completeness::Complete`** and is the corpus's only Reconfigure
def, so this is live. CR 702.151a says "**another** target creature you control", so the requirement
needs `TargetCreatureWithFilter { controller: You, exclude_self: true }` — `exclude_self` exists
(`card_definition.rs:3238`) and is honoured on this path (`casting.rs:6486-6487`). Do **not** copy
CARDS-1's equip repair verbatim; it lacks the exclusion. Update the `t7b` Reconfigure pin
(`cards1_equip_target_repair.rs:686-687`) in the same commit.

Two probes the plan must include. (1) A discriminating browser-path probe per half, each **watched
failing** by revert. (2) A probe for the *second* failure mode of the same seam: **4 Aura defs
declare no `Enchant` keyword at all** (`animate_dead`, `curse_of_opulence`, `open_the_armory`,
`sram_senior_edificer`), for which `casting.rs`'s Aura gate is skipped entirely. Two are `inert`
so nothing is live today — pin that, because it is the shape that rots silently. Delete the
`KNOWN_FALSE_OFFERS` entry on closure and let its own staleness assertion prove the deletion.
**Wire: none** — the provider is not the serialized wire and the synth site adds no variant.

---

**PB-DX21 — `PB-DX21: CR 508.1 — attackers may be declared without limit (OOS-M11-9)` · CORRECTNESS**

`crates/engine/src/rules/combat.rs:41-75` guards `handle_declare_attackers` on step, active player,
priority holder and per-attacker legality, and on nothing else; `:69-71` initialises `CombatState`
only when it is `None`, so a second declaration reuses the existing combat. CR 508.1 makes
declaring attackers a **once-per-combat turn-based action**. The blocker side already has the
guard this side lacks — `combat.rs:1103` `if combat.defenders_declared.contains(&player) { return
Err(GameStateError::AlreadyDeclaredBlockers(player)) }` (`error.rs:63`) — so the seed's own "check
whether CR 509.1 is covered too" resolves **covered**; do not widen.

**Three consequences, not the one the seed states.** The row says a re-declaration "overwrites
`combat.attackers`". (1) It does not: `:743-745` **inserts** into a map, so declarations accumulate,
and a repeated same-id entry overwrites that creature's **attack target mid-combat**. (2) `:795-805`
pushes a fresh `GameEvent::AttackersDeclared` and immediately runs `abilities::check_triggers` +
`flush_pending_triggers`, so **every attack trigger re-fires per declaration**. (3) `:759` assigns
`ps.attackers_declared_this_turn = attackers.len() as u32`, clobbering the raid count read at
`effects/mod.rs:10032` for `Condition::YouAttackedWithNOrMore`. Each needs its own discriminating
probe; the trigger re-fire is the one a human hits first.

Reachability: **14 deck-legal `Complete` vigilant creatures** (untapped attackers are re-offered
forever by `legal_actions.rs:779`/`:791`, which filters candidates on `!obj.status.tapped`), and
SIM-1 made a vigilant commander castable from the command zone. A human clicking attack twice in
the browser reaches all three. Non-vigilant attackers are self-limiting only because `combat.rs:113`
rejects an already-tapped attacker — which is an accident, not a guard.

Scope: reject a second declaration in the same combat phase with a dedicated `GameStateError`
mirroring the blockers guard. **Prefer reading `combat.attackers` over adding a field**: an
`attackers_declared` set mirroring `defenders_declared` would move HASH
(`state/hash.rs:4344` hashes the sibling), and this batch does not otherwise need a bump —
gate-compute rather than assume. On closure, **delete both client-side mitigations and say why**:
`heuristic_bot.rs:49`'s `RepeatKey::DeclareAttackers` cap (which its own comment calls "a preference
cap, not a legality cap") and `local_game_playthrough.rs:127/138/151/276/405`'s per-combat policy.
Leaving them behind re-creates the "harmless because unreachable" argument SIM-1 already burned
this project on. Expect golden-script and SR-9b per-step fingerprint churn only where a script
actually re-declares. **Wire: none expected.**

---

**PB-DX22 — `PB-DX22: make the fuzzer a real instrument (OOS-UI2-1 + OOS-SIM3-1 + OOS-SIM1-4)` · EVIDENCE INTEGRITY**

> **✅ SHIPPED 2026-08-03** (`scutemob-196`, merge `95f53b78`). The §2.4-flagged open measurement
> is SETTLED: the offer was SUPPRESSED (empty `commander_ids`), not late — OOS-SIM1-4 was the
> cause. Post-fix first-cast band 3-29 over 20 seeds. Seeds OOS-DX22-1..11 filed in
> `docs/audits/decision-point-audit.md` §8.1; every pre-merge fuzz seed is dead (OOS-DX22-7).

`crates/simulator/src/bin/fuzzer.rs:331-339` loads each library straight from `deck.main_deck` and
**never shuffles** (grep the file: zero `shuffle`), while `deck.rs:90-148` appends its ~34 basics
**last** and `zone.rs:159-161`'s `top()` is `v.last()`. `engine.rs:3485-3500` deals no opening hand.
So the first non-land is personal draw ~35-40, i.e. game turn ≈136-156 in a 4-player game — which
is why UI-2 measured 25,964 hand observations with zero non-lands at `--max-turns 80` and SIM-3
measured its earliest cast at turn 143 at the default cap. **Both are right; the seed's word
"never" is a horizon artefact and this batch must record it as a threshold.** Every historical
"fuzz parity" acceptance claim taken below ~140 turns is a claim about a land-only game.

`OOS-SIM1-4` is the same instrument's other blindness: `fuzzer.rs:322-327` places the commander
`.in_zone(ZoneId::Command(*pid))` but never calls `builder.player_commander` (the only production
registrar is `setup.rs:276`), so `commander_ids` is empty and CR 903.8 tax, CR 903.9a zone return
and CR 903.10a commander damage have **never been fuzzed**.
`crates/simulator/tests/local_game.rs:78` repeats the defect and must be fixed with it.

Scope: shuffle from the game's own seeded RNG exactly as `setup.rs:280` does, and register the
commander in both builders. **Both changes re-roll every recorded seed**, which is the entire
reason they are one batch — pay it once. Every seeded pin in `tools/play-server/src/main.rs`
(`COMBAT_SEED`, `TARGET_SEED`, `UI1_SEED`/`SIM1_SEED`/`UI2_SEED`, `UI3_SPLIT_COMBAT_SEED`, `SEED`,
`DISTINCTIVE_SEED`) and every recorded crash-report seed must be re-derived, not adjusted.

**Sequencing: this batch MUST follow PB-DX19.** Shuffling makes spells castable at ordinary depths,
which makes `indomitable_archangel`'s stack overflow common rather than rare (§2.1). Running this
first turns a rare turn-191 abort into a routine one and will look like a regression caused by this
batch.

**One open measurement the plan must settle first.** SIM-1 added a command-zone cast loop
(`legal_actions.rs:675-693`) and a commander is not in the library, so a bot should be able to cast
its commander around game turn 12-24 — more than a hundred turns before SIM-3's 143. Either SIM-3
measured a pre-SIM-1 build or something suppresses that offer for bots. One instrumented
`mtg-fuzzer --games 5 --seed 1` at HEAD settles it, and the answer changes how much of the "the
fuzzer has never cast a spell" claim survives. Do this **before** writing the acceptance criteria.
**Wire: none** (`crates/simulator` only).

---

**PB-DX23 — `PB-DX23: dredge has no answer channel for anyone (OOS-DX2-5 + OOS-DX2-2 + OOS-DX2-7)` · CORRECTNESS**

`grep -rn "ChooseDredge" crates/simulator/src/ tools/` returns **zero** hits: there is no
`LegalAction::ChooseDredge` variant at all, so neither a bot nor the **human seat in the shipped
browser** can answer a dredge offer. The seed scoped this to "the bots never dredge"; it is one
client wider. The consequence is not a lost option, it is a permanent draw-cadence corruption: the
draw step defers (`turn_actions.rs:1252-1259` draws with `offer_dredge: true` →
`replacement.rs:953-977` pushes a `PendingDraw` and returns `DredgeOffered` **without performing
the draw**), and next turn's draw discharges the stale entry (`:946-950`, `:1122-1135`) before
deferring the current one — forever. `golgari_grave_troll` declares no `completeness` field, so it
is `Complete` and deck-legal, and it is the corpus's only dredge def.

Two riders on the same path, both of which become observable only once the offer is answerable.
**`OOS-DX2-2`**: `perform_remaining_draws` (`replacement.rs:1563-1591`) and
`resolve_declined_pending_draw` (`:1130`) both pass `offer_dredge: false`, so after one deferral
the tail of a "draw three" is dredge-immune — CR 121.2 applies dredge per individual draw. PB-DP5
§3.3's argument for `false` is about not restarting a CR 616.1 application on the *same* draw and
does not extend to the tail; say so in the commit rather than silently flipping the boolean.
**`OOS-DX2-7`**: the stale-entry discharge at `:946-950` is an engine-made decision recorded in no
row of the decision ledger — it auto-declines on the player's behalf and then draws from a library
that has had a full turn cycle to be reordered. It is invisible to `decision_gate` **by
construction** (the gate walks card-def `Effect`/`Condition` DSL variants over `all_cards()`, and
dredge is a `KeywordAbility` reached through none of them) — that is a fresh instance of
`OOS-DP10-9`, not a gate bug, and it belongs in the audit's §3.1 as an AUTO-CHOSEN row whether or
not this batch closes it.

**Watch item — `OOS-DX2-3`, which this batch must not re-close the way it was closed once already.**
It says `pending_draws` can hold two entries for one player: `replacement.rs:946-950` discharges,
`:1127-1135` re-enters `perform_one_draw`, and both `:970` and `:991` `push_back` unconditionally,
so the inner call can push between them. PB-DX2 marked it CLOSED on a **structural** proof ("both
push sites are downstream of the discharge") — a claim about *where* the pushes are, not *when*
they run — and a re-review reproduced it empirically and **reopened** it. It is LOW and currently
unreachable (`grep -rn "ReplacementTrigger::WouldDraw" crates/card-defs/src/defs/` returns **0**, so
the `NeedsChoice` arm has no corpus caller) and it is deliberately unfixed: the obvious "clear
entries before each push" repair would silently destroy the re-deferred draw. It is listed here
rather than parked because **this batch edits that exact discharge**, so whoever takes PB-DX23 owns
the question. Pin is `pb_dx2_command_gates.rs:1272`. Two corrections to make while you are there:
`replacement.rs:855`'s function doc was properly corrected ("The queue is NOT bounded to one entry
per player") but the inline comments at the two push sites (`:959-962`, `:983-984`) **still assert
the retracted claim**, so a reader who greps to the push site gets the falsified story.

Scope: add `LegalAction::ChooseDredge { card }`, emit it from `LegalActionProvider` whenever a
`PendingDraw` is outstanding, map it in `params.rs`, give `heuristic_bot` a weight, and surface it
in the play-server's blocking-decision UI (the machinery PB-DP7/DP8/DP9 and UI-1 built). **Wire:
none** — `Command::ChooseDredge` and `GameEvent::DredgeChoiceRequired` already exist and
`LegalAction` is simulator-local, not the serialized wire; gate-compute anyway. Mandatory probe:
drive a real game (no state pokes) with a Grave-Troll in the graveyard and assert the draw count
after three turns, watched failing by revert.

*(PB-DX24 .. PB-DX41 and the standing PB-DX7..DX18 — see the note at the head of this subsection.)*

---

## 5. Parked — real, do not queue

| item | why parked |
|---|---|
| **OOS-DX1-1** + **OOS-DX1-2** | CR 603.4 is fail-open for leave-the-battlefield intervening-ifs (`abilities.rs:10445`, `:10455`), and `condition_is_queue_time_evaluable` is over-conservative by one variant (`effects/mod.rs:10131`). **Measured 0 corpus reach**: none of the 31 `intervening_if: Some` defs has a leave-the-battlefield trigger, and `Condition::TargetIsLegal` has zero users. Re-rank when the first such def is authored, or fold into an LKI-aware `check_condition` batch. |
| **OOS-DX5-4** | the CR 702.26e phased-out guard's `SingleObject` exemption (`layers.rs:645-654`) is a deliberate, in-source-documented deviation held to keep PB-DX5's 79 pinned effects byte-identical. Lift the constraint once, with `OOS-DX5-1`, or not at all. |
| **OOS-DX5-5** | CR 611.2f delayed-begin effects. **Measured population: 1** (`mistrise_village`), already `partial`. Capability gate for a card nobody can play. |
| **OOS-CARDS2-5** | `Effect::TurnPermanentFaceDown` — `FaceDownKind` (`types.rs:1762-1777`) has five variants, all *entering* mechanisms. **1 def unblocked** (`cyber_conversion`, `inert`). Both PROTOCOL and HASH for one card. |
| **OOS-CARDS2-1** + **OOS-CARDS2-2** | SR-37 fixture hygiene: the fixture is only as current as a gitignored `cards.sqlite` and nothing schedules a refresh (`.github/workflows/ci.yml` references neither `tools/refresh-card-fidelity-fixture.py` nor `tools/scryfall-import/`); and R4's subtype comparison is a multiset of *words*, a deliberately accepted limitation documented at `cards2_printed_field_fidelity.rs:337-357`. Both fail **loud**. Fix opportunistically inside any batch that touches the fixture. |
| **OOS-UI3-2** + **OOS-UI3-4** | both need an engine-side "publicly revealed / revealed to whom" notion before the view model can carry it (`redact.rs:122-139` decides purely from zone entitlement). **M10a-shaped**; a wire bump should carry both at once. Under-disclosure only — Architecture Invariant 7 is not at risk. |
| **OOS-UI3-3** | one more entry in `stores.js:274`'s `PASS_UNTIL_PREDICATES` table. Pure client UX; rides any play-frontend batch. |
| **OOS-DX5-2**, **OOS-DX5-6**, **OOS-DX6-3** | **design records, not work** — see §1e. Do not queue. |
| **`proliferate`** (v2's designated successor) | **23** `Complete` defs (v2 said 25; re-measured two ways — 30 defs use `Effect::Proliferate`, 7 are `partial`/`known_wrong`, and `decision_gate.rs`'s BASELINE independently lists 23) on PB-DP9's `AnswerEffectChoice` channel — the highest-count remaining agency row, and the machinery is built. Unranked because it is agency restoration on otherwise-correct cards and eight live-wrong items now sit above it. Carried forward from v2 unchanged. |
| v2's §5 parked table in full (`OOS-RS-4`, `OOS-RS1-1`/`OOS-OS8-2`, `OOS-RS-6`, *hidden_strings optionality*, `OOS-RS3-2`, the latent block `OOS-RS3-3`/`OOS-RS4-1/2/4`/`OOS-DP3-5`/`OOS-DP6-5/6/9/10`/`OOS-DP7-9/10`/`OOS-DP8-4/5/11/13`/`OOS-DP9-5/14/16/19`, the CR 800.4a sweep, the 31 legacy dormant seeds) | **inherited unchanged.** Nothing in this census closes or re-activates any of them. Read v2 §5 for the per-item reason. |

---

## 6. Source-doc updates applied by this task

**Zero engine/simulator/tool/card-def code changed.** Doc-only edits:

1. **`memory/primitives/seed-rerank-2026-08-02.md`** — this file (new). The authoritative queue.
2. **`memory/primitives/seed-rerank-2026-07-27.md`** — §4's queue banner'd **SUPERSEDED** with a
   pointer here; the header's "This document is the authoritative primitive queue" claim scoped to
   §1-§3 (which remain the canonical record of the RS/DP triage). No shipped row edited, no
   history rewritten.
3. **`docs/audits/decision-point-audit.md`** — §8.1 given a pointer to this queue; the
   `OOS-M11-10` ID-collision note updated to record that this document disambiguates the two as
   `(equip)` / `(loyalty)` and that the **closed equip row** is the one to renumber whenever a
   task next touches `crates/simulator/src/params.rs` (fewest external cites). No seed row's
   status column edited — the verdicts live here, cited by line, so the registry stays a filing
   record rather than becoming a second queue.
4. **`CLAUDE.md`** — Current State's queue pointer repointed at this document and "Next dispatch"
   changed from **PB-DX7** to **PB-DX19**, with the reason in one clause.
5. **`memory/workstream-state.md`** — queue pointer updated; a short handoff entry recording the
   census totals, the five headline findings and the two seeds this task filed.

**Two seeds filed by this task** (§1f): `OOS-RR3-1` (the `#[default] Completeness::Complete`
population has never been reviewed — 965 of 1,803 defs) and `OOS-RR3-2` (the corpus-wide
stale-blocker-note re-check that `OOS-DX3-1`'s closure named and did not file — 67 machine-checkable
notes). Both are recorded here rather than in the audit's §8.1, because §8.1 is "seeds filed by
shipped PB-DP work" and neither came from a PB.

**One census-integrity instruction for the next re-rank**: run all three passes (workstream-state
handoffs, the monthly archive, and the §8.1 registry **plus** the CARDS-2 evidence record) and
reconcile them. Each of the three misses rows the others carry — pass A misses 20, pass B misses at
least one and records almost everything as an unresolvable range, pass C misses 10. §1a records
which and why.
