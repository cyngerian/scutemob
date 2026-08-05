# Primitive Batch Plan: PB-DX21 — CR 508.1: attackers may be declared without limit

**Generated**: 2026-08-04
**Primitive**: a per-combat "the CR 508.1 turn-based action has been performed" marker on
`CombatState`, plus a dedicated `GameStateError` rejecting a second declaration — the attacker-side
mirror of the CR 509.1a blocker guard that already exists.
**CR Rules**: 508.1, 508.1a, 508.1f, 508.1k, 508.1m, 508.2, 508.2b, 508.3a–508.3e, 508.4, 508.4c,
508.8, 506.3, 506.3a–506.3f, 509.1, 509.1a, 509.1g, 509.1i, 500.8, 117.4, 732
**Seed**: `OOS-M11-9` (v3 queue `memory/primitives/seed-rerank-2026-08-02.md` §4 rank 3, lines
873–907)
**Cards affected**: **0 new, 0 completeness flips**; 1 mandatory card-def *comment* correction
(`windbrisk_heights.rs:7-16`) and 2 reachability fixtures used by probes
(`samut_voice_of_dissent`, `nadaar_selfless_paladin`, `aurelia_the_warleader`, `windbrisk_heights`)
**Dependencies**: none (PB-DX6's attack-tax payment fields are already in `Command::DeclareAttackers`
and this batch does not touch them)
**Deferred items from prior PBs**: none carried in. This batch *closes* the last combat-side item
`M11-local` left open.
**Task**: `scutemob-200` · **Branch**:
`feat/pb-dx21-cr-5081-attackers-may-be-declared-without-limit-oos-`
**Stage-0 observations (measured, do not re-derive)**: `memory/primitives/pb-DX21-stage0.md`
**Baseline**: **4,388 / 0 / 5** full-workspace, `--workspace --no-fail-fast` to a file.

---

## 0. What is being fixed, in one paragraph

`crates/engine/src/rules/combat.rs:41-75` guards `handle_declare_attackers` on step
(`:51-55`), active player (`:57-61`), priority holder (`:63-68`) and per-attacker legality
(`:77-195`) — and on nothing else. `:69-72` initialises `CombatState` **only when it is
`None`**, so a second `Command::DeclareAttackers` in the same combat reuses the existing
`CombatState` and runs the whole body again. CR 508.1 makes declaring attackers a
once-per-combat turn-based action. The blocker side has had the matching guard since MR-M6-10
(`combat.rs:1103-1105` → `GameStateError::AlreadyDeclaredBlockers`, `error.rs:63-64`); the
attacker side never got one.

---

## 1. CR research and verdicts

### 1.1 Full CR 508.1 text (from the MCP rules server, verbatim)

> **508.1.** First, the active player declares attackers. This turn-based action doesn't use the
> stack. To declare attackers, the active player follows the steps below, in order. If at any point
> during the declaration of attackers, the active player is unable to comply with any of the steps
> listed below, the declaration is illegal; the game returns to the moment before the declaration
> (see rule 732, "Handling Illegal Actions").
>
> **508.1a** The active player chooses which creatures that they control, **if any**, will attack.
> The chosen creatures must be untapped, they can't also be battles, and each one must either have
> haste or have been controlled by the active player continuously since the turn began.
>
> **508.1b** If the defending player controls any planeswalkers, is the protector of any battles, or
> the game allows the active player to attack multiple other players, the active player announces
> which player, planeswalker, or battle each of the chosen creatures is attacking.
>
> **508.1c** The active player checks each creature they control to see whether it's affected by any
> restrictions […] If any restrictions are being disobeyed, the declaration of attackers is illegal.
>
> **508.1d** The active player checks each creature they control to see whether it's affected by any
> requirements […] If a creature can't attack unless a player pays a cost, that player is not
> required to pay that cost […] If a requirement that says a creature attacks if able during a
> certain turn refers to a turn with multiple combat phases, the creature attacks if able during
> **each declare attackers step** in that turn.
>
> **508.1e** If any of the chosen creatures have banding or a "bands with other" ability, the active
> player announces which creatures, if any, are banded with which.
>
> **508.1f** The active player taps the chosen creatures. Tapping a creature when it's declared as an
> attacker isn't a cost; attacking simply causes creatures to become tapped.
>
> **508.1g** If there are any optional costs to attack with the chosen creatures (expressed as costs
> a player may pay "as" a creature attacks), the active player chooses which, if any, they will pay.
>
> **508.1h** If any of the chosen creatures require paying costs to attack, or if any optional costs
> to attack were chosen, the active player determines the total cost to attack. […] Once the total
> cost is determined, it becomes "locked in."
>
> **508.1i** If any of the costs require mana, the active player then has a chance to activate mana
> abilities (see rule 605, "Mana Abilities").
>
> **508.1j** Once the player has enough mana in their mana pool, they pay all costs in any order.
> Partial payments are not allowed.
>
> **508.1k** Each chosen creature still controlled by the active player becomes an attacking
> creature. It remains an attacking creature until it's removed from combat or the combat phase
> ends, whichever comes first. See rule 506.4.
>
> **508.1m** Any abilities that trigger on attackers being declared trigger.
>
> **508.2.** Second, the active player gets priority.
>
> **508.2a** Abilities that trigger on a creature attacking trigger **only at the point the creature
> is declared as an attacker**. They will not trigger if a creature attacks and then that creature's
> characteristics change to match the ability's trigger condition.
>
> **508.2b** Any abilities that triggered on attackers being declared or that triggered during the
> process described in rules 508.1 are put onto the stack before the active player gets priority […]
>
> **508.3a** An ability that reads "Whenever [a creature] attacks, …" triggers **if that creature is
> declared as an attacker**. […] Such abilities won't trigger if a creature is put onto the
> battlefield attacking.
>
> **508.3b** An ability that reads "Whenever [a player, planeswalker, or battle] is attacked, …"
> triggers if one or more creatures **are declared as attackers** attacking that player or permanent.
> It won't trigger if a creature is put onto the battlefield attacking that player or permanent.
>
> **508.3c** An ability that reads "Whenever [a player] attacks with [a creature], …" triggers if a
> creature that player controls **is declared as an attacker**.
>
> **508.3d** An ability that reads "Whenever [a player] attacks, …" triggers if one or more creatures
> that player controls **are declared as attackers**.
>
> **508.3e** An ability that reads "Whenever [a player] attacks [another player], …" triggers if one
> or more creatures the first player controls are declared as attackers attacking the second player.
> It won't trigger if a creature is put onto the battlefield attacking […]
>
> **508.4.** If a creature is put onto the battlefield attacking, its controller chooses which
> defending player, planeswalker a defending player controls, or battle a defending player protects
> it's attacking as it enters the battlefield […] Such creatures are "attacking" but, for the
> purposes of trigger events and effects, **they never "attacked."**
>
> **508.4c** A creature that's put onto the battlefield attacking or that is stated to be attacking
> **isn't affected by requirements or restrictions that apply to the declaration of attackers**.
>
> **508.6.** A player is "attacking [a player]" if the first player controls a creature that is
> attacking the second player. A player has "attacked [a player]" if the first player **declared** one
> or more creatures as attackers attacking that player.
>
> **508.7a** [reselection] The attacking creature isn't removed from combat and **it isn't considered
> to have attacked a second time**. […]
>
> **508.8.** **If no creatures are declared as attackers** or put onto the battlefield attacking,
> skip the declare blockers and combat damage steps.

### 1.2 VERDICT 1 — declaring attackers is once per combat

**Confirmed.** CR 508.1 opens the *declare attackers step*'s turn-based actions with "**First**, the
active player declares attackers" and CR 508.2 continues "**Second**, the active player gets
priority." The declaration is a single, ordered turn-based action of the step, not a priority action
a player may repeat while they hold priority. CR 508.1d's "during **each declare attackers step**"
and CR 508.7a's "it isn't considered to have attacked a second time" both presuppose exactly one
declaration per declare-attackers step. Rejecting a second one is therefore CR-mandated, not a
policy choice.

### 1.3 VERDICT 2 (the decisive one) — an EMPTY declaration IS a declaration; the guard may **not** key on `combat.attackers`

