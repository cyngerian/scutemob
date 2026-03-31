// Khalni Heart Expedition — {1}{G}, Enchantment
// Landfall — Whenever a land you control enters, you may put a quest counter on this.
// Remove three quest counters from this and sacrifice it: Search your library for up to
// two basic land cards, put them onto the battlefield tapped, then shuffle.
//
// TODO: Landfall trigger — TriggerCondition::WheneverLandEntersBattlefield does not exist.
// When this trigger is added, implement:
//   TriggerCondition::WheneverLandEntersBattlefield { controller: TargetController::You }
//   → Effect::AddCounter { target: EffectTarget::Source, counter: CounterType::Quest, count: 1 }
use crate::cards::helpers::*;

pub fn card() -> CardDefinition {
    CardDefinition {
        card_id: cid("khalni-heart-expedition"),
        name: "Khalni Heart Expedition".to_string(),
        mana_cost: Some(ManaCost { generic: 1, green: 1, ..Default::default() }),
        types: types(&[CardType::Enchantment]),
        oracle_text: "Landfall — Whenever a land you control enters, you may put a quest counter on this enchantment.\nRemove three quest counters from this enchantment and sacrifice it: Search your library for up to two basic land cards, put them onto the battlefield tapped, then shuffle.".to_string(),
        abilities: vec![
            // TODO: Landfall trigger — TriggerCondition::WheneverLandEntersBattlefield not yet
            // implemented. Once it exists, add:
            //   AbilityDefinition::Triggered {
            //       trigger_condition: TriggerCondition::WheneverLandEntersBattlefield {
            //           controller: TargetController::You,
            //       },
            //       effect: Effect::AddCounter {
            //           target: EffectTarget::Source,
            //           counter: CounterType::Quest,
            //           count: 1,
            //       },
            //       intervening_if: None, targets: vec![], modes: None, trigger_zone: None,
            //   }

            // Remove three quest counters from this enchantment and sacrifice it: search for
            // up to two basic lands, put them onto battlefield tapped, then shuffle.
            AbilityDefinition::Activated {
                cost: Cost::Sequence(vec![
                    Cost::RemoveCounter {
                        counter: CounterType::Quest,
                        count: 3,
                    },
                    Cost::SacrificeSelf,
                ]),
                effect: Effect::Sequence(vec![
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
                ]),
                timing_restriction: None,
                targets: vec![],
                activation_condition: None,
                activation_zone: None,
                once_per_turn: false,
            },
        ],
        ..Default::default()
    }
}
