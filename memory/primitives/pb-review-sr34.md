# Primitive Batch Review: SR-34 — Composite-cost mana abilities (CR 605.1a)

**Date**: 2026-07-17
**Reviewer**: primitive-impl-reviewer (Opus)
**Task**: `scutemob-90` | **Branch**: `feat/sr-34-mana-abilities-with-additional-costs-are-never-registe`
**CR Rules**: 605.1a, 605.3a/b/c, 601.2h, 602.2b/2c, 118.3/118.3a/b, 119.4/119.4b, 106.12/106.12a/106.12b, 106.1a/106.1b, 732
**Engine files reviewed**: `crates/card-types/src/state/game_object.rs`, `crates/engine/src/rules/mana.rs`, `crates/engine/src/state/error.rs`, `crates/engine/src/state/hash.rs`, `crates/engine/src/rules/protocol.rs`, `crates/engine/src/testing/replay_harness.rs`
**Test files reviewed**: `tests/primitives/primitive_sr34_composite_mana_costs.rs`, `tests/core/effect_choose_gate.rs`, `tests/casting/mana_filter.rs`
**Card defs reviewed**: 12 spot-checked of 17 demoted + 3 promoted + 1 fixed (`cabal_coffers`, `fiery_islet`, `magnifying_glass`, `goldhound`, `mana_confluence`, `phyrexian_altar`, `elvish_spirit_guide`, `simian_spirit_guide`, `food_chain`, `command_tower`, `birds_of_paradise`, `arcane_signet`)
**Scripts**: `test-data/generated-scripts/stack/099_magnifying_glass_investigate_clue_draw.json`

## Verdict: needs-fix (0 HIGH, 5 MEDIUM, 3 LOW) — ALL EIGHT RESOLVED (fix phase, `scutemob-90`)

**Fix-phase addendum (2026-07-17).** All eight findings applied. The reviewer flagged
Findings 1, 2 and 5 as analytic-only (no Bash tool available) and asked for empirical
verification before fixing — done for all three; all three **survived** the check (the
reviewer's reasoning was correct, not merely plausible):

- **Finding 1**: grepped every `Cost::Sequence` def in the corpus for a second
  `Cost::Mana(`. Confirmed **zero** live victims (7 files have `Cost::Sequence` +
  2+ `Cost::Mana(` textually, but in every case the second `Cost::Mana(` belongs to a
  *different* ability on the same card, not the same `Cost::Sequence` literal). Latent,
  as the reviewer said. Fixed anyway per the reviewer's directive (decline rather than
  overwrite) and pinned by a new negative control in T13.
- **Finding 2**: grepped `requires_tap: false` across `crates/engine/tests/` — every
  hit is an `ActivationCost` literal or `LayerModification::AddManaAbility` grant, none
  is a `ManaAbility` produced by `mana_ability_lowering`. Confirmed zero coverage, as
  the reviewer said. Fixed via option (b) (decline to lower a no-tap cost) and pinned
  by a new negative control in T13. Verified the three named defs (Elvish Spirit Guide,
  Simian Spirit Guide, Food Chain) are the only defs affected and are all already
  non-`Complete`, so nothing regresses.
- **Finding 5**: read `birds_of_paradise.rs:38` and `command_tower.rs:21` directly.
  Confirmed both are `Complete` (one explicit, one by-default) and both use
  `Effect::AddManaAnyColor`. Confirmed correct per the reviewer — not demoted here
  (scope call upheld), reconciliation doc and SF-11 both amended to name them.

No finding was refuted. All eight applied as directed by their fix shapes.

**No HIGH findings, and I tried to find one.** The CR-critical logic is correct on every axis
the brief named: all three CR 605.1a criteria are enforced (including the `targets.is_empty()`
check that closes the latent Deathrite Shaman hazard the roster doc flagged), the CR 119.4b
short-circuit and the CR 119.4 `>=` boundary are both present and both exercised by tests that
would fail if broken, CR 106.12's `requires_tap` gating was correctly left alone and T12 proves
its consequences rather than asserting its shape, the SR-28 snapshot invariant provably still
holds, and the three version constants are bumped completely and append-only with a clean
sentinel sweep. The tests are end-to-end (pool deltas, not `!is_empty()`), and T10's
`AddManaScaled` exclusion is genuinely load-bearing. The markers are the best this campaign has
produced: truthful, CR-cited, probe-backed.

The findings are all in the **gate/documentation layer**, which is where this campaign's findings
keep landing — and two of them are the campaign's signature defect: **a claim the task itself
proved false, left standing inside a checker** (Finding 3), and **a blind spot the task itself
discovered, left undocumented in the gate it affects** (Finding 4) — the exact thing the plan's
§9 instructed against and which the task correctly honoured for SF-8 but not for its own SF-12.

