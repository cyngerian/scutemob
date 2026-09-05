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

---

## §3 THE SECOND DECK-LEGAL SUBJECT IS LIVE-WRONG FOR A DIFFERENT REASON, AND THIS BATCH'S OWN CLAIM ABOUT IT WAS REFUTED BY EXECUTION

`crates/simulator/tests/pb_dx49_saga_blanking_channel.rs`'s module doc records, one batch old
and in the tree:

> *"**Chapter III** … Its grant is an `EffectFilter::CreaturesYouControl` continuous effect,
> and that filter resolves its controller through `state.objects.get(&source_id)` at
> layer-application time … CR 714.4 sacrifices the Saga in the same window it resolves in,
> **the source id is gone, and the filter matches nothing.** A draft of this file used it and
> failed on exactly that."*

`binding_the_old_gods` is `Complete` **by derive** (it declares no `Completeness` line at all)
and therefore **deck-legal**, and it is a member of this batch's census axis (iv). The
coordinator read that note, connected it to this seed, and published — in a task comment —
that *"the deck-legal live-wrong count is at least TWO"*. **That was published before it was
executed, and execution refuted it.**

Measured on a real `LocalGame` human-seat drive, engine untouched, no `objects_mut()` poke:

1. CR 714.3b puts the crossing lore counter on (`CounterAdded { counter: Lore, count: 3 }`).
2. CR 117.5 runs SBAs **before** putting triggers on the stack, so `rules/sba.rs`'s
   *"don't sacrifice while a chapter ability is on the stack"* guard does not see a trigger
   still sitting in `pending_triggers`. The Saga **is** sacrificed and its `ObjectId` retired.
   **The CR 608.2h condition is genuinely reached.**
3. With the chapter on the stack and the Saga gone, `lki_objects` reads
   `Some((PlayerId(1), "Binding the Old Gods"))` — **PB-DX39's capture half works on this card.**
4. The chapter resolves (`AbilityTriggered` + `AbilityResolved` both in the journal) and
   **`state.continuous_effects()` is EMPTY.** No `ApplyContinuousEffect` ever ran, so
   `snapshot_affected_set` was never called and `EffectFilter::CreaturesYouControl` was
   **never consulted.**

**The symptom reproduces; the stated cause does not.** The real blocker is one link upstream:
`rules/resolution.rs`'s card-registry fallback for a `PendingTriggerKind::Normal` trigger opens
with `let obj = state.fizzle_object(source_object);`, and `fizzle_object`
(`state/diagnostics.rs:373`) is a documented **live-only** `self.objects` lookup that returns
no LKI. A departed source yields `None`, the arm falls through to `(None, None)`, and **the
whole ability resolves as a complete no-op.** That is CR 113.7a-wrong (*"an ability exists on
the stack independently of its source"*) for **every** registry-fallback triggered ability
whose source has left — not only Sagas. Out of PB-DX39's scope, filed as **`OOS-DX39-3`**.

### §3.1 THE CONSEQUENCE FOR THIS BATCH'S YIELD CLAIM, STATED BEFORE ANYONE ELSE HAS TO FIND IT

`OOS-DX5-3` and `OOS-DX5-7` are closed and `umezawas_jitte` is repaired. But **"this batch
repairs all 21 deck-legal resolution-site defs" is NOT a claim the evidence supports**, and it
is withdrawn here rather than left standing in a headline. What the fix guarantees is that
**the FILTER answers correctly when it is consulted**. Whether a given axis-(iv) member
*reaches* the filter is a property of its own dispatch path, and at least one deck-legal member
— `binding_the_old_gods` — provably does not, for a reason one link upstream that this batch
does not touch. The other 15 axis-(iv) deck-legal members are **unmeasured individually**;
saying so is worth more than a number nobody executed. *A class fix repairs the arithmetic, not
every caller's route to it.*

### §3.2 The probe shipped WRONG-WAY-ROUND rather than as the thing that was asked for

The brief asked for two green probes proving the `CreaturesYouControl` half through a real
`LocalGame` on this card. That is **impossible at HEAD** and the agent said so instead of
substituting something that looked like it. What shipped:

- `dx39_c4_binding_chapter_iii_grant_is_still_unreachable_and_the_blocker_is_downstream` —
  a real `LocalGame` drive that genuinely reaches the CR 608.2h condition and pins that the
  chapter registers **no continuous effect at all**. Its own failure message instructs the next
  reader to **invert** it for the controller's creatures and **keep it negated** for the
  opponent's. (The standalone "an opponent must not gain it" probe was deliberately NOT written:
  today nobody gains the keyword, so it would pass **vacuously**. Both directions live inside
  `c4` with the asymmetry spelled out.)
