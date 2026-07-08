// Ashaya, Soul of the Wild — {3}{G}{G}, Legendary Creature — Elemental */*
// Ashaya's power and toughness are each equal to the number of lands you control.
// Nontoken creatures you control are Forest lands in addition to their other types.
//
// CDA (*/*): power: None, toughness: None per KI-4.
// PB-AC3: the "P/T = lands you control" CDA is expressible via the already-shipped
// AbilityDefinition::CdaPowerToughness{PermanentCount{Land}} (see Ulvenwald Hydra for the
// pattern) — NOT a new PB-AC3 primitive and NOT the blocker for this card.
// Static: Nontoken creatures you control gain Land + Forest types.
// TODO: "nontoken" filter — EffectFilter has no nontoken-exclusion variant
// (EffectFilter::CreaturesYouControl includes tokens). This is the remaining blocker:
// authoring the CDA without fixing the nontoken-scoped type-grant below would leave the
// static ability granting Forest/Land types to token creatures too (wrong game state per
// oracle text "Nontoken creatures you control..."). Card stays blocked under W6
// no-partials policy until a nontoken-scoped EffectFilter (or equivalent) ships.
use crate::cards::helpers::*;

pub fn card() -> CardDefinition {
    CardDefinition {
        card_id: cid("ashaya-soul-of-the-wild"),
        name: "Ashaya, Soul of the Wild".to_string(),
        mana_cost: Some(ManaCost { generic: 3, green: 2, ..Default::default() }),
        types: full_types(
            &[SuperType::Legendary],
            &[CardType::Creature],
            &["Elemental"],
        ),
        oracle_text: "Ashaya, Soul of the Wild's power and toughness are each equal to the number of lands you control.\nNontoken creatures you control are Forest lands in addition to their other types.".to_string(),
        power: None,
        toughness: None,
        abilities: vec![
            // Layer 4: Add Land + Forest types to creatures you control.
            AbilityDefinition::Static {
                continuous_effect: ContinuousEffectDef {
                    layer: EffectLayer::TypeChange,
                    modification: LayerModification::AddCardTypes(
                        [CardType::Land].into_iter().collect(),
                    ),
                    filter: EffectFilter::CreaturesYouControl,
                    duration: EffectDuration::WhileSourceOnBattlefield,
                    condition: None,
                },
            },
            AbilityDefinition::Static {
                continuous_effect: ContinuousEffectDef {
                    layer: EffectLayer::TypeChange,
                    modification: LayerModification::AddSubtypes(
                        [SubType("Forest".to_string())].into_iter().collect(),
                    ),
                    filter: EffectFilter::CreaturesYouControl,
                    duration: EffectDuration::WhileSourceOnBattlefield,
                    condition: None,
                },
            },
        ],
        ..Default::default()
    }
}
