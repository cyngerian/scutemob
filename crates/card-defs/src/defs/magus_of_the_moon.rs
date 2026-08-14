// Magus of the Moon — {2}{R}, Creature — Human Wizard 2/2
// Nonbasic lands are Mountains.
//
// PB-DX43 (CR 305.6/305.7): identical treatment to `blood_moon.rs` — a single Layer-4
// `SetLandTypes(Mountain)` static now fully expresses this card.
// `LayerModification::SetLandTypes` (`rules/layers.rs`) itself implements BOTH halves
// of CR 305.7 once its payload intersects a basic land type: it SETS the land-subtype
// subset of `subtypes` to `{Mountain}` (preserving card types, supertypes, and
// non-land subtypes — OOS-ADJ-7, so Ancient Den stays an Artifact and Dryad Arbor
// stays a Creature) AND it clears the land's keywords/mana abilities/activated
// abilities/triggered abilities/abilities. The Mountain's own "{T}: Add {R}" mana
// ability is no longer authored here at all: CR 305.6 makes it an INTRINSIC ability of
// any object with the land card type and the Mountain subtype, and
// `rules::layers::derive_intrinsic_land_mana_abilities` supplies it automatically in
// Layer 4, right after the clearing above — idempotent, so it also closes
// `OOS-DX27-10` (Blood Moon + Magus of the Moon together grant exactly ONE
// `{T}: Add {R}`, not two) with no dedup logic on either card def.
//
// This replaces the prior three-static shape (Layer-4 `SetLandTypes` + a separate
// Layer-6 `RemoveAllAbilities` + a separate Layer-6 `AddManaAbility`, the last two
// deliberately mis-ordered so a later timestamp let the grant survive the removal).
// See `blood_moon.rs`'s module doc for the full CR 305.7 "doesn't remove abilities
// granted by other effects" argument for why the removal belongs in Layer 4.
use crate::cards::helpers::*;

pub fn card() -> CardDefinition {
    CardDefinition {
        card_id: cid("magus-of-the-moon"),
        name: "Magus of the Moon".to_string(),
        mana_cost: Some(ManaCost {
            generic: 2,
            red: 1,
            ..Default::default()
        }),
        types: creature_types(&["Human", "Wizard"]),
        oracle_text: "Nonbasic lands are Mountains.".to_string(),
        power: Some(2),
        toughness: Some(2),
        abilities: vec![
            // CR 613.1b/f: "Nonbasic lands are Mountains."
            // Layer 4: SetLandTypes — sets the LAND-subtype subset to exactly {Mountain},
            // preserving card types, supertypes, and non-land subtypes (OOS-ADJ-7). The
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