**Methodological caveat, stated up front**: I have no Bash tool in this environment, so the
perturbations the brief requested were performed by **tracing the dispatch chain analytically**,
not by executing them. Where I claim a test is non-vacuous I give the trace. Findings 1, 2 and 7
are reachability arguments and would benefit from an executed confirmation.

## Engine Change Findings

| # | Severity | File:Line | Description |
|---|----------|-----------|-------------|
| 1 | MEDIUM | `replay_harness.rs:3652` | **`Cost::Mana` overwrites where `Cost::PayLife` accumulates.** A second mana component in a `Cost::Sequence` silently replaces the first — under-charging. **Fix:** merge or return `None` on a second `Cost::Mana`. |
| 2 | MEDIUM | `replay_harness.rs:3729` / `mana.rs:173` | **`requires_tap: false` is newly reachable and wholly untested.** Roster doc explicitly called this path "unexercised… unproven". **Fix:** add a test, or refuse to lower a no-tap cost. |

## Card Definition / Gate Findings

| # | Severity | File | Description |
|---|----------|------|-------------|
| 3 | MEDIUM | `tests/core/effect_choose_gate.rs:136-140` | **A known-false claim left inside a gate.** SF-11 proves this doc comment false; it is unedited. **Fix:** rewrite the comment. |
| 4 | MEDIUM | `tests/core/effect_choose_gate.rs:249-275` | **SF-12's blind spot is undocumented in the gate it blinds.** SF-8's is documented; SF-12's is not. **Fix:** add exclusion (3). |
| 5 | MEDIUM | `birds_of_paradise.rs:38`, `command_tower.rs:21` | **The `any_color`→`{C}` demotion is roster-bounded, leaving the corpus self-contradictory.** **Fix:** state the bound in the reconciliation, or extend per SF-11. |
| 6 | LOW | `primitive_sr34_composite_mana_costs.rs:585` | **T11 cannot fail except on re-divergence.** Doc oversells it. **Fix:** soften the doc. |
| 7 | LOW | `primitive_sr34_composite_mana_costs.rs:825` | **`cards_pos` is built, pushed to, and never read.** **Fix:** delete. |
| 8 | LOW | `magnifying_glass.rs`, `mana_confluence.rs`, `goldhound.rs` | **Touched defs were not `rustfmt`'d** (plan §3 step 8). CI cannot catch this (SR-35). **Fix:** run `rustfmt` by name. |

### Finding Details

#### Finding 1: `Cost::Mana` overwrites where `Cost::PayLife` accumulates

**STATUS: RESOLVED (`scutemob-90` fix phase).** Empirically verified latent (grepped
every `Cost::Sequence` def for two `Cost::Mana` components in the same sequence — zero
hits). Fixed as directed: `mana_ability_cost_components`'s `Cost::Mana` arm now returns
`false` (declines to lower) when `acc.mana_cost` is already `Some`, rather than
overwriting. Pinned by a new negative control in
`sr34_gates_are_not_vacuous` (T13): a synthetic `Cost::Sequence([Mana({1}), Tap,
Mana({1})])` def registers 0 mana abilities / 1 activated ability.

**Severity**: MEDIUM (latent — no live victim in the current corpus)
**File**: `crates/engine/src/testing/replay_harness.rs:3652-3659`
**CR Rule**: 601.2h — *"The player pays the total cost… Partial payments are not allowed."*

```rust
Cost::Mana(m)     => { acc.mana_cost = Some(m.clone()); true }   // overwrite
Cost::PayLife(n)  => { acc.life_cost += n;              true }   // accumulate
```

**Issue**: `walk` recurses through `Cost::Sequence` accumulating components, but the two
scalar components disagree about what "accumulate" means. `Cost::Sequence([Mana({1}), Tap,
Mana({1})])` yields `mana_cost = {1}`, not `{2}` — a **silent partial payment**, contrary to
CR 601.2h. No def in the corpus has two `Cost::Mana` components today, so there is no live
victim, and I could not execute a probe to confirm exhaustively.

This is worth a MEDIUM despite being latent because it is *the exact defect class SR-34 exists
to eliminate* — a cost component that is present in the data, looks paid, and is not — reintroduced
inside the very predicate written to eliminate it, with no gate over it. `mana_ability_cost_components`
is where a future author's `Cost::Sequence` lands, and the failure is silent in both directions
(no error, no `debug_assert`, right-looking `ManaAbility`).

**Fix**: make the two symmetric. Either merge (`acc.mana_cost = Some(merge(existing, m))`) or —
cheaper and more honest, since no card needs it — return `false` when `acc.mana_cost.is_some()`,
so a two-mana-component cost declines to lower and stays on the stack rather than under-charging.
Add a line to T13's negative controls pinning whichever is chosen.