- `dx39_c5_binding_chapter_iii_source_lki_is_captured_while_the_chapter_is_on_the_stack` — the
  one positive assertion that IS reachable: the Saga's LKI is in `lki_objects` with
  `controller == PlayerId(1)` at exactly the instant `source_view_at_resolution` would read it.

### §3.3 A REVERT ROW THAT IS A COVERAGE MEASUREMENT, FOR THE SECOND TIME IN THIS BATCH

Under **R1** (delete the `.or_else(|| state.lki_object_snapshot(source_id))` LKI branch from
`source_view_at_resolution`) `c1`/`c2`/`c3` go RED and **`c4`/`c5` stay GREEN** — because on that
card the LKI *read* is never reached, so a revert of the read cannot move it. Rather than record
that as an UNDISCRIMINATED row, a **second** revert was executed: **R2**, neutralising the
`is_source_of_a_pending_ability` disjunct in `capture_lki_snapshot` (the LKI *capture*), reddens
`c5` and `c1` and leaves `c2`/`c3` green — informative rather than a gap, because Mardu departs
as an *activation cost* and is captured by the other clause, which R2 cannot reach. **Two
reverts were needed to discriminate five probes, and neither alone would have done it** —
PB-DX20b's "two reverts were not enough" finding, arrived at from the other direction.

Two mechanical hazards worth carrying forward, both caught by the agent rather than by a gate:
- **`cp -p` on restore preserves the BACKUP's mtime, so `cargo` does not rebuild** and the next
  test run reports the *reverted binary's* results against *restored* source. Caught by md5,
  fixed with `touch` and a forced rebuild; the published green is post-rebuild.
- A backup of `layers.rs` taken at 20:02 was **stale within 7 minutes** because a sibling agent
  wrote the file at 20:09. Restoring from it would have silently reverted another agent's work.
  Detected by md5 before use. *In a multi-agent worktree a file backup has an expiry date.*

---

## §4 What shipped

**`crates/engine/src/rules/layers.rs`**
- `SourceView<'a>` — one borrowed view (`controller`, `attached_to`,
  `chosen_creature_type: Option<&'a SubType>`, `chosen_color`). **Borrowed, not owned**, because
  `effect_applies_to` is on the layer walk and `SubType` wraps a `String`; an owned view would
  allocate per arm, per effect, per object.
- `source_view_live` (CR 611.3a — a static ability's source is on the battlefield by
  construction, **no fallback**) and `source_view_at_resolution` (CR 608.2h / CR 113.7a —
  live-then-LKI, the ONLY LKI-consulting constructor, with exactly one caller).
- `effect_applies_to` split into a thin live-path wrapper plus
  `effect_applies_to_inner(.., source: Option<&SourceView<'_>>)`; **all 20 source-relative arms
  consume the parameter and every other conjunct is byte-preserved** — the self-exclusions, the
  zone guards, the `chars` type/subtype/colour/supertype tests, the combat-attacker tests and
  `CreaturesOpponentsControl`'s inequality all unchanged.
- `snapshot_affected_set` resolves the at-resolution view **once, outside** its candidate loop.
  Previously each of the 20 arms did its own map lookup per candidate.
- `is_effect_active`'s two reads left **live-only with an in-source reason each**.

**`crates/engine/src/state/mod.rs`** — `is_source_of_a_pending_ability` (delegating to PB-DX52's
exhaustive `stack_registry::source_of`, so a 26th `StackObjectKind` is a compile error rather
than a silent `None`), one shared `store_lki_snapshot`, `capture_source_lki_for_pending_ability`,
the widened disjunctive gate, and the SR-24 `COUPLING:` comment rewritten to state the
three-reader contract truthfully.

**`crates/engine/src/rules/abilities.rs`** — three capture calls with the ordering reason at each.

### §4.1 A DEVIATION FROM THE BRIEF, DISCLOSED: the `discard_self` capture is a MEASURED NO-OP