**The brief's stated preference ("PREFER reading `combat.attackers` over adding a field",
seed-rerank line 899) is refuted. A real marker field is required.** Three independent CR-grounded
reasons, any one of which is sufficient:

1. **CR 508.1a: "chooses which creatures that they control, *if any*, will attack."** The
   empty choice is an explicit, in-rules outcome of the action, not the absence of the action.
   CR 508.8 then defines downstream behaviour for exactly that case ("If **no creatures are declared
   as attackers** … skip the declare blockers and combat damage steps"), which is only meaningful if
   the declaration happened and produced an empty set. So "declared nothing" and "has not declared"
   are different game states, and `combat.attackers.is_empty()` cannot tell them apart.

2. **The empty declaration is a live, shipped client action.** `crates/simulator/src/params.rs:474`
   maps a `LegalAction::DeclareAttackers` with default params to
   `Command::DeclareAttackers { attackers: vec![] }`, and `tools/play-server/src/api.rs:298-306`
   documents it verbatim as "a legal, **irreversible** 'I attack with nothing' that the engine
   accepts silently", explicitly ruling out rejecting it ("declaring no attackers is legal under
   CR 508.1 and rejecting it would deadlock a combat"). An `attackers`-keyed guard would let a
   player who declined to attack then attack anyway — contradicting that doc's own word
   *irreversible*, and CR 508.1.

3. **CR 508.4 / 506.3 populate `combat.attackers` without any declaration.**
   `crates/engine/src/effects/mod.rs:1462-1505` implements "put onto the battlefield attacking" by
   inserting straight into `combat.attackers` (`:1502-1504`) — it never calls
   `handle_declare_attackers`. CR 508.4 says such creatures "never *attacked*" and CR 508.4c says
   they "isn't affected by requirements or restrictions that apply to the declaration of attackers".
   An `attackers`-keyed guard would therefore refuse a player's **first, legal** declaration in any
   combat where a CR 508.4 creature entered attacking first — a *new* correctness bug in the fix.

**Consequence for the plan**: `CombatState` gains a `bool`, it is hashed, and `HASH_SCHEMA_VERSION`
moves. That is accepted and budgeted (§6).

### 1.4 VERDICT 3 — CR 506.3 / 508.4 paths do not go through `handle_declare_attackers`, so the guard cannot break them

**Confirmed by source, not by inference.** The only caller of
`combat::handle_declare_attackers` in the whole tree is `crates/engine/src/rules/engine.rs:445`
(the `Command::DeclareAttackers` arm of `process_command`). The CR 508.4 path is
`effects/mod.rs:1462-1505` (`TokenSpec.enters_attacking` → direct
`combat.attackers.insert(id, target)`), which touches neither the marker nor the guard.
CR 506.3a–506.3f (noncreature / wrong-controller / dead-defender entrants never become attacking)
are likewise decided outside this function. **The guard is therefore invisible to every
"put onto the battlefield attacking" effect, which is exactly what CR 508.4's "never *attacked*"
requires.**

### 1.5 VERDICT 4 — the guard is per **combat**, not per turn (CR 500.8 / 506.5)

**Confirmed by source.** `turn_actions.rs:25/28` dispatch the per-step turn-based actions:
`Step::EndOfCombat => end_combat(state)`, and `end_combat` sets `state.combat = None`
(`turn_actions.rs:2507`) on *entry* to the EndOfCombat step. An extra combat phase re-enters
`Step::BeginningOfCombat` (`turn_structure.rs:57-63` and `:83-88`, LIFO `additional_phases`), whose
turn-based action is `begin_combat` (`turn_actions.rs:1894-1898`), which installs a **fresh**
`CombatState::new(active)`. A marker stored on `CombatState` is therefore automatically cleared at
each combat boundary with no counter to maintain — the same signal `HeuristicBot::in_combat`
(`heuristic_bot.rs:72-80,154-159`) and `PolicyState::in_combat`
(`local_game_playthrough.rs:157-180`) already key on. `engine.rs:2962` also nulls `state.combat` on
the concede/abandoned-turn path (MR-M2-15), which is likewise correct.

**This is not a theoretical case.** `aurelia_the_warleader` is `Complete` and deck-legal, has
**Vigilance** (`aurelia_the_warleader.rs:26`) and a `WhenAttacks` trigger that untaps all your
creatures and grants an additional combat phase (`:52-73`) — PB-DX1 made that trigger actually fire.
MR-M11-09 already found one regression (a per-*turn* cap silently disabling attacks in every extra
combat). Probe **T5** (§4) pins the correct behaviour.

### 1.6 VERDICT 5 — CR 509.1a, the blockers side, is COVERED. Do not widen.

**Confirmed at HEAD.** CR 509.1a is the exact structural twin of CR 508.1a ("chooses which creatures
they control, **if any**, will block"), and CR 509.1 opens the declare-blockers step's turn-based
actions the same way. The engine enforces it at `crates/engine/src/rules/combat.rs:1101-1105`:

```
// MR-M6-10: each defending player may only declare blockers once per combat step
// (CR 509.1a — each defending player declares independently, not repeatedly).
if combat.defenders_declared.contains(&player) {
    return Err(GameStateError::AlreadyDeclaredBlockers(player));
}
```

keyed on `CombatState::defenders_declared: OrdSet<PlayerId>`
(`crates/card-types/src/state/combat.rs:48-50`), inserted on the success path only at
`combat.rs:1652` (inside the same `if let Some(combat) = state.combat.as_mut()` block that records
the blockers, after every validation), and hashed at `state/hash.rs:4430`. A regression probe exists
at `crates/engine/tests/combat/combat.rs:1701`. **PB-DX21 changes nothing on the blockers side.**
The blockers guard *is a set of players* because CR 509.1a lets each defending player declare
independently; the attackers guard *is a bool* because CR 508.1 gives the action to exactly one
player, already named by `CombatState::attacking_player`. That asymmetry is deliberate and must be
documented at the new field (§2.1).

### 1.7 The three consequences the guard closes, re-verified in source

Stage 0 verified all three unmodified at branch HEAD; they are restated here with the line numbers
the probes assert against.

| # | Consequence | Site | Live on |
|---|---|---|---|
| (a) | Declarations **accumulate**; a repeated same-id entry **overwrites that creature's `AttackTarget` mid-combat** (the seed's "overwrites `combat.attackers`" wording is wrong — `:745` is an `insert` into an `OrdMap`, not a replace of the map) | `combat.rs:743-747` | any vigilant attacker |
| (b) | **Every attack trigger re-fires per declaration** — a fresh `GameEvent::AttackersDeclared` (`:795-798`) is pushed and `abilities::check_triggers` + `flush_pending_triggers` run again (`:800-806`). CR 508.2a/508.3a-e say these trigger *only at the point the creature is declared*, i.e. once. | `combat.rs:795-806` | `nadaar_selfless_paladin` (Complete, Vigilance + `WhenAttacks`) |
| (c) | **`attackers_declared_this_turn` is clobbered** — `:759` is an assignment, not `+=` | `combat.rs:753-761`, read by `Condition::YouAttackedWithNOrMore` at `effects/mod.rs:10215` | `legions_landing.rs:76`, `windbrisk_heights.rs:71` (both `YouAttackedWithNOrMore(3)`, both deck-legal) |

**A fourth consequence the brief does not list** (stage 0 §"A FOURTH consequence"): `combat.rs:818-820`
resets `state.turn.players_passed = OrdSet::new()` and re-grants priority to the declarer on every
accepted declaration. A client re-declaring therefore also resets the CR 117.4 pass-round and can
hold a combat open indefinitely **without any attacker changing** — this is the *empty* declaration's
only consequence, and it is a fourth independent reason the guard cannot key on `combat.attackers`.
Probe **T4** covers it.

---

## 2. Engine changes

> **Ordering rule for the runner**: land §2.1 → §2.2 → §2.3 → §2.4 → §2.5 as one compiling unit
> (the field, its `new()` init, the 5 struct-literal sites, the hash arm, the error variant, the
> guard, the set site). Do **not** attempt a partial landing: a field that decides legality but is
> not hashed breaks SR-9b's per-step fingerprint cross-validation.

### Change 1: the marker field

**File**: `crates/card-types/src/state/combat.rs`
**Action**: add one field to `pub struct CombatState`, placed **immediately before**
`defenders_declared` (currently `:48-50`) so the two declaration markers read as a pair:

- name: `attackers_declared`
- type: `bool`
- attribute: `#[serde(default)]` (backward compatibility: an older serialized `CombatState`
  deserialises as `false`, i.e. "the declaration has not been performed". This is the only value
  expressible and the runner must state in the field doc that it is a deliberate, lossy default —
  an old snapshot mid-combat will permit one extra declaration. Precedent: PB-SFT's
  `Effect::SacrificePermanents.filter`, hash History `- 9:`.)

**Doc comment must state, in this order**:
1. CR 508.1 — declaring attackers is a once-per-combat turn-based action; this records that it has
   been performed.
2. CR 508.1a + CR 508.8 — **`true` even for an empty declaration.** Do not replace this field with
   `!attackers.is_empty()`; see §1.3 of this plan and the three reasons there.
3. CR 508.4 / 506.3 — creatures **put onto the battlefield attacking** populate `attackers` without
   ever setting this flag, because CR 508.4 says they "never *attacked*". Cite
   `effects/mod.rs:1502-1504`.
4. Why this is a `bool` and its sibling `defenders_declared` is an `OrdSet<PlayerId>`: CR 508.1
   gives the action to exactly one player (`attacking_player`); CR 509.1a lets each defending player
   declare independently.
5. Cleared naturally when `CombatState` is dropped at end of combat
   (`turn_actions.rs:2507`) and rebuilt fresh at `BeginningOfCombat`
   (`turn_actions.rs:1897`) — so the marker is **per combat phase**, CR 500.8 / 506.5.

**Also**: `CombatState::new` (`combat.rs:85-98`) gains `attackers_declared: false,` in the struct
literal, adjacent to `defenders_declared: OrdSet::new(),` at `:92`.

### Change 2: the error variant

**File**: `crates/engine/src/state/error.rs`
**Action**: insert a new variant **directly above** `AlreadyDeclaredBlockers` (currently `:63-64`),
so the two combat guards are adjacent in the enum:

```
/// CR 508.1 (PB-DX21 / OOS-M11-9): <doc — see below>
#[error("player {0:?} has already declared attackers this combat phase")]
AlreadyDeclaredAttackers(PlayerId),
```

- **Name**: `AlreadyDeclaredAttackers`. **Shape**: single unnamed `PlayerId`, exactly mirroring
  `AlreadyDeclaredBlockers(PlayerId)`.
- **It must carry no `ObjectId` and no card name.** `tools/play-server/src/api.rs:105` renders a
  `Rejected(GameStateError)` as **text** in a 422 addressed to the acting seat; a variant naming an
  object would be a new hidden-information channel (Architecture Invariant 7 — and MR-M11-01 twice
  over: a redaction gate checks the channel it was written for). A bare `PlayerId` is public
  information.
- **Doc comment must include the SR-8 note**, mirroring `BlockedByPendingDecision`'s at `:98-100`:
  *"Not part of the SR-8 wire closure (`GameStateError` is reachable from none of
  `Command`/`GameEvent`/`ReplayLog`) — adding this variant is not a protocol change."* This is a
  claim the runner must **verify by executing the gate** (§6.2), not assume.
