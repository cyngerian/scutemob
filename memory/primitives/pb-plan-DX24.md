# Primitive Batch Plan: PB-DX24 — the lowering drops `trigger_zone`; the two index spaces disagree

**Generated**: 2026-08-05
**Task**: `scutemob-202` · **Branch**: `feat/pb-dx24-the-lowering-drops-triggerzone-the-two-index-spaces-`
**Queue**: `memory/primitives/seed-rerank-2026-08-02.md` §4 rank **6**
**Seeds**: `OOS-DX1-3` (live-wrong on `nether_traitor`) + `OOS-DX1-4` (latent, 7 queue sites)
**Stage 0 (already done, do NOT re-derive)**: `memory/primitives/pb-DX24-stage0.md`
**Baseline at HEAD `cceda74c`**: tests **4,413 / 0 / 5**; `PROTOCOL_VERSION` **35**; `HASH_SCHEMA_VERSION` **73**
**Predicted wire consequence**: **none** — PROTOCOL **35** / HASH **73** both predicted **unmoved**.
This is a *prediction to be gate-computed*, never a fact. See §8.

**Dependencies**: PB-DX1 (`InterveningIf::CardDef`, the lossy-lowering table),
PB-35 (`TriggerZone` + `collect_graveyard_carddef_triggers`), PB-OS4b / PB-RS4 (the
`effective_abilities(is_transformed)` contract). All shipped.

**Deferred items from prior PBs carried here**: none claimed. `OOS-DX1-4` is explicitly
described in `docs/audits/decision-point-audit.md:1213` as *"a defect in the alternative (b)
dispatch style that PB-DX1's plan rejected — it stays open on the paths that already use it"*,
which is precisely this batch's second half.

---

## 0. What Stage 0 established (summary only — the file is authoritative)

- `nether_traitor.rs` is `Completeness::Complete`, deck-legal, and pairs
  `TriggerCondition::WheneverCreatureDies` with `trigger_zone: Some(TriggerZone::Graveyard)`
  (`:35`/`:41`/`:57`/`:60`).
- `build_face_ability_vectors` (`crates/engine/src/testing/replay_harness.rs:2449`–`:3871`)
  has **40** trigger-lowering arms; exactly **one** (`WheneverPermanentEntersBattlefield`,
  guard at `:3049-3051`) checks `trigger_zone`. The `WheneverCreatureDies` arm
  (`:3261-3275`, push at `:3283-3301`) swallows it through the `..` rest pattern.
- `collect_graveyard_carddef_triggers` (`crates/engine/src/rules/abilities.rs:7112`, body
  `:7118`–`:7221`) has exactly **one** `fires` arm — `GameEvent::PermanentEnteredBattlefield`
  × `WheneverPermanentEntersBattlefield`. There is **no graveyard dispatch path for a death
  trigger at all**.
- The `trigger_zone: Some(_)` corpus population is **3** defs: `bloodghast` (`partial`),
  `squee_goblin_nabob` (`known_wrong`), `nether_traitor` (**`Complete`**).
- OOS-DX1-4 queue sites Q1–Q7 and the read-side list: stage 0 §3.

**Net today**: Nether Traitor's ability functions from the **battlefield** (where CR says it
does nothing) and does **not** function from the **graveyard** (where CR says it is the only
place it functions). Both halves are wrong, in opposite directions, on one deck-legal card.

---

## 1. CR basis — full text from the MCP, with the verdicts derived

### 1.1 Which zone the ability functions in — **CR 113.6 and CR 113.6m are the load-bearing rules**

> **113.6.** Abilities of an instant or sorcery spell usually function only while that object
> is on the stack. Abilities of all other objects usually function only while that object is on
> the battlefield. The exceptions are as follows:
>
> **113.6a** Characteristic-defining abilities function everywhere, even outside the game and
> before the game begins. (See rule 604.3.)
>
> **113.6b** An ability that states which zones it functions in functions only from those zones.
>
> **113.6c** An ability that states which zones it doesn't function in functions everywhere
> except for the specified zones, even outside the game and before the game begins.
>
> **113.6k** A trigger condition that can't trigger from the battlefield functions in all zones
> it can trigger from. Other trigger conditions of the same triggered ability may function in
> different zones.
>
> **113.6m** An ability whose cost or effect specifies that it moves the object it's on out of
> a particular zone functions only in that zone, unless its trigger condition or a previous part
> of its cost or effect specifies that the object is put into that zone or, if the object is an
> Aura, that the object it enchants leaves the battlefield. The same is true if the effect of
> that ability creates a delayed triggered ability whose effect moves the object out of a
> particular zone.

**Derivation for Nether Traitor.** Its effect is *"return **this card from your graveyard** to
the battlefield"* — an effect that moves the object it's on out of the graveyard. Its trigger
condition speaks about **another** creature, so it does **not** specify that Nether Traitor
itself is put into the graveyard, and no previous part of the effect does either. **CR 113.6m
therefore makes the ability function only from the graveyard.** (CR 113.6b would reach the same
place for a card that spelled the zone out; CR 113.6m is the rule that gets there from the
printed text alone, and is the rule the `trigger_zone` DSL field is a shorthand for.)
CR 113.6k does not apply: this trigger condition *can* trigger from the battlefield in general —
it is 113.6m, not the trigger condition, that confines it.

> **Answer to the question the brief asks explicitly**: when Nether Traitor is **on the
> battlefield** and another creature dies, **nothing happens**. The ability is not functioning.
> This is CR 113.6 + CR 113.6m, not intuition — and it is what the engine gets wrong today.

### 1.2 What kind of trigger this is, and that it looks back in time

> **603.6.** Trigger events that involve objects changing zones are called "zone-change
> triggers." … The most common zone-change triggers are enters-the-battlefield triggers and
> leaves-the-battlefield triggers.
>
> **603.6a** Enters-the-battlefield abilities trigger when a permanent enters the battlefield.
> These are written, "When [this object] enters, . . ." or "Whenever a [type] enters, . . ."
> Each time an event puts one or more permanents onto the battlefield, all permanents on the
> battlefield (including the newcomers) are checked for any enters-the-battlefield triggers that
> match the event.
>
> **603.6c** Leaves-the-battlefield abilities trigger when a permanent moves from the
> battlefield to another zone … These are written as, but aren't limited to, "When [this object]
> leaves the battlefield, . . ." or **"Whenever [something] is put into a graveyard from the
> battlefield, . . ."** (See also rule 603.10.) …
>
> **603.6e** Some Auras have triggered abilities that trigger on the enchanted permanent leaving
> the battlefield. These triggered abilities can find the new object that permanent card became
> in the zone it moved to; they can also find the new object the Aura card became in its owner's
> graveyard after state-based actions have been checked. See rule 400.7.

> **603.10.** Normally, objects that exist immediately after an event are checked to see if the
> event matched any trigger conditions … However, some triggered abilities are exceptions to
> this rule; the game **"looks back in time"** to determine if those abilities trigger, using
> the **existence of those abilities** and the appearance of objects **immediately prior to the
> event**. The list of exceptions is as follows:
>
> **603.10a** Some zone-change triggers look back in time. These are **leaves-the-battlefield
> abilities**, abilities that trigger when a card leaves a graveyard, and abilities that trigger
> when an object that all players can see is put into a hand or library.

**Derivation.** Nether Traitor's trigger condition is verbatim CR 603.6c's second example, so it
is a **leaves-the-battlefield ability**, so by **CR 603.10a it looks back in time** — and the
look-back is over *the existence of the ability* immediately prior to the event. Immediately
prior to the event Nether Traitor was on the battlefield, where (CR 113.6m) the ability did not
exist as a functioning ability. **Hence the simultaneity ruling** (§1.5) follows from the CR and
is not a special case.

