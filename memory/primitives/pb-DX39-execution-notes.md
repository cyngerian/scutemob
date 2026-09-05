# PB-DX39 — execution notes (scutemob-230)

v4 queue rank 15. Seeds: **OOS-DX5-3** (headline) + **OOS-DX5-7**'s named residual.
Merge base **604b7242** (PB-DX52 merged).

---

## §0 Stage 0 — measured before any production line

### §0.1 Pre-edit baseline — REPRODUCES PB-DX52's close pin EXACTLY

`cargo test --workspace --no-fail-fast` on this branch **before any edit**:

```
targets(result lines): 65   passed 5156   failed 0   ignored 5
```

**5,156 / 0 / 5 on 65 result-producing targets** — PB-DX52's published pin, reproduced with
no correction owed. (`OOS-DX51-5`'s non-reproducing-pin failure did not recur; this is the
second consecutive batch in which an inherited pin reproduces.)

### §0.2 The seed's preferred option does NOT work as written, and the reason is SR-24

The task brief prefers option (a): *"resolving source-relative filters against the LKI
snapshot the engine already keeps (`GameState.lki_objects`, SR-13) … so nothing new is
stored (wire NONE)"*, with the parenthetical *"check whether `SacrificeSelf` as a COST
captures LKI of the source the way `sacrificed_creature_lki` does for creatures, and
whether `attached_to` survives into that snapshot"*.

Measured at HEAD, the answers are **no, no, and yes**:

1. **`lki_objects` is EMPTY for both subjects.** `GameState::capture_lki_snapshot`
   (`state/mod.rs`) *stores* a snapshot only when the departing permanent's
   layer-resolved characteristics carry one of four keywords —
   `const LKI_RELEVANT_KEYWORDS: [KeywordAbility; 4] = [Wither, Infect, Deathtouch, Lifelink]`
   — an SR-24 optimisation whose own in-source comment states the coupling:
   *"Adding a NEW READER that consults a fifth snapshot keyword is NOT machine-caught."*
   Umezawa's Jitte (Legendary Artifact — Equipment) and Mardu Ascendancy (Enchantment)
   carry none of the four. **Option (a) is not "read the LKI"; it is "read the LKI AND
   make it carry the source."**
2. **`sacrificed_creature_lki` does not cover it.** That field is populated in the
   *sacrifice-ANOTHER-permanent* block and carries `SacrificedCreatureLki`
   (power/toughness/mana value) for *creatures*. It carries no `attached_to`, no
   `controller`, and `SacrificeSelf` writes to it not at all.
3. **The payload is already right when a snapshot IS taken.**
   `move_object_to_zone` clones `old_object` *before* the CR 400.7 field resets, and
   `capture_lki_snapshot` stores `old_object.clone()` with layer-resolved characteristics.
   So `attached_to`, `controller`, `chosen_creature_type` and `chosen_color` all survive
   into the snapshot. **Only the gate is wrong, not the payload.**

### §0.3 The two moments are DIFFERENT, and a stack-membership test covers only one

| subject | how the source leaves | is its ability on the stack at capture time? |
|---|---|---|
| Umezawa's Jitte (`OOS-DX5-3`) | destroyed **in response** to the activated ability | **YES** |
| Mardu Ascendancy (`OOS-DX5-7`) | `Cost::SacrificeSelf`, paid **during** activation | **NO** |

`rules/abilities.rs`'s sacrifice-self block carries the comment *"Move source to graveyard
**before pushing to stack**"* — and it means it: the `move_object_to_zone` call runs while
`state.stack_objects` does not yet contain the ability. So a gate widened only to *"is this
object the source of something on the stack"* covers the Jitte and **misses Mardu entirely**,
i.e. it closes the headline seed and leaves the residual open. Both moments must be handled
or half the batch is dead.

### §0.4 WIRE PREDICTION, PER OPTION — written before any production line