- **No exhaustive-match sites exist.** A tree-wide search for a `match` on `GameStateError` returns
  none (the type is consumed through `thiserror`'s `Display`/`Debug`). `tools/play-server`'s
  `halt_reason_summary` (`view.rs:2464-2490`) enumerates `HaltReason`, not `GameStateError`, and
  deliberately does **not** forward the engine text. Verify with `cargo build --workspace`.

### Change 3: the guard — exact placement

**File**: `crates/engine/src/rules/combat.rs`
**Action**: insert the guard as a new block **between line 68 (the closing `}` of the
`NotPriorityHolder` check) and line 69 (the `// Initialize CombatState if not already set` comment)**.

```
// CR 508.1 (PB-DX21, OOS-M11-9): declaring attackers is a once-per-combat
// turn-based action. Rejected HERE, before the CombatState init below and
// before any validation, tapping (508.1f) or cost payment (508.1j), so a
// refused re-declaration leaves the game byte-identical (CR 732: "the game
// returns to the moment before the declaration").
if state.combat.as_ref().is_some_and(|c| c.attackers_declared) {
    return Err(GameStateError::AlreadyDeclaredAttackers(player));
}
```

**Why exactly there — all four constraints are binding:**

| Constraint | Reason |
|---|---|
| **After** the step check (`:51-55`) | error precedence must mirror the blockers side, which checks the step first (`:1085-1089`). A `DeclareAttackers` outside the step is a `InvalidCommand`, not an `AlreadyDeclared`. |
| **After** the active-player (`:57-61`) and priority (`:63-68`) checks | a non-active or non-priority player must still get the specific error they get today; changing that would move existing assertions for no reason. |
| **Before** `state.combat = Some(CombatState::new(player))` at `:69-72` | **that assignment is a state mutation.** Guarding after it would install a `CombatState` as a side effect of a rejected command. |
| **Before** the attacker-validation loop (`:77-195`), the attack-tax affordability check (`:294-354`) and, crucially, the tax **payment** at `:713-741` | an early return that has already taken mana or life would be a regression. The tax path debits `ps.mana_pool` (`:719`) and `ps.life_total` (`:734`). |

The guard reads `state.combat` immutably and returns before any `as_mut()`, so it is trivially
side-effect free.

### Change 4: the marker set site — success path only

**File**: `crates/engine/src/rules/combat.rs`
**Action**: set the marker inside the existing `if let Some(combat) = state.combat.as_mut()` block
at `:743-747`, immediately after the attacker-insert loop:

```
if let Some(combat) = state.combat.as_mut() {
    for (attacker_id, target) in &attackers {
        combat.attackers.insert(*attacker_id, target.clone());
    }
    // CR 508.1 / 508.1a / 508.8 (PB-DX21): the turn-based action has now been
    // performed. Set on the SUCCESS path only -- every `return Err` above leaves
    // it clear, so a rejected declaration (an unaffordable CR 508.1h tax, an
    // illegal attacker, a goad violation) does NOT lock out a legal retry.
    // Set even when `attackers` is EMPTY: CR 508.1a's "if any" makes the empty
    // choice a completed declaration, and CR 508.8 defines the game's behaviour
    // for it. Mirrors `handle_declare_blockers`' `defenders_declared.insert` at
    // :1652, which is likewise inside this same shape.
    combat.attackers_declared = true;
}
```

**Why this exact site:**

- It is **after every `return Err`** in the function (the last is the exert validation ending at
  `:669`), so a rejected declaration cannot set the marker. This is what makes the
  retry-after-rejection tests in `pb_dx6_unflattened_payment_sites.rs` and
  `pb_dp4_attack_tax_and_payment_deadline.rs` safe. Probe **T6** pins it.
- It is **before** the CR 603.3d suspended-trigger early return at `:811-813`
  (`if state.pending_trigger_targets.is_some() { … return Ok(events) }`). That path has already
  **accepted** the declaration; it merely owes a priority grant. If the marker were set after it, a
  declaration that suspended on a trigger target choice would leave the marker clear and be
  re-declarable — the exact bug, in the one path hardest to test.
- It is structurally identical to the blockers side (`:1645-1653`), which is the strongest argument
  a reviewer can check cheaply.

**Do NOT** put the marker set inside the `if !attackers.is_empty()` block at `:753-761` — that block
is the raid-count/`attacked_this_turn` bookkeeping and is deliberately gated on non-emptiness (CR
508.4 / Bloodsoaked Champion ruling). Putting the marker there would reintroduce the exact hole
§1.3 exists to close.

### Change 5: hashing

**File**: `crates/engine/src/state/hash.rs`
**Action**: in `impl HashInto for CombatState` (`:4422-4444`), add

```
// CR 508.1 (PB-DX21): the once-per-combat declaration marker.
self.attackers_declared.hash_into(hasher);
```

immediately **before** `self.defenders_declared.hash_into(hasher);` at `:4430`, matching the struct
field order.

**Critical, and load-bearing for §4's probe T7**: `tests/core/hash_schema.rs:709-716` states the
suite's own coverage cap — the canonical fixture builder **cannot populate `combat`** (it is one of
the five named exclusions, alongside `stack_objects`, `pending_triggers`, `replacement_effects`,
`lki_objects`). So `stream_fingerprint` will move only via the v40 mechanism
(`HASH_SCHEMA_VERSION` is the stream's first byte) — the v69/v72 version-sentinel-byte-only case,
**not** the v70/v71 payload-bytes case. **The new field's own bytes are therefore covered by no
existing gate**, and a direct `HashInto` unit test (T7) is mandatory. This is the exact hazard
ENG-1's review Finding 1 and ENG-2's History row both record; do not repeat it.

### Change 6: every struct-literal construction site of `CombatState`

`CombatState` has **no** `Default` impl, so every struct literal is exhaustive and a new field is a
compile error at each. Grep `defenders_declared:` — there are exactly **5** literal sites, all in
tests (the ~110 other construction sites use `CombatState::new(..)` and are unaffected):

| File | Line | Action |
|---|---|---|
| `crates/engine/tests/rules/static_grants.rs` | 463 (`*state.combat_mut() = Some(CombatState {`) — field at **471** | add `attackers_declared: false,` |
| `crates/engine/tests/rules/static_grants.rs` | 991 — field at **1002** | add `attackers_declared: false,` |
| `crates/engine/tests/primitives/primitive_pb_xa.rs` | 66-67 (`fn combat_with_attacker`) — field at **75** | add `attackers_declared: false,` |
| `crates/engine/tests/primitives/primitive_pb_xa2.rs` | 64-65 (`fn combat_with_attacker`) — field at **73** | add `attackers_declared: false,` |
| `crates/engine/tests/primitives/primitive_pb_xa2.rs` | 83-84 (`fn combat_with_blocker`) — field at **92** | add `attackers_declared: false,` |

`false` is correct at all five: these fixtures hand-build a combat to exercise layer filters and
`TargetFilter.is_attacking`/`is_blocking`, and none of them issues a `Command::DeclareAttackers`.
The runner must **re-derive this list by grep**, not trust the table, and report any site the table
misses.

### Change 7 (REQUIRED BY CONSEQUENCE): the legal-action offer

> **This is the one item in the plan that goes beyond the coordinator's three-bullet scope, and it
> is not optional — it is forced by the mandated deletion in §3.2. Read §3.3 before deciding.
> If the coordinator refuses it, §3.2 cannot be executed as written and the runner must stop and
> report rather than weaken an existing green assertion.**

**File**: `crates/simulator/src/legal_actions.rs`
**Action**: at `:878`, extend the offer condition so the action is not offered once the CR 508.1
action has been performed:

```
if state.turn().step == Step::DeclareAttackers
    && is_active
    && stack_empty
    && !state.combat().as_ref().is_some_and(|c| c.attackers_declared)
{
```

(the exact accessor spelling must match the sealed-state API `legal_actions.rs` already uses for
`state.combat()` at `:924`; the runner adapts.)

**Rationale, in the project's own terms:**

1. **SR-38.** The provider must not offer an action the engine will refuse.
   `local_game_playthrough.rs:424-434` asserts precisely this for the human path — *"Any error at
   all is a failure — `Rejected` included. The policy only ever submits an action the game offered
   it one instant earlier, so a rejection means the offer was wrong."*
2. **It is what actually fixes the user-visible symptom.** `OOS-M11-9`'s stated symptom is "a human
   clicking attack twice in the browser". The engine guard alone converts a silent state corruption
   into a 422 error strip with the button still present. With the offer suppressed the button
   disappears, which is the correct UX and closes the seed as a *player-facing* defect.
3. **PB-DX20's precedent, exactly.** That batch deleted `KNOWN_FALSE_OFFERS` with its whole
   mechanism so that "any refusal in that driver is now fatal, which is what proves the closure".
   The same shape applies here: with the offer suppressed and the client mitigations deleted,
   `local_game_playthrough`'s rejection-is-fatal assertion becomes the proof.
4. **SIM-6's precedent.** `LegalAction::ActivateAbility` suppression when the eligible sacrifice set
   is empty, mirroring `offerable_cast_plan`.
5. **The S8 argument for *not* touching the provider is stale.** `heuristic_bot.rs:106-110` justifies
   the client-side mitigation as keeping "the provider's action list, and therefore every recorded
   `mtg-fuzzer` seed, untouched". PB-DX22 (`OOS-DX22-7`) already declared **every** pre-2026-08-03
   fuzz seed dead, and PB-DX32's fix cycle (L6) wrote the `MOVED_MSG` re-measure instruction into
   the seeded gates for exactly this eventuality.

**Blast radius, to be MEASURED not predicted** (§5 Stage 4): every seeded fixture whose bot picks
uniformly from the action list can diverge. The known candidates:
`crates/simulator/tests/pb_dx32_fuzz_output.rs` T2.2 / T3.1 / T4.1 / T4.3 / T6.3 (their `MOVED_MSG`
already names them), `crates/simulator/tests/sim5_bot_cast_discipline.rs` T3.3,
`crates/simulator/tests/local_game_playthrough.rs`, `crates/simulator/tests/pb_dx22_fuzz_instrument.rs`,
and any `tools/play-server` HTTP probe pinning an action list or `command_count`. **Note that
suppression only changes the list in a window where the active player has already declared and the
stack is empty — the runner must A/B rather than assume the churn is large or small.**

**The blockers side stays unsuppressed** (`legal_actions.rs:923-954` still offers
`DeclareBlockers` after `defenders_declared` contains the player — this is why PB-DX32's fuzz
rejection table shows `AlreadyDeclaredBlockers (4)`). File it as a seed (§8 `OOS-DX21-2`); do
**not** widen into it, per the coordinator.

---

## 3. Deleting the two client-side mitigations

### 3.1 Mitigation A — `HeuristicBot`'s `RepeatKey::DeclareAttackers` cap

**File**: `crates/simulator/src/heuristic_bot.rs`

**What it is**: `RepeatKey` (`:27-35`) has a `DeclareAttackers` variant whose `cap()` is **1**
(`:45-49`); `RepeatKey::of` maps `LegalAction::DeclareAttackers` to it (`:60`);
`is_capped_repeat` (`:164-171`) makes `score_action` return **0** for a capped action (`:176-178`),
which is *below* `PassPriority`'s 1, so the bot passes instead. `refresh_repeat_scope`
(`:147-160`) resets the two combat keys on the `combat.is_none() → is_some()` edge
(`in_combat`, `:72-80`), which is what makes the cap per-combat rather than per-turn — the
MR-M11-09 regression fix documented at `:116-129`. Its own doc calls it "a **preference** cap, not a
legality cap" (`:112-114`).

**Why it was added**: `:95-102` — M11-local S8, seed 1, turn 19, **20,000 commands**, because
"Neither `StubProvider` nor `combat.rs::handle_declare_attackers` gates 'attackers have already
been declared this combat'". Both halves of that sentence are false after this batch.

**Why deleting it is required**: leaving it re-creates the "harmless because unreachable" argument
SIM-1 already burned this project on. Concretely: the cap is the only thing that would keep the bot
seats away from the new guard, so with it in place **no bot game would ever exercise the guard**,
and a regression that dropped the marker would be invisible to every simulator and fuzz test in the
tree. Its doc is also, post-batch, an *aspirationally-wrong comment* in the conventions.md sense
(`memory/conventions.md:216-230`): it asserts a live engine gap that no longer exists.

**Exact deletions**:
- `RepeatKey::DeclareAttackers` variant (`:31-32`).
- Its arm in `cap()` (`:45-49`) — the remaining arm keeps `DeclareBlockers` at 1; rewrite the
  comment so it no longer speaks for both.
- Its arm in `RepeatKey::of` (`:60`).
- **Rewrite, do not delete**, the `repeats_this_turn` doc block (`:81-130`): instance 2 (`:95-102`)
  becomes a *historical* record with "**CLOSED by PB-DX21** (`OOS-M11-9`) —
  `handle_declare_attackers` now rejects a second declaration with
  `GameStateError::AlreadyDeclaredAttackers`, and `legal_actions.rs` no longer offers it", and the
  MR-M11-09 extra-combat note (`:116-129`) must be kept and re-scoped to `DeclareBlockers` alone.
  Same for the `RepeatKey` type doc at `:21-26` ("re-declaring the same combat with a *different*
  attacker set…") and `in_combat`'s doc at `:70-80`.
- **Keep** `in_combat` and the combat-scoped reset in `refresh_repeat_scope` — `DeclareBlockers` is
  still combat-scoped and still capped.

**Test churn caused**: bot behaviour changes only in a combat where the bot has already declared and
still has an untapped eligible creature. With §2.7's suppression in place the action is no longer
offered there, so the bot's *chosen* action is the same one the cap used to force — meaning the
churn from A + C together is expected to be **small**, and possibly zero on many seeds. **Measure it
(§5 Stage 4); do not claim it.** Without §2.7, deleting the cap makes the bot pick a
now-illegal action, get a `Rejected`, and fall back to `PassPriority`
(`local_game.rs:931-942` — verified: a rejected bot command is *not* fatal, it records a
`RejectedCommand` and passes), so there is **no livelock**, but there is one extra recorded
rejection per priority window in that step and every seeded fixture moves anyway.

### 3.2 Mitigation B — the scripted human policy's per-combat cap

**File**: `crates/simulator/tests/local_game_playthrough.rs`

**What it is**: `struct PolicyState` (`:157-169`) with `in_combat` and
`declared_attackers_this_combat`, `PolicyState::refresh_scope` (`:171-180`) mirroring the bot's
combat-entry edge, the `policy.refresh_scope(state)` call in `choose` (`:214`), the
`if policy.declared_attackers_this_combat == 0 {` gate wrapping step 3 of the policy (`:280-296`,
with the increment at `:293`), and the `let mut policy = PolicyState::default();` + threading in
`play()` (`:405-413`).

**Why it was added**: `:143-151` — SIM-1 made the human's commander castable, seed 1's human
commander is `Samut, Voice of Dissent` (**Vigilance**), and without the cap seed 1 halted
`InfiniteLoop` at turn 17 having applied exactly 20,000 commands, **19,351 of them
`DeclareAttackers` in one turn**.

**Why deleting it is required**: identical argument to §3.1, plus a stronger one — this test's own
failure contract (`:424-434`) is *"the policy only ever submits an action the game offered it one
instant earlier, so a rejection means the offer was wrong."* With the cap deleted **that assertion
becomes the closure proof for `OOS-M11-9`**: the test can only stay green if the offer layer and
the engine agree. Leaving the cap in place would mean the batch shipped a guard that its own
end-to-end driver never touches.

**Exact deletions**: the whole `PolicyState` struct and `impl` (`:127-180`), the `policy` parameter
of `choose` (`:208-212`), the `refresh_scope` call (`:214`), the `if … == 0 {` wrapper and
increment (`:280`, `:293`, `:296`), and the `policy` local + call-site threading in `play()`
(`:405-407`, `:413`). Step 3's own comment (`:272-279`) must be rewritten to cite PB-DX21 rather
than describe the cap.

**Test churn caused**: this is the crux — see §3.3.

### 3.3 The forced consequence, stated plainly

Deleting §3.2's cap **without** §2.7's offer suppression turns
`test_s8_scripted_human_playthrough_is_clean_on_five_seeds` **RED**, by construction: the policy
will re-declare, the engine will now reject, and `:424-434` records that as
`error = Some("engine rejected a just-offered action (DeclareAttackers): …")`.

There are exactly three responses, and only one is acceptable:

| Option | Verdict |
|---|---|
| **A. Ship §2.7 (suppress the offer).** Policy falls through to `PassPriority`; the test stays green; the rejection-is-fatal assertion becomes the closure proof. | **RECOMMENDED — this is the plan's default.** |
| **B. Weaken the test** so an `AlreadyDeclaredAttackers` rejection is tolerated. | **REFUSED.** It guts the only SR-38 end-to-end assertion in the tree, to work around a defect the batch is supposed to close. |
| **C. Keep the policy cap, keyed on the engine's own marker instead of a private counter.** | **REFUSED.** It is not a deletion; the coordinator's brief mandates deletion precisely so the mitigation cannot mask a regression. |

If the coordinator declines §2.7 the runner must **stop and report** (conventions.md
"Implement-phase default-to-defer", `memory/conventions.md:205-214`), not pick B or C.

---

## 4. Probes

**File**: **new** `crates/engine/tests/primitives/pb_dx21_declare_attackers_once_per_combat.rs`
**Registration (SR-9a)**: add `mod pb_dx21_declare_attackers_once_per_combat;` to
`crates/engine/tests/primitives/main.rs`, in lexicographic position **between**
`mod pb_dx20_keyword_carried_target_requirements;` (`:34`) and `mod pb_dx2_command_gates;` (`:35`).
**Never** add a top-level `crates/engine/tests/*.rs` — `tests/no_stray_test_binaries.rs` fails the
suite if one reappears, and a dropped `mod` line silently deletes the whole file's coverage.

**Every probe below must be watched failing by an executed revert.** The revert for T1–T6 is
*delete the guard block from `combat.rs` (§2.3)*, rebuild (confirm `Compiling mtg-engine` appears in
the captured output — a stale binary is a silent pass, PB-DX32 §7 R7), run, record the exact failure
text, restore, and confirm `git diff` clean before the next one. T7's revert is *delete the
`attackers_declared.hash_into(hasher)` line*.

### T1 — (a) attack-target overwrite

**Fixture**: `samut_voice_of_dissent` (`Complete`; **Vigilance** at
`samut_voice_of_dissent.rs:31`, so she stays untapped after declaring and `combat.rs:115`'s
already-tapped check cannot mask the defect — that check is "an accident, not a guard", per the
brief). 4-player table, `Step::DeclareAttackers`, p1 active with priority, Samut on p1's
battlefield with no summoning sickness.

1. `process_command(Command::DeclareAttackers { player: p1, attackers: vec![(samut, AttackTarget::Player(p2))], .. })` → `Ok`.
2. `process_command(… attackers: vec![(samut, AttackTarget::Player(p3))] …)`.

**Assertions**: (2) is `Err(GameStateError::AlreadyDeclaredAttackers(p1))` (match on the variant,
not the string); and `state.combat().unwrap().attackers.get(&samut) == Some(&AttackTarget::Player(p2))`
— *the target did not move*. Assert both, in that order.
**Pre-fix behaviour to record in the doc comment**: (2) returns `Ok` and the target is `p3`.

### T2 — (b) attack-trigger re-fire

**Fixture**: `nadaar_selfless_paladin` (`Complete` explicitly at `:81`; **Vigilance** at `:34`
**and** `TriggerCondition::WhenAttacks` with `once_per_turn: false` at `:47-56` — the only
`Complete` def in the corpus that is both, verified by intersecting the `KeywordAbility::Vigilance`
and `TriggerCondition::WhenAttacks` grep sets over `crates/card-defs/src/defs/`; the other two in
the intersection are `sun_titan` (`partial`) and `aurelia_the_warleader` (`once_per_turn: true`,
which would *absorb* the re-fire and make the probe vacuous — do not use her here)).

Declare Nadaar attacking p2, resolve/flush, then attempt a second declaration of Nadaar.

**Assertions**, in order of strength:
1. the second command is `Err(AlreadyDeclaredAttackers(p1))`;
2. the number of `GameEvent::AttackersDeclared` observed across the whole combat is exactly **1**;
3. the `WhenAttacks` trigger fired exactly once — assert on the observable the effect leaves.
   `Effect::VentureIntoDungeon` is the effect; the runner must **first check by running** whether
   the venture path is choice-free in this fixture (there are precedents in
   `crates/engine/tests/mechanics_a_d/dungeon_cards.rs` and `dungeon_resolution.rs`). **If it
   raises a blocking decision, do not fight it** — substitute assertion (3) with a count of queued
   `PendingTrigger`s / of `StackObject`s attributable to Nadaar's ability after each command, and
   say in the test doc that the dungeon observable was declined for that reason. Do not silently
   drop assertion (3).

**Pre-fix behaviour**: two `AttackersDeclared` events and two ventures.

### T3 — (c) `attackers_declared_this_turn` raid-count clobber

**Fixture**: p1 controls three creatures (at least one vigilant — `samut_voice_of_dissent`) plus
**`windbrisk_heights`** (`Complete` by derive; `activation_condition:
Some(Condition::YouAttackedWithNOrMore(3))` at `:71`). `legions_landing.rs:76` carries the same
condition and may be used as a second, cheaper state-level assertion.

1. Declare **three** attackers. Assert `state.expect_player(p1).attackers_declared_this_turn == 3`.
2. Attempt a second declaration naming only the vigilant creature.
3. Assert `Err(AlreadyDeclaredAttackers(p1))` **and** `attackers_declared_this_turn` is **still 3**.
4. **The consequence assertion, which is what makes this probe about a card and not a field**:
   Windbrisk Heights' `{W},{T}` ability is still activatable (its `Condition::YouAttackedWithNOrMore(3)`
   still holds), exercised through the real activation path, not by reading the condition.

**Pre-fix behaviour**: step 2 succeeds, `attackers_declared_this_turn` becomes **1**, and Windbrisk
Heights goes dead for the rest of the turn.

### T4 — the EMPTY declaration counts as a declaration (CR 508.1a / 508.8 / 117.4)

**This probe is mandatory**: §1.3 is the plan's single largest deviation from the brief, and this is
the test that makes it real.

Same table as T1 (Samut untapped and eligible, so the second declaration would otherwise be legal
in every other respect).

1. `Command::DeclareAttackers { attackers: vec![] }` → `Ok`. Assert
   `state.combat().unwrap().attackers.is_empty()` — the map the brief wanted to key on is **empty**.
2. `Command::DeclareAttackers { attackers: vec![(samut, AttackTarget::Player(p2))] }` →
   assert `Err(AlreadyDeclaredAttackers(p1))`.
3. Assert `state.combat().unwrap().attackers.is_empty()` still.
4. **The fourth consequence (stage 0)**: capture `state.turn().players_passed` immediately after
   step 1, have another player pass, then attempt the rejected declaration in step 2, and assert
   `players_passed` was **not** reset by it (`combat.rs:818` runs only on the success path). This
   pins that a rejected re-declaration cannot hold the CR 117.4 pass-round open.

**The test doc must state, in words, that an `!attackers.is_empty()` guard passes steps 1–3
and fails nothing** — i.e. this test is the discriminator between the two candidate
implementations, not just between fixed and unfixed. The runner must **execute that second
revert too**: temporarily replace the guard's condition with
`state.combat.as_ref().is_some_and(|c| !c.attackers.is_empty())`, rebuild, confirm T4 goes red
while T1/T2/T3 stay green, restore. Record both revert outputs.

### T5 — the marker is per COMBAT, not per turn (CR 500.8 / 506.5)

**Fixture**: `aurelia_the_warleader` (`Complete`; Vigilance `:26`, `WhenAttacks`/`once_per_turn:
true` → `Effect::Sequence([untap all creatures you control, AdditionalCombatPhase { followed_by_main:
false }])` at `:52-73`).

Declare Aurelia attacking p2 in the first combat; resolve her trigger; advance through
EndOfCombat into the extra combat phase; declare attackers again in combat #2.

**Assertions**: the second combat's declaration is **`Ok`**; and (non-vacuity, so the probe cannot
pass by never reaching combat 2) `state.turn().in_extra_combat == true` at that moment and
`state.combat().unwrap().attackers_declared == false` immediately after `begin_combat` installed the
fresh `CombatState`.

