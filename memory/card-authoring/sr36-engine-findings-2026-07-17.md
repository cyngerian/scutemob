# Engine findings from SR-36 (`scutemob-92`) — for the next SR

Filed, **not fixed** here, per the SR protocol. SR-36's declared surface was SF-8 + SF-9;
these fall outside it. The three MEDIUMs from `memory/primitives/pb-review-sr36.md` were
fixed in-task (commit `530ba541`) and are not repeated here.

Prior findings that remain open and are **not** superseded by SR-36: **SF-10** (`ManaAbility`
has no `activation_condition` — Tainted Field taps with no Swamp), **SF-11** (`any_color`
produces colorless, CR 106.1a/106.1b), **SF-12** (the colour gate is structurally blind to
every "any color" land), **EF-13** (105 `partial` defs register no behaviour and are `Inert`
by taxonomy). All four are in `memory/card-authoring/sr34-engine-findings-2026-07-17.md` /
`sr33-engine-findings-2026-07-17.md`. SR-36 touched none of them.

---

## SG-1 — `LegalActionProvider` ignores `ManaAbility::life_cost` and `ActivationCost::life_cost`

**Severity: MEDIUM. Introduced-in-effect by SR-36, though the code is unchanged.**

`crates/simulator/src/legal_actions.rs:399` builds the bot's legal-action list without
checking affordability of a life cost. Before SR-36 that was harmless for non-mana
abilities: the life cost was silently dropped, so *every* activation was affordable and the
provider's optimism was accidentally correct. SF-9 made the cost real and
`handle_activate_ability` now returns `GameStateError::InsufficientLife`, so the provider
can offer a bot an action the engine will reject.

The same gap exists for SR-34's `ManaAbility::life_cost` (horizon lands, Mana Confluence) —
that one has been live since SR-34 and is not new.

Not observed as a test failure: no suite test drives the simulator to a life total below a
card's life cost. The failure needs a bot at low life holding a fetchland or a horizon land
— reachable in a real game, not in the current tests.

**Fix shape**: mirror `handle_tap_for_mana`'s step 5b / `handle_activate_ability`'s new
check in the provider — `life_total >= life_cost`, short-circuited on `life_cost > 0` per
CR 119.4b. Cheap; the reason it is filed rather than fixed is that the simulator is outside
this task's surface and unreviewed here.

---

## SG-2 — `try_as_tap_mana_ability`'s non-`Controller` refusal is untested and its rationale is arguable

**Severity: LOW.**

SR-36 added a guard refusing to lower an `Effect::AddManaScaled` whose `player` is not
`PlayerTarget::Controller` (`testing/replay_harness.rs`), because the stackless
`TapForMana` path always pays the activating player. **No card in the corpus exercises it**
— all 9 SF-8 roster rows use `Controller` (verified via `all_cards()`), so the guard is
defensive and its `return None` branch is dead today. That is cheap correctness insurance
for a real hazard, but it is unpinned: nothing would catch its deletion.

Separately, its comment calls staying on the stack "correct-but-slow, not wrong". That
undersells it against SR-33's own position: CR 605.3b says a mana ability does not use the
stack, so a mana ability that does grants opponents a priority window it should not — which
is precisely why SR-33 rewrote 88 dual lands and why Cabal Coffers was `Partial` rather than
`Complete`. The phrasing should match SR-33's.

**Fix shape**: a synthetic `CardDefinition` in a test with `player: PlayerTarget::Opponent`,
asserting it registers 0 mana abilities and 1 activated ability; reword the comment.

---

## SG-3 — `registered_colors`'s `scaled_amount.is_none()` filter is the weaker of two symmetric fixes

**Severity: LOW. Reviewer's Finding 6; recorded rather than actioned.**

SR-36 fixed an asymmetry in `every_complete_land_registers_each_printed_tap_mana_color`
(`tests/core/effect_choose_gate.rs`) by dropping scaled abilities from the **registered**
side, matching the parser's existing "for each" / "equal to" exclusion on the **printed**
side. Verified non-vacuous: perturbing `swamp.rs` to also add `{G}` still fails with
`invented [Green]`.

The reviewer's point stands: the *stronger* fix is to include the scaled clause's colour on
**both** sides — the parser knows the colour (it parses `{B}` before hitting the "for each"
tail), and the registered side has it in `produces`. That would let the gate check Cabal
Stronghold's `{B}` rather than ignore it, narrowing the exclusion to amounts only, which is
all it was ever meant to cover. Both sides currently drop the whole clause, so a scaled
ability that registered an outright *wrong colour* would pass.

Not done here: it widens a gate beyond the defect SR-36 was chartered to fix, and the
colour it would newly check is already checked by activation for all 9 roster rows. Pairs
naturally with SF-12, which is the same gate's other blind spot.

---

## Not a finding — recorded so it is not re-filed

**The SF-8 roster is smaller than SR-34 filed it.** `memory/card-authoring/sr34-engine-findings-2026-07-17.md`
§SF-8 speculated that "by the same shape Everflowing Chalice, Elvish Guidance, Brightstone
Ritual, Battle Hymn, Black Market" were victims, while explicitly flagging them "re-check
each against the registry — not re-verified in this task". Re-checked against `all_cards()`:
**none of them is a victim.** Battle Hymn, Brightstone Ritual and Jeska's Will carry
`AddManaScaled` on an `AbilityDefinition::Spell` (spells resolve through the stack, where
the dynamic amount was always evaluated correctly); Black Market carries it on an
`AbilityDefinition::Triggered` (a triggered mana ability, CR 605.1b, which resolves through
`execute_effect` — also always correct); Everflowing Chalice and Elvish Guidance mention
`AddManaScaled` only in *aspirational* marker notes describing abilities that are not
authored yet. The real roster is 9 ability rows across 6 cards, all fixed by SR-36.

The caveat was warranted, and the general lesson is the one CLAUDE.md already records: a
roster derived from "by the same shape" reasoning is a hypothesis. This one was tested by
enumerating `all_cards()` and filtering on `AbilityDefinition::Activated` + the effect
variant — not by grepping source, which would have matched all five and re-filed a
non-defect.
