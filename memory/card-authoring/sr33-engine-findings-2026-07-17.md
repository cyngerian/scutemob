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

**Not actioned in SR-33**: out of scope (the task is the `Choose` stub). For the Signets,
Cluestones and filter lands, the consequence is "right mana, wrong mechanism": they produce
the correct colours but via the stack, so an opponent can respond and they cannot be used
mid-cast — which is what a Signet is *for*. A CR violation with real consequences, but not
corrupted game state.

> **Correction (found in review).** That tabling argument does **not** cover the three
> horizon lands (Fiery Islet, Nurturing Peatland, Silent Clearing). They are blocked here
> *and* on **SF-3**, and via SF-3 they add `{C}` — a colour they do not print. So they are
> not "right mana, wrong mechanism"; they are the wrong mana, and they were `Complete`.
> SR-33 demoted all three to `known_wrong`. Fixing them needs **both** primitives.

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

## SF-3 — `Effect::AddManaChoice` adds **one colorless mana**, whatever the card prints

**Severity: HIGH. Gated and demoted in SR-33; the primitive is still missing.**

> **Correction.** The first draft of this finding said `AddManaChoice` "means 'any colour' —
> strictly over-permissive". **That was wrong**, and wrong in the exact way this task
> indicts the `path_to_exile` ALLOWLIST entry for (SF-5, EF-2): it reasoned from the
> variant's *name* and stopped at the match arm's head without reading the body. Caught in
> review. The corrected facts follow.

```rust
AddManaChoice { player: PlayerTarget, count: EffectAmount }
```

There is nowhere to say *which* colours — the variant cannot express "Add {U} or {R}" even
in principle. Its **only** execution site (`effects/mod.rs:2120`, confirmed by exhaustive
grep: the other three hits are the declaration, `hash.rs`, and a CR 605.1b classifier in
`rules/mana.rs:608`) shares an arm with `AddManaAnyColor`, and the body is:

```rust
// M9+: interactive mana color choice. For now, add colorless.
ps.mana_pool.add(ManaColor::Colorless, 1);
```

So it adds **one `{C}`** — not a colour the card prints at all — and **ignores `count`**.

The asymmetry that hides this: `AddManaAnyColor` escapes into a real `ManaAbility` with
`any_color: true` via `try_as_tap_mana_ability` (`replay_harness.rs:3641`) and so never
reaches that arm. `AddManaChoice` is **not** recognised there, so its users always route
through the stack and into the colorless body. Sharing the arm is therefore not the
harmless simplification it reads as.

**Victims (all were `Complete`, all now `known_wrong` in SR-33):** Fiery Islet, Nurturing
Peatland, Silent Clearing (each printing `{T}, Pay 1 life: Add {X} or {Y}` and adding `{C}`),
and Glistening Sphere (whose "Add three mana of any one colour" adds one colorless — wrong
on both amount and colour). `grand_warlord_radha` was already `partial`.
`tests/core/effect_choose_gate.rs` now gates the variant out of `Complete`.

**This also falsifies SF-1's tabling argument for the three horizon lands** — see the
correction note there. They are not "right mana via the stack"; they are the wrong mana.

**To fix properly** the variant needs a colour list and `count` support (or deletion in
favour of per-colour abilities) — but the three horizon lands are blocked on **SF-1** as
well, since their cost is `{T}, Pay 1 life`, so per-colour abilities alone would not make
them mana abilities.

---

## SF-4 — `Effect::MayPayThenEffect` pays when able, which is wrong for opponents

**Severity: MEDIUM.** Deliberately left alone by SR-33; recorded so the reasoning is not lost.

Unlike `MayPayOrElse` (a pure stub, now gated), `MayPayThenEffect` honours its `payer` and
pays when able. `effects/mod.rs` documents this as a legal deterministic choice under
CR 118.12, and for a *beneficial* optional cost paid by the controller that is defensible.

It would not be defensible if the payer were an **opponent**: "each opponent may pay {2}; if
they do, …" would resolve as every opponent always paying, the opposite of how the card
plays.

