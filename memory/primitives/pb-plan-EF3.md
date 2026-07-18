# Primitive Batch Plan: PB-EF3 — attack-trigger target fidelity + defending-player target

**Generated**: 2026-07-18
**Primitive**: Two correctness-first changes to the "whenever a creature you control
attacks" trigger path.
  - **(A) EF-W-MISS-10 (HIGH, bug)**: forward the DSL `targets` into the runtime attack
    trigger in `enrich_spec_from_def` (currently hardcoded `targets: vec![]`), and fix the
    auto-target registry fallback that raw-indexes `def.abilities` with a *runtime* index.
  - **(B) EF-W-MISS-4 (MED, capability)**: add `EffectTarget::AttackTarget` (the player or
    planeswalker the triggering attacker is attacking) and `PlayerTarget::DefendingPlayer`
    (the defending player, CR 508.4), with the defending player **captured at attack-trigger
    dispatch** and threaded to `EffectContext`.
**CR Rules**: 508.1 (declare attackers), 508.4 (defending player / attack target), 506.4c
(attacked planeswalker removed → attacker attacks nothing), 603.2 (triggered abilities),
601.2c (targets), 113.7a (LKI capture).
**Cards affected**: 3 ship (1 flip + 2 new) + 5 documented-blocked/deferred + 1 mis-listed.
**Dependencies**: none (all prerequisite primitives — `WheneverCreatureYouControlAttacks`,
`TapPermanent`, `PreventNextUntap`, `DealDamage`, `TargetPermanentWithFilter`,
`triggering_creature_id` plumbing, `defending_player_id` field — already exist).
**Deferred items from prior PBs**: this PB IS the scheduled owner of EF-W-MISS-10 and
EF-W-MISS-4 (deferred from W-MISS, `scutemob-97`). No other carry-forward.

**TODO sweep (roster-recall gate, MANDATORY)**: grepped
`crates/card-defs/src/defs/` for TODOs naming this primitive:
- `hellrider.rs:27` — TODO naming "the player or planeswalker IT'S attacking … No PlayerTarget
  or EffectTarget variant resolves to the specific attack target … AttackTargetOf(...)".
  **Forced add** (already in brief; confirms `EffectTarget::AttackTarget`).