#### Finding 2: `requires_tap: false` is newly reachable via the lowering and wholly untested

**STATUS: RESOLVED (`scutemob-90` fix phase), option (b) — close the seam rather than
prove it.** Empirically verified the "wholly untested" claim: grepped `requires_tap:
false` across `crates/engine/tests/`; every hit is an `ActivationCost` literal or an
`AddManaAbility`/`LayerModification` grant, none is a `ManaAbility` produced by
`mana_ability_lowering`. `mana_ability_cost_components` now returns `None` when
`!acc.requires_tap` after the walk, so a cost with no `Cost::Tap` component declines to
lower at all — the three affected defs (Elvish Spirit Guide, Simian Spirit Guide, Food
Chain, all already non-`Complete`) revert to their pre-SR-34 stack-using behaviour.
Verified via `git diff main..HEAD` on `mana_ability_cost_components`'s callers that no
`Complete` def's cost lacks `Cost::Tap` (all Signets/horizon lands/filter lands/
Magnifying Glass include it). Pinned by a new negative control in T13.

**Severity**: MEDIUM (no live victim — all three defs are non-`Complete`)
**File**: `crates/engine/src/testing/replay_harness.rs:3729`; consumed at `crates/engine/src/rules/mana.rs:173`
**CR Rule**: 106.12 (the `{T}` predicate), 605.1a

**Issue**: Before SR-34, every one of `try_as_tap_mana_ability`'s five return sites hardcoded
`requires_tap: true`, and the gate only admitted `Cost::Tap` — so **nothing in the codebase had
ever produced a `requires_tap: false` mana ability from a def**. `sr34-affected-defs.md` §"Surprising
things" item 1 says exactly this and warns: *"The `false` path is unexercised and should be treated
as unproven, not as existing capability (the `feedback_verify_full_chain` trap)."*

SR-34 makes it reachable: `mana_ability_lowering` now overwrites the hardcoded `true` with
`components.requires_tap`, and `Cost::Mana`-only / `Cost::SacrificeSelf`-only costs both lower.
The reconciliation confirms this fired — Elvish Spirit Guide, Simian Spirit Guide and Food Chain
"DID start registering as (free, repeatable, stackless) mana abilities post-widening."

In that path `handle_tap_for_mana` step 6 is skipped entirely, so **there is no tapped-status check
and no exhaustion mechanism** — a `requires_tap: false` mana ability is unboundedly repeatable
within a single priority window, now with no stack and no priority window at all.

I verified the mitigation and it holds: all three defs are non-`Complete`
(`elvish_spirit_guide.rs:34` KnownWrong, `simian_spirit_guide.rs:31` Partial, `food_chain.rs:35`
KnownWrong), so `validate_deck` rejects them per invariant #9. **No live victim.** The
reconciliation's reasoning here is sound and its scope call is right.

What is missing is coverage. Grep of `crates/engine/tests/` finds no test exercising a
`requires_tap: false` **`ManaAbility` produced by the lowering** (the `requires_tap: false` hits
are `ActivationCost` literals and `LayerModification::AddManaAbility` grant literals — a different
path). So a newly-reachable engine path that the task's own roster doc flagged as unproven ships
with zero tests and nothing stopping a future `Complete` def from landing on it.

**Fix**: pick one. (a) Add a T13-style control: a synthetic `Cost::Mana`-only or
`Cost::SacrificeSelf`-only mana-producing def, assert it lowers with `requires_tap: false` and
that `TapForMana` pays and produces correctly. (b) Or make `mana_ability_cost_components` return
`None` when `!acc.requires_tap` — declining to lower a no-tap cost keeps the unproven path
unreachable and costs nothing (the three affected defs are already non-`Complete` and stay so).
(b) is the more defensible default given the roster doc's own warning. Either way, record the
choice.

#### Finding 3: A claim the task proved false is left standing inside a gate

**STATUS: RESOLVED (`scutemob-90` fix phase).** Rewrote `no_complete_def_uses_the_add_mana_choice_stub`'s
doc comment to state the truth: `AddManaAnyColor` produces the identical `{C}` as
`AddManaChoice`, this gate blocks only the latter, and the omission is an acknowledged,
scope-bounded inconsistency (SF-11) rather than a principled asymmetry. Gate behaviour
unchanged, per the reviewer's explicit instruction not to extend it here.

**Severity**: MEDIUM
**File**: `crates/engine/tests/core/effect_choose_gate.rs:136-140`
**Oracle/Source**: the task's own `sr34-engine-findings-2026-07-17.md` §SF-11

The gate's doc comment still reads, verbatim and unedited:

> Unlike `AddManaAnyColor` — which escapes into a real `ManaAbility` with `any_color: true` via
> `try_as_tap_mana_ability` and so never reaches that arm — `AddManaChoice` is not recognised
> there, so its users always route through the stack and into the colorless arm. **That asymmetry
> is why sharing the match arm is not the harmless simplification it looks like.**

SF-11 is this task's own finding and it says, correctly and with probe evidence: **"The asymmetry
does not exist."** `handle_tap_for_mana` step 8 (`mana.rs:317-321`) adds
`ManaColor::Colorless` and `Effect::AddManaAnyColor` adds `ManaColor::Colorless` — the same one
colorless mana on both paths. I verified both halves in source; SF-11 is right.

**Issue**: the task correctly declined to *change the gate's behaviour* (extending it to
`AddManaAnyColor` demotes defs beyond the roster and moves headline coverage — a legitimate
scope call, consistent with EF-13's deferral). But it conflated that with **correcting a comment
it had just proven false**, which costs nothing, demotes nothing, and moves no coverage. The
result is a false justification sitting in a checker, which is precisely the defect class this
campaign keeps re-finding (CLAUDE.md: SR-33's marker sweep found 42% of notes wrong; the brief
notes two of SR-34's own five findings were "false claims written INTO existing gates"). The task
recognised the pattern for SF-8 — it annotated `mana_filter.rs:299-316` and
`effect_choose_gate.rs:255-275` rather than leaving them silent — and then reproduced it here.

Note the plan anticipated this exact move at §9: *"Update the doc comment to record what the
exclusion became rather than deleting it silently."*

**Fix**: rewrite lines 136-140 to state the truth: `AddManaChoice` and `AddManaAnyColor` produce
the identical wrong result (one colorless — CR 106.1a/106.1b: colorless is a mana *type*, not a
colour), the gate blocks only the former, this is an **acknowledged inconsistency bounded by
scope, not a principled asymmetry**, and it is tracked as SF-11 in
`memory/card-authoring/sr34-engine-findings-2026-07-17.md`. Do not change the gate's behaviour.

#### Finding 4: SF-12's blind spot is undocumented in the gate it blinds

**STATUS: RESOLVED (`scutemob-90` fix phase).** Added exclusion (3) to
`printed_tap_mana_colors`'s doc comment: "Add one mana of any color" is invisible to
this gate on both sides (parser needs a `{` the clause lacks; `registered_colors` reads
`produces.keys()`, empty when `any_color: true`), citing CR 106.1a/106.1b and SF-11/
SF-12 by name. Also added a doc note on `every_complete_land_registers_each_printed_tap_mana_color`
acknowledging the misleading name (iterates all `Complete` defs, not just lands) without
renaming it (the name is quoted by several `memory/` docs; a rename should update those
in the same change, out of this fix phase's scope).

**Severity**: MEDIUM
**File**: `crates/engine/tests/core/effect_choose_gate.rs:249-275` (`printed_tap_mana_colors` doc) and `:373` (`every_complete_land_registers_each_printed_tap_mana_color`)
**Plan**: §9 — *"The `AddManaScaled` blind spot must be written into that gate's doc… A gate whose blind spot is undocumented is how SF-8 survived; do not let it survive twice."*

**Issue**: The doc comment enumerates exactly two exclusions — (1) granted abilities, (2)
dynamic-amount clauses (SF-8) — both well written. A grep of the whole file for
`any color|any_color|SF-12|SF-11` returns **one hit**, at line 136, and that is Finding 3's
false claim in a *different* test's doc.

So SF-12 — which this task discovered, verified against both halves of the chain, and rated
MEDIUM — is documented **only** in the findings doc. The gate itself is silent about it. Per
SF-12 the gate is blind to every "any color" card for two independent reasons (the parser needs a
`{` that "Add one mana of any color" lacks, so `printed` is empty and the card is `continue`d at
`:382`; and `registered_colors` reads `ma.produces.keys()`, which is empty when `any_color: true`).
Mana Confluence, Command Tower and Birds of Paradise all pass this gate **vacuously**.

This is the same structural mistake as Finding 3 and it is the one the plan named explicitly.
The task honoured the instruction for the blind spot it *inherited* (SF-8) and not for the one it
*found* (SF-12) — which is the harder case and the reason the instruction exists.

**Fix**: add exclusion (3) to `printed_tap_mana_colors`'s doc comment: an "Add one mana of any
color" clause is invisible to this gate on both sides (parser produces no colour; `any_color: true`
leaves `produces` empty), so such cards pass vacuously while producing `{C}` — CR 106.1a/106.1b —
tracked as SF-11/SF-12; cite the "whichever lands first must not land alone" note. Consider also
renaming the misleading `every_complete_land_…` (it iterates **all** Complete defs, not just
lands — which is correct and is why the Signet coverage claim at `:245` is true, but the name
says otherwise).

#### Finding 5: the `any_color` demotion is roster-bounded, leaving the corpus self-contradictory

**STATUS: RESOLVED (`scutemob-90` fix phase).** Verified by direct read: both
`birds_of_paradise.rs:38` (`Completeness::Complete`, explicit) and `command_tower.rs:21`
(`Complete` by default, no marker) use `Effect::AddManaAnyColor` and are confirmed
Complete today. **Not demoted** — the scope call is upheld per the reviewer's explicit
instruction. `sr34-roster-reconciliation.md` §1 amended with a scope-bound paragraph
naming both cards as calibration cases outside the 27-def roster. SF-11 amended with a
"Known live `Complete` victims outside the SR-34 roster" paragraph naming both by file
and line.

**Severity**: MEDIUM (pre-existing defect; correctly filed as SF-11 — the finding is about the *presentation*)
**Files**: `crates/card-defs/src/defs/birds_of_paradise.rs:38`, `crates/card-defs/src/defs/command_tower.rs:21`
**CR Rule**: 106.1a/106.1b — colorless is a mana type, not a colour
**Invariant**: #9 — a `Complete` card must not silently produce wrong game state

**Issue**: SR-34 demoted 8 defs to `KnownWrong` on the grounds (reconciliation §1, verbatim)
that *"'Add one mana of any color' producing `{C}` is not a lesser version of the right answer…
`{C}` is outside the legal option set entirely."* That reasoning is correct and CR-grounded.

It is also true of cards the task left `Complete`:

- **`birds_of_paradise.rs:38`** — `completeness: Completeness::Complete`, ability
  `Effect::AddManaAnyColor` (`:18`). Oracle: *"{T}: Add one mana of any color."* Produces `{C}`.
- **`command_tower.rs:21`** — no marker, so `Complete` by default; `Effect::AddManaAnyColor` (`:14`).

These are two of the most-played cards in the format, they are **deck-legal today**, and they ship
the identical defect for which Mana Confluence (`mana_confluence.rs:20`) was demoted with the note
*"this is wrong state, not an omitted clause."* The corpus now returns two contradictory verdicts
on one bug, and `sr34_roster_markers_match_the_reconciliation` pins the demoted half as a
requirement while nothing pins the other half.

**The scope call is defensible** — extending the demotion corpus-wide moves headline coverage and
needs its own roster, exactly as EF-13 was deferred for the same reason, and SF-11 files it with a
correct fix shape. The finding is that **the reconciliation does not say so.** §1 introduces the
taxonomy as *"Stated once, applied uniformly"*, which is true only within the 27-def roster and
reads, at corpus scope, as a claim the artifact does not support. A future reader diffing
Mana Confluence against Birds of Paradise finds an unexplained contradiction rather than a
recorded, bounded decision.

**Fix**: (a) amend `sr34-roster-reconciliation.md` §1 to state the bound explicitly — the taxonomy
was applied to SR-34's 27-def roster only; the same defect demonstrably persists in `Complete`
defs outside it (name Birds of Paradise and Command Tower as the calibration cases), tracked as
SF-11. (b) Add a sentence to SF-11 naming them as known live `Complete` victims, so its roster is
partially built rather than merely asserted to exist — the same upgrade SF-9 correctly received
when Staff of Compleation and Voldaren Estate were named. Do **not** demote them in this task.

#### Finding 6: T11 cannot fail except on re-divergence

**STATUS: RESOLVED (`scutemob-90` fix phase).** Doc comment rewritten to state that the
identity holds by construction (partition guard) and that lowering correctness is
pinned separately by T1/T10/T13 and `sr34_certified_defs_produce_exactly_their_printed_mana`.

**Severity**: LOW
**File**: `crates/engine/tests/primitives/primitive_sr34_composite_mana_costs.rs:585-623`

`is_tap_mana_ability_agrees_with_the_lowering` asserts
`mana_abilities.len() + activated_abilities.len() == activated_count`. Both loops in
`enrich_spec_from_def` (`:2126`, `:2149`) now call the *same function*, and the second is the exact
negation of the first — so every `Activated` ability lands in exactly one list **by construction**.
The identity is a tautology under the current structure and would hold even if
`mana_ability_lowering` returned `None` for every input (everything falls into
`activated_abilities`; the sum still balances).

That is fine — it is precisely the re-divergence guard the plan's §3 step 5 asked for, and it will
redden the day someone re-splits the predicate. The `checked > 100` denominator guard is good
practice. The issue is only that the doc comment reads as though it verifies the *lowering*, when
it verifies the *partition*.

**Fix**: adjust the doc to say what it pins — that the two lists are complementary by construction
and this test fails only if the single predicate is re-split — and note that the correctness of
the lowering itself is pinned by T1/T10/T13 and `sr34_certified_defs_produce_exactly_their_printed_mana`.

#### Finding 7: dead `cards_pos` in T13

**STATUS: RESOLVED (`scutemob-90` fix phase).** Deleted both lines
(`let mut cards_pos = all_cards();` / `cards_pos.push(bare_tap_def.clone());`).

**Severity**: LOW
**File**: `crates/engine/tests/primitives/primitive_sr34_composite_mana_costs.rs:825-826`

```rust
let mut cards_pos = all_cards();
cards_pos.push(bare_tap_def.clone());   // never read afterwards
```

The positive control goes through `defs_pos` (the `HashMap`), not a `CardRegistry`, so `cards_pos`
is built, mutated and dropped. Harmless, but it implies a registry round-trip that does not
happen. **Fix:** delete both lines.

#### Finding 8: touched defs were not `rustfmt`'d

**STATUS: RESOLVED (`scutemob-90` fix phase).** Ran `rustfmt --check` by name over all
29 defs this task's full diff touches (`git diff --name-only main..HEAD -- crates/card-defs/`),
not just the three named. Confirmed the SF-7 mechanism directly: `rustfmt` (both plain
and `--check`) reports these files as **already formatted** even though the
`once_per_turn`-drift indentation is visibly wrong — the `vec!` macro body containing
the long struct literal exceeds `max_width` and rustfmt silently leaves it verbatim
rather than erroring. Hand-fixed the drift (re-indented `once_per_turn: false,` and its
closing brace/bracket to match sibling fields) in 15 files:
`ashnods_altar.rs`, `cabal_stronghold.rs`, `crypt_of_agadeem.rs`, `druids_repository.rs`,
`gemstone_array.rs`, `goldhound.rs`, `maelstrom_of_the_spirit_dragon.rs`,
`magnifying_glass.rs`, `mana_confluence.rs`, `phyrexian_altar.rs`, `phyrexian_tower.rs`,
`secluded_courtyard.rs`, `staff_of_compleation.rs`, `unclaimed_territory.rs`,
`voldaren_estate.rs`. The other 14 touched defs were checked and found already
correctly formatted at the lines this task's diff actually edits (confirmed via
`git diff main..HEAD` per file — the drift pattern is pre-existing and this task's own
edits, where they touch the same struct shape, are correctly indented).

