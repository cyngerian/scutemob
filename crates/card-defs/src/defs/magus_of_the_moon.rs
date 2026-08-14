// Magus of the Moon — {2}{R}, Creature — Human Wizard 2/2
// Nonbasic lands are Mountains.
// (Identical effect to Blood Moon: Layer 4 SetLandTypes + Layer 6 RemoveAllAbilities +
// Layer 6 AddManaAbility({T}: Add {R}) on all nonbasic lands. OOS-ADJ-7, PB-DX27 rider:
// SetLandTypes (not SetTypeLine) preserves the Artifact/Creature card type an artifact
// land or creature land has — the printed card never touches card types, per the
// 2020-08-07 ruling: "This effect doesn't affect names or supertypes.")
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
            // preserving card types, supertypes, and non-land subtypes (OOS-ADJ-7).
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
            // Layer 6: RemoveAllAbilities — removes any activated/triggered abilities
            // the nonbasic lands had (mana abilities, etc.).
            AbilityDefinition::Static {
                continuous_effect: ContinuousEffectDef {
                    layer: EffectLayer::Ability,
                    modification: LayerModification::RemoveAllAbilities,
                    filter: EffectFilter::AllNonbasicLands,
                    duration: EffectDuration::WhileSourceOnBattlefield,
                    condition: None,
                },
            },
            // Layer 6: Grant "{T}: Add {R}" per the ruling's third sentence. Listed
            // AFTER RemoveAllAbilities so it gets a strictly later timestamp (each
            // `AbilityDefinition::Static` entry is registered with its own
            // incrementing timestamp) and survives the removal in Layer 6 timestamp
            // order.
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
