// Maestros Theater — Land
// When this land enters, sacrifice it. When you do, search your library for a basic
// Island, Swamp, or Mountain card, put it onto the battlefield tapped, then shuffle
// and you gain 1 life.
//
// W5 note: The trigger is "when this land enters, sacrifice it. When you do, search..."
// This is an ETB trigger that sacrifices self and then triggers a "when you do" clause.
// The "when you do" is modeled as a direct Sequence under the ETB trigger effect,
// since the sacrifice is an immediate instruction and the search follows it.
use crate::cards::helpers::*;

pub fn card() -> CardDefinition {
    CardDefinition {
        card_id: cid("maestros-theater"),
        name: "Maestros Theater".to_string(),
        mana_cost: None,
        types: types(&[CardType::Land]),
        oracle_text: "When this land enters, sacrifice it. When you do, search your library for a basic Island, Swamp, or Mountain card, put it onto the battlefield tapped, then shuffle and you gain 1 life.".to_string(),
        abilities: vec![
            AbilityDefinition::Triggered {
                trigger_condition: TriggerCondition::WhenEntersBattlefield,
                effect: Effect::Sequence(vec![
                    // TODO: Sacrifice self from triggered ability — reflexive trigger pattern
                    // ("When you do, search...") not yet expressible. Currently skips sacrifice.
                    // Effect should be: sacrifice this permanent, then on success trigger the search.
                    // Search for basic Island, Swamp, or Mountain.
                    Effect::SearchLibrary {
                        player: PlayerTarget::Controller,
                        filter: TargetFilter {
                            basic: true,
                            has_card_type: Some(CardType::Land),
                            has_subtypes: vec![
                                SubType("Island".to_string()),
                                SubType("Swamp".to_string()),
                                SubType("Mountain".to_string()),
                            ],
                            ..Default::default()
                        },
                        reveal: false,
                        destination: ZoneTarget::Battlefield { tapped: true },
                        shuffle_before_placing: false,
                        also_search_graveyard: false,
                    },
                    Effect::Shuffle { player: PlayerTarget::Controller },
                    Effect::GainLife {
                        player: PlayerTarget::Controller,
                        amount: EffectAmount::Fixed(1),
                    },
                ]),
                intervening_if: None,
                targets: vec![],
                modes: None,
                trigger_zone: None,
            },
        ],
        ..Default::default()
    }
}
