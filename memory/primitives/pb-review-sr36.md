# Primitive Batch Review: SR-36 — SF-8 (`AddManaScaled` taps for exactly 1) + SF-9 (`Cost::PayLife` silently unpaid)

**Date**: 2026-07-17
**Reviewer**: primitive-impl-reviewer (Opus)
**Worktree**: `/home/skydude/projects/scutemob/.worktrees/scutemob-92`
**Diff**: `1a3602bc`, `91ce106c` (`git diff 7b2310dd..HEAD`)
**Plan**: `memory/primitives/pb-plan-sr36.md`
**Findings source**: `memory/card-authoring/sr34-engine-findings-2026-07-17.md` §SF-8 / §SF-9
**CR rules verified via MCP**: 119.4, 119.4a, 119.4b, 605.1, 605.1a, 605.1b (118.3 cited by the code, not independently re-quoted here)

**Engine files reviewed**: `crates/card-types/src/state/game_object.rs`,
`crates/engine/src/rules/mana.rs`, `crates/engine/src/rules/abilities.rs`,
`crates/engine/src/testing/replay_harness.rs`, `crates/engine/src/state/hash.rs`,
`crates/engine/src/rules/protocol.rs`, `crates/engine/src/effects/mod.rs` (read-only:
`resolve_amount`, `matches_filter`), `crates/simulator/src/legal_actions.rs`,
`crates/simulator/src/driver.rs`

