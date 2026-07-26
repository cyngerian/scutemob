# Primitive Batch Plan: PB-DP1 — priority after cast / activate / special action goes to the ACTOR

<!-- last_updated: 2026-07-26 -->

**Generated**: 2026-07-26
**Task**: `scutemob-149`
**Branch**: `feat/pb-dp1-priority-after-castactivatespecial-action-goes-to-the`
**Primitive**: no new type, variant or field. This PB changes **which `PlayerId` is written
into the existing `Turn::priority_holder`** at the sites that follow a spell cast, an ability
activation, or a special action — from `state.turn.active_player` to the actor — and closes
the matching `players_passed` gaps (CR 117.4).
**CR rules**: 117.3a, 117.3b, **117.3c**, 117.3d, **117.4**, 116.2 (a/b/f/g/h/k), **116.3**,
601.2i, 602.2b, 605.3a/b, 606.1, 702.167a
**Cards affected**: **0** — no card def is touched. **Expected coverage flips: 0.** Confirmed:
this is a core-rules fix reachable with no card in the deck (audit §3.2). `docs/authoring-status.md`
must not move.
**Dependencies**: none.
**Deferred items from prior PBs**: none carried in. PB-RS4 (`scutemob-146`) is closed; the RS
queue is paused at RS5 and is untouched by this PB.

> **WIRE: PROTOCOL 27 / HASH 63 UNCHANGED.** No `Command` field, no `Effect` variant, no
> `GameEvent` variant, no struct/enum shape change. `Turn::priority_holder` is already
> `Option<PlayerId>` and already hashed (`state/hash.rs`). The sentinels in
> `crates/engine/tests/primitives/pb_os6_dfc_flip_conditions.rs:876/880`,
> `pb_os10_singleton_cleanup.rs:94/100`, `pbp_power_of_sacrificed_creature.rs:795` and the
> SR-17 gate in `crates/engine/tests/core/hash_schema.rs` **must all still read 27 / 63 at the
> end of this PB.** If you find yourself needing to re-pin either, **STOP and report** — that
> means the edit strayed outside the plan.

---

## 0. What was verified, and where the WIP file was wrong

Every one of the 34 `priority_holder =` sites in the WIP sweep was re-read in source. The
sweep is accurate. Four corrections / additions:

| # | WIP said | Truth (verified in source) |
|---|---|---|
| C1 | Group A: `engine.rs:1461` (craft) is a behaviour fix | `handle_activate_craft` gates `state.turn.active_player == player` at `engine.rs:1272-1278`. `Some(active)` ≡ `Some(player)` there. Still change it (correctness-by-construction + citation), but it is a **no-op**, not a flip. |
| C2 | Group A is 14 flips | Only **6** of the 14 can actually change behaviour: `casting.rs:4715`, `abilities.rs:1387`, `:1552`, `:1967`, `:2341`, `:8791`. The other 8 sit behind an `active_player != player` sorcery-speed gate (`abilities.rs:1643` forecast, `:2040` unearth, `:2428` embalm, `:2597` eternalize, `:2777` encore, `:8846` saddle, `:9095` scavenge, `engine.rs:1272` craft) and are identity writes. This is the single most important input to the fallout forecast. |
| C3 | Group D is 6 handlers | **8.** The WIP missed `handle_activate_loyalty_ability` (`engine.rs:2451-2661`) and `handle_level_up_class` (`engine.rs:2667-2816`). Both are activated abilities (CR 606.1, CR 716.2a → 602.2b), both reset `players_passed` (`:2654`, `:2814`) and **neither ever writes `priority_holder`**. They are the same defect shape as the six listed. Systematic-sweep gate (AC 5511) requires them. |
| C4 | Group D items "write priority_holder nowhere" | Three sub-shapes, not one. See §5 — `PlayLand` is already fully correct and needs only a comment fix; foretell/plot/suspend/companion are missing the **`players_passed` reset** (CR 117.4), not the priority grant; `turn_face_up`/loyalty/level-up are missing the **grant**, not the reset. Treating them uniformly would be wrong in both directions. |

**Group C is confirmed correct as filed** — see §6. All four claimed false positives are false
positives.

**PRESERVE gate re-verified**: `rules/mana.rs::handle_tap_for_mana` checks priority at `:47`,
touches `priority_holder` nowhere else and leaves `players_passed` untouched (`:617-618`).
**No code in `mana.rs` is changed by this PB.** Only two doc-comment lines there are corrected
(§7, optional step).

---

## 1. CR text (verified via mtg-rules MCP, 2026-07-26)

```
117.3.  Which player has priority is determined by the following rules:

117.3a  The active player receives priority at the beginning of most steps and phases,
        after any turn-based actions (such as drawing a card during the draw step; see
        rule 703) have been dealt with and abilities that trigger at the beginning of
        that phase or step have been put on the stack. No player receives priority
        during the untap step. Players usually don't get priority during the cleanup
        step (see rule 514.3).

117.3b  The active player receives priority after a spell or ability (other than a mana
        ability) resolves.

117.3c  If a player has priority when they cast a spell, activate an ability, or take a
        special action, that player receives priority afterward.

117.3d  If a player has priority and chooses not to take any actions, that player passes.
        If any mana is in that player's mana pool, they announce what mana is there. Then
        the next player in turn order receives priority.

117.4.  If all players pass in succession (that is, if all players pass without taking any
        actions in between passing), the spell or ability on top of the stack resolves or,
        if the stack is empty, the phase or step ends.

116.3.  If a player takes a special action, that player receives priority afterward.
        [116.3 HAS NO SUBRULES. "CR 116.3a/b/c/d" DO NOT EXIST in the current CR — they
         are the pre-renumber names of what is now 117.3a-d / 117.4.]

116.2a  Playing a land is a special action. ... A player can take this action any time they
        have priority and the stack is empty during a main phase of their turn.
116.2b  Turning a face-down creature face up is a special action. A player can take this
        action any time they have priority.
116.2f  A player who has a card with suspend in their hand may exile that card. This is a
        special action. A player can take this action any time they have priority, but only
        if they could begin to cast that card by putting it onto the stack.
116.2g  A player who has chosen a companion may pay {3} to put that card from outside the
        game into their hand. This is a special action. A player can take this action any
        time they have priority and the stack is empty during a main phase of their turn,
        but only if they haven't done so yet this game.
116.2h  A player who has a card with foretell in their hand may pay {2} and exile that card
        face down. This is a special action. A player may take this action any time they
        have priority during their turn.
116.2k  A player who has a card with plot in their hand may exile that card. This is a
        special action. A player can take this action any time they have priority during
        their own turn while the stack is empty.

601.2i  Once the steps described in 601.2a-h are completed, effects that modify the
        characteristics of the spell as it's cast are applied, then the spell becomes cast.
        Any abilities that trigger when a spell is cast or put onto the stack trigger at
        this time. If the spell's controller had priority before casting it, they get
        priority.

602.2   ... [only 602.2a and 602.2b exist. CR 602.2e DOES NOT EXIST.]
602.2b  The remainder of the process for activating an ability is identical to the process
        for casting a spell listed in rules 601.2b-i. Those rules apply to activating an
        ability just as they apply to casting a spell. An activated ability's analog to a
        spell's mana cost (as referenced in rule 601.2f) is its activation cost.

605.3a  A player may activate an activated mana ability whenever they have priority,
        whenever they are casting a spell or activating an ability that requires a mana
        payment, or whenever a rule or effect asks for a mana payment, even if it's in the
        middle of casting or resolving a spell or activating or resolving an ability.
605.3b  An activated mana ability doesn't go on the stack, so it can't be targeted,
        countered, or otherwise responded to. Rather, it resolves immediately after it is
        activated.
```

