// Wood Elves — {2}{G}, Creature — Elf Scout 1/1
// When this creature enters, search your library for a Forest card,
// put that card onto the battlefield, then shuffle.
use crate::cards::helpers::*;

pub fn card() -> CardDefinition {
    CardDefinition {
        card_id: cid("wood-elves"),
        name: "Wood Elves".to_string(),
        mana_cost: Some(ManaCost { generic: 2, green: 1, ..Default::default() }),
        types: creature_types(&["Elf", "Scout"]),
        oracle_text: "When this creature enters, search your library for a Forest card, put that card onto the battlefield, then shuffle.".to_string(),
        power: Some(1),
        toughness: Some(1),
        abilities: vec![
            AbilityDefinition::Triggered {
                trigger_condition: TriggerCondition::WhenEntersBattlefield,
                effect: Effect::Sequence(vec![
                    Effect::SearchLibrary {
                        player: PlayerTarget::Controller,
                        filter: TargetFilter {
                            has_card_type: Some(CardType::Land),
                            has_subtype: Some(SubType("Forest".to_string())),
                            ..Default::default()
                        },
                        reveal: false,
                        destination: ZoneTarget::Battlefield { tapped: false },
                        shuffle_before_placing: false,
                        also_search_graveyard: false,
                    },
                    Effect::Shuffle { player: PlayerTarget::Controller },
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