**Card defs reviewed (9)**: `cabal_coffers.rs`, `cabal_stronghold.rs`, `crypt_of_agadeem.rs`,
`staff_of_compleation.rs`, `voldaren_estate.rs`, `aetherflux_reservoir.rs`,
`yawgmoth_thran_physician.rs`, `athreos_god_of_passage.rs`, `crossway_troublemakers.rs`
(plus a corpus-wide `Cost::PayLife` grep across all 1,748 defs to bound SF-9's blast radius)

**Tests reviewed**: `tests/primitives/primitive_sr36_scaled_mana_and_life_costs.rs` (all 11),
`tests/primitives/primitive_sr34_composite_mana_costs.rs` (T10/T11/`sr34_roster_markers_…`),
`tests/casting/mana_filter.rs`, `tests/core/effect_choose_gate.rs`

**Tooling note**: no Bash tool in this session — every finding below is proven by reading the
dispatch chain, not by running the suite. Where I say a test cannot distinguish two behaviours,
the proof is structural and stated inline so it can be checked without running anything.

## Verdict: needs-fix (0 HIGH, 3 MEDIUM, 6 LOW)

**There is no HIGH finding, and I am not going to manufacture one.** Both engine fixes are
correct against the CR text I verified independently; the two paths really are disjoint by
construction; `+=` cannot double-count; the ordering of step 6c is right for every
`EffectAmount` in the roster and the doc comment's reason for it is true; the hash and
protocol bumps are complete and consistent; and every activation test's expected amount is
chosen so it differs from the pre-fix constant `1` (I checked each board individually — the
one hazard the brief named first). The three `Partial -> Complete` upgrades each survive a
full-def re-read against oracle text.

The finding the brief predicted is there, though, and it is the usual shape: **the sharpest
problems are in the checkers and their prose, not the code**. Two doc comments written *in this
diff* say things the same diff makes false, and they contradict each other; one new test does
not test the property its own doc comment says it pins, which matters because that test is the
named evidence for a marker upgrade to `Complete`; and SF-9's fix silently resolved a third
card's recorded blocker without anyone updating its note — the third consecutive stale-marker
finding on this campaign.

## Engine Change Findings

| # | Severity | File:Line | Description |
|---|----------|-----------|-------------|
| 4 | LOW | `testing/replay_harness.rs:3885` | **`cost_to_activation_cost`'s doc still says `PayLife` is silently ignored.** Three lines above the fix. **Fix:** rewrite the sentence. |
| 5 | LOW | `rules/abilities.rs:524` | **"before any cost is paid" is false** — the tap cost is paid at 488-523. **Fix:** say "before the mana cost". |
| 7 | LOW | `testing/replay_harness.rs:3805-3810` | **"correct-but-slow, not wrong" contradicts SR-33's own CR 605.3b position**; guard also untested. **Fix:** reword + add a synthetic test. |
| 8 | LOW | `simulator/src/legal_actions.rs:399` | **`LegalActionProvider` ignores `life_cost`.** Bot is offered unpayable activations. **Fix:** add the check or file it. |

## Card Definition Findings

| # | Severity | Card | Description |
|---|----------|------|-------------|
| 3 | **MEDIUM** | `yawgmoth_thran_physician.rs:54-62` | **Stale marker note — SF-9 resolved its named "real blocker".** **Fix:** rewrite the note to the surviving blocker. |
| 9 | LOW | `voldaren_estate.rs:83-86` | **The note's reason for not lowering is not the real reason.** **Fix:** restate it. |

## Test / Checker Findings

| # | Severity | File:Line | Description |
|---|----------|-----------|-------------|
| 1 | **MEDIUM** | `tests/casting/mana_filter.rs:376-386` | **Doc comment false on both halves, and contradicts `effect_choose_gate.rs` in the same commit.** **Fix:** rewrite. |
| 2 | **MEDIUM** | `tests/primitives/primitive_sr36_scaled_mana_and_life_costs.rs:734-799` | **The test does not exercise `basic: true`; its decoy is not a Swamp.** **Fix:** use a nonbasic Swamp decoy. |
| 6 | LOW | `tests/core/effect_choose_gate.rs:403` | **`scaled_amount.is_none()` is the weaker of two symmetric fixes.** **Fix:** consider including the scaled colour on both sides. |

---

### Finding Details

#### Finding 1: `mana_filter.rs`'s SR-36 note is false on both halves and contradicts the other gate written in the same commit

**Severity**: MEDIUM
**File**: `crates/engine/tests/casting/mana_filter.rs:376-386`
**Issue**: The note added by this task to `test_add_mana_scaled_orphan_fix_all_cards` says:

> **Cabal Stronghold and Crypt of Agadeem's scaled amounts are not verified by activation
> anywhere in the suite as of this task** — only their `Completeness::Partial` marker is
> checked (`primitive_sr34_composite_mana_costs.rs::sr34_roster_markers_match_the_reconciliation`).
> That is a real, named gap, not silently dropped coverage; closing it is §3 work (card-def
> marker reconciliation), out of this task's scope.

Every clause of that is false **as of the same commit**:
- Their scaled amounts *are* verified by activation, by
  `primitive_sr36_scaled_mana_and_life_costs.rs::cabal_stronghold_counts_only_basic_swamps`
  and `::crypt_of_agadeem_counts_only_black_creature_cards_in_graveyard`.
- Their `Completeness::Partial` marker is *not* checked, because they are no longer `Partial`
  — this task upgraded both to `Complete` and **deleted their rows** from
  `sr34_roster_markers_match_the_reconciliation` (`primitive_sr34_composite_mana_costs.rs:1197-1201`).
- The "§3 work, out of this task's scope" framing describes work this task actually did.

`tests/core/effect_choose_gate.rs:279-285`, written in the same commit, states the opposite and
correctly names all three activation tests. A future worker reading `mana_filter.rs` would
conclude coverage is missing and either duplicate it or, worse, treat the two `Complete`
markers as unbacked. This is precisely the "documented hazard that nothing executes" /
stale-prose class CLAUDE.md records against SR-9b and the marker sweep.
**Fix**: rewrite the paragraph to state that the three composite-cost scaled sources are
covered by the three named tests in `primitive_sr36_scaled_mana_and_life_costs.rs`, and that
this test's list stays bare-`Cost::Tap` cards only because that is its scope — not because
coverage is missing elsewhere. Drop the `sr34_roster_markers_match_the_reconciliation`
reference entirely; those rows no longer exist.

#### Finding 2: `cabal_stronghold_counts_only_basic_swamps` does not exercise `basic: true` — the decoy is not a Swamp

**Severity**: MEDIUM
**File**: `crates/engine/tests/primitives/primitive_sr36_scaled_mana_and_life_costs.rs:734-799`
**Oracle**: Cabal Stronghold — "{3}, {T}: Add {B} for each **basic** Swamp you control."
**Issue**: The test's doc comment claims:

> The board carries a nonbasic Swamp (Cabal Coffers is not a Swamp at all; a nonbasic Swamp
> decoy is the point) — `TargetFilter::basic` must be live.

The board carries no nonbasic Swamp. The decoy is `Cabal Coffers`, whose def is
`types(&[CardType::Land])` (`cabal_coffers.rs:10`) — **no subtypes at all**. The filter is
`TargetFilter { has_card_type: Land, has_subtype: Some(Swamp), basic: true }`
(`cabal_stronghold.rs:39-44`), and `matches_filter` (`effects/mod.rs:7930-7984`) rejects
Cabal Coffers on `has_subtype` before `basic` (line 7978) is ever consulted. The other two
board permanents are basic Swamps. **Therefore deleting `basic: true` from the def leaves the
count at 2 and the test green** — it cannot distinguish "basic Swamp" from "Swamp", which is
the one clause distinguishing Cabal Stronghold from Cabal Coffers.

This matters more than a normal test gap for two reasons. First, the file's own header
(lines 721-732) advertises exactly this property — "Each board contains a decoy the count MUST
exclude, so a filter that silently degraded to a raw count would fail rather than pass with a
coincidentally-equal number" — and `effect_choose_gate.rs:279-285` names this test as one of the
three activation tests the `Partial -> Complete` upgrades rest on. Second, CLAUDE.md records
that "several `TargetFilter` fields are silently ignored by `matches_filter`", which is the
reason to test the field rather than trust it.

The def itself is **correct** — I traced the chain and `basic` is honoured at
`effects/mod.rs:7978-7984` against `SuperType::Basic`, and `EffectAmount::PermanentCount`
(`effects/mod.rs:6749-6771`) routes through `matches_filter`. So this is a vacuous-checker
finding, not a wrong-game-state finding. The `crypt_of_agadeem` sibling test is fine: its green
`Elvish Archdruid` decoy really is excluded by the `colors` filter, and the asserted `2`
(rather than `0` or `3`) is only reachable if `colors` is live.
**Fix**: replace the `Cabal Coffers` decoy with a **nonbasic Swamp** the corpus already has —
e.g. `Watery Grave` or `Overgrown Tomb` (both `Complete`, both carry the `Swamp` subtype
without `SuperType::Basic`) — and keep the expected count at 2. Then delete `basic: true` from
`cabal_stronghold.rs` locally and confirm the test goes red before restoring it; record which
assertion caught it, per the plan's §5 non-vacuity instruction.

#### Finding 3: Yawgmoth, Thran Physician's `Partial` note names a blocker SF-9 just deleted

**Severity**: MEDIUM
**File**: `crates/card-defs/src/defs/yawgmoth_thran_physician.rs:54-62`
**Oracle**: "Pay 1 life, Sacrifice another creature: Put a -1/-1 counter on up to one target
creature and draw a card." / "{B}{B}, Discard a card: Proliferate."
**Issue**: The marker note reads:

> The real blocker is Cost::PayLife: replay_harness.rs:3774 has no ActivationCost
> representation for it and silently drops it, so 'Pay 1 life, Sacrifice another creature: ...'
> would resolve as a free sacrifice. **Un-author until PayLife is representable in
> ActivationCost.**

SF-9 made `PayLife` representable in `ActivationCost` and paid in `handle_activate_ability`.
The note's named primary blocker is gone, its line reference (`replay_harness.rs:3774`) no
longer says what it claims, and its instruction to a future author ("un-author until…") now
has its precondition satisfied. The card is **still not upgradeable** — its stated engine gap
#2 (`Cost::Sacrifice` does not exclude the source despite "sacrifice another") survives, and I
did not re-verify that claim — so this is a note-accuracy finding, not a missed upgrade. But
CLAUDE.md records two consecutive stale-marker HIGHs and a third in the SR-34 task itself, and
the marker sweep found 42% of notes wrong; a note whose cited blocker was removed by the very
change under review is squarely in that class. Not HIGH because no wrong game state is
reachable and the card stays correctly non-`Complete`.
**Fix**: rewrite the note so engine gap #2 (`Cost::Sacrifice` not excluding the source) is the
stated blocker, record that `Cost::PayLife` is representable and paid as of SR-36
(`scutemob-92`), and delete the "un-author until PayLife is representable" instruction. Do
**not** upgrade the marker on the strength of this — per the `megrim.rs` calibration case, the
sacrifice clause must be verified independently first.

#### Finding 4: `cost_to_activation_cost`'s doc comment still says `PayLife` is ignored

**Severity**: LOW
**File**: `crates/engine/src/testing/replay_harness.rs:3885`
**Issue**: `/// Handles Tap, Mana, Sacrifice, DiscardCard, and Sequence (recursively).`
`/// Unrecognised cost components (`PayLife`) are silently ignored.` — three lines above
`flatten_cost_into`, whose `Cost::PayLife(n) => ac.life_cost += *n` arm (line 3929) is the SF-9
fix. The plan (§2 step 2) said to delete the "no ActivationCost representation yet" comment;
the inline one was deleted, this one was not.
**Fix**: rewrite to name `PayLife` as handled (`ActivationCost::life_cost`, CR 118.3/119.4) and
list what is genuinely dropped (`ExileFromHand` — a spell alt-cost, per line 3930).

#### Finding 5: `abilities.rs`'s life-check comment says "before any cost is paid"; the tap is already paid

**Severity**: LOW
**File**: `crates/engine/src/rules/abilities.rs:524`
**Issue**: The comment reads "life-cost legality check, **before any cost is paid**". The tap
cost is paid immediately above at lines 488-523 (`obj.status.tapped = true` at 516-518, plus a
`PermanentTapped` event). The check is before the *mana* cost, which is what the plan asked for
and is fine — an `Err` discards the whole `GameState` regardless, as the payment comment at
592-597 correctly explains. So this is prose, not behaviour. Flagged because this file's
convention (and mana.rs's step 5b, which *is* genuinely before all payment) is that these
comments state a constraint precisely enough to be relied on.
**Fix**: change to "before the mana cost is paid" and lean on the existing 592-597 explanation
rather than restating a stronger guarantee than the code provides.

#### Finding 6: `registered_colors`'s `scaled_amount.is_none()` is the weaker of two symmetric fixes

**Severity**: LOW
**File**: `crates/engine/tests/core/effect_choose_gate.rs:403`
**Issue**: The brief flagged this as the highest-risk change and asked whether a card could now
register a colour it does not print. **I could not construct a live one, and I believe no live
hole exists today.** The reasoning:

- The only `Complete` defs with *both* a parsed clause and a scaled clause are Cabal Stronghold
  and Crypt of Agadeem. For Crypt the filter is a no-op (both arms are `{B}`). For Stronghold it
  prevents a false `invented [Black]`, which is the stated reason it was added.
- Every other `AddManaScaled` tap source (Gaea's Cradle, Elvish Archdruid, Priest of Titania,
  Marwyn, Circle of Dreams Druid, Howlsquad Heavy, Cabal Coffers) prints only the "for each"
  clause, so `printed` is empty and the caller's `if printed.is_empty() { continue; }` skipped
  them **before** this change too. No coverage moved for them.
- The reverse asymmetry fails **loudly**, which is the safe direction: a def that models a fixed
  `{T}: Add {G}` as `AddManaScaled` would drop Green from `registered` and the gate reports
  `missing [Green]`.

What the change does cost is strength. A `Complete` card whose scaled arm registers a colour it
never prints is now invisible to this gate — concretely, changing `cabal_stronghold.rs`'s scaled
arm to `ManaColor::Green` leaves `printed = {C}`, `registered = {C}`, and the gate green. That
is not a regression (pre-SF-8 that arm registered nothing, so the gate was equally blind), and
all nine current scaled rows have their colour pinned by activation elsewhere. But the
*symmetric alternative* is strictly stronger and was not considered in the doc comment: the
colour in "Add **{B}** for each basic Swamp" **is** printed and this parser already parses it —
keeping it on both sides needs no `scaled_amount` filter at all and would catch the invented
colour. The exclusion's stated justification ("this parser reads colours, never amounts") argues
for dropping the *amount*, not the colour.
**Fix**: either (a) include the scaled clause's colours in `printed` and drop the
`scaled_amount.is_none()` filter, so the gate checks scaled arms' colours like every other arm;
or (b) keep the current shape and add one sentence to the doc comment recording that a future
`Complete` scaled arm's colour is **not** covered here and its author must add an activation
test. Do not leave it implying the two options are equivalent.

#### Finding 7: the non-`Controller` refusal is untested, and "correct-but-slow, not wrong" contradicts the project's own CR 605.3b position

**Severity**: LOW
**File**: `crates/engine/src/testing/replay_harness.rs:3805-3810, 3817-3819`
**CR Rule**: 605.1a — "An activated ability is a mana ability if… **it could add mana to a
player's mana pool** when it resolves" (verified via MCP: "a player's", not "its controller's").
**Issue**: Two small things. (1) The guard is correct and I support it — the stackless path
always pays the activating player, so refusing to lower is better than paying the wrong player.
But the comment calls the result "correct-but-slow, not wrong." Per CR 605.1a such an ability
*is* a mana ability, so leaving it on the stack violates CR 605.3b — which is the exact defect
SR-33 demoted 88 dual lands for ("registers zero mana abilities (CR 605.1a)"). SF-10's note in
the findings doc is honest about the same trade-off; this comment is not. (2) I verified by grep
that all 15 `Effect::AddManaScaled` sites in the corpus use `PlayerTarget::Controller`, so the
guard is a latent-defect guard with zero live cards — same class as SR-34's Finding-1 double-
`Cost::Mana` guard, which is likewise untested. No test in
`primitive_sr36_scaled_mana_and_life_costs.rs` mentions `PlayerTarget` at all.
**Fix**: reword to match SF-10's framing ("leaves a CR 605.3b timing deviation rather than a
wrong-player payment; zero live cards, guarded so a future def cannot pay the wrong player
silently"), and add a synthetic-def test asserting a non-`Controller` `AddManaScaled` lowers to
zero mana abilities and one activated ability.

#### Finding 8: the simulator's legal-action provider ignores `life_cost`

**Severity**: LOW
**File**: `crates/simulator/src/legal_actions.rs:399-401`
**Issue**: `legal_actions` gates activated abilities on `ability.cost.mana_cost` only —
`grep` finds **zero** references to life or `life_total` in the file. Post-SF-9 a bot at 1 life
is still offered Doom Whisperer's `Pay 2 life: Surveil 2` and every fetchland crack, and
`process_command` now returns `Err(InsufficientLife)` where it used to succeed. I checked the
consumer: `driver.rs:233-259` catches the `Err` and falls back to `PassPriority`, so this
degrades bot play and adds log noise rather than crashing or corrupting a game — hence LOW, not
MEDIUM. Note this is the same gap SR-34 opened for `ManaAbility::life_cost` (horizon lands,
Mana Confluence) and it was not filed then either; SF-9 widens it to 28 ability rows including
the 11 fetchlands.
**Fix**: add `ability.cost.life_cost == 0 || player.life_total >= life_cost as i32` alongside
the `can_afford` check (and the `ManaAbility::life_cost` equivalent on the TapForMana path), or
file it into `memory/card-authoring/sr36-engine-findings-2026-07-17.md` per the plan's §8
instruction to file rather than fix out-of-scope discoveries.

#### Finding 9: Voldaren Estate's note gives a reason for not lowering that is not the real reason

**Severity**: LOW
**File**: `crates/card-defs/src/defs/voldaren_estate.rs:83-86`
**Issue**: The note says the ability "stays on the stack path… **deliberately not lowered,
since adding an arm would trade this colour bug for the cost-drop bug SF-9 fixed elsewhere**."
That reasoning does not hold post-SF-9: lowering it would move it to `handle_tap_for_mana`,
where SR-34's `ManaAbility::life_cost` already pays the life (step 5b/6b) — there is no cost-drop
to trade into. The plan's §3 gives the real reason and it is a good one: `AddManaAnyColorRestricted`
produces colorless on *both* paths (SF-11), so lowering buys nothing and adds surface. The
outcome (leave it alone, stay `KnownWrong`) is right; only the stated reason is wrong.
**Fix**: restate as "not lowered because `AddManaAnyColorRestricted` produces `{C}` on both the
mana-ability and stack paths (SF-11), so lowering would move the ability without fixing the
colour bug — the life cost is paid correctly either way as of SR-36."

---

## What I checked and found clean (the brief's seven questions)

| # | Brief's question | Verdict |
|---|-----------------|---------|
| 1 | Is any new/edited gate vacuous? A scaled test expecting **1** would pass both ways. | **Clean on the amount axis** — I checked every board individually. Gaea's Cradle 0-creature → 0 and 3-creature → 3; `mana_filter.rs` Cradle → 0, Archdruid → 2, Priest → 2 (opponent's Elf, also pinning `EachPlayer` scope), Marwyn → 2 (via a `+1/+1` counter, so it differs from base power *and* from 1), Circle of Dreams → 2, Howlsquad → 2, Nyxbloom → 9 (distinguishes "marker tripled" = 3 from "real count tripled" = 9), Coffers → 3B + `{2}` spent, Stronghold → 2B + `{3}` spent, Crypt → 2B + `{2}` spent. **Not one expected value is 1.** Life: 39/38/37/36 and `InsufficientLife` with the pre-command state re-asserted. **One vacuity found on a different axis** — Finding 2 (`basic: true`). |
| 2 | Is `scaled_amount.is_none()` narrowed too far? Construct a case that now passes and should not. | **Could not construct a live one.** Full reasoning in Finding 6. It is weaker than the symmetric alternative and I filed that as LOW, but no `Complete` def slips through today and nothing regressed relative to pre-SF-8. |
| 3 | Did deleting 3 rows from `sr34_roster_markers_match_the_reconciliation` delete coverage? | **No — coverage moved and got stronger.** The rows asserted only `completeness == Partial`. Each card's real properties are now asserted by activation: registers as a `ManaAbility` + not in `activated_abilities` + off the stack (`stack_objects().is_empty()`) + correct scaled amount + generic cost actually spent. Upgrading to `Complete` also newly subjects all three to the broad `Complete`-scoped gates (`every_complete_land_registers_each_printed_tap_mana_color`, `effect_choose_gate`, the registry gate) — I traced each: Coffers is skipped (`printed` empty), Stronghold passes `{C}`/`{C}`, Crypt passes `{B}`/`{B}`. The one property genuinely weakened is Stronghold's `basic` clause — Finding 2. |
| 4 | The 3 upgrades to `Complete` — does **every** printed clause work (`megrim.rs` rule)? | **Yes, all three.** Verified each def line-by-line against MCP oracle text. **Cabal Coffers**: one ability; filter `Land + subtype Swamp` correctly implements the 2021-06-18 ruling ("counts each land you control with the subtype Swamp, not just ones named Swamp") — a `basic: true` here would have been a bug. **Cabal Stronghold**: `{T}: Add {C}` is `mana_pool(0,0,0,0,0,1)` = colorless 1 ✓; scaled arm has `basic: true` ✓ (correct, though untested — Finding 2). **Crypt of Agadeem**: all three clauses present — "This land enters tapped" as a self `ReplacementModification::EntersTapped` (the same shape 14 other lands use), `{T}: Add {B}` = `mana_pool(0,0,1,0,0,0)` ✓, and the scaled arm's `colors: {Black}` is "any of" semantics at `effects/mod.rs:7962-7966`, which is right for "black creature card" (a B/G creature counts). |
| 5 | Step 6c ordering — does any `EffectAmount` read state steps 6/6b mutate? | **Correct, and the doc comment's reason is true, not assumed.** I read `resolve_amount`'s arms: `PermanentCount` (`effects/mod.rs:6749-6771`) filters on `zone == Battlefield && is_phased_in() && controller`, then `matches_filter(&chars, …)` — and `matches_filter` takes `&Characteristics`, which **structurally cannot** carry tapped status, so the {T} in step 6 is invisible to it by type. `CardCount` reads a graveyard. `PowerOf(Source)` (Marwyn) is unaffected by tapping. Step 6b spends mana/life; no `EffectAmount` reads a mana pool or life total in this roster. Placement before step 7's sacrifice is right for the CR 400.7 reason the comment gives (`PowerOf(Source)` needs a live `source`), and the comment correctly flags that no current card combines the two. |
| 6 | Double-charge — are the paths really disjoint? | **Yes, proven at the source.** `enrich_spec_from_def` (`replay_harness.rs:2118-2130` and `2133-2173`) calls the *same* `mana_ability_lowering(ab_targets, cost, effect)`; the second loop's condition is its exact negation (`if !is_tap_mana_ability`). An ability in `mana_abilities` is therefore never in `activated_abilities`, so `handle_tap_for_mana`'s step 6b and `handle_activate_ability`'s life payment cannot both fire for one ability. I also checked the other construction paths the brief named: `Reconfigure` and `Outlast` build `ActivationCost`s with no `PayLife` component (`cost_to_activation_cost(&Cost::Mana(…))` / a literal with `life_cost: 0`); `LayerModification::AddManaAbility` / `AddActivatedAbility` grant into one list or the other, never both, and each `Command` pays only its own list's cost once. `is_tap_mana_ability_agrees_with_the_lowering` (T11) pins the partition, and its own doc correctly warns it is a partition guard, not a lowering-correctness guard. |
| 7 | Does `flatten_cost_into`'s `+=` double-count across abilities? | **No.** `flatten_cost_into` is called from exactly three sites (`replay_harness.rs:3888, 2219, 2236`), and every one passes a **fresh** accumulator: `cost_to_activation_cost` does `let mut ac = ActivationCost::default();` per call, and the two `Reconfigure` sites call `cost_to_activation_cost` (not `flatten_cost_into`) per ability. No `ActivationCost` is ever reused across abilities. `+=` is correct and necessary — the walk is recursive over `Cost::Sequence` and `mana_ability_cost_components`'s sibling accumulator (line 3685) uses `+=` for the same reason. No corpus def has two `PayLife` components today (verified by grep), so this is a latent-defect guard, correctly built. |

## CR Coverage Check

| CR Rule | Implemented? | Tested? | Notes |
|---------|-------------|---------|-------|
| 605.1a (mana ability by what it does, incl. dynamic amount) | Yes | Yes | `handle_tap_for_mana` step 6c; `gaea_cradle_*`, `elvish_archdruid_counts_only_elves`, `cabal_*`, `crypt_*`. Rule text verified via MCP — "could add mana to **a player's** mana pool", which is why Finding 7's guard is a deviation and not a rules requirement. |
| 605.3b (mana ability never uses the stack) | Yes | Yes | `stack_objects().is_empty()` asserted in `cabal_coffers_is_a_real_mana_ability`, `cabal_stronghold_counts_only_basic_swamps`. Not asserted in `crypt_of_agadeem_…` (LOW-grade omission; the sibling covers the shape). |
| 106.12b (mana-production replacement multiplies the **real** amount) | Yes | Yes | Step 7b `base_preview` substitution; `gaea_cradle_scaled_amount_is_multiplied_by_a_mana_production_replacement` → 9, which is the assertion that distinguishes "marker tripled" (3) from correct (9). |
| 118.3 / 119.4 (life payment legal only if `life_total >= amount`) | Yes | Yes | `abilities.rs:532-541`; `non_mana_ability_insufficient_life_is_rejected` (required 2, actual 1, pre-state re-asserted). Rule text verified via MCP. |
| 119.4b (0 life always payable, at any life total) | Yes | Yes | Short-circuit `if life_cost > 0` at `abilities.rs:532` and `mana.rs:164`; `non_mana_ability_life_cost_zero_is_legal_at_negative_life` activates at −3 life. Rule text verified via MCP — including "even if an effect says players can't pay life", which neither path models (out of scope, no card). |
| 601.2h (tap/mana/life/sacrifice payable in any order) | Yes | n/a | Cited at `mana.rs:202-211` and `abilities.rs:592-597`; the latter correctly declines to overclaim a transactional guarantee. See Finding 5 for the one imprecise clause. |
| 400.7 (source is a dead ObjectId after sacrifice) | Yes | No | Step 6c is ordered before step 7 for this reason; no card combines `scaled_amount` with `sacrifice_self`, so untested by construction. Correctly documented as a forward-looking constraint rather than a live fix. |

## Card Def Summary

| Card | Oracle Match | TODOs Remaining | Game State Correct | Notes |
|------|-------------|-----------------|-------------------|-------|
| `cabal_coffers.rs` | Yes | 0 | Yes | Filter matches the 2021-06-18 ruling (any land with subtype Swamp). `Complete` upgrade justified. |
| `cabal_stronghold.rs` | Yes | 0 | Yes | Both clauses modelled; `basic: true` correct and honoured by `matches_filter`. `Complete` justified — but see Finding 2: the test that claims to pin `basic` does not. |
| `crypt_of_agadeem.rs` | Yes | 0 | Yes | All three clauses (enters-tapped replacement + 2 abilities). `Complete` justified. |
| `staff_of_compleation.rs` | Yes | 0 | No (declared) | Stays `KnownWrong`; note correctly rewritten — SF-9 half resolved and probed (40→37, 40→36), surviving blocker is SF-11's colour bug only. Note's claims are backed by tests in this diff. |
| `voldaren_estate.rs` | Yes | 0 | No (declared) | Stays `KnownWrong`; note updated but its "why not lowered" reason is wrong — Finding 9. |
| `yawgmoth_thran_physician.rs` | Partial (declared) | 2 | n/a | **Stale note — Finding 3.** Not on the plan's roster; SF-9 resolved its cited blocker as a side effect. |
| `aetherflux_reservoir.rs` | Yes | 1 | Now pays 50 life | Stays `Partial`, correct per plan §3. Its note is truncated mid-sentence ("needs a…") — **pre-existing**, not introduced here; plan explicitly permitted leaving it. Worth a follow-up. |
| `athreos_god_of_passage.rs` | n/a | 1 | n/a | Note mentions `Cost::PayLife` only in a `MayPayOrElse` context — unaffected by SF-9, correctly untouched. |
| `crossway_troublemakers.rs` | n/a | 0 | n/a | `Cost::PayLife(2)` is inside `Effect::MayPayThenEffect`, a different dispatch path — correctly out of scope. |

## Version-bump verification

| Item | Status |
|------|--------|
| `PROTOCOL_VERSION` 3 → 4 | Present (`protocol.rs:82`), with an accurate `- 4:` History entry naming both fields and correctly noting the closure size is unchanged (`EffectAmount` was already reachable via `Effect`). |
| `PROTOCOL_SCHEMA_FINGERPRINT` re-pinned | `45dd82a1…` (`protocol.rs:100`), matching the appended `PROTOCOL_HISTORY` tail row (`protocol.rs:167-172`). Append-only respected — rows 2 and 3 untouched. |
| `HASH_SCHEMA_VERSION` 41 → 42 | Present, with `- 42:` doc entry (`hash.rs:385-389`) that correctly states where each field is fed into the stream. |
| `HashInto for ManaAbility` | `life_cost` + `scaled_amount` both hashed (`hash.rs:1436-1439`), positioned as the doc claims. |
| `HashInto for ActivationCost` | `life_cost` hashed (`hash.rs:2693`) with a PB-S H1 citation consistent with its neighbours. |
| `Option<Box<EffectAmount>>` boxing rationale | Documented at `game_object.rs:215-219` against clippy's `large_enum_variant` on `LayerModification::AddManaAbility` — a real constraint, correctly stated rather than narrated. |
| SR-6 arrow (card-defs must not reach the engine) | Intact — `EffectAmount` lives in `card-types` (`cards/card_definition.rs`), so `ManaAbility::scaled_amount` adds no dependency. This was the plan's named "one thing that could sink this design"; it holds. |

## Recommended fix order

1. Finding 2 (`basic` decoy) — it is the evidence for a `Complete` marker; fix it and prove non-vacuity by reverting the def field.
2. Finding 1 (`mana_filter.rs` note) — false prose that contradicts another gate in the same commit.
3. Finding 3 (Yawgmoth note) — stale marker; the campaign's most expensive recurring defect class.
4. Findings 4, 5, 9 — one-line prose corrections.
5. Findings 6, 7, 8 — either act or file into `memory/card-authoring/sr36-engine-findings-2026-07-17.md` per plan §8.

## Previous Findings

None — first review of SR-36.
