# Primitive Batch Plan: PB-OS11 — FINAL PB-OS batch (RemoveCounter mana-ability lowering + batch filtered-attack trigger)

**Generated**: 2026-07-19
**Primitive(s)**:
1. **RemoveCounter mana-ability lowering** — let a `Cost::RemoveCounter` self-referential counter cost register as a TRUE mana ability (CR 605.1a), not a stack-using activated ability (OOS-LKI-3, reframed).
2. **Batch filtered-attack trigger** — `TriggerCondition::WheneverYouAttack` gains an optional attacker-set filter so "Whenever you attack with one or more [filtered] creatures" fires exactly ONCE per combat when ≥1 declared attacker matches (OOS-TS-1, reframed).
**CR Rules**: 605.1a, 602.2/602.2c, 118.3, 106.12; 508.1, 508.1m, 603.2/603.2c, 205.3, 111.1, 614.1c
**Cards affected**: 4 guaranteed clean flips (`workhorse` NEW, `anim_pakal_thousandth_moon`, `general_kreat_the_boltbringer`, `hermes_overseer_of_elpis`) + up to 2 opportunistic backfill (`gemstone_array`, `druids_repository` — execution-verify)
**Dependencies**: none new. Reuses PB-EF8 (`exile_self_from_hand` lowering template), PB-EWC (`EntersWithCounters`), PB-TS (`TokenSpec.count: EffectAmount`), PB-N (`WheneverCreatureYouControlAttacks{filter}`, `triggering_creature_filter`).
**Deferred items from prior PBs**: none carried forward. `najeela_the_blade_blossom` ("Whenever a Warrior attacks") stays ENGINE-BLOCKED — it needs per-Warrior firing + UntapAll + multi-keyword grant + extra-combat, out of scope.

---

## ⚠️ PREAMBLE — both task premises were STALE; corrected against MCP source

Per project feedback (`feedback_verify_cr_before_implement.md`, `feedback_oversight_primitive_category_not_cards.md`): oversight/plan claims are NOT authoritative; the printed oracle (MCP) and engine source are. Both singletons were reframed after source verification:

- **OOS-LKI-3 was based on OBSOLETE Workhorse oracle text.** The dispatch brief (and the retriage plan §3) describe Workhorse as *"{T}, Sacrifice Workhorse: Add X colorless mana, X = +1/+1 counters"* and prescribe a `SacrificedCreatureLki`-counter-capture engine change. **MCP `lookup_card "Workhorse"` returns the modern errata:** *"This creature enters with four +1/+1 counters on it. / Remove a +1/+1 counter from this creature: Add {C}."* There is **no sacrifice and no X-mana**. The `SacrificedCreatureLki`-counter work is therefore MOOT — no card in the corpus needs it. **OOS-LKI-3 is reframed** to the real, narrower gap Workhorse actually exercises: a `Cost::RemoveCounter` **mana** ability with no `{T}` cannot be lowered to a true mana ability (documented in `druids_repository.rs` / `gemstone_array.rs` known_wrong notes: *"mana_ability_cost_components refuses Cost::RemoveCounter … requires_tap:false path is unexercised"*).

- **OOS-TS-1's re-scope ("add `exclude_subtype` to `TargetFilter`") was ALSO stale.** `TargetFilter.exclude_subtypes: Vec<SubType>` already exists (`card_definition.rs:3132`) and is already enforced in `matches_filter` (`effects/mod.rs:8856`). The REAL surviving gap is the **firing semantics**: Anim Pakal is a *batch* trigger that fires **once** ("with one or more non-Gnome creatures"). `WheneverCreatureYouControlAttacks{filter}` fires **once per matching attacker** (over-triggers — 3 non-Gnome attackers → 3 counters + 3 token batches, wrong). `WheneverYouAttack` fires once but has **no filter**. Neither models the batched-filtered attack. The fix is a **filter on the once-firing batch trigger**.

- **Wire prediction corrected.** The brief expected `PROTOCOL 25→26`. Source says otherwise: `TriggerCondition` is explicitly **outside** the wire closure (protocol.rs v25/PB-OS10 note, lines 229-232), and `ManaAbility` field additions have historically been **HASH-only** (SR-34 v41, PB-EF8 v51 — no PROTOCOL bump). This batch therefore expects **HASH 62→63 only, NO PROTOCOL bump.** Let the machine gates decide (see "Wire" section) — do not bump PROTOCOL preemptively.