**Contrast, and this contrast is load-bearing for §3.4**: Bloodghast's landfall is an
**enters-the-battlefield** ability (CR 603.6a), which is **not** in CR 603.10a's list, so it uses
CR 603.10's normal rule — objects existing **immediately after** the event. A Bloodghast that
arrives in the graveyard in the same batch as a land entering **is** in the graveyard immediately
after the event and **does** trigger. The two paths must therefore behave differently, and the
guard §3.4 adds must be applied to the death arm **only**.

### 1.3 Trigger timing and control

> **603.2.** Whenever a game event or game state matches a triggered ability's trigger event,
> that ability automatically triggers. The ability doesn't do anything at this point.
>
> **603.3.** Once an ability has triggered, its controller puts it on the stack as an object
> that's not a card the next time a player would receive priority. …
>
> **603.3a** A triggered ability is controlled by the player who controlled its source at the
> time it triggered, unless it's a delayed triggered ability. …
>
> **603.3b** If multiple abilities have triggered since the last time a player received
> priority, the abilities are placed on the stack in a two-part process. First, each player, in
> APNAP order, puts each triggered ability they control with a trigger condition that isn't
> another ability triggering on the stack in any order they choose. …
>
> **603.3d** The remainder of the process for putting a triggered ability on the stack is
> identical to the process for casting a spell listed in rules 601.2c–d. …

> **108.4.** A card doesn't have a controller unless that card represents a permanent or spell …
>
> **108.4a** If anything asks for the controller of a card that doesn't have one (because it's
> not a permanent or spell), **use its owner instead**.

**Derivation.** CR 603.3a asks for the controller of the source; a graveyard card has none;
CR 108.4a says use its owner. The existing code's `owner` binding at `abilities.rs:7124`/`:7128`
and its use as the `PendingTrigger` controller at `:7213-7217` is therefore **CR-correct as
written** — the new death arm must reuse it verbatim, not invent a controller.

### 1.4 Object identity

> **400.7.** An object that moves from one zone to another becomes a new object with no memory
> of, or relation to, its previous existence. This rule has the following exceptions.

**Derivation, with the engine consequence spelled out.** The dying creature has **two**
`ObjectId`s in the event: `object_id` (the pre-death **battlefield** id) and `new_grave_id` (the
post-death **graveyard** id). Nether Traitor's graveyard object id (`obj_id` in
`collect_graveyard_carddef_triggers`) lives in the same id space as `new_grave_id`, **not**
`object_id`. An `exclude_self` comparison written against `object_id` would be comparing a
graveyard id to a battlefield id and would **never** match — i.e. it would silently fail open.
§3.3 requires the comparison against `new_grave_id`, with `object_id` compared as well (free,
and correct in the CR 400.7 sense that neither identity may be the source).

### 1.5 Nether Traitor — printed text and the two rulings that decide the edge cases

MCP `lookup_card("Nether Traitor")`:

- Mana cost `{B}{B}`, Creature — Spirit, 1/1, keywords `["Shadow","Haste"]`.
- Oracle: *"Haste / Shadow (…) / **Whenever another creature is put into your graveyard from the
  battlefield, you may pay {B}. If you do, return this card from your graveyard to the
  battlefield.**"*

The two rulings this batch is decided by:

> **[2021-03-19]** *If Nether Traitor and another creature are put into your graveyard at the
> same time, Nether Traitor's ability won't trigger. This is because **it must be in your
> graveyard before the creature dies** in order for its ability that returns it to the
> battlefield to trigger.*

> **[2021-03-19]** *If multiple creatures are put into your graveyard at the same time, Nether
> Traitor's ability **triggers for each of them**. Once you return it to the battlefield, you may
> pay {B} for the other abilities as they resolve, but they'll have no effect if you do. Even if
> Nether Traitor returns to your graveyard, it's considered a new object and won't be returned.*

Both are consequences of §1.1 + §1.2 + §1.4, not extra rules. Ruling 1 is the CR 603.10a
look-back; ruling 2 is *N* separate `CreatureDied` events (so *N* triggers — the per-event loop
gives this for free) plus CR 400.7 (the later triggers find nothing to return).

**Per the standing memory note, CR text is authoritative and rulings are used only to find edge
cases to test.** Here the rulings agree with the CR derivation and are cited as the test oracle,
not as the rule.

---

## 2. The decision: fix shape for OOS-DX1-3

### 2.1 Recommendation — **(a), the narrow fix, in a structural form**

**Take (a).** Do **not** add `trigger_zone` to the runtime `TriggeredAbilityDef`.
Realise (a) as **an extraction plus a single filter at one call site**, not as 40 repeated
`continue` guards.

### 2.2 Why not (b) — the field on `TriggeredAbilityDef`

Four reasons, each independently sufficient:

1. **It would be a stored-and-never-read field.** Every consumer of the runtime
   `triggered_abilities` vector is battlefield-scoped by construction:
   `collect_triggers_for_event` iterates `obj.zone == ZoneId::Battlefield` objects
   (`abilities.rs:4848-4853` for the death family, and the same shape everywhere else), and the
   graveyard dispatch does not read the runtime vector at all — it reads the card registry
   (`abilities.rs:7132-7135`). A `trigger_zone` on `TriggeredAbilityDef` would be written by the
   lowering and consulted by nobody. That is exactly the **`OOS-DP10-5` / DP-24
   "accepted-and-discarded" class** this project already has a seed for; shipping a fresh
   instance of it to close a different seed is a bad trade.
2. **It costs both fingerprints.** `TriggeredAbilityDef` is reachable from `Characteristics`
   (`triggered_abilities: Vec<TriggeredAbilityDef>`), which is a `CLOSURE_MUST_CONTAIN` entry —
   this is exactly PB-DX1 §3.5's argument, and PB-DX1 measured it: adding `InterveningIf::CardDef`
   moved **PROTOCOL 31 → 32 and HASH 68 → 69**. Adding a field here would move both again, for a
   field nothing reads.
3. **PB-DX1's (a′)-vs-(a) argument does not carry over.** (a′) was chosen there because the
   datum had to reach a *reader* (`check_intervening_if`) and the compiler could be made to force
   it. There is no reader here, so there is nothing for the compiler to force, and the
   construction-site churn (~140 `TriggeredAbilityDef` literals) buys nothing.
4. **It answers the wrong question.** The bug is not "the runtime type cannot represent the
   zone"; it is "an ability that does not function on the battlefield is being installed on a
   battlefield object". Deleting the entry is the honest encoding of "this ability is not here".

### 2.3 Why (a) is *not* the same mistake as dispatch style (b) that PB-DX1 rejected

PB-DX1 §3.3 rejected re-routing **34 battlefield trigger conditions** onto registry dispatch, for
four reasons. Each is checked here and **none applies to a graveyard source**:

| PB-DX1 §3.3 objection | Does it apply to PB-DX24? |
|---|---|
| 1. Discards layer resolution — Humility/Dress Down (CR 613.1f) would stop suppressing triggers | **No.** CR 613.1 continuous effects that remove abilities apply to *permanents*; the source here is a graveyard card, which no ability-removal effect in this corpus reaches. Reading the printed card **is** the correct source of truth for a graveyard object, and `collect_graveyard_carddef_triggers` has read the registry since PB-35 for exactly that reason. |
| 2. Discards the runtime filters (`etb_filter`, `death_filter`, …) the lowering exists to carry | **No** — because §3.3 re-derives the death filter **from the card-def `TriggerCondition` itself** at the dispatch site, mirroring the battlefield arm clause for clause. The information is not lost; it is read one link earlier. |
| 3. Breaks tokens and copies (`def.abilities` is the wrong truth for them) | **No.** A token that dies ceases to exist (CR 111.7 / SBA 704.5d) and cannot be a graveyard trigger source; a graveyard *card* always has a `card_id`, and the code already `continue`s when `card_id` is `None` (`:7129-7131`). |
| 4. The model carries an index-space asymmetry (`def.abilities` vs `effective_abilities`) | **Yes — and this batch closes it.** That asymmetry *is* `OOS-DX1-4`, §4 below. The objection is answered by fixing it, not by avoiding the style. |

