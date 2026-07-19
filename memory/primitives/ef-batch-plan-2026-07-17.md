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
| ~~EF-W-PB2-8~~ ✅ CLOSED (scutemob-109) | MED | `Cost::ExileSelfFromHand` (+ `activation_zone: Hand`) | simian_spirit_guide + elvish_spirit_guide → Complete |
| ~~EF-W-PB2-5~~ ✅ CLOSED (scutemob-110) | MED | `EffectDuration::WhileYouControlSource` | olivia_voldaren + dragonlord_silumgar → Complete |
| EF-W-PB2-3 | MED | granted `any_color` ManaAbility → real color choice (not `Colorless`) | elven_chorus (+ future granted-any-color) |
| EF-W-MISS-6 | LOW* | ~~card-invokable `Effect::TransformSelf`~~ ✅ DONE (scutemob-106); Battle/Super Nova SPLIT → OOS-EF5-1/2 | 11 body-only DFCs + Invasion of Ikoria + Sephiroth |
| EF-W-MISS-7 | LOW | `ToughnessOfSacrificedCreature`, runtime `max_cmc`, "if you do" sacrifice `Condition` | Momentous Fall, Birthing Ritual, Eldritch Evolution, Victimize |
| EF-W-MISS-8 | LOW | ~~`WheelDraw` "greatest number discarded" variant~~ ✅ DONE (scutemob-112) | Windfall |
| EF-W-MISS-9 | LOW | ~~spell-only single-target `TargetRequirement`~~ ✅ DONE (scutemob-112) | Misdirection |

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

### PB-EF5 — card-invokable self-transform + `CardType::Battle`  ·  capability  ·  ✅ DONE (scutemob-106)
> **SHIPPED 2026-07-18. EF-W-MISS-6 PARTIALLY CLOSED** — the `Effect::TransformSelf` half is
> DONE; Battle + Sephiroth SPLIT OUT with justification (coordinator scoping, task-comment
> recorded). Added `Effect::TransformSelf` — a unit `Effect` variant that flips the resolving
> ability's own source DFC (`ctx.source`) **in place** through the existing transform machinery.
> `handle_transform`'s flip core was extracted into `pub(crate) fn transform_permanent_in_place`
> (engine.rs) shared by both the `Command::Transform` path (byte-identical: keeps its
> zone/controller/daybound-`Err` validation) and the new executor arm; the arm honors the
> **CR 701.27f/701.28e once-per-instruction** rule via a transient `EffectContext` bool
> (`source_transformed_this_resolution`, latched only on an actual `PermanentTransformed` event,
> so a non-DFC/meld/daybound no-op doesn't consume the instruction; propagates through
> Sequence/Conditional/ForEach). No new wire *state* from the bool (transient). **Yield honestly
> ~2, not the queue's "~7–9"** — TransformSelf is *necessary* for all 11 body-only DFCs but
> *sufficient* for few: **docent_of_perfection** (author Complete) + **bloodline_keeper**
> (author Complete — the plan's "tap N Vampires" 2nd blocker was falsified vs real oracle: it's
> `{B}` + `activation_condition`) are the clean new Completes; **delver_of_secrets** demoted
> Complete→partial (integrity: shipped Complete but never transformed — swan_song failure mode);
> **thaumatic_compass** stayed partial (front TransformSelf complete; Spires back face's
> "untap attacker AND remove from combat" needs a remove-from-combat primitive — the def had
> modeled a *fabricated* "{T}: Tap target creature", caught in /review, corrected + demoted);
> **growing_rites_of_itlimoc** authored partial (transform clause wired, ETB clause blocked).
> The 8 others each have a distinct out-of-scope 2nd blocker (§9 OOS-EF5-3/4). Wire:
> **PROTOCOL 9→10, HASH 47→48**, both machine-forced (new Effect variant reaches the SR-8
> fingerprint + GameState hash closures), re-pinned from failing gates, history rows appended.
> **Seeds filed** (§9): OOS-EF5-1 (`CardType::Battle`/Siege full CR 310 subsystem — Invasion of
> Ikoria), OOS-EF5-2 (Sephiroth "Super Nova" — bespoke keyword action, own project),
> OOS-EF5-3 (return-transformed / enters-transformed as a NEW object — edgar/fable/nicol_bolas/
> grist), OOS-EF5-4 (DFC flip-condition primitives incl. (g) `Effect::RemoveFromCombat`).
> **Review**: 2 HIGH + 1 MED filed; on verification vs cards.sqlite (authoritative) 2 were
> false positives (docent P/T + token — the def was correct), 1 confirmed with a different root
> cause (thaumatic fabricated ability → fixed + demoted). 3396 tests. Plan/review:
> `memory/primitives/pb-plan-EF5.md` / `pb-review-EF5.md`.
- **Findings**: EF-W-MISS-6 (TransformSelf half CLOSED; Battle/Sephiroth split → OOS-EF5-1/2).
- **Fix**: `Effect::TransformSelf` (+ `TransformNamed`?) so a triggered/activated/conditional
  ability can flip a DFC without the external `Command::Transform`; add `CardType::Battle`
  (Invasion of Ikoria) and the "Super Nova" keyword (Sephiroth). Wire change → PROTOCOL bump.
- **Candidates (13)**: the 11 body-only DFCs (thaumatic_compass, delver_of_secrets, …),
  Invasion of Ikoria, Sephiroth.
- **Discounted ship**: **~7–9.** LOW severity but the biggest clean-coverage mover in the set;
  sequence it right after the correctness batches.