This probe is the direct successor to MR-M11-09 and must cite it.

### T6 — the marker is set on the SUCCESS path only

**Fixture**: the cheapest rejection available *after* the guard — an unaffordable CR 508.1h attack
tax. Reuse the shape of `pb_dx6_unflattened_payment_sites.rs`'s helpers (a synthetic
`GameRestriction::CantAttackYouUnlessPay { cost_per_creature }` on a p2 permanent, `combat.rs:257`)
with an **empty** p1 mana pool.

1. Declare one attacker into the taxed defender → `Err(InvalidCommand("attack tax: …"))`.
2. Assert `state.combat()` is either `None` **or** has `attackers_declared == false` (the guard
   runs before the `CombatState` init, so which of the two holds depends on whether a prior
   `BeginningOfCombat` ran — assert the disjunction and say why).
3. Add mana to p1's pool and re-issue the identical command → **`Ok`**.
4. Assert `attackers_declared == true` after step 3.

**Revert for T6 is different from T1–T5**: move the `combat.attackers_declared = true;` assignment
from `:743-747` up to just after the guard (i.e. set it on entry). Rebuild, confirm T6 reddens at
step 3 (`AlreadyDeclaredAttackers` on a legal retry), restore.

This probe is the one that protects the ~20 existing retry-after-rejection call sites in
`pb_dx6_unflattened_payment_sites.rs` (21 `Command::DeclareAttackers` occurrences) and
`pb_dp4_attack_tax_and_payment_deadline.rs`.

