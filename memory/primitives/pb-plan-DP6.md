# Primitive Batch Plan: PB-DP6 — DP-15, intervening-if at queue time (CR 603.4)

**Generated**: 2026-07-26
**Task**: `scutemob-154` · branch `feat/pb-dp6-intervening-if-not-checked-at-queue-time-false-positi`
**Primitive**: a shared queue-time intervening-if gate for card-def (`Option<Condition>`)
triggered abilities, applied at every trigger-queue site that has the field in hand.
**CR Rules**: 603.4 (primary), 603.2, 603.2g, 603.3, 603.3b/d, 613.1d/f, 712.8d/e (face)
**Wire expectation**: **PROTOCOL 27 / HASH 64 UNCHANGED** — see §6.
**Dependencies**: PB-OS4b / PB-RS4 (face-aware ability gathering), PB-EF3 (CardDefETB index
namespace), PB-RS3 (`begin_combat` sweep). All shipped.
**Predicted card yield**: **11 defs change behaviour** (9 stop over-firing, **2 start firing
at all** — see §5). **0 completeness-marker flips.**

---

## 0. Reading order for the runner

1. §1 — which mechanism DP-15 is about (the pre-survey's §A hypothesis is **half right**).
2. §2 — the partitioned 82-site inventory. **This is the load-bearing deliverable.**
3. §3 — the fix shape (one helper + one exhaustive predicate).
4. §4 — per-site defaults (hard constraint 3).
5. §5 — card-def table + caveat keep/clear table (criterion 5536).
6. §6 — wire gate. §7 — tests. §8 — falsified pre-survey bullets. §9 — seeds.

Do **not** start coding before reading §3 and §4 together: the *default* is the part of this
PB that can make the engine worse than the status quo.

---

## 1. Which mechanism DP-15 is about

There are **two** independent intervening-if mechanisms. The pre-survey's §A reading is
**confirmed in structure and wrong in three details**.

### Mechanism 1 — card-def `Option<Condition>` (**this is DP-15**)

- Declared: `AbilityDefinition::Triggered { intervening_if: Option<Condition>, .. }` —
  `crates/card-types/src/cards/card_definition.rs:342`.
- Evaluated by `crate::effects::check_condition` — `crates/engine/src/effects/mod.rs:8879`.
- **Queue-time gates that exist today: exactly two and a half.**
  - `rules/replacement.rs:1834-1839` — the ETB (`WhenEntersBattlefield`) arm. *(The audit's
    cite `1446-1456` is stale.)*
  - `rules/abilities.rs:6949-6955` — the graveyard-zone (`TriggerZone::Graveyard`) sweep.
    *(The audit's cite `6910-6916` is stale — that range is the landfall filter.)*
  - `rules/replacement.rs:1862-1874` — a **hardcoded** `TributeNotPaid` gate that does not
    consult `intervening_if` at all (it reads `tribute_was_paid`). Half a gate.
- **Resolution-time re-check: `rules/resolution.rs:2139-2156`** (the audit's `2119-2135` is
  ~20 lines stale). This is correct and **must be retained** (hard constraint 2).

### Mechanism 2 — runtime `Option<InterveningIf>` (**not DP-15; already well covered**)

- Declared: `TriggeredAbilityDef { intervening_if: Option<InterveningIf> }` —
  `crates/card-types/src/state/game_object.rs:889`. The 2-variant enum
  (`ControllerLifeAtLeast`, `SourceHadNoCounterOfType`) is at `game_object.rs:816-827`.
  *(The pre-survey said the struct is `TriggeredAbility` at `:817-827`; `:817-827` is the
  enum, the struct field is `:889`.)*
- Evaluated by `rules/abilities.rs::check_intervening_if` (`:9058-9077`).
- **13 queue-time call sites, all in `rules/abilities.rs`**: `:4502`, `:4565`, `:4815`,
  `:4860`, `:4957`, `:5403`, `:5713`, `:5771`, `:5829`, `:6270`, `:6454`, `:6783`, `:6839`
  — plus one resolution-time site at `rules/resolution.rs:2226`. *(The pre-survey said 14
  in `abilities.rs`; the 14th is the `pub fn` declaration at `:9058`.)*
- Mechanism 2 is **queue-gated everywhere it is queued**. It is not DP-15 and is not in
  scope. Its own residual gaps are seeded in §9.

### The third thing, which the audit did not see

A **card-def condition can be lost entirely** — not merely un-checked at queue time — when
the ability is *lowered* into mechanism 2. `testing::replay_harness::build_face_ability_vectors`
(`:2134-3415`) converts a handful of `TriggerCondition`s (`WhenDies`, `WhenAttacks`,
`WhenBlocks`, `WhenDealsCombatDamageToPlayer`, …) into `TriggeredAbilityDef`s and hardcodes
`intervening_if: None` at **every** push (see the confession comment at `:2315-2316`).
`Condition` and `InterveningIf` are different types, so the card-def condition cannot ride
along. Those triggers are then dispatched by `collect_triggers_for_event`
(`abilities.rs:6782`), which gates on the **mechanism-2** field (`None`), and resolved by
`resolution.rs:2205-2239`, which also reads the mechanism-2 field (`None`).

**Consequence: for a lowered trigger the intervening-if is checked in neither place.**
Corpus exposure: `aurelia_the_warleader.rs` and `karlach_fury_of_avernus.rs` (`WhenAttacks`
+ `IsFirstCombatPhase`) and `tatyova_steward_of_tides.rs`
(`WheneverPermanentEntersBattlefield` + `ControlAtLeastNOtherLands(6)`).

**This is OUT OF SCOPE and is the one place the "STOP and say so" clause bites.** Closing it
requires either a new `Condition`-typed field on `TriggeredAbilityDef` — which is part of
`Characteristics`, is hashed (`state/hash.rs:3337`), and is therefore a **HASH bump** — or
re-routing those trigger conditions off the lowering onto a `CardDefETB`-style dispatch,
which is a multi-session refactor. Neither belongs in a no-wire PB. Seeded as **OOS-DP6-1**
with the derivable index correspondence written down for whoever takes it.

> Runner action: before writing §9's seed text, prove the claim with one throwaway probe
> (Aurelia-shaped def, extra combat phase, assert the untap/token fires when it must not).
> If the probe *fails to reproduce*, say so in the review — the claim is derived from source
> reading, not from execution.

---

## 2. Full partitioned inventory of the 82 `PendingTrigger::blank` sites

**Partition rule (mechanical, reproducible).** A `blank` site can carry a mechanism-1
`intervening_if` **iff** the enclosing block destructures `AbilityDefinition::Triggered`
from the card registry. That destructure is exhaustively enumerable:

```
rg -n "AbilityDefinition::Triggered" crates/engine/src --glob '!testing/*'
```

On this branch that grep returns **15** non-`testing` sites; 11 of them own a `blank` push,
2 are the ETB/Tribute pair in `replacement.rs`, 1 is the graveyard sweep, and 1
(`abilities.rs:6189`) is a `retain` post-filter with no `blank` of its own. Everything else
in the 82 is keyword/hardcoded machinery or a mechanism-2 sweep.

Counts by file (verified on this branch): `abilities.rs` 46, `turn_actions.rs` 22,
`resolution.rs` 6, `replacement.rs` 3, `effects/mod.rs` 2, `casting.rs` / `mana.rs` /
`miracle.rs` 1 each = **82**. The pre-survey's table is **correct**.

### Category A — IN SCOPE (14 sites)

| # | site (`blank` line) | destructure | trigger condition(s) | today | action |
|---|---|---|---|---|---|
| A1 | `rules/turn_actions.rs:317` | `:288` | `AtBeginningOfYourUpkeep` / `AtBeginningOfEachUpkeep` | **ungated** (self-documented at `:265-266`) | add gate |
| A2 | `rules/turn_actions.rs:475` | `:450` | `AtBeginningOfFirstMainPhase` | **ungated** | add gate |
| A3 | `rules/turn_actions.rs:539` | `:514` | `AtBeginningOfPostcombatMain` | **ungated** | add gate |
| A4 | `rules/turn_actions.rs:745` | `:720` | `AtBeginningOfYourEndStep` | **ungated** (self-documented at `:701`) | add gate |
| A5 | `rules/turn_actions.rs:1717` | `:1695` | `AtBeginningOfCombat` | **ungated** | add gate |
| A6 | `rules/mana.rs:870` | `:822` | `WhenTappedForMana` (stack branch) | **ungated** | add gate |
| A6b | `rules/mana.rs:843-852` (**no `blank`** — immediate resolution, CR 605.4a) | `:822` | `WhenTappedForMana` (triggered *mana* ability) | **ungated at both ends** | add gate (see §4) |
| A7 | `rules/replacement.rs:1854` | `:1824` | `WhenEntersBattlefield` | **gated, context incomplete** | repair ctx |
| A8 | `rules/replacement.rs:1872` | `:1858` | `TributeNotPaid` | hardcoded gate; `intervening_if` **deliberately ignored** | add gate *in addition to* the tribute check |
| A9 | `rules/abilities.rs:3763` | `:3746` | `WhenYouCastThisSpell` | **ungated** (comment at `:3752` admits it) | add gate |
| A10 | `rules/abilities.rs:4077` | `:4056` | `WhenExertedAsAttacks` | **ungated** | add gate |
| A11 | `rules/abilities.rs:5054` | `:5027` | `WhenDealsCombatDamageToPlayer` | **ungated** | add gate |
| A12 | `rules/abilities.rs:5907` | `:5900` | `WhenTurnedFaceUp` (kind `TurnFaceUp`) | **ungated** at queue **and** at resolution | add gate; seed the resolution half |
| A13 | `rules/abilities.rs:5955` | `:5943` | `WheneverRingTemptsYou` | **ungated** | add gate |
| A14 | `rules/abilities.rs:6962` | `:6892` | `TriggerZone::Graveyard` + `WheneverPermanentEntersBattlefield` | **gated** (`:6949-6955`) | convert to shared helper, no behaviour change |

**Category C (deliberately ignored, NOT fixed here): `rules/abilities.rs:6189`** — the
`WheneverYouSacrifice` `retain` post-filter. It holds the card-def ability
(`def.effective_abilities(is_transformed).get(t.ability_index)`, `:6184-6187`) and never
reads `intervening_if`. **Do not add a gate here.** The trigger it filters was queued by the
*mechanism-2* sweep, so `t.ability_index` is a runtime index being used to index the
card-def list — a pre-existing index-namespace mismatch. Reading `intervening_if` through a
possibly-wrong index could suppress the wrong trigger, which hard constraint 3 forbids.
Seeded as **OOS-DP6-2**.

### Category B — NOT APPLICABLE (68 sites)

No card-def `AbilityDefinition::Triggered` in scope at the push. Two sub-kinds:

**B1 — hardcoded keyword / rules machinery (54 sites).** No `intervening_if` exists to read.

`casting.rs:4207` (Madness) · `turn_actions.rs:70` (Suspend), `:107` (Vanishing), `:151`
(Fading), `:198` (Echo), `:243` (Cumulative upkeep), `:561` (Unearth), `:592` (Encore),
`:609` (Dash), `:629` (Blitz), `:651` (Warp), `:683` (Impending), `:827`, `:861`, `:893`,
`:928` (delayed actions), `:1306` (Madness), `:2242` (delayed action) ·
`miracle.rs:103` · `resolution.rs:1518` (Ravenous), `:1550` (Squad), `:1576` (Offspring),
`:1613` (Gift), `:2484` (Vanishing sacrifice), `:5814` (Suspend) ·
`replacement.rs:1682` (**Saga chapter** — `AbilityDefinition::SagaChapter` has no
`intervening_if` field at all; CR 714.2b) ·
`abilities.rs:1553`, `:1964` (Madness), `:2981`, `:3026` (keyword ETB), `:3062` (Hideaway),
`:3103`, `:3159` (Backup), `:3201` (Champion), `:3274`, `:3330` (Partner-with),
`:3452`, `:3528` (ETB-adjacent keyword), `:4183` (Ring level 2), `:4354` (becomes blocked),
`:4627` (Recover), `:4650`, `:5691`, `:5749`, `:5807` (exile riders), `:4677`, `:4716`
(Haunt), `:5116` (combat-damage keyword), `:5176` (Renown), `:5237`, `:5275` (Cipher),
`:5644` (Ring level 4), `:6361` (delayed) ·
`effects/mod.rs:4826`, `:8640` (Madness).

**B2 — mechanism-2 sweeps, already queue-gated (14 sites).** Gate line in parentheses.

`abilities.rs:4546` (gate `:4502`), `:4576` (`:4565`), `:4825` (`:4815`), `:4872` (`:4860`),
`:4964` (`:4957`), `:5453` (`:5403`), `:5724` (`:5713`), `:5782` (`:5771`), `:5840`
(`:5829`), `:6277` (`:6270`), `:6463` (`:6454`), `:6795` (`:6783`), `:6850` (`:6839`).
That is 13; the 14th, `abilities.rs:5453`'s sibling in the same block, shares gate `:5403`.

> **Runner: re-derive B1/B2 mechanically rather than trusting the labels above.** The
> `AbilityDefinition::Triggered` grep is the authority; the B1 sub-labels (which keyword
> owns which line) were read off two lines of context and are *descriptive*, not load-bearing.
> If any B1 line turns out to destructure a card-def triggered ability, promote it to A and
> say so in the review.

---

## 3. Fix shape: one helper + one exhaustive predicate

### 3.1 Why a shared helper and not per-site gates

Per-site gates were tried once already (the MR-B12-07/08 inline duplicate at the ETB site,
which handled `OpponentHasPoisonCounters` and silently `_ => true`'d every other variant).
The comment at `replacement.rs:1830-1833` is the tombstone. Eleven new copies of a
four-line `if let Some(cond)` is eleven chances to build the wrong `EffectContext`.

### 3.2 The two new functions

**(a) `crates/engine/src/effects/mod.rs`, immediately after `check_condition` (after `:9333`):**

```rust
/// CR 603.4: can this condition be evaluated faithfully at the moment the trigger
/// event occurs, i.e. before the ability is put on the stack and before targets or
/// cast-time flags are available on an `EffectContext`?
///
/// `false` means "the engine cannot answer honestly here" — NOT "the condition is
/// false". Callers must treat `false` as *do not suppress the trigger* (PB-DP6 hard
/// constraint 3: wrongly suppressing a trigger is worse than over-firing).
///
/// EXHAUSTIVE ON PURPOSE — no `_` arm. A new `Condition` variant must be classified
/// here before it compiles (the SR-5 idiom).
pub fn condition_is_queue_time_evaluable(cond: &Condition) -> bool { /* ... */ }
```

Arms returning **`false`** (with the reason in a comment):

| variant | why it cannot be answered at queue time |
|---|---|
| `TargetIsLegal { .. }` | reads `ctx.targets`; targets are chosen later, in `flush_pending_triggers`. An empty `targets` makes this **false**, which would suppress a trigger CR 603.4 says must fire. |
| `WasOverloaded` | `ctx.was_overloaded` is never propagated into a trigger context, not even at resolution (`resolution.rs:2165-2194` sets 8 fields and not this one). |
| `WasBargained` | same — although `GameObject.was_bargained` exists (`resolution.rs:622`), the resolution-time trigger ctx does not read it, so gating on it at queue time would be *stricter than resolution*. |
| `WasCleaved` | same as `WasOverloaded`. |
| `EvidenceWasCollected` | same (object field exists at `resolution.rs:625`, trigger ctx does not read it). |
| `GiftWasGiven` | same. |
| `SacrificeFired` | `ctx.sacrifice_fired` is per-*resolution* state (CR 608.2c/608.2h); it has no meaning before the ability is on the stack. |

Recursive arms: `Not(a)` → `f(a)`; `And(a, b)` / `Or(a, b)` → `f(a) && f(b)` (conservative —
one unanswerable arm makes the whole clause unanswerable). Every other variant → `true`.

**(b) `crates/engine/src/rules/abilities.rs`, immediately before `check_intervening_if`
(before `:9053`):**

```rust
/// CR 603.4: evaluate a **card-definition** intervening-if at the moment the trigger
/// event occurs. `true` = queue the trigger; `false` = the ability does not trigger
/// at all.
///
/// `intervening_if` MUST be handed in by the caller, taken from the same
/// `AbilityDefinition::Triggered` the caller matched on. Do NOT re-derive it by
/// index inside this helper: the callers iterate three different index spaces
/// (`def.abilities`, `def.effective_abilities(is_transformed)`, and the runtime
/// vec), and face-awareness (CR 712.8d/e, PB-OS4b/PB-RS4) is inherited from
/// whichever list the caller walked.
///
/// The context mirrors `rules/resolution.rs:2160-2177` for the fields that exist
/// before the ability reaches the stack; `targets` is necessarily empty, which is
/// why `condition_is_queue_time_evaluable` exists.
pub(crate) fn carddef_intervening_if_holds_at_queue_time(
    state: &GameState,
    intervening_if: Option<&crate::cards::card_definition::Condition>,
    controller: PlayerId,
    source: ObjectId,
) -> bool {
    let Some(cond) = intervening_if else { return true };
    if !crate::effects::condition_is_queue_time_evaluable(cond) {
        return true; // hard constraint 3: never suppress on an unanswerable condition
    }
    // CR 113.7a: several callers legitimately hold an LKI source (combat damage,
    // cast triggers, graveyard triggers) — `fizzle_object`, not a bare lookup
    // (SR-25 ratchet), and a vanished source simply contributes 0/0.
    let (kicker_times_paid, x_value) = state
        .fizzle_object(source)
        .map(|o| (o.kicker_times_paid, o.x_value))
        .unwrap_or((0, 0));
    let mut ctx = crate::effects::EffectContext::new_with_kicker(
        controller, source, vec![], kicker_times_paid,
    );
    ctx.x_value = x_value;
    crate::effects::check_condition(state, cond, &ctx)
}
```

### 3.3 What this repairs beyond the headline

`EffectContext::new` zero-fills `kicker_times_paid` (`effects/mod.rs:190`) and `x_value`
(`:195`). The **existing** ETB gate (`replacement.rs:1835`) uses `EffectContext::new`, so
`Condition::WasKicked` at ETB queue time is **unconditionally false** — even though
`obj.kicker_times_paid` was set 1,150 lines earlier at `resolution.rs:619`, before
`queue_carddef_etb_triggers` runs at `resolution.rs:1757`.

**Therefore `thieving_skydiver.rs` and `nullpriest_of_oblivion.rs` never queue their kicked
ETB trigger at all today.** That is a live **false negative** — the exact failure mode hard
constraint 3 is written to prevent — shipped inside the gate the audit held up as one of the
two correct ones. Neither card has a test (`rg thieving_skydiver crates/engine/tests` → 0).
Building the context correctly in the shared helper fixes it. See §7 T1.

### 3.4 Call-site mechanics the runner will hit

- **`turn_actions.rs` A1–A5**: the condition must be evaluated inside the `filter_map`
  closure, which already borrows `state` immutably twice (`let registry = &state.card_registry;`
  plus `state.objects.values()`). Adding a third immutable reborrow for
  `carddef_intervening_if_holds_at_queue_time(state, …)` is legal. If the borrow checker
  complains because `state: &mut GameState`, bind `let sref: &GameState = state;` at the top
  of the collection block and use `sref` throughout — do **not** restructure into a two-pass
  collect-then-check, which would evaluate the condition at a different moment than the
  event. Each of the five destructures must grow `intervening_if` (they currently use
  `AbilityDefinition::Triggered { trigger_condition, .. }`).
- **A5 (`begin_combat`) and A2/A3 (`main_phase_actions`)** already filter `controller != active`;
  keep the gate **after** that filter so a false condition on an irrelevant permanent costs
  nothing.
- **A7 (ETB)**: replace the inline `if let Some(cond) { … check_condition … }` at
  `replacement.rs:1834-1839` with the helper. Behaviour changes only for `WasKicked` /
  `XValueAtLeast`.
- **A8 (Tribute)**: keep `if !tribute_was_paid` and AND the helper in. The destructure at
  `:1858` must grow `intervening_if`.
- **A14 (graveyard)**: replace `:6949-6955` with the helper. `owner` stays the controller
  argument and `obj_id` the source — identical to today.
- **A6b (`mana.rs:843-852`)**: this branch never builds a `PendingTrigger`; it calls
  `execute_effect` inline (CR 605.4a). Gate it with the same helper before the
  `if targets.is_empty() && is_mana_producing_effect(effect)` split, so both branches share
  one gate. Note in the code comment that CR 603.4 applies to a triggered *mana* ability
  exactly as it does to any other trigger even though it never uses the stack (CR 605.4a).

---

## 4. Per-site default when the condition cannot be evaluated (hard constraint 3)

**Global rule: the default is `true` — queue the trigger — at every one of the 14 sites.**
The resolution-time re-check (`resolution.rs:2139-2156`) is retained, so an unanswerable
condition degrades to exactly today's behaviour: the ability goes on the stack and does
nothing if the condition is false at resolution. No site defaults to `false`.

Site-by-site defence of the *source* and *controller* the helper is given — getting these
wrong is the other way to suppress a trigger wrongly.

| site | `source` | `controller` | source's zone at gate time | risk & defence |
|---|---|---|---|---|
| A1–A5 | `obj.id` (battlefield permanent) | `obj.controller` | Battlefield | Identical to resolution's `source_object`. `SourceOnBattlefield` / `SourceHasCounters` / `SourceIsSolved` all answer correctly. Zero risk. |
| A6 | `trigger_source_id` | `player` (the tapper) | Battlefield | `player` is the mana-ability activator, which is the trigger source's controller on this path (`mana.rs:792` filters `o.controller == player`). Correct. |
| A6b | same | same | Battlefield | same |
| A7 | `new_id` (entering permanent) | `controller` | Battlefield (already moved) | Unchanged from today except the ctx repair. `WasCast` (`obj.was_cast`, set at `resolution.rs:731`) and `WasKicked`/`XValueAtLeast` (set at `:619`/`:628`) all precede `queue_carddef_etb_triggers` at `:1757`. **Verify this ordering holds on the other 11 `queue_carddef_etb_triggers` call sites** (`effects/mod.rs:1657/4333/5649/5976/6215/6496`, `lands.rs:406`, `resolution.rs:3240/4486/6231/6455/6681/6922/7616`) — a token/flicker path that has not set `was_cast` will (correctly) answer `WasCast` false, which is the printed behaviour of The One Ring and Geological Appraiser. |
| A8 | `new_id` | `controller` | Battlefield | same as A7 |
| A9 | `*source_object_id` — **the spell's stack object** | `caster` | **Stack** | The only site where the source is not a permanent. `SourceOnBattlefield` would answer **false** here, and `SourceHasCounters` would answer false. No corpus def pairs `WhenYouCastThisSpell` with an `intervening_if` (§5 table: zero), so this is unreachable today — but it is the one site where a future author could be surprised. **Add a source-code comment saying so.** Do not special-case it; CR 603.4 asks the question against the game state as it is, and "this permanent is on the battlefield" genuinely is false while the spell is on the stack. |
| A10 | `source_id` (the exerted attacker) | `controller` | Battlefield (guarded at `:4043`) | Guard already requires `zone == Battlefield && is_phased_in()`. Zero risk. |
| A11 | `source_id` (the damage dealer) | `controller` | Battlefield (guarded at `:5014`) | Same guard. LKI read via `fizzle_object` at `:5013` is already in place. |
| A12 | `*permanent` | `ctrl` | Battlefield | Just turned face up (CR 708.8) — full characteristics available. |
| A13 | `obj_id` | `*tempted_player` | Battlefield (guarded at `:5929-5932`) | The controller passed is the *tempted* player, which the guard has already equated to `obj.controller`. Correct. |
| A14 | `obj_id` | `owner` | **Graveyard** | Unchanged from today. `SourceOnBattlefield` answers false, which is correct for a graveyard-zone trigger. |

**Explicitly rejected alternative:** gating in `flush_pending_triggers`
(`abilities.rs:6989+`) instead of at each collect site. Rejected because CR 603.4 pins the
check to "when the trigger event occurs", and `flush_pending_triggers` runs later — after
SBAs, after other triggers in the same batch have been collected, and (CR 603.3) only "the
next time a player would receive priority". Gating there would be a *different rule*, and it
would also diverge from the two gates that already exist (both at collect time).

---

## 5. Card-def classification (criterion 5536)

### 5.1 All 24 defs with `intervening_if: Some(..)`

`rg -l '^\s*intervening_if: Some' crates/card-defs/src/defs/` → **24 files, not 25**. (The
pre-survey's 25 counts `twilight_prophet.rs`, whose only hit is a *comment* at `:6`.)

| # | def | trigger condition | condition | dispatch site | status after PB-DP6 |
|---|---|---|---|---|---|
| 1 | `acererak_the_archlich.rs:61` | `WhenEntersBattlefield` | `Not(CompletedSpecificDungeon)` | A7 | already correct — no change |
| 2 | `geological_appraiser.rs:26` | `WhenEntersBattlefield` | `WasCast` | A7 | already correct — no change |
| 3 | `the_one_ring.rs:51` | `WhenEntersBattlefield` | `WasCast` | A7 | already correct — no change |
| 4 | `vivisection_evangelist.rs:32` | `WhenEntersBattlefield` | `OpponentHasPoisonCounters(3)` | A7 | already correct — no change |
| 5 | **`nullpriest_of_oblivion.rs:45`** | `WhenEntersBattlefield` | **`WasKicked`** | A7 | **FLIP — starts triggering at all** (§3.3) |
| 6 | **`thieving_skydiver.rs:36`** | `WhenEntersBattlefield` | **`WasKicked`** | A7 | **FLIP — starts triggering at all** (§3.3) |
| 7 | **`dragonmaster_outcast.rs:44`** | `AtBeginningOfYourUpkeep` | `YouControlNOrMoreWithFilter{6, Land}` | A1 | **FLIP — stops over-firing** |
| 8 | **`hellkite_tyrant.rs:36`** | `AtBeginningOfYourUpkeep` | `YouControlNOrMoreWithFilter{20, Artifact}` | A1 | **FLIP** (def is `partial` — deck-blocked, latent) |
| 9 | **`ingenious_prodigy.rs:73`** | `AtBeginningOfYourUpkeep` | `SourceHasCounters{+1/+1, 1}` | A1 | **FLIP** (def is `known_wrong` — deck-blocked, latent) |
| 10 | **`land_tax.rs:65`** | `AtBeginningOfYourUpkeep` | `OpponentControlsMoreLandsThanYou` | A1 | **FLIP — stops over-firing** |
| 11 | **`revel_in_riches.rs:45`** | `AtBeginningOfYourUpkeep` | `YouControlNOrMoreWithFilter{10, Treasure}` | A1 | **FLIP** — a `WinGame` trigger that currently goes on the stack every upkeep |
| 12 | **`simic_ascendancy.rs:66`** | `AtBeginningOfYourUpkeep` | `SourceHasCounters{growth, 20}` | A1 | **FLIP** (def is `partial` — deck-blocked, latent) |
| 13 | **`birthing_ritual.rs:45`** | `AtBeginningOfYourEndStep` | `YouControlNOrMoreWithFilter{1, Creature}` | A4 | **FLIP — stops over-firing** |
| 14 | **`case_of_the_locked_hothouse.rs:35`** | `AtBeginningOfYourEndStep` | `And(…)` | A4 | **FLIP — stops over-firing** |
| 15 | **`contaminant_grafter.rs:57`** | `AtBeginningOfYourEndStep` | `OpponentHasPoisonCounters(3)` | A4 | **FLIP — stops over-firing** |
| 16 | **`growing_rites_of_itlimoc.rs:46`** | `AtBeginningOfYourEndStep` | `YouControlNOrMoreWithFilter{4, Creature}` | A4 | **FLIP — stops over-firing** |
| 17 | **`raiders_wake.rs:46`** | `AtBeginningOfYourEndStep` | `YouAttackedThisTurn` | A4 | **FLIP — stops over-firing** |
| 18 | **`searslicer_goblin.rs:46`** | `AtBeginningOfYourEndStep` | `YouAttackedThisTurn` | A4 | **FLIP — stops over-firing** |
| 19 | **`thaumatic_compass.rs:57`** | `AtBeginningOfYourEndStep` | `YouControlNOrMoreWithFilter{7, Land}` | A4 | **FLIP — stops over-firing** |
| 20 | **`loyal_apprentice.rs:80`** | `AtBeginningOfCombat` | `YouControlYourCommander` | A5 | **FLIP — stops over-firing** (`Complete`; the caveat's named divergent case) |
| 21 | **`siege_gang_lieutenant.rs:70`** | `AtBeginningOfCombat` | `YouControlYourCommander` | A5 | **FLIP — stops over-firing** (`Complete`) |
| 22 | `aurelia_the_warleader.rs:33` | `WhenAttacks` | `IsFirstCombatPhase` | **lowering (§1)** | **NOT FIXED — OOS-DP6-1** |
| 23 | `karlach_fury_of_avernus.rs:42` | `WhenAttacks` | `IsFirstCombatPhase` | **lowering (§1)** | **NOT FIXED — OOS-DP6-1** |
| 24 | `tatyova_steward_of_tides.rs:89` | `WheneverPermanentEntersBattlefield` | `ControlAtLeastNOtherLands(6)` | **lowering (§1)** | **NOT FIXED — OOS-DP6-1** |

**Yield: 17 defs change behaviour on paper; discounted per `feedback_pb_yield_calibration`
to 11 that a player will actually observe** — rows 5–7, 10, 11, 13–18, 20, 21 are
`Complete`-or-reachable; rows 8, 9, 12 are `partial`/`known_wrong` and therefore
`validate_deck`-blocked (latent). **Completeness-marker flips: 0**, as §8 of the audit
predicted — a def that stops over-firing does not change its marker, and rows 5/6 do not
either (their markers describe unrelated gaps).

**Zero card-def source edits are required by the engine change.** The two caveat edits in
§5.2 are documentation.

### 5.2 Caveat keep/clear table (criterion 5536)

| file:lines | caveat text | disposition | reason |
|---|---|---|---|
| `loyal_apprentice.rs:21-30` | "`intervening_if` is checked only at resolution (resolution.rs:2125-2135), never at queue time, though CR 603.4 requires both. Divergent case: …" | **CLEAR** — replace with a one-line "PB-DP6 (`scutemob-154`): queue-time gate added; both halves of CR 603.4 now hold." | The statement becomes false. Also its line cite was already stale (the re-check is at `:2139-2156`). |
| `siege_gang_lieutenant.rs:18-23` | same claim, cross-referencing loyal_apprentice | **CLEAR** — same replacement | same |
| `acererak_the_archlich.rs:10`, `:42-44`, `:60` | "checked at trigger time and at resolution (CR 603.4)" | **KEEP** | Already true (ETB path) and stays true. |
| `the_one_ring.rs:10`, `:34` · `geological_appraiser.rs:25` · `nullpriest_of_oblivion.rs:36` | "checked at trigger and resolution" | **KEEP** | True after the fix. For `nullpriest` it becomes true *for the first time* — no text change needed. |
| `hellkite_tyrant.rs:31` · `dragonmaster_outcast.rs:21` · `revel_in_riches.rs:40` · `birthing_ritual.rs:16`, `:40` · `case_of_the_locked_hothouse.rs:30` · `siege_gang_lieutenant.rs:8` · `loyal_apprentice.rs:9` | "re-checked at resolution (CR 603.4)" | **KEEP** | The resolution re-check is retained (hard constraint 2). These sentences remain accurate; blanket-deleting them would be wrong. |
| `emeria_the_sky_ruin.rs:29-30`, `:42` · `garruks_uprising.rs:23`, `:68-71` · `inventors_fair.rs:6`, `:74-77` · `jadar_ghoulcaller_of_nephalia.rs:32-36` · `dwynen_s_elite.rs:22-24` · `ophiomancer.rs:22-23`, `:54-56` · `vampire_socialite.rs:28-29`, `:34-35` · `thousand_faced_shadow.rs:6-9`, `:65-66` · `guardian_project.rs:6` | "the `Condition`/`InterveningIf` DSL lacks the needed variant" | **KEEP — different class** | These are DSL-expressiveness gaps, not queue-time gaps. PB-DP6 retires none of them. **Rider worth recording:** `garruks_uprising.rs:68-71` and `inventors_fair.rs:74-77` name **`InterveningIf`** (mechanism 2) as the blocker, but the def-level field is `Option<Condition>` (mechanism 1) and `check_condition` already has `YouControlNOrMoreWithFilter`. `ophiomancer.rs:54` says exactly this ("Blocker stale"). Those two blocker notes are **stale**, and both cards may be authorable today. Seeded as **OOS-DP6-3**; do not author them in this PB. |

---

## 6. Wire gate: exact expectation and falsifier

**Expectation: `PROTOCOL_VERSION` 27 and `HASH_SCHEMA_VERSION` 64 both unchanged.**

Basis: PB-DP6 adds two free functions and edits control flow. It adds **no** `Command`,
`GameEvent`, `Effect`, `Condition`, `TriggerCondition`, `PendingTriggerKind` or
`StackObjectKind` variant; **no** field on `GameState`, `PendingTrigger`, `StackObject`,
`TriggeredAbilityDef` or `Characteristics`; and no change to any `HashInto` impl.

**What would falsify it (and is therefore forbidden without escalating):**

1. Adding an `Option<Condition>` field to `TriggeredAbilityDef` to close §1's lowering-drop.
   `TriggeredAbilityDef` is inside `Characteristics`, hashed at `state/hash.rs:3337` →
   **HASH bump**. This is OOS-DP6-1; **do not do it here**.
2. Adding a `condition_evaluated_at_queue_time` marker to `PendingTrigger` → HASH bump.
3. Adding a `GameEvent::TriggerSuppressedByInterveningIf` for diagnosability → PROTOCOL bump.
   Tempting (a suppressed trigger is invisible in the event log — see OOS-DP6-4) and
   explicitly deferred.

**Gates that decide it, not the runner:** `declaration_fingerprint_is_pinned` (HASH) and the
`PROTOCOL_SCHEMA_FINGERPRINT` check. **Never hand-bump.** If either fires, stop and report —
that is evidence the design drifted into (1)/(2)/(3).

**SR-25 `bare_lookup_ratchet` expectation: unchanged.** The helper's only lookup is
`state.fizzle_object(source)`, which the counter does not count as bare. The 11 call sites
add zero `.objects/.players/.zones.get(` occurrences. Pinned ceilings that could move if the
runner takes a shortcut: `src/rules/abilities.rs` **75**, `src/rules/turn_actions.rs` **7**,
`src/rules/replacement.rs` **24**, `src/rules/mana.rs` **8**, `src/effects/mod.rs` **110**.
The ratchet fails in **both** directions (`bare_lookup_ratchet.rs:249` and `:261`), so a
"harmless" conversion to `expect_object` in a swept file also breaks the build until re-pinned.

**SR-7**: every `PendingTrigger` still goes through `PendingTrigger::blank`. This PB removes
pushes; it never constructs one another way.

**SR-4**: the helper introduces no new silent-failure site in `effects/mod.rs` /
`rules/resolution.rs` — `fizzle_object` is the already-classified LKI vocabulary, and its
`unwrap_or((0, 0))` is documented as a rules-correct fallback, not a swallowed bug.

**SR-9a**: tests go in `crates/engine/tests/primitives/`, registered in
`crates/engine/tests/primitives/main.rs` (add `mod pb_dp6_intervening_if_queue_time;` after
`:25`). No new top-level `tests/*.rs`.

---

## 7. Tests, with per-test fail-before predictions

**New file**: `crates/engine/tests/primitives/pb_dp6_intervening_if_queue_time.rs`
**Registration**: `crates/engine/tests/primitives/main.rs` (after line 25).
**Pattern to copy**: `crates/engine/tests/primitives/pb_rs3_at_beginning_of_combat_sweep.rs`
(it already builds Siege Gang Lieutenant / Loyal Apprentice states and asserts on trigger
presence) and `pb_dp3_modal_mode_announcement.rs` for the probe idiom.

| # | test | asserts | fail-before prediction |
|---|---|---|---|
| T1 | `test_dp6_etb_waskicked_gate_uses_object_kicker_count` | Kicked Nullpriest-shaped ETB trigger **is queued**; unkicked is not. | **FAILS before** — the kicked case queues nothing today (`EffectContext::new` zero-fills `kicker_times_paid`). This is the §3.3 false negative. |
| T2 | `test_dp6_upkeep_trigger_not_queued_when_condition_false` | Land Tax–shaped upkeep trigger with `OpponentControlsMoreLandsThanYou` false ⇒ `pending_triggers` empty after `upkeep_actions`. | **FAILS before** — trigger is queued. |
| T3 | `test_dp6_upkeep_trigger_queued_when_condition_true` | same, condition true ⇒ queued. | Passes before and after (non-regression). |
| T4 | `test_dp6_end_step_trigger_not_queued_when_condition_false` | Searslicer-shaped `YouAttackedThisTurn` false at end step ⇒ not queued. | **FAILS before.** |
| T5 | `test_dp6_begin_combat_trigger_not_queued_without_commander` | Loyal Apprentice with no commander on battlefield ⇒ not queued at `BeginningOfCombat`. | **FAILS before** — this is the exact divergence `loyal_apprentice.rs:23-26` documents. |
| T6 | `test_dp6_resolution_recheck_retained` | Condition **true at queue time, false at resolution** ⇒ trigger goes on the stack and resolves with **no effect** (CR 603.4 second sentence). | Passes before and after — the pin that hard constraint 2 was honoured. |
| T7 | `test_dp6_first_main_and_postcombat_main_gates` | A2/A3 sweeps gate. | **FAILS before.** |
| T8 | `test_dp6_unevaluable_condition_does_not_suppress` | An upkeep trigger whose `intervening_if` is `Condition::TargetIsLegal { index: 0 }` (or `And(YouAttackedThisTurn, WasCleaved)`) is **still queued**. | Passes before (nothing gates) and **must still pass after** — this is the hard-constraint-3 regression pin. Its value is entirely post-fix. |
| T9 | `test_dp6_graveyard_gate_unchanged` | Bloodghast-shaped graveyard landfall trigger with a false condition still not queued; with a true condition still queued. | Passes before and after — proves the A14 refactor is behaviour-neutral. |
| T10 | `test_dp6_tribute_not_paid_respects_intervening_if` | A `TributeNotPaid` ability with a false `intervening_if` is not queued even when tribute was not paid. | **FAILS before** (A8's field is ignored). No corpus card exercises it; synthetic def. |
| T11 | `test_dp6_face_aware_gate_reads_back_face_condition` | A transformed permanent's back-face upkeep trigger is gated by the **back face's** condition, not the front's. | Passes before *vacuously* (nothing gates); must pass after. Pins the PB-OS4b/PB-RS4 contract through the new helper. |
| T12 | `test_dp6_condition_evaluability_predicate_is_exhaustive` | Pure unit test over `condition_is_queue_time_evaluable`: the 7 `false` variants answer false, `Not`/`And`/`Or` propagate, a representative state-only variant answers true. | New — no before/after. |

### Existing tests: predicted impact

| test | prediction | reasoning |
|---|---|---|
| `primitives/pb_ac8_restrictions_and_wingame.rs:332 test_wingame_via_intervening_if_upkeep_trigger` | **UNCHANGED** | It pushes `PendingTrigger::blank(..., Normal)` **directly** (`:379-384`), bypassing the upkeep sweep entirely, and asserts only the resolution re-check. The pre-survey flagged it as a likely casualty; it is not one. |
| `mechanics_e_l/evolve.rs:1002 test_evolve_intervening_if_fails_at_resolution` | **UNCHANGED** | Evolve is `KeywordAbility::Evolve` with its own keyword-specific resolution check — neither mechanism 1 nor 2. |
| `mechanics_e_l/graft.rs:857 test_graft_resolution_recheck_intervening_if` | **UNCHANGED** | Same reasoning (keyword path). |
| `primitives/pb_os9_lieutenant_commander_control.rs:618/:677` and `primitives/pb_rs3_at_beginning_of_combat_sweep.rs:353/:408` | **AT RISK — verify first** | Four tests named `..._intervening_if_fails_when_commander_removed` / `..._creates_tokens` on exactly the A5 sweep. If any of them removes the commander **after** `begin_combat` runs, it still passes. If one removes the commander **before**, the trigger will no longer be queued and its "trigger present on stack, then resolves with no effect" assertion breaks. **Any such change is CR-justified** (CR 603.4 sentence 2: the ability "triggers only if" the condition is true — a trigger that never triggers cannot be on the stack) and must be rewritten to assert absence-from-`pending_triggers` rather than a no-op resolution. Do **not** weaken the gate to keep them green. |
| Golden scripts (`test-data/generated-scripts/`) | **AT RISK — run and read** | Any script whose expected event stream contains a `TriggerPutOnStack`/`AbilityResolved` for one of §5.1's 15 flipping defs. Grep the corpus for those card names before starting; SR-9c forbids silent skips, so a broken script will surface. |

**Do not adjust a test to fit.** Every changed assertion needs a one-line CR 603.4
justification in the diff, per the task brief.

---

## 8. Pre-survey bullets that turned out to be WRONG

| bullet | verdict |
|---|---|
| §A: "the runtime type is `TriggeredAbility { intervening_if }` at `game_object.rs:817-827`" | **Wrong.** The struct is `TriggeredAbilityDef`; `:816-827` is the `InterveningIf` **enum**; the field is at `:889`. |
| §A: "mechanism 2 has **14** call sites, all in `rules/abilities.rs`" | **Wrong by one.** 13 call sites in `abilities.rs` (`:4502/4565/4815/4860/4957/5403/5713/5771/5829/6270/6454/6783/6839`) + 1 in `resolution.rs:2226`. The 14th `abilities.rs` hit is the `pub fn` declaration at `:9058`. |
| §A: "queue-time check exists at exactly **one** place: `replacement.rs:~1829-1839`, plus a hardcoded `TributeNotPaid` sibling" | **Wrong — two and a half.** The graveyard sweep (`abilities.rs:6949-6955`) is a real, general mechanism-1 gate that the survey attributed to mechanism 2. |
| §A hypothesis: "mechanism 1 is the real gap; mechanism 2 may have its own, smaller holes" | **Half right.** Mechanism 1 is the gap. Mechanism 2 has **no** queue-time holes found. The *third* class — condition dropped by lowering (§1) — is the real "smaller hole", and it is worse than DP-15 because it fails at both ends. |
| §A/§C implicit assumption that mechanism 1's ETB gate is correct | **Wrong, and this is the biggest single finding.** The gate builds `EffectContext::new`, which zero-fills `kicker_times_paid`, so `Condition::WasKicked` is unconditionally false and **two `Complete`-path defs never trigger at all** (§3.3). A false **negative** inside the gate the audit called class-A. |
| §B: "82 `PendingTrigger::blank` sites; `abilities.rs` 46 / `turn_actions.rs` 22 / `resolution.rs` 6 / `replacement.rs` 3 / `effects/mod.rs` 2 / `casting.rs`,`mana.rs`,`miracle.rs` 1 each" | **Correct**, verified line-by-line. |
| §B: "the audit's three named sites are a starting roster" | **Correct** — the true in-scope roster is **14**, of which the audit named 3 and one of those (`turn_actions.rs:264-266`) is a comment, not a site. |
| §C: "**25** files contain `intervening_if: Some(...)`" | **Wrong — 24.** The 25th is `twilight_prophet.rs:6`, a comment. |
| §C spot-check list | **Mostly right, three corrections.** `karlach`/`aurelia`/`tatyova` are **not** fixed by this PB (lowering-drop, §1). `acererak_the_archlich` is an **ETB** def (already correct), not a live-wrong one — the survey listed it among the live-wrong candidates. `hellkite_tyrant`, `simic_ascendancy`, `ingenious_prodigy` are deck-blocked (`partial`/`known_wrong`), so their flips are latent. |
| §C: "expect the live-wrong set to be materially smaller than 25" | **Correct** — 15 defs flip, 11 observably. |
| §C: two caveats name the queue-time gap | **Correct**, exactly two (`loyal_apprentice.rs:21-30`, `siege_gang_lieutenant.rs:18-23`). |
| §D: "confirm whether `check_condition` reads raw characteristics" | **Resolved — it is layer-correct.** Every battlefield-scanning arm uses `layers::expect_characteristics` (`effects/mod.rs:8899/8910/9008/9024/9063/9084/9102/9115/9126/9294/9388`). The graveyard/hand/library arms use `obj.characteristics` deliberately, per CR 400.2. No W3-LC hazard. |
| §D: "`bare_lookup_ratchet` will likely need re-pinning" | **Predicted NOT to move** — the helper uses `fizzle_object`. If it moves, the runner took a shortcut. |
| §D: the three named at-risk tests | **All three predicted UNCHANGED.** The real at-risk set is four *different* tests in `pb_os9_lieutenant_commander_control.rs` / `pb_rs3_at_beginning_of_combat_sweep.rs` (§7). |
| §E: "predict the flip count explicitly and be prepared for it to be 0" | Prediction: **11 observable behaviour flips, 0 completeness flips.** Not 0. |

---

## 9. Seeds (file into `docs/audits/decision-point-audit.md` §8.1)

| seed | finding | class |
|---|---|---|
| **OOS-DP6-1** | **A card-def intervening-if is silently *dropped* when the ability is lowered into a runtime `TriggeredAbilityDef`.** `testing::replay_harness::build_face_ability_vectors` (`:2134-3415`) hardcodes `intervening_if: None` on every push because `Condition` and `InterveningIf` are different types (self-documented at `:2315-2316`). Affected `TriggerCondition`s include `WhenDies`, `WhenAttacks`, `WhenBlocks`, `WhenDealsCombatDamageToPlayer`. The condition is then checked **neither** at queue time (`abilities.rs:6782` reads the mechanism-2 field) **nor** at resolution (`resolution.rs:2205-2239` likewise). Live corpus exposure: `aurelia_the_warleader.rs:33`, `karlach_fury_of_avernus.rs:42` (both `IsFirstCombatPhase` — Aurelia untaps and gets an extra combat on *every* combat phase, not just the first) and `tatyova_steward_of_tides.rs:89`. **Fix options:** (a) an `Option<Condition>` field on `TriggeredAbilityDef` — hashed via `Characteristics`, **HASH bump**; (b) re-route these conditions to a `CardDefETB`-style dispatch. A no-bump (c) exists but is fragile: `build_face_ability_vectors` appends one runtime entry per matching card-def ability **in `def.abilities` order, per condition**, so the *k*-th runtime `SelfAttacks` entry corresponds to the *k*-th `WhenAttacks` card-def ability — that correspondence is derivable at the queue site, but would have to be mirrored at resolution and re-derived on every face change. Deliberately deferred by PB-DP6 (wire constraint). | correctness, **worse than DP-15**, deferred (wire) |
| **OOS-DP6-2** | **`abilities.rs:6189`'s `WheneverYouSacrifice` `retain` indexes the card-def list with a runtime index.** The trigger it filters was queued by `collect_triggers_for_event` (runtime `characteristics.triggered_abilities` index space), and the retain looks it up with `def.effective_abilities(is_transformed).get(t.ability_index)` (card-def index space). PB-DP6 declined to add an intervening-if gate here for exactly this reason (hard constraint 3). The mismatch is pre-existing and independently worth fixing. | correctness, latent |
| **OOS-DP6-3** | **Two `partial` blocker notes name the wrong DSL type and are stale.** `garruks_uprising.rs:68-71` and `inventors_fair.rs:74-77` say the card is blocked because **`InterveningIf`** has only `ControllerLifeAtLeast` / `SourceHadNoCounterOfType`. The def-level field is `Option<Condition>`, and `Condition::YouControlNOrMoreWithFilter` already exists and is already used by six shipped defs. `ophiomancer.rs:54` records the same realisation ("Blocker stale") for its own case. Both cards are plausibly authorable today; re-triage them. | documentation / possible card yield |
| **OOS-DP6-4** | **A trigger suppressed by CR 603.4 is invisible in the event stream.** After PB-DP6 the ability simply never appears; there is no `GameEvent` distinguishing "the condition was false" from "the card does nothing". Same diagnosability class as OOS-DP4-5 (forced decline vs player decline). Cheap client-side derivation is possible in M11-local; an engine-side event is a PROTOCOL bump. | diagnosability |
| **OOS-DP6-5** | **`PendingTriggerKind::TurnFaceUp` never re-checks its intervening-if at resolution.** It routes to `StackObjectKind::TurnFaceUpTrigger` (`abilities.rs:8281-8290`), a resolution path with no CR 603.4 second-sentence check — unlike `TriggeredAbility`. PB-DP6 gives it a queue-time gate (A12); the resolution half stays open. Latent: no corpus `WhenTurnedFaceUp` def carries an `intervening_if`. | correctness, latent |
| **OOS-DP6-6** | **`WasBargained` / `EvidenceWasCollected` / `GiftWasGiven` / `WasOverloaded` / `WasCleaved` are never propagated into a triggered ability's resolution context.** `resolution.rs:2165-2194` populates 8 ctx fields and none of these, even though `GameObject.was_bargained` / `.evidence_collected` are set at `resolution.rs:622/625` specifically so "ETB triggers can check" them (the comments say so). Any def using one of them as an ETB intervening-if would read `false` at both ends. PB-DP6 classifies them **non-evaluable** at queue time to avoid making it worse. Zero corpus exposure today. Fix is a 5-line ctx extension at the resolution site plus removing them from `condition_is_queue_time_evaluable`'s `false` set. | correctness, latent |
| **OOS-DP6-7** | **A triggered *mana* ability that resolves inline (`mana.rs:843-852`, CR 605.4a) has no resolution-time intervening-if check.** PB-DP6 gates it at trigger time (A6b); because it never uses the stack there is no second check to add, which is arguably fine (nothing can change between the two moments) — recorded so a future reader does not mistake the asymmetry for an omission. | documentation / known divergence |
| **OOS-DP6-8** | **The audit's §4.8 site cites are stale by 300–400 lines.** `replacement.rs:1446-1456` → actually `:1834-1839`; `abilities.rs:6910-6916` → actually `:6949-6955`; `resolution.rs:2119-2135` → actually `:2139-2156`. The stale cites propagated verbatim into two card-def caveats (`loyal_apprentice.rs:22`, `siege_gang_lieutenant.rs:19`). PB-DP6 corrects the two caveats; the audit rows should be corrected at close-out. | documentation |

Out-of-scope items carried forward unchanged: **DP-12** (costless "you may" on triggers has
no DSL representation) and **DP-14** (same-controller trigger ordering) are untouched by this
PB and remain as filed.

---

## 10. Verification checklist

- [ ] `cargo check -p mtg-engine` after the helper lands, before any call site is wired.
- [ ] All 14 Category-A sites wired (11 new gates, 2 refactors, 1 ctx repair); **`abilities.rs:6189` deliberately not wired** (OOS-DP6-2).
- [ ] `condition_is_queue_time_evaluable` is exhaustive — **no `_` arm** — and every `false` arm carries its reason.
- [ ] `cargo build --workspace` (hard constraint 6 — simulator / TUI / replay-viewer exhaustive matches).
- [ ] `cargo test --all` green; new file registered in `tests/primitives/main.rs`.
- [ ] `cargo clippy --all-targets -- -D warnings`.
- [ ] `cargo fmt --check` **and** `tools/check-defs-fmt.sh` (SR-35).
- [ ] `PROTOCOL_VERSION == 27` and `HASH_SCHEMA_VERSION == 64` **unchanged**; fingerprint gates green with no hand edit.
- [ ] `bare_lookup_ratchet` green with **no** ceiling edits (if it demands one, justify in the review).
- [ ] Golden scripts run; any changed expectation carries a CR 603.4 justification.
- [ ] Two caveats cleared (`loyal_apprentice.rs`, `siege_gang_lieutenant.rs`); the ~20 "re-checked at resolution" caveats **left alone**.
- [ ] Audit `docs/audits/decision-point-audit.md` §4.8 row (`D` → `A`), §5 DP-15 row, §8 PB-DP6 row updated; OOS-DP6-1..8 filed in §8.1.
- [ ] ESM criteria 5535/5536/5537/5538 satisfied.

## 11. Risks

1. **Suppressing a trigger that should fire** — the only way this PB makes the engine worse.
   Mitigated by the `true` default (§4), the exhaustive evaluability predicate (§3.2), and
   T8. If in doubt at any site, **queue the trigger**.
2. **Wrong `source` in the built context** — turns a correct condition into a silently wrong
   one. §4's table names the source and controller per site; the runner must not invent one.
3. **Borrow-checker pressure in the five `turn_actions.rs` closures** may tempt a
   collect-then-check restructure that moves the evaluation to a different moment. §3.4
   forbids it; use a `&GameState` rebind instead.
4. **Scope creep into OOS-DP6-1.** The three `WhenAttacks`/`WheneverPermanentEnters` defs
   will look like they should be fixable in the same pass. They are not, without a HASH bump.
5. **Golden-script churn** on 15 flipping defs is the most likely source of a long fix cycle.
   Grep the script corpus for those card names *before* writing the engine change.
6. **The A9 stack-object source** (`WhenYouCastThisSpell`) makes `SourceOnBattlefield`
   answer false. Unreachable today; leave a comment rather than a special case.
