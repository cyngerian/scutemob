---
name: EF Batch Plan (2026-07-17)
description: Consolidated, deduped, correctness-first PB batch plan for the 20 engine findings filed by the W-PB2 / W-EMPTY / W-MISS authoring waves + EF-13.
type: plan
---

# EF Batch Plan — 2026-07-17 (task scutemob-98)

> **STATUS UPDATE (2026-07-18, scutemob-99): PB-EF1 SHIPPED + swan_song demote DONE.**
> `TargetFilter.exclude_self` is now honored at all executor sites (PermanentCount
> resolver, `eligible_sacrifice_targets` for the SacrificePermanents effect + the
> MayPayThenEffect optional-cost path, `UntapAll`, `YouControlNOrMoreWithFilter`
> condition, and the activated-ability sacrifice cost via new
> `ActivationCost.sacrifice_exclude_self`). **Closed: EF-W-PB2-1, EF-W-EMPTY-1,
> EF-W-MISS-2, marker EF-4, marker EF-5, OOS-TS-2.** Wire bump was necessary after all
> (ActivationCost is in both the HASH and PROTOCOL closures): **HASH 43→44, PROTOCOL
> 5→6**, machine-forced. Cards flipped Complete: éomer, izoni, korvold, yawgmoth,
> commissar_severina_raine, + new copperhorn_scout (6). disciple_of_freyalise stayed
> `partial` — a SECOND blocker surfaced: **new finding EF-EF1-A** below. Coverage
> **60.0%** post-merge (1,070 clean of 1,782): PB-EF1 flipped/authored **+6** (éomer,
> izoni, korvold, yawgmoth, commissar, copperhorn), and the scutemob-100 swan_song demote
> merged in from main is **-1**, net +5 over the 1,065 pre-EF1 baseline. 3344 tests. See §5.
>
> **swan_song demote (EF-W-MISS-1) DONE by scutemob-100** (merge `615c4319`, out of band
> from PB-EF1). `swan_song.rs` now ships `known_wrong`; the live-wrong `Complete` integrity
> violation is removed. The real fix (token recipient) remains **PB-EF2**. This branch
> merged main in to pick up that change so its coverage numbers are accurate.
>
> **Next dispatch: PB-EF2** (`CreateToken` recipient — fixes swan_song properly) per §2.

> **STATUS UPDATE (2026-07-18, scutemob-102): PB-EF2 SHIPPED.** `TokenSpec` gained
> `recipient: PlayerTarget` (`#[serde(default)]`, default `Controller` — all 201 existing
> `Effect::CreateToken`/`CreateTokenAndAttachSource` construction sites unchanged, per the
> plan's design decision — recipient lives on `TokenSpec`, not as a sibling field on the
> `Effect::CreateToken` variant). `PlayerTarget` gained `ControllerOfCounteredSpell`
> (captured into new `EffectContext::countered_spell_controller` by `Effect::CounterSpell`
> the instant a valid target position resolves, BEFORE the `cant_be_countered` check — An
> Offer ruling 2022-04-29) and `ControllerOfTriggeringObject`. The `CreateToken` executor
> now loops over `resolve_player_target_list(state, &spec.recipient, ctx)` and applies
> `apply_token_creation_replacement` **per recipient**, so token doubling (Doubling Season
> etc.) keys off the recipient, not `ctx.controller`. **Closed: EF-W-MISS-1.** `swan_song`
> flipped back `known_wrong` → `Complete` (recipient now correct); new card
> `an_offer_you_cant_refuse.rs` authored `Complete`. Wire bump was necessary (both types
> are in the HASH and PROTOCOL closures): **HASH 44→45, PROTOCOL 6→7**, machine-forced (a
> second re-pin was needed after switching the `PlayerTarget::Default` impl from a manual
> `impl Default` to `#[derive(Default)]` + `#[default]` to satisfy `clippy::derivable_impls`
> — both fingerprints moved again within the same version-45/7 tail row, no further bump).
> Golden script `test-data/generated-scripts/tokens/001_swan_song_creates_bird.json`
> un-retired (its assertion was already correct — `zones.battlefield.p1`); a SEPARATE
> pre-existing approved script, `stack/045_swan_song_counters_damnation.json`, was found
> asserting the Bird onto `zones.battlefield.p2` (the pre-fix bug's exact shape) and fixed
> in place. 8 new tests in `pb_ef2_create_token_recipient.rs`, all verified non-vacuous by
> a temporary revert-and-rerun. Coverage **60.0% → 60.1%** (1,070 → 1,072 clean of 1,782 →
> 1,783; +2 clean: swan_song todo→clean, an_offer_you_cant_refuse new clean). 3354 tests
> (3344 + 8 new + 2 from the un-retired/gate-driven baseline shift). This clears PB-EF2;
> next per queue order below: PB-EF3 → PB-EF3b → capability batches EF4..EF12.

> **STATUS UPDATE (2026-07-18, scutemob-103): PB-EF3 SHIPPED.** Both correctness halves landed.
> **(A) EF-W-MISS-10 (HIGH) CLOSED** — `enrich_spec_from_def` now forwards each card-def
> `AbilityDefinition::Triggered { targets, .. }` into the runtime `TriggeredAbilityDef.targets`
> across **all 30** enrich blocks (was hardcoded `targets: vec![]`), and the auto-target fallback
> in `flush_pending_triggers` is guarded by trigger kind: `PendingTriggerKind::Normal` treats the
> runtime `triggered_abilities[idx].targets` as authoritative (no fall-through), `CardDefETB` keeps
> the `def.abilities.get(idx)` raw-index lookup. A regression sweep found 4 pre-existing sites
> mis-tagged `Normal` while raw-indexing `def.abilities` (WhenYouCastThisSpell, WhenExertedAsAttacks,
> the WhenDealsCombatDamageToPlayer carddef fallback = the Throat Slitter path, WheneverRingTemptsYou)
> and reclassified them to `CardDefETB` (their correct kind). **(B) EF-W-MISS-4 (MED) CLOSED** —
> added `EffectTarget::AttackTarget` (the player *or planeswalker* the triggering attacker is
> attacking; Player→ResolvedTarget::Player, Planeswalker present→Object, Planeswalker gone→fizzle per
> CR 506.4c, resolved **lazily** from live `state.combat.attackers[triggering_creature_id]` with a
> captured `ctx.defending_player` fallback only when the attacker itself has left combat, CR 113.7a)
> and `PlayerTarget::DefendingPlayer` (the defending player only, CR 508.4 — planeswalker's
> controller). The defending player is captured per-attacker at `AttackersDeclared` into the EXISTING
> `PendingTrigger.defending_player_id` (no new PendingTrigger field / no shape churn) and threaded to
> new `StackObject.defending_player` → new `EffectContext.defending_player`. Substituting
> EachOpponent/Controller (wrong in 4-player) is avoided — each per-attacker trigger carries its own
> defender. Wire bump necessary: **PROTOCOL 7→8** (enum variants in the SR-8 fingerprint closure),
> **HASH 45→46** (`StackObject.defending_player` in the GameState hash closure), both machine-forced,
> history rows appended. **Cards shipped (3, honest discount from the ~5-6 estimate):**
> `ojutai_soul_of_winter.rs` (new, MISS-10 — the card W-MISS authored/reviewed/removed unshipped),
> `hellrider.rs` (flip partial→Complete, TODO removed), `raid_bombardment.rs` (new). **5 candidates
> stayed blocked with real, distinct blockers** (NOT authored partial): Silumgar (defending-player-
> scoped *continuous* -1/-1 needs a locked `EffectFilter::CreaturesControlledBy` — **filed OOS-EF3-1**
> below), Brutal Hordechief (ability 2 "opponents block if able + you choose blocks" inexpressible),
> Norn's Decree + Karazikar (multiple distinct trigger gaps each), Cunning Rhetoric (a *defender-side*
> "opponent attacks you" trigger + play-from-exile — different primitive, not a defending-player
> target at all). Note: "Dragonlord Ojutai" was a mis-listed candidate — it's a combat-damage trigger
> with no target, not a MISS-10 card. **Review**: 0 HIGH; 2 MEDIUM + 3 LOW, **all 5 fixed before
> collect** (MED-1: AttackTarget wrongly redirected to pw controller instead of fizzling on CR 506.4c
> — fixed to lazy live-combat resolution; MED-2: B1 tagged *every* attack trigger with
> `defending_player_id`, giving non-targeted effects like Utvara/Dromoka a spurious stack target that
> wrongly fizzled the ability if the defender left — fixed by gating the annihilator/afflict shortcut
> to `SelfAttacks*`/`SelfBecomesBlocked` events only). No further wire bump from the fixes.
> Coverage **60.1% → 60.2%** (1,072 → 1,075 clean of 1,783 → 1,785). **3364 tests** (was 3354). This
> clears the correctness group. Next per queue order: **PB-EF4** (TriggeringCreature as effect
> subject/source).