### T7 — the new field is actually in the hash stream

**Mandatory, and it is the only gate that covers the field's bytes** — see §2.5. Build two
`CombatState` values via `CombatState::new(p1)` that differ **only** in `attackers_declared`,
feed each through `HashInto` into a fresh hasher (the direct-unit-test pattern
`crates/engine/tests/primitives/pb_eng2_targets_announced.rs` established for the v72 bump), and
assert the two digests differ.

**Revert**: delete the `self.attackers_declared.hash_into(hasher);` line, rebuild, confirm T7 goes
red with both digests equal, restore.

### T8 — (optional but cheap) the CR 509.1a twin still holds

A one-line companion asserting `AlreadyDeclaredBlockers` still fires, so a future refactor of the
attacker guard cannot collaterally break its sibling. `crates/engine/tests/combat/combat.rs:1701`
already covers this; if the runner judges T8 redundant, **say so in the plan-execution notes rather
than omitting it silently**.

---

## 5. Execution stages

### Stage 0 — baseline (already done; re-confirm only)

`memory/primitives/pb-DX21-stage0.md` records **4,388 / 0 / 5**. Re-run
`cargo test --workspace --no-fail-fast` **to a file** (never `| tail` — a tail pipe hid a compile
failure and faked a green run on 2026-08-02) before any edit and confirm the number. Also record
`PROTOCOL_VERSION` and `HASH_SCHEMA_VERSION` by **executing**
`cargo test -p mtg-engine --test core protocol_schema` and `--test core hash_schema`, not by reading
the constants.