**TODO-sweep gate (roster-recall, MANDATORY) — result: 2 forced adds beyond the brief.**
`Grep "WheneverYouAttack"` + attack-batch keywords across `crates/card-defs/src/defs/` surfaced two cards whose source self-identifies as needing exactly this batch-filtered-attack primitive:
- `general_kreat_the_boltbringer.rs:28-32` — *"'Whenever one or more Goblins you control attack' … WheneverCreatureYouControlAttacks fires per-creature (over-triggers). WheneverYouAttack fires once but doesn't check for Goblins … A WheneverOneOrMoreCreaturesWithSubtypeAttack trigger variant is needed."* (`partial`)
- `hermes_overseer_of_elpis.rs:70-73` — *"ENGINE-BLOCKED: 'Whenever you attack with one or more Birds, scry 2.' No once-per-combat batched subtype attack trigger exists … Omitted per W5 policy."* (`partial`)
Both are **forced adds** (verified as needing the primitive), not in the original 2-card brief. `najeela_the_blade_blossom` also names an attack-type-filtered trigger but needs additional primitives → NOT added. `glimmer_lens`, `karlach`, `kazuul`, `grand_warlord_radha`, `clavileno`, `caesar`, `chivalric_alliance`, `mishra`, `legions_landing`, `seasoned_dungeoneer` reference `WheneverYouAttack` but do NOT need the filter (their triggers are genuinely unfiltered "whenever you attack") — they only need the mechanical `{ filter: None }` migration.

---

## CR Rule Text (abridged from MCP; full children consulted)

- **CR 605.1a** — "An activated ability is a mana ability if it meets all of the following criteria: it doesn't require a target …, it could add mana …, and it isn't a loyalty ability." → "Remove a +1/+1 counter: Add {C}" qualifies: no target, adds mana, not loyalty. It MUST be a mana ability (resolves without the stack, usable while paying costs).
- **CR 602.2c / 118.3** — costs are paid on activation; the permanent must have ≥ the required counters before activation (CR 118.3 legality).
- **CR 508.1 / 508.1m / 603.2c** — "Whenever you attack with one or more [X]" is a single batched trigger evaluated at declare-attackers; fires once per combat, not once per attacker.
- **CR 205.3 / 111.1** — the created Gnome tokens are subtype Gnome; per Anim Pakal ruling 2023-11-10 they "were never declared as attacking creatures," so they never re-fire an attack trigger (no inflation).
- **CR 614.1c** — "enters with four +1/+1 counters" is a self-replacement, applied at ETB.

### Card oracle text (verified via MCP `lookup_card`)

- **Workhorse** — `{6}` Artifact Creature — Horse, 0/0. *"This creature enters with four +1/+1 counters on it. / Remove a +1/+1 counter from this creature: Add {C}."*
- **Anim Pakal, Thousandth Moon** — `{1}{R}{W}` Legendary Creature — Human Soldier, 1/2. *"Whenever you attack with one or more non-Gnome creatures, put a +1/+1 counter on Anim Pakal, then create X 1/1 colorless Gnome artifact creature tokens that are tapped and attacking, where X is the number of +1/+1 counters on Anim Pakal."* Rulings: (a) if Anim Pakal is gone at resolution, use LKI counter count; (b) each Gnome enters attacking a player/PW/battle of your choice; (c) Gnomes were never *declared* attackers — attack triggers don't fire for them; (d) Anim Pakal need not be one of the attackers.
- **General Kreat, the Boltbringer** — `{2}{R}` Legendary Creature — Goblin Soldier, 2/2. *"Whenever one or more Goblins you control attack, create a 1/1 red Goblin creature token that's tapped and attacking. / Whenever another creature you control enters, General Kreat deals 1 damage to each opponent."* (second ability already authored `Complete`-quality).
- **Hermes, Overseer of Elpis** — `{3}{U}` Legendary Creature — Elder Wizard, 2/4. *"Whenever you cast a noncreature spell, create a 1/1 blue Bird creature token with flying and vigilance. / Whenever you attack with one or more Birds, scry 2."* (first ability already authored).

---

## PART A — RemoveCounter mana-ability lowering (OOS-LKI-3 reframed)

### A-Change-1: add `remove_counter` to `ManaAbilityCost` (harness-internal accumulator)
**File**: `crates/engine/src/testing/replay_harness.rs`
**Action**: add `remove_counter: Option<(CounterType, u32)>` field to the private `struct ManaAbilityCost` (L3771-3777) and to its initializer (L3854-3860, `remove_counter: None`).

### A-Change-2: accept `Cost::RemoveCounter` in `mana_ability_cost_components`
**File**: `crates/engine/src/testing/replay_harness.rs` (`fn mana_ability_cost_components`, L3807)
**Action**: move `Cost::RemoveCounter { .. }` OUT of the "not lowerable" fall-through arm (L3844-3851) into an accepting arm:
```rust
Cost::RemoveCounter { counter, count } => {
    if acc.remove_counter.is_some() { return false; } // one counter-cost component only
    acc.remove_counter = Some((counter.clone(), *count));
    true
}
```
**Rationale (CR 605.1a / CR 602.2c)**: `Cost::RemoveCounter` is documented (card_definition.rs:1269) as removing counters **from the source permanent** — it is self-referential, identified by `Command::TapForMana`'s `source` ObjectId, exactly like the already-accepted `SacrificeSelf` / `ExileSelfFromHand`. The old comment claiming it "needs a caller-supplied ObjectId" was imprecise; it does not.