Both options were costed. **Neither prediction is inherited from the v4 memo's LOW-MED
`"HASH if a snapshot must be stored; none if derivable at resolution"` cell**; both are
derived from the gates' own closure rules at HEAD (PROTOCOL 43 / HASH 84, closure type
counts 98 / 132).

**Option (a) — resolve through `lki_objects`, widening only WHEN a snapshot is stored.**
- `HASH_SCHEMA_VERSION` **UNMOVED**. The change adds no type, no variant and no field to
  any hashed declaration; `lki_objects: OrdMap<ObjectId, GameObject>` and its `HashInto`
  arm already exist (`state/hash.rs`, `public_state_hash`). This is SR-24's own precedent
  read in reverse — that batch *narrowed* the same gate and its doc records the measured
  conclusion: *"touches no `HashInto` impl and no serde shape, so neither SR-17 fingerprint
  moves."* Narrowing and widening are the same edit with the sign flipped.
- The **stream** fingerprint is likewise **UNMOVED**: it digests `canonical_fixture()`, a
  static state in which no object ever departs a zone, so no `capture_lki_snapshot` call
  can run and no entry can reach `lki_objects`.
- `PROTOCOL_SCHEMA_FINGERPRINT` **UNMOVED**: nothing is added to the
  `Command` / `GameEvent` / `Effect` / `Characteristics` closure.
- **Predicted: ZERO bumps, both closure type counts unchanged at 98 / 132.**
- The one thing that DOES move is the *runtime* `public_state_hash` of a game in which a
  keyword-less permanent departs while its ability is pending — that is a value, not a
  declaration, so it moves no version, and it is a fuzz/loop-detection perturbation to be
  measured rather than a wire change.

**Option (b) — store a source snapshot (`controller` + `attached_to`) on the `StackObject`
at activation.**
- `HASH_SCHEMA_VERSION` **+1**: `StackObject` is hashed field-by-field
  (`state/hash.rs`), so a new field is a declaration change.
- `PROTOCOL_SCHEMA_FINGERPRINT` **unmoved**: `StackObject` is engine state and is not
  reachable from the four closure roots.
- Plus the full sentinel re-pin (48+ HASH sites), a history row, and a frozen-prefix
  re-pin.

### §0.5 THE CHOICE IS (a), AND THE DECIDING ARGUMENT IS CR-CORRECTNESS, NOT WIRE COST

Option (b) is not merely more expensive — **it is CR-wrong**, and the Jitte's own ruling
says so in one sentence. Verified verbatim via MCP (Umezawa's Jitte, ruling 2005-02-01):

> *"If the Jitte leaves the battlefield after the '+2/+2' mode is announced but before it
> resolves, the bonus is given to the creature that was **most recently equipped** once the
> ability resolves."*

A snapshot captured **at activation** answers *"the creature equipped when the ability was
activated"*. Re-equip the Jitte in response (Equip is sorcery-speed, so the realistic route
is a second player's effect moving it, or any `AttachEquipment` at instant speed) and then
destroy it, and (b) names the **old** creature while the ruling names the **new** one.
Option (a) reads the LKI of the moment the source last existed — which is the definition of
*"most recently equipped"*. **LKI is not a cheaper approximation of the snapshot; it is the
mechanism CR 608.2h names**, and CR 113.7a states the same rule for abilities:
*"if the source is no longer in the zone it's expected to be in at that time, its last known
information is used."*

### §0.6 THE MOMENT THE SNAPSHOT REPRESENTS — stated, and pinned

Two different moments, and conflating them is the whole defect:

- **WHICH objects are affected** is determined at **RESOLUTION** — CR 611.2c: *"the set of
  objects it affects is determined when that continuous effect begins. After that point,
  the set won't change."* That is `snapshot_affected_set`, called from
  `Effect::ApplyContinuousEffect` **during** resolution. **This batch does not move it**;
  PB-DX5's T12 is the control that proves it, and it must stay green and byte-identical.
- **WHAT the source's controller / attachment WAS** is answered by CR 608.2h: the current
  information if the source is still in its expected zone, otherwise **its last known
  information** — i.e. the values as of the instant it last existed on the battlefield.

So: *the set is determined at resolution, from the source as it most recently existed.*
Pinned by a probe that makes the two moments disagree (the source departs between
activation and resolution, and the board changes in between).

### §0.7 A CORRECTION TO THE TASK BRIEF, RECORDED RATHER THAN INHERITED

The brief says the current behaviour is *"legal Magic in no case the ruling describes (the
bonus is never simply lost)"*. **That is false as written**, and the same ruling block says
so:

> *"Choosing the 'Equipped creature gets +2/+2 until end of turn' mode does nothing if the
> Jitte isn't equipped to a creature when the ability resolves."*

The bonus **is** simply lost — legally — when the Jitte is on the battlefield and unequipped
at resolution. What is illegal is losing it when the Jitte has **left the battlefield**.
The distinction is load-bearing for this batch: a fix that made `AttachedCreature` fall back
to *anything* whenever the set came out empty would break the legal case while fixing the
illegal one. Both directions are pinned.

### §0.8 THE SITE LIST WAS A FLOOR, AND THE THREE MISSING SITES ARE THE MULTI-LINE SPELLING

The task brief and the coordinator's launch comment both say
*"`state.objects.get(&source_id)` occurs **17** times in `layers.rs`"*, with the brief adding
*"— a FLOOR, enumerate every one yourself"*. Enumerated by parsing the function body
(lines 767..1268) and splitting it into arms:

- `effect_applies_to` has **37** `EffectFilter` match arms;
- **20** of them are source-relative (mention `effect.source`);
- there are **20** source reads, not 17.

The three the same-line count misses are `AttachedCreature`, `AttachedLand` and
`AttachedPermanent`, which spell the read across a line break:

```rust
state
    .objects
    .get(&source_id)
```

That is `OOS-DX50`'s and `OOS-DX20b`'s lesson — *a census is only as wide as the spelling
its regex matched* — recurring **inside the census of the batch dispatched to fix a
source-read defect**, and it is not a harmless undercount: `AttachedCreature` is the
headline seed's own arm, so a fix that swept "the 17" by matching the same-line spelling
would have missed `OOS-DX5-3` entirely while reporting a complete sweep.