**Severity**: LOW
**Files**: `crates/card-defs/src/defs/magnifying_glass.rs:24,37`, `mana_confluence.rs:16-19`, `goldhound.rs:30`

Plan §3 step 8 and the §10 checklist both require: *"Run `rustfmt` over each touched def **by
name**, and re-check the file afterwards"* — because per SF-7/SR-35 `cargo fmt` reaches **zero**
card defs (`include!`/`#[path]` are invisible to rustfmt), so CI's `fmt --check` green says
nothing about them. The touched defs show obviously unformatted bodies (`once_per_turn: false,` at
12 spaces inside a 16-space block; `}],` at column 0 in `mana_confluence.rs:19`).

The drift is **pre-existing** (a prior mass-insert of `once_per_turn`, not SR-34's doing) and the
task's "fmt clean" status claim is technically true — which is exactly SR-35's point. But the
checklist item was not done. **Fix:** run `rustfmt` by name over the touched defs and re-inspect
(rustfmt exits 0 while abandoning an over-`max_width` macro body). Low priority; SR-35 is the real
remedy.

## What I verified as correct (and tried to break)

| Claim | Verdict | Evidence |
|---|---|---|
| CR 605.1a criterion 1 (no target) | **Correct** | `mana_ability_lowering:3720` checks `targets.is_empty()` **first**. This also closes the latent hazard `sr34-affected-defs.md` §7 named: pre-SR-34 the loop dropped `targets` on the floor and Deathrite Shaman was safe only by luck. Now safe by construction. |
| CR 605.1a criterion 2 (could add mana) | **Correct** | `try_as_tap_mana_ability` gates the effect shape. |
| CR 605.1a criterion 3 (not loyalty) | **Correct, structurally** | Verified the doc's claim rather than trusting it: `LoyaltyAbility` is a **separate** `AbilityDefinition` variant (`card_definition.rs:470`), so the `if let AbilityDefinition::Activated` match at `:2119` cannot see one. Unreachable, not merely unhandled. |
| CR 119.4b short-circuit | **Correct + exercised** | `mana.rs:162` guards on `life_cost > 0`. T7 (`zero_life_cost_ability_is_legal_at_negative_life`) taps a Forest at life **−1**. Trace: remove the guard → `0 < -1` is false → still passes. So T7 as written does **not** catch the unguarded `>=` form… but the code is correct anyway, because `u32` `life_cost: 0` with `life_total: -1` gives `-1 < 0` → **true** → `InsufficientLife`. Re-traced: the unguarded form *does* redden T7. Test is real. |
| CR 119.4 `>=` boundary | **Correct + exercised** | `mana.rs:164` is `life_total < life_cost` → legality is `>=`. T5 pays 1 life at exactly 1 life and asserts life → 0. A `>` boundary reddens it. |
| CR 106.12 / 106.12a / 106.12b | **Correct, and the plan's reasoning verified against the CR** | MCP 106.12: *"includes the {T} symbol in its activation cost"* — **includes**, not **is**. So steps 7b/8/10's `requires_tap` gating is right untouched. T12 proves the *consequence* (Nyxbloom triples a Signet → W=3/R=3) and the trigger test proves 106.12a off a horizon land. These are the highest-value tests in the suite and they were not skipped. |
| CR 605.3b (no stack) | **Proven, not shaped** | T3, T8 and the certified test all assert `stack_objects().is_empty()` **after a real `process_command`**, and T8 additionally proves the stack held *only* the funded spell. This is not the SF-5 anti-pattern. |
| SR-28 snapshot invariant | **Holds** | 6b (`mana.rs:210-227`) mutates only the controller's pool and life total — neither the source's characteristics nor its zone — so `source_pre_cost_chars` at `:241` is byte-identical either side. The extended comment at `:200-209` states the boundary correctly. |
| T10 non-vacuous (`AddManaScaled` exclusion) | **Confirmed by trace** | `cabal_coffers.rs` has **exactly one** ability. Delete `mana_ability_lowering:3725` → cost `Sequence[Mana({2}),Tap]` passes `mana_ability_cost_components`, `try_as_tap_mana_ability(AddManaScaled)` returns `Some(produces={B:1})` → lowering succeeds → `mana_abilities` 0→1 **and** `activated_abilities` 1→0. **Both** T10 assertions flip. The exclusion is load-bearing and T10 pins it. |
| T13 non-vacuous | **Confirmed by trace** | Negative control uses `Cost::Sequence([DiscardCard, Tap])`; adding a `DiscardCard => true` arm flips both assertions. |
| Single predicate for both lists | **Correct** | `:2126` and `:2149` call `mana_ability_lowering`. The `AddManaMatchingType` divergence is structurally dead. |
| `PROTOCOL_VERSION` 2→3 | **Complete, append-only** | `:75` bumped, History line `- 3:` at `:68-74`, `PROTOCOL_HISTORY` row **appended** at `:154-159`, baseline row 2 (`:146-153`) **unedited**, tail fingerprint `c23d09a7…` == `PROTOCOL_SCHEMA_FINGERPRINT` (`:92`). |
| `HASH_SCHEMA_VERSION` 40→41 | **Complete, append-only** | `hash.rs:384` bumped; row 41 appended at `:474-482` with both digests; rows 39/40 unedited. Sentinel sweep **complete**: grep for `HASH_SCHEMA_VERSION, 40` returns **zero** source hits (only the plan doc). SR-27's half-done-bump failure mode avoided. |
| `HashInto for ManaAbility` | **Correct** | `hash.rs:1416-1417` feeds both new fields; SR-19's field gate not allowlisted around. |
| `InsufficientLife` | **Correct** | `error.rs:85` — typed, carries `required`/`actual`, CR-cited. Off the SR-8 wire (errors are returned, not embedded; `PROTOCOL_ROOTS` unchanged). |
| `ManaPool::spend` return discarded | **Non-issue** | `player.rs:185` returns `()`. |
| Oracle match, fixed/promoted defs | **Correct (MCP-verified)** | Magnifying Glass def now `{T}: Add {C}` + `{4},{T}: Investigate` — matches Scryfall exactly; the old `{1}`/`{3}` was a genuine def bug and catching it is a real win. Fiery Islet `{T}, Pay 1 life: Add {U} or {R}` + `{1},{T},Sac: Draw` — matches; two per-colour arms is the correct `tainted_field` pattern per `memory/decisions.md`. Goldhound matches. |
| Script `stack/099` re-cost | **Correct** | Now `{4}` with `ability_index: 0`, pool 6 colorless, and a note explaining the SR-34 index shift. Verified against oracle, not against the def — the re-cost is right *because* the def was wrong, and the script had indeed been generated from the broken def. Good catch; SF-6 handled here. |
| `mana_filter.rs` module doc | **Correct — plan risk 4 avoided** | Explicitly states filter lands' `{W/B}` remains **unenforced** (`can_spend` ignores `hybrid`) rather than claiming they are fixed. This was the named trap and the task did not fall in it. |
| SF-8 blind spot documented | **Yes, in both gates** | `mana_filter.rs:299-316` and `effect_choose_gate.rs:255-275`. Decided explicitly and written down, per §9. Contrast Finding 4. |
| Markers truthful | **Yes (12 spot-checked)** | Each names the real blocker, cites CR, and cites probe evidence (`mana_confluence.rs:20`, `goldhound.rs:33`, `phyrexian_altar.rs:29`, `cabal_coffers.rs:38`). Materially better than the 42%-wrong baseline SR-33 found. |
| Scope discipline (SF-8/9/10/11/12 filed not fixed) | **Correct, with one exception** | All five are genuinely separable and each has a fix shape. The exception is Finding 3: SF-11's *comment* was provable-false and free to fix; leaving it is a half-measure. |

## CR Coverage Check

| CR Rule | Implemented? | Tested? | Notes |
|---------|-------------|---------|-------|
| 605.1a | Yes | Yes | T1, T10, T13, `sr34_certified_defs…`; all three criteria enforced |
| 605.3a | Yes | Yes | T8, T9 — pool persists across Commands; no recursion needed |
| 605.3b | Yes | Yes | T3, T8, certified test — real activation, stack asserted empty |
| 605.3c | Vacuous | n/a | Resolves within the activating Command; no activated-but-unresolved window |
| 601.2h | Partial | No | **Finding 1** — a second `Cost::Mana` is a silent partial payment |
| 602.2b/2c | Yes | Yes | Cost paid before production; sacrifice after the SR-28 boundary |
| 118.3 / 118.3a | Yes | Yes | T4 (`InsufficientMana`, source untapped) |
| 118.3b | Yes | Yes | T5, T6 |
| 119.4 | Yes | Yes | T5 — `>=` boundary at exactly 1 life |
| 119.4b | Yes | Yes | T7 — `life_cost: 0` at life −1 |
| 106.12 | Yes | Yes | `requires_tap` correctly left as the predicate |
| 106.12a | Yes | Yes | trigger test off Fiery Islet |
| 106.12b | Yes | Yes | T12 — Nyxbloom triples a Signet |
| 106.1a/106.1b | Yes (taxonomy) | Partial | `sr34_roster_markers…` pins 17; **Finding 5** — roster-bounded |
| 732 | Free | Yes | `process_command` by-value; T4/T6 assert via a pre-command clone |

## Card Def Summary

| Card | Oracle Match | TODOs | Game State Correct | Notes |
|------|-------------|-------|-------------------|-------|
| Fiery Islet / Nurturing Peatland / Silent Clearing | Yes | 0 | Yes | Promoted to `Complete`; per-colour arms, life paid, stackless. The headline yield, correctly earned. |
| Magnifying Glass | Yes (fixed) | 0 | Yes | Def bug found and fixed; stays `Complete` on merit. Finding 8 (fmt). |
| 7 Signets, Darkwater Catacombs, Viridescent Bog | Yes | 0 | Yes | Certified by pool delta; errata note on the two lands verified. |
| Cabal Coffers / Stronghold, Crypt of Agadeem | Yes | 0 | Amount right, mechanism wrong | `Partial` — correct call; SF-8 seam documented in the marker. |
| Ashnod's Altar, Phyrexian Tower, Temple of the Dragon Queen | Yes | 0 | Right mana, wrong mechanism | `Partial` — correct. |
| Mana Confluence, Staff of Compleation, Voldaren Estate, Phyrexian Altar, Goldhound, Druids' Repository, Gemstone Array, Three Tree City, Maelstrom, Secluded Courtyard, Unclaimed Territory | Yes | 0 | No — ships `{C}` | `KnownWrong` — correct per CR 106.1b. Finding 8 on some. |
| **Birds of Paradise, Command Tower** | Yes | 0 | **No — ships `{C}`** | **Finding 5** — `Complete`, identical defect, outside the roster. |
| Elvish/Simian Spirit Guide, Food Chain | No (pre-existing) | 0 | No | Non-`Complete`; newly lower to no-tap mana abilities — **Finding 2**. |

## Previous Findings

First review of SR-34. No prior review file.

## Recommendation

Ship after Findings 3 and 4 — both are doc-comment edits inside gates, zero behaviour change,
zero coverage movement, and both are the campaign's most-repeated defect (a false or silent
claim in a checker). Finding 5 is a one-paragraph amendment to the reconciliation plus two
sentences on SF-11. Findings 1 and 2 are the only code changes and both are latent with no live
victim; Finding 2 in particular has a zero-risk resolution (decline to lower a no-tap cost).
Findings 6-8 are optional cleanup.

The engine change itself is correct, well-placed, well-reasoned about the CR, and better tested
than any primitive batch I have reviewed in this campaign. The plan's two riskiest traps —
the `AddManaScaled` capture and the "filter lands look fixed" false note — were both
identified and both avoided.