### A-Change-3: relax the no-tap guard for a remove-counter cost
**File**: `crates/engine/src/testing/replay_harness.rs` (`mana_ability_cost_components`, L3873)
**Action**: change the guard to also permit a no-tap cost when `remove_counter` is present:
```rust
if !acc.requires_tap && !acc.exile_self_from_hand && acc.remove_counter.is_none() {
    return None;
}
```
**Rationale (SR-34 Finding 2)**: the no-tap guard exists to bar *free, repeatable, stackless* mana. A remove-counter cost is a genuine per-activation resource cost bounded by counters present (each activation consumes one) — it is self-exhausting exactly like `exile_self_from_hand` (PB-EF8's precedent for this relaxation). Not free, not an infinite loop. Document this alongside the existing PB-EF8 relaxation comment.

### A-Change-4: carry `remove_counter` through `mana_ability_lowering`
**File**: `crates/engine/src/testing/replay_harness.rs` (`fn mana_ability_lowering`, L3899)
**Action**: after `ma.exile_self_from_hand = components.exile_self_from_hand;` (L3915) add `ma.remove_counter = components.remove_counter.clone();`.

### A-Change-5: add `remove_counter` field to `ManaAbility`
**File**: `crates/card-types/src/state/game_object.rs` (`pub struct ManaAbility`, L172-245)
**Action**: add, mirroring the `exile_self_from_hand` field (L238-244):
```rust
/// PB-OS11 (CR 605.1a / CR 602.2c): if `Some((counter, n))`, activating this
/// mana ability removes `n` counters of that type from the source permanent as
/// its activation cost (the source stays on the battlefield — no zone move,
/// unlike `sacrifice_self`). Self-referential (source = the TapForMana source
/// ObjectId). `handle_tap_for_mana` validates ≥ n present (CR 118.3) and pays
/// it. Lowered from `Cost::RemoveCounter` by `mana_ability_lowering`. `None`
/// for the overwhelming majority. Relaxes the no-tap guard (self-exhausting,
/// like `exile_self_from_hand`). Workhorse: `Some((PlusOnePlusOne, 1))`.
#[serde(default)]
pub remove_counter: Option<(crate::state::types::CounterType, u32)>,
```
Struct-literal construction sites in `replay_harness.rs` (`try_as_tap_mana_ability`, `mana_pool_to_ability`) all use `..Default::default()` → no change needed there. `ManaAbility::tap_for` / `::treasure` likewise. Compiler flags any exhaustive full-field literal.

### A-Change-6: pay the counter cost in `handle_tap_for_mana`
**File**: `crates/engine/src/rules/mana.rs` (`fn handle_tap_for_mana`, L37)
**Action**: insert a new cost-payment step **between** the life-cost payment (ends L305) and the SR-28 pre-cost snapshot (L321-334) — i.e. it pays before mana is produced (CR 602.2c) and before any sacrifice/exile cost, and (like mana/life) it does not move the source. Mirror the activated-path payment already at `abilities.rs:1176-1201`:
```rust
// PB-OS11 (CR 602.2c / CR 118.3): pay a remove-counter cost for a mana ability
// (Workhorse "Remove a +1/+1 counter: Add {C}"). Self-referential; no zone move.
if let Some((ref counter, count)) = ability.remove_counter {
    let obj_ref = state.object(source)?;
    let current = obj_ref.counters.get(counter).copied().unwrap_or(0);
    if current < count {
        return Err(GameStateError::InvalidCommand(format!(
            "mana ability requires removing {count} {counter:?} counter(s) but only {current} present (CR 118.3)"
        )));
    }
    let obj_mut = state.object_mut(source)?;
    // decrement / remove-key at zero, matching the activated-path semantics
    ...
    events.push(GameEvent::CounterRemoved { object_id: source, counter: counter.clone(), count });
}
```
Add the same legality pre-check to the **pure-validation** region (alongside the `mana_cost`/`life_cost` checks at L213-234) so an unaffordable activation touches nothing (transaction cleanliness, matching SR-34's pattern). Reuse the **existing** `GameEvent::CounterRemoved { object_id, counter, count }` (events.rs:504) — no new GameEvent variant.
**Note**: removing the last +1/+1 counter makes Workhorse 0/0 → dies via SBA after the ability resolves; the mana is still produced (cost paid, ability resolved). This is correct and is a required test.

### A-Change-7: HashInto arm for the new field
**File**: `crates/engine/src/state/hash.rs` (`impl HashInto for ManaAbility`, L1779-1800)
**Action**: add `self.remove_counter.hash_into(hasher);` after `self.exile_self_from_hand.hash_into(hasher);` (L1798). Add a HASH-history entry (see Wire section). `Option<(CounterType, u32)>` — confirm `CounterType` implements `HashInto` (it does — used throughout); tuple `HashInto` may need the elements hashed individually if no tuple impl exists (hash each: `if let Some((c, n)) = &self.remove_counter { c.hash_into(hasher); n.hash_into(hasher); }` guarded, else a discriminant marker).

### A-Card-1 (NEW): `crates/card-defs/src/defs/workhorse.rs`
Create the file (does not exist today) and register it in the defs module list (the `mod`/registry the sibling defs use — compiler/registry gate will flag if missed).
```
name "Workhorse", cid("workhorse"), mana_cost {6}, types [Artifact, Creature] subtypes ["Horse"],
power Some(0), toughness Some(0), oracle_text (exact MCP text),
abilities: [
  // CR 614.1c: enters with four +1/+1 counters (self-replacement, fixed count).
  AbilityDefinition::Replacement {
      trigger: ReplacementTrigger::WouldEnterBattlefield { filter: ObjectFilter::Any },
      modification: ReplacementModification::EntersWithCounters {
          counter: CounterType::PlusOnePlusOne,
          count: Box::new(EffectAmount::Fixed(4)),
      },
      is_self: true, unless_condition: None,
  },
  // CR 605.1a: mana ability — Remove a +1/+1 counter: Add {C}. No tap.
  AbilityDefinition::Activated {
      cost: Cost::RemoveCounter { counter: CounterType::PlusOnePlusOne, count: 1 },
      effect: Effect::AddMana { player: PlayerTarget::Controller, mana: mana_pool(0,0,0,0,0,1) }, // {C}
      timing_restriction: None, targets: vec![], activation_condition: None,
      activation_zone: None, once_per_turn: false, modes: None,
  },
],
completeness: Completeness::Complete,   // execution-verified below
```
Pattern reference: `eomer_king_of_rohan.rs:35-53` (EntersWithCounters self-replacement) + `gemstone_array.rs`/`druids_repository.rs` (RemoveCounter mana structure, minus their any-color bug). After A-Change-2/3/4, `enrich_spec_from_def`'s `mana_ability_lowering` lowers this Activated ability into a true `ManaAbility { produces:{Colorless:1}, requires_tap:false, remove_counter:Some((PlusOnePlusOne,1)) }`. **Workhorse produces fixed `{C}`, so it does NOT touch the `AddManaAnyColor` color bug** that keeps gemstone/druids `known_wrong` — this is the clean flip.

### A-Backfill (execution-verify, opportunistic): `gemstone_array`, `druids_repository`
A-Change-2/3 will ALSO lower these two cards' `Cost::RemoveCounter` (no-tap) + `Effect::AddManaAnyColor` activated abilities into mana abilities. On the lowered path, `handle_tap_for_mana`'s `any_color` branch (PB-EF12) produces the **chosen** color — which would FIX their documented color bug (they currently add colorless). **Runner protocol (SR-34/36 — probe by execution, not source):** after the engine change, write an executing test tapping each for a chosen non-colorless color; if it produces the chosen color AND the counter is removed AND no golden script that `ActivateAbility`'d them breaks (they move from `activated_abilities` to `mana_abilities`, so any script must switch to `TapForMana`), flip both `known_wrong → Complete` and update their notes. If EITHER fails, keep `known_wrong`, revert their note to reflect the new lowered-path reason, and file an OOS seed. `crucible_of_the_spirit_dragon` stays `partial` (its payoff is "Remove **X** storage counters" — dynamic count, still inexpressible with fixed-`u32` `Cost::RemoveCounter`; unaffected — it does not even author the RemoveCounter ability). `golgari_grave_troll` is unaffected (its RemoveCounter cost feeds `Effect::Regenerate`, not mana → `try_as_tap_mana_ability` returns `None`, stays activated).
**Runner MUST** `Grep "Cost::RemoveCounter" crates/card-defs/src/defs/` and confirm the full swept set: any def pairing `Cost::RemoveCounter` (self, no other blocking component) with `Effect::AddMana*` will newly lower. Verify each by execution; the 17 files listed mostly pair RemoveCounter with non-mana effects and are unaffected.

---

## PART B — batch filtered-attack trigger (OOS-TS-1 reframed)

### B-Change-1: `WheneverYouAttack` unit → struct variant
**File**: `crates/card-types/src/cards/card_definition.rs` (`enum TriggerCondition`, `WheneverYouAttack` at L3557)
**Action**: change to
```rust
WheneverYouAttack {
    /// PB-OS11: optional filter on the DECLARED ATTACKER SET (layer-resolved
    /// characteristics + is_token/is_nontoken). The trigger fires ONCE per combat
    /// iff at least one attacker controlled by the trigger's controller matches.
    /// `None` = any attack (legacy behaviour). Used for "Whenever you attack with
    /// one or more non-Gnome creatures / Goblins / Birds" (CR 508.1/508.1m/603.2c).
    /// Distinct from `WheneverCreatureYouControlAttacks{filter}` which fires per
    /// matching attacker.
    #[serde(default)]
    filter: Option<TargetFilter>,
},
```
Mirrors the unit→struct migration `WheneverCreatureYouControlAttacks` already underwent (see replay_harness.rs:2871 comment).

### B-Change-2: map the filter to `triggering_creature_filter`
**File**: `crates/engine/src/testing/replay_harness.rs` (the "Whenever you attack" conversion, L3168-3189)
**Action**: change the match pattern from `trigger_condition: TriggerCondition::WheneverYouAttack,` (unit, L3171) to `trigger_condition: TriggerCondition::WheneverYouAttack { filter },` and set `triggering_creature_filter: filter.clone(),` (currently hardcoded `None`, L3188). Reuse the existing runtime field — no new `TriggeredAbilityDef` field.

### B-Change-3: apply the filter on the `ControllerAttacks` firing path (the core engine fix)
**File**: `crates/engine/src/rules/abilities.rs` (`fn collect_triggers_for_event`, the trigger-matching loop at L6404; the `ControllerAttacks` event is dispatched from the `AttackersDeclared` handler at L4147-4170)
**Action**: add a branch, parallel to the `AnyCreatureYouControlAttacks` block (L6413-6471), for `TriggerEvent::ControllerAttacks`:
```rust
if event_type == TriggerEvent::ControllerAttacks {
    if let Some(ref filter) = trigger_def.triggering_creature_filter {
        // Fire ONCE iff ≥1 declared attacker controlled by this trigger's
        // controller matches the filter (CR 508.1m/603.2c). `obj` is the
        // trigger source; `obj.controller` is "you".
        let any_match = state.combat.attackers.keys().any(|aid| {
            let ao = match state.objects.get(aid) { Some(o) => o, None => return false };
            if ao.controller != obj.controller { return false; }
            // is_token / is_nontoken are GameObject runtime fields (matches_filter
            // can't see them) — guard explicitly, mirroring L6440/6456.
            if filter.is_token && !ao.is_token { return false; }
            if filter.is_nontoken && ao.is_token { return false; }
            let ac = crate::rules::layers::expect_characteristics(state, *aid);
            crate::effects::matches_filter(&ac, filter)   // honors has_subtype, exclude_subtypes, etc.
        });
        if !any_match { continue; }
    }
}
```
**Precondition to verify (runner)**: `state.combat.attackers` (an `OrdMap<ObjectId, AttackTarget>`, combat.rs:30) is populated at the moment `ControllerAttacks` triggers are collected. It is — the sibling `AnyCreatureYouControlAttacks` dispatch in the same `AttackersDeclared` block operates on live battlefield attackers, and combat is set up before `AttackersDeclared` is emitted. If a probe shows it empty, fall back to threading the attacker set through `collect_triggers_for_event`. Confirm by execution.
**Once-firing invariant**: `ControllerAttacks` is already dispatched once per controller-source (not per attacker) at L4161-4168, so adding a skip-if-no-match keeps the "fires once" semantic. `exclude_subtypes:[Gnome]` (Anim Pakal) and `has_subtype:Goblin/Bird` (Kreat/Hermes) are all honored by `matches_filter`.

### B-Change-4: HashInto arm for the reshaped variant
**File**: `crates/engine/src/state/hash.rs` (`impl HashInto for TriggerCondition`)
**Action**: the `WheneverYouAttack` arm must now hash the `filter` field (was a bare unit discriminant). Update the arm; add the HASH-history note.

### B-Change-5: migrate all bare-unit construction sites (compiler-enforced, exhaustive)
Every `TriggerCondition::WheneverYouAttack` **construction** becomes `WheneverYouAttack { filter: None }` EXCEPT the three roster cards (which get real filters, Part-B cards below). Known constructing card defs (from grep — compiler will flag any missed):
`legions_landing.rs:74`, `caesar_legions_emperor.rs:38`, `mishra_claimed_by_gix.rs:32`, `chivalric_alliance.rs:24`, `seasoned_dungeoneer.rs:55`, plus `grand_warlord_radha.rs` if it constructs (verify). Comment-only references (`karlach`, `kazuul`, `glimmer_lens`, `clavileno`) need no change. Also update any engine/test construction/match sites: `crates/engine/tests/rules/trigger_variants.rs`, `crates/engine/tests/primitives/pb_os6_dfc_flip_conditions.rs`. **No `tools/` (TUI / replay-viewer) sites** — `TriggerCondition` is not in their exhaustive `StackObjectKind`/`KeywordAbility` matches (grep confirmed 0 hits under `tools/`).

### B-Card-1: `crates/card-defs/src/defs/anim_pakal_thousandth_moon.rs` (rewrite `known_wrong` → `Complete`)
Replace the single unfiltered trigger with:
```rust
AbilityDefinition::Triggered {
    once_per_turn: false,
    trigger_condition: TriggerCondition::WheneverYouAttack {
        filter: Some(TargetFilter {
            exclude_subtypes: vec![SubType("Gnome".to_string())],
            ..Default::default()
        }),
    },
    effect: Effect::Sequence(vec![
        // "put a +1/+1 counter on Anim Pakal, ..."
        Effect::AddCounter {
            target: EffectTarget::Source,
            counter: CounterType::PlusOnePlusOne,
            count: 1,
        },
        // "... then create X 1/1 colorless Gnome artifact creature tokens tapped
        //  and attacking, where X = number of +1/+1 counters on Anim Pakal"
        //  (evaluated AFTER the counter is added → post-increment count).
        Effect::CreateToken {
            spec: TokenSpec {
                name: "Gnome".to_string(),
                card_types: [CardType::Artifact, CardType::Creature].into_iter().collect(),
                subtypes: [SubType("Gnome".to_string())].into_iter().collect(),
                colors: imbl::OrdSet::new(),   // colorless
                power: 1, toughness: 1,
                count: EffectAmount::CounterCount {
                    target: EffectTarget::Source,
                    counter: CounterType::PlusOnePlusOne,
                },
                tapped: true,
                enters_attacking: true,
                ..Default::default()
            },
        },
    ]),
    intervening_if: None, targets: vec![], modes: None, trigger_zone: None,
},
```
`completeness: Completeness::Complete`.
**Filter rationale (oracle-reconciled)**: oracle is "non-Gnome creatures" → `exclude_subtypes:[Gnome]` ALONE is correct; `is_nontoken` is NOT in the oracle and would be redundant (the created Gnome tokens self-exclude as Gnomes). **Decoy/no-inflation** is satisfied structurally: the Gnome tokens *enter* attacking (ruling 2023-11-10) and are never *declared* attackers, so no `AttackersDeclared`/`ControllerAttacks` event fires for them (verify by execution — Test B4). **Known minor edge (documented, non-blocking)**: ruling (a) says if Anim Pakal leaves before resolution, use LKI counter count; `CounterCount{Source}` reads live counters. In the normal single-trigger resolution Anim Pakal is present, so the count is correct; the mid-resolution-removal LKI case is an accepted deviation (no non-leaves-trigger LKI counter reader today — `CounterCountAtLastKnownInformation` is scoped to leaves-battlefield triggers). Flag for the reviewer; do NOT block `Complete` on it (consistent with corpus precedent for mid-resolution source removal).

### B-Card-2: `crates/card-defs/src/defs/general_kreat_the_boltbringer.rs` (`partial` → `Complete`)
Add the first ability (keep the existing creature-ETB→damage ability). New attack half:
```rust
AbilityDefinition::Triggered {
    once_per_turn: false,
    trigger_condition: TriggerCondition::WheneverYouAttack {
        filter: Some(TargetFilter {
            has_subtype: Some(SubType("Goblin".to_string())),
            controller: TargetController::You,
            ..Default::default()
        }),
    },
    effect: Effect::CreateToken { spec: TokenSpec {
        name: "Goblin".to_string(),
        card_types: [CardType::Creature].into_iter().collect(),
        subtypes: [SubType("Goblin".to_string())].into_iter().collect(),
        colors: [Color::Red].into_iter().collect(),
        power: 1, toughness: 1,
        count: EffectAmount::Fixed(1),
        tapped: true, enters_attacking: true,
        ..Default::default()
    }},
    intervening_if: None, targets: vec![], modes: None, trigger_zone: None,
},
```
Remove the TODO block; `completeness: Completeness::Complete`.

### B-Card-3: `crates/card-defs/src/defs/hermes_overseer_of_elpis.rs` (`partial` → `Complete`)
Add the second ability (keep the existing cast-noncreature→Bird ability):
```rust
AbilityDefinition::Triggered {
    once_per_turn: false,
    trigger_condition: TriggerCondition::WheneverYouAttack {
        filter: Some(TargetFilter {
            has_subtype: Some(SubType("Bird".to_string())),
            controller: TargetController::You,
            ..Default::default()
        }),
    },
    effect: Effect::Scry { player: PlayerTarget::Controller, amount: EffectAmount::Fixed(2) }, // verify Scry signature
    intervening_if: None, targets: vec![], modes: None, trigger_zone: None,
},
```
Remove the ENGINE-BLOCKED note; `completeness: Completeness::Complete`. (Runner: confirm the exact `Effect::Scry` field names against an existing Scry card.)

---

## Wire (SR-8) — expected HASH-only, NO PROTOCOL bump

**HASH 62 → 63 (certain).** Two `HashInto` bodies gain shape:
- `ManaAbility` gains `remove_counter: Option<(CounterType, u32)>` (A-Change-5/7).
- `TriggerCondition::WheneverYouAttack` unit → struct-with-`filter` (B-Change-1/4).
Add ONE combined HASH-history entry in `state/hash.rs` above `pub const HASH_SCHEMA_VERSION: u8 = 63;`, in the established `- NN: PB-OS11 (2026-07-19) — …` format (see the `- 51:`/`- 41:` ManaAbility entries and the `- 28:` TriggerCondition entry for wording). `#[serde(default)]` on both new shapes keeps pre-bump serialized states/defs deserializing (remove_counter → None; filter → None).

**PROTOCOL — expect NO bump; let the machine gate decide.** Evidence both changed types are OUTSIDE the wire-frame closure:
- `TriggerCondition` is explicitly outside the closure — protocol.rs v25/PB-OS10 note (L229-232): *"TriggerEvent/TriggerCondition … neither is in the wire closure — that half of this batch is a HASH-only change."*
- `ManaAbility` field additions have been HASH-only historically: SR-34 (HASH v41, `mana_cost`/`life_cost`) and PB-EF8 (HASH v51, `exile_self_from_hand`) added ManaAbility fields with **no** PROTOCOL entry. ManaAbility lives in `GameState`/`Characteristics`, not in any `Command`/`GameEvent`/`ReplayLog`.
- `GameEvent::CounterRemoved` is **reused** (already in the closure; no shape change).
**Runner action**: after implementing, run `cargo test -p mtg-engine --test protocol_schema` (or the protocol fingerprint test). It is EXPECTED TO PASS UNCHANGED (PROTOCOL stays 25, fingerprint unchanged). Only if it FAILS does a wire-closure type actually move — in that case re-pin `PROTOCOL_SCHEMA_FINGERPRINT`, bump `PROTOCOL_VERSION 25→26`, and append a PROTOCOL-history row naming exactly which type moved. Do NOT bump PROTOCOL preemptively (corrects the dispatch brief's `25→26` assumption).

---

## Exhaustive match / construction sites (the #1 compile-error source)

| File | Symbol | Action |
|------|--------|--------|
| `crates/card-types/src/cards/card_definition.rs` | `enum TriggerCondition::WheneverYouAttack` | unit → struct `{ filter }` (B-Change-1) |
| `crates/card-types/src/state/game_object.rs` | `struct ManaAbility` | add `remove_counter` field (A-Change-5) |
| `crates/engine/src/state/hash.rs` | `HashInto for ManaAbility` | hash `remove_counter` (A-Change-7) |
| `crates/engine/src/state/hash.rs` | `HashInto for TriggerCondition` (WheneverYouAttack arm) | hash `filter` (B-Change-4) |
| `crates/engine/src/state/hash.rs` | `HASH_SCHEMA_VERSION` + history | 62→63, combined entry |
| `crates/engine/src/testing/replay_harness.rs` | `struct ManaAbilityCost` + init | add `remove_counter` (A-Change-1) |
| `crates/engine/src/testing/replay_harness.rs` | `mana_ability_cost_components` | accept RemoveCounter + relax no-tap (A-Change-2/3) |
| `crates/engine/src/testing/replay_harness.rs` | `mana_ability_lowering` | carry field (A-Change-4) |
| `crates/engine/src/testing/replay_harness.rs` | `WheneverYouAttack` conversion (L3171) | destructure `{ filter }`, set `triggering_creature_filter` (B-Change-2) |
| `crates/engine/src/rules/mana.rs` | `handle_tap_for_mana` | remove-counter payment step (A-Change-6) |
| `crates/engine/src/rules/abilities.rs` | `collect_triggers_for_event` | ControllerAttacks filter branch (B-Change-3) |
| card defs (bare-unit `WheneverYouAttack`) | `legions_landing`, `caesar_legions_emperor`, `mishra_claimed_by_gix`, `chivalric_alliance`, `seasoned_dungeoneer`, (`grand_warlord_radha`?) | `{ filter: None }` (B-Change-5) |
| tests | `rules/trigger_variants.rs`, `primitives/pb_os6_dfc_flip_conditions.rs` | `{ filter: None }` at any construction/match |
| defs module/registry | `workhorse.rs` new file | register in the defs list (A-Card-1) |

`cargo build --workspace` after the engine edits catches every remaining exhaustive site (per the standing gotcha; the runner MUST run it, not just `cargo check -p mtg-engine`).

---

## Unit Tests

**Part A — `crates/engine/tests/` mana suite** (pattern: existing `handle_tap_for_mana` / Spirit Guide no-tap tests; SR-34/36 mana-lowering tests):
- `test_workhorse_enters_with_four_counters` — Workhorse ETB → 4 +1/+1 counters, layer-resolved P/T = 4/4 (CR 614.1c).
- `test_workhorse_remove_counter_adds_colorless` — `TapForMana` (no `{T}`, no chosen_color): +1 `{C}` to pool, one +1/+1 counter removed, Workhorse now 3/3. Proves the lowered no-tap remove-counter mana-ability path (CR 605.1a).
- `test_workhorse_activatable_while_paying` / lowered-as-mana-ability assertion — assert Workhorse's ability is in `mana_abilities` (lowered), NOT `activated_abilities` (CR 605.3b: no stack). Mirrors SR-34 lowering assertions.
- `test_workhorse_last_counter_removal_then_dies` — from 1 counter: remove → mana still added, Workhorse 0/0 → SBA death after resolution.
- `test_workhorse_insufficient_counters_rejected` — 0 counters: `TapForMana` errors (CR 118.3), state untouched.
- `test_remove_counter_mana_ability_amount_reads_source_counters` — (later-added-counters-count analogue) add extra +1/+1 counters via another effect, then remove-for-mana still works and counts against the current total; different-permanent counters do NOT affect Workhorse's payment (self-referential source).
- `test_gemstone_array_any_color_lowered` / `test_druids_repository_any_color_lowered` — (backfill, execution-verify) `TapForMana` with `chosen_color: Some(Green)` produces green (not colorless), counter removed. Drives the flip decision (Complete vs keep known_wrong).

**Part B — `crates/engine/tests/rules/trigger_variants.rs`** (pattern: existing attack-trigger tests + `WheneverCreatureYouControlAttacks{filter}` tests):
- `test_you_attack_filter_none_fires_on_any_attack` — legacy `{ filter: None }` unchanged (regression guard for the 5 migrated cards).
- `test_anim_pakal_gnome_only_attack_does_not_fire` — attack with ONLY a Gnome creature → trigger does NOT fire (no counter, no tokens). Negative case for `exclude_subtypes`.
- `test_anim_pakal_nongnome_attack_fires_once` — attack with 1 non-Gnome → fires ONCE: +1 counter, then create 1 Gnome token (count = post-increment counter total). CR 508.1m.
- `test_anim_pakal_multiple_nongnome_attackers_fires_once` — attack with 3 non-Gnome creatures → fires EXACTLY ONCE (not 3×): +1 counter, 1 token batch = current counters. The batch-vs-per-creature correctness core.
- `test_anim_pakal_token_count_scales_with_counters` — successive combats: counters 0→1 (1 token), 1→2 (2 tokens), proving `CounterCount{Source}` reads post-increment.
- `test_anim_pakal_created_gnomes_do_not_inflate_next_trigger` — the DECOY test: created Gnome tokens enter attacking but are NOT declared attackers → they neither re-fire Anim Pakal's own trigger that combat NOR are counted; next combat's non-Gnome attack still fires exactly once (ruling 2023-11-10).
- `test_general_kreat_goblin_attack_fires_once` / `test_general_kreat_no_goblin_attack_does_not_fire` — has_subtype filter, once-firing (Goblin present vs absent).
- `test_hermes_bird_attack_scry_2` / `test_hermes_no_bird_attack_no_scry` — has_subtype filter, once-firing.

**Every test cites its CR section** (invariant #8). Backfill/flip decisions for gemstone/druids follow the SR-34/36 "probe by execution" guardrail — a passing executing test is required before any `known_wrong → Complete` flip.

---

## Verification Checklist

- [ ] `cargo build --workspace` clean (catches all exhaustive TriggerCondition + ManaAbility sites — MUST run, not just `cargo check`)
- [ ] A-Change-1..7 applied; Workhorse authored `Complete`, executes as a lowered no-tap remove-counter mana ability
- [ ] B-Change-1..5 applied; anim_pakal / general_kreat / hermes authored `Complete`
- [ ] gemstone_array / druids_repository execution-verified (flipped to Complete OR kept known_wrong with updated note + seed)
- [ ] HASH 62→63 bumped with combined history entry; `state::hash` gate green
- [ ] `protocol_schema` fingerprint test run — PASSES UNCHANGED (PROTOCOL stays 25) OR, if it fails, PROTOCOL 25→26 + fingerprint re-pin + history row
- [ ] All 5 migrated bare-unit `WheneverYouAttack` defs compile with `{ filter: None }`; their behavior unchanged (regression test)
- [ ] `cargo test --all` passes; `cargo clippy -- -D warnings` clean
- [ ] `cargo fmt --check` AND `tools/check-defs-fmt.sh` (SR-35 — new workhorse.rs def)
- [ ] No remaining TODO/ENGINE-BLOCKED in the 3 flipped card defs
- [ ] `validate_deck` accepts all flipped defs (invariant #9 — must be `Complete`, no unmarked partial)

---

## Risks & Edge Cases

- **Backfill blast radius (A).** Accepting `Cost::RemoveCounter` in mana-lowering re-classifies EVERY self, no-blocking-component `RemoveCounter`+mana ability from activated → mana ability. Confirmed relevant set: gemstone_array, druids_repository (any-color, may flip). Any golden script that `ActivateAbility`'d these breaks — must switch to `TapForMana`. Runner greps the full `Cost::RemoveCounter` set and execution-verifies each; keep known_wrong + seed on any regression rather than force a flip.
- **No-tap guard safety (A).** Relaxing for `remove_counter` is safe only because the cost is self-exhausting (bounded by counters). Do NOT relax for any other cost. Document alongside the PB-EF8 relaxation comment.
- **`state.combat.attackers` availability (B).** The ControllerAttacks filter branch reads live combat state at collect-time. Verify populated by execution (Test B); fallback: thread the attacker set into `collect_triggers_for_event`.
- **Batch-once semantics (B).** The core correctness point: 3 matching attackers must fire ONCE. `WheneverCreatureYouControlAttacks{filter}` is the WRONG (per-creature) trigger — do not use it. Pinned by `test_anim_pakal_multiple_nongnome_attackers_fires_once`.
- **Anim Pakal LKI edge.** `CounterCount{Source}` vs the "use LKI if Anim Pakal left" ruling — accepted minor deviation, flagged for reviewer, non-blocking (no non-leaves LKI counter reader exists).
- **Token no-inflation (B).** Relies on enters-attacking tokens not emitting `AttackersDeclared`. Pinned by the decoy test; if the engine DOES emit a declare event for entering-attacking tokens, that is a separate pre-existing bug to file, not patch here.
- **Scry signature (B-Card-3).** Confirm `Effect::Scry` field names against an existing Scry card before authoring Hermes.