**Purpose.** The card-authoring waves W-PB2 (`scutemob-95`), W-EMPTY (`scutemob-96`),
and W-MISS (`scutemob-97`) filed 19 engine findings, and the marker sweep
(`scutemob-88`) left EF-13 deferred for a coordinator decision. This plan consolidates
all 20 into an **ordered, deduped, correctness-first** PB queue with discounted yield
estimates, names the recommended first dispatch, and presents the EF-13 options.

**This is now the active engine-primitive queue.** The PB-AC chain (AC0..AC9) that §0 of
the campaign plan pointed at is COMPLETE; the marker sweep + W-PB2/EMPTY/MISS waves that
followed it are also complete. What remains for clean-coverage movement is the primitive
work catalogued here, then the cohort-backfill authoring behind each PB.

**Source docs (authoritative per-finding detail):**
- `memory/card-authoring/w-pb2-engine-findings-2026-07-17.md` — EF-W-PB2-1..8
- `memory/card-authoring/w-empty-engine-findings-2026-07-17.md` — EF-W-EMPTY-1
- `memory/card-authoring/w-miss-engine-findings-2026-07-17.md` — EF-W-MISS-1..10
- `memory/card-authoring/marker-sweep-engine-findings-2026-07-16.md` — EF-13 (+ EF-1..12)
- `memory/primitives/pb-retriage-CC.md` — open OOS seeds (deduped against below)

**Yield discipline.** Per `feedback_pb_yield_calibration.md`, filed rosters overcount
2–3×. Each batch below lists **candidates** (cards named in the findings) and a
**discounted ship** estimate (expected clean-Complete after authoring, at the
measured ~40–55% primitive-batch rate). "Flip" = a currently-`partial`/`known_wrong`/
`inert` def that becomes Complete; "author" = a missing/empty def written fresh.

---

## 1. Classification + dedup (AC 4818)

### 1a. The two clusters — why dedup matters

**Cluster A — `exclude_self` is unenforced outside the target-validation path.**
`TargetFilter.exclude_self: bool` already ships (PB-XS, `card_definition.rs:3016`,
HASH 19) and is honored by the declarative target-validation path and the trigger
auto-target picker (both thread `self_id`/`trigger.source`). But every executor that
matches a filter *without* a threaded source ObjectId silently ignores it, because the
shared predicate `matches_filter(&Characteristics, &TargetFilter)` receives no ObjectId
and structurally cannot compare a candidate to the source. **Five filed findings + two
older seeds are the same root cause:**

| Finding | Executor that ignores `exclude_self` | Card(s) |
| --- | --- | --- |
| EF-W-PB2-1 | `PermanentCount` amount resolver (`effects/mod.rs:6749`) | éomer (ships `known_wrong`) |
| EF-W-EMPTY-1 | `eligible_sacrifice_targets` → cost path **and** `SacrificePermanents` effect path | disciple_of_freyalise, korvold |
| EF-W-MISS-2 | `Effect::UntapAll` executor | Copperhorn Scout |
| marker EF-4 (dedup) | `Cost::Sacrifice` lowering (`replay_harness.rs:3743`) | (= EF-W-EMPTY-1 cost half) |
| marker EF-5 (dedup) | `Condition::YouControlNOrMoreWithFilter` (`effects/mod.rs:8508`) | "you control another X" conditions |
| OOS-TS-2 (dedup) | `Cost::SacrificeOther` for Izoni | Izoni, Thousand-Eyed |
| OOS-XA-2 (adjacent) | `is_tapped`/`is_untapped` — same "field the predicate can't see" shape | (deferred, own seed) |

These collapse into **one PB (PB-EF1)**. The preferred fix (per EF-W-EMPTY-1 option (a))
is to thread the source `ObjectId` into `eligible_sacrifice_targets`/`matches_filter` and
honor `exclude_self` at each executor site — one change closes the whole cluster including
the older marker/OOS seeds. This is the single highest-leverage correctness item.

**Cluster B — "the just-triggered object as the effect's subject/source."**
Two findings are the *same* gap (a continuous effect can't select the triggering
creature) and one is its point-effect sibling (a damage effect can't be sourced from the
triggering permanent):