**The two-part rule this PB implements**, and it is two parts, not one:

1. **CR 117.3c / 601.2i / 602.2b / 116.3** — the ACTOR receives priority afterward.
2. **CR 117.4** — an action taken between passes breaks the succession, so `players_passed`
   must be emptied. Every Group-A site already does (2); several Group-D sites do not.

---

## 2. Order of work (non-negotiable)

Probe tests are written and **verified failing** before any engine line moves (AC 5512).
Steps are numbered so they can be reverted individually.

```
Step 1   probe test file, RED
Step 2   casting.rs   (1 site)                      Group A
Step 3   abilities.rs (12 sites)                    Group A
Step 4   engine.rs craft (1 site)                   Group A (no-op)
Step 5   Group B ruling applied (3 comment fixes)   engine.rs
Step 6   Group D-a  PlayLand comments               lands.rs        (no-op)
Step 7   Group D-b  players_passed resets (x4)      foretell/plot/suspend/commander
Step 8   Group D-c  priority grants (x3)            engine.rs        SEPARATELY REVERTABLE
Step 9   Group C comment/citation fixes             resolution.rs, combat.rs, priority.rs
Step 10  mana.rs comment-only citation fix          OPTIONAL, comment-only
Step 11  probe tests GREEN
Step 12  full engine test suite; triage per §9
Step 13  golden scripts; triage per §9
Step 14  simulator / LocalGame; §10
Step 15  gates + wire-sentinel confirmation
Step 16  audit doc + wip + seeds
```

---

## 3. Step 1 — probe tests (write first, run RED)

**File (new)**: `crates/engine/tests/primitives/pb_dp1_actor_priority.rs`
**Registration**: add `mod pb_dp1_actor_priority;` to
`crates/engine/tests/primitives/main.rs`, alphabetically **between line 21
(`mod pb_ac9_wheel_and_misc;`) and line 22 (`mod pb_ef10_sacrifice_driven_amounts;`)**.

> SR-9a gate: never create a top-level `crates/engine/tests/*.rs`; a file in a group dir
> with no `mod` line is silently uncompiled and `tests/no_stray_test_binaries.rs` fails.

Run with:
```
~/.cargo/bin/cargo test -p mtg-engine --test primitives pb_dp1_actor_priority
```

