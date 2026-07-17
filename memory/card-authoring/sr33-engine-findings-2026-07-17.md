# Engine findings from SR-33 (`scutemob-89`) — for SR-34+

Filed, **not fixed** here, per the SR protocol. Each was surfaced while fixing the 88
`Effect::Choose` lands and verified against source; the ones marked
**[empirically proven]** were demonstrated with a live probe, not read off the code.

Ordered by severity.

---

## SF-1 — A mana ability with an additional cost is never registered as a mana ability **[empirically proven]**

**Severity: HIGH.** Same *consequence* as SR-33's EF-1 (CR 605.1a violation: the ability
uses the stack, `TapForMana` cannot find it, it cannot be activated while casting), but a
different and wider root cause — and unlike EF-1 it is an **engine** gap, not a def gap.

`enrich_spec_from_def` (`testing/replay_harness.rs:2115`) only considers an ability for
`try_as_tap_mana_ability` when `matches!(cost, Cost::Tap)`:

```rust
if let AbilityDefinition::Activated { cost, effect, .. } = ability {
    if matches!(cost, Cost::Tap) {              // <-- the whole gate
        if let Some(ma) = try_as_tap_mana_ability(effect) { … }
```

But CR 605.1a makes an ability a mana ability based on **what it does**, not what it costs:
"it doesn't require a target, it could add mana when it resolves, and it's not a loyalty
ability." Nothing about the cost. So every one of these is a mana ability that the engine
treats as a stack-using activated ability:

| Shape | Cards |
|---|---|
| `{1}, {T}: Add {C}{C}` | all ten Signets, Cluestones, Viridescent Bog, Darkwater Catacombs, Magnifying Glass |
| `{T}, Pay 1 life: Add {B} or {G}` | horizon lands (Nurturing Peatland, Silent Clearing, Fiery Islet, …) |
| `{W/B}, {T}: Add …` | all filter lands (already noted as a known limitation in `tests/casting/mana_filter.rs`) |
| `{2}, {T}: Add {B} for each Swamp` | Cabal Coffers, Cabal Stronghold |

`mana_filter.rs` records the filter-land case as an accepted simplification; that note
understates it — this is not specific to filter lands, it is every non-`Cost::Tap` mana
source in the corpus.

Fixing it needs more than widening the `matches!`: `ManaAbility` has no field for an
additional cost (only `requires_tap`, `sacrifice_self`, `damage_to_controller`), and
`handle_tap_for_mana` (`rules/mana.rs`) has no cost-payment step. So this is a real
primitive: `ManaAbility` needs an activation cost and `TapForMana` needs to pay it.

**Not actioned in SR-33**: out of scope (the task is the `Choose` stub), and unlike the
88 lands these cards are not silently *wrong* — they produce the right mana, just via the
stack. That is a CR violation with real consequences (an opponent can respond; they cannot
be used mid-cast, which is what a Signet is *for*), but it is not corrupted game state.

---

## SF-2 — CR 305.6 is not implemented: basic land subtypes grant no mana abilities **[empirically proven]**

**Severity: HIGH as an authoring hazard; the 14 known victims are fixed.**

CR 305.6 gives a land with a basic land type the corresponding intrinsic mana ability.
The engine does not implement this. The only `305.6` reference in `rules/` is at
`layers.rs:1802`, and it is about the **Domain** ability word (counting basic land types),
not granting mana.

Probe (temporary, since removed):

```
PROBE Indatha Triome:   layer-resolved mana_abilities=0 subtypes={Forest, Plains, Swamp}
                        TapForMana(0) -> Err(InvalidAbilityIndex)
PROBE Thundering Falls: layer-resolved mana_abilities=0 subtypes={Island, Mountain}
                        TapForMana(0) -> Err(InvalidAbilityIndex)
PROBE Forest:           layer-resolved mana_abilities=1
```

`Forest` works only because `forest.rs` **authors the ability explicitly** — that is the
house convention, and it is load-bearing. Twelve defs did not follow it and instead carried
the comment:

> `// Mana production handled by basic land subtypes Plains/Swamp/Forest (CR 305.6).`

which described a mana source that does not exist. Those lands produced **nothing at all**
while marked `Complete` — a worse failure than the 88 SR-33 lands, which at least made one
colour. **Fixed in SR-33** (9 Triomes, 3 surveil lands) by authoring the ability explicitly
and rewriting the false comments; `tests/core/effect_choose_gate.rs` now fails any
`Complete` land that prints a plain `{T}: Add` colour it cannot produce, so this cannot
recur silently.

**Still open**: the decision itself. Either implement CR 305.6 in the layers (at which
point every def that authors its own basic-type ability — starting with the five basics —
double-registers and must be stripped), or make "model it explicitly" the documented rule.
Today it is neither: it is a convention that some defs followed and some didn't, with no
gate on the difference. **The gate added by SR-33 only covers `Complete` defs**, so a
`partial`/`inert` land can still carry the false comment.

---

## SF-3 — `Effect::AddManaChoice` carries no colour list

**Severity: MEDIUM.**

```rust
AddManaChoice { player: PlayerTarget, count: EffectAmount }
```

There is nowhere to say *which* colours. `effects/mod.rs:2120` handles it in the same arm
as `AddManaAnyColor`, so it means "any colour" — strictly over-permissive for
`{T}: Add {G}, {W}, or {U}` (Noble Hierarch could make black). It is also unknown to
`try_as_tap_mana_ability`, so it registered no mana ability at all. Both hierarchs were
rewritten to explicit per-colour abilities in SR-33.

Remaining users: `nurturing_peatland`, `silent_clearing`, `fiery_islet`, `glistening_sphere`,
`grand_warlord_radha`. The first three are horizon lands and are also blocked on **SF-1**
(their cost is `{T}, Pay 1 life`), so they cannot be fixed by the same rewrite. Either give
the variant a colour list or delete it in favour of per-colour abilities.

---

## SF-4 — `Effect::MayPayThenEffect` pays when able, which is wrong for opponents

**Severity: MEDIUM.** Deliberately left alone by SR-33; recorded so the reasoning is not lost.

Unlike `MayPayOrElse` (a pure stub, now gated), `MayPayThenEffect` honours its `payer` and
pays when able. `effects/mod.rs` documents this as a legal deterministic choice under
CR 118.12, and for a *beneficial* optional cost paid by the controller that is defensible.

It is not defensible when the payer is an **opponent**: "each opponent may pay {2}; if they
do, …" resolves as every opponent always paying, which is the opposite of how the card
plays. 7 `Complete` defs use it (Crossway Troublemakers, Hazoret's Monument, Leaf-Crowned
Visionary, Miara, Nadir Kraken, Springbloom Druid, Tainted Observer). SR-33 did **not** gate
this variant: doing so would demote 7 cards on a premise that is only true for some of them,
and the per-card judgement is authoring work, not a machine gate. Worth a triage pass:
split by whether `payer` is the controller.

---

## SF-5 — A data-model test can pin a defect as a requirement

**Severity: process finding, no code.**

`card_def_fixes::test_dimir_guildgate_modal_color` asserted that the guildgate's tap ability
**must** be `Effect::Choose` with exactly two choices, and had been green since it was
written. What it actually pinned was a land that registered zero mana abilities, used the
stack, and could only make blue — the SR-33 defect, held in place *by a test*, with a CR
citation on it.

The tell is that it only ever read the def's shape. It asserted that the data matched
someone's expectation, never that the expectation worked. Rewritten in SR-33 to activate the
ability and assert the mana that comes out.

This is the same lesson as SR-9b's "two mutual rejections are equivalent, and worthless" and
SR-9c's vacuous assertion paths: **the ninth-plus consecutive SR finding whose sharpest
result is a hole in a checker rather than a bug in engine code.** Worth a sweep of the other
"data model test verifying the card definition is correct" tests in `card_def_fixes.rs` —
that phrase names the exact anti-pattern.