### Stage 1 — the primitive (engine only, no client)

Changes 1–6 (§2.1–§2.6) + probes T1–T7. Gates: `cargo build --workspace`,
`cargo test -p mtg-engine`, `cargo clippy --workspace --all-targets -- -D warnings`,
`cargo fmt --check`, `tools/check-defs-fmt.sh`.
Expect `core hash_schema` to FAIL here — that is the gate doing its job; §6.1 is how to answer it.

### Stage 2 — HASH bump

§6.1, computed from the failing gate's own output. Re-pin every sentinel **by symbol**, not by
memory of the count.

### Stage 3 — the offer layer (§2.7)

`legal_actions.rs` one condition. Gates: `cargo test -p mtg-simulator`, `cargo test -p play-server`.
Record the before/after of any pin that moves; do not re-tune a seed silently.

### Stage 4 — delete both mitigations (§3.1, §3.2) and re-measure

Run `cargo test --workspace --no-fail-fast` to a file. Triage **every** new failure into one of:
(i) a legitimate seed move from Stage 3 → re-measure and re-pin with the reason recorded at the pin;
(ii) a test that genuinely declared twice in one combat → §7;
(iii) a real defect in this batch → fix.
Never re-pin without classifying.

### Stage 5 — docs and comments

§7.3. Includes the **mandatory** `windbrisk_heights.rs` comment correction, which touches
`crates/card-defs/` and therefore requires `tools/check-defs-fmt.sh` (SR-35: `cargo fmt` checks
**zero** of the 1,803 defs and still exits 0 — this exact trap caught PB-DX19).

