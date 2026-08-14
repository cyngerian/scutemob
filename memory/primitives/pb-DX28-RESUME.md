# PB-DX28 — RESUME NOTE (paused 2026-08-14 for a system reboot)

**Read this first, then `pb-plan-DX28.md` and `pb-plan-DX28-part2.md`.**

## State of the branch

`feat/pb-dx28-untargeted-choice-class-the-owner-axis-oos-dx4-6-oos`

| commit | what |
|---|---|
| `c5b9e459` | the plan + the full census. Compiles, tests green. **This is the last good tree.** |
| HEAD (WIP) | **DOES NOT COMPILE, deliberately.** A partial type-addition pass, stopped mid-flight. |

The WIP commit adds `TargetOwner`, `TargetFilter.owner`, `TriggerCondition::WheneverCreatureDies
.owner`, `DeathTriggerFilter.owner_you`/`owner_opponent`, and the `mtg_engine` re-export. It fails
with **46 `E0063: missing field 'owner'`** errors across `crates/card-defs/src/defs/*.rs` — every
def that constructs `WheneverCreatureDies`. That is pure mechanical churn (`owner: None` on each),
not a design problem.

**One design finding is already banked in that commit and is worth keeping**: `DeathTriggerFilter`
lives in `state/game_object.rs`, which **cannot** import `cards::card_definition` types (the module
dependency runs `cards/` → `state/`, never the reverse), so the owner scope is decomposed into two
bools mirroring the existing `controller_you`/`controller_opponent` rather than storing a
`TargetOwner`.

## To resume

1. `~/.cargo/bin/cargo build --workspace` and add `owner: None,` to the 46 sites. Then the tree is
   back to compiling and part 1 can continue.
2. Re-dispatch the **part 1** run (`pb-plan-DX28.md` §2 owner axis + §3 `sword_of_war_and_peace` /
   `EffectTarget::DamagedPlayer`) with `primitive-impl-runner`. Remaining work in that run:
   engine enforcement sites, the three card-def repairs, `pb_dx28_owner_axis.rs` probes with a
   revert matrix, `filter_states_a_quality`'s exclusion-list entry.
3. Then the **part 2** run (`pb-plan-DX28.md` §1 + all of `pb-plan-DX28-part2.md`) — the
   untargeted-choice channel.
4. Then §4 allowlist retirement, the wire bumps, gates, `/review`.

If any of this looks doubtful, `git reset --hard c5b9e459` costs ~78 lines and loses nothing but
typing.

## Numbers that must not be re-guessed

* Pre-edit baseline **4,605 / 0 / 5**, 46 result-producing targets, measured on this branch before
  any edit. Full log: it was written to the session scratchpad and is **gone after the reboot** —
  if the delta has to be itemised by test name (AC 6453 requires it), re-measure the baseline at
  `c5b9e459` rather than trusting this line.
* **PROTOCOL 36 / HASH 75** at `c5b9e459`. Both must be taken from the failing gates' own output at
  the end, never predicted (PB-DX27's brief predicted "wire impact NONE" and the gate refuted it).
* Coverage **1,136 / 1,803 = 63.0%**, 0 flips expected.

## Census verdicts (AC 6448) — already done, do not redo

Full disposition tables are `pb-plan-DX28.md` §0. Headlines:

* Untargeted class: **18** `Complete` members, not the seed's floor of 14. Four the seed never
  named: `cloud_of_faeries`, `rewind`, `takenuma_abandoned_mire` (the only graveyard-zone member),
  `sword_of_war_and_peace` (a *player* clause — different repair, see plan §3).
* Owner class: **2** `Complete` members (`staff_of_compleation`, `nether_traitor`).
* Three refutations, each load-bearing: the **six mutate defs** already enforce ownership
  open-coded in `casting.rs` (`target_obj.owner != player`); `hanweir_battlements` is correct
  (`Effect::Meld` checks owner *and* controller); and **`nether_traitor`'s allowlist note cites
  `fecundity` as a member of this class, which is false** — `fecundity`'s gap is
  `ControllerOf(TriggeringCreature)`, a controller gap, as its own `partial` note says. That
  citation is corrected in place by this batch (plan §4).

## ESM

Task `scutemob-210`, still `in_progress`, **0 of 6 criteria satisfied** — correct, none is
verifiably met yet. Session `b029895a-2c85-4aa8-9806-82967ca02528` ended at the pause.