Each test carries a `///` doc comment naming its CR rule (conventions.md "Tests cite their
rules source"). Nine probes, minimum:

| # | name | CR cited | assertion | must FAIL before Step 2-8 |
|---|---|---|---|---|
| P1 | `test_dp1_non_active_player_casting_instant_retains_priority` | 117.3c, 601.2i | 4-player, active `p1`, `p1` passes → `p2` holds priority; `p2` casts an instant; assert `state.turn().priority_holder == Some(p2)` | YES (currently `Some(p1)`) |
| P2 | `test_dp1_actor_can_respond_to_own_spell` | 117.3c | continuation of P1: `p2` immediately casts a second instant **without any intervening `PassPriority`**; assert both objects on the stack and `priority_holder == Some(p2)` | YES (2nd cast returns `NotPriorityHolder`) |
| P3 | `test_dp1_actor_can_respond_to_own_activated_ability` | 117.3c, 602.2b | `p2` (non-active) activates an instant-speed activated ability, then activates/casts again with no pass; assert `priority_holder == Some(p2)` after the first activation | YES |
| P4 | `test_dp1_non_active_player_cycling_retains_priority` | 702.29a, 602.2b, 117.3c | `p2` cycles a card during `p1`'s turn; assert `priority_holder == Some(p2)` | YES (`abilities.rs:1552`) |
| P5 | `test_dp1_non_active_player_crewing_retains_priority` | 702.122a, 117.3c | `p2` crews a vehicle during `p1`'s turn (crew has no sorcery-speed gate — verified, no `active_player` check in `handle_crew_vehicle`); assert `priority_holder == Some(p2)` | YES (`abilities.rs:8791`) |
| P6 | `test_dp1_active_player_casting_still_holds_priority` | 117.3c | control probe: `p1` (active) casts; assert `priority_holder == Some(p1)`. Guards against a mis-targeted edit that writes some *other* player. | NO — green both sides, by design |
| P7 | `test_dp1_mana_ability_does_not_reset_players_passed` | 605.3a/b, 117.3b parenthetical | `p1` passes (so `players_passed` contains `p1`), `p2` taps a land for mana; assert `players_passed` still contains `p1` **and** `priority_holder == Some(p2)`. **This is the PRESERVE regression pin.** | NO — green both sides; it exists to fail loudly if someone "tidies" `mana.rs` |
| P8 | `test_dp1_foretell_resets_players_passed` | 116.2h, 116.3, 117.4 | `p1` is active and has already... — see note below | YES (`foretell.rs`) |
| P9 | `test_dp1_special_action_actor_holds_priority_after_turn_face_up` | 116.2b, 116.3 | `p2` turns a face-down permanent face up while holding priority; assert `priority_holder == Some(p2)` and `players_passed` is empty | Partially — the grant is currently absent but incidentally correct; assert `players_passed.is_empty()` (already true, `engine.rs:1621`) **and** `priority_holder == Some(p2)`; the value-add is that it pins the invariant against Step 8 |

> **P8 construction note.** Foretell is own-turn-only (CR 116.2h; engine gate at
> `foretell.rs`), so the actor is the active player. The observable defect is the
> **`players_passed` non-reset**, not the holder. Build it as: `p1` active in their own main
> phase, `p1` passes → `p2` holds priority, `p2` passes, `p3` passes … then get priority back
> to `p1` (via a stack object resolving, which grants AP priority per CR 117.3b and resets
> `players_passed`) — that path does not discriminate. **Simpler and valid**: construct the
> state directly with `GameStateBuilder`, set `priority_holder = Some(p1)` and seed
> `players_passed` with `{p2, p3}` through the builder, issue `Command::ForetellCard`, then
> assert `state.turn().players_passed.is_empty()`. If the builder cannot seed
> `players_passed`, reach the same state by having `p2` and `p3` pass first while `p1` holds
> a stack object, then have `p1` foretell. **Do not fake it by mutating `GameState` directly
> — it is sealed `pub(crate)` (SR-3).**

**Additional probes if cheap** (not required, but they close the §5 dispositions):
`test_dp1_plot_resets_players_passed`, `test_dp1_suspend_resets_players_passed`,
`test_dp1_loyalty_activation_grants_actor_priority` (CR 606.1 / 602.2b / 117.3c).

**Gate before proceeding**: capture the RED output. A probe that passes pre-fix is a
test-validity bug and is a **fix-phase HIGH** (conventions.md), not a LOW — rewrite it or
escalate. In particular, P1/P2/P3 passing pre-fix means the setup never actually put a
non-active player on priority.

---

## 4. Group A — the actor receives priority (14 sites)

The mechanical transform at every site is identical:

```rust
// BEFORE
    let active = state.turn.active_player;
    state.turn.priority_holder = Some(active);
    ...
    events.push(GameEvent::PriorityGiven { player: active });

// AFTER
    state.turn.priority_holder = Some(player);
    ...
    events.push(GameEvent::PriorityGiven { player });
```

**Do not forget the companion `PriorityGiven` event.** `grep priority_holder` does not find
it; it is a separate line 10-20 lines below the assignment in every `abilities.rs` handler.
Leaving it on `active` produces an event that lies about state (Architecture Invariant 4)
and will not fail to compile.

**`let active` is dead after the edit in every one of these handlers** (verified: `active`
has exactly two uses per handler — the assignment and the `PriorityGiven` push). Delete the
binding; leaving it produces an `unused_variables` warning and `clippy -D warnings` fails.

### Step 2 — `crates/engine/src/rules/casting.rs` (1 site)

| line | fn | actor expression | in scope? | flips? |
|---|---|---|---|---|
| **4715** | `handle_cast_spell` (`:54`) | **`player`** — the `player: PlayerId` parameter at `:56` | yes; and `casting.rs:214` proves `priority_holder == Some(player)` on entry, so CR 601.2i's antecedent ("if the spell's controller had priority before casting it") is satisfied by construction | **YES — the headline flip** |

Edits:
- `:4712` comment. **Currently a misquote**: `// CR 601.2i: "Then the active player receives priority."` — CR 601.2i says no such thing. Replace with:
  ```rust
  // CR 601.2i: "If the spell's controller had priority before casting it, they get
  // priority." The `:214` guard proves `player` held priority on entry, so the
  // antecedent holds and `player` — not necessarily the active player — gets it back
  // (CR 117.3c).
  // CR 117.4: an action was taken between passes, so the pass-round restarts.
  ```
- `:4715` → `state.turn.priority_holder = Some(player);`
- `:4907-4909` `events.push(GameEvent::PriorityGiven { player: state.turn.active_player })` → `events.push(GameEvent::PriorityGiven { player });` (note: this push is ~190 lines below the assignment — do not miss it).
- Leave `:4714` `players_passed = OrdSet::new()` as-is (already CR 117.4-correct).

### Step 3 — `crates/engine/src/rules/abilities.rs` (12 sites)

Every one of these has a `player: PlayerId` second parameter and a
`priority_holder != Some(player)` guard at the top of the handler — **verified, all 12** — so
`Some(player)` is always a live, priority-holding seat. Actor expression is `player` at all
12; do not use `stack_obj.controller`, `obj.controller`, or `state.turn.active_player`.

| assign | event | fn (start line) | entry priority guard | CR class of the ability | flips? |
|---|---|---|---|---|---|
| **1387** | 1404 | `handle_activate_ability` (`:130`) | `:144` | generic activated ability, CR 602.2b | **YES** (sorcery-speed abilities are separately gated at `:240-242`; instant-speed ones flip) |
| **1552** | 1558 | `handle_cycle_card` (`:1417`) | `:1423` | Cycling, CR 702.29a — instant speed, **no** AP gate | **YES** |
| **1753** | 1759 | `handle_activate_forecast` (`:1611`) | `:1619` | Forecast, CR 702.57a/b | no — AP-gated at `:1643` |
| **1967** | 1982 | `handle_activate_bloodrush` (`:1778`) | `:1785` | Bloodrush, CR 702.94a — instant speed, **no** AP gate | **YES** (narrow: needs an attacking creature to target) |
| **2102** | 2108 | `handle_unearth_card` (`:1994`) | `:2000` | Unearth, CR 702.83a | no — AP-gated at `:2040` |
| **2341** | 2347 | `handle_ninjutsu` (`:2145`) | `:2152` | Ninjutsu, CR 702.49a — **no** AP gate | **YES** in principle (needs an unblocked attacker you control, so effectively AP) |
| **2504** | 2510 | `handle_embalm_card` (`:2382`) | `:2388` | Embalm, CR 702.87a | no — AP-gated at `:2428` |
| **2681** | 2687 | `handle_eternalize_card` (`:2551`) | `:2557` | Eternalize, CR 702.129a | no — AP-gated at `:2597` |
| **2857** | 2863 | `handle_encore_card` (`:2731`) | `:2737` | Encore | no — AP-gated at `:2777` |
| **8791** | 8797 | `handle_crew_vehicle` (`:8611`) | `:8625` | Crew, CR 702.122a — instant speed, **no** AP gate | **YES** |
| **9000** | 9006 | `handle_saddle_mount` (`:8819`) | `:8829` | Saddle, CR 702.171a | no — AP-gated at `:8846` |
| **9202** | 9208 | `handle_scavenge_card` (`:9048`) | `:9055` | Scavenge, CR 702.97a | no — AP-gated at `:9095` |

**Citation replacements in this file (the bogus `CR 602.2e`, twelve occurrences):**

| line | current text | replacement |
|---|---|---|
| **128** (fn doc for `handle_activate_ability`) | `/// After activation, the active player receives priority (CR 116.3b).` | `/// CR 602.2b -> 601.2i: after activation, the player who activated the ability receives priority (CR 117.3c). "CR 116.3b" does not exist; the priority rules live in CR 117.3.` |
| **1384** | `// CR 602.2e: After activating, the active player receives priority.` | `// CR 602.2b -> 601.2i / CR 117.3c: the activating player receives priority afterward.` + `// CR 117.4: reset the pass-round; an action was taken between passes.` |
| **1549** | `// 8. Reset priority (CR 602.2e): active player gets priority.` | `// 8. CR 602.2b -> 601.2i / CR 117.3c: the activating player gets priority (CR 117.4: reset the pass-round).` |
| **1750** | `// 12. Reset priority (CR 602.2e): active player gets priority.` | same shape as 1549 (renumber the step prefix) |
| **1964** | `// 9. Reset priority (CR 602.2e): active player gets priority.` | same |
| **2099** | `// 9. Reset priority (CR 602.2e): active player gets priority.` | same |
| **2338** | `// 13. Reset priority (CR 602.2e): active player gets priority.` | same |
| **2501** | `// 11. Reset priority (CR 602.2e): active player gets priority.` | same |
| **2678** | `// 11. Reset priority (CR 602.2e): active player gets priority.` | same |
| **2854** | `// 11. Reset priority (CR 602.2e): active player gets priority.` | same |
| **8788** | `// CR 602.2e / CR 116.3b: After activating, the active player receives priority.` | `// CR 602.2b -> 601.2i / CR 117.3c: the activating player receives priority afterward. (Neither "CR 602.2e" nor "CR 116.3b" exists.)` |
| **8997** | `// CR 602.2e / CR 116.3b: After activating, the active player receives priority.` | same as 8788 |
| **9199** | `// 12. Reset priority (CR 602.2e): active player gets priority.` | same shape as 1549 |

For the eight AP-gated no-op sites, append to the comment: `// (This handler is AP-gated at
:NNNN, so this is an identity write today; it is written as \`player\` so the site stays
correct if the gate is ever relaxed.)` — naming the actual gate line.

### Step 4 — `crates/engine/src/rules/engine.rs` craft (1 site)

| line | fn | actor | in scope? | flips? |
|---|---|---|---|---|
| **1461** | `handle_activate_craft` (`:1240`) | **`player`** (`:1242`) | yes | **NO** — `:1272-1278` rejects unless `state.turn.active_player == player` |

Edits:
- `:1458` comment → `// CR 702.167a: craft is an activated ability (CR 602.2b -> 601.2i), so the activating player receives priority (CR 117.3c). Identity write today: :1272 already requires player == active_player ("activate only as a sorcery"). CR 117.4: reset the pass-round.`
- `:1459` keep. `:1460` delete the `let active` binding. `:1461` → `Some(player)`.
- No `PriorityGiven` push exists in this handler; do **not** add one (adding an event is a wire-observable change in event streams and is out of scope).

**Known out-of-scope observations here (record, do not fix):** `handle_activate_craft` has no
`priority_holder == Some(player)` guard at all, and it resolves the craft immediately instead
of putting an activated ability on the stack (CR 702.167a says it uses the stack). Both are
pre-existing and belong to DP-21/DP-23's class. → seed **OOS-DP1-2**.

---

## 5. Group D — the missing sites, disposition per item

AC 5511 requires an explicit disposition for each. There are **eight** handlers, not six
(see §0 C3), in three sub-shapes.

### D-a. Already fully correct — comment fix only

| handler | file:line | why already correct |
|---|---|---|
| `handle_play_land` | `rules/lands.rs:25` | `:31-36` rejects unless `priority_holder == Some(player)`, so `priority_holder` is **already** `Some(player)` on exit — CR 116.3 satisfied by construction. `:419` resets `players_passed` — CR 117.4 satisfied. Zero behaviour change needed. |

**Disposition: IN SCOPE, comment-only (Step 6).** Two comments describe the behaviour
incorrectly and cite the wrong rule:
- `lands.rs:23-24`: `/// \`players_passed\` is reset (a game action occurred), but the active player /// retains priority.` → `/// \`players_passed\` is reset (CR 117.4 — an action was taken between passes) and the /// **acting** player retains priority (CR 116.2a / 116.3). The \`:31\` guard proves the /// actor already held it, so no write is needed here.`
- `lands.rs:417-418`: `// 11. Reset players_passed — a game action occurred, so the priority round // starts fresh. The active player retains priority (CR 117.3b).` → same correction; **CR 117.3b is the wrong rule** (it governs priority after a *resolution*, not after a special action). Cite CR 117.4 for the reset and CR 116.3 for the retention.

### D-b. Missing the CR 117.4 `players_passed` reset — real, observable

| handler | file | has priority guard | resets `players_passed`? | writes `priority_holder`? |
|---|---|---|---|---|
| `handle_foretell_card` | `rules/foretell.rs` (guard `:48`) | yes | **NO** | no (already `Some(player)`) |
| `handle_plot_card` | `rules/plot.rs` (guard `:55`) | yes | **NO** | no (already `Some(player)`) |
| `handle_suspend_card` | `rules/suspend.rs` (guard `:59`) | yes | **NO** | no (already `Some(player)`) |
| `handle_bring_companion` | `rules/commander.rs:914` | **no** guard; AP + main-phase + empty-stack gated at `:941-958` | **NO** | no |

Observable defect: `players_passed` accumulates across a special action. Sequence
`p1 pass, p2 pass, p3 foretells, p3 pass, p4 pass` completes the round without `p1` and `p2`
getting priority back — a direct CR 117.4 violation ("without taking any actions in between").

**Disposition: IN SCOPE (Step 7).** At the end of each handler, immediately before the final
`Ok(events)`, add:

```rust
    // CR 116.3: the acting player receives priority afterward. The `:NN` guard already
    // proves `priority_holder == Some(player)`, so no write is needed — but CR 117.4
    // requires the pass-round to restart, because an action was taken between passes.
    state.turn.players_passed = imbl::OrdSet::new();
```

(For `commander.rs::handle_bring_companion` the guard sentence is `the CR 702.139a sorcery-speed
gate at :941-958 forces player == active_player`, and note that no priority guard exists —
that is DP-21's class, seeded as OOS-DP1-2, not fixed here.)

Import note: `foretell.rs`, `plot.rs` and `suspend.rs` may not already import `imbl::OrdSet`.
Use the fully-qualified `imbl::OrdSet::new()` (the pattern used at `engine.rs:755`) rather
than adding a `use` — smaller diff, no unused-import risk.

### D-c. Missing the CR 116.3 / 117.3c priority grant

| handler | file:line | resets `players_passed`? | writes `priority_holder`? | CR class |
|---|---|---|---|---|
| `handle_turn_face_up` | `engine.rs:1469`, tail `:1620-1621` | **yes** (`:1621` `.clear()`) | **NO** | special action, CR 116.2b |
| `handle_activate_loyalty_ability` | `engine.rs:2451`, tail `:2653-2654` | **yes** (`:2654`) | **NO** | activated ability, CR 606.1 → 602.2b |
| `handle_level_up_class` | `engine.rs:2667`, tail `:2813-2814` | **yes** (`:2814`) | **NO** | activated ability, CR 716.2a → 602.2b |

All three are *incidentally* CR-correct whenever the actor held priority (the holder simply
stays put). None of the three has an entry priority guard, so the explicit write is not a
strict identity — this is why Step 8 is **separately revertable**.

`engine.rs:1620` also carries an **aspirationally-wrong comment** —
`// CR 116.2b: Special action; reset priority to active player.` — describing a write that
does not exist and prescribing the wrong recipient. conventions.md forbids leaving it
standing.

**Disposition: IN SCOPE (Step 8), as an independently revertable step.**

At `engine.rs:1620-1621` replace with:
```rust
    // CR 116.2b / CR 116.3: turning a face-down permanent face up is a special action;
    // the player who took it receives priority afterward.
    // CR 117.4: an action was taken between passes, so the pass-round restarts.
    state.turn.players_passed = imbl::OrdSet::new();
    state.turn.priority_holder = Some(player);
```
(Use `= imbl::OrdSet::new()` for consistency with every other site rather than `.clear()`;
the two are equivalent on an `imbl::OrdSet`.)

At `engine.rs:2653-2654` replace with:
```rust
    // CR 606.1 -> 602.2b -> 601.2i / CR 117.3c: activating a loyalty ability is
    // activating an ability, so the activating player receives priority afterward.
    // CR 117.4: reset the pass-round.
    state.turn.players_passed = imbl::OrdSet::new();
    state.turn.priority_holder = Some(player);
```

At `engine.rs:2813-2814` replace with the same shape, citing `CR 716.2a -> 602.2b -> 601.2i /
CR 117.3c` and `CR 117.4`.

**Escape hatch for Step 8.** If the full-suite run in Step 12 produces failures that trace
specifically to one of these three writes handing priority to a seat that did not previously
hold it (symptom: a subsequent `PassPriority` by a *different* seat now errors
`NotPriorityHolder`, in a test that never intended the actor to hold priority) — revert **only
that one write**, keep the comment fix, and file it as a seed. Do not weaken the Group A
work to accommodate it. Report the revert in the implement commit message.

**Why not add the missing entry priority guards?** Adding
`if state.turn.priority_holder != Some(player) { return Err(NotPriorityHolder) }` to
`handle_turn_face_up`, `handle_activate_craft`, `handle_activate_loyalty_ability`,
`handle_level_up_class` and `handle_bring_companion` is a **new rejection path**, i.e. new
enforcement surface, not a redirection of an existing write. That is DP-21's scope
("Loyalty abilities: no priority check, no active-player check, no split-second check") and
it is explicitly out of PB-DP1. → seed **OOS-DP1-2**.

### D-d. Swept and explicitly NOT in scope

| handler | why not |
|---|---|
| `handle_transform` (`engine.rs:1086`) | `Command::Transform` is a harness/effect-driven flip (CR 712.18), not a CR player action. Correctly touches no priority. |
| `handle_venture_into_dungeon` (`:2162`), `handle_ring_tempts_you` (`:2255`), `handle_choose_dungeon_room` (`:526`) | engine-internal/script-driver commands standing in for resolution-time effects, not CR 117.3c actions. |
| `ChooseDredge`, `ChooseMiracle`, `OrderBlockers`, `OrderReplacements`, `ReturnCommanderToCommandZone`, `LeaveCommanderInZone`, `TakeMulligan`, `KeepHand` | replacement-effect / resolution-time / pregame choices. CR 117.3 grants no priority for any of them. |
| `PayEcho`, `PayCumulativeUpkeep`, `PayRecover` | Group B — see §5.5. |
| `DeclareAttackers`, `DeclareBlockers` | turn-based actions — Group C, see §6. |
| `TapForMana` | **PRESERVE.** Not touched. |

### 5.5 Group B ruling — echo / cumulative upkeep / recover

**Sites**: `engine.rs:757` (`handle_pay_echo`, fn `:590`), `:958`
(`handle_pay_cumulative_upkeep`, fn `:768`), `:1072` (`handle_pay_recover`, fn `:997`).

**Verified facts** (all three read in source):
1. **None of the three has a priority guard.** They are dispatched off a `pending_*` vector
   keyed by `(player, object)`; the only validation is that a matching pending entry exists.
2. All three are *resolution-time* choices belonging to the echo / cumulative-upkeep /
   recover **triggered ability**, which per CR 702.30a / 702.24a / 702.59a is made while that
   ability resolves. **No player holds priority at that moment.** CR 117.3c's antecedent is
   therefore false, exactly as for the cipher and suspend free-casts in Group C.
3. The trigger's own resolution already grants AP priority at `resolution.rs:7744`
   (CR 117.3b). These handlers run *later*, out of band, because the pause DP-11 describes
   was never implemented.

**RULING: leave the behaviour exactly as it is (AP keeps priority, `players_passed` resets).
Fix only the comment. Change zero lines of logic at `:755/757`, `:956/958`, `:1070/1072`.**

Justification:
- CR gives no rule that assigns priority to the payer, so "actor-by-analogy" would be
  inventing one. The task's constraint is that the resulting comment must not cite a rule
  that does not say what the comment claims — inventing an actor rule here would violate that
  in the opposite direction.
- The AP-priority write is a *bodge standing in for the missing pause*, and the missing pause
  is PB-DP4's declared scope (`decision-point-audit.md` §8). Changing the bodge without the
  pause would move the deviation, not close it, and would risk the "silently reassign priority
  mid-round" regression class that PB-DP1 is trying to *remove*.
- Zero test/script fallout, which keeps the Group A signal clean in Step 12.

**Comment replacement (Step 5), identical shape at all three sites** — `:754`, `:955`,
`:1069` currently read `// Grant priority to the active player.`:

```rust
    // CR 702.30a: paying (or declining) echo is a choice made while the echo triggered
    // ability RESOLVES. No player holds priority at that moment, so CR 117.3c does not
    // apply and there is no actor to hand priority to.
    //
    // The engine has no pause at that point (see DP-11 in docs/audits/decision-point-audit.md
    // -> PB-DP4): `Command::PayEcho` is accepted out of band, whenever it arrives. This
    // block re-establishes a clean CR 117.3b priority round (active player, fresh pass set)
    // so the out-of-band command does not leave the round half-passed. It is deliberately
    // NOT the CR 117.3c actor rule, and PB-DP1 left it alone on purpose.
```
Use `CR 702.24a` for `:955` (cumulative upkeep) and `CR 702.59a` for `:1069` (recover).

Seed: **OOS-DP1-1** — "three cost-payment handlers reassign priority to the AP out of band;
correct fix is the DP-11 pause, tracked by PB-DP4."

---

## 6. Group C — confirmed correct as-is, DO NOT change behaviour

All four claims in the WIP file are **confirmed**. Read and verified line by line.

| site | fn | verdict | evidence |
|---|---|---|---|
| `engine.rs:1759` | `enter_step`, cleanup-SBA round (`:1731-1762`) | **correct — audit false positive** | AP receives priority at the start of a step after TBAs and trigger-flush. CR 117.3a verbatim. |
| `engine.rs:1805` | `enter_step`, `has_priority()` branch (`:1765-1806`) | **correct — audit false positive** | same; and `:1797-1801` already handles the dead-AP case by walking APNAP. |
| `combat.rs:1373` | `handle_declare_blockers` | **correct — audit false positive** | Declaring blockers is a **turn-based action** (CR 509.1); priority after a TBA goes to the AP (CR 117.3a). The `player` parameter here is the *defending* player, so `Some(player)` would be actively wrong. |
| `resolution.rs:5175` | cipher free-cast inside `resolve_top_of_stack` | **correct — comment only** | The copy is cast *during resolution* of the cipher trigger (`:5125-5184`). No player holds priority mid-resolution, so CR 601.2i's "if the spell's controller had priority before casting it" is **false** and the controller does not get priority. AP is right, via CR 117.3b once the trigger finishes (`:7744`). |
| `combat.rs:680` | `handle_declare_attackers` | **correct as-is** | already `Some(player)`, and `:46` rejects unless `player == active_player` (CR 508.1). |
| `resolution.rs:114` | fizzle | correct | CR 117.3b |
| `resolution.rs:7744` | after resolution | correct | CR 117.3b |
| `resolution.rs:7983` | after countering | correct | CR 117.3b |
| `resolution.rs:5835` | suspend free-cast during resolution | correct | `players_passed` reset only, no holder write; same reasoning as cipher |
| `turn_structure.rs:103`, `:140` | `advance_step` / `advance_turn` | correct | sets `None` |
| `engine.rs:1636`, `:1641` | `handle_pass_priority` | correct | CR 117.3d / 117.4 |
| `engine.rs:1812`, `:1815`, `:1883`, `:1893` | `handle_concede` | correct | priority reassignment on player loss |

### Step 9 — Group C citation fixes (comments only, zero logic)

The codebase systematically uses the **pre-renumber** rule numbers `116.3a/b/c/d`. In the
current CR those are `117.3a/b/c/d` and `117.4`; CR 116.3 has no subrules. In-engine
occurrences to correct:

| file:line | current | replacement |
|---|---|---|
| `rules/resolution.rs:8` (module doc) | `//! After resolution: priority resets to the active player (CR 116.3b).` | `//! After resolution: priority resets to the active player (CR 117.3b).` |
| `rules/resolution.rs:35` | `/// After resolution, the active player receives priority (CR 116.3b).` | `... (CR 117.3b).` |
| `rules/resolution.rs:5172` | `// CR 116.3b: Casting a spell resets priority (all players must pass again).` | `// CR 117.4: an action was taken between passes, so the pass-round restarts. // CR 601.2i: the cipher copy is cast DURING resolution — its controller did not have // priority before casting it, so they do NOT get priority. The active player gets it // when the trigger finishes resolving (CR 117.3b, resolution.rs:7744).` |
| `rules/resolution.rs:5833-5834` | `// CR 116.3b: Casting a spell resets priority. All players must // pass again before the newly-cast suspend spell resolves.` | same shape, referencing the suspend free-cast (CR 702.62a) instead of cipher |
| `rules/resolution.rs:7741` | `// CR 116.3b: After resolution (and trigger flushing), the active player receives priority.` | `// CR 117.3b: ...` |
| `rules/priority.rs:14` (fn doc for `pass_priority`) | `/// CR 116.3d: "If all players pass in succession ..."` — the quoted text is CR **117.4**, not 117.3d | `/// CR 117.4: "If all players pass in succession (that is, if all players pass without /// taking any actions in between passing), the spell or ability on top of the stack /// resolves or, if the stack is empty, the phase or step ends." /// CR 117.3d: the passing player announces floating mana, then the next player in turn /// order receives priority.` |
| `rules/priority.rs:50` (fn doc for `next_priority_player`) | `/// CR 116.3: "Which player has priority is determined by the following rules:"` | `/// CR 117.3: "Which player has priority is determined by the following rules:" /// CR 117.3d: the next player in turn order receives priority.` |
| `rules/combat.rs:1370-1371` | `// Grant priority to the active player so players can respond to triggers // (including Flanking triggers) before combat damage is dealt.` | prepend `// CR 509.1: declaring blockers is a turn-based action; CR 117.3a gives the ACTIVE // player priority after it — not the defending player who issued the command.` (keeps the existing rationale sentence) |
| `rules/combat.rs:678` | `// Grant priority to the active player (combat actions reset priority).` | `// CR 508.1 / CR 117.3a: declaring attackers is a turn-based action; the declaring // player is the active player (enforced at :46), so \`Some(player)\` is the active // player here. CR 117.4: reset the pass-round.` |

**Out of scope for citation fixes** (record in the seed, do not touch): ~60 `"note"` strings
in `test-data/generated-scripts/**/*.json`, the `cr_sections_tested` array in
`baseline/001_priority_pass_empty_stack.json:7`, `docs/mtg-engine-milestone-reviews.md:326-327`,
and seven `memory/abilities/ability-*.md` planning records. These are historical documents and
inert data; rewriting them inflates the diff for zero correctness value and would swamp the
review. → seed **OOS-DP1-3**.

---

## 7. Step 10 — `mana.rs` comment-only correction (OPTIONAL, no code)

`rules/mana.rs:35-36` and `:617-618` describe the PRESERVE behaviour correctly but
mis-classify a mana ability as a *special action* and cite `CR 605.5`:

- `:35-36`: `/// Per CR 605.5, activating a mana ability is a special action. The player /// retains priority and \`players_passed\` is not reset.`
- `:617-618`: `// 11. Player retains priority. players_passed is unchanged. //    (CR 605.5: mana abilities are special actions; they do not reset priority.)`

A mana ability is an **activated ability** (CR 605.1a), not one of CR 116.2's twelve special
actions. Correct replacement:

```rust
    // 11. Player retains priority; `players_passed` is unchanged.
    //     CR 605.3b: a mana ability doesn't use the stack and resolves immediately.
    //     CR 117.3c: the activating player receives priority afterward — and the `:47`
    //     guard proves they already held it, so this is a no-op by construction.
    //     CR 117.3b's parenthetical ("other than a mana ability") is why a mana ability
    //     does not hand priority back to the active player the way a resolution does.
    //     The `players_passed` non-reset is a deliberate, long-standing engine choice
    //     (it keeps floating mana from restarting the pass-round); PB-DP1 preserves it
    //     verbatim and pins it with `test_dp1_mana_ability_does_not_reset_players_passed`.
```

**Zero code lines change in `mana.rs`.** If the runner has any doubt, skip Step 10 entirely
and fold it into OOS-DP1-3 — the PRESERVE constraint outranks the comment tidy.

---

## 8. Exhaustive-match sites

**None.** No enum gains a variant and no struct gains a field, so there are no exhaustive
matches to extend. Specifically **not** affected:
`state/hash.rs` (`Turn::priority_holder` already hashed; `GameEvent::PriorityGiven` arm at
`:4172` unchanged), `tools/replay-viewer/src/view_model.rs` (`:215` reads `priority_holder`
generically), `tools/tui/src/play/app.rs:240` and `panels/phase_bar.rs:12` (same),
`crates/simulator/src/legal_actions.rs:193` and `local_game.rs:310` (both derive the acting
seat *from* `priority_holder`, so they follow the fix automatically).

Still run `~/.cargo/bin/cargo build --workspace` after the implement phase — that is the
standing gate, and it is the only thing that proves the tools crates still compile.

---

## 9. Fallout forecast and the triage rule

### 9.1 What can break, and why

The blast radius is narrower than "wide": only **6 of 14** Group-A sites can flip behaviour
(§0 C2), and the flip only manifests when a **non-active player** casts or activates. Most
tests and scripts drive everything from `p1` as the active player and are unaffected.

Two failure modes:

- **F-a. `pass_all(&[p1, p2, ...])` panics with `NotPriorityHolder`.** ~150 test modules
  define a local `pass_all(state, players)` that iterates a fixed list and `panic!`s on
  error (representative: `crates/engine/tests/mechanics_a_d/cycling.rs:40-50`). If a test
  has a non-active player act and then calls `pass_all` starting at `p1`, it now panics.
  **This is the dominant mode.**
- **F-b. A direct assertion moves.** There are only **31** `assert*(...priority_holder...)`
  sites across the whole `crates/engine/tests` tree — small and hand-checkable. Enumerated:
  `core/six_player.rs:74,81,85,89,93,97`; `core/priority.rs:19,31,38,42,46,116`;
  `core/resolution.rs:384,641`; `casting/casting.rs:88,199,729`;
  `core/invariants.rs:490,816,889`; `core/state_foundation.rs:34`;
  `casting/mana_and_lands.rs:119,495,514`; `core/turn_invariants.rs:47,104`;
  `core/concede.rs:42,81`; `rules/split_second.rs:597,677`; `rules/abilities.rs:190`.
  The property tests (`core/invariants.rs`, `core/turn_invariants.rs`) only pass priority —
  they never cast — so they are expected to stay green; if one of them fails, that is a
  **real regression** signal (see 9.3).

### 9.2 Most-likely test groups, ranked

| group / file | why |
|---|---|
| `crates/engine/tests/rules/split_second.rs` | 16 `priority_holder` refs; split second is inherently about a non-active player trying to respond |
| `crates/engine/tests/casting/casting.rs`, `casting/spell_cost_modification.rs` (24 refs), `casting/x_cost_spells.rs` | the casting flip lands here first |
| `crates/engine/tests/mechanics_a_d/cycling.rs` (11), `mechanics_a_d/crew.rs` (16) | the two clean instant-speed flips (P4, P5) |
| `crates/engine/tests/mechanics_a_d/bloodrush.rs` (5), `mechanics_m_z/ninjutsu.rs` (14) | the two narrow flips |
| `crates/engine/tests/rules/protection.rs` (14), `rules/targeting.rs`, `rules/grant_flash.rs` (12) | flash/instant-speed responses by non-active seats |
| `crates/engine/tests/mechanics_m_z/plot.rs` (22), `mechanics_e_l/foretell.rs` (23), `mechanics_m_z/suspend.rs` (9) | Step 7's `players_passed` resets |
| `crates/engine/tests/core/resolution.rs`, `core/turn_structure.rs` | step-advance timing shifts if a `players_passed` reset changes when a round completes |

### 9.3 Golden-script directories, ranked

Run:
```
~/.cargo/bin/cargo test -p mtg-engine --test scripts run_all_scripts
```
For a single script: `SCRIPT_FILTER=<name_without_ext> ~/.cargo/bin/cargo test -p mtg-engine --test scripts run_all_scripts -- --nocapture`.
**Do not start the replay-viewer HTTP server** (OOM/SIGKILL from an agent context —
`gotchas-infra.md`).

| dir | why |
|---|---|
| `test-data/generated-scripts/stack/` | densest in "p1 casts, p2 responds" patterns; `019_pass_priority_with_stack_item.json` is the canonical one (its note at `:125` already describes CR 117.3c behaviour under the stale name "CR 116.3c") |
| `test-data/generated-scripts/baseline/` | `016_two_bolts_same_player.json`, `017_sol_ring_then_cast_spell.json`, `019_...` |
| `test-data/generated-scripts/combat/` | `125_ninjutsu_...`, `075_crew_smugglers_copter_attacks.json`, `178_bloodrush_...` |
| `test-data/generated-scripts/replacement/`, `commander/` | lower risk; mostly single-actor |

The harness maps a script action's `priority_player` straight into the command's `player`
field (`crates/engine/src/testing/replay_harness.rs` `translate_player_action`), and the
engine rejects a non-holder. So a script whose `pass_priority` action names `p1` immediately
after a `p2` cast will now fail with `NotPriorityHolder`.

### 9.4 The triage rule — "test encoded the bug" vs "real regression"

For each failure, answer **one** question: *at the moment of the failure, who took the last
game action, and did they hold priority when they took it?*

**Classify as TEST-ENCODED-THE-BUG** (fix the test/script) when **all** hold:
1. The last action before the failure was a `CastSpell` / `ActivateAbility` / cycle / crew /
   bloodrush / ninjutsu / special action, and
2. the actor was **not** `state.turn().active_player`, and
3. the failure is either `NotPriorityHolder` on the next `PassPriority`, or an assertion
   that the holder is the active player.

Fix: reorder the `pass_all` list so it **starts with the actor** and wraps in APNAP order
(the actor, then clockwise); or update the assertion to `Some(<actor>)`. **Add
`/// CR 117.3c — the actor, not the active player, receives priority after casting/activating`
to the test's doc comment** — every touched test and every touched script `"note"` must carry
the citation. For scripts, insert the extra `pass_priority` step(s) needed and set
`"note": "CR 117.3c: <actor> retains priority after casting; the round restarts with them."`

**Classify as REAL REGRESSION** (investigate the engine edit) when **any** hold:
- The actor **was** the active player — the edit should have been an identity write, so a
  failure means the wrong variable was substituted (e.g. `stack_obj.controller`, `obj.owner`,
  or the defending player instead of `player`). Re-read the site.
- A `core/invariants.rs` / `core/turn_invariants.rs` property test fails — those never cast,
  so they cannot be encoding the cast-priority bug.
- The failure is `PlayerNotFound`, a panic, a hang, or `priority_holder == Some(<a player
  with has_lost/has_conceded>)` — that is the "actor died during their own action" hazard
  (§11 R2), not a test artifact.
- A `Group C` site's test moved — Group C was not supposed to change at all.
- `test_dp1_mana_ability_does_not_reset_players_passed` (P7) fails — the PRESERVE gate broke.

**Hard rule from conventions.md**: if a supposedly-fixed test still passes against the
*pre-fix* engine, it is a test-validity bug and a **fix-phase HIGH**, not a LOW.

**Counting discipline**: record the final tally as `N tests updated (bug-encoded) / M scripts
updated / 0 real regressions` in the implement commit message. A non-zero real-regression
count must be resolved, not carried.

---

## 10. Step 14 — simulator / M11-local `LocalGame`

**Good news, verified in source**: nothing in `crates/simulator` hardcodes "the active player
holds priority". `legal_actions.rs:193` gates on `priority_holder != Some(player)` and
`local_game.rs:310` derives the acting seat from `priority_holder`. Both follow the fix.

**The M11-S1 parity test is self-referential, not baseline-pinned.**
`crates/simulator/tests/local_game.rs:177 test_local_game_bot_only_matches_game_driver_for_fixed_seeds`
compares a `GameDriver` run against a `LocalGame` run **of the same seed** and asserts
`winner` / `turn_count` / `total_commands` match *each other*. Both sides move identically
under this fix, so it is expected to stay green. **If it fails, that is a real regression** —
it means the two paths diverged, which the fix should not cause.

The remaining tests in that file:
- `:228 test_local_game_halts_awaiting_human_at_first_priority` — asserts `decision.player ==
  PlayerId(1)`, `decision.seq == 1` at the **first** priority of the game, which is step-start
  AP priority (CR 117.3a, Group C, unchanged). Expected green.
- `:260`, `:315`, `:367`, `:403`, `:466`, `:507` — idempotence, seat-guard, journal, illegal-
  command and stale-seq tests. All structural. Expected green.
- `:552 test_local_game_max_consecutive_passes_halts` — behavioural; the trajectory changes,
  so it **may** move. If it does: this is expected-and-correct, not a regression. Update the
  expectation and add `/// CR 117.3c — a non-active actor now retains priority, which changes
  the bot-game trajectory; the halt threshold is unchanged.` Do **not** relax
  `max_consecutive_passes` to make it pass.

Run: `~/.cargo/bin/cargo test -p mtg-simulator`.

**Do not** attempt a full fuzzer-baseline diff as an oracle — OOS-M11-3 records that the
fuzzer is not run-to-run deterministic for long games, so a baseline diff cannot distinguish
this fix from that nondeterminism.

---

## 11. Risks and edge cases

- **R1 — the companion `PriorityGiven` event.** 12 sites in `abilities.rs` plus
  `casting.rs:4907`. Missing one leaves an event claiming the wrong player got priority while
  the state says otherwise; it compiles and most tests pass. **Mitigation**: after Step 3, run
  `rg -n "PriorityGiven \{ player: active \}" crates/engine/src` — it must return **zero**
  hits, and `rg -n "let active = state.turn.active_player" crates/engine/src/rules/abilities.rs`
  must also return zero.
- **R2 — the actor dies during their own action.** `handle_pay_echo` / `_cumulative_upkeep` /
  `_recover` / `_craft` run `sba::check_and_apply_sbas` before granting priority, and
  `process_command` runs `check_and_flush_triggers` after several handlers. If a Phyrexian-mana
  life payment or a triggered drain kills the actor, `priority_holder = Some(<dead player>)`
  would violate INV-PI-02 (`core/invariants.rs:186`). Symmetric with the pre-existing "AP
  dies" hazard, and the Group-A sites all guard `priority_holder == Some(player)` at *entry*
  only. **Mitigation**: INV-PI-02 and `core/concede.rs` are the detectors; if one fires, do
  not add a liveness check inside the handler (new surface) — report it as a finding.
- **R3 — Step 8 hands priority to a non-holder.** `handle_turn_face_up`,
  `handle_activate_loyalty_ability` and `handle_level_up_class` have no entry priority guard.
  Mitigated by making Step 8 independently revertable (§5 D-c escape hatch).
- **R4 — Step 7 changes *when* a step advances.** Adding a `players_passed` reset to
  foretell/plot/suspend/companion means a round that used to complete now needs more passes.
  Any test that counted passes to reach a step will need one more pass. That is
  bug-encoded-in-the-test, and CR 117.4 is the citation to put on the fix.
- **R5 — comment-only steps drifting into code.** Steps 5, 6, 9 and 10 change **zero**
  executable lines. Verify with `git diff --stat` per step; a logic hunk in one of those
  files is a scope escape.
- **R6 — the temptation to re-pin a fingerprint.** If `core/protocol_schema.rs`,
  `core/hash_schema.rs` or any `PROTOCOL_VERSION`/`HASH_SCHEMA_VERSION` sentinel fails,
  **STOP** — do not re-pin. It means an unintended shape change slipped in.
- **R7 — DP-21's split-second hole widens.** Post-fix, a non-active player who casts an
  instant now retains priority and can cast again. `casting.rs:6887 has_split_second_on_stack`
  and `abilities.rs:153` already gate that path, so split second holds. But
  `handle_activate_loyalty_ability` has **no** split-second check (DP-21) — the fix makes that
  gap slightly easier to hit. Record, do not fix. → OOS-DP1-2.
- **R8 — over-collecting the citation sweep.** Rewriting the ~60 JSON `"note"` strings would
  produce a diff so large the reviewer cannot see the engine change. Explicitly out of scope
  (§6, OOS-DP1-3).

---

## 12. Step 15 — gates

```
~/.cargo/bin/cargo fmt --check
tools/check-defs-fmt.sh                       # SR-35 — cargo fmt checks none of the 1,798 defs
~/.cargo/bin/cargo clippy --workspace --all-targets -- -D warnings
~/.cargo/bin/cargo build --workspace          # catches replay-viewer / TUI breakage
~/.cargo/bin/cargo test --all
```

Explicit wire confirmation to state in the commit message and in `pb-review-DP1.md`:
- `crates/engine/tests/core/protocol_schema.rs` green, `PROTOCOL_VERSION == 27` unchanged
- `crates/engine/tests/core/hash_schema.rs` green, `HASH_SCHEMA_VERSION == 63` unchanged
- `git diff --stat` shows **no** file under `crates/card-defs/` and **no** change to
  `docs/authoring-status.md` — coverage stays 1,139/1,804 = 63.1%

---

## 13. Step 16 — close-out

1. `docs/audits/decision-point-audit.md`:
   - §5 Tier 0 **DP-1** row → mark `SHIPPED (PB-DP1, scutemob-149)`; correct the row's
     "~20 sites" to the verified breakdown (14 Group A of which 6 flip, 3 Group B ruled
     no-change, 8 Group D, 5 false positives).
   - §8 **PB-DP1** row → mark shipped; note the two handlers the audit's site list missed
     (`handle_activate_loyalty_ability`, `handle_level_up_class`).
   - Correct §5 DP-1's own claim that `combat.rs:1373` and `engine.rs:1759/:1805` are DP-1
     sites — they are CR 117.3a and were false positives.
   - Bump `<!-- last_updated: 2026-07-26 -->` if the file carries one.
2. `memory/primitive-wip.md`: tick steps 1-11; record the four §0 corrections so the review
   agent sees that the plan contradicted the sweep with evidence.
3. File seeds (in `memory/workstream-state.md` "Last Handoff", and cross-reference from the
   audit doc):
   - **OOS-DP1-1** — echo / cumulative-upkeep / recover reassign priority to the AP out of
     band; real fix is the DP-11 pause (PB-DP4 owns it).
   - **OOS-DP1-2** — five handlers take a player action with **no** entry priority guard:
     `handle_activate_craft`, `handle_turn_face_up`, `handle_activate_loyalty_ability`,
     `handle_level_up_class`, `handle_bring_companion`. Plus: craft resolves immediately
     instead of using the stack (CR 702.167a), and loyalty has no split-second check
     (DP-21's class).
   - **OOS-DP1-3** — stale pre-renumber CR citations (`116.3a/b/c/d` for what is now
     `117.3a-d` / `117.4`) survive in ~60 golden-script `"note"` fields, the
     `cr_sections_tested` array of `baseline/001_priority_pass_empty_stack.json`,
     `docs/mtg-engine-milestone-reviews.md:326-327`, and seven `memory/abilities/*.md`
     records. Cosmetic; batch it into a doc pass, not a PB.
4. Update `CLAUDE.md` "Current State" with a one-line snapshot delta only; the narrative goes
   to `memory/archive/claude-md-changelog-2026-07.md`.

---

## 14. Verification checklist

- [ ] Probe file `crates/engine/tests/primitives/pb_dp1_actor_priority.rs` created and
      registered in `primitives/main.rs`; **RED output captured before any engine edit**
- [ ] P1-P5, P8 fail pre-fix; P6, P7 pass pre-fix and post-fix
- [ ] Group A: 14 assignments changed to `Some(player)`; 13 `PriorityGiven` pushes changed;
      `rg "PriorityGiven \{ player: active \}" crates/engine/src` returns 0
- [ ] `rg "let active = state.turn.active_player" crates/engine/src/rules/abilities.rs` returns 0
- [ ] `rg "602\.2e" crates/engine/src` returns 0
- [ ] `rg "116\.3[abcd]" crates/engine/src` returns 0
- [ ] Group B: `engine.rs:755/757`, `:956/958`, `:1070/1072` logic **byte-identical**; only
      comments changed
- [ ] Group C: `resolution.rs:114/5175/5835/7744/7983`, `combat.rs:680/1373`,
      `engine.rs:1759/1805`, `turn_structure.rs:103/140` logic **byte-identical**
- [ ] Group D-a: `lands.rs` logic byte-identical; two comments corrected
- [ ] Group D-b: `players_passed` reset added to foretell / plot / suspend / bring_companion
- [ ] Group D-c: priority grant added at `engine.rs:1621`, `:2654`, `:2814` (or documented
      partial revert per the §5 escape hatch)
- [ ] `mana.rs` has **zero** code-line changes
- [ ] Full `cargo test --all` green; failure triage recorded as `N test / M script updated,
      0 real regressions`; every updated test and script note cites **CR 117.3c** (or 117.4
      for the Step-7 pass-round changes)
- [ ] `cargo test -p mtg-simulator` green; any `max_consecutive_passes` expectation change
      documented with a CR citation
- [ ] `cargo build --workspace`, `cargo clippy --workspace --all-targets -- -D warnings`,
      `cargo fmt --check`, `tools/check-defs-fmt.sh` all clean
- [ ] **PROTOCOL 27 / HASH 63 unchanged**; no sentinel re-pinned
- [ ] Zero files changed under `crates/card-defs/`; `docs/authoring-status.md` unmoved;
      **0 coverage flips**
- [ ] Audit doc §5/§8 updated; seeds OOS-DP1-1/2/3 filed; `primitive-wip.md` ticked
