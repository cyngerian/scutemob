// Springbloom Druid — {2}{G} Creature — Elf Druid 1/1.
// When this creature enters, you may sacrifice a land. If you do, search your
// library for up to two basic land cards, put them onto the battlefield tapped,
// then shuffle.
use crate::cards::helpers::*;

pub fn card() -> CardDefinition {
    CardDefinition {
        card_id: cid("springbloom-druid"),
        name: "Springbloom Druid".to_string(),
        mana_cost: Some(ManaCost { generic: 2, green: 1, ..Default::default() }),
        types: creature_types(&["Elf", "Druid"]),
        oracle_text: "When this creature enters, you may sacrifice a land. If you do, search your library for up to two basic land cards, put them onto the battlefield tapped, then shuffle.".to_string(),
        power: Some(1),
        toughness: Some(1),
        abilities: vec![
            // PB-AC2 (CR 118.12): "you may sacrifice a land. If you do, search your
            // library for up to two basic land cards, put them onto the battlefield
            // tapped, then shuffle." Springbloom Druid is not itself a land, so the
            // sacrifice-cost filter cannot accidentally target the source.
            AbilityDefinition::Triggered {
                once_per_turn: false,
                trigger_condition: TriggerCondition::WhenEntersBattlefield,
                effect: Effect::MayPayThenEffect {
                    cost: Cost::Sacrifice(TargetFilter {
                        has_card_type: Some(CardType::Land),
                        ..Default::default()
                    }),
                    payer: PlayerTarget::Controller,
                    then: Box::new(Effect::Sequence(vec![
                        Effect::SearchLibrary {
                            player: PlayerTarget::Controller,
                            filter: basic_land_filter(),
                            reveal: false,
                            destination: ZoneTarget::Battlefield { tapped: true },
                            shuffle_before_placing: false,
                            also_search_graveyard: false,
                        },
                        Effect::SearchLibrary {
                            player: PlayerTarget::Controller,
                            filter: basic_land_filter(),
                            reveal: false,
                            destination: ZoneTarget::Battlefield { tapped: true },
                            shuffle_before_placing: false,
                            also_search_graveyard: false,
                        },
                        Effect::Shuffle { player: PlayerTarget::Controller },
                    ])),
                },
                intervening_if: None,
                targets: vec![],
                modes: None,
                trigger_zone: None,
            },
        ],
        ..Default::default()
    }
}
