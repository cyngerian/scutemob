// Blood Moon — {2}{R}, Enchantment
// Nonbasic lands are Mountains.
//
// PB-DX43 (CR 305.6/305.7): a single Layer-4 `SetLandTypes(Mountain)` static now
// fully expresses this card. `LayerModification::SetLandTypes` (`rules/layers.rs`)
// itself implements BOTH halves of CR 305.7 once its payload intersects a basic land
// type: it SETS the land-subtype subset of `subtypes` to `{Mountain}` (leaving card
// types, supertypes, and non-land subtypes untouched — OOS-ADJ-7, so an artifact land
// like Ancient Den/Treasure Vault stays an Artifact and Dryad Arbor stays a Creature),
// AND it clears the land's keywords/mana abilities/activated abilities/triggered
// abilities/abilities ("It loses all abilities generated from its rules text, its old
// land types, ..."). The Mountain's own "{T}: Add {R}" mana ability ("...and it gains
// the appropriate mana ability for each new basic land type") is no longer authored
// here at all: CR 305.6 makes it an INTRINSIC ability of any object with the land card
// type and the Mountain subtype, and `rules::layers::derive_intrinsic_land_mana_
// abilities` supplies it automatically, in Layer 4, right after the clearing above —
// idempotent, so it also closes `OOS-DX27-10` (Blood Moon + Magus of the Moon
// together grant exactly ONE `{T}: Add {R}`, not two) with no dedup logic on either
// card def.
//
// This replaces the prior three-static shape (Layer-4 `SetLandTypes` + a separate
// Layer-6 `RemoveAllAbilities` + a separate Layer-6 `AddManaAbility`, the last two
// deliberately mis-ordered so a later timestamp let the grant survive the removal).
// That shape is now obsolete and was itself a latent CR 305.7 violation: a blanket
// Layer-6 `RemoveAllAbilities`, timestamp-ordered against every other Layer-6 effect,
// could strip an earlier-timestamped Layer-6 ability GRANTED to the land by another
// source (Cryptolith Rite, Chromatic Lantern, The World Tree, ...) — which CR 305.7's
// final sentence explicitly forbids ("this doesn't remove any abilities that were
// granted to the land by other effects"). Moving the removal into Layer 4 fixes that:
// any such Layer-6 grant now runs strictly AFTER this static's Layer-4 clearing and so
// survives it regardless of timestamp.
use crate::cards::helpers::*;

pub fn card() -> CardDefinition {
    CardDefinition {
        card_id: cid("blood-moon"),
        name: "Blood Moon".to_string(),
        mana_cost: Some(ManaCost {
            generic: 2,
            red: 1,
            ..Default::default()
        }),
        types: types(&[CardType::Enchantment]),
        oracle_text: "Nonbasic lands are Mountains.".to_string(),
        abilities: vec![
            // Layer 4: Nonbasic lands' LAND subtypes become exactly {Mountain}. Card
            // types, supertypes, and non-land subtypes are untouched (OOS-ADJ-7). The
            // same primitive also clears the land's other abilities and — via the
            // engine's CR 305.6 intrinsic-mana derivation — grants "{T}: Add {R}"; see
            // the module doc above.
            AbilityDefinition::Static {
                continuous_effect: ContinuousEffectDef {
                    layer: EffectLayer::TypeChange,
                    modification: LayerModification::SetLandTypes(
                        [SubType("Mountain".to_string())].into_iter().collect(),
                    ),
                    filter: EffectFilter::AllNonbasicLands,
                    duration: EffectDuration::WhileSourceOnBattlefield,
                    condition: None,
                },
            },
        ],
        ..Default::default()
    }
}