| Finding | Gap | Card(s) |
| --- | --- | --- |
| EF-W-PB2-6 | `EffectFilter::TriggeringCreature` missing (continuous-effect filter) | dragon_tempest, ogre_battledriver, shared_animosity |
| EF-W-MISS-5 (**exact dedup of PB2-6**) | same | ogre_battledriver, Atarka, Fervent Charge, Goblin Piledriver, Muxus |
| EF-W-PB2-7 | `Effect::DealDamage` has no `source` override (sibling) | dragon_tempest, scourge_of_valkas |

PB2-6 and MISS-5 are **one finding double-filed**; merged below. PB2-7 is a closely
related point-effect variant and rides in the same PB (**PB-EF4**). Wire note: PB2-7
changes the `DealDamage` shape → PROTOCOL bump; PB2-6/MISS-5 add an `EffectFilter`
variant → also a wire change (the SR-8 closure reaches the card DSL) → PROTOCOL bump.

### 1b. Full classification table (all 20 findings)

**Correctness bugs** — a shipped or authorable def produces *wrong game state*; the fix
changes behaviour:

| Finding | Sev | What's wrong | Live-wrong in a `Complete` def today? |
| --- | --- | --- | --- |
| EF-W-MISS-1 | HIGH | `swan_song` gives the Bird to the caster, not the countered spell's controller | **YES** — swan_song has no `completeness` field → defaults `Complete` |
| EF-W-MISS-10 | HIGH | targeted `WheneverCreatureYouControlAttacks` drops its target (`enrich` hardcodes `targets: vec![]`) | No — all shipped users pass empty targets; Ojutai/Soul of Winter were *removed*, not shipped |
| EF-W-MISS-3 ✅ CLOSED (scutemob-104) | MED | granted keyword-triggers (Melee/Battle Cry/Annihilator via `AddKeyword`) are silent no-ops (static keywords grant fine; only trigger-bearing keywords) | FIXED by PB-EF3b — `layers::calculate_characteristics` now synthesizes the derived trigger from post-layers keywords via the shared `derived_attack_trigger_for_keyword` helper; Adriana authored Complete exercises it |
| EF-W-PB2-1 | MED | `PermanentCount` ignores `exclude_self` (éomer +1 too many) | No — éomer ships `known_wrong` (honestly marked) |
| EF-W-EMPTY-1 | MED | sacrifice cost/effect path ignores `exclude_self` (can sac the source itself) | No — disciple/korvold ship `partial` |
| EF-W-MISS-2 | MED | `UntapAll` ignores `exclude_self` | No |

**Capability gaps** — a feature is missing; no card ships wrong, cards sit blocked:

| Finding | Sev | Missing primitive | Candidates |
| --- | --- | --- | --- |
| EF-W-PB2-6 ≡ EF-W-MISS-5 | MED | `EffectFilter::TriggeringCreature` | ogre_battledriver, shared_animosity, Atarka, Fervent Charge, Goblin Piledriver, Muxus |
| EF-W-PB2-7 | MED | `Effect::DealDamage { source: Option<EffectTarget> }` | dragon_tempest, scourge_of_valkas |
| EF-W-PB2-2 | MED | `TargetRequirement::TargetOpponent` | shaman_of_the_pack, raiders_wake, forbidden_orchard, ajani_sleeper_agent |
| EF-W-MISS-4 | MED | "defending player / planeswalker" target for attack triggers | hellrider, Brutal Hordechief, Raid Bombardment, Norn's Decree, Karazikar, Silumgar, Cunning Rhetoric |
| EF-W-PB2-4 | MED | modal `AbilityDefinition::Activated { modes }` | goblin_cratermaker + modal-activated cohort |
| EF-W-PB2-8 | MED | `Cost::ExileSelfFromHand` (+ `activation_zone: Hand`) | simian_spirit_guide (+ Elvish/other pitch-for-mana) |
| EF-W-PB2-5 | MED | `EffectDuration::WhileYouControlSource` | olivia_voldaren + gain-control lends |
| EF-W-PB2-3 | MED | granted `any_color` ManaAbility → real color choice (not `Colorless`) | elven_chorus (+ future granted-any-color) |
| EF-W-MISS-6 | LOW* | card-invokable `Effect::TransformSelf` (+ `CardType::Battle`, "Super Nova") | 11 body-only DFCs + Invasion of Ikoria + Sephiroth |
| EF-W-MISS-7 | LOW | `ToughnessOfSacrificedCreature`, runtime `max_cmc`, "if you do" sacrifice `Condition` | Momentous Fall, Birthing Ritual, Eldritch Evolution, Victimize |
| EF-W-MISS-8 | LOW | `WheelDraw` "greatest number discarded" variant | Windfall |
| EF-W-MISS-9 | LOW | spell-only single-target `TargetRequirement` | Misdirection |

\* EF-W-MISS-6 is severity LOW but **the highest single-PB card yield** (13 candidates) —
severity ≠ priority. It is a capability gap, sequenced by yield below.

**Taxonomy / bookkeeping** (not a card-yield finding):

| Finding | Sev | Issue |
| --- | --- | --- |
| EF-13 | MED | 105 defs marked `partial` register no behaviour → are `Inert` by the taxonomy; misreports the `todo`/`empty` buckets. Coordinator call — see §3. |

### 1c. Dedup summary
- **EF-W-PB2-6 and EF-W-MISS-5 are the same finding** (`EffectFilter::TriggeringCreature`) — counted once.
- **EF-W-PB2-1, EF-W-EMPTY-1, EF-W-MISS-2** share the `exclude_self`-executor root with the
  older **marker EF-4/EF-5** and **OOS-TS-2** — one PB closes all.
- **EF-W-PB2-3** is the granted-mana-ability sibling of SR-37's `Effect::AddManaAnyColor`
  work (SR-37 fixed only the Effect path; the ManaAbility path is still stubbed) — not a
  duplicate, but blocked on the same interactive-color-choice design.
- **EF-W-MISS-1** needs a `CreateToken { recipient }` primitive; the same primitive unblocks
  **An Offer You Can't Refuse** ("its controller creates two Treasures") — noted, not
  double-counted.
- No EF finding duplicates an *open* OOS seed outright except OOS-TS-2 (folded into PB-EF1).

---

## 2. Ordered batch queue (AC 4819) — correctness-first