`capture_source_lki_for_pending_ability` is **battlefield-only**. `lki_objects` is reachable from
the `pub fn lki_objects()` accessor and is folded into `public_state_hash`, so snapshotting a card
leaving a player's **hand** (the CR 702.34 Channel case) would put hidden information into a public
store — **Architecture Invariant 7**. The `discard_self` call therefore does nothing today; it is
present so the three self-move blocks stay uniform and `r6b` can require a fourth if one appears.
Filed as **`OOS-DX39-1`** with the population stated as UNMEASURED rather than assumed zero.

### §4.2 GATES THAT FIRED ON THIS BATCH'S OWN WORK — three, all answered rather than weakened

1. **SR-25's `bare_lookup_ratchet`** caught `layers.rs` dropping 54 → 36 bare lookups. The ceiling
   was **LOWERED** with the derivation stated (20 per-arm reads replaced by 2 constructors =
   exactly 18), per PB-DX49's rule that *a stale-high ceiling is slack a regression hides in*. It
   also caught the first draft of `capture_source_lki_for_pending_ability` using a bare
   `.objects.get(..)`; it now uses `expect_object`, because `handle_activate_ability` validates the
   source before any cost is paid, so a `None` there is an engine bug.
2. **PB-DX27's `live_identifier_mentions_are_ratcheted`** fired on this batch's own card-def note.
   Answered the way its own message asks: the note was **reworded into R1's PRIMARY vocabulary**
   (`has no`, `no variant`) so the primary gate can SEE it, then given a
   `REVIEWED_CONTRAST_MENTIONS` row with a stated reason, ceiling 109 → 110. That is the same GOOD
   direction PB-DX36's two moves took — *a def joins because the batch SHIPPED the repair its note
   had been silent about*, not because a note went stale. **Rewording here makes the note MORE
   visible to the gate, which is the opposite of editing prose to dodge a needle.**
3. **`clippy --workspace --all-targets -- -D warnings` FIRED ON THE FINAL TREE**, and the cause was
   one line above the three it reported: `doc_lazy_continuation` on lines 55-57 of the probe file,
   because **line 54 opened with `+ \`LayerModification::…\``, which markdown reads as a list
   bullet**. Line 54 was reworded rather than the three symptom lines indented. It slipped through
   because the probe agent ran clippy on the simulator target only.

---

## §5 Revert matrix — EXECUTED BY THE COORDINATOR, 6 rows, 6 discriminating, 0 UNDISCRIMINATED

Run by the coordinator against the final tree rather than accepted from the four delegated
reports; all three engine files verified restored **byte-exactly** afterwards (`diff -q`, clean).

| row | what it undoes | reddens |
|---|---|---|
| R0 | *(control, unreverted)* | **nothing — all 33 green** |
| **R1** | delete `.or_else(\|\| state.lki_object_snapshot(..))` from `source_view_at_resolution` — the READ | `t1`,`t4`,`t5`,`t6`, `c1`,`c2`,`c3`, `r3`, `r5b` — **9** |
| **R2** | neutralise the `is_source_of_a_pending_ability` disjunct — the STACK capture clause | `c1`, `c5`, `t1` |
| **R3** | drop the `sacrifice_self` capture call — the ACTIVATION-COST capture clause | `c2`,`c3`, `t4`,`t5`,`t6`, `r6b` |
| **R4** | give `source_view_live` an LKI fallback — the OVER-WIDE direction | `r3`, `r5b` **only** |
| **R5** | point `snapshot_affected_set` at the live constructor | `r5`, `t1`,`t4`,`t5`,`t6`, `c1`,`c2`,`c3` — **8** |
| **R7** | move the `sacrifice_self` capture to AFTER the move | same set as R3 — so `r6b` catches ORDER, not just presence |

### §5.1 R2 AND R3 ARE PRECISE COMPLEMENTS, AND THAT IS THE PROOF BOTH CLAUSES ARE LOAD-BEARING

R2 reddens the Jitte probes and leaves the Mardu ones green; R3 does the exact opposite. Neither
revert alone can discriminate the other's subject, because the two subjects leave the battlefield
at different moments (§0.3). **A batch that built only one clause would have passed every probe it
thought to write for its own half** — which is precisely how `OOS-DX5-7`'s residual survived
PB-DX5's fix of the same mechanism.