### PB-EF6 — `TargetRequirement::TargetOpponent`  ·  capability  ·  ✅ DONE (scutemob-107)
> **SHIPPED 2026-07-18. EF-W-PB2-2 CLOSED.** Added `TargetRequirement::TargetOpponent` (unit
> variant, hash discriminant 18). Validation threads the source's controller (`caster`) into
> `validate_player_satisfies_requirement` (previously took no caster) — `Ok` iff `id != caster`,
> else `InvalidTarget` (CR 102.2/102.3/601.2c; **no teams model exists**, confirmed — opponent =
> any non-controller). Object-side `valid` match rejects it (`TargetPlayer | TargetOpponent =>
> false`). Both trigger auto-target pickers (outer + UpToN-inner in `flush_pending_triggers`) got a
> dedicated arm that picks the first active opponent with **NO `.or_else` self-fallback** — the
> trigger is removed from the stack when the source has no opponent (CR 603.3d), never redirected to
> the controller. `is_target_legal` unchanged (opponent-ness is a declaration-time restriction that
> can't change at resolution). Wire bump necessary: **PROTOCOL 10→11, HASH 48→49**, both
> machine-forced (new `TargetRequirement` variant reaches the SR-8 fingerprint + GameState hash
> closures), re-pinned from failing-gate output, history rows appended.
> **Roster recall beat the brief's 2-flip estimate: 3 clean flips → Complete** —
> shaman_of_the_pack (inert→Complete; ETB `LoseLife{PermanentCount Elf/You}`), raiders_wake
> (partial→Complete; Raid end-step discard w/ `YouAttackedThisTurn` intervening-if),
> **vengeful_bloodwitch** (known_wrong→Complete — NOT in the brief; its marker's sole blocker was
> this exact variant) — plus a latent **legal-but-wrong self-target on the shipped-`Complete`
> `fell_specter`** corrected (TargetPlayer→TargetOpponent; stays Complete). **4 stayed non-Complete
> with truthful markers on their REAL surviving blockers**: blood_tribute (`EffectAmount::HalfLife`),
> blessed_alliance (idx3 fixed, idx0 kept TargetPlayer for "target player gains 4 life";
> Escalate/`mode_targets` blocker), forbidden_orchard (target fixed but **dead on OOS-EF6-1** +
> `AddManaAnyColor`/EF-W-PB2-3), ajani_sleeper_agent (no-op +1/-3, spell-type filter, Compleated).
> flare_of_malice left untouched (wrong-oracle, full re-author). **Review**: 0 HIGH, 0 MEDIUM, 3 LOW
> (CR 102.4→102.2 citation; shaman You-restriction decoy; object-target rejection test) — all fixed.
> **9 new tests** in `pb_ef6_target_opponent.rs` (primitives group 506→515) incl. the required
> 4-player accept-opponent/reject-self + a no-opponent-removed-from-stack decoy + object-target
> rejection, all proven non-vacuous (LOW-2's proof also surfaced that `matches_filter` never reads
> `TargetFilter.controller` — the real count-gate is `PermanentCount`'s outer `controller:
> PlayerTarget` field). New seed **OOS-EF6-1** filed (§10). Coverage **60.5% → 60.7%** (1,084 →
> 1,087 clean of 1,792; +3 = the 3 flips; fell_specter was already Complete so no movement).
> Plan/review: `memory/primitives/pb-plan-EF6.md` / `pb-review-EF6.md`.
- **Findings**: EF-W-PB2-2.
- **Fix**: add `TargetOpponent` + validation restricting candidates to opponents of the
  source's controller (CR 115.x). Wire change → PROTOCOL bump.
- **Candidates (4)**: shaman_of_the_pack (flip `partial`), raiders_wake, forbidden_orchard, ajani_sleeper_agent.
- **Discounted ship**: **~3.**

### PB-EF7 — modal `AbilityDefinition::Activated { modes }`  ·  capability  ·  ✅ DONE (scutemob-108)
> **SHIPPED 2026-07-18. EF-W-PB2-4 CLOSED.** Added `modes: Option<ModeSelection>` to
> `AbilityDefinition::Activated` (DSL) and the runtime `ActivatedAbility`, plus
> `modes_chosen: Vec<usize>` to `Command::ActivateAbility`. `handle_activate_ability` now
> validates the mode choice (CR 601.2b via 602.2b: range / min-max / no-duplicate-per-700.2d /
> ascending-sort) and splits per-mode target announcement via the existing PB-AC4
> `ModeSelection.mode_targets` + `validate_targets_positional` — **all before any cost is paid**
> (CR 602.2 illegal-activation rewind). **Approach (a): the chosen mode's effect is baked into
> `embedded_effect` at activation** (both eligible cards cost `SacrificeSelf`, so the source is
> gone at resolution, CR 400.7) — `resolution.rs` is UNCHANGED, and the mode is frozen so no
> intervening board change can re-derive it (LKI). Multi-mode + `mode_targets` is hard-rejected
> (flag-don't-extend, mirrors casting's Escalate+mode_targets reject). Corpus sweep re-derived the
> cohort from `all_cards()` (activated abilities on the gated `Effect::Choose`) = **3**:
> **goblin_cratermaker** (known_wrong→**Complete**; 2 modes, colorless filter as
> `exclude_colors:{WUBRG}`+`non_land`, confirmed honored by `matches_filter`) and **cankerbloom**
> (known_wrong→**Complete**; 3 modes, Proliferate mode has an EMPTY target slice per CR 700.2c so
> it no longer demands an artifact+enchantment on the board). **umezawas_jitte stays
> `known_wrong`** — modal is now expressible but a **second, distinct blocker survives** (its
> counter trigger fires only on combat-damage-to-players; oracle is "deals combat damage" to any
> recipient) → filed **OOS-EF7-1** (`w-pb2-engine-findings-2026-07-17.md`). Mechanical surface:
> 774 `AbilityDefinition::Activated` def literals got `modes: None,` (brace-matched, no
> `once_per_turn` corruption) + ~289 test/engine `modes_chosen`/`modes` backfills. Wire bump:
> **PROTOCOL 11→12, HASH 49→50**, both machine-forced (Command frame + DSL `modes` reach the SR-8
> fingerprint; runtime `ActivatedAbility.modes` reaches the GameState hash), re-pinned from
> failing-gate output, history rows appended. **15 tests** in `pb_ef7_modal_activated.rs` (fwd +
> reverse mode decoys, exclude_colors decoy, LKI discriminator with a second colorless permanent
> added mid-resolution, invalid-index / non-modal / duplicate / min-max-bounds / multi-mode+
> mode_targets / unchoosable-mode-no-legal-target rejections, per-card mode tests, version
> sentinels), all proven non-vacuous. **Review**: 0 HIGH, 2 MEDIUM (both test-quality: LKI
> discriminator + validation-branch coverage), 3 LOW — all fixed. Coverage **60.7% → 60.8%**
> (1,087 → 1,089 clean of 1,792; +2 = the 2 flips). Plan/review:
> `memory/primitives/pb-plan-EF7.md` / `pb-review-EF7.md`.
- **Findings**: EF-W-PB2-4.
- **Fix**: add `modes: Option<ModeSelection>` + `mode_targets` to `Activated`, mirror the
  `Spell`/`Triggered` modal announce/validate/resolve path. Wire change → PROTOCOL bump.
- **Candidates**: goblin_cratermaker (flip `known_wrong`) + the modal-activated cohort
  (sweep `all_cards()` for activated abilities currently forced onto the gated `Effect::Choose`).
- **Discounted ship**: **~2–4** (re-run the corpus sweep to size the cohort before dispatch).

### PB-EF8 — `Cost::ExileSelfFromHand` (activation from hand)  ·  capability  ·  ✅ DONE (scutemob-109)
> **SHIPPED 2026-07-18. EF-W-PB2-8 CLOSED.** Added `Cost::ExileSelfFromHand` (DSL) +
> `ActivationZone::Hand` (decorative; the cost variant is the single behavioral source of truth,
> mirroring how `Cost::DiscardSelf`/Channel drives hand activation with no zone marker). The two
> cards ("Exile this card from your hand: Add {mana}") are **mana abilities** (CR 605.1a — no
> target, could add mana), so they lower through `mana_ability_lowering` → `handle_tap_for_mana`
> and resolve **stacklessly** (CR 605.3b/605.5 — no priority reset, `players_passed` untouched),
> never the stack-using `handle_activate_ability`. `mana_ability_cost_components` gained an
> accepting arm and the SR-34 **no-tap guard was relaxed scoped to `exile_self_from_hand` only**
> (CR 400.7 makes the exile inherently one-shot/self-consuming, so the "free repeatable stackless
> mana ability" seam that guard closed does not apply; a negative-control test pins that Food-Chain
> `SacrificeSelf`-only / `Cost::Mana`-only no-tap costs still decline). `handle_tap_for_mana` now
> fetches the ability before the zone check and branches Hand-vs-Battlefield legality (owner check,
> mirroring Channel), then exiles the source to `ZoneId::Exile` (`GameEvent::ObjectExiled`,
> reusing the pitch-cost hand-exile shape) before producing mana. CR 106.12: no `{T}` → mana
> replacements (Nyxbloom/Mana Reflection) and WhenTappedForMana triggers correctly do NOT fire
> (both gated on `requires_tap`). **Corpus sweep = 2 flips**: **simian_spirit_guide**
> (inert→**Complete**) and **elvish_spirit_guide** (known_wrong→**Complete**, killing a live
> free-infinite-`{G}` bug). False positives verified out of scope: saw_it_coming (Foretell),
> chrome_mox (Imprint), gemstone_caverns (Luck-counter ETB). **PROTOCOL 12→13, HASH 50→51**
> (both machine-forced — `Cost`/`ActivationZone` reach the SR-8 wire closure; the two new runtime
> bool fields reach the GameState hash). `Command::TapForMana` frame unchanged. 7 new tests
> (happy ×2, CR-605.5 stackless invariant, decoy A/B each proven non-vacuous, lowering-gate
> positive+negative control, CR-106.12 `requires_tap` invariant). /review 0 HIGH / 0 MED / 1 LOW
> (elvish oracle_text "card"→"creature", fixed). Coverage 60.7% → **60.9%** (1,091/1,792, +2). All
> gates green. Plan/review: `memory/primitives/pb-plan-EF8.md` / `pb-review-EF8.md`.
- **Findings**: EF-W-PB2-8 — CLOSED.
- **Fix**: add `Cost::ExileSelfFromHand` + `activation_zone: Hand`, mirroring `Cost::DiscardSelf`.
- **Candidates**: simian_spirit_guide (flip `partial`) + other pitch-for-mana / activate-from-hand cards.
- **Discounted ship**: **~2–3** → **2 shipped.**

### PB-EF9 — `EffectDuration::WhileYouControlSource`  ·  capability ✅ DONE (scutemob-110, 2026-07-18)
- **Findings**: EF-W-PB2-5. ✅ CLOSED.
- **Shipped**: `EffectDuration::WhileYouControlSource(PlayerId)` (CR 611.2b/c) + one-shot expiry
  (`expire_while_you_control_source_effects`, per-iteration in `check_and_apply_sbas`) +
  `recompute_object_controller`. Never-resumes enforced by permanent removal (not a live check);
  phased-out source stays controlled (CR 702.26e). **Discovery: no control-reversion existed in the
  engine at all** (WhileSourceOnBattlefield/UntilEndOfTurn gain-control never reverted) — this PB
  built it. PROTOCOL 13→14, HASH 51→52.
- **Yield: 2 shipped** — olivia_voldaren + dragonlord_silumgar → Complete. roil_elemental stays
  partial (optional "you may" wrapper inexpressible — MayPayOrElse stub / MayPayThenEffect auto-pays);
  kellogg_dangerous_mind stays partial (sacrifice-N-of-subtype cost). **OOS-EF9-1 filed** (latent
  never-reverts gap: WhileSourceOnBattlefield + UntilEndOfTurn gain-control — sarkhan_vol,
  zealous_conscripts, karrthus_tyrant_of_jund; `test_gain_control_until_eot_expires` is vacuous re:
  reversion). Coverage 60.9% → **61.0%** (1093/1792).

### PB-EF10 — sacrifice-driven `EffectAmount` / runtime `max_cmc`  ·  capability ✅ DONE (scutemob-111, 2026-07-18)
- **Findings**: EF-W-MISS-7 (three sub-gaps). ✅ CLOSED.
- **Shipped**: `EffectAmount::ToughnessOfSacrificedCreature` (disc 22) +
  `EffectAmount::ManaValueOfSacrificedCreature` (disc 23) + `TargetFilter.max_cmc_amount:
  Option<Box<EffectAmount>>` (runtime "or less" search cap, honored by `SearchLibrary` only) +
  `Condition::SacrificeFired` (disc 48, CR 608.2c/608.2h "if you do"). Single data-model
  migration backs all three: `SacrificedCreatureLki { power, toughness, mana_value }` replaces
  the old `Vec<i32>` powers carrier on `EffectContext`/`StackObject`/`AdditionalCost::Sacrifice`.
  `sacrifice_permanents_for_player` now returns the LKI of everything actually sacrificed.
  **Bonus fix** (found while authoring Victimize): `Effect::MoveZone` never applied
  `ZoneTarget::Battlefield { tapped }` — fixed to mirror the sibling `SearchLibrary` pattern.
  PROTOCOL 14→15, HASH 52→53.
- **Yield: 3 shipped** (Momentous Fall, Eldritch Evolution, Victimize) **+ 2 unlisted forced-adds**
  from the mandatory TODO sweep (Miren, the Moaning Well; Diamond Valley — both sub-gap-1 only).
  Birthing Ritual stays inert (OOS-EF10-1: the "look at top 7 / place one / bottom-random" dig has
  no primitive). Birthing Pod stays blocked on a distinct gap (needs EXACT mana value, not "or
  less" — noted, not fixed). **OOS-EF10-1 filed** (`w-miss-engine-findings-2026-07-17.md`).
  15 unit tests (3 decoys, all proven non-vacuous). Coverage delta: see
  `python3 tools/authoring-report.py` output in the collection report.

### PB-EF11 — low-yield singletons  ·  capability (cleanup)  ·  ✅ DONE (`scutemob-112`, 2026-07-18)
- **Findings**: EF-W-MISS-8 (`WheelDraw` "greatest discarded" — Windfall), EF-W-MISS-9
  (spell-only single-target `TargetRequirement` — Misdirection). **Both CLOSED.**
- **Discounted ship**: **~2** (one card each). Bundle to amortize the PB overhead.
- **Shipped: 2/2 as planned.** COMMIT 1 — `WheelDraw::GreatestDiscarded` (CR 121.1): the
  `Effect::WheelHand` executor restructured into a two-pass dispose-all-then-draw-max branch
  keyed on the draw variant (`ThatMany`/`Fixed` byte-identical); **Windfall** Complete.
  PROTOCOL 15→16, HASH 53→54. COMMIT 2 — spell-only
  `TargetRequirement::TargetSpellWithSingleTarget` (CR 115.7a/115.7b/115.10): validates
  zone==Stack, kind ∈ {`Spell`, `MutatingCreatureSpell`}, exactly one declared target, +
  self-target prevention; **Misdirection** Complete (Pitch alt cost — exile a blue card, no
  life — + `Effect::ChangeTargets { must_change: true }`). PROTOCOL 16→17, HASH 54→55.
  Two non-vacuous decoys per feature. Coverage 61.0% → **61.2%** (1,100/1,798; +2 Complete).
  3466 tests. **Also folded in a pre-existing main-breakage fix** (9 `imbl`/`equivalent`
  `.get(id)`→`.get(*id)` sites — fresh dep resolve picks `equivalent 1.0.2`; Cargo.lock is
  untracked). /review: see `memory/primitives/pb-review-EF11.md`.

### PB-EF12 — granted `any_color` ManaAbility color choice  ·  capability  ·  ✅ DONE (scutemob-114, 2026-07-18) — **CLOSES THE EF QUEUE**
> **SHIPPED 2026-07-18. EF-W-PB2-3 CLOSED. THE EF QUEUE IS COMPLETE.** The colour choice rides the
> activation Command (coordinator decision, `memory/decisions.md` 2026-07-18, CR 605.3b — a mana
> ability never uses the stack, so the choice is made at activation): `Command::TapForMana` gains
> `chosen_color: Option<ManaColor>` (`#[serde(default)]`), validated in `handle_tap_for_mana` against
> the offered set — for an `any_color: true` ManaAbility it must be `Some(c)` with `c ∈ WUBRG`;
> `Some(Colorless)` is rejected (CR 106.1b — colorless is a type, not a colour) and `None` is rejected
> (**no silent Colorless default** — the SR-37 stub eliminated); a fixed-colour ability rejects any
> `Some(_)`. The chosen colour flows into both the step-7b mana-replacement preview (so Caged Sun names
> the real colour) and the step-8 pool addition. This serves BOTH the intrinsic tap path (Command
> Tower-shape lands/rocks whose `AddManaAnyColor` lowers via `try_as_tap_mana_ability`) AND the
> **granted** path (`LayerModification::AddManaAbility(any_color:true)` for creatures you control —
> Cryptolith Rite / Citanul Hierophants / Paradise Mantle / Bootleggers Stash were shipping `Complete`
> while silently producing colorless, a latent bug no gate caught; now correct). **No HASH bump**
> (`Command` is not in the GameState hash closure; colour lands in `ManaPool`, already per-colour) —
> **PROTOCOL 17→18** only, machine-forced, fingerprint re-pinned, history row appended. 106 existing
> `TapForMana` literals backfilled `chosen_color: None`; simulator (`legal_actions.rs`/`mana_solver.rs`/
> `random_bot.rs`) + script harness emit a concrete engine-legal colour (SR-38 precedent), pinned by a
> new simulator legality test. **Yield: elven_chorus flipped Complete (grant wired) + 16 restored to
> Complete** (birds_of_paradise, chromatic_lantern, city_of_brass, darksteel_ingot,
> decanter_of_endless_water, dragons_hoard, dragonstorm_globe, elvish_harbinger, goldhound,
> mana_confluence, mox_jasper, mox_opal, ornithopter_of_paradise, patchwork_banner, patriars_seal,
> staff_of_compleation) — **17 total**. **7 held back on real second blockers** with rewritten notes
> (command_tower/arcane_signet/commanders_sphere/path_of_ancestry/mox_amber — commander-colour-identity
> restriction, unenforceable at runtime; forbidden_orchard/glistening_sphere — unrelated blockers); one
> eyeballed restore (deathrite_shaman) was reverted after the refined gate caught it (targeted ability,
> CR 605.1a disqualifies it from mana-ability status). **Gate refinement** (`effect_choose_gate.rs`):
> `registered_colors` maps `any_color`→all five WUBRG (was `{Colorless}`);
> `no_complete_def_uses_an_any_color_mana_stub` narrowed to flag only UNSERVED usages (restricted/amount
> variants always; plain `AddManaAnyColor` iff the def registers no `any_color` mana ability), with the
> served-vs-unserved logic pinned non-vacuously and the "mixed served+unserved" hole documented +
> asserted-absent. **OOS-EF12-1 filed** (the unserved any-color family: `AddManaAnyColorRestricted`,
> `AddManaOfAnyColorAmount`, `AddManaChoice`, plain `AddManaAnyColor` on spell/triggered/sacrifice-other
> costs — still Colorless; plus the commander-colour-identity restriction on Command Tower et al.).
> 7 new primitive tests (`pb_ef12_any_color_choice.rs`, decoys empirically non-vacuous) + 2 gate tests +
> 1 simulator test. **3476 tests** (was 3453). Coverage **61.1% → 62.1%** (1,098 → 1,117 clean of
> 1,796 → 1,798). Plan/review: `memory/primitives/pb-plan-EF12.md` / `pb-review-EF12.md`.
- **Findings**: EF-W-PB2-3 — CLOSED.
- **Discounted ship**: **17 shipped** (est. was ~1–2; the family re-examination surfaced 16 restorable
  demoted rocks/lands beyond the named elven_chorus flip).

### Queue summary

| PB | Class | Findings | Discounted ship | Wire bump |
| --- | --- | --- | ---: | --- |
| *(demote swan_song)* | integrity | EF-W-MISS-1 | — | none (marker) |
| **PB-EF1** ✅ DONE | correctness | PB2-1, EMPTY-1, MISS-2 (+EF-4/5, OOS-TS-2) | **6 shipped** | HASH+PROTOCOL |
| PB-EF2 | correctness | MISS-1 | ~2 | PROTOCOL+HASH |
| **PB-EF3** ✅ DONE | correctness+cap | MISS-10, MISS-4 | **3 shipped** | PROTOCOL+HASH |
| **PB-EF3b** ✅ DONE | correctness | MISS-3 | **1 Complete (Adriana) + 1 partial (Skyhunter)** | none |
| **PB-EF4** ✅ DONE | capability | PB2-6≡MISS-5, PB2-7 | **7 shipped** | PROTOCOL+HASH |
| **PB-EF5** ✅ DONE | capability | MISS-6 | **2 shipped** | PROTOCOL+HASH |
| **PB-EF6** ✅ DONE | capability | PB2-2 | **3 flips + fell_specter fix** | PROTOCOL+HASH |
| **PB-EF7** ✅ DONE | capability | PB2-4 | **2 shipped** | PROTOCOL+HASH |
| **PB-EF8** ✅ DONE | capability | PB2-8 | **2 shipped** | PROTOCOL+HASH |
| **PB-EF9** ✅ DONE | capability | PB2-5 | **2 shipped** | PROTOCOL+HASH |
| **PB-EF10** ✅ DONE | capability | MISS-7 | **3 shipped + 2 forced-adds** | PROTOCOL+HASH |
| **PB-EF11** ✅ DONE | capability | MISS-8, MISS-9 | **2 shipped** | PROTOCOL+HASH (×2) |
| **PB-EF12** ✅ DONE | capability | PB2-3 | **17 shipped** | PROTOCOL only |

> **✅ THE EF QUEUE IS COMPLETE (2026-07-18, scutemob-114).** All 20 findings (EF-W-PB2-1..8,
> EF-W-EMPTY-1, EF-W-MISS-1..10) + EF-13 are closed; every PB-EF1..EF12 shipped. Remaining
> any-color work is deferred as OOS-EF12-1 (unserved `AddManaAnyColor` family + commander-colour-
> identity restriction). Next campaign work is cohort-backfill authoring behind the shipped
> primitives, not further EF primitives.

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
> ✅ **CLOSED 2026-07-19 by PB-OS2 (`scutemob-128`).** The optional-cost path now threads the
> already-layer-resolved `Vec<SacrificedCreatureLki>` (returned by `sacrifice_permanents_for_player`)
> up through `pay_optional_cost` → `try_pay_optional_cost` → the `Effect::MayPayThenEffect` executor,
> which sets `ctx.sacrificed_creature_lki` / `ctx.sacrifice_fired` **before** running `then` — mirroring
> the mandatory `Effect::SacrificePermanents` executor and the activated-cost site. `disciple_of_freyalise`
> front face flipped `partial`→`Complete`. No new DSL type; **no PROTOCOL/HASH bump**. (Field name in the
> original finding was `sacrificed_creature_powers`; it became `sacrificed_creature_lki` in PB-EF10.)

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
> ✅ **CLOSED — PB-OS5 (`scutemob-135`, 2026-07-19).** Added `EffectAmount::OtherAttackersSharingCreatureType
> { relative_to: EffectTarget }` (discriminant 24) — at resolution counts OTHER attacking creatures (any
> controller) sharing ≥1 layer-resolved creature type with `relative_to` (the triggering creature),
> Changeling-safe (CR 702.73a). `shared_animosity` inert→**Complete**. The adjacent goblin-tribal count
> cards did NOT need this variant: `goblin_piledriver` (new→**Complete**, +2/+0 via `Sum(count,count)`)
> and `goblin_rabblemaster` (pump implemented, stays partial on forced-attack) reuse the existing
> `AttackingCreatureCount{EachPlayer, exclude_self}`; `muxus_goblin_grandee` attack-half (new, stays
> partial on the ETB reveal/put — see below) reuses `PermanentCount{Controller, exclude_self}`. Single
> PROTOCOL 19→20 + HASH 56→57. Muxus's ETB reveal-top-six/put-Goblins primitive remains open under
> **OOS-EF10 / PB-OS8**. Record: `memory/primitives/pb-plan-OS5.md`, `pb-review-OS5.md`.

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

## 9. New findings/seeds filed by PB-EF5 (scutemob-106)

PB-EF5 shipped `Effect::TransformSelf` (CR 701.27a/f, 712.18) — a unit `Effect` that flips
the resolving ability's own source DFC in place. 2 cards flipped Complete (thaumatic_compass,
docent_of_perfection), 1 integrity demote (delver_of_secrets — mismarked `Complete`, never
actually transformed), 2 additional cards authored beyond the plan's baseline discretion
(bloodline_keeper shipped **Complete** — the plan's stated 2nd blocker for it, "tap N other
creatures" activation cost, was **verified false** against the real oracle text: "{B}: Transform
this creature. Activate only if you control five or more Vampires" is a plain mana cost +
`activation_condition`, both of which already existed; growing_rites_of_itlimoc authored
`partial`, transform half wired, ETB half genuinely blocked). Full deviation note in the
implementation report. Four seeds below (§7's OOS-EF5-1/2 were already filed by the coordinator
pre-dispatch; OOS-EF5-3/4 are new, surfaced during this batch's per-card chain-verification).

### OOS-EF5-1 (capability, coordinator-filed pre-dispatch) — `CardType::Battle` / Siege subsystem
See PB-EF5 plan §7 / coordinator DECISION 2 (`memory/primitive-wip.md`). CR 310 (defense
counters, protector-designation SBAs, Siege "defeated → exile + cast transformed"). Unblocks
Invasion of Ikoria // Zilortha. A whole PB; not touched by this batch.

### OOS-EF5-2 (capability, coordinator-filed pre-dispatch) — Sephiroth "Super Nova" bespoke keyword action
See PB-EF5 plan §7 / coordinator DECISION 3. FF-set DFC back-face keyword action, its own
engine project, unrelated to body-only-DFC flips. Not touched by this batch.

### OOS-EF5-3 (capability, new — surfaced by this batch) — return-transformed / enter-the-battlefield-transformed
> ⚠️ **NARROWED — PB-OS4 (`scutemob-130`), 2026-07-19 — NOT fully closed.** The
> return-transformed *mechanism* shipped: `Effect::ExileSourceAndReturnTransformed` (exile the
> source, then return it to the battlefield as a **new object** on its back face — CR 400.7 new
> object, back-face characteristics **layer-resolved**, no counters/auras carried, Saga CR 714.4
> no-sacrifice). PROTOCOL 18→19, HASH 55→56 (single bump). **BUT no candidate card flipped
> Complete**, because implementation+review surfaced a deeper, general transform gap:
> **transformed permanents do not gather their back-face non-keyword abilities** (static/ETB/upkeep
> trigger/activated) — `register_static_continuous_effects`, `queue_carddef_etb_triggers`, and the
> upkeep trigger scan all iterate the FRONT `def.abilities` unconditionally (only *keywords* read
> `back_face`, layers.rs). Filed as **OOS-OS4-2** (§11). Consequences: `fable_of_the_mirror_breaker`
> ships **partial** (ch. III return-transformed wired = real primitive usage, but ch. I token-trigger
> + ch. II bounded discard inexpressible, and the back-face Reflection activated ability is
> non-functional per OOS-OS4-2); `edgar_charmed_groom` **left unauthored** (would re-register its
> front Vampire anthem onto the returned artifact = wrong game state until OOS-OS4-2);
> `nicol_bolas_the_ravager` + `grist_voracious_larva` **left unauthored** (planeswalker-back
> starting-loyalty gap = **OOS-OS4-1**, §11; grist additionally needs an entered-from-graveyard
> trigger condition). The two unused effect variants the runner speculatively added
> (`ReturnSourceToBattlefieldTransformed[NextEndStep]`) were removed (SHIP-NARROWED, W6
> no-speculative-machinery). Full record: `memory/primitive-wip.md`, `memory/primitives/pb-review-OS4.md`.

A permanent is exiled (or dies) and returns as a **new object**, already on its back face.
This is a fundamentally different mechanism than `TransformSelf` (which flips a permanent
**in place**, same `ObjectId`, CR 712.18). Needed by:
- **edgar_charmed_groom** — dies → delayed trigger returns it to the battlefield transformed
  at the next end step.
- **fable_of_the_mirror_breaker** — Saga chapter III: exile, return transformed.
- **nicol_bolas_the_ravager** — `{4}{U}{B}{R}`: exile, return transformed.
- **grist_voracious_larva** — re-verified via MCP/oracle-text lookup during this batch (the
  plan's table description, "ETB mill 3; if a creature card in GY, transform," was **stale/
  wrong**): the real oracle text is "Whenever Grist or another creature you control enters, if
  it entered from your graveyard or you cast it from your graveyard, you may pay {G}. If you
  do, exile Grist, then return it to the battlefield transformed under its owner's control." —
  the identical return-transformed mechanism, not a `TransformSelf` case at all. Moved here
  from the plan's OOS-EF5-4(e) slot (see below) — it was miscategorized as a "2nd blocker
  needing a condition," when the actual blocker is the flip *mechanism* itself.
- **Fix shape**: a `ReturnTransformed`/`enters_transformed` flag on the zone-change/return
  effect (`Effect::MoveZone` or a dedicated `Effect::ReturnTransformed`) + Saga-chapter
  integration for fable. New wire type → PROTOCOL bump. A whole PB (4 cards).

### OOS-EF5-4 (capability, new — DFC flip-condition primitives, batchable) — distinct 2nd blockers
> ✅ **SHIPPED-NARROWED — PB-OS6 (`scutemob-136`), 2026-07-19.** Three of the five sub-primitives
> shipped, flipping 3 cards to `Complete`; the other two were honestly narrowed to follow-up seeds.
> **Shipped:** (a) `delver_of_secrets` partial→Complete via `Condition::TopCardIsInstantOrSorcery`;
> (b) `legions_landing` NEW→Complete via `Condition::YouAttackedWithNOrMore(u32)` +
> `PlayerState.attackers_declared_this_turn` (CR 508.4 — only declared attackers count); (g)
> `thaumatic_compass` partial→Complete via `Effect::RemoveFromCombat { target }` +
> `GameEvent::RemovedFromCombat` + a shared `remove_from_combat` helper factored out of
> `apply_regeneration` (CR 506.4). Single batched **PROTOCOL 20→21 / HASH 57→58**.
> **Narrowed/deferred:** (c) `westvale_abbey` → new seed **OOS-OS6-1** (multi-count sacrifice
> *activation cost* — needs an `ActivationCost.sacrifice_count` field **and** a plural
> `Command::ActivateAbility.sacrifice_targets` Command-wire reshape, ~90-site churn for a single
> card; `kellogg_dangerous_mind` "Sacrifice five Treasures" rides the same primitive). (d)
> `growing_rites_of_itlimoc`'s ETB "look at top N, take a match, bottom the rest" → **PB-OS8**
> (`Effect::LookAtTopThenPlace` family; building a minimal duplicate here would force a throwaway
> wire type + a second PROTOCOL bump when OS8 lands). growing_rites stays `partial` (its end-step
> `TransformSelf` half already works). Plan `memory/primitives/pb-plan-OS6.md`, review
> `pb-review-OS6.md` (clean bill), queue entry `oos-retriage-plan-2026-07-18.md` §3 (PB-OS6).
> Per-item status: (a) ✅ (b) ✅ (g) ✅ (c) → OOS-OS6-1 (d) → PB-OS8 (e) already moved to OOS-EF5-3.

The remaining roster DFCs whose transform clause could use `TransformSelf` but whose
**surviving** clause needs a separate primitive (verified against real oracle text, not the
plan's table, during this batch):
- **(a) delver_of_secrets** — "top card of library is instant/sorcery" reveal `Condition`
  (only `TopCardIsCreatureOfChosenType` exists). Demoted to `partial` this batch (§6a
  integrity fix); needs this primitive to reach Complete.
- **(b) legions_landing** — an "attacked with N+ creatures" trigger/condition
  (`TriggerCondition::WheneverYouAttack` is a bare unit, no count field, verified by full
  scan of `TriggerCondition`). Left unauthored — authoring it now would not exercise
  `TransformSelf` at all (the flip clause is the ONLY thing blocked), so there is nothing to
  gain by a partial ship; wait for this primitive.
- **(c) westvale_abbey** — a **multi-count** sacrifice cost (`Cost::Sacrifice(TargetFilter)`
  has no count field; "Sacrifice five creatures" cannot be expressed). Left unauthored for
  the same reason as (b) — the transform ability itself can't be modeled at all without this,
  so `TransformSelf` gets no corpus usage from this card either.
- **(d) growing_rites_of_itlimoc** — a "look at top N, put a matching card into hand, bottom
  the rest" effect (only `Scry`/`Surveil` exist, which reorder rather than selectively draw).
  Authored `partial` THIS BATCH: the end-step transform-if-4-creatures clause IS wired via
  `TransformSelf` (real corpus usage), the ETB clause is the omitted, truthfully-marked
  blocker. Back face (2 mana abilities) fully implemented.
- **(e) grist_voracious_larva** — REMOVED from this list; re-verification found it belongs to
  OOS-EF5-3 (return-transformed mechanism), not a 2nd-condition blocker. The plan's original
  table entry for this card was stale/wrong — see OOS-EF5-3 above.
- **(g) thaumatic_compass** — a **remove-from-combat** effect primitive. The front (search +
  end-step `TransformSelf`) is complete, but the Spires of Orazca back face
  ("{T}: Untap target attacking creature an opponent controls **and remove it from combat**")
  has no way to express the combat-removal clause — only `Effect::Regenerate` references
  removal-from-combat, internally, with no standalone effect. `Effect::UntapPermanent` and an
  `is_attacking`/`controller: Opponent` target filter DO exist, so the untap + target are
  modeled, but the omitted combat-removal clause keeps the def `partial` (demoted from a
  mistaken Complete during /review — the pre-fix def modeled a **fabricated** "{T}: Tap target
  creature an opponent controls", a legal-but-wrong ability that did not match the printed
  card at all). Fix shape: `Effect::RemoveFromCombat { target }` (CR 506.4/508). Found by the
  /review pass, corroborated against cards.sqlite — a third PB-EF5 case where a per-card claim
  (this time the reviewer's *and* the def's own oracle comment) didn't match the printed card.
- **bloodline_keeper — REMOVED from this list entirely.** The plan's table listed its 2nd
  blocker as a "tap N other creatures" activation cost; the real oracle text ("{B}: Transform
  this creature. Activate only if you control five or more Vampires") has no such cost — it's
  a mana cost plus an `activation_condition`, both already in the DSL. **Authored Complete this
  batch**, not left as a seed. Lesson for the next planner: verify each roster card's oracle
  text directly (MCP/cards.sqlite) rather than trusting a prior recon pass's per-card blocker
  claims — this is the second PB-EF5-adjacent case (after grist) where the filed 2nd blocker
  didn't match the printed card.
- **Fix shape**: (a)/(b)/(c)/(d)/(g) are each small, independent primitives; several could ship
  in one PB together. None requires a new wire type by itself (a `Condition` variant, a
  `TriggerCondition` count field, a `Cost::Sacrifice` count field, and an
  `Effect::RemoveFromCombat` are all additive to existing enums already in the SR-8 closure —
  still verify at plan time).

---

## 10. New finding filed by PB-EF6 (scutemob-107)

### OOS-EF6-1 (correctness, pre-existing) — `WhenTappedForMana` triggers can't resolve a declared target
> ✅ **CLOSED — PB-OS3 (`scutemob-129`), 2026-07-19.** Fixed via **Option B**: in
> `fire_mana_triggered_abilities`, the push-to-stack branch now queues the trigger as the existing
> `PendingTriggerKind::CardDefETB` instead of `Normal`. `CardDefETB`'s flush lookup
> (`has_ability_targets` / target resolution) uses `def.abilities.get(ability_index)` — the raw def
> index the mana path already sets — so the declared `targets` resolve; the `Normal` path instead
> read the runtime `characteristics.triggered_abilities` vec, which `enrich_spec_from_def` never
> populated for `WhenTappedForMana`. `CardDefETB` carries no ETB semantics (a pure raw-index marker,
> per PB-EF3 A2); `doubler_applies_to_trigger` keys on `triggering_event` (None), so no spurious
> doubling; the immediate-mana branch is untouched. `forbidden_orchard` `known_wrong`→**Complete**
> (recipient wired to `DeclaredTarget{0}`; both halves compose). 4-player decoy compose test +
> `all_cards()` roster sweep (only forbidden_orchard among the 7 targets). **No PROTOCOL/HASH bump.**
> Plan `pb-plan-OS3.md`, review `pb-review-OS3.md` (clean bill).

`rules/mana.rs::fire_mana_triggered_abilities` queues a `TriggerCondition::WhenTappedForMana`
trigger as `PendingTriggerKind::Normal` with the ability's **raw `def.abilities` index**. The
trigger auto-target picker in `flush_pending_triggers`, for a `Normal`-kind trigger, reads its
target requirements from the runtime `characteristics.triggered_abilities` — which
`enrich_spec_from_def` **never populates for `WhenTappedForMana`** (unlike `WhenEntersBattlefield`
etc.). So a `WhenTappedForMana` trigger that declares a target (`targets: vec![...]`) gets **no
target selected** — the declared target is dead. This is the exact `PendingTriggerKind::Normal`
vs raw-`def.abilities`-index mismatch class that PB-EF3 (EF-W-MISS-10) fixed for the *attack*
enrich blocks, but the mana-trigger dispatch path was not in that sweep.

- **Instance**: `forbidden_orchard.rs` — "Whenever you tap this land for mana, target opponent
  creates a 1/1 colorless Spirit creature token." PB-EF6 gave it a correct
  `TargetRequirement::TargetOpponent` and PB-EF2 gives `TokenSpec.recipient`, but wiring
  `recipient: DeclaredTarget{0}` produced **0 tokens** (the target never resolves), so the
  recipient change was reverted and the def stays `known_wrong` on this gap **plus** the
  `AddManaAnyColor` blocker (EF-W-PB2-3). Verified empirically: `mana_triggers::test_mana_trigger_forbidden_orchard`
  went 1 token → 0 tokens when the recipient was wired, then restored on revert.
- **Fix shape**: mirror PB-EF3's EF-W-MISS-10 fix on the mana path — either forward the def's
  `AbilityDefinition::Triggered { targets }` into the runtime `triggered_abilities` for
  `WhenTappedForMana` in `enrich_spec_from_def`, or classify the queued trigger with the correct
  kind so the picker uses the def raw-index lookup. Small; no new wire type. Unblocks
  forbidden_orchard's token-recipient once EF-W-PB2-3 (AddManaAnyColor) is also resolved.
- **Verified**: PB-EF6 impl 2026-07-18 — the recipient wiring was attempted, root-caused, and
  reverted; the surviving-blocker marker on `forbidden_orchard.rs` records the full dispatch chain.

## 11. New finding filed by PB-EF9 (scutemob-110)

### OOS-EF9-1 (correctness, pre-existing) — `WhileSourceOnBattlefield` / `UntilEndOfTurn` gain-control never reverts control

> ✅ **PARTIALLY RESOLVED — PB-OS1 (`scutemob-116`), 2026-07-18.** The
> `UntilEndOfTurn`/`UntilYourNextTurn` half is FIXED: `recompute_object_controller` is now wired
> into `expire_end_of_turn_effects` **and** `expire_until_next_turn_effects`, so those durations
> revert control (CR 514.2 / 611.2b / 613.7). The vacuous `test_gain_control_until_eot_expires` was
> de-vacuoused (asserts control reverts) plus stacked-control + timing tests added. **Roster
> correction**: the `all_cards()` sweep found only **2** in-scope cards — `sarkhan_vol`,
> `zealous_conscripts` — NOT 3. `karrthus_tyrant_of_jund` uses `EffectDuration::Indefinite`, which is
> **correct** (Karrthus grants *permanent* control, CR 611.2a — no "for as long as" clause; the
> Scryfall ruling confirms control "doesn't wear off during the cleanup step" — verified by
> primitive-impl-reviewer), so it is out of scope by design, not a bug. No PROTOCOL/HASH bump.
> **REMAINING (still open, carried forward)**: the `WhileSourceOnBattlefield` gain-control reversion
> half — a *different* removal path (SBA when the source leaves, not the end-of-turn passes) with its
> own reconcile site — was explicitly deferred. Refile/track that half as the surviving OOS-EF9-1.

`Effect::GainControl` writes `obj.controller` imperatively and pushes a Layer-2 `SetController`
continuous effect, but `calculate_characteristics` treats `SetController` as a **no-op** (control
lives on `GameObject`, not `Characteristics`) and there is **no reconcile loop**. The `expire_*`
passes for `UntilEndOfTurn`/`UntilYourNextTurn` `retain` the effect out of `continuous_effects` but
**never touch `obj.controller`**. So a `WhileSourceOnBattlefield` gain-control (before the PB-EF9
flip) and every `UntilEndOfTurn` gain-control **keeps the borrowed permanent under the borrower's
control forever** after the effect should have ended — legal-but-wrong for any such def shipped
`Complete`.
- **Instances**: `sarkhan_vol.rs`, `zealous_conscripts.rs`, `karrthus_tyrant_of_jund.rs` (Threaten-
  style `UntilEndOfTurn` steals — the borrowed creature should return at cleanup and does not).
- **Vacuous test**: `test_gain_control_until_eot_expires` (`primitives/primitive_pb32.rs`) asserts the
  *effect* is removed but NOT that control reverts — it passes while the bug is live.
- **Fix shape (already built by PB-EF9)**: wire `recompute_object_controller` into
  `expire_end_of_turn_effects` (and the until-next-turn pass) for removed `SetController` effects,
  exactly as PB-EF9 does for `WhileYouControlSource`. Deferred here because it changes existing
  Threaten behavior and touches golden scripts/tests — a follow-up micro-PB with the helper in place.

## 12. New finding filed by PB-EF10 (scutemob-111)

### OOS-EF10-1 (capability) — no "look at top N, place one, rest to bottom random" primitive
"Look at the top seven cards of your library. Then you may sacrifice a creature. If you do, you
may put a creature card with mana value X or less from among those cards onto the battlefield...
Put the rest on the bottom of your library in a random order" (Birthing Ritual) has no `Effect`
primitive. `SearchLibrary` searches the **whole library**, not a looked-at top-N subset, and has
no bottom-randomize destination — using it would ship wrong game state (ignores both the top-7
scoping and the bottom-random remainder), which is exactly the class of error W5/W-MISS policy
forbids. All three of Birthing Ritual's OTHER mechanics (end-step trigger, "if you control a
creature" intervening-if, optional sacrifice, runtime MV cap = 1 + sacrificed creature's mana
value) are expressible after PB-EF10 — this dig is the only remaining blocker.
- **Instance**: `birthing_ritual.rs` (authored `inert` — `abilities: vec![]`, since the only
  mechanic the card has is fully gated on this primitive).
- **Fix shape**: a new `Effect::LookAtTopThenPlace { count: EffectAmount, filter: TargetFilter,
  destination: ZoneTarget, rest_to: BottomRandomOrder | Graveyard, optional: bool }` that (a)
  scopes candidates to the looked-at top N (not the whole library, unlike `SearchLibrary`), (b)
  honors a runtime `max_cmc_amount` (already exists on `TargetFilter` as of PB-EF10), (c) places
  at most one matching card, (d) sends the remainder to the bottom in a randomized
  (deterministic-by-ObjectId in M7, non-deterministic in M10+) order. Likely reusable for other
  impulse-style "look at N, take one, rest bottomed" cards beyond Birthing Ritual.
- **Also noted, distinct blocker, not filed as a new seed**: `birthing_pod.rs` needs mana value
  **equal to** 1 + the sacrificed creature's MV, not "N or less" — `TargetFilter.max_cmc_amount`
  is an upper-bound cap and would wrongly accept cheaper creatures too. A paired
  `min_cmc_amount: Option<Box<EffectAmount>>` (same runtime-resolution mechanism) or a dedicated
  exact-match runtime filter would close it; small, but out of this PB's declared scope
  (`Implement-phase default-to-defer`, `memory/conventions.md`).

---

## 13. New seeds filed by PB-OS4 (scutemob-130)

PB-OS4 shipped `Effect::ExileSourceAndReturnTransformed` (return-transformed as a new object —
OOS-EF5-3, narrowed above). During implementation+review two distinct out-of-scope blockers were
surfaced that keep all four OOS-EF5-3 candidate cards from flipping Complete. Both are their own
PBs.

### OOS-OS4-1 (capability) — planeswalker-back-face starting loyalty on enter/return-transformed
A DFC whose **back face is a planeswalker** (nicol_bolas_the_ravager → Nicol Bolas, the Arisen;
grist_voracious_larva → Grist, the Plague Swarm) cannot be authored: `CardFace`
(`card_definition.rs`) has **no `starting_loyalty`** field, and neither the enter-transformed nor
the return-transformed path assigns loyalty counters. Such a card would enter transformed with **0
loyalty** and be put into the graveyard by SBA 704.5i (CR 306.5b / 704.5i) on the next SBA check —
wrong game state. **Fix shape**: add `CardFace.starting_loyalty: Option<u32>` (wire-affecting —
`CardFace` is in the SR-8 closure → PROTOCOL bump) and assign `CounterType::Loyalty` counters in
both the ETB-transformed and return-transformed object-construction paths (CR 306.5b); then author
the two planeswalker back faces (3 loyalty abilities each, some complex). Blocks
`nicol_bolas_the_ravager`, `grist_voracious_larva`. (grist **additionally** needs an
"entered-from-your-graveyard / cast-from-your-graveyard" trigger condition that does not exist in
the DSL — a second, smaller sub-blocker.)

### OOS-OS4-2 (correctness/capability, cross-cutting) — transformed permanents don't gather back-face non-keyword abilities
> ✅ **RESOLVED — PB-OS4b, `scutemob-134`, 2026-07-19.** Ability gathering is now face-aware at every
> battlefield site (two channels: runtime characteristics-vector base-rebuild at the `apply_face_change`
> transform boundary + `CardDefinition::effective_abilities` for `def.abilities` direct-index sites).
> `docent_of_perfection` + `bloodline_keeper` were live-wrong and now stay `Complete` (pinned by
> execution); `growing_rites_of_itlimoc` + `thaumatic_compass` back abilities now function (stay
> partial, unrelated gaps); `fable_of_the_mirror_breaker` Reflection ability now reachable (message
> corrected, stays partial). Wire-neutral (PROTOCOL 19 / HASH 56). Latent roster-unreachable
> enter-transformed *replacement*-gathering gap documented in source. Record:
> `memory/primitives/pb-plan-OS4b.md` / `pb-review-OS4b.md`; queue row in
> `oos-retriage-plan-2026-07-18.md` (PB-OS4b). **The edgar_charmed_groom half is spun out as OOS-OS4-3
> below.**
A permanent showing its **back face** (via in-place `TransformSelf` OR the new return-transformed
path) does **not** use its back face's non-keyword abilities. `register_static_continuous_effects`
(`rules/replacement.rs`), `queue_carddef_etb_triggers` (`rules/replacement.rs`), and the upkeep
triggered-ability scan (`rules/turn_actions.rs`) all iterate the **front** `def.abilities`
unconditionally — only *keywords* consult `back_face` (`rules/layers.rs`). Consequences: (a) a
transformed permanent's back-face static / ETB / upkeep-trigger / activated abilities never
function; (b) the transformed permanent wrongly **retains its front face's** static/ETB abilities
(e.g. a returned Edgar Markov's Coffin — an artifact — wrongly re-registers Edgar's front "Other
Vampires get +1/+1" anthem). **This likely also affects already-shipped PB-EF5 in-place
`TransformSelf` Complete markers** (e.g. any card whose back face has a non-keyword ability —
audit thaumatic_compass / docent_of_perfection back faces). **Fix shape**: make ability gathering
**face-aware** — when `obj.is_transformed`, gather from `def.back_face`'s abilities across all four
call sites (static registration, ETB queue, upkeep/triggered scan, and activated-ability lookup).
Broad blast radius across the general transform machinery; needs its own PB + review (may change
the behavior of already-shipped TransformSelf cards). Blocks `edgar_charmed_groom` (wrong state
without it) and the back-face activated ability of `fable_of_the_mirror_breaker`'s Reflection of
Kiki-Jiki (both authored honestly around the gap by PB-OS4: edgar left unauthored, fable partial
with the blocker named).

### OOS-OS4-3 (capability, micro — filed by PB-OS4b, `scutemob-134`, 2026-07-19) — edgar_charmed_groom return-from-graveyard-transformed
The face-aware-gathering blocker (OOS-OS4-2) that kept `edgar_charmed_groom` unauthored is now
resolved, but the card still cannot ship: it dies and **returns from the graveyard transformed**
(as Edgar Markov's Coffin), and the `Effect` that expresses this —
`Effect::ReturnSourceToBattlefieldTransformed` (return **from graveyard**, distinct from the shipped
`ExileSourceAndReturnTransformed`) — was **removed in the PB-OS4 narrowing** as unusable-before-OS4b.
Re-adding it is a **genuine wire bump** (PROTOCOL 19→20, HASH 56→57: new `Effect` variant + `HashInto`
arm + protocol History/epoch rows + re-pinned fingerprints/`FROZEN_HISTORY_PREFIX_DIGEST`/sentinels).
PB-OS4b deliberately kept its mandatory scope wire-neutral and deferred this rather than bundle an
unrelated bump into a correctness PB (per AC 5040 / one-bump-per-PB discipline). **Fix shape**: re-add
the `Effect` variant (executor + wire) in its own commit + author `edgar_charmed_groom.rs` (front:
Vampire anthem `Static` + `WhenDies → ReturnSourceToBattlefieldTransformed`; back Edgar Markov's
Coffin: `AtBeginningOfYourUpkeep → create lifelink Vampire token + bloodline counter; if ≥3, remove
and transform back`) + a full-lifecycle test (dies → returns as Coffin new object → upkeep makes a
Vampire + counter, Coffin does NOT grant the front anthem → 3 counters transform back → anthem
returns). The back-face upkeep loop now functions thanks to OS4b. CR 306.5b not needed (Edgar's back
is not a planeswalker). Ships `edgar_charmed_groom` **Complete**. Micro-PB, ~1 flip.
