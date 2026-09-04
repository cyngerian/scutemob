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