### §5.2 R4 REDDENS ONLY SOURCE GATES, AND THIS TIME THAT IS A MEASUREMENT RATHER THAN A GAP

`OOS-DX52-2` records that *"a row that reddens only a source gate is telling you the behaviour has
no probe"*. R4 moves no behavioural probe — and here the correct reading is the other one: `r7`
measures the exposed live-path population at **0 statics**, with the 4 emblem members unreachable
because CR 114.1 keeps an emblem in the command zone and nothing ever retires its `ObjectId`. There
is no behaviour to probe, so gates are the right instrument. Filed as **`OOS-DX39-6`**, which
refines `OOS-DX52-2` into a pair: *a source-gate-only row means either "no probe" or "no reachable
behaviour", and a population measurement is what tells them apart.*

### §5.3 R5 PRODUCED NO VERDICT ON ITS FIRST RUN, AND THE FAILURE MODE IS THE DANGEROUS DIRECTION

Swapping `snapshot_affected_set` onto the live constructor leaves `source_view_at_resolution` with
no callers, so `-D warnings` turns it into `error: function ... is never used` and the crate does
not build. **A matrix that does not separate "the gate stayed silent" from "the crate did not
build" reports the wrong verdict in the SAFE-LOOKING direction** — the implementer hit the same
shape twice with plants that named nonexistent enum variants and with a detector that matched
`error: test failed` (which means the test *ran*). The harness now prints `BUILD FAILED (not a gate
verdict)` rather than a row of greens; R5 was re-run with `#[allow(dead_code)]` and is
discriminating at 8 red. Filed as **`OOS-DX39-8`**.

---

## §6 Benches — MEASURED, SIX RUNS, SAME-CODE BAND FIRST, VERDICT NO REGRESSION

`effect_applies_to` is on the layer walk, so a regression was genuinely possible and the A/B is
owed rather than optional. Matched-set A/B against merge base `604b7242`, each revision in its own
worktree with its own `CARGO_TARGET_DIR`, **all three merge-base runs taken before any HEAD run was
compiled**, on a machine with no test suite or agent running (PB-DX52's contaminated first A/B is
the reason that ordering is stated).

**Same-code repeatability band, measured FIRST across three merge-base runs: 0.46% – 3.80%**
(widest `board_wipe_4p`).

| bench | base min-max (µs) | head min-max (µs) | band | Δ mean | verdict |
|---|---|---|---|---|---|
| `board_wipe_4p` | 118.48-122.98 | 118.38-121.69 | 3.80% | +0.05% | overlap — noise |
| `full_turn_4p` | 216.11-217.36 | 214.57-217.14 | 0.58% | −0.36% | overlap — noise |
| `full_turn_6p` | 344.32-347.29 | 341.49-342.88 | 0.86% | −1.06% | non-overlapping |
| `priority_cycle_4p` | 24.22-24.34 | 23.85-24.00 | 0.46% | −1.49% | non-overlapping |
| `priority_cycle_6p` | 38.29-39.08 | 37.75-38.61 | 2.05% | −1.13% | overlap — noise |
| `sba_check` | 14.86-15.15 | 14.97-15.46 | 1.90% | +0.89% | overlap — noise |

**Verdict: NO REGRESSION.** Every interval is inside or barely outside the same-code band, and the
one bench that moved the slow way (`sba_check`, +0.89%) overlaps.

**THE APPARENT ~1-1.5% IMPROVEMENT IS DELIBERATELY NOT CLAIMED**, on the ground this queue has
used three times before (PB-DX51, PB-DX35, PB-DX52): **the two non-overlapping benches include the
control.** `priority_cycle_4p` executes no line this batch touched, and it moves −1.49% — the same
order as `full_turn_6p`'s −1.06%. A uniform shift across a bench that cannot be affected is a
build/layout artefact of two separate compilations, not an effect.

**The mechanism is also bounded rather than argued.** There IS a real saving in the change —
`snapshot_affected_set` previously let each of its candidates' matching arm do its own
`state.objects.get(&source_id)`, and now resolves the view **once** for the whole scan — but that
path is a mass-filter RESOLUTION, and none of the six benches resolves one. On the live path the
lookup count is essentially unchanged (one arm runs per call, and the wrapper does the lookup the
arm used to do). So there is no mechanism on any benched path that could produce the observed
shift, which is the tell.