So the rejection of style (b) was right **for battlefield triggers** and does not extend to a
source that is, by CR 113.6b/113.6m, not on the battlefield at all. This paragraph exists so the
next reader does not re-derive it, and so nobody "corrects" §3 back toward the runtime vector.

### 2.4 The mechanism — how the skip is made structurally uniform

A per-arm `continue` repeated 40 times is exactly the shape that rots (a 41st arm silently
reintroduces the loss). The mechanism:

**Extract the trigger-lowering region into its own function whose input can never contain a
zone-scoped ability.**

- The region is contiguous: `replay_harness.rs:2527` (the `WhenDies` comment) through `:3862`
  (close of the `WhenBecomesTarget` loop), immediately before the `(mana_abilities,
  activated_abilities, triggered_abilities)` return at `:3863`. Everything before `:2527` is the
  two activated/mana loops, which must keep seeing the **unfiltered** slice (a graveyard-activated
  ability such as Reassembling Skeleton's is carried by `activation_zone`, a different field, and
  must not be dropped).
- New private function:

  ```
  fn build_face_triggered_abilities(abilities: &[&AbilityDefinition]) -> Vec<TriggeredAbilityDef>
  ```

  containing the region verbatim, with `let mut triggered_abilities = Vec::new();` at the top and
  `triggered_abilities` returned at the bottom.
- New predicate, written as an **exhaustive match on `TriggerZone`** so a future variant is a
  compile error rather than a silent misclassification (the SR-5 idiom):

  ```
  /// CR 113.6b / CR 113.6m: an ability that states which zone it functions in
  /// functions only from that zone, so it must never be lowered onto the
  /// battlefield object's runtime trigger vector. Exhaustive on `TriggerZone`
  /// deliberately: a new variant must be classified here, not defaulted.
  fn lowers_onto_the_battlefield(ability: &AbilityDefinition) -> bool
  ```

  returning `false` for `AbilityDefinition::Triggered { trigger_zone: Some(z), .. }` where `z`
  matches `TriggerZone::Graveyard` (the only variant today,
  `card_definition.rs:4385-4388`), `true` otherwise.
- Single call site inside `build_face_ability_vectors`, after the two activated/mana loops:

  ```
  let battlefield_triggers: Vec<&AbilityDefinition> =
      abilities.iter().filter(|a| lowers_onto_the_battlefield(a)).collect();
  let triggered_abilities = build_face_triggered_abilities(&battlefield_triggers);
  ```

- **Delete** the per-arm guard at `:3049-3051` and rewrite the comment at `:3029-3032` — one
  mechanism, not two. (Leaving it would be harmless but would re-teach the wrong pattern; the
  differential gate T7 in §6 proves the deletion is safe.)

**Compile note the runner must verify, not assume**: every one of the 40 loops is
`for ability in abilities { if let AbilityDefinition::Triggered { .. } = ability { … } }`. With
the parameter typed `&[&AbilityDefinition]`, `ability` binds as `&&AbilityDefinition`; Rust's
default binding modes peel through both references, so each `if let` and every field binding
inside it compiles **unchanged**, giving **zero per-arm edits**. If any arm turns out to use
`ability` for something other than the `if let` scrutinee, or the ergonomics do not hold, the
fallback is to take `abilities: &[AbilityDefinition]` and pass a filtered
`Vec<AbilityDefinition>` (a clone — behaviourally identical, slightly more allocation). **Stop
and report** if a third shape is needed.

### 2.5 The index-space check the brief demands

**Claim to verify before editing, not after**: filtering changes the **runtime**
`characteristics.triggered_abilities` index space (entries are removed) and does **not** touch the
**card-def** `def.abilities` index space that `PendingTriggerKind::CardDefETB` depends on.

Why it holds:

- Every `CardDefETB` `ability_index` is produced by a `for (idx, ability) in
  def.abilities.iter().enumerate()` loop in `rules/abilities.rs` (Q1–Q7 of stage 0 §3). **None of
  those loops is inside `build_face_ability_vectors`**, and `def.abilities` is not modified by
  anything here.
- The extracted region contains **no `.enumerate()`, no index arithmetic, and no positional
  assumption** — it only `push`es onto `triggered_abilities` in arm order. The runner must
  confirm this with a scan of the extracted range before landing the change (`rg -n
  '\.enumerate\(\)|\[idx|position\(' ` over the extracted function).
- The only corpus-visible runtime-vector change is `nether_traitor` losing its single lowered
  entry. `bloodghast` is already skipped today; `squee_goblin_nabob`'s
  `AtBeginningOfYourUpkeep` has no lowering arm at all. **T7 (§6) proves this over all 1,803
  defs**, so the claim is measured, not argued.
- `OOS-DP6-2` (a runtime index consumed as a card-def index at `abilities.rs:6252`) is a
  **pre-existing** mismatch on the `WheneverYouSacrifice` retain filter. It is not made worse
  here: the two lists were already not in bijection (the runtime vec omits every
  non-lowered ability), and `nether_traitor` has no `WheneverYouSacrifice`. Runner: confirm by
  reading that site, record the confirmation, do **not** widen scope to it.

### 2.6 Source gate — feasible, and here is how it is written

**Yes, feasible, and it is the reason the extraction is worth doing.** Two gates, in
`crates/engine/tests/core/pb_dx24_trigger_zone_roster.rs`:

- **G-A (structural)**: read `crates/engine/src/testing/replay_harness.rs`, strip **line and
  block** comments (PB-DX32's M8 lesson: a `/* … */`-wrapped row defeated a line-comment-only
  scanner while every probe stayed green), locate `fn build_face_triggered_abilities` and take its
  body by brace matching, then assert **`trigger_zone` occurs zero times inside that body**.
  A 41st arm that destructures `trigger_zone` — the only way an arm can swallow it — fails the
  gate. Failure message must say: *"the trigger-lowering function must never see `trigger_zone`;
  the CR 113.6b/113.6m filter lives at its single call site in `build_face_ability_vectors`
  (`lowers_onto_the_battlefield`). Do not add a per-arm guard — extend the filter."*
- **G-B (call-site uniqueness)**: assert `build_face_triggered_abilities(` appears exactly
  **twice** in the stripped file (definition + one call) and that the call's argument is the
  filtered binding by name (`battlefield_triggers`). A second, unfiltered call site is the other
  way the invariant can be lost.

Both gates are proven by executing a revert (§6 T8/T9).

### 2.7 The wire prediction, stated as falsifiable

> No type declaration changes: `TriggeredAbilityDef` gains no field, `PendingTrigger` gains no
> field, no `Command`/`GameEvent`/`Effect` variant is added, `TriggerZone` is unchanged.
> **Therefore `core protocol_schema` and `core hash_schema` must both stay green with
> PROTOCOL 35 / HASH 73.**
>
> **Falsifier**: if either gate reddens, a type in the wire/hash closure was changed by this
> batch without the plan noticing. **Stop, read the failure text, report it** — do not hand-edit
> a constant. Both values are read out of the gate's own output, never predicted into the pin.

---

## 3. Engine changes

### Change 1 — the lowering learns the filter (OOS-DX1-3, half 1)

**File**: `crates/engine/src/testing/replay_harness.rs`
**Action**: as designed in §2.4 — add `lowers_onto_the_battlefield`, extract
`build_face_triggered_abilities`, filter at the single call site, delete the per-arm guard at
`:3049-3051` and its comment at `:3029-3032`.
**CR**: 113.6b / 113.6m — an ability that states its zone functions only from that zone.
**Callers of `build_face_ability_vectors` that must be unaffected** (verify by compiling, and by
`cargo test -p mtg-engine --test core face_dereg_parity`): `enrich_spec_from_def`
(`replay_harness.rs:3950`), `rules/face.rs:104` (`apply_face_change`), `rules/resolution.rs:888`
(the disturb back-face rebuild). All three pass a `&[AbilityDefinition]`; all three keep working
because the filter is inside `build_face_ability_vectors`, not at their call sites — which is
exactly why the filter must live there and not in the callers.

