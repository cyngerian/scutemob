# PB-DX26 — equip-ability authoring spec (21 defs)

> Working spec for the mechanical half of PB-DX26 (`OOS-CARDS1-3`). Measured, not guessed:
> every printed equip line below was MCP-verified (`mcp__mtg-rules__lookup_card`) on
> 2026-08-11, and the 21-def roster was re-derived by enumerating `all_cards()`
> (`core::pb_dx26_attach_keyword_roster::r1`), never by grep — SR-36.

## The defect

`state/keyword_registry.rs:98` classifies `K::Equip` as `KeywordHandling::Marker` whose
carrier is "`Effect::AttachEquipment` … activated through `AbilityDefinition::Activated`".
A marker **synthesises nothing**. So a def carrying only
`AbilityDefinition::Keyword(KeywordAbility::Equip)` has no equip ability for `StubProvider`
to offer, no ability index for a client to name, and no `Command::ActivateAbility` that
could reach one. Where `OOS-M11-10(equip)` was "the picker never asks for a target", this is
**"there is no action to pick"** — the same playtest symptom, one link earlier.

Pre-fix measurement (`--nocapture`, run before any edit):

```
R2 measured marker-without-ability set = 21 of 21
R1 measured equip-marker roster (21) = { … }
R3 measured deck-legal Complete subset = 10
```

## The edit

For each def below, insert an `AbilityDefinition::Activated` **and keep the existing
`AbilityDefinition::Keyword(KeywordAbility::Equip)` marker** (the card really does have the
keyword; the marker is what `view-model::format_keyword` and `state/hash.rs` read, and
removing it would change what the card *is* in order to fix what it *does*).

Reference shape — copy `crates/card-defs/src/defs/skullclamp.rs`'s equip block verbatim,
changing only the mana cost:

```rust
// Equip {N}: attach this Equipment to target creature you control.
// CR 702.6b: Equip is an activated ability; CR 702.6d: sorcery speed only.
AbilityDefinition::Activated {
    cost: Cost::Mana(ManaCost { generic: N, ..Default::default() }),
    effect: Effect::AttachEquipment {
        equipment: EffectTarget::Source,
        target: EffectTarget::DeclaredTarget { index: 0 },
    },
    timing_restriction: Some(TimingRestriction::SorcerySpeed),
    // PB-DX26 (OOS-CARDS1-3) / CR 702.6a: "Equip {N}" means "[Cost]: Attach this permanent
    // to target creature you control." Printed line MCP-verified as plain "Equip {N}" with
    // no CR 702.6c quality restriction, so the requirement is the unmodified 702.6a one.
    targets: vec![TargetRequirement::TargetCreatureWithFilter(TargetFilter {
        controller: TargetController::You,
        ..Default::default()
    })],
    activation_condition: None,
    activation_zone: None,
    once_per_turn: false,
    modes: None,
},
```

## The 21 defs, with their MCP-verified equip cost and disposition

### A. Deck-legal `Complete` (10) — repair in place, completeness UNCHANGED

| def | printed line | cost to author |
|---|---|---|
| `bone_saw` | Equip {1} | `generic: 1` |
| `kite_shield` | Equip {3} | `generic: 3` |
| `paradise_mantle` | Equip {1} | `generic: 1` |
| `the_reaver_cleaver` | Equip {3} | `generic: 3` |
| `sword_of_feast_and_famine` | Equip {2} | `generic: 2` |
| `sword_of_light_and_shadow` | Equip {2} | `generic: 2` |
| `sword_of_sinew_and_steel` | Equip {2} | `generic: 2` |
| `sword_of_truth_and_justice` | Equip {2} | `generic: 2` |
| `sword_of_war_and_peace` | Equip {2} | `generic: 2` |
| `umezawas_jitte` | Equip {2} | `generic: 2` |

None carries a CR 702.6c quality restriction ("equip [quality] {N}"), so every one gets the
unmodified 702.6a requirement.

### B. Non-`Complete` (11) — dispositioned ONE AT A TIME, none dropped silently

