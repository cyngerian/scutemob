// Blood Moon — {2}{R}, Enchantment
// Nonbasic lands are Mountains.
// CR 305.7: Blood Moon turns all nonbasic lands' LAND SUBTYPES into Mountain (Layer 4,
// SetLandTypes — NOT SetTypeLine, which would also wrongly strip the Artifact/Creature
// card type from an artifact land or creature land) and removes all abilities (Layer 6),
// then grants the Mountain's own "{T}: Add {R}" mana ability. Per the 2020-08-07 ruling:
// "This effect doesn't affect names or supertypes ... Nonbasic lands will lose any other
// land types and abilities they had. They will gain the land type Mountain and gain the
// ability '{T}: Add {R}.'" OOS-ADJ-7 (PB-DX27 rider): the prior SetTypeLine-based
// implementation replaced the WHOLE type line, silently making Ancient Den/Treasure
// Vault stop being artifacts and Dryad Arbor stop being a creature.
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
            // types, supertypes, and non-land subtypes are untouched (OOS-ADJ-7).
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
            // Layer 6: Remove all abilities from nonbasic lands.
            AbilityDefinition::Static {
                continuous_effect: ContinuousEffectDef {
                    layer: EffectLayer::Ability,
                    modification: LayerModification::RemoveAllAbilities,
                    filter: EffectFilter::AllNonbasicLands,
                    duration: EffectDuration::WhileSourceOnBattlefield,
                    condition: None,
                },
            },
            // Layer 6: Grant "{T}: Add {R}" (the Mountain's own mana ability), per the
            // ruling's third sentence. Listed AFTER RemoveAllAbilities so it gets a
            // strictly later timestamp (each `AbilityDefinition::Static` entry is
            // registered with its own incrementing timestamp — see
            // `replacement::register_static_continuous_effects`) and survives the
            // removal in Layer 6 timestamp order.
            AbilityDefinition::Static {
                continuous_effect: ContinuousEffectDef {
                    layer: EffectLayer::Ability,
                    modification: LayerModification::AddManaAbility(ManaAbility {
                        produces: imbl::ordmap! { ManaColor::Red => 1 },
                        requires_tap: true,
                        sacrifice_self: false,
                        any_color: false,
                        damage_to_controller: 0,
                        ..Default::default()
                    }),
                    filter: EffectFilter::AllNonbasicLands,
                    duration: EffectDuration::WhileSourceOnBattlefield,
                    condition: None,
                },
            },
        ],
        ..Default::default()
    }
}