### Change 2 — the lossy-lowering table row (OOS-DX1-3, the doc half)

**File**: `crates/engine/src/testing/replay_harness.rs:2448` (the `trigger_zone` row of the table
at `:2440-2448`).

Replace the current row:

```
/// | `trigger_zone`     | **no runtime home** — dropped; `collect_triggers_for_event` scans the battlefield only; the graveyard sweep is a separate registry path. Seeded OOS-DX1-3 |
```

with exactly:

```
/// | `trigger_zone`     | **honoured (PB-DX24)** — an ability carrying `trigger_zone: Some(_)` is filtered out of the input to `build_face_triggered_abilities` at its single call site (`lowers_onto_the_battlefield`), so no lowering arm can install it on a battlefield object. CR 113.6b/113.6m: it functions only from the named zone. Its dispatch lives in `rules::abilities::collect_graveyard_carddef_triggers`, which reads the card registry directly. Closes OOS-DX1-3; `core::pb_dx24_trigger_zone_roster` fails if a future arm re-swallows the field. |
```

**Also correct the arm count in the same doc block.** Lines `:2445`/`:2446` say "all 34 push
sites" / "31 of 34 sites"; stage 0 re-measured **40** arms at HEAD. Rewrite those two cells to
state the count **and the counting rule** ("`trigger_condition:` match arms inside
`build_face_triggered_abilities`, measured at PB-DX24"), so the next reader can re-derive rather
than trust — the PB-DX19 "three published counts" lesson.

### Change 3 — the graveyard dispatch learns `WheneverCreatureDies` (OOS-DX1-3, half 2)

**File**: `crates/engine/src/rules/abilities.rs`

**3.1 New parameter on `collect_graveyard_carddef_triggers`** (`:7112`):

```
    arrived_in_graveyard_this_batch: &std::collections::HashSet<ObjectId>,
```

**3.2 The batch-arrival set**, computed **once** at the top of `check_triggers`
(`abilities.rs:2962-2964`, before the `for event in events` loop), mirroring the existing
`left_battlefield` idiom at `:6534-6556`:

```
// CR 603.10a: a leaves-the-battlefield ability looks back in time — the game asks
// whether the ability EXISTED immediately prior to the event. A card that arrived in a
// graveyard as part of THIS event batch was on the battlefield immediately prior, where
// (CR 113.6m) its graveyard-zone ability did not function. Gatherer, Nether Traitor:
// "If Nether Traitor and another creature are put into your graveyard at the same time,
// Nether Traitor's ability won't trigger."
```

Collect `new_grave_id` from every `GameEvent` in `events` that carries one — at minimum
`CreatureDied`, `PermanentDestroyed`, `PermanentSacrificed`, `AuraFellOff`. **The runner must
enumerate the real set by matching on `GameEvent` rather than trusting this list**, and must state
in the execution notes which variants were included and which were excluded and why.

**3.3 The new `fires` arm** in `collect_graveyard_carddef_triggers`'s match at `:7146`, added
beside the existing `PermanentEnteredBattlefield` arm:

```
GameEvent::CreatureDied {
    object_id: pre_death_id,
    new_grave_id,
    controller: death_controller,
    pre_death_characteristics,
    ..
} => match trigger_condition {
    TriggerCondition::WheneverCreatureDies { controller, exclude_self, nontoken_only, filter } => { … }
    _ => false,
},
```

The body **mirrors the battlefield arm at `:4866-4916` clause for clause** — do not re-invent it,
and put a comment naming that line range so a future edit to one is visibly an edit to the other:

| clause | battlefield arm | graveyard arm (this change) | CR |
|---|---|---|---|
| `controller_you` | `dying_controller != obj.controller` → skip | `*death_controller != owner` → skip | 108.4a: a graveyard card's controller is its owner; `owner` is already bound at `:7124`/`:7128` |
| `controller_opponent` | `dying_controller == obj.controller` → skip | `*death_controller == owner` → skip | 108.4a |
| `exclude_self` | `dying_obj_id == obj_id` → skip | `*new_grave_id == obj_id \|\| *pre_death_id == obj_id` → skip | **400.7** — §1.4: `obj_id` is a graveyard id, so `new_grave_id` is the comparison that can match; `pre_death_id` is free and correct |
| `nontoken_only` | `dying_is_token` → skip | same, read `state.objects.get(new_grave_id).is_some_and(\|o\| o.is_token)` | 111.7 |
| `filter` (`triggering_creature_filter`) | `filter.is_token && !dying_is_token` → skip, then `matches_filter(pre_death_characteristics.unwrap_or(graveyard chars), filter)` | identical | 603.10a (LKI characteristics), 613.1d |
| **look-back** | *(n/a — a battlefield observer is trivially present)* | **`arrived_in_graveyard_this_batch.contains(&obj_id)` → skip** | **603.10a** + the Gatherer ruling (§1.5) |

**The look-back guard is applied on this arm only, never on the `PermanentEnteredBattlefield`
arm** — §1.2's contrast: an ETB trigger is not in CR 603.10a's list, so Bloodghast arriving in the
graveyard in the same batch as a land entering **does** trigger. Write that reason at the guard;
it is the one place a future reader will be tempted to "unify" the two arms and be wrong.

**Owner-vs-controller deviation, stated not hidden.** Printed text is *"put into **your**
graveyard"*, which is an **ownership** condition on the dying card; the DSL's
`TargetController::You` is a **controller** condition, and this arm keeps the controller reading
so that one DSL field has one meaning at both dispatch sites. `nether_traitor.rs:30-34` already
documents this and it is the allowlisted class in `core::completeness_deviation_scan`
(`OOS-DX4-1`). Add a comment at the new clause citing `OOS-DX4-1` and
`nether_traitor.rs:30-34`; **do not** "improve" it here by reading the dying object's `owner` —
that would give the same DSL field two different meanings at two sites, which is the defect class
this whole batch exists to remove.

**3.4 The intervening-if gate** at `:7199-7206` is already threaded
(`carddef_intervening_if_holds_at_queue_time(state, intervening_if.as_ref(), owner, obj_id)`,
CR 603.4) and is **outside** the `fires` match, so it covers the new arm with no edit. Confirm by
reading, and say so in the execution notes rather than assuming.

**3.5 The push**:

```
triggers.push(PendingTrigger {
    ability_index: idx,                                   // card-def index (CardDefETB space)
    triggering_event: Some(TriggerEvent::AnyCreatureDies), // CR 603.2d doubler matching
    entering_object_id: Some(*new_grave_id),               // mirrors the battlefield arm's reuse
    ..PendingTrigger::blank(obj_id, owner, PendingTriggerKind::CardDefETB)  // CR 108.4a/603.3a
});
```

**3.6 The new call site**: inside `check_triggers`'s `GameEvent::CreatureDied` arm
(`abilities.rs:4552`), immediately **after** the battlefield `AnyCreatureDies` block closes at
`:4947`:

```
// CR 113.6b/113.6m (PB-DX24): a `trigger_zone: Some(Graveyard)` death trigger fires from
// the graveyard, not from the battlefield. Mirrors the ETB call at `:2988`.
collect_graveyard_carddef_triggers(state, &mut triggers, event, Some(*new_grave_id), &arrived);
```

The existing call at `:2988` gains the new `&arrived` argument (the set is empty of ETB ids by
construction, and the guard is not consulted on that arm anyway — see 3.3).

**3.7 Resolution trace — traced, not guessed.** For the pushed trigger:

1. `flush_pending_triggers` reads `once_per_turn` at `abilities.rs:8105-8151`. For
   `kind != Normal` the runtime lookup is skipped (`:8118-8125`) and the **card-registry
   fallback** at `:8126-8147` reads `def.effective_abilities(obj.is_transformed)[ability_index]`
   — a card-def index, which is what we pushed, and `is_transformed` is `false` for a graveyard
   object (`state/mod.rs:1557-1558`, "DFC transform state is reset on zone change", CR 712.8a).
   So `once_per_turn` is honoured on this path already. `nether_traitor` has
   `once_per_turn: false`; the mechanism is nonetheless correct for a future card.
2. CR 603.2d doubling: `compute_trigger_doubling` → `doubler_applies_to_trigger`'s
   `TriggerDoublerFilter::CreatureDeath` arm (`abilities.rs:9884-9893`) matches on
   `triggering_event ∈ {SelfDies, AnyCreatureDies}` **only**. **Runner action**: read
   `compute_trigger_doubling` and determine whether it additionally scopes the doubler to the
   trigger *source*. If a death doubler would now double a **graveyard-sourced** trigger, that is
   almost certainly wrong (real doublers say "abilities of permanents you control") — **file a
   seed `OOS-DX24-n` with the measurement and do not fix it in this batch**; scope discipline.
   If it is already scoped, record that it is.
3. The stack object: `PendingTriggerKind::CardDefETB` is turned into its `StackObjectKind` in the
   same `flush_sorted` match as the existing graveyard ETB triggers — no new arm.
4. `resolution.rs:2206-2236` (the `is_carddef_etb` branch) reads the effect from the registry via
   `def.effective_abilities(obj.is_transformed).get(ability_index)`, yielding
   `Effect::MayPayThenEffect { cost: Cost::Mana({B}), payer: PlayerTarget::Controller,
   then: Effect::MoveZone { target: EffectTarget::Source, to: Battlefield { tapped: false } } }`
   (`nether_traitor.rs:41-52`). `PlayerTarget::Controller` resolves to the trigger's controller,
   which is `owner` per 3.5. `EffectTarget::Source` is the graveyard object, which `MoveZone`
   moves to the battlefield — **the identical shape Bloodghast already resolves through today**
   (`bloodghast.rs:59-63`), which is why this needs no resolution-side change.

**Runner obligation**: walk steps 1–4 in the source and record in
`memory/primitives/pb-DX24-execution-notes.md` what each one actually says. If any step diverges
from this trace, **stop and report** — a trace that turns out wrong is news, not something to
route around.

### Change 4 — the two index spaces (OOS-DX1-4)

See §4 for the per-site disposition. Files: `crates/engine/src/rules/abilities.rs` (Q1, Q2, Q3,
Q4, Q6, Q7), `crates/engine/src/rules/resolution.rs` (Q5 comment only).

### Change 5 — exhaustive-match / construction-site sweep

**There is none, and that is a claim to verify rather than assume.** No enum gains a variant, no
struct gains a field, so no exhaustive match anywhere in the workspace needs a new arm. The
runner must nevertheless run the standard sweep and report it empty:

| file | what to check | expected |
|---|---|---|
| `crates/engine/src/state/hash.rs` | `TriggeredAbilityDef` / `PendingTrigger` / `PendingTriggerKind` hash impls | **unchanged** |
| `crates/engine/src/rules/protocol.rs` | wire closure list | **unchanged** |
| `crates/view-model/src/lib.rs` | `stack_kind_info` (`StackObjectKind`), `format_keyword` (`KeywordAbility`) | **unchanged** — no new variant |
| `tools/tui/src/play/panels/stack_view.rs` | `StackObjectKind` match | **unchanged** |
| `crates/engine/src/state/keyword_registry.rs` | SR-5 `handling` classification | **see §7** |
| `crates/simulator/`, `tools/play-server/`, `tools/tui/` | any source line | `git diff main..HEAD --numstat` over these must be **EMPTY** |

If any of these is non-empty at the end, the design drifted — stop and report.

---

## 4. OOS-DX1-4 — per-site disposition

### 4.0 The invariant that decides most of it, established at HEAD

`is_transformed == true` is reachable on **battlefield permanents only**:

- It is set `true` in exactly one place: `resolution.rs:853` (disturb — a permanent entering
  back-face-up), on the battlefield object.
- It is otherwise flipped only by `rules/face.rs::apply_face_change`, whose doc (`face.rs:42-63`)
  says "flip a **battlefield permanent's** `is_transformed` flag" and whose module doc (`:26-29`)
  makes it the only permitted mutator.
- It is reset to `false` on **every** zone change: `state/mod.rs:1557-1558`, comment
  *"CR 712.8a / CR 400.7: DFC transform state is reset on zone change."*

Consequence: a **stack** object and a **graveyard** object always have `is_transformed == false`,
so at those sites `effective_abilities(is_transformed)` is *definitionally* `abilities`.

### 4.1 The dispositions

| # | site | source object's zone | `is_transformed` reachable? | disposition | reason |
|---|---|---|---|---|---|
| **Q1** | `abilities.rs:3147` — Backup (CR 702.165a) | battlefield (just entered) | **yes** — a disturb permanent enters with `is_transformed: true` | **FIX** | Both the `.enumerate()` and the `def.abilities[idx+1..]` slice must come from **one** binding (`let eff = def.effective_abilities(obj.is_transformed);` then `eff.iter().enumerate()` and `&eff[idx+1..]`) so index and slice can never diverge. CR 702.165a's "printed below this one" is a property of the **visible face**. |
| **Q2** | `abilities.rs:3764` — `WhenYouCastThisSpell` | **stack** | **no** (§4.0) | **FIX anyway** | The two ends are *accidentally* equal, not structurally. `stack_obj` is already in hand at `:3760`; using `def.effective_abilities(stack_obj.is_transformed)` makes queue and read (`resolution.rs:2216`) the **same expression**, so the pair stops depending on a distant invariant. **Zero behaviour change** — assert that in the commit message and prove it with T7's differential. |
| **Q3** | `abilities.rs:4119` — `WhenExertedAsAttacks` | battlefield | **yes** | **FIX** | An attacking transformed DFC is ordinary. Read side is `resolution.rs:2216` (`effective_abilities`), so today the pair genuinely disagrees. |
| **Q4** | `abilities.rs:5157` — `WhenDealsCombatDamageToPlayer` | battlefield | **yes** | **FIX** | Same as Q3. |
| **Q5** | `abilities.rs:6080` — face-down turn-up (`WhenTurnedFaceUp`) | battlefield | **no** — **CR 712.2**: a transforming double-faced card **can't be turned face down**, so a `PermanentTurnedFaceUp` source is never a transformed DFC | **RE-SCOPE — no code change** | Its read side is **also** `def.abilities` (`resolution.rs:7663`, with the comment at `:7665-7676` already saying so). The pair is self-consistent, and the shared face-blindness is unreachable. **Action: correct the comment at `resolution.rs:7665-7676`** to record that PB-DX24 measured the reachability and closed the *other* six sites, so the note stops reading as an open TODO. |
| **Q6** | `abilities.rs:6135` — `WheneverRingTemptsYou` | battlefield | **yes** | **FIX** | `obj_id` is in hand; the object is fetched at `:6130`. |
| **Q7** | `abilities.rs:7135` — the graveyard sweep (and this batch's new death arm) | **graveyard** | **no** (§4.0) | **FIX anyway** | Same argument as Q2 — and stronger here, because this batch is *adding* a second dispatch shape to the same loop. Make the expression uniform now rather than leaving the new arm resting on a reset-on-zone-change invariant three files away. |

**Fixed: Q1, Q2, Q3, Q4, Q6, Q7. Re-scoped with a comment correction: Q5.**
Every fix is the same one-token substitution `def.abilities` →
`def.effective_abilities(<obj>.is_transformed)`, except Q1 which additionally shares one binding
between the enumerate and the slice.

### 4.2 The measurement that settles reachability (SR-36 — enumerate, never grep)

**Stage 1 work, before any Q-site edit.** Write a throwaway probe (or the roster gate itself) that
enumerates `all_cards()` and, for each def with `back_face: Some(face)`, scans `face.abilities`
for each shape below. Record every count in the execution notes, including the zeros.

| shape looked for on the **back** face | site it makes live |
|---|---|
| `AbilityDefinition::Keyword(KeywordAbility::Backup(_))` | Q1 |
| `Triggered { trigger_condition: WhenYouCastThisSpell, .. }` | Q2 |
| `Triggered { trigger_condition: WhenExertedAsAttacks, .. }` | Q3 |
| `Triggered { trigger_condition: WhenDealsCombatDamageToPlayer { .. }, .. }` | Q4 |
| `Triggered { trigger_condition: WhenTurnedFaceUp, .. }` | Q5 |
| `Triggered { trigger_condition: WheneverRingTemptsYou, .. }` | Q6 |
| `Triggered { trigger_zone: Some(_), .. }` | Q7 |

**The fixes land regardless of the counts** — they are one-token uniformity edits with zero
behaviour change where the count is 0, and a real repair where it is not. The measurement decides
**what the probes can be written against** (a real corpus DFC vs a synthetic `CardFace` fixture),
and it is what the roster gate pins.

### 4.3 The roster gate, with a non-vacuity floor

**File**: `crates/engine/tests/core/pb_dx24_trigger_zone_roster.rs` (same file as §2.6's source
gates).

- **R1** — the `trigger_zone: Some(_)` population, pinned **by symbol** (card **names** read off
  `all_cards()`, not file paths): exactly `{"Bloodghast", "Squee, Goblin Nabob", "Nether
  Traitor"}`, count **3**. Non-vacuity floor: `all_cards().len() >= 1_700` asserted in the same
  test, so a broken enumeration cannot make an empty roster look correct. Failure message tells a
  future author: *a new `trigger_zone` def must be added here AND must have a dispatch arm in
  `collect_graveyard_carddef_triggers`, or it will silently never fire.*
- **R2** — the §4.2 back-face population, pinned per site with the measured numbers.
  **A roster pinned empty rots silently**, so R2 carries its own non-vacuity floor: the number of
  defs with `back_face: Some(_)` at all, asserted `> 0` and pinned at the measured value, with a
  message saying which number moved.

### 4.4 The DFC back-face probes for the fixed sites

For every Q-site whose §4.2 count is **> 0**, a probe using the real corpus card. For every
Q-site whose count is **0**, a probe using a **synthetic** `CardDefinition` with a `CardFace`
back face carrying the ability — the exact idiom `pb_rs4_face_aware_residuals.rs` already uses
(`crates/engine/tests/primitives/pb_rs4_face_aware_residuals.rs:30-46` for the imports;
that file is the precedent for this whole half of the batch and must be read before writing
these). Each probe: put the permanent on the battlefield with `is_transformed: true` via
`apply_face_change` (never by poking the field — `face.rs:26-29`), fire the site's event, assert
the trigger fires with the **back**-face ability's `ability_index`. Revert for each: restore
`def.abilities` at that one site; the probe must redden.

---

## 5. Card definition changes

### `crates/card-defs/src/defs/nether_traitor.rs`

**Comment-only. `completeness` stays `Complete`. Zero DSL changes.**

The def is already correct — `trigger_zone: Some(TriggerZone::Graveyard)` is exactly right and
was right when written; the engine was not reading it. Add to the header comment (and/or above
the `trigger_zone` line):

- CR 113.6m: the ability functions **only** from the graveyard, because its effect moves the card
  out of the graveyard and its trigger condition does not put it there.
- The engine honours this as of PB-DX24 (`scutemob-202`) — before it, the ability was lowered onto
  the battlefield object and never dispatched from the graveyard.
- The Gatherer simultaneity ruling (§1.5) and where it is enforced
  (`check_triggers`'s `arrived_in_graveyard_this_batch`).

Leave `:30-34`'s owner-vs-controller note **exactly as is** (it is accurate and is the
`OOS-DX4-1` allowlist's counterpart).

**SR-35**: `cargo fmt --check` does **not** check card defs. Run `tools/check-defs-fmt.sh` after
this edit — a comment rewrap is precisely the class it catches (PB-DX19 hit this).

**No other def is touched.** `bloodghast` and `squee_goblin_nabob` keep their markers; neither
becomes correct as a result of this batch (Bloodghast's `partial` is about the missing "you may",
Squee's `known_wrong` is about a trigger condition with no lowering arm and no dispatch arm —
**record explicitly in the execution notes that PB-DX24 does not close either**).

**New card definitions**: none.

---

## 6. Tests — every probe with the revert that must make it fail

**Convention (SR-9a)**: engine integration tests are nine targets under
`crates/engine/tests/<group>/`. **Never** add a top-level `tests/*.rs` —
`tests/no_stray_test_binaries.rs` fails the suite if one appears. New files need a `mod` line:

- `crates/engine/tests/primitives/main.rs` — confirmed at `:1-30`, plain `mod <name>;` lines in
  alphabetical order. Add `mod pb_dx24_trigger_zone_and_index_spaces;` after
  `mod pb_dx23_dredge_tail_and_query;`.
- `crates/engine/tests/core/main.rs` — same shape. Add `mod pb_dx24_trigger_zone_roster;`.

**Architecture Invariant 8**: every test cites its CR section in its doc comment and in its
failure message.

**Every gate below must be watched failing by EXECUTING the revert** (rebuild confirmed in the
captured output — a stale binary that "passes" is the R7 class this project has been bitten by
repeatedly), then restored, with `git diff` confirmed clean before the next one.

### File A — `crates/engine/tests/primitives/pb_dx24_trigger_zone_and_index_spaces.rs`

| id | test | asserts | CR | **revert that must redden it** |
|---|---|---|---|---|
| **T1** | `test_dx24_nether_traitor_does_not_trigger_from_the_battlefield` | Nether Traitor on the battlefield, another creature you control dies → **zero** `PendingTrigger` with `source == nether_id`. Non-vacuity: the same event produces ≥1 trigger overall from a control card (e.g. a Blood-Artist-shaped battlefield death watcher), so a fixture that fires nothing cannot pass. | 113.6 / **113.6m** | Remove the filter at `build_face_ability_vectors`' call site (restore the unfiltered slice) → the lowered `AnyCreatureDies` entry returns → 1 trigger. |
| **T2** | `test_dx24_nether_traitor_triggers_from_the_graveyard` | Nether Traitor in `ZoneId::Graveyard(p1)`, a creature p1 controls dies → **exactly one** trigger with `source == nether_gy_id`, `kind == CardDefETB`, `controller == p1` (owner, CR 108.4a), `triggering_event == Some(AnyCreatureDies)`, `entering_object_id == Some(new_grave_id)`, and `ability_index` equal to the **card-def** index of the Triggered ability (2, asserted by re-deriving it from `all_cards()`, not hard-coded). | 603.6c / 113.6b / **108.4a** | Delete the new `GameEvent::CreatureDied` arm in `collect_graveyard_carddef_triggers` → 0 triggers. |
| **T3** | `test_dx24_nether_traitor_returns_itself_end_to_end` | Full flow through `process_command`: creature dies → trigger flushes → resolves → **Nether Traitor is on the battlefield**. Paired negative in the same test: with no black mana available it stays in the graveyard (so the assertion discriminates the *return*, not merely the trigger). | 603.3 / 603.3a / 118.12 (`MayPayThenEffect`) | Same revert as T2. |
| **T4** | `test_dx24_simultaneous_death_does_not_trigger` | Nether Traitor **and** another creature die in the **same** event batch → **zero** triggers sourced at the Traitor's graveyard object. Non-vacuity: a second sub-case where the Traitor was already in the graveyard fires **one**, from the *same* helper, so the fixture is proven capable of firing. | **603.10a** + the Gatherer ruling (§1.5) | Delete the `arrived_in_graveyard_this_batch` guard → 1 trigger. |
| **T5** | `test_dx24_exclude_self_compares_the_graveyard_identity` | A `CreatureDied` whose `new_grave_id` **is** the Traitor's graveyard object → zero triggers. | **400.7** / 603.10a | Change the `exclude_self` clause to compare only `pre_death_id` (the battlefield id) → fires, because the two id spaces never meet (§1.4). *This revert is the whole point of the test — it fails open, silently, which is why it must be pinned.* |
| **T6** | `test_dx24_graveyard_death_filters_mirror_the_battlefield_path` | (a) an **opponent's** creature dying does **not** trigger Nether Traitor (`controller: Some(You)`); (b) a **token** dying does not trigger a synthetic def with `nontoken_only: true`; (c) a subtype `filter` mismatch does not trigger a synthetic filtered def; (d) each of (a)–(c)'s positive counterpart **does** trigger. | 108.4a / 111.7 / 603.10a / 613.1d | For each half, delete the corresponding clause in the new arm → the negative case fires. |
| **T7** | `test_dx24_lowering_drops_every_zone_scoped_ability_over_the_corpus` | **Differential over all of `all_cards()`**: for each def, the triggered vector lowered from `def.abilities` equals the triggered vector lowered from `def.abilities` with all `trigger_zone: Some(_)` abilities removed (`TriggeredAbilityDef` derives `PartialEq, Eq` — `card-types/src/state/game_object.rs:895`, so `assert_eq!` on the `Vec` is exact). Non-vacuity: assert that at least **1** def (in fact exactly the 3 of R1) has a non-identity *input* under the removal, and that at least one of those has a **non-empty** difference today — otherwise the test is trivially true. | 113.6b / 113.6m | Remove the filter → `nether_traitor`'s two vectors differ → red, naming the card. |
| **T8** | *(in File B, see below)* | source gate G-A | — | Add a `trigger_zone` binding to any arm inside `build_face_triggered_abilities`. |
| **T9** | *(in File B)* | source gate G-B | — | Add a second, unfiltered call to `build_face_triggered_abilities`. |
| **T10..** | one per fixed Q-site (Q1, Q2, Q3, Q4, Q6, Q7) | §4.4's DFC back-face probes | 712.8d/e, 702.165a (Q1), 701.54d (Q6), 113.6b (Q7) | Restore `def.abilities` at that one site. |

**T7's access problem, and its two answers.** `build_face_ability_vectors` is `pub(crate)`
(`replay_harness.rs:2449`) and an integration test is an external crate.
**Preferred**: promote it to `pub` — it lives in `crates/engine/src/testing/`, a module whose
stated purpose is sharing with tests and the replay viewer, and `enrich_spec_from_def` beside it
is already `pub`. **Fallback if that is refused by a gate**: drive it through the public proxy
`enrich_spec_from_def` (`replay_harness.rs:3949-3950` calls
`build_face_ability_vectors(&def.abilities)` and pushes the result onto the `ObjectSpec`), and
compare `ObjectSpec` triggered-ability vectors instead. Pick one, record which, and say why.

### File B — `crates/engine/tests/core/pb_dx24_trigger_zone_roster.rs`

- **G-A / G-B**: the source gates of §2.6. Must strip **line and block** comments before scanning
  (PB-DX32 M8: a `/* … */` wrap defeated a line-comment-only scanner).
- **R1 / R2**: the roster gates of §4.3, both with non-vacuity floors.

### Test count expectation

Roughly **+16 to +20** `#[test]` functions (T1–T7, T10-family × 6, G-A, G-B, R1, R2). The final
number is **measured**, not predicted — record it against the 4,413 baseline.

---

## 7. SR gates and invariants to check explicitly

- **SR-8 (wire)**: no `Command` / `GameEvent` / `Effect` variant is added. §2.7's prediction.
  Gate-execute `--test core protocol_schema` and `--test core hash_schema`, and read the numbers
  out of the output.
- **SR-5 (keyword registry)**: `trigger_zone` is a field on `AbilityDefinition::Triggered`, not a
  `KeywordAbility`, so `state::keyword_registry::handling` is **not** expected to move.
  **But run `cargo test -p mtg-engine --test core keyword_registry` and report the result** —
  PB-DX20 and PB-DX23 were both caught by this gate finding a *handling site* the brief had
  missed, twice in a row. If it reddens, the design touched a keyword's behaviour and the plan is
  wrong; stop and report.
- **SR-4 (silent failures)**: the new `fires` arm's `_ => false` fall-throughs are *classification*
  arms in a predicate, not silent failures — but the new `continue`s must each carry the CR cite
  that justifies them. No `expect_*`/`lki_*` diagnostic is required (nothing is being looked up
  that could be absent), except the `state.objects.get(new_grave_id)` read in the `nontoken_only`
  and `filter` clauses, which must follow whatever the battlefield arm at `:4893-4896` already
  does (it `continue`s on `None`) — mirror it, do not invent.
- **SR-6**: `crates/card-defs` must not gain an engine dependency. The only card-def edit is a
  comment; `git diff --numstat -- crates/card-defs/` should be one file.
- **SR-9a**: no top-level `tests/*.rs`. Both new files go in a group with a `mod` line.
- **SR-35**: `tools/check-defs-fmt.sh` after the `nether_traitor.rs` comment edit.
- **SR-36**: the §4.2 measurement enumerates `all_cards()`. **Never** grep the source for it.
- **Architecture Invariant 1**: no IO added to the engine. The source gates read files, but they
  are *tests*, and they follow `decision_gate.rs`'s existing `read_ct` idiom.

---

## 8. Coverage

**Expected: 0 completeness flips.** `nether_traitor` stays `Complete`; `bloodghast` stays
`partial`; `squee_goblin_nabob` stays `known_wrong`; no def is added or removed.

**How it is proven** — *not* by an empty `crates/card-defs` diff, because this batch mandates a
comment-only def edit:

1. Run `python3 tools/authoring-report.py`.
2. Diff `docs/authoring-status.md`: the **body must be byte-identical**; the only permitted
   differences are the git-sha / date stamp lines. Same for
   `docs/authoring-status-missing.txt` and `docs/authoring-status-prev.json`.
3. Revert the regeneration churn (`git checkout --`) before committing.
4. Expected pin, unchanged: **1,133 / 1,803 = 62.8%**.
5. `tools/check-defs-fmt.sh` must pass on the edited def (SR-35).

---

## 9. Stage list — executable in order, each with its own verification

### Stage 0 — re-verify, do not re-derive
Read `memory/primitives/pb-DX24-stage0.md`. Re-read every cited line at HEAD to confirm it still
says what stage 0 says (PB-DX23 merged since the census was taken on this branch's merge-base;
confirm nothing shifted). Additionally establish, and record:
- the extracted region's exact boundaries (`:2527`/`:2531` to `:3862`);
- that no arm in that region uses `ability` outside the `if let` scrutinee
  (`rg -n 'ability' ` over the range and read the hits);
- that the region contains no `.enumerate()` / index arithmetic (§2.5);
- the `GameEvent` variants that carry a `new_grave_id` (§3.2).
**Verify**: `cargo test --workspace --no-fail-fast` to a file → **4,413 / 0 / 5**, residual list
empty. Record `PROTOCOL_VERSION` and `HASH_SCHEMA_VERSION` read from source **and** confirmed by
executing `--test core protocol_schema` / `--test core hash_schema`.

### Stage 1 — the OOS-DX1-4 measurement (SR-36)
Enumerate `all_cards()` for §4.2's seven shapes on back faces. Record every count including zeros
in `memory/primitives/pb-DX24-execution-notes.md`. Decide, per Q-site, real-corpus vs synthetic
fixture for its probe.
**Verify**: the measurement runs and its output is committed to the execution notes.

### Stage 2 — fail-before for the lowering half
Write **T1** and **T7** against unmodified HEAD and **watch them fail**. Capture the failure text
verbatim (this is the historical record that the bug was real).
**Verify**: `cargo test -p mtg-engine --test primitives pb_dx24` shows both red, with a message
naming Nether Traitor.

### Stage 3 — implement the lowering filter (Change 1 + Change 2)
Extract `build_face_triggered_abilities`, add `lowers_onto_the_battlefield`, filter at the single
call site, delete the per-arm guard at `:3049-3051`, rewrite the table row at `:2448` and the
stale arm counts at `:2445-2446`.
**Verify**: `cargo build --workspace`; `cargo test -p mtg-engine --test primitives pb_dx24` →
T1 + T7 green; `cargo test -p mtg-engine --test core face_dereg_parity` green (the
`apply_face_change` caller); `cargo test -p mtg-engine --test primitives pb_dx1_lowered_intervening_if`
green (the other consumer of this function's contract);
`cargo test --workspace --no-fail-fast` to a file — **no pre-existing test may redden**. If one
does, it was asserting the defect; report it before changing it.

### Stage 4 — fail-before + implement the graveyard death dispatch (Change 3)
Write **T2, T3, T4, T5, T6**, watch each fail, then implement §3.1–§3.6. Walk the §3.7 resolution
trace in the source and record what it actually says.
**Verify**: all of T2–T6 green; `cargo test -p mtg-engine --test primitives pb_l_landfall` green
(the pre-existing Bloodghast graveyard-dispatch coverage must not move — it is the proof that the
new parameter and the new arm did not disturb the ETB path);
`cargo test -p mtg-engine --test mechanics_e_l graveyard_abilities` green.

### Stage 5 — the two index spaces (Change 4)
Fix Q1, Q2, Q3, Q4, Q6, Q7; correct Q5's comment at `resolution.rs:7665-7676`. Write the §4.4
probes; execute a per-site revert for each.
**Verify**: `cargo test -p mtg-engine --test primitives pb_dx24` green; `--test primitives
pb_rs4_face_aware_residuals` green; `--test primitives pb_ac7_ability_index_desync` green (the
closest existing index-space coverage); `--test core` green.

### Stage 6 — the gates (§2.6, §4.3)
Write File B: G-A, G-B, R1, R2. Execute all four reverts, including the **block-comment** variant
for G-A/G-B and R1 (PB-DX32 M8).
**Verify**: `cargo test -p mtg-engine --test core pb_dx24` green; each of the four reverts watched
red with the rebuild confirmed in the captured output; all restored, `git diff` clean.

### Stage 7 — close-out gates
- `cargo build --workspace` clean.
- `cargo test --workspace --no-fail-fast` to a **file** (never `| tail` — the 2026-08-02 lesson);
  record the count; residual list must be empty.
- `cargo clippy --workspace --all-targets -- -D warnings` clean.
- `cargo fmt --check` **and** `tools/check-defs-fmt.sh` (SR-35).
- `cargo test -p mtg-engine --test core protocol_schema` / `--test core hash_schema` —
  **gate-executed**; report PROTOCOL and HASH read out of the run, not predicted.
- `cargo test -p mtg-simulator` / `cargo test -p play-server` — both unmoved.
- `python3 tools/authoring-report.py` → body byte-identical → revert the churn (§8).
- Scope diffs reported: `git diff main..HEAD --numstat -- crates/simulator/ tools/` must be
  **EMPTY**; `-- crates/card-defs/` exactly one file (comment-only);
  `-- crates/card-types/` **EMPTY** (no DSL change).
- Update `docs/audits/decision-point-audit.md` rows `OOS-DX1-3` (CLOSED, with the corrected arm
  count and the "34 sites" figure superseded in place, not deleted) and `OOS-DX1-4` (CLOSED for
  Q1/Q2/Q3/Q4/Q6/Q7; **re-scoped with reason** for Q5).
- File new seeds `OOS-DX24-*` for anything found and deliberately not fixed — at minimum the two
  §10 candidates below, if the runner's investigation confirms them.

---

## 10. Risks & edge cases

1. **The `&[&AbilityDefinition]` ergonomics.** §2.4's zero-edit claim rests on default binding
   modes peeling two references. It is standard Rust and the fallback is stated, but **verify by
   compiling before writing the other 900 lines of this batch** — it is Stage 3's first action.
2. **The batch-arrival set may be coarser than one SBA round.** `check_triggers(state, events)`
   receives whatever event slice its caller assembled. If a slice can span two genuinely
   *sequential* deaths, the CR 603.10a guard would over-suppress (a Traitor that arrived earlier in
   the same slice would be wrongly excluded). The runner must establish the slice's granularity by
   reading `check_triggers`' callers, record the finding, and — if the slice is coarser than one
   simultaneous-event batch — **file it as a seed and state the deviation direction** rather than
   guessing a refinement. Over-suppressing is the safer failure (it matches the ruling in the
   common case) but it must be *known*, not assumed.
3. **CR 603.2d doubling of a graveyard-sourced trigger** (§3.7 step 2). Almost certainly wrong if
   unscoped; investigate, file, do not fix here.
4. **`nether_traitor` losing its runtime trigger entry could shift a runtime index** for another
   ability on the same card. It cannot — the Traitor's other two abilities are
   `AbilityDefinition::Keyword`, which the trigger lowering never touches — but T7's differential
   proves it corpus-wide rather than relying on that reading.
5. **Golden scripts / SR-9b fingerprints.** A behaviour change on a `Complete` card can move a
   golden script. Nether Traitor appears in no script the planner found, but the full-workspace run
   in Stage 3 is the arbiter. If a script moves, **report the diff before reconciling it**, and
   reconcile by *strengthening* (the PB-DX3b precedent), never by weakening an assertion.
6. **Q2's "zero behaviour change" claim** rests on §4.0's invariant. If the runner finds any path
   that sets `is_transformed` on a stack object, §4.0 is false and Q2 becomes a live repair — which
   is *good news*, but it changes the probe design. Pin the invariant with an assertion in the Q2
   probe so a future path that breaks it is caught.
7. **A separate, self-consistent face-blindness at Q2** is worth filing and **not** fixing here: a
   card cast with **disturb** (CR 702.146a / 712.11a) is cast as its *back* face, yet both ends of
   the `WhenYouCastThisSpell` dispatch read the front face (`is_transformed` is only set at ETB,
   `resolution.rs:852-853`, i.e. *after* the cast). A back-face `WhenYouCastThisSpell` would be
   invisible at both ends — face-blind, not disagreeing, so out of `OOS-DX1-4`'s scope. **File as
   `OOS-DX24-n` with the measured back-face count from §4.2.**
8. **`squee_goblin_nabob` remains broken and must be said so.** Its `AtBeginningOfYourUpkeep` +
   `trigger_zone: Graveyard` pair has **neither** a lowering arm **nor** a dispatch arm; after this
   batch it still never fires. It is `known_wrong` and deck-illegal, so this is not a regression —
   but the execution notes must state it, so nobody reads "PB-DX24 closed the `trigger_zone` gap"
   as "every `trigger_zone` def now works." Candidate seed.
9. **The `MayPayThenEffect` auto-pay question.** T3 asserts an end-to-end return. If the engine
   auto-pays `{B}` without a player decision, T3 still discriminates (the paired no-mana case), but
   the runner should record whether a real choice is offered — a costless-or-auto-paid "may" is the
   **DP-12 / `OOS-DP10-9`** class and, if it applies here, `nether_traitor`'s `Complete` marker
   deserves scrutiny in a *later* batch. **Do not demote it in this one**; file the observation.
10. **Two probes measuring the same thing.** T1 and T7 both die if the filter is removed. That is
    deliberate — T1 is behavioural (does the wrong card trigger?) and T7 is structural (does *any*
    def lower a zone-scoped ability?). Keep both; note the overlap so a future reader does not
    delete one as redundant.
