---
name: EF Batch Plan (2026-07-17)
description: Consolidated, deduped, correctness-first PB batch plan for the 20 engine findings filed by the W-PB2 / W-EMPTY / W-MISS authoring waves + EF-13.
type: plan
---

# EF Batch Plan — 2026-07-17 (task scutemob-98)

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
| EF-W-MISS-3 | MED | granted keyword-triggers (Melee/Battle Cry/Annihilator via `AddKeyword`) are silent no-ops (static keywords grant fine; only trigger-bearing keywords) | No — no Complete def grants a trigger-keyword to others yet |
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

### PB-EF3b — granted keyword-triggers fire  ·  CORRECTNESS
- **Findings**: EF-W-MISS-3.
- **Fix**: synthesize the keyword-derived triggered ability (Melee / Battle Cry / Annihilator)
  when a keyword is added by a continuous effect, not only from **printed** keywords in
  `builder.rs`. Today `LayerModification::AddKeyword` inserts into `keywords` but the derived
  trigger is never built, so an anthem granting a trigger-keyword to *other* creatures registers
  the keyword and the trigger silently never fires (static keywords like flying/haste grant fine).
- **Candidates (2)**: Adriana, Skyhunter Strike Force (Lieutenant grants).
- **Discounted ship**: **~2.** Small correctness fix; likely no schema bump (runtime synthesis,
  no new DSL type). Sequenced in the correctness group (labeled `3b` to keep the later
  numbering + cross-refs stable — it runs before the capability batches below).

### PB-EF4 — TriggeringCreature as effect subject/source  ·  capability (Cluster B)
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
| **PB-EF1** ⭐ | correctness | PB2-1, EMPTY-1, MISS-2 (+EF-4/5, OOS-TS-2) | ~4–5 | none |
| PB-EF2 | correctness | MISS-1 | ~2 | PROTOCOL+HASH |
| PB-EF3 | correctness+cap | MISS-10, MISS-4 | ~5–6 | PROTOCOL (MISS-4) |
| PB-EF3b | correctness | MISS-3 | ~2 | none |
| PB-EF4 | capability | PB2-6≡MISS-5, PB2-7 | ~4–5 | PROTOCOL |
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

## 3. EF-13 — options for the coordinator (decision left to user)

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