Ordering rule: (1) live-wrong `Complete` defs first (integrity — invariant #9), (2) other
correctness bugs, (3) capability gaps by discounted yield. Discounted ship = expected
clean-Complete after the PB + its backfill authoring.

### ► IMMEDIATE (coordinator one-liner, before any PB): demote `swan_song`
EF-W-MISS-1 is the **only live-wrong `Complete` def** in the set. Per invariant #9 a wrong
`Complete` def corrupts replay history. **Demote `swan_song.rs` to `known_wrong`** (add a
`completeness: Completeness::known_wrong("token goes to caster, not countered spell's
controller — needs CreateToken recipient, EF-W-MISS-1")` — one line) to remove the
integrity violation *now*. The real fix (recipient primitive) is **PB-EF2** below. This
demote is not a PB and should not wait in the queue.

### PB-EF1 — `exclude_self` enforcement sweep  ·  CORRECTNESS  ·  **RECOMMENDED FIRST DISPATCH**
- **Findings**: EF-W-PB2-1, EF-W-EMPTY-1, EF-W-MISS-2 (+ closes marker EF-4/EF-5, OOS-TS-2).
- **Fix**: thread source `ObjectId` into `eligible_sacrifice_targets`/`matches_filter` and
  honor `exclude_self` at each executor (`PermanentCount` amount resolver, sacrifice
  cost + `SacrificePermanents` effect, `UntapAll`, `YouControlNOrMoreWithFilter`
  condition). Field already exists → **no HASH/PROTOCOL schema change** (behaviour only).
- **Candidates (7)**: éomer (flip `known_wrong`→Complete), disciple_of_freyalise (flip
  front-face), korvold, commissar_severina_raine, yawgmoth_thran_physician, Izoni, Copperhorn Scout.
- **Discounted ship**: **~4–5 flips.** Low risk (each is a "honor a field already set");
  éomer is grep-verified the *only* `PermanentCount+exclude_self` user, so zero-regression.
- **Why first**: highest correctness leverage, smallest schema blast radius, closes the
  most already-filed findings (5 filed + 2 older) in one PB.

### PB-EF2 — `CreateToken` player-scoped recipient  ·  CORRECTNESS (fixes the demoted swan_song)
- **Findings**: EF-W-MISS-1.
- **Fix**: add `recipient: PlayerTarget` (default `Controller`) to `Effect::CreateToken`
  + `PlayerTarget::ControllerOfCounteredSpell` / `…OfTriggeringObject`. Wire change →
  **PROTOCOL + HASH bump**.
- **Candidates (2)**: swan_song (flip back to Complete), An Offer You Can't Refuse (author).
- **Discounted ship**: **~2.** Small, but it clears a HIGH integrity finding.

### PB-EF3 — attack-trigger target fidelity + defending-player  ·  CORRECTNESS + capability
- **Findings**: EF-W-MISS-10 (correctness — forward the DSL `targets` in the enrich block
  and fix the fallback to match the Triggered ability, not raw-index `def.abilities`),
  EF-W-MISS-4 (capability — a "defending player/planeswalker" `PlayerTarget`/`EffectTarget`).
- **Candidates (9)**: Ojutai, Soul of Winter (re-author, MISS-10); hellrider (flip),
  Brutal Hordechief, Raid Bombardment, Norn's Decree, Karazikar, Silumgar, Cunning Rhetoric (MISS-4).
- **Discounted ship**: **~5–6.** MISS-10 is a pure bug fix; MISS-4 is a new target that
  is *correct-in-4-player* (substituting EachOpponent/Controller is wrong in Commander).
- **Note**: MISS-10 and MISS-4 are separable if the PB proves too large; MISS-10 (bug) goes first.

### PB-EF3b — granted keyword-triggers fire  ·  CORRECTNESS  ·  ✅ DONE (scutemob-104, merge pending)
> **SHIPPED 2026-07-18.** EF-W-MISS-3 CLOSED. Shared helper `derived_attack_trigger_for_keyword`
> (single source of truth for the printed path in `builder.rs` + the granted path in
> `layers::calculate_characteristics`); post-layer reconciliation appends the derived trigger for
> each trigger-keyword in the final (post-layers) keyword set not already present, deduped by exact
> description → printed+granted collapse to one entry (OrdSet model; CR 702.121b/91b/86b "each
> instance triggers separately" is not representable — documented limitation, decoy-pinned). Melee/
> Myriad/Provoke kind-tags in `AttackersDeclared` switched raw→resolved read. **Adriana Complete
> (+1 clean coverage)**; **Skyhunter partial** (Lieutenant "control your commander" grant condition
> unrepresentable → OOS-EF3b-1). 8 tests, all decoys non-vacuous. **No PROTOCOL/HASH bump** (synthesis
> lands only in computed `Characteristics`). Filed OOS-EF3b-2 (extend helper to full keyword-trigger
> set) + OOS-EF3b-3 (pre-existing `RemoveKeyword` stale-trigger asymmetry). Coverage 60.1% → **60.2%**.
- **Findings**: EF-W-MISS-3 ✅ CLOSED.
- **Fix**: synthesize the keyword-derived triggered ability (Melee / Battle Cry / Annihilator)
  when a keyword is added by a continuous effect, not only from **printed** keywords in
  `builder.rs`. Today `LayerModification::AddKeyword` inserts into `keywords` but the derived
  trigger is never built, so an anthem granting a trigger-keyword to *other* creatures registers
  the keyword and the trigger silently never fires (static keywords like flying/haste grant fine).
- **Candidates (2)**: Adriana, Skyhunter Strike Force (Lieutenant grants).
- **Discounted ship**: **~2.** Small correctness fix; likely no schema bump (runtime synthesis,
  no new DSL type). Sequenced in the correctness group (labeled `3b` to keep the later
  numbering + cross-refs stable — it runs before the capability batches below).

### PB-EF4 — TriggeringCreature as effect subject/source  ·  capability (Cluster B)  ·  ✅ DONE (scutemob-105)
> **SHIPPED 2026-07-18.** EF-W-PB2-6 (≡ EF-W-MISS-5) and EF-W-PB2-7 CLOSED. Added
> `EffectFilter::TriggeringCreature` (continuous-effect subject, resolved to
> `SingleObject(ctx.triggering_creature_id)` at `ApplyContinuousEffect` execution, mirroring
> `EffectFilter::Source`; `None` → applies to nothing) and `Effect::DealDamage.source:
> Option<EffectTarget>` (`#[serde(default)]`; `None` = existing `ctx.source` behaviour, `Some(t)`
> resolves to one ObjectId used as the damage source across all 12 attribution reads —
> doubling/prevention/`damage_source_characteristics` for infect/lifelink/deathtouch/wither +
> `damage_source_controller` for lifelink gain + the `source:` of DamageDealt/PoisonCountersGiven,
> in both Player and Object branches). LKI-source correctness: when `source:
> Some(TriggeringCreature)` and the triggering creature has left before the trigger resolves, it
> falls back to `ctx.triggering_creature_id` (LKI-readable, SR-13 pattern), not `ctx.source`.
> **Roster-recall TODO sweep found 2 forced adds beyond the 8-card brief** (dreadhorde_invasion,
> warstorm_surge) → **7 cards shipped Complete** (est. was ~4–5): dragon_tempest (flip inert, BOTH
> primitives), scourge_of_valkas (flip partial — merges self + "another Dragon" halves into one
> `exclude_self:false` trigger), ogre_battledriver (flip inert, TriggeringCreature ×2),
> atarka_world_render (NEW), fervent_charge (NEW), dreadhorde_invasion (flip partial, lifelink
> grant), warstorm_surge (flip partial, DealDamage source + existing PowerOf(TriggeringCreature)).
> **3 stayed out**: shared_animosity `inert` (per-trigger "attacking creatures sharing a type with
> the triggering creature" count `EffectAmount` still missing → **filed OOS-EF4-1** in §8;
> subject-half closed, count-half not — honest double-blocker, NOT authored Complete);
> goblin_piledriver + muxus_goblin_grandee OUT OF SCOPE (self-attack `EffectFilter::Source` /
> ETB reveal — neither PB-EF4 primitive is their blocker; not created). terror_of_the_peaks kept
> `source: None` (deliberate contrast — "this creature deals..." = ctx.source). Wire bump
> necessary: **PROTOCOL 8→9, HASH 46→47**, both machine-forced (new EffectFilter variant +
> reshaped DealDamage reach the SR-8 fingerprint + GameState hash closures), fingerprints re-pinned
> from failing-gate output, history rows appended. **Review**: 0 HIGH, 0 MEDIUM, 2 LOW, both fixed
> before collect (LOW-1: departed-triggering-creature LKI fallback; LOW-2: redundant
> `has_card_type:Creature` on the Dragon-count filter). **3383 tests** (was 3364). Coverage 60.2%
> → **60.5%** (1,075 → 1,083 clean, +7; corpus 1,785 → 1,789). Plan/review:
> `memory/primitives/pb-plan-EF4.md` / `pb-review-EF4.md`. Next per queue: **PB-EF5** (card-invokable
> self-transform + CardType::Battle — highest yield, ~7–9).
- **Findings**: EF-W-PB2-6 ≡ EF-W-MISS-5 (`EffectFilter::TriggeringCreature`), EF-W-PB2-7
  (`DealDamage` source-override).
- **Fix**: add `EffectFilter::TriggeringCreature` (read `triggering_creature_id` from ctx)
  and an optional `source: Option<EffectTarget>` on `DealDamage`. Wire change → **PROTOCOL bump**.
- **Candidates (8)**: dragon_tempest (both halves — flip `inert`), scourge_of_valkas (flip),
  ogre_battledriver (flip), shared_animosity, Atarka, Fervent Charge, Goblin Piledriver, Muxus.
- **Discounted ship**: **~4–5.**

### PB-EF5 — card-invokable self-transform + `CardType::Battle`  ·  capability  ·  HIGHEST YIELD
- **Findings**: EF-W-MISS-6.
- **Fix**: `Effect::TransformSelf` (+ `TransformNamed`?) so a triggered/activated/conditional
  ability can flip a DFC without the external `Command::Transform`; add `CardType::Battle`
  (Invasion of Ikoria) and the "Super Nova" keyword (Sephiroth). Wire change → PROTOCOL bump.
- **Candidates (13)**: the 11 body-only DFCs (thaumatic_compass, delver_of_secrets, …),
  Invasion of Ikoria, Sephiroth.
- **Discounted ship**: **~7–9.** LOW severity but the biggest clean-coverage mover in the set;
  sequence it right after the correctness batches.

### PB-EF6 — `TargetRequirement::TargetOpponent`  ·  capability
- **Findings**: EF-W-PB2-2.
- **Fix**: add `TargetOpponent` + validation restricting candidates to opponents of the
  source's controller (CR 115.x). Wire change → PROTOCOL bump.
- **Candidates (4)**: shaman_of_the_pack (flip `partial`), raiders_wake, forbidden_orchard, ajani_sleeper_agent.
- **Discounted ship**: **~3.**

### PB-EF7 — modal `AbilityDefinition::Activated { modes }`  ·  capability
- **Findings**: EF-W-PB2-4.
- **Fix**: add `modes: Option<ModeSelection>` + `mode_targets` to `Activated`, mirror the
  `Spell`/`Triggered` modal announce/validate/resolve path. Wire change → PROTOCOL bump.
- **Candidates**: goblin_cratermaker (flip `known_wrong`) + the modal-activated cohort
  (sweep `all_cards()` for activated abilities currently forced onto the gated `Effect::Choose`).
- **Discounted ship**: **~2–4** (re-run the corpus sweep to size the cohort before dispatch).

### PB-EF8 — `Cost::ExileSelfFromHand` (activation from hand)  ·  capability
- **Findings**: EF-W-PB2-8.
- **Fix**: add `Cost::ExileSelfFromHand` + `activation_zone: Hand`, mirroring `Cost::DiscardSelf`.
- **Candidates**: simian_spirit_guide (flip `partial`) + other pitch-for-mana / activate-from-hand cards.
- **Discounted ship**: **~2–3.**

### PB-EF9 — `EffectDuration::WhileYouControlSource`  ·  capability
- **Findings**: EF-W-PB2-5.
- **Fix**: add the duration variant + its continuous-effect expiry check (differs from
  `WhileSourceOnBattlefield` only under gain-control of the source).
- **Candidates**: olivia_voldaren (flip `partial`, gain-control half) + similar borrow-a-creature lands.
- **Discounted ship**: **~1–2.**

### PB-EF10 — sacrifice-driven `EffectAmount` / runtime `max_cmc`  ·  capability
- **Findings**: EF-W-MISS-7 (three sub-gaps).
- **Fix**: `EffectAmount::ToughnessOfSacrificedCreature`; runtime-computed `max_cmc` on
  `SearchLibrary` (`N + sacrificed MV`); a `Condition` reporting whether a resolution-time
  `SacrificePermanents` fired ("if you do").
- **Candidates (4)**: Momentous Fall, Birthing Ritual, Eldritch Evolution, Victimize.
- **Discounted ship**: **~3.** (Three independent sub-gaps — could be micro-PBs.)

### PB-EF11 — low-yield singletons  ·  capability (cleanup)
- **Findings**: EF-W-MISS-8 (`WheelDraw` "greatest discarded" — Windfall), EF-W-MISS-9
  (spell-only single-target `TargetRequirement` — Misdirection).
- **Discounted ship**: **~2** (one card each). Bundle to amortize the PB overhead.

### PB-EF12 — granted `any_color` ManaAbility color choice  ·  capability (blocked on design)
- **Findings**: EF-W-PB2-3.
- **Blocker**: needs the same interactive/deterministic color-choice mechanism the gated
  `Effect::AddManaAnyColor` family needs (SR-37 fixed only the Effect path). **Do not
  dispatch until the color-choice design lands** — otherwise it re-introduces the
  `Colorless` stub on the granted path.
- **Candidates**: elven_chorus (flip `partial`) + future granted-any-color grants.
- **Discounted ship**: **~1–2**, gated behind a color-choice design decision.

### Queue summary

| PB | Class | Findings | Discounted ship | Wire bump |
| --- | --- | --- | ---: | --- |
| *(demote swan_song)* | integrity | EF-W-MISS-1 | — | none (marker) |
| **PB-EF1** ✅ DONE | correctness | PB2-1, EMPTY-1, MISS-2 (+EF-4/5, OOS-TS-2) | **6 shipped** | HASH+PROTOCOL |
| PB-EF2 | correctness | MISS-1 | ~2 | PROTOCOL+HASH |
| **PB-EF3** ✅ DONE | correctness+cap | MISS-10, MISS-4 | **3 shipped** | PROTOCOL+HASH |
| **PB-EF3b** ✅ DONE | correctness | MISS-3 | **1 Complete (Adriana) + 1 partial (Skyhunter)** | none |
| **PB-EF4** ✅ DONE | capability | PB2-6≡MISS-5, PB2-7 | **7 shipped** | PROTOCOL+HASH |
| PB-EF5 | capability | MISS-6 | ~7–9 | PROTOCOL |
| PB-EF6 | capability | PB2-2 | ~3 | PROTOCOL |
| PB-EF7 | capability | PB2-4 | ~2–4 | PROTOCOL |
| PB-EF8 | capability | PB2-8 | ~2–3 | maybe |
| PB-EF9 | capability | PB2-5 | ~1–2 | maybe |
| PB-EF10 | capability | MISS-7 | ~3 | maybe |
| PB-EF11 | capability | MISS-8, MISS-9 | ~2 | PROTOCOL |
| PB-EF12 | capability (gated) | PB2-3 | ~1–2 | maybe |

**Total discounted ship across the queue: ~37–47 flips/authors** (from ~62 candidates),
consistent with the campaign's measured primitive-batch rate. **Correctness batches
(demote + PB-EF1, EF2, EF3, EF3b) come first** and clear all six correctness findings
(MISS-1, MISS-10, MISS-3, PB2-1, EMPTY-1, MISS-2), including the
one live-wrong `Complete` def.

**Recommended first dispatch: PB-EF1** (`exclude_self` enforcement sweep) — highest
correctness leverage, no schema bump, closes 5 filed findings + 2 older seeds, and every
candidate is a low-risk "honor a field that already ships." Run the **swan_song demote**
as a coordinator one-liner in the same sitting.

---

## 3. EF-13 — RESOLVED: Option A (`scutemob-101`, 2026-07-18)

> **DONE.** The coordinator chose **Option A**. The no-behaviour `Partial` class,
> enumerated from the compiled registry (`all_cards()` + `card_registry_gate::registers_no_behavior`
> + `completeness == Partial`), was **101 defs** (not 105 — PB-EF1 and the W-* waves
> flipped a few since the marker sweep; the compiled-registry enumeration is authoritative,
> as this plan warned). **Zero** `KnownWrong` defs registered no behaviour, so the gate
> safely covers `KnownWrong` too.
>
> **Changes shipped:**
> - All 101 flipped `Completeness::partial(...)` → `Completeness::inert(...)`, each def's
>   existing blocker note preserved (all were already truthful "blocked on X" descriptions).
> - `tests/core/card_registry_gate.rs` gained `test_no_behavior_defs_are_inert_not_partial_or_known_wrong`
>   (forbids `Partial`/`KnownWrong` while `registers_no_behavior` is true) + the non-vacuity
>   proof `no_behavior_kind_gate_is_not_vacuous` (a synthetic no-behaviour canary must be
>   flagged as Partial/KnownWrong and NOT as Inert/Complete). Also proven load-bearing by
>   reverting one real flip → the corpus gate reddens.
> - `tools/authoring-report.py` rerun. **Reporting shift (deliberate):** `todo` 655→554,
>   `empty` 57→158 (both ±101). **Clean-coverage headline unchanged: 1,070 = 60.0%.**
> - **No HASH/PROTOCOL bump** — marker-only, no engine behaviour change (`Inert` and
>   `Partial` are both non-`Complete`; `validate_deck` rejected both alike before and after,
>   so invariant #9 held throughout).
>
> The options table below is retained for the record.

**Finding**: 105 defs are marked `partial` but `registers_no_behavior` is true for them —
by the `Completeness` taxonomy they are `Inert`, not `Partial`. Not a safety issue
(`validate_deck` rejects `Inert` and `Partial` identically, invariant #9 holds); it is a
**bookkeeping + trust** issue that misreports the campaign's `todo`/`empty` buckets. The
count is **105 from the compiled registry** (`all_cards()` + `registers_no_behavior`), not
99 from a source scan — the source regex `abilities:\s*vec!\[\s*\]` also matches
`mana_abilities: vec![]` (the recurring corpus trap). **Count this class from `all_cards()`,
never from source text.**

| Option | What it does | Pros | Cons |
| --- | --- | --- | --- |
| **A — Reclassify now + add the gate** (finding's recommendation) | Flip the 105 `Partial→Inert`; add `assert!(!(registers_no_behavior(d) && matches!(completeness, Partial\|KnownWrong)))` so it can't recur | Taxonomy becomes trustworthy; machine-enforced forever; pairs with `seedborn_muse`/`scavenging_ooze` already fixed | Moves headline buckets (`todo` ~667→~562, `empty` ~62→~167) — a reporting shift the campaign owner should make deliberately, not silently |
| **B — Defer, keep as a tracked debt** | Leave markers; note in campaign plan | No headline churn now; these are inherited drift, not new | Taxonomy stays unreliable; the same misread that spawned the marker sweep persists; the fix only gets harder as more defs land |
| **C — Fold into the next authoring pass** | Reclassify a def to `Inert` only as each is next touched by a PB cohort | Amortized, no big-bang; each change reviewed in context | Slow; the report stays wrong in the meantime; easy to forget the un-touched tail |

**Recommendation (non-binding): Option A**, run as its own small `chore:`-class task so the
bucket shift is one reviewable commit, and land the gate in the same change so it never
recurs. It does **not** block the PB queue — PB-EF1 can be dispatched independently. If the
owner prefers to avoid headline churn mid-campaign, **Option C** is the safe compromise.

---

## 5. New finding filed by PB-EF1 (scutemob-99)

### EF-EF1-A (MEDIUM) — `PowerOfSacrificedCreature` is not captured in the optional-cost sacrifice path
`EffectAmount::PowerOfSacrificedCreature` reads `ctx.sacrificed_creature_powers`
(`effects/mod.rs`), which is populated **only** at the activated-ability sacrifice-cost
site (`handle_activate_ability` pushes `sacrificed_lki_powers`). The optional-cost
sacrifice path used by `Effect::MayPayThenEffect` → `pay_optional_cost` →
`sacrifice_permanents_for_player` never captures the sacrificed creature's LKI power into
`ctx`, so any "sacrifice a creature; if you do, gain/draw X where X is that creature's
power" **optional** effect would resolve X = 0.

- **Instance**: `disciple_of_freyalise.rs` front face ("you may sacrifice another creature.
  If you do, you gain X life and draw X cards, where X is that creature's power"). PB-EF1
  closed its exclude_self blocker but this is a distinct, surviving blocker, so the card
  stayed `partial`.
- **Fix shape**: thread the `EffectContext` (or an out-param) into
  `sacrifice_permanents_for_player` and push the pre-zone-move layer-resolved power into
  `ctx.sacrificed_creature_powers`, mirroring the activated-cost site. Small, isolated;
  no new DSL/wire type. Micro-PB candidate; also unblocks any future optional-sacrifice
  "for each power" effect.
- **Verified**: source read 2026-07-18 — `sacrifice_permanents_for_player` takes no `ctx`
  and does not touch `sacrificed_creature_powers`; only `handle_activate_ability` does.

---

## 6. New finding filed by PB-EF3 (scutemob-103)

### OOS-EF3-1 (capability) — defending-player-scoped *continuous* effect (locked EffectFilter)
`PlayerTarget::DefendingPlayer` (added by PB-EF3) covers *point* effects scoped to the
defending player (life loss, damage, draw). It does **not** cover a *continuous* effect whose
affected set is "creatures the defending player controls", because a `ContinuousEffectDef` is
evaluated by the layer system independently of the resolving `EffectContext` — the defending
player must be **captured into the registered `ContinuousEffectDef` instance** at creation
(an `EffectFilter::CreaturesControlledBy(PlayerId)`-style *locked* filter), not read from
`ctx` at layer-application time.

- **Instance**: `silumgar_the_drifting_death.rs` — "Whenever a Dragon you control attacks,
  creatures **defending player controls** get -1/-1 until end of turn." The -1/-1 is a
  one-shot continuous effect (`ApplyContinuousEffect { ContinuousEffectDef { filter, .. } }`)
  whose `filter` must resolve to the defending player's creatures. Left unauthored (not
  `partial`) by PB-EF3; this is its real, distinct blocker.
- **Also unblocks**: Karazikar's "tap target creature **that player** controls and goad it"
  needs the same defending-player-scoped *target filter* (a target-selection sibling), plus
  goad — a related but separate gap.
- **Fix shape**: add an `EffectFilter::CreaturesControlledBy(PlayerId)` (or a
  `DefendingPlayer`-locked filter variant) that a continuous-effect builder can stamp with the
  captured defending player at creation. New DSL/wire type → PROTOCOL bump. Medium-size;
  candidate to fold into a future "defending-player-scoped set" PB alongside Karazikar's target
  filter + goad.
- **Verified**: PB-EF3 review 2026-07-18 — `EffectFilter` has no defending-player scope and a
  continuous effect cannot read the resolving `EffectContext`.

---

## 8. New finding filed by PB-EF4 (scutemob-105)

### OOS-EF4-1 (capability) — per-trigger "attacking creatures sharing a property with the triggering creature" count `EffectAmount`
`EffectFilter::TriggeringCreature` (added by PB-EF4) supplies the *subject* of a triggered
continuous effect, but there is no `EffectAmount` variant that counts *other attacking creatures
matching a property of the triggering creature*, evaluated per-trigger against the trigger
source's layer-resolved characteristics.

- **Instance**: `shared_animosity.rs` — "Whenever a creature you control attacks, it gets +1/+0
  until end of turn **for each other attacking creature that shares a creature type with it**."
  PB-EF4 closes the subject half (the buff can now be aimed at the triggering attacker via
  `EffectFilter::TriggeringCreature`), but the amount — a dynamic count of other attackers whose
  layer-resolved subtypes intersect the triggering creature's subtypes — has no representation.
  Left `inert` (NOT authored `partial`): authoring it Complete would ship a +0 buff on every
  firing (wrong game state). Honest double-blocker.
- **Also blocks / adjacent**: `goblin_piledriver.rs` ("+2/+0 for each other attacking Goblin") and
  `muxus_goblin_grandee.rs`'s attack half ("+1/+1 for each other Goblin you control") need the
  same *family* of dynamic "count other attackers/permanents matching a filter" `EffectAmount`,
  though their subject is `ctx.source` (self-attack, `EffectFilter::Source`) not
  `TriggeringCreature`, and Muxus additionally needs an ETB reveal/put primitive.
- **Fix shape**: add an `EffectAmount` variant that, at resolution, counts battlefield objects
  matching a filter that can reference the triggering/source creature's own resolved
  characteristics (e.g. `OtherAttackersSharingCreatureType { relative_to: EffectTarget }` or a
  more general `CountMatchingRelativeTo`). Resolution-time count keyed on layer-resolved subtypes;
  no continuous-effect storage needed. New DSL/wire type → PROTOCOL bump. Medium-size; candidate
  to fold into a "dynamic relative-count amounts" PB alongside the Goblin-tribal count.
- **Verified**: PB-EF4 impl 2026-07-18 — `EffectAmount` (card_definition.rs) audited; no variant
  counts "other attackers matching a property of the trigger source." `shared_animosity.rs` note
  rewritten to reflect the subject-half closure + surviving count-half gap.

---

## 4. Notes carried forward
- **Wire bumps**: PB-EF2/EF3/EF4/EF5/EF6/EF7/EF11 add or reshape wire types (the SR-8
  fingerprint closure reaches the card DSL) → each will force a `PROTOCOL_VERSION` bump and
  most a `HASH_SCHEMA_VERSION` bump. Batch them where a wave ships several at once to
  minimize version churn; the machine gates (`protocol_schema`, sentinel hash tests) will
  force the bump either way.
- **No gated-stub effects** in any backfill authoring (`Effect::Choose`, `MayPayOrElse`,
  `AddManaChoice`, `AddManaAnyColor` family) — they are barred from Complete. Author to a
  truthful marker if a residual clause needs one (W-PB2 guardrails carry forward).
- **Probe by execution, not source-tracing** (SR-34/36 lesson): each flipped card needs an
  executing test path proving the ability registers and produces correct game state.
- **Adjacent open seeds not in scope but worth folding into the right PB**: OOS-XA-1/XA-2
  (`is_blocking`/`is_tapped` target predicates — same "predicate can't see the field" shape
  as PB-EF1; consider a combined "TargetFilter runtime predicates" PB), OOS-XS-3
  (`LayerModification::AddSubtype`, needed by olivia_voldaren's `{1}{R}` half alongside PB-EF9).

---

## 7. New findings filed by PB-EF3b (scutemob-104)

### OOS-EF3b-1 (capability) — "you control your commander" (Lieutenant) continuous-grant condition
Lieutenant-style abilities ("As long as you control your commander, [static effect]") need a
condition on a continuous-effect grant (`ContinuousEffectDef.condition`) that evaluates
"the effect's controller currently controls their commander." No such condition exists:
`Condition` (card_definition.rs) has no commander variant, and `TargetFilter` has no
`is_commander` field, so `Condition::YouControlPermanent(filter)` cannot express it either
(a `TargetFilter` can't identify "is a commander," only printed characteristics).

- **Instance**: `skyhunter_strike_force.rs` (PB-EF3b) — "Lieutenant — As long as you control
  your commander, other creatures you control have melee." Authored `partial`: Flying +
  printed Melee modeled and correct, the Lieutenant anthem omitted (not modeled wrong).
- **Also blocks**: any other Lieutenant-keyword card (the keyword recurs across multiple
  printings) and any other "as long as you control your commander" static-ability card.
- **Fix shape**: add a `Condition::YouControlYourCommander` (or a `CommanderControlled` flag
  on `TargetFilter`) that `is_effect_active` / the static-registration path can check against
  the effect's controller's `commander_ids` + battlefield presence. Small, isolated addition;
  likely no PROTOCOL bump if modeled as a new `Condition` variant reusing existing wire shape
  (verify at plan time — `Condition` is inside the SR-8 closure).
- **Verified**: PB-EF3b recon 2026-07-18 — `Condition` and `TargetFilter` enums audited,
  neither expresses "is my commander."

### OOS-EF3b-2 (capability) — extend `derived_attack_trigger_for_keyword` to the full builder-synthesized keyword-trigger set
PB-EF3b's shared helper (`state::builder::derived_attack_trigger_for_keyword`) and the
`layers::calculate_characteristics` reconciliation it feeds only cover the three keywords
briefed in scope: Melee, Battle Cry, Annihilator N. `builder.rs`'s `for kw in
spec.keywords.iter()` loop synthesizes derived `TriggeredAbilityDef`s for several more
trigger-bearing keywords inline — Dethrone, Training, Enlist, Persist, Undying, and others —
none of which get a granted-keyword reconciliation. A future card granting one of these
(e.g. "Other creatures you control have dethrone") would repeat EF-W-MISS-3's silent no-op.

- **Also affects the Myriad/Provoke tag-read fix** (PB-EF3b Change 4): the raw→resolved read
  switch is defense-in-depth for these two (harmless for printed keywords, correct index for
  any future granted instance) but a *granted* Myriad/Provoke still produces no derived
  trigger at all today, because the helper doesn't synthesize one for them.
- **Fix shape**: widen the `match kw` in `derived_attack_trigger_for_keyword` to cover the
  remaining keywords whose derived defs are already built inline in `builder.rs`'s loop,
  moving each into the shared helper the same way PB-EF3b did for the first three. No new
  DSL/wire type — purely consolidating existing per-keyword `TriggeredAbilityDef` literals
  behind the one helper. Straightforward extension PB once a card actually needs one of these
  keywords granted.
- **Verified**: PB-EF3b implementation 2026-07-18 — `builder.rs` loop enumerated; Dethrone
  (~line 548 pre-batch), Training, Enlist, Persist, Undying, and others remain inline,
  untouched by this batch's helper extraction (deliberately, per plan scope).

### OOS-EF3b-3 (correctness, pre-existing) — `RemoveKeyword` leaves a stale derived trigger
`LayerModification::RemoveKeyword(kw)` (`layers.rs` ~L1207) executes only
`chars.keywords.remove(kw)`. For a **printed** trigger-keyword the derived `TriggeredAbilityDef`
lives in base `chars.triggered_abilities` (built by `builder.rs`), and `RemoveKeyword` never
touches that vec — so `collect_triggers_for_event` (reading resolved chars) still finds and fires
the trigger after the keyword was supposedly removed (e.g. a printed Melee still pumps after
`RemoveKeyword(Melee)`). **Pre-existing** — true for every printed trigger-keyword before PB-EF3b,
not introduced or worsened by it; surfaced by the reviewer because PB-EF3b formalizes the
keyword→derived-trigger relationship. `RemoveAllAbilities` is unaffected (it clears
`triggered_abilities` too, ~L1204), which is why the Humility path is correct; the asymmetry is
only in the single-keyword `RemoveKeyword` path.

- **Fix shape**: either (a) have `RemoveKeyword(kw)` also drop any `triggered_abilities` entry
  whose description matches `derived_attack_trigger_for_keyword(kw)`, or (b) drive the PB-EF3b
  reconciliation from keyword presence in **both** directions (rebuild derived triggers from the
  final keyword set rather than append-only). Option (b) composes with OOS-EF3b-2. No wire/DSL type.
- **Test gaps to add when fixed** (reviewer Finding 3, additive): a Melee-**token** case
  (`make_token` now benefits from the PB-EF3b reconciliation — currently an unasserted bonus), a
  planeswalker-attack Melee case, and a `RemoveKeyword`-after-grant case (this finding).
- **Verified**: PB-EF3b review 2026-07-18 (`memory/primitives/pb-review-EF3b.md` Finding 2).