### Stage 6 — close-out

Coverage regeneration (`tools/authoring-report.py`, expect a byte-identical body modulo the
sha/date stamp lines, then revert the churn), seed filing (§8), handoff.

---

## 6. Fingerprints

### 6.1 HASH — **expected to MOVE, 72 → 73**

`CombatState` is inside the `GameState` serde closure (`GameState::combat: Option<CombatState>`;
`hash.rs:1152` imports it), so `declaration_fingerprint_is_pinned`
(`tests/core/hash_schema.rs:1121-1139`) **will** fail. That is expected and correct.

**The bump must be computed from the failing gate's own output, never predicted.** The gate prints
the new digest in its own failure message. Do all of the following in one commit:

1. `pub const HASH_SCHEMA_VERSION: u8 = 72;` → `73` (`hash.rs:743`).
2. Append a `/// - 73: PB-DX21 (2026-08-04, OOS-M11-9 — CR 508.1's once-per-combat declaration
   marker): …` History line above it (the suite requires the `- N:` line by convention). It must
   state: `CombatState` gains `attackers_declared: bool` (`#[serde(default)]`), reachable from
   `GameState` via `combat: Option<CombatState>`; **`decl_fingerprint` MOVES** (new field in the
   serde closure); **`stream_fingerprint` moves per the v40 mechanism only**
   (`HASH_SCHEMA_VERSION` is the stream's first byte) because `canonical_fixture()` **cannot**
   populate `combat` (`tests/core/hash_schema.rs:711-716` names it as one of the five exclusions) —
   the v69/v72 version-sentinel-byte-only case, with the field's own bytes covered by probe **T7**
   and by nothing else; and **`PROTOCOL_VERSION` is UNMOVED** because `CombatState` is reachable
   from none of `Command`/`GameEvent`/`ReplayLog` (verified: zero occurrences of `CombatState` in
   `crates/engine/src/rules/protocol.rs`).
3. **APPEND** a `HashSchemaEpoch { version: 73, … }` row to `HASH_SCHEMA_HISTORY`
   (`hash.rs:805-…`, after the v72 row at `:1133-1149`) with both digests pasted from the gate's
   output. **Never edit a shipped row** — `frozen_prefix` pins a digest of the whole prior history.
4. Re-pin every `HASH_SCHEMA_VERSION` sentinel. **Measured at branch HEAD: 43 occurrences of
   `HASH_SCHEMA_VERSION, 72u8` across 42 test files, plus 1 occurrence of `HASH_SCHEMA_VERSION, 72`
   (no suffix) at `crates/engine/tests/core/hash_schema.rs:1249` — 44 total.** The runner must
   **re-derive this by grep** and report the count it actually found; do not trust this number.
   Note the two spellings (`72u8` and bare `72`) — a single-pattern sweep will miss the second.
   `tools/play-server` reads the constant symbolically (`view.rs:2544`, `main.rs:3041`) and needs no
   edit; the play-server report probe (`main.rs:3039-3042`) compares against
   `mtg_engine::HASH_SCHEMA_VERSION` and will not move.

### 6.2 PROTOCOL — **expected UNMOVED at 35, but must be gate-executed**

`GameStateError` is reachable from none of `Command`/`GameEvent`/`ReplayLog` — the claim is written
into `error.rs:98-100` for `BlockedByPendingDecision` and applies identically here. `CombatState`
likewise does not appear in `protocol.rs`. **Do not assume this.** Execute
`cargo test -p mtg-engine --test core protocol_schema` and report the result. If it moves, stop and
report — an unexpected protocol move means the closure is wider than this plan believed.

---

## 7. Expected churn

### 7.1 Rust tests

**Discovered by execution, not by grep.** There are **344** `Command::DeclareAttackers` occurrences
across **66** files under `crates/engine/tests/` alone; the overwhelming majority are one declaration
per freshly-built state, and cross-*turn* declarations are unaffected (combat is nulled at
EndOfCombat entry). Only a **second declaration inside one combat** breaks. Procedure:

1. Land Stage 1, run `cargo test --workspace --no-fail-fast` to a file.
2. For each failure, read whether it is (i) a genuine two-declarations-in-one-combat test, (ii) a
   retry-after-rejection test (must NOT fail — if one does, §2.4's placement is wrong and that is a
   real defect in this batch), or (iii) Stage-3/4 seed movement.
3. A category-(i) test is **fixed by splitting the combat**, not by relaxing the assertion.

Highest-suspicion files by occurrence count: `pb_dx6_unflattened_payment_sites.rs` (21 — these are
mostly *independent* fixtures and retry-after-rejection, so expect them to stay green and treat any
failure there as a §2.4 defect), `mechanics_e_l/keywords.rs` (18), `rules/creature_triggers.rs` (11),
`pbn_subtype_filtered_triggers.rs` (9), `pb_dx1_lowered_intervening_if.rs` (9),
`pb_ef3_attack_trigger_targets.rs` (9).

### 7.2 Golden scripts and SR-9b

Golden scripts express the command as `"action": "declare_attackers"` (see
`test-data/generated-scripts/combat/015_declare_attackers_unblocked.json:108`). Procedure:
`grep -c '"action": "declare_attackers"' test-data/generated-scripts/**/*.json`, then **manually
inspect only the files with ≥2**, checking whether the repeats are in the same combat or in
different turns/phases. Cross-turn repeats are fine.

**SR-9b fingerprint risk**: the per-step fingerprint cross-validates the JSON-script regime against
the direct-`Command` regime. Because the new field is hashed and `build_initial_state` is
deterministic, a script that never re-declares produces an identical *sequence*; only the state hash
values move, and they move for **every** script identically (the version-sentinel byte). The runner
must confirm the SR-9b suite is green after the HASH re-pin and report if any script's per-step
fingerprint diverges *between* regimes (which would be a real defect, not a re-pin).

`crates/engine/src/testing/replay_harness.rs` also drives declarations; check it for any
auto-declare loop (its `:2438` comment already mentions Aurelia's unbounded extra combats).

### 7.3 Comments and docs that become wrong (conventions.md `:216-230` — never leave the aspirational version standing)

| File | Lines | Required edit |
|---|---|---|
| `crates/card-defs/src/defs/windbrisk_heights.rs` | 7-16 | **MANDATORY.** The "KNOWN RESIDUAL" says the raid count "is ASSIGNED, not accumulated" and gives *"attacking with three and then one drops the count to one"* as the example. PB-DX21 closes the **within-one-combat** half of that; the **extra-combat** half (attack with 3 in combat 1, then 1 in combat 2) remains, because `begin_combat` gives a fresh marker. Rewrite to say exactly that, cite PB-DX21 and the new seed `OOS-DX21-1`. This is a comment edit only — **0 completeness flips**. Run `tools/check-defs-fmt.sh`. |
| `crates/card-defs/src/defs/legions_landing.rs` | 66-71 | Optional companion note if the runner judges the same residual applies; do not change `completeness`. |
| `crates/simulator/src/heuristic_bot.rs` | 21-26, 45-49, 70-80, 81-130 | §3.1 — rewrite, do not merely delete code. |
| `crates/simulator/tests/local_game_playthrough.rs` | 127-180, 272-296, 405-407 | §3.2. |
| `tools/play-server/src/api.rs` | 298-306 | The word *irreversible* was aspirational and is now **true**. Add the PB-DX21 citation and note that the offer is now suppressed after a declaration (§2.7). |
| `crates/engine/src/rules/combat.rs` | 748-761 | The "overwritten (not accumulated) on multi-combat turns" comment stays true for *multi-combat* and becomes wrong for *multi-declaration*. Re-scope it. |
| `docs/audits/decision-point-audit.md` | §8.1 `OOS-M11-9` row | Mark **CLOSED** with the merge SHA. |
| `memory/primitives/seed-rerank-2026-08-02.md` | 873-907 | Banner the PB-DX21 row **✅ SHIPPED**, and correct line 899's "PREFER reading `combat.attackers`" in place with the §1.3 refutation — do not delete it. |
| `CLAUDE.md` | Current State `OOS-M11-9` mention | Move to the CLOSED list at collect. |
| `docs/mtg-engine-simulator.md` | wherever `RepeatKey`/`OOS-M11-9` is described | Check and update. |

### 7.4 Benchmarks

`full_turn_4p` (220–222 µs) and `priority_cycle_4p` (25.5–26.0 µs): the guard is one `Option` read
and one `bool` compare, executed once per `DeclareAttackers` command. Expect noise-level movement.
Run them and report; do not skip on the assumption.

### 7.5 Card-def coverage

**0 flips, pre-committed.** Prove it the PB-DX19 way — regenerate `tools/authoring-report.py` and
show the report body is byte-identical (modulo the git-sha/date stamp lines), **not** by an empty
`crates/card-defs` diff, because §7.3 mandates a comment edit there. Expect
**1,133/1,803 = 62.8%** unmoved.

---

## 8. Seeds to file

| ID | Content |
|---|---|
| `OOS-DX21-1` | **The extra-combat half of the raid-count clobber survives.** `combat.rs:759` still assigns; PB-DX21 makes re-assignment impossible *within* a combat but a second combat phase (CR 500.8/506.5) still overwrites `attackers_declared_this_turn`. Live on `windbrisk_heights` and `legions_landing`, both `Complete` and deck-legal, both `YouAttackedWithNOrMore(3)`. Closing it needs the field to become a per-turn accumulation with per-creature dedup (CR 508.6 "has attacked"), which is a different primitive. Record the wrong-way-round pin the successor must invert. |
| `OOS-DX21-2` | **`legal_actions.rs` still offers `DeclareBlockers` after `defenders_declared` contains the player** (`:923-954`) — the CR 509.1a twin of the offer hole §2.7 closes on the attacker side. Measured evidence already exists: PB-DX32's rejection-class table shows `AlreadyDeclaredBlockers (4)` in a 20-game fuzz run. Deliberately not widened into, per the PB-DX21 brief. |
| `OOS-DX21-3` | **CR 508.1i is still not honoured** — pre-existing `OOS-DP4-2`, re-confirmed: the engine determines and pays the CR 508.1h total inside one command with no mana-ability window. Cross-reference only; do not re-file if `OOS-DP4-2` is still open. |
| `OOS-DX21-4` | **An old serialized `CombatState` deserialises with `attackers_declared: false`** and permits one extra declaration mid-combat. Unavoidable with `#[serde(default)]` on a `bool`; recorded so a future rewind/replay batch knows. |
| `OOS-DX21-5+` | Whatever the runner finds. |

---

## 9. Verification checklist

- [ ] Stage-0 baseline re-confirmed at **4,388 / 0 / 5**, to a file, before any edit
- [ ] `cargo build --workspace` clean (proves no exhaustive `GameStateError` match exists)
- [ ] All 5 `CombatState` struct-literal sites updated; list re-derived by grep and reported
- [ ] Guard is before the `CombatState` init at `:69-72` and before any tax payment — verified by
      reading the diff, and by T6
- [ ] Marker set on the success path only, and **before** the CR 603.3d early return at `:811-813`
- [ ] Probes T1–T7 (and T8 or a written reason) all present, all **watched failing by an executed
      revert**, each revert's failure text recorded verbatim
- [ ] T4's **second** revert executed: the `!attackers.is_empty()` variant reddens T4 and only T4
- [ ] `cargo test -p mtg-engine --test core hash_schema` — HASH **72 → 73**, both digests pasted
      from the gate's own failure output, history row APPENDED (no shipped row edited)
- [ ] All 44 (re-derive!) `HASH_SCHEMA_VERSION` sentinels re-pinned, both spellings
- [ ] `cargo test -p mtg-engine --test core protocol_schema` — **PROTOCOL 35 unmoved**, executed
- [ ] `heuristic_bot.rs`'s `RepeatKey::DeclareAttackers` deleted **and its doc block rewritten**
- [ ] `local_game_playthrough.rs`'s `PolicyState` deleted **and step 3's comment rewritten**
- [ ] `test_s8_scripted_human_playthrough_is_clean_on_five_seeds` **green with no cap** — this is
      the closure proof for `OOS-M11-9`
- [ ] Every moved seeded pin classified and re-measured with the reason recorded **at the pin**
- [ ] Golden-script sweep run (`grep -c '"action": "declare_attackers"'`), files with ≥2 inspected
- [ ] `cargo clippy --workspace --all-targets -- -D warnings` clean
- [ ] `cargo fmt --check` **and** `tools/check-defs-fmt.sh` (SR-35 — the second is the only one that
      looks at the 1,803 defs, and §7.3 edits one)
- [ ] `cargo test --workspace --no-fail-fast` to a file, residual list empty
- [ ] Coverage **1,133/1,803 = 62.8%** unmoved, proven by byte-identical report regeneration
- [ ] Benches run and reported
- [ ] Seeds `OOS-DX21-1..N` filed in `docs/audits/decision-point-audit.md` §8.1
- [ ] `OOS-M11-9` marked CLOSED in the audit doc, the v3 queue row, and CLAUDE.md

---

## 10. Risks and edge cases

1. **§2.7 is outside the coordinator's stated three-bullet scope.** It is required by the mandated
   deletion in §3.2 (see §3.3). Escalate before executing if there is any doubt; do **not** silently
   weaken `local_game_playthrough.rs`'s rejection-is-fatal assertion.
2. **The brief's preferred implementation is refused.** §1.3 gives three CR-grounded reasons the
   guard cannot key on `combat.attackers`. The cost is a HASH bump the brief hoped to avoid
   (seed-rerank line 900-902 says "this batch does not otherwise need a bump — gate-compute rather
   than assume"). It does need one; it is gate-computed.
3. **The field's own hash bytes are covered by no automated fixture** (`canonical_fixture()` cannot
   populate `combat`). T7 is the only thing standing between this batch and the exact hole ENG-1's
   review Finding 1 found. Do not skip it, and do not let its revert be a stale-binary pass.
4. **CR 603.3d suspended-trigger path.** If a `WhenAttacks` trigger needs a target choice, the
   function returns early at `:811-813` after `mark_flush_resume_site`. The marker must already be
   set (§2.4). If it were not, a declaration that suspends would be re-declarable — the hardest
   variant to notice. Consider adding this as a T2 sub-case if `pb_dp8_trigger_target_choice.rs`
   provides a cheap fixture.
5. **`aurelia_the_warleader` untaps all your creatures**, so in her extra combat every creature is
   eligible again. T5 must not accidentally assert on tapped-ness as a proxy for the marker.
6. **The blockers guard must not be collaterally changed.** T8 (or the existing
   `combat.rs`-test at `combat/combat.rs:1701`) is the cheap insurance.
7. **`GameStateError` text reaches a client as a 422** (`api.rs:105`). The new variant carries a
   `PlayerId` and nothing else — Architecture Invariant 7 holds by construction, but a reviewer
   should re-check the rendered string.
8. **Deleting the bot cap without §2.7 does not livelock** — `local_game.rs:931-942` falls back to
   `PassPriority` on a rejected bot command and records a `RejectedCommand`. Verified in source.
   It does, however, raise the SR-38 rejection rate that `MAX_BOT_REJECTION_PER_MILLE_AT_GATE_CONFIG`
   (pinned 40 at the gate config, 30 at the binary) ratchets. If §2.7 ships, this pressure goes
   away; if it does not, expect T2.2 in `pb_dx32_fuzz_output.rs` to move and possibly breach.
9. **`tools/tui`** builds `DeclareAttackers` commands from `LegalAction`
   (`tools/tui/src/play/input.rs`, `panels/action_menu.rs`) and is covered by §2.7 with no TUI edit;
   confirm by `cargo build --workspace`.
10. **Never `| tail` a test run.** Capture to a file (2026-08-02 incident: a tail pipe hid a compile
    failure and faked a green run).
