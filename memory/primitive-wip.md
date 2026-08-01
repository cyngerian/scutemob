# Primitive batch WIP — PB-DX6

**Batch**: PB-DX6 — the last two unflattened mana-cost payment sites
**Seeds**: OOS-RS2-1 + OOS-DP4-1 (`memory/primitives/seed-rerank-2026-07-27.md` §4 + the PB-DX6
dispatch brief at that file's `:994`)
**Task**: `scutemob-172` · **Branch**: `feat/pb-dx6-the-last-two-unflattened-mana-cost-payment-sites-oos-`
**Phase**: plan

> Prior batch (PB-DX5, `scutemob-170`) shipped; its result summary lives in
> `memory/primitives/pb-plan-DX5.md` + `pb-review-DX5.md` and in CLAUDE.md's Current State.

## Premise — re-verified on this branch before planning (2026-08-01)

All four points of the brief hold. Cited by symbol, not line, per the PB-DX2 lesson.

1. **`rules/engine.rs::handle_turn_face_up`** derives `mana_cost` from the card def and then calls
   `player_state.mana_pool.can_spend(&mana_cost, None)` / `.spend(&mana_cost, None)` on the **raw,
   unflattened** cost. This is true for **all three** `TurnFaceUpMethod` branches — `MorphCost`,
   `DisguiseCost` and `ManaCost` — not only the `ManaCost` branch the brief names. All three share
   one payment block.
2. **`crates/card-types/src/state/player.rs::debug_assert_flattened`** is a bare `debug_assert!`.
   Its own doc block already says, in as many words, that it "fires NEVER" in release and that
   "release correctness rests entirely on the three call sites … actually flattening".
3. **`kitchen_finks`** is `Completeness::Complete` with
   `hybrid: [ColorColor(Green, White), ColorColor(Green, White)]` and `generic: 1`.
4. **`Command::DeclareAttackers`** (`rules/command.rs`) carries `enlist_choices` and
   `exert_choices` only — no payment-choice fields. `rules/combat.rs` diverts any
   `CantAttackYouUnlessPay` whose `cost_per_creature` has a non-empty `hybrid`/`phyrexian` **or
   `x_count > 0`** into `unpayable_tax_defenders`, and rejects the declaration outright if a
   declared attacker targets such a defender (PB-DP4's hard rejection, which cites OOS-DP4-1 in
   its own error string).

## Roster — enumerated from `all_cards()` (1,804 defs), not grepped (SR-36)

Measured by a throwaway `#[test]` walking `all_cards()`; the scratch file was deleted after the
run and its findings are recorded here. A permanent version of this enumeration is a deliverable
of the implement phase.

**Site 1a — manifest / cloak flip (`TurnFaceUpMethod::ManaCost`, pays `def.mana_cost`).**
A manifested or cloaked card is a face-down creature that flips for its own printed mana cost, so
the roster is every **creature** def whose printed cost carries a hybrid or Phyrexian pip:
**5 defs**, of which **3 are `Complete` and therefore deck-legal**:

| def | pips | completeness |
|---|---|---|
| Kitchen Finks | `{G/W}{G/W}` | **Complete** |
| Blade Historian | hybrid | **Complete** |
| Boggart Ram-Gang | hybrid | **Complete** |
| Deathrite Shaman | hybrid | `known_wrong` (deck-illegal) |
| Vexing Shusher | hybrid | `partial` (deck-illegal) |

**The brief named one card; the live-wrong roster is three.** The sixth-consecutive-batch caveat
applies in the usual direction — the published roster undercounted.

**Site 1b — `MorphCost` / `DisguiseCost`.** **0** of the 7 defs carrying
`Morph`/`Megamorph`/`Disguise` have a hybrid or Phyrexian pip in that cost. Both branches share
the defective payment block and are fixed with it, but they are **latent**, not live.

**Site 2 — attack tax.** Exactly **2** defs produce `CantAttackYouUnlessPay`: Propaganda and
Ghostly Prison, both `cost_per_creature = {2}` with empty `hybrid`/`phyrexian` and `x_count: 0`.
**0** defs carry a pipped or X attack tax. Fully **latent**, exactly as briefed.

**Reachability of site 1 (why it is live, not latent).** 3 defs can put a card face down this
way: `cryptic_coat`, `reality_shift`, `write_into_being`.

## Note for the plan — a class the two new fields do NOT close

`combat.rs` funnels `x_count > 0` into the *same* `unpayable_tax_defenders` bucket as the pips.
`hybrid_choices` + `phyrexian_life_payments` cannot make an **X** attack tax payable — that needs
an x-payment field and a CR 601.2b-shaped announcement. Whatever the fix does, the X arm must keep
rejecting, must keep saying so accurately, and the residue should be filed as its own seed rather
than left folded under the rejection message's existing OOS-DP4-1 cite.

## Phases

- [x] premise verified
- [ ] plan
- [ ] implement
- [ ] review
- [ ] fix
- [ ] close