> **Correction (found in review).** That risk is **hypothetical today**. Every one of the 7
> `Complete` users passes `PlayerTarget::Controller`; **none** has an opponent payer
> (Crossway Troublemakers, Hazoret's Monument, Leaf-Crowned Visionary, Miara, Nadir Kraken,
> Springbloom Druid, Tainted Observer). So the carve-out is better founded than the first
> draft argued, not weaker: pay-when-able for a *beneficial* cost paid by the card's own
> controller is a legal deterministic choice, and that is the only case in the corpus.

The distinction from `MayPayOrElse` survives inspection and is not special pleading:
`MayPayOrElse` destructures away `cost`/`payer` and collects nothing (`effects/mod.rs:3196`),
whereas `MayPayThenEffect` resolves the payer, calls `try_pay_optional_cost`, and runs `then`
only if the cost was actually paid (`:3203`) — a complete, legal transaction.

Worth keeping only as a **tripwire**: if a def ever authors `MayPayThenEffect` with a
non-`Controller` payer, it needs the interactive-choice work first. That is a cheap gate
someone could add (assert every `MayPayThenEffect.payer` is `Controller`) and SR-33 did not,
because no card needs it yet.

---

## SF-7 — `cargo fmt` covers **zero** of the 1,749 card defs **[proven]**

**Severity: MEDIUM (a CI gate that silently excludes most of the corpus).**

`cargo fmt --check` is a CI step and it passes. It is also not looking at any card def.

```
crates/card-defs/src/defs/mod.rs:
    include!(concat!(env!("OUT_DIR"), "/card_defs_generated.rs"));
```

`build.rs` discovers the defs and emits `#[path = "…"] pub mod <card>;` into `OUT_DIR`.
rustfmt walks the **module tree from the crate root** and cannot expand `include!` or
`env!`, so it never discovers a single def module. Proof: `cargo fmt --check` reports clean
while `rustfmt --check crates/card-defs/src/defs/tundra.rs` — the same file, named directly
— reports a diff.

This is a consequence of SR-6's extraction (defs moved behind `build.rs` discovery) and has
presumably been true since. SR-33 hit it head-on: a scripted transform emitted
`AbilityDefinition::Activated {` at column 0 in 88 files and `cargo fmt` reported clean.

Second-order trap found while fixing it: naming the files directly *still* left 51
unformatted, silently and with **exit 0** — rustfmt abandons a macro body it cannot fit in
`max_width`, and the generated single-line `Effect::AddMana { … mana: mana_pool(…) }`
exceeded it. So `rustfmt <file>` reporting success does not mean the file was formatted.
Splitting the long line let rustfmt take the whole body.

**Fix options**: (a) add the def files to CI as an explicit `rustfmt` invocation over
`crates/card-defs/src/defs/*.rs`; (b) have `build.rs` emit real `mod` statements into a
checked-in file rather than `OUT_DIR`. (a) is cheap and does not disturb SR-6's arrow
direction. **Do not** simply run `cargo fmt` over the corpus expecting coverage — it is
structurally incapable of reaching it.

---

## SF-6 — SR-33 shifted `activated_abilities` indices on the rewritten lands (latent)

**Severity: LOW — nothing breaks today. Recorded so it is not rediscovered as a mystery.**

`enrich_spec_from_def` deliberately excludes tap-for-mana abilities from
`activated_abilities` "so `ability_index` does not shift". Before SR-33, a
`Cost::Tap` + `Effect::Choose` ability **failed** `try_as_tap_mana_ability`, so it fell
through that exclusion and was registered as a *non-mana* activated ability at index 0.
Now it is correctly recognised and excluded.

Consequence: on any rewritten land that also has a real non-mana activated ability, the
non-mana ability's `ability_index` moves down. The only such card in the changed set is
**Creeping Tar Pit** (its manland ability: 1 → 0). Verified by grep that no test or script
in `tests/` or `test-data/` references it or any other affected land by name, so nothing
regresses. But a `Command::ActivateAbility` written against a pre-SR-33 index would now
silently activate a *different* ability rather than erroring — the failure is quiet, which
is why it is written down.

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