- `thousand_faced_shadow.rs:6` — different primitive (intervening-if on ETB "if it's
  attacking"); NOT in scope.
- All other `defending player` / `player or planeswalker` matches (warchief_giant,
  ulamogs_crusher, kazuul_tyrant, goblin_king, elvish_champion, etc.) reference the phrase for
  unrelated mechanics (menace, annihilator, evasion) — verified none use
  `WheneverCreatureYouControlAttacks` with a defending-player effect this PB unblocks.
- **Result: 1 card (hellrider) self-identifies; already in the roster.** No hidden additions.

---

## Primitive Specification

### Part A — MISS-10: attack trigger drops its declared target (HIGH, pure bug)

`enrich_spec_from_def` (the single production path that lowers a card def's
`AbilityDefinition::Triggered` into a runtime `TriggeredAbilityDef`; re-exported via
`lib.rs`, consumed by `GameStateBuilder::build`, so this covers real games, not just
scripts — confirmed no other builder constructs the attack `TriggeredAbilityDef`) contains
~30 `with_triggered_ability` blocks. **Every one hardcodes `targets: vec![]`**, discarding
the DSL ability's `targets: Vec<TargetRequirement>` (card_definition.rs:320-323).

Today a targeted attack trigger only ever gets its targets by *accident*: the auto-target
picker's registry fallback (`abilities.rs:6709-6723`) does
`def.abilities.get(trigger.ability_index)`, but `trigger.ability_index` for a `Normal`-kind
trigger is an index into the **runtime** `characteristics.triggered_abilities` vec
(`abilities.rs:6339`, `idx` from `resolved_chars.triggered_abilities.iter().enumerate()`),
**not** into `def.abilities`. For `Ojutai, Soul of Winter` the def order is
`[Keyword(Flying), Keyword(Vigilance), Triggered{attack}]` while the runtime order is
`[Triggered{attack}]` (keywords aren't triggered abilities). So `ability_index == 0`, and
`def.abilities.get(0)` returns `Flying` — not a `Triggered` — and the target is silently
dropped. The trigger goes on the stack with no target; `TapPermanent`/`PreventNextUntap`
resolve against an empty list (wrong game state, not merely omitted text).

**Fix**: make the runtime `triggered_abilities` authoritative for targets (forward `targets`
in the enrich blocks), and restrict the raw-index `def.abilities` registry fallback to
`CardDefETB` kind, where `ability_index` genuinely indexes `def.abilities`
(`abilities.rs:6438`, `def.abilities.iter().enumerate()`).

### Part B — MISS-4: defending-player / attack-target primitives (MED, capability)

- **`EffectTarget::AttackTarget`** — resolves to the player *or planeswalker* the triggering
  attacker is attacking. Used by `DealDamage` (Hellrider, Raid Bombardment: "deals 1 damage
  to the player or planeswalker it's attacking"). Must resolve to `ResolvedTarget::Player` for
  `AttackTarget::Player`, `ResolvedTarget::Object(pw)` for `AttackTarget::Planeswalker`.
- **`PlayerTarget::DefendingPlayer`** — resolves to the *defending player* only (CR 508.4:
  attacked player, or the controller of the attacked planeswalker). Companion to the finding's
  title ("no *defending player* target"); reads the same captured `ctx.defending_player` that
  `AttackTarget`'s player case uses. Ships 0 *new* Complete cards this PB (its would-be users —
  Brutal Hordechief, Silumgar — are blocked on *other* primitives, see below), but it is a
  ~5-line rider on plumbing `AttackTarget` requires anyway and is pinned by a decoy test.

**Design decision — capture-at-dispatch vs. lazy (justified)**:

The defending player is **captured at attack-trigger dispatch** into the *existing*
`PendingTrigger.defending_player_id` field, then threaded (mirroring `damaged_player`) to a
new `StackObject.defending_player` → `EffectContext.defending_player`. Rationale:
- CR 508.4 fixes the defending player at declaration; CR 113.7a says a triggered ability uses
  last-known information. A player-scoped effect (Brutal Hordechief's life loss, and
  `AttackTarget`'s Player case) must reflect the defender determined when attackers were
  declared, even if the attacker leaves combat (CR 506.4) before the trigger resolves.
- This mirrors the codebase's established pattern exactly: `AttackersDeclared` already
  captures `defending_player_id` per-attacker for `SelfAttacks`/annihilator triggers
  (`abilities.rs:3555-3573`), and `damaged_player`/`triggering_creature_id` already flow
  PendingTrigger → StackObject → EffectContext.
- `defending_player_id` is already a *sanctioned generic* field on `PendingTrigger`
  (`pending_trigger_shape.rs` EXPECTED_FIELDS line 64) — **no new PendingTrigger field, no
  shape-test churn.**

For `EffectTarget::AttackTarget` the **planeswalker (object) case is resolved lazily** from
`state.combat.attackers[ctx.triggering_creature_id]` at execution, because a planeswalker
recipient cannot be represented by a captured `PlayerId`. This is safe: an attack trigger is
put on the stack and resolves within the declare-attackers step, so `GameState::combat` is
present (it is cleared only at `EndOfCombat`). If the attacker has left the `attackers` map,
fall back to the captured `ctx.defending_player` as `ResolvedTarget::Player` (CR 113.7a); if
neither is available (e.g. the attacked planeswalker was removed — CR 506.4c: the attacker is
now attacking nothing), resolve to empty and the damage fizzles, which is correct.

**Why substituting `EachOpponent`/`Controller` is wrong** (do not shortcut): in a 4-player
game an attacker attacks exactly one defending player; `EachOpponent` would hit all three
and `Controller` is nonsensical. Each per-attacker trigger instance carries its own
`triggering_creature_id` and its own captured `defending_player_id`, so multiple attackers
attacking different players each resolve to the correct single defender.

---

## CR Rule Text (key excerpts)

- **508.4**: "…its controller chooses which defending player, planeswalker a defending player
  controls, or battle a defending player protects it's attacking…" — the attack target is a
  player OR a planeswalker (controlled by a defending player). Defending player of a
  planeswalker = the planeswalker's controller.
- **506.4c**: "If a creature is attacking a planeswalker … removing that planeswalker …
  doesn't remove that creature from combat. It continues to be an attacking creature, although
  it is not attacking any player, planeswalker, or battle." — justifies the empty/fizzle
  branch when the attacked object is gone.
- **113.7a** (LKI): a triggered ability that has triggered uses the game's last-known
  information for objects/players it references — justifies capture-at-dispatch.

---

## Engine Changes

### Change A1 — forward `targets` in the attack enrich block (and the sibling blocks)

**File**: `crates/engine/src/testing/replay_harness.rs`
**Action**: In the `WheneverCreatureYouControlAttacks` enrich block (2992-3014), add `targets`
to the `AbilityDefinition::Triggered { … }` destructure and set `targets: targets.clone()`
in the `TriggeredAbilityDef { … }` literal (currently `targets: vec![]` at line 3012).
**Then** apply the same one-line change to **every** enrich `with_triggered_ability` block
that destructures `AbilityDefinition::Triggered { … }` and currently writes `targets: vec![]`
(the ~28 blocks at the `if let AbilityDefinition::Triggered {` sites listed by
`grep -n 'AbilityDefinition::Triggered\|targets: vec!\[\]' replay_harness.rs`, e.g. death
2947-2982, combat-damage 3022-, and the others through 3396). This makes the runtime
`triggered_abilities` vec authoritative for targets across *all* card-def triggers, which is
the precondition for the A2 fallback fix to be non-regressing.
**Pattern**: the death block at 2947-2982 is the closest sibling.
**CR**: 601.2c — a triggered ability's targets are declared from its `TargetRequirement`s.

> Note: blocks that build triggers from keyword-derived / synthesized conditions still write
> `targets: vec![]` legitimately — only blocks that pattern-match `AbilityDefinition::Triggered`
> out of `def.abilities` carry an authorable `targets` to forward.

### Change A2 — fix the auto-target registry fallback

**File**: `crates/engine/src/rules/abilities.rs` (6686-6727)
**Action**: The `from_runtime` branch (6689-6708) currently returns `None` when the runtime
ability's `targets` is empty, falling through to `def.abilities.get(trigger.ability_index)`
(6709-6723) — which is a **runtime index into a def vec** and is wrong for `Normal` kind.
Change so that:
- For `PendingTriggerKind::Normal`: the runtime
  `obj.characteristics.triggered_abilities.get(trigger.ability_index).targets` is
  authoritative (return it even when empty; do **not** fall through to the def raw-index).
- For `PendingTriggerKind::CardDefETB`: keep the `def.abilities.get(trigger.ability_index)`
  registry lookup (here `ability_index` correctly indexes `def.abilities`, per
  `abilities.rs:6438/6504`).
**CR**: 603.3d (auto-select legal targets for card-def triggered abilities).
**Why safe**: after A1, every `Normal` card-def trigger's targets live in the runtime vec, so
removing the def raw-index fallthrough for `Normal` cannot drop a target that was previously
found by accident. Proven by the full suite staying green (it exercises the existing targeted
`Normal` triggers — dies/ETB/etc.).

### Change B1 — capture the defending player at attack-trigger dispatch

**File**: `crates/engine/src/rules/abilities.rs` (3873-3889, the
`AnyCreatureYouControlAttacks` loop inside the `AttackersDeclared` handler)
**Action**: change `for (attacker_id, _) in attackers` to `for (attacker_id, attack_target)`;
capture `let pre_len = triggers.len();` before `collect_triggers_for_event`; compute the
defending player with the same match used at 3565-3570:
```
let defending_player = match attack_target {
    AttackTarget::Player(pid) => Some(*pid),
    AttackTarget::Planeswalker(pw_id) => state.objects.get(pw_id).map(|o| o.controller),
};
for t in &mut triggers[pre_len..] { t.defending_player_id = defending_player; }
```
**Pattern**: identical to the `SelfAttacks` capture at 3555-3573.
**CR**: 508.4 / 113.7a.

### Change B2 — thread `defending_player_id` → StackObject → EffectContext

**File**: `crates/card-types/src/state/stack.rs`
**Action**: add `pub defending_player: Option<PlayerId>` to `StackObject` (beside
`damaged_player` at line 446); default `None` in the constructor default block (551-553) and
any full-literal `StackObject` constructor (`trigger_default`).

**File**: `crates/engine/src/rules/abilities.rs` (~7801, flush_pending_triggers)
**Action**: after `stack_obj.triggering_creature_id = trigger.entering_object_id;` add
`stack_obj.defending_player = trigger.defending_player_id;` (covers both the Normal and
CardDefETB `TriggeredAbility` branches, which share this tail).

**File**: `crates/engine/src/rules/resolution.rs` (2107-2109 kicker path, 2197-2199
non-kicker path)
**Action**: after `ctx.triggering_creature_id = stack_obj.triggering_creature_id;` add
`ctx.defending_player = stack_obj.defending_player;` in **both** blocks.

**File**: `crates/engine/src/effects/mod.rs` (EffectContext struct 48-160, `::new` 164-,
`::new_with_kicker`)
**Action**: add `pub defending_player: Option<PlayerId>` field (doc: "CR 508.4: the defending
player of the attacker whose attack triggered this ability; captured at dispatch from
`PendingTrigger.defending_player_id`. Read by `PlayerTarget::DefendingPlayer` and as the
Player fallback of `EffectTarget::AttackTarget`."); initialize `None` in both constructors.

### Change B3 — add the DSL variants

**File**: `crates/card-types/src/cards/card_definition.rs`
**Action**:
- `EffectTarget` (enum at 2446): add `AttackTarget` variant with a doc citing CR 508.4/506.4c.
- `PlayerTarget` (enum at 2480): add `DefendingPlayer` variant citing CR 508.4.

### Change B4 — resolve the new variants

**File**: `crates/engine/src/effects/mod.rs`
**Action**:
- `resolve_player_target_list` (6343): add arm
  `PlayerTarget::DefendingPlayer => ctx.defending_player.filter(|p| alive).into_iter().collect()`
  (mirror the `DamagedPlayer` arm at 6458-6473).
- `resolve_effect_target_list_indexed` (6174): add arm `EffectTarget::AttackTarget =>`:
  look up `ctx.triggering_creature_id` in `state.combat.as_ref()?.attackers`; map
  `AttackTarget::Player(p) → ResolvedTarget::Player(p)` (if alive),
  `AttackTarget::Planeswalker(id) → ResolvedTarget::Object(id)` (if still on battlefield);
  else fall back to `ctx.defending_player → ResolvedTarget::Player`; else `vec![]`.
**Verify**: `DealDamage`'s executor already deals to both `ResolvedTarget::Player` and
`ResolvedTarget::Object` (it handles "any target" spells) — confirm the resolved list is
iterated and damage routed by variant.

### Change B5 — hash the new StackObject field

**File**: `crates/engine/src/state/hash.rs` (~3617-3619, the StackObject `HashInto` block that
already hashes `damaged_player`/`combat_damage_amount`/`triggering_creature_id`)
**Action**: add `self.defending_player.hash_into(hasher);`.

### Change C — exhaustive-match / wire updates

| File | Match / gate | Action |
|------|--------------|--------|
| `crates/engine/src/effects/mod.rs` | `match player` in `resolve_player_target_list` | new `DefendingPlayer` arm (B4) |
| `crates/engine/src/effects/mod.rs` | `match target` in `resolve_effect_target_list_indexed` | new `AttackTarget` arm (B4) |
| `crates/card-types/src/state/stack.rs` | `StackObject` literal/default | add `defending_player` field + init |
| `crates/engine/src/state/hash.rs` | `HashInto for StackObject` | hash new field (B5) |
| `crates/engine/src/rules/protocol.rs` | `PROTOCOL_VERSION` | **7 → 8** (see wire) |
| `crates/engine/src/state/hash.rs` | `HASH_SCHEMA_VERSION` | **45 → 46** (see wire) |
| — | any *other* exhaustive `match` on `EffectTarget`/`PlayerTarget` | **run `cargo build --workspace`** and add arms the compiler names (candidate sites: simulator target enumeration, replay-viewer/tui display, `ForEachTarget`/validation matches). Do not assume this table is exhaustive — the compiler is the authority. |

**Wire-bump justification (machine-forced, do not guess numbers)**:
- **PROTOCOL 7 → 8**: `EffectTarget` and `PlayerTarget` are inside the SR-8 fingerprint
  closure (`Effect::DealDamage.target: EffectTarget`, `Effect::GainLife.player: PlayerTarget`
  → `Characteristics` → the three wire frames). Adding a variant changes
  `PROTOCOL_SCHEMA_FINGERPRINT`; `tests/protocol_schema.rs` will fail with the recomputed
  digest until `PROTOCOL_VERSION` is bumped and a `PROTOCOL_HISTORY` row appended.
- **HASH 45 → 46**: `StackObject.defending_player` is a new field in the `GameState` hash
  closure (`stack_objects`). The hash-schema gate will force the bump; append the history row.
  (The two versions stay separate per SR-8 — the fingerprint closure stops at `GameState`, so
  the StackObject field is a HASH change but not a PROTOCOL one; the enum variants are the
  PROTOCOL change.)

---

## Card Definition Fixes / New Cards

### SHIP (3)

#### `ojutai_soul_of_winter.rs` — NEW (MISS-10)
**Oracle**: "Flying, vigilance / Whenever a Dragon you control attacks, tap target nonland
permanent an opponent controls. That permanent doesn't untap during its controller's next
untap step." 5/6, {5}{W}{U}, Legendary Creature — Dragon.
**Full chain**: `WheneverCreatureYouControlAttacks { filter: TargetFilter { has_subtype:
Some(Dragon), .. } }` → `targets: vec![TargetRequirement::TargetPermanentWithFilter(
TargetFilter { non_land: true, controller: Opponent, .. })]` → `effect: Sequence[
TapPermanent { target: DeclaredTarget{0} }, PreventNextUntap { target: DeclaredTarget{0} }]`.
Keywords Flying + Vigilance. **All primitives exist** (verified: `non_land`, `has_subtype`,
`TapPermanent`, `PreventNextUntap`, `TargetPermanentWithFilter`). Authorable Complete **after
A1/A2** (the target was the only blocker — this is the card W-MISS authored, reviewed, then
removed unshipped). Does NOT need Part B.

#### `hellrider.rs` — FLIP partial → Complete (MISS-4, TODO forced-add)
**Oracle**: "Haste / Whenever a creature you control attacks, this creature deals 1 damage to
the player or planeswalker it's attacking." 3/3, {2}{R}{R}.
**Current state**: `partial`, abilities = `[Keyword(Haste)]` + a TODO naming this exact
primitive (lines 22-38).
**Fix**: replace the TODO with `AbilityDefinition::Triggered { trigger_condition:
WheneverCreatureYouControlAttacks { filter: default }, effect: DealDamage { target:
EffectTarget::AttackTarget, amount: Fixed(1) }, targets: vec![], .. }`; set
`completeness: Complete` (drop the `partial(...)` marker). Uses `EffectTarget::AttackTarget`
(damage recipient may be a planeswalker). *Note: added via pre-existing TODO sweep (was
already in brief; sweep confirms it self-identifies).*

#### `raid_bombardment.rs` — NEW (MISS-4)
**Oracle**: "Whenever a creature you control with power 2 or less attacks, this enchantment
deals 1 damage to the player or planeswalker that creature is attacking." {2}{R}, Enchantment.
**Full chain**: `WheneverCreatureYouControlAttacks { filter: TargetFilter { max_power:
Some(2), .. } }` (the `triggering_creature_filter` is applied by `collect_triggers_for_event`
at abilities.rs:6148-6157 via `matches_filter`, which honors `max_power`) → `effect:
DealDamage { target: EffectTarget::AttackTarget, amount: Fixed(1) }`. No targets. Source is
the enchantment (`ctx.source`). **All primitives exist** after Part B. Authorable Complete.

### DEFERRED / BLOCKED (documented — do NOT author partial)

- **`silumgar_the_drifting_death.rs` — DEFERRED (file OOS-EF3-1)**. "Whenever a Dragon you
  control attacks, creatures **defending player controls** get -1/-1 until end of turn." The
  effect is a one-shot continuous effect (`ApplyContinuousEffect { ContinuousEffectDef {
  filter: EffectFilter, .. } }`) whose affected set must be "creatures the *defending player*
  controls." `EffectFilter` has no defending-player scope, and a continuous effect is
  evaluated by the layer system independently of the resolving `EffectContext`, so the
  defending player would have to be **captured into the registered `ContinuousEffectDef`
  instance** at creation (an `EffectFilter::CreaturesControlledBy(PlayerId)`-style locked
  filter). That is a distinct, larger primitive than `PlayerTarget::DefendingPlayer`. **File
  as OOS-EF3-1**; do not stretch this PB to cover it.
- **`brutal_hordechief.rs` — BLOCKED**. Ability 1 ("defending player loses 1 life and you gain
  1 life") IS expressible now: `Sequence[LoseLife { player: DefendingPlayer, amount: 1 },
  GainLife { player: Controller, amount: 1 }]`. **But** ability 2 ("{3}{R/W}{R/W}: Creatures
  your opponents control block this turn if able, and you choose how those creatures block")
  is inexpressible (force-block-all-opponent-creatures + attacker-chosen block assignment — no
  primitive). Card cannot be `Complete`; leave unauthored. This is the only would-be user of
  the player-only `DefendingPlayer` variant; it unblocks the moment ability 2 becomes
  expressible.
- **`norns_decree.rs` — BLOCKED**. Two gaps: (1) "Whenever one or more creatures an opponent
  controls deal combat damage to you, that opponent gets a poison counter" — batch
  combat-damage trigger + poison; (2) "Whenever a player attacks, if one or more players being
  attacked are poisoned, the attacking player draws a card" — a *player-attacks* trigger with
  a condition over the set of defending players' poison counters. Neither is this PB's
  primitive. (Note: `norns_choirmaster.rs` exists and is a *different* card.)
- **`karazikar_the_eye_tyrant.rs` — BLOCKED**. "Whenever you attack a player, tap target
  creature **that player** controls and goad it" needs a target filter scoped to the defending
  player's creatures (defending-player-controlled — same missing scope as Silumgar) plus goad;
  "Whenever an opponent attacks another one of your opponents…" is a distinct opponent-vs-
  opponent attack trigger. Multiple gaps.
- **`cunning_rhetoric.rs` — BLOCKED, and NOT a MISS-4 shape**. "Whenever an opponent attacks
  **you** and/or one or more planeswalkers you control, exile the top card of **that player's**
  library. You may play that card…" This is a *defender-side* trigger (the enchantment's
  controller is the one being attacked), needing a "whenever a player attacks you" trigger +
  capture of the *attacking* player + play-from-exile with any-color mana. Different primitive
  entirely — classify honestly as out of scope for the defending-player target.

### Mis-listed candidate (not a MISS-10 card)

- **Dragonlord Ojutai** (`dragonlord_ojutai.rs` exists) — its ability is "Whenever Dragonlord
  Ojutai deals combat **damage** to a player, look at the top three cards…" — a
  combat-damage trigger with **no target**. It is neither a MISS-10 (targeted attack trigger)
  nor a MISS-4 case; the brief/WIP double-listed "Ojutai" and "Soul of Winter" as two cards,
  but the actual MISS-10 card is the single card **Ojutai, Soul of Winter**. Verify
  `dragonlord_ojutai.rs`'s current status separately; **out of scope for PB-EF3**.

---

## Unit Tests

**File**: `crates/engine/tests/primitives/pb_ef3_attack_trigger_targets.rs` (new; add
`mod pb_ef3_attack_trigger_targets;` to `crates/engine/tests/primitives/main.rs`).

Part A (MISS-10):
- `test_attack_trigger_forwards_declared_target` — build Ojutai, Soul of Winter + a Dragon
  attacker + an opponent's nonland permanent; declare the attack; auto-target selection picks
  the opponent's permanent; after resolution it is **tapped** and flagged
  doesn't-untap-next-untap-step. CR 508.1m / 601.2c.
- `test_attack_trigger_target_not_dropped_decoy` — same setup; assert the tapped object is the
  intended target. **Fails if** A1's `targets: targets.clone()` reverts to `vec![]` OR if A2's
  runtime-authoritative path reverts to the def raw-index fallback (Ojutai's `Triggered` is at
  def index 2 behind Flying/Vigilance, so the raw-index fallback returns `Flying` and drops
  the target — this is the exact regression, and the decoy pins it).

Part B (MISS-4):
- `test_hellrider_damages_defending_player_4p` — 4-player game; Hellrider's controller (A)
  attacks player **B** with a creature; B loses 1 life. **Decoy**: assert C and D life
  totals unchanged (guards against `EachOpponent` substitution). CR 508.4.
- `test_hellrider_damages_attacked_planeswalker` — attacker attacks a planeswalker C controls;
  the **planeswalker** takes 1 damage (loyalty −1), **not** its controller C's life. Pins the
  `AttackTarget::Planeswalker → ResolvedTarget::Object` branch (CR 506.4c / 508.4).
- `test_defending_player_target_multiplayer` — Brutal-Hordechief-style
  `LoseLife { player: DefendingPlayer, amount: 1 }` authored inline on an enriched test spec
  (BH is not shipped, so construct the ability in-test); attacker attacks B → B loses 1;
  decoy asserts C/D unchanged. Pins `PlayerTarget::DefendingPlayer`.
- `test_raid_bombardment_power_filter` — a power-3 attacker does NOT trigger Raid Bombardment;
  a power-2 attacker does and deals 1 to the defending player. Pins `max_power` filter +
  `AttackTarget`.
- `test_defending_player_captured_survives_attacker_removal` — declare attack (capturing the
  defender), remove the attacker from combat before the trigger resolves, resolve; the
  `DefendingPlayer` life-loss still hits the captured defender (CR 113.7a). Justifies
  capture-at-dispatch over lazy.

**Pattern**: follow the 4-player combat setup in `crates/engine/tests/combat/` and the
existing `SelfAttacks`/annihilator defending-player tests; use `enrich_spec_from_def` for
inline specs (per the existing primitive tests).

---

## Verification Checklist

- [ ] Engine primitives compile (`cargo check -p mtg-engine`)
- [ ] `cargo build --workspace` (proves the GameState seal AND surfaces every exhaustive
      `EffectTarget`/`PlayerTarget` match arm the compiler requires)
- [ ] A1 applied to the attack block **and** all sibling `AbilityDefinition::Triggered` enrich
      blocks that wrote `targets: vec![]`; A2 fallback guarded to `CardDefETB`
- [ ] `PROTOCOL_VERSION` 7→8 + `PROTOCOL_HISTORY` row; `HASH_SCHEMA_VERSION` 45→46 + history
      row (both driven by the failing schema/fingerprint gates, not guessed)
- [ ] 3 cards Complete: `ojutai_soul_of_winter.rs` (new), `hellrider.rs` (flip),
      `raid_bombardment.rs` (new); no remaining TODO in `hellrider.rs`
- [ ] Blocked/deferred cards documented (not authored partial); OOS-EF3-1 filed for Silumgar
- [ ] New tests pass; decoys proven to fail on reverting each fix
- [ ] `cargo test --all` green (proves A1/A2 non-regression across existing targeted triggers)
- [ ] `cargo clippy -- -D warnings`; `cargo fmt --check` + `tools/check-defs-fmt.sh`

---

## Risks & Edge Cases

- **A1/A2 regression surface**: A2 removes the `Normal`-kind def raw-index fallthrough. This
  is only safe *because* A1 forwards targets in every `AbilityDefinition::Triggered` enrich
  block. If a block is missed, a targeted `Normal` trigger on a multi-ability card silently
  loses its target. Mitigation: forward in *all* such blocks (Change A1) and rely on the full
  suite (which exercises dies/ETB targeted triggers) to catch a miss.
- **`ability_index` semantics differ by kind**: `Normal` → runtime `triggered_abilities`;
  `CardDefETB` → `def.abilities`. The A2 guard must key on `trigger.kind`, not on whether
  targets are empty.
- **Attacked-object gone at resolution** (CR 506.4c): `AttackTarget` must resolve to empty
  (fizzle) when the attacked planeswalker was removed and no captured player fallback applies.
  Verify `DealDamage` no-ops cleanly on an empty resolved list.
- **Per-attacker fan-out**: `AnyCreatureYouControlAttacks` fires the trigger on every matching
  permanent per attacker; the capture loop must tag `defending_player_id` on exactly the
  `triggers[pre_len..]` slice for *that* attacker (as the SelfAttacks block does), or a
  multi-attacker turn cross-wires defenders.
- **Wire double-bump**: both PROTOCOL (enum variants) and HASH (StackObject field) move in the
  same PB; keep them separate per SR-8 and append both history rows.
- **`PlayerTarget::DefendingPlayer` ships 0 new Complete cards** — it is a companion to the
  finding and a rider on `AttackTarget`'s capture, pinned by a decoy test; it is not
  speculative infrastructure (the capture is required for `AttackTarget`'s Player case), but
  reviewers should note its card yield lands when Brutal Hordechief / Silumgar's *other*
  blockers clear.