| def | printed line | disposition | remaining blocker (re-verified against the CURRENT enums, 2026-08-11) |
|---|---|---|---|
| `sword_of_body_and_mind` | Equip {2} | **REPAIR + FLIP UP to `Completeness::Complete`** | none — its `partial` note named the missing Equip {2} as its *only* blocker ("Protection from green/blue and the combat-damage trigger ARE implemented"), and re-reading the def confirms that. The file-header `TODO` claiming multi-colour protection is unexpressible is itself stale: the def carries two separate `AddKeyword(ProtectionFrom)` statics. Delete that stale header TODO in the same edit. |
| `blade_of_the_bloodchief` | Equip {1} | repair equip, stay `partial` | no `Condition` testing the equipped creature's subtype ("two counters instead if equipped creature is a Vampire"). Re-word the note so it no longer says the equip ability "should be authored" — it now is. |
| `blackblade_reforged` | Equip {7} (+ "Equip legendary creature {3}") | repair the PLAIN Equip {7}, stay `partial` | the CR 702.6c variant cost "Equip legendary creature {3}" has no DSL representation (`AbilityDefinition::Activated` has no per-quality alternate cost), and the dynamic +1/+1-per-land clause is still unauthored. Author **only** the plain Equip {7}. |
| `commanders_plate` | Equip {5} (+ "Equip commander {3}") | repair the PLAIN Equip {5}, stay `partial` | same 702.6c variant gap, plus dynamic protection from colours outside the commander's colour identity. Author **only** the plain Equip {5}. |
| `empyrial_plate` | Equip {2} | repair equip, stay `partial` | the dynamic +1/+1-per-card-in-hand static is unauthored (its note's own "verify first" question about `layers.rs` controller resolution is unanswered). |
| `glimmer_lens` | Equip {1}{W} | repair equip, stay `partial` | no `TriggerCondition` for "equipped creature and at least one other creature attack". **Cost is `{1}{W}`, not generic** — `ManaCost { generic: 1, white: 1, .. }`. Strike the note's now-false "Equip {1}{W} cost is also not modeled" clause. |
| `illusionists_bracers` | Equip {3} | repair equip, stay `partial` | ability copying is not in the DSL. |
| `mask_of_memory` | Equip {1} | repair equip, stay `known_wrong` | "you **may** draw two cards. If you do, discard" is implemented as a mandatory draw. |
| `sword_of_the_animist` | Equip {2} | repair equip, stay `partial` | `TriggerCondition::WhenEquippedCreatureAttacks` does not exist — re-verified: the enum has only `WhenEquippedCreatureDealsCombatDamage` and `…ToPlayer`. |
| `sword_of_the_paruns` | Equip {3} | repair equip, stay `partial` | no `Condition::EquippedCreatureIsTapped` / `EffectFilter::TappedCreaturesYouControl` — re-verified: both strings appear ONLY in this def's own TODO comment. The "{3}: You may tap or untap equipped creature" ability is also unauthored. |
| `umbral_mantle` | Equip {0} | repair equip, stay `partial` | `{Q}` (untap symbol) — re-verified: `requires_untap_self` appears ONLY in this def's own comment, so `ActivationCost` still lacks it. **Cost is `{0}`** — `ManaCost::default()`, an empty cost, which is legal (`bone_saw` is a `{0}` card). |

Every blocker string above was re-checked against the current enums rather than copied
forward — `OOS-DX3-1`'s durable lesson is that **a blocker note is a dated claim**.

## Rules for the edit

1. **Keep the `Keyword(KeywordAbility::Equip)` marker.** Add the `Activated` ability beside it.
2. Every `targets` vector is exactly `vec![TargetRequirement::TargetCreatureWithFilter(
   TargetFilter { controller: TargetController::You, ..Default::default() })]` —
   `core::cards1_equip_target_roster::r2` asserts this shape for every member and will fail
   on an under-restrictive bare `TargetCreature` or an extra filter clause.
3. `timing_restriction: Some(TimingRestriction::SorcerySpeed)` on every one (CR 702.6d).
4. Do **not** touch `oracle_text` — `core::cards2_printed_field_fidelity` diffs it against a
   committed Scryfall fixture (SR-37).
5. Do **not** change any `completeness` marker except `sword_of_body_and_mind`'s
   (`partial` → `Complete`).
6. Where a `completeness` note or a file-header `TODO` claims the equip ability is missing or
   unexpressible, that claim is now false — correct it in the same edit. Do not leave a note
   that will send the next author looking for a gap that is closed.
7. Run `tools/check-defs-fmt.sh` (SR-35 — `cargo fmt` checks **none** of the 1,803 defs and
   still exits 0).