Two further reads live in `is_effect_active` (`WhileSourceOnBattlefield`'s battlefield
test and `check_static_condition`'s controller). They are **deliberately excluded and
must stay live-only**: routing the first through LKI would make a departed source's static
ability run forever, which is the exact opposite of CR 611.2b. Pinned by a gate so a later
batch cannot "finish the job".

---

## §1 The census (AC 7361) — the class is 15× the two seeds, and the fourth axis is nobody's

Printed by `core::pb_dx39_source_relative_roster::t_dx39_census_report` (SR-36: enumerated
from `all_cards()`, never grepped; every figure below is the test's own output, not a
transcription).

```
scanned denominator (all_cards()):            1803
source-relative filter variants (derived):    20  (of 37 declared)
total source-relative occurrences:            229
distinct defs carrying one:                   147
of those, deck-legal `Complete`:              108
by ContinuousEffectDef site:  { static: 184, resolution: 41, emblem: 4 }
```

| axis | occ | defs | deck-legal | verdict |
|---|---|---|---|---|
| (i) `SacrificeSelf`-cost | 1 | 1 (`mardu_ascendancy`) | **0** | live-wrong, ALWAYS, 0 exposure |
| (ii) attached, **resolution-time** | 1 | 1 (`umezawas_jitte`) | **1** | live-wrong on a race |
| (ii-static) attached, static (CR 611.3a) | 97 | 44 | 30 | **out of scope, control** |
| (iii) instant/sorcery mass (PB-DX5's fixed class) | 11 | 7 | 4 | **CONTROL — must stay green** |
| **(iv) the residual nobody named** | **28** | **20** | **16** | **live-wrong on a race** |

### §1.1 THE THREE NAMED AXES DO NOT COVER THE CLASS, AND THE FOURTH IS THE YIELD

The task brief partitions the population three ways — sacrifice-self, attached, and the
PB-DX5-fixed instant/sorcery control class. The census refuses that partition: the whole
**resolution-generated** class is **41 occurrences / 29 defs / 21 deck-legal**, and the three
named axes account for 13 occurrences of it. The other **28 occurrences across 20 defs, 16 of
them deck-legal `Complete`**, are `Activated` / `Triggered` / `LoyaltyAbility` / `SagaChapter`
abilities with a cost that does **not** move the source and a filter that is not attachment-based:

> `Battle Cry Goblin, Binding the Old Gods, Castle Embereth, Crashing Drawbridge,
> Craterhoof Behemoth, Elspeth Storm Slayer, Elvish Warmaster, Ezuri, Felidar Retreat,
> Goblin Bushwhacker, Goldnight Commander, Goro-Goro, Kolaghan, Lathliss, Massacre Wurm,
> Mirror Entity, Purphoros, Sarkhan Vol, Vault of the Archangel, Vito`

They are neither certainties (axis (i), where the source is gone by construction) nor
already-fixed (axis (iii)). They are **races**: activate, the opponent responds by killing the
source, and the pump silently applies to nobody. `Craterhoof Behemoth` and `Mirror Entity` are
the recognisable ones — kill the Hoof in response to its own trigger and today the whole team
loses the pump. **This is why the batch's yield is not "two cards"**: one shared arithmetic on
the locked path repairs all 21 deck-legal resolution-site defs at once.

### §1.2 THE DECK-LEGAL SPLIT OF THE TWO SEEDS IS THE OPPOSITE OF THE MEMO'S CELL

The v4 memo row 15 says *"`umezawas_jitte` (deck-legal `Complete`, live-wrong) +
`mardu_ascendancy` (`partial`)"* — correct, and its consequence is not drawn anywhere:
**axis (i)'s deck-legal blast radius is ZERO**. `mardu_ascendancy` is `partial` on its nontoken
attack filter, so `validate_deck` rejects it and no player has ever been able to lose that
+0/+3 in a real game. `OOS-DX5-7`'s residual is a genuine defect with **no current player-facing
exposure**, and saying so is more useful than implying otherwise. The exposure this batch
actually removes is axis (ii)'s **1** and axis (iv)'s **16**.

### §1.3 THE INVERSE ORACLE AXIS AND THE STRUCTURAL AXIS DO NOT NEST, IN EITHER DIRECTION

16 defs print an "until end of turn" pump on a sacrifice- or equip-cost ability. Only **3** are
in the structural population (`etchings_of_the_chosen`, `mardu_ascendancy`, `umezawas_jitte`);
**13 are oracle-only** and **144 structural defs are not on the oracle axis**. Both differences
are printed rather than reduced to an overlap count — PB-DX26's, PB-DX43's and PB-DX15a's
shared lesson, which is that *a roster derived from one declaration construct measures that
construct*, and the fix for a short census is a second axis rather than a better matcher.

### §1.4 THE NESTING WALK IS GENERIC BY CONSTRUCTION, NOT A HAND-LISTED MATCH

Each `AbilityDefinition` is serialised and the tree walked for any object carrying all of
`layer`/`modification`/`filter`/`duration`. That is total by construction, which is the only
answer to PB-DX26's `Effect::RollDice` finding (an eleventh nesting site invisible to a
`Box`/`Vec` count). The source-level form count is recorded as **evidence** (8 `Box<Effect>` +
2 `Vec<Effect>` + 1 `Vec<(u32,u32,Effect)>` = 11, reproducing PB-DX26's corrected figure) and
the roster says in its own doc that this is not the mechanism.

**One revert row is a coverage measurement rather than a pass, and it is disclosed in the test
itself**: deleting the walk's array recursion reddens R4/R5/R6/R9 and leaves **R3 GREEN**,
because `mardu_ascendancy`'s path carries no array segment. A shallow walk satisfies axis (i)
completely — so R3's green is not evidence that the census reached anything nested, and the
file says so where a reader will find it.

---

## §2 Fail-before evidence, and the two limits it exposed

### §2.1 The RED output, captured at the unfixed merge base

Four probes RED, every failure on the **verdict** assertion rather than on a floor (each
probe's non-vacuity floors — the source really absent from `state.objects`, the ability
really on the stack, the Jitte really attached — passed in every case):

```
dx39_t1_jitte_bonus_survives_the_jitte_being_destroyed_in_response
  CR 608.2h + Jitte ruling 2005-02-01: the bonus goes to the creature that was most
  recently equipped, even though the Jitte itself is gone
    left: Some(1)   right: Some(3)

dx39_t4_mardu_sacrifice_self_pumps_every_creature_you_control
    left: Some(2)   right: Some(5)

dx39_t5_mardu_does_not_pump_an_opponents_creature
    left: Some(2)   right: Some(5)

dx39_t6_mardu_set_is_determined_at_resolution_not_activation
    left: Some(2)   right: Some(5)

test result: FAILED. 2 passed; 4 failed
```

The two that passed are the **controls**, and they are the direction a careless widening
breaks: `dx39_t2` (the ruling's legal-empty half — an unequipped Jitte on the battlefield
pumps nobody) and `dx39_t3` (the live attachment wins while the source is alive). Both green
before and after.

### §2.2 THE BRIEF'S OWN `t6` SPECIFICATION IS CR-WRONG, AND THE PROBE WAS WRITTEN TO THE CR INSTEAD

The coordinator's brief specified *"a creature that enters the battlefield BETWEEN activation
and resolution is NOT a member"*. That is the **activation-time** rule and there is no such
rule: CR 611.2c determines the set *"when that continuous effect begins"*, which is at
**resolution**, so a creature that entered in the meantime **IS** a member. The probe was
written to CR 611.2c and the shipped engine agrees. Recorded in the test's own docstring.
*A brief is a claim like any other* — this is the second one this batch has had to refute,
after the site-count floor.

### §2.3 `OOS-DX5-7`'s SUBJECT CANNOT BE DRIVEN THROUGH `LocalGame` AT ALL, AND THAT IS STATED RATHER THAN SUBSTITUTED

Acceptance criterion 7359 asks for both subjects *"on a real `LocalGame`/`HumanChoice`
drive"*. For `mardu_ascendancy` that is **impossible at HEAD**, and the refusal is
Architecture Invariant 9 doing its job:

```
PB-DX39 channel game must start: Engine(IncompleteCardsInGame {
  count: 1, first_name: "Mardu Ascendancy", first_kind: "partial", .. })
```

`validate_deck` rejects non-`Complete` cards, `mardu_ascendancy` is `partial`, and this batch
deliberately does **not** promote it (its nontoken attack filter is still missing). So **no
validated game can contain the card**, which is the same fact §1.2 states from the census
side: the residual's deck-legal blast radius is zero.

What was written instead is named rather than passed off as the thing asked for: the Mardu
channel probes drive `StubProvider::legal_actions` → `action_to_command_with_params` →
`process_command`, which is the **same production mapping `LocalGame::submit` calls**, and the
test file's own module doc states exactly what that omits. **And the criterion's substance is
then met on a different card** — see §3.

### §2.4 THE CHANNEL PROBES CARRY NO FAIL-BEFORE EVIDENCE OF THEIR OWN, AND SAY SO

The engine repair landed while the channel file was being written, so all three channel probes
were first executed against the **fixed** tree. They are channel-**reachability** pins, not
discriminating evidence, and the file says so in its own doc rather than leaving a reader to
assume otherwise. (`c1` does carry a `require_stack: 1` response-ordering floor proving it
genuinely reaches the destroyed-in-response condition.) The discriminating evidence for the
engine change is §2.1's primitive RED plus the executed revert matrix in §6.
