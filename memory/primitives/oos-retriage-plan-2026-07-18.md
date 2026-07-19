# OOS Seed Retriage — 2026-07-18 (task scutemob-115)

> **The next active engine-primitive queue.** The EF queue is complete (`scutemob-99..114`);
> there is no open PB queue. This plan enumerates **every** open OOS seed from all source
> docs (not CLAUDE.md's headline list, which is incomplete), chain-verifies each against the
> **current post-EF-queue engine** (PROTOCOL 18 / HASH 55), classifies it, and ranks the
> live candidates into a correctness-first queue (**PB-OS1..N**). Format matches
> `ef-batch-plan-2026-07-17.md` so the dispatch loop consumes it unchanged.
>
> **Precedent**: `scutemob-98` (EF triage → `ef-batch-plan-2026-07-17.md`).
> **Method (binding, per `feedback_retriage_verification` / `feedback_verify_full_chain` /
> `feedback_pb_yield_calibration`)**: a seed is *resolved/stale* only if the **entire dispatch
> chain** exists (DSL variant → enrich/builder arm → executor/dispatch → filter honored),
> never on variant existence alone; yields discounted 2-3×; card scope verified from the
> compiled registry / oracle text, not from seed notes.

---

## 0. Headline

- **84 raw `OOS-*` / `EF-EF*` grep matches** across `memory/` + `docs/`. After removing
  template placeholders (`OOS-seed-name`, `OOS-confirmed`, `OOS-flagged`), the `-N`
  naming-convention tokens (`OOS-EWC-N`, `OOS-LKI-N`, `OOS-LKI-Power-N`, `OOS-XA2-N`,
  `OOS-EWCD-N`), and bare umbrella/heading tokens (`OOS-LKI`, `OOS-XA`, `OOS-XA2`, `OOS-EAT`,
  `OOS-EWC`, `OOS-EF5`, `OOS-EF3b`, `OOS-AC7`) → **65 distinct real `OOS-*`/`EF-EF*` seeds**
  (70 enumerated rows across the §1 buckets, deduped by the two alias pairs
  `OOS-XA-3`≡`OOS-XA2-3` and `OOS-AC9-MULTINAME`≡`OOS-AC9-SEARCHNAME`), **plus 3 stale
  `EF-W-*` finding banners** (`EF-W-MISS-3/6`, `EF-W-PB2-8` — findings, not `OOS-` seeds)
  folded into §1a. (`OOS-LKI-5` is an alias of the `OOS-LKI-Power` umbrella, counted once.)
- **Classification** (enumerated rows per bucket): **23 resolved/stale** (§1a — 13 already
  doc-marked; **10 newly verified this task** — *the headline finding: the EF/EWC/EAT/AC9
  waves silently closed several older seeds nobody had re-checked*), **16 active-PB-candidate**
  (5 correctness + 11 capability), **7 defer** (subsystem / one-off / high-complexity),
  **24 dormant-0-yield-defensive**.
- **The queue**: **PB-OS1 → PB-OS11** below, correctness-first. **PB-OS1 = OOS-EF9-1**
  (gain-control never reverts — **legal-but-wrong on 3 shipped `Complete` cards**, invariant #9,
  and *the fix helper already exists*). Fully specified in §4.
- **Zero engine/card-def code changed by this task** — planning only. Source docs updated for
  the 10 newly-resolved seeds + 3 stale banners (§6).

---

## 1. Complete seed inventory (AC 4935)

Every open and closed seed, its source doc, and its classification. **No seed silently
dropped.** Legend: **RESOLVED** = full chain exists / card shipped; **CANDIDATE** = ranked
into the PB queue (§3); **DEFER** = real but out-of-scope (subsystem/one-off/high-complexity);
**DORMANT** = filed defensively, 0 cards in the current corpus.

### 1a. Resolved / stale (chain-verified closed) — 23 rows (10 newly verified this task + 13 already doc-marked)

| Seed | Source doc | Resolved by | Chain evidence (verified 2026-07-18) |
| --- | --- | --- | --- |
| **OOS-XS-3** | pb-retriage-CC | AddSubtypes + PB-EF9 (`scutemob-110`) | `LayerModification::AddSubtypes` exists (`layers.rs:1127`); `olivia_voldaren.rs` uses it for the `{1}{R}` "becomes a Vampire" clause **and is `Completeness::Complete`**. Seed's blocker ("`AddSubtype` LayerModification does not exist") no longer holds. **NEW.** |
| **OOS-LKI-Power-2** | pb-retriage-CC | PB-EWC + PB-EAT | `ReplacementModification::EntersWithCounters { count: Box<EffectAmount> }` accepts `EffectAmount::PowerOf`; `master_biomancer.rs` uses `count: PowerOf(Source)` for the counter half and the type half (OOS-EWC-1, `EntersAsAdditionalType`) — **def is `Complete` (no marker)**. **NEW.** |
| **OOS-EWC-2** | pb-retriage-CC | card-authoring 2026-05-15 | `golgari_grave_troll.rs` exists, uses self-ETB `EntersWithCounters { count: CardCount(Graveyard) }`, **`Complete`**. **NEW.** |
| **OOS-TS-3** | pb-retriage-CC | PB-AC9 (`scutemob-52`, 13-site token-doubling rewire) | `Effect::CreateTokenAndAttachSource` arm now calls `apply_token_creation_replacement` (`effects/mod.rs:862`). Living Weapon Germ tokens now double. **NEW.** |
| **OOS-TS-4** | pb-retriage-CC | PB-LKI-CC | `EffectAmount::CounterCountAtLastKnownInformation` exists; `chasm_skulker.rs` death trigger uses it (`:57`). Pre-death counter snapshot threaded via `PendingTrigger.lki_counters`. **NEW.** |
| **OOS-AC8-1** | pb-plan-AC8 | PB-AC9 + W-EMPTY (`scutemob-96`) | `Effect::SetNoMaximumHandSize` exists (`card_definition.rs:2441`); `sea_gate_restoration.rs` uses it and is **`Complete`**. **NEW.** |
| **OOS-AC9-TOKREPL** | pb-plan-AC9 | PB-AC9 (not deferred) | The 13/13-site token-doubling completeness fix shipped in AC9 (CLAUDE.md AC9 record). The "defer to seed" branch was not taken. **NEW.** |
| **EF-W-MISS-3** | w-miss-findings | PB-EF3b (`scutemob-104`) | `derived_attack_trigger_for_keyword` synthesizes granted keyword-triggers post-layer; Adriana `Complete`. Doc lacked a CLOSED banner. **NEW banner (§6).** |
| **EF-W-MISS-6** (TransformSelf half) | w-miss-findings | PB-EF5 (`scutemob-106`) | `Effect::TransformSelf` exists + used. (Battle/Sephiroth halves split → OOS-EF5-1/2, still open.) Doc lacked a banner on the TransformSelf half. **NEW banner (§6).** |
| **EF-W-PB2-8** | w-pb2-findings | PB-EF8 (`scutemob-109`) | `Cost::ExileSelfFromHand` exists (`card_definition.rs:1268`); simian/elvish spirit guide flipped `Complete`. Doc lacked a CLOSED banner. **NEW banner (§6).** |
| OOS-TS-2 | pb-retriage-CC | PB-EF1 | (already doc-marked ✅) `ActivationCost.sacrifice_exclude_self`; izoni Complete. |
| OOS-XS-2 | pb-retriage-CC | PB-XA | (already doc-marked) `is_attacking` enforced at validate sites. |
| OOS-XS-5 | pb-retriage-CC | PB-XS-E | (already doc-marked) trigger-side `exclude_self`. |
| OOS-XA-1 | pb-retriage-CC | PB-XA2 | (already doc-marked) `is_blocking`. |
| OOS-XA-2 | pb-retriage-CC | PB-XA2 | (already doc-marked) `is_tapped`/`is_untapped`. |
| OOS-XA-3 | pb-retriage-CC | scutemob-30 | (already doc-marked) `is_nontoken` 0-yield audit. |
| OOS-XA2-3 | pb-retriage-CC | scutemob-30 | (already doc-marked) `is_nontoken` audit (dup of XA-3). |
| OOS-LKI-Power-3 | pb-retriage-CC | scutemob-29 | (already doc-marked) GameEvent LBA hash arms. |
| OOS-LKI-Power (umbrella) / OOS-LKI-5 | pb-retriage-CC / pb-plan-CD | PB-LKI-Power | (already doc-marked) `SourcePowerAtLastKnownInformation`. `OOS-LKI-5` was a proposed alias for this. |
| OOS-LKI-1 | pb-retriage-CC | — | (already doc-marked) CONFIRMED-NO-INTERACTION (Hardened Scales). |
| OOS-LKI-2 | pb-retriage-CC | — | (already doc-marked) CONFIRMED-WORKING (Parallel Lives). |
| OOS-EWC-1 | pb-retriage-CC | PB-EAT | (already doc-marked) `EntersAsAdditionalType` (Master Biomancer type half). |
| OOS-EWC-3 | pb-retriage-CC | PB-EWC-D | (already doc-marked) `CreatureControlledByOfSubtype` (Dragonstorm Globe). |

### 1b. Active-PB-candidates (open, real yield) — 16

Ranked and specified in §3. Correctness (5): **OOS-EF9-1**, **EF-EF1-A**, **OOS-EF6-1**,
**OOS-AC6-1**, **OOS-EF3b-3**. Capability (11): **OOS-EF5-3**, **OOS-EF4-1**, **OOS-EF5-4**
(sub-batch), **OOS-EF3-1**, **OOS-EF10-1**, **OOS-EF3b-1**, **OOS-EF12-1**, **OOS-XS-1**,
**OOS-EF7-1**, **OOS-LKI-3**, **OOS-TS-1**. Source docs: `ef-batch-plan` §5-§12,
`w-pb2-engine-findings`, `pb-retriage-CC`.

### 1c. Defer (real, out-of-scope) — 7

| Seed | Source | Why defer |
| --- | --- | --- |
| **OOS-EF5-1** | ef-batch-plan §9 | `CardType::Battle`/Siege — whole CR 310 subsystem (defense counters, protector-designation SBAs, defeated→exile+cast). Its own multi-session PB. 1 card (Invasion of Ikoria). |
| **OOS-EF5-2** | ef-batch-plan §9 | Sephiroth "Super Nova" — bespoke FF-set keyword action; own engine project. 1 card. |
| **OOS-XS-4** | pb-retriage-CC | Skrelv — 3 orthogonal primitives (ChooseColor at activation + `AddProtectionFromColor` + color-keyed block restriction). Post-alpha protection-from-color batch. 1 card. |
| **OOS-AC7-1** | pb-plan-AC7 | `EffectAmount::PoisonDifference` (`MAX(9-x,0)`) — Vraska Betrayal's Sting −9 one-off. Marginal. |
| **OOS-AC7-2** | pb-plan-AC7 | Non-target "choose a creature you control" at resolution + grant (Final Showdown mode 1). Needs a resolution-time non-target chooser (M10-adjacent). |
| **OOS-AC7-3** | pb-plan-AC7 | Frodo, Sauron's Bane cluster — conditional-on-own-subtype activated ability + Ring mechanics. Multi-blocker. |
| **OOS-AC6-2** | pb-plan-AC6 | Generic upkeep-sweep queue-then-evaluate ordering (Land Tax). Correctness-adjacent but 0 confirmed wrong-state card; fold into a future upkeep-trigger PB. |

### 1d. Dormant-0-yield-defensive (open, 0 cards in corpus) — 24 (25 tokens; `OOS-AC9-MULTINAME`≡`OOS-AC9-SEARCHNAME`)

Filed preventively; **build only when a card demands it** (AC-chain precedent: "do not build
primitives that unblock zero cards"). Kept in the backlog, **not** in the PB queue.

`OOS-EAT-1` (EntersAsAdditional **CardType**), `OOS-EAT-2` (…Color), `OOS-EAT-3` (…Supertype);
`OOS-LKI-Power-1` (SourceToughnessAtLKI), `OOS-LKI-Power-4` (AnyCreatureDies source-power),
`OOS-LKI-Power-5` (non-creature SBA power LKI), `OOS-LKI-4` (AnyCreatureDies source-counter);
`OOS-XS-E-1` (3 dies-side cards audit), `OOS-XS-E-2` (self-inclusive ETB regression sweep);
`OOS-AC8-2` (can't-win/can't-lose guard); `OOS-AC9-SEARCHNAME`≡`OOS-AC9-MULTINAME` (multi-name
search), `OOS-AC9-FILTERMANA` (3-way filter-land choice, M10), `OOS-AC9-ELSPETH` (live-filter vs
fixed-set approximation, M10), `OOS-AC9-AMASSCHOICE` (deterministic Army pick, M10);
`OOS-EF3b-2` (extend `derived_attack_trigger_for_keyword` to the full builder-synthesized
keyword-trigger set — Dethrone/Training/Enlist/Persist/Undying + granted Myriad/Provoke; unblocks
0 current cards, bites only when a future card *grants* one of these keywords);
`OOS-XA2-1` (color-predicate audit — likely already correct), `OOS-XA2-2` (has_name audit),
`OOS-XA2-4` (`CombatRole` enum refactor), `OOS-XA2-5` (runtime-predicate helper extraction);
`OOS-EWCD-1` (card-type receiver filter), `OOS-EWCD-2` (supertype receiver filter),
`OOS-EWCD-3` (multi-subtype AND receiver); `OOS-AC7-4` (conditional — partial creature-type
replacement, only if `SetCardTypes` not adopted); `OOS-TFS` (**never actually filed** — a
`pb-review-XA` *recommendation* to file an intervening-if-conditions seed "when the workstream
reaches that priority"; no card, no engine gap recorded — logged here so the recommendation
isn't lost).

> Note: **`min_cmc_amount`** (Birthing Pod's "MV **equal to** 1 + sacrificed creature's MV")
> is recorded in `ef-batch-plan` §12 as a distinct blocker but was **deliberately not filed as
> a seed**. It rides with **OOS-EF10-1** (PB-OS8) — the same runtime-resolution mechanism.

---

## 2. Chain-verification notes (AC 4936)

The method that mattered: **the EF queue shipped 12 PBs + PROTOCOL 2→18 in one day**, so several
seeds filed *before* that queue were re-checked against the current engine rather than trusted.
Four older seeds turned out **silently resolved** — the primitive they were blocked on had since
shipped for an unrelated reason, and their host card was already flipped `Complete` by a
backfill wave nobody cross-referenced back to the seed:

- **OOS-XS-3** (filed by PB-XS, 2026-05-14): claimed `LayerModification::AddSubtype` "does not
  exist." It exists now as `AddSubtypes` (plural, `OrdSet`), and `olivia_voldaren.rs` — flipped
  `Complete` by PB-EF9 for its *other* half — already uses it. Full chain: DSL variant
  (`AddSubtypes`) → layer application (`layers.rs:1127`, with SetTypeLine/SetCreatureTypes
  dependency handling) → live in a `Complete` def. **Resolved.**
- **OOS-LKI-Power-2** (filed by PB-LKI-Power, 2026-05-13): claimed `EntersWith` takes a static
  `u32`. PB-EWC widened it to `count: Box<EffectAmount>`; `master_biomancer.rs` reads
  `PowerOf(Source)` through it and is `Complete`. **Resolved.**
- **OOS-EWC-2 / OOS-TS-3 / OOS-TS-4 / OOS-AC8-1 / OOS-AC9-TOKREPL**: each host card now compiles
  `Complete` using the exact primitive the seed asked for (verified by reading the def + the
  engine dispatch site, not the note). **Resolved.**

The **stale-banner** finds are the mirror: **EF-W-MISS-3**, **EF-W-MISS-6** (TransformSelf half),
and **EF-W-PB2-8** were closed by PB-EF3b/EF5/EF8 respectively but their finding docs never got
the `✅ CLOSED` banner every sibling finding carries — a bookkeeping drift that would mislead the
next triage. Banners added (§6).

**Correctness confirmations (still open, verified against source):**

- **OOS-EF9-1** — `expire_end_of_turn_effects` (`layers.rs:1583`) `retain`s the `UntilEndOfTurn`
  `SetController` effect out of `continuous_effects` but **never calls
  `recompute_object_controller`** (which *does* exist, `layers.rs:1797`, wired only into PB-EF9's
  `expire_while_you_control_source_effects`). So `obj.controller` is never reverted. `sarkhan_vol`,
  `zealous_conscripts`, `karrthus_tyrant_of_jund` all ship **`Complete`** → legal-but-wrong on
  invariant-#9 cards. **The fix is built and idle.** This is why it is PB-OS1.
- **OOS-EF6-1** — `WhenTappedForMana` triggers queue as `PendingTriggerKind::Normal` with a raw
  `def.abilities` index the auto-picker can't read (exact class PB-EF3 fixed for *attack* triggers,
  not swept on the mana path). `forbidden_orchard` `known_wrong`. Verified: no
  `WhenTappedForMana` case in `enrich_spec_from_def`'s runtime `triggered_abilities` population.
- **OOS-EF7-1** — only `WhenEquippedCreatureDealsCombatDamageToPlayer` exists
  (`card_definition.rs:3389`); no any-recipient variant. Umezawa's Jitte `known_wrong`.
- **OOS-EF5-4** sub-primitives verified absent: `Cost::Sacrifice` has **no count field**
  (Westvale Abbey "Sacrifice five creatures" inexpressible); `TriggerCondition::WheneverYouAttack`
  is a **bare unit** with no count field (Legions' Landing "attacked with 3+").
- **OOS-TS-1** — *partially* superseded: `WheneverCreatureYouControlAttacks` now carries
  `filter: Option<TargetFilter>` (Dragon/Vampire-attacks works), and PB-TS shipped
  `TokenSpec.count: EffectAmount`. Anim Pakal's **surviving** blocker is narrower than filed:
  the attacker filter must express "nontoken **AND not-Gnome**" — needs a subtype-**exclusion**
  on `TargetFilter` (has `is_nontoken`, no `exclude_subtype`). Re-scoped, still open, 1 card.

---

## 3. Ordered PB queue (AC 4937) — correctness-first

**Ordering rule** (same as `ef-batch-plan` §2): (1) live-wrong `Complete` defs first (integrity,
invariant #9); (2) other correctness bugs; (3) capability gaps by discounted yield. Discounted
ship = expected clean-`Complete` after the PB + its backfill authoring, at the historical
2-3× overcount discount.

> **Wire-bump expectation** (SR-8): any PB that adds/reshapes a DSL type (`Effect`,
> `EffectAmount`, `TargetFilter`, `Condition`, `Cost`, `TriggerCondition`, `PlayerTarget`,
> `EffectFilter`, `EffectDuration`) is inside the fingerprint closure → **forces a
> `PROTOCOL_VERSION` bump and usually `HASH_SCHEMA_VERSION`**. Behaviour-only fixes (honor an
> existing field; wire an existing helper) do **not**. Flagged per-PB below.

### PB-OS1 — gain-control reversion (OOS-EF9-1) · CORRECTNESS · **RECOMMENDED FIRST DISPATCH**
- **Findings**: OOS-EF9-1. **The only live-wrong `Complete` group in the backlog.**
- **Fix**: wire the **already-existing** `recompute_object_controller` into
  `expire_end_of_turn_effects` and `expire_until_next_turn_effects` for each removed
  `SetController` continuous effect (exactly as PB-EF9 wired it into
  `expire_while_you_control_source_effects`). **No new DSL type → no PROTOCOL/HASH schema change**
  (behaviour only; but it *changes existing Threaten behaviour*, so golden scripts/tests that
  asserted the borrowed creature stays will move).
- **Candidates (3+)**: `sarkhan_vol`, `zealous_conscripts`, `karrthus_tyrant_of_jund` (all
  `Complete` today, all currently keep the stolen creature forever). Sweep every
  `Effect::GainControl` + `UntilEndOfTurn` user for more.
- **Discounted ship**: **0 new flips, 3+ integrity fixes.** Yield is *correctness*, not coverage —
  these are already `Complete` and already counted; the PB makes them *correct*.
- **Why first**: invariant #9 (a wrong `Complete` def corrupts replay history), the fix is the
  smallest possible (wire an idle helper), and the vacuous test
  `test_gain_control_until_eot_expires` (asserts the effect is removed but not that control
  reverts) must be de-vacuoused — a canary the reviewer will demand.
- **Fully specified in §4.**

### PB-OS2 — optional-cost sacrifice power (EF-EF1-A) · CORRECTNESS · micro
> ✅ **SHIPPED 2026-07-19 (`scutemob-128`).** Threaded the layer-resolved `Vec<SacrificedCreatureLki>`
> up through `pay_optional_cost`/`try_pay_optional_cost` into the `Effect::MayPayThenEffect` executor
> (`ctx.sacrificed_creature_lki`/`sacrifice_fired` set before `then`). `disciple_of_freyalise` front face
> `partial`→`Complete` (sole flip; birthing_ritual dig-blocked, ziatora triggered-`may`-blocked — both
> correctly stay partial). Decoy (anthem + wrong-creature) + decline-no-leak + card-integration tests.
> **No PROTOCOL/HASH bump.** EF-EF1-A CLOSED. Plan `pb-plan-OS2.md`, review `pb-review-OS2.md` (clean bill).
- **Findings**: EF-EF1-A.
- **Fix**: thread `EffectContext` (or an out-param) into `sacrifice_permanents_for_player` and
  push the pre-zone-move layer-resolved power into `ctx.sacrificed_creature_powers`, mirroring the
  activated-cost site `handle_activate_ability`. **No new DSL type → no schema bump.**
- **Candidates (1+)**: `disciple_of_freyalise` front face (flip `partial`→`Complete`); unblocks any
  future "you may sacrifice a creature; if you do, [X] where X = its power" optional effect.
- **Discounted ship**: **~1.** Tiny, isolated, pairs naturally with PB-OS1 (both are LKI-through-
  sacrifice threading in the same files).

### PB-OS3 — WhenTappedForMana target dispatch (OOS-EF6-1) · CORRECTNESS
> ✅ **SHIPPED 2026-07-19 (`scutemob-129`).** Root cause was a `PendingTriggerKind`/ability-index
> index-space mismatch: `fire_mana_triggered_abilities` queued the targeted mana trigger as
> `PendingTriggerKind::Normal` with a raw `def.abilities` index, but the `Normal`-kind flush
> auto-picker reads the runtime `characteristics.triggered_abilities` vec (never populated for
> `WhenTappedForMana`). Fix = **Option B**: reclassify the queued kind to the existing
> `PendingTriggerKind::CardDefETB` (whose flush lookup uses the raw `def.abilities` index the mana
> path already holds). One-identifier change in `rules/mana.rs`; immediate-mana branch untouched.
> `forbidden_orchard` `known_wrong`→**Complete** (recipient wired to `DeclaredTarget{0}`; both
> halves compose — PB-EF12 colour + PB-OS3 target). 4-player decoy compose test (stack-object
> target asserted, decoys prove recipient) + no-regression + `all_cards()` roster sweep (7 defs,
> only forbidden_orchard targets). Non-vacuity proven (revert-to-Normal fails). **No PROTOCOL/HASH
> bump.** OOS-EF6-1 CLOSED. Plan `pb-plan-OS3.md`, review `pb-review-OS3.md` (clean bill).
- **Findings**: OOS-EF6-1 (+ partially unblocks forbidden_orchard alongside the already-shipped
  EF-W-PB2-3/PB-EF12 any-color fix).
- **Fix**: mirror PB-EF3's EF-W-MISS-10 fix on the mana path — forward the def's
  `AbilityDefinition::Triggered { targets }` into the runtime `triggered_abilities` for
  `WhenTappedForMana` in `enrich_spec_from_def`, OR classify the queued trigger so the picker uses
  the def raw-index lookup. **No new wire type → likely no schema bump** (enrich-population change).
- **Candidates (1)**: `forbidden_orchard` (`known_wrong`→`Complete` — its two blockers are now both
  addressable: any-color via PB-EF12, target-dispatch via this).
- **Discounted ship**: **~1.** Small; closes a `known_wrong`.

### PB-OS4 — return-transformed / enters-transformed (OOS-EF5-3) · capability · **highest yield**
- **Findings**: OOS-EF5-3.
- **Fix**: a `ReturnTransformed`/`enters_transformed` flag on the zone-change/return effect
  (`Effect::MoveZone` or a dedicated `Effect::ReturnTransformed`) — a permanent exiled/dies and
  returns as a **new object** already on its back face (distinct from in-place `TransformSelf`,
  CR 712.18) — plus Saga-chapter integration for Fable. **New wire type → PROTOCOL bump.**
- **Candidates (4)**: `edgar_charmed_groom` (dies→delayed return transformed), `fable_of_the_mirror_breaker`
  (Saga ch. III exile→return transformed), `nicol_bolas_the_ravager` (`{4}{U}{B}{R}` exile→return
  transformed), `grist_voracious_larva` (re-verified via oracle: identical return-transformed
  mechanism, *not* a TransformSelf case).
- **Discounted ship**: **~2-3** of 4 (Fable's Saga integration is the risk).

### PB-OS5 — dynamic relative-count `EffectAmount` (OOS-EF4-1) · capability
- **Findings**: OOS-EF4-1.
- **Fix**: an `EffectAmount` variant counting battlefield objects matching a filter that can
  reference the triggering/source creature's own layer-resolved characteristics (e.g.
  `OtherAttackersSharingCreatureType { relative_to: EffectTarget }` or a general
  `CountMatchingRelativeTo`). Resolution-time count on layer-resolved subtypes; no continuous
  storage. **New wire type → PROTOCOL bump.**
- **Candidates (3)**: `shared_animosity` (`inert`→Complete; subject half already closed by PB-EF4),
  `goblin_piledriver` ("+2/+0 per other attacking Goblin"), `muxus_goblin_grandee` attack half
  (Muxus additionally needs an ETB reveal/put primitive → likely stays partial).
- **Discounted ship**: **~2** of 3.

### PB-OS6 — DFC flip-condition primitive batch (OOS-EF5-4) · capability · sub-batch
- **Findings**: OOS-EF5-4 (a/b/c/d/g — several small, independent primitives, each additive to an
  existing enum; verified absent §2).
- **Fix**: (a) `Condition` "top card is instant/sorcery" reveal (Delver); (b) count field on
  `TriggerCondition::WheneverYouAttack` (Legions' Landing); (c) count field on `Cost::Sacrifice`
  (Westvale Abbey); (d) "look at top N, put a matching card into hand, bottom the rest"
  (Growing Rites ETB — *overlaps* PB-OS8's `LookAtTopThenPlace` family); (g)
  `Effect::RemoveFromCombat { target }` (Thaumatic Compass back face, CR 506.4/508). Each is
  additive to an enum already in the SR-8 closure — **verify wire impact at plan time**; several
  will PROTOCOL-bump.
- **Candidates (5)**: `delver_of_secrets` (partial→Complete), `legions_landing`, `westvale_abbey`,
  `growing_rites_of_itlimoc` (partial→Complete), `thaumatic_compass` (partial→Complete).
- **Discounted ship**: **~3** of 5. Sub-primitives can ship in one PB or split; sequence (a)/(g)
  first (smallest), fold (d) into PB-OS8 if that ships first.

### PB-OS7 — defending-player-scoped continuous filter (OOS-EF3-1) · capability
- **Findings**: OOS-EF3-1.
- **Fix**: `EffectFilter::CreaturesControlledBy(PlayerId)` (or a `DefendingPlayer`-locked filter)
  that a continuous-effect builder **stamps with the captured defending player at creation** (the
  layer system can't read the resolving `EffectContext`). **New wire type → PROTOCOL bump.**
- **Candidates (2)**: `silumgar_the_drifting_death` ("creatures defending player controls get
  -1/-1"), Karazikar (needs the target-selection sibling + goad — partial credit).
- **Discounted ship**: **~1-2.** Candidate to fold Karazikar's target-filter + goad into the same PB.

### PB-OS8 — look-at-top-N-place-one (OOS-EF10-1) · capability
- **Findings**: OOS-EF10-1 (+ the unfiled `min_cmc_amount` Birthing Pod sub-blocker rides here).
- **Fix**: `Effect::LookAtTopThenPlace { count: EffectAmount, filter: TargetFilter, destination,
  rest_to: BottomRandomOrder | Graveyard, optional: bool }` — scopes candidates to the looked-at
  top N (unlike `SearchLibrary`), honors the existing runtime `max_cmc_amount`, places ≤1, bottoms
  the rest in randomized (deterministic-by-ObjectId in M7) order. Add `min_cmc_amount:
  Option<Box<EffectAmount>>` for Birthing Pod's exact-MV. **New wire type → PROTOCOL bump.**
- **Candidates (2+)**: `birthing_ritual` (`inert`→Complete), `birthing_pod` (via `min_cmc_amount`),
  + impulse-style "look at top N, take one, rest bottomed" cards on sweep.
- **Discounted ship**: **~2.**

### PB-OS9 — Lieutenant / "you control your commander" condition (OOS-EF3b-1) · capability
- **Findings**: OOS-EF3b-1.
- **Fix**: `Condition::YouControlYourCommander` (or a `CommanderControlled` flag on `TargetFilter`)
  checked against the effect controller's `commander_ids` + battlefield presence. Likely a **new
  `Condition` variant reusing existing wire shape** — verify PROTOCOL impact at plan time (`Condition`
  is in the SR-8 closure).
- **Candidates (1-2)**: `skyhunter_strike_force` (partial→Complete) + any other Lieutenant-keyword
  card (recurs across printings).
- **Discounted ship**: **~1-2.**

### PB-OS10 — spells-only single-target-distinctness + Jitte trigger (OOS-XS-1 + OOS-EF7-1) · capability · cleanup singletons
- **Findings**: OOS-XS-1, OOS-EF7-1 (bundled — two independent 1-card cleanups, low blast radius).
- **Fix**: OOS-XS-1 — inter-target distinctness (`TargetRequirement::TargetPermanentDistinctFrom(usize)`
  or a post-bind duplicate-rejection pass) for Hidden Strings; OOS-EF7-1 — a
  `WhenEquippedCreatureDealsCombatDamage` (any recipient) `TriggerCondition` distinct from the
  `…ToPlayer` variant, for Umezawa's Jitte (whose modal ability is already expressible post-PB-EF7).
  Both add/extend enums in the SR-8 closure → **PROTOCOL bump.**
- **Candidates (2)**: `hidden_strings`, `umezawas_jitte` (`known_wrong`→Complete).
- **Discounted ship**: **~2.** Two singletons, PB-EF11-style cleanup batch.

### PB-OS11 — cost-payment LKI counter + Anim Pakal attacker exclusion (OOS-LKI-3 + OOS-TS-1) · capability · cleanup singletons
- **Findings**: OOS-LKI-3, OOS-TS-1 (bundled cleanup).
- **Fix**: OOS-LKI-3 — `EffectContext.sacrificed_creature_counters` (parallel to
  `sacrificed_creature_powers`, populated at the activated-cost sacrifice site) for Workhorse
  ("{T}, sacrifice this: add X mana, X = +1/+1 counters"); OOS-TS-1 — a subtype-**exclusion**
  field on `TargetFilter` (`exclude_subtype`) so Anim Pakal's attacker filter can say "nontoken
  AND not-Gnome" (attacker-filter + TokenSpec.count already exist). **PROTOCOL bump** (TargetFilter
  field + EffectContext shape).
- **Candidates (2)**: `workhorse`, `anim_pakal_thousandth_moon`.
- **Discounted ship**: **~2.**

### Queue summary

| PB | Seed(s) | Class | Discounted ship | Wire bump |
| --- | --- | --- | --- | --- |
| **PB-OS1** | OOS-EF9-1 | correctness (integrity) | 3+ fixes (0 new flips) | none |
| ~~PB-OS2~~ ✅ SHIPPED `scutemob-128` | EF-EF1-A | correctness (micro) | 1 (disciple_of_freyalise) | none |
| ~~PB-OS3~~ ✅ SHIPPED `scutemob-129` | OOS-EF6-1 | correctness | 1 (forbidden_orchard) | none |
| PB-OS4 | OOS-EF5-3 | capability | ~2-3 | PROTOCOL |
| PB-OS5 | OOS-EF4-1 | capability | ~2 | PROTOCOL |
| PB-OS6 | OOS-EF5-4 (a/b/c/d/g) | capability (sub-batch) | ~3 | PROTOCOL (some) |
| PB-OS7 | OOS-EF3-1 | capability | ~1-2 | PROTOCOL |
| PB-OS8 | OOS-EF10-1 (+min_cmc) | capability | ~2 | PROTOCOL |
| PB-OS9 | OOS-EF3b-1 | capability | ~1-2 | verify |
| PB-OS10 | OOS-XS-1 + OOS-EF7-1 | capability (singletons) | ~2 | PROTOCOL |
| PB-OS11 | OOS-LKI-3 + OOS-TS-1 | capability (singletons) | ~2 | PROTOCOL |

**Total discounted ship across the queue: ~19-22 clean flips** + the PB-OS1 integrity correction
on 3 already-`Complete` cards. Correctness group (PB-OS1..OS3) first, then capability by yield.
**Defer** (§1c) and **dormant** (§1d) seeds are *not* in the queue — pull one forward only when a
targeted card makes it worthwhile (`OOS-EF12-1`'s commander-color-identity restriction and
`OOS-XS-4`'s protection-from-color are the most likely to graduate, both needing a runtime
color-subset mechanism the engine lacks today).

> **On OOS-EF12-1** (deferred from the correctness/capability split deliberately): the unserved
> `any_color` family + commander-colour-identity restriction (command_tower, arcane_signet,
> commanders_sphere, path_of_ancestry, mox_amber — 5 held-back `known_wrong` after PB-EF12) needs
> **either** a resolution-time colour channel for the stack-resolved `AddManaAnyColor` family
> **or** a runtime colour-subset restriction mechanism (`compute_color_identity` is deck-build-only
> today). That is a design decision, not a mechanical port — it belongs in an M10-adjacent mana PB
> with a coordinator decision on the restriction mechanism, not in this correctness-first queue.
> Listed as a **candidate** (§1b) but ranked below the queue; promote when the mechanism is chosen.

---

## 4. PB-OS1 — fully specified & dispatchable (AC 4937)

> ✅ **COLLECTED — `scutemob-116`, 2026-07-18.** Shipped exactly as specified: wired the idle
> `recompute_object_controller` into `expire_end_of_turn_effects` + `expire_until_next_turn_effects`
> (behaviour-only, **no PROTOCOL/HASH bump** as predicted). De-vacuoused
> `test_gain_control_until_eot_expires` (proven fail-then-pass) + added stacked-control and
> UntilEndOfTurn-vs-UntilYourNextTurn timing tests. **Roster from `all_cards()`: 2 in-scope cards**
> (`sarkhan_vol`, `zealous_conscripts`) — NOT 3; `karrthus_tyrant_of_jund` correctly uses
> `EffectDuration::Indefinite` (permanent control, CR 611.2a — reviewer-confirmed, not a bug).
> Golden-script sweep: 0 scripts encoded the old behaviour. 3479 tests, all gates green, /review
> clean (1 MEDIUM doc-only, fixed). **WhileSourceOnBattlefield reversion deferred** (separate
> SBA-removal path) — surviving half of OOS-EF9-1, carried forward as a follow-up.


**Title**: `PB-OS1: gain-control reversion — UntilEndOfTurn/UntilYourNextTurn SetController never reverts (OOS-EF9-1)`
**Class**: CORRECTNESS (invariant #9 — legal-but-wrong on shipped `Complete` defs)
**Pipeline**: `/implement-primitive` (plan → implement → review → fix). Agent:
`primitive-impl-planner` → `primitive-impl-runner` → `primitive-impl-reviewer`.

### Scope (what to change)
1. **`crates/engine/src/rules/layers.rs`** — in `expire_end_of_turn_effects` (`:1583`) and
   `expire_until_next_turn_effects` (`:1631`): around the `.filter(|e| e.duration != …).collect()`
   reassignment that drops the `UntilEndOfTurn` / `UntilYourNextTurn` continuous effects (note: it
   is a filter-collect reassignment, **not** a `retain`), collect the `ObjectId`s of every
   removed `SetController` effect's target and call `recompute_object_controller(state, id)` on
   each (mirroring the Step-2/Step-3 pattern already in
   `expire_while_you_control_source_effects`, `:~1740-1782`). Reuse the existing helper verbatim
   — it already recomputes from owner + still-active Layer-2 `SetController` effects in timestamp
   order (CR 613.7), so **stacked control** (a second still-active steal) is handled correctly.
2. **De-vacuous the canary**: `test_gain_control_until_eot_expires`
   (`crates/engine/tests/primitives/primitive_pb32.rs`) currently asserts only that the *effect*
   is removed. Add an assertion that **control reverts to the owner** after cleanup. This test
   passes today while the bug is live — it must fail before the fix and pass after.

### Explicitly NOT in scope
- `WhileSourceOnBattlefield` gain-control reversion. OOS-EF9-1 names it, but `WhileSourceOnBattlefield`
  is removed by SBA when the source leaves, not by the end-of-turn passes — it needs its own
  reconcile site. **Flag it in the close-out** as a follow-up (or fold in only if the planner
  confirms the same helper covers it without touching more golden scripts). Default: defer to keep
  PB-OS1's blast radius minimal.

### Wire-change expectation
- **None.** No DSL/`Effect`/wire type added or reshaped — this wires an existing helper into two
  existing passes. **No `PROTOCOL_VERSION` / `HASH_SCHEMA_VERSION` bump.** (If the planner finds a
  hash arm must change, that contradicts this expectation and is a signal to stop and re-scope.)

### Cards (verified `Complete` today, currently legal-but-wrong)
- `sarkhan_vol.rs` (Threaten-style `UntilEndOfTurn` steal), `zealous_conscripts.rs`,
  `karrthus_tyrant_of_jund.rs`. **Mandatory roster sweep** (per SR-34/36 lesson, enumerate from
  `all_cards()` not grep): every `Effect::GainControl` with `EffectDuration::UntilEndOfTurn` or
  `UntilYourNextTurn`. Report the full affected set in the close-out — these are integrity fixes,
  not coverage flips, so the *count* is the deliverable.

### Mandatory tests
1. **De-vacuoused canary** (above): steal a creature UntilEndOfTurn → cleanup → assert
   `state.objects[stolen].controller == owner` (and the creature is back under the original
   player's control for combat/priority).
2. **Stacked-control correctness**: two SetController effects on the same object, the
   UntilEndOfTurn one expires, the other (WhileSourceOnBattlefield) persists → control stays with
   the *second* controller, does **not** snap to owner. (Proves the helper's "keep the other
   active effect" path, not a blind owner-reset.)
3. **APNAP/timing**: creature stolen on opponent's turn returns at *that* turn's cleanup, not the
   controller's next untap (UntilEndOfTurn vs UntilYourNextTurn distinction).
4. **Golden-script reconciliation**: any `test-data/generated-scripts/` script that asserted the
   borrowed creature stays must be updated to the correct (reverting) behaviour, with a CR 611.2b/
   613.7 citation. Expect a handful; the reviewer will check none silently encode the old bug.

### Why this is the right first dispatch
Highest correctness leverage (invariant #9 on 3+ `Complete` cards), **smallest possible fix** (an
idle, already-reviewed helper wired into two existing sites), no schema blast radius, and it
de-vacuouses a test that currently lies. It is the direct analogue of `ef-batch-plan`'s "demote
swan_song / PB-EF1 first" correctness-first opening.

---

## 5. Dispatch loop notes (carried forward)

- **No gated-stub effects** in any backfill authoring (`Effect::Choose`, `MayPayOrElse`,
  `AddManaChoice`, `AddManaAnyColor` family) — barred from `Complete`. Author to a truthful marker
  if a residual clause needs one (W-PB2 guardrail).
- **Probe by execution, not source-tracing** (SR-34/36): each flipped card needs an executing test
  path proving the ability registers and produces correct game state.
- **Enumerate rosters from `all_cards()`, not grep** — the `abilities:\s*vec!\[\s*\]` /
  `mana_abilities: vec![]` trap recurs; count affected cards from the compiled registry.
- **Verify each roster card's oracle text directly (MCP/cards.sqlite)** rather than trusting a prior
  recon's per-card blocker claim — PB-EF5 caught two cards (grist, bloodline_keeper) whose filed
  2nd blocker didn't match the printed card.
- **Batch wire bumps** where a PB ships several DSL changes at once to minimize `PROTOCOL` churn;
  the machine gates (`protocol_schema`, sentinel hash tests) force the bump either way.

---

## 6. Source-doc updates applied by this task (AC 4938)

**Zero engine/card-def code changed.** Doc-only edits, all in `memory/`:

1. `memory/primitives/pb-retriage-CC.md` — added ✅ RESOLVED banners to **OOS-XS-3** (olivia /
   AddSubtypes + PB-EF9), **OOS-LKI-Power-2** (master biomancer / PB-EWC+PB-EAT), **OOS-EWC-2**
   (golgari grave-troll authored), **OOS-TS-3** (living weapon / PB-AC9), **OOS-TS-4** (chasm
   skulker / PB-LKI-CC).
2. `memory/primitives/pb-plan-AC8.md` — noted **OOS-AC8-1** RESOLVED (SetNoMaximumHandSize +
   sea_gate_restoration Complete via PB-AC9/W-EMPTY).
3. `memory/card-authoring/w-miss-engine-findings-2026-07-17.md` — added CLOSED banners to
   **EF-W-MISS-3** (PB-EF3b) and **EF-W-MISS-6** TransformSelf half (PB-EF5).
4. `memory/card-authoring/w-pb2-engine-findings-2026-07-17.md` — added CLOSED banner to
   **EF-W-PB2-8** (PB-EF8).

Everything else (13 already-doc-marked resolved, 16 candidates, 7 defer, 24 dormant) is
enumerated here; no per-doc status change was needed for those.
