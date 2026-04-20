// Khalni Heart Expedition — {1}{G}, Enchantment
// Landfall — Whenever a land you control enters, you may put a quest counter on this.
// Remove three quest counters from this and sacrifice it: Search your library for up to
// two basic land cards, put them onto the battlefield tapped, then shuffle.
//
// CR 207.2c: Landfall is an ability word with no dedicated CR rule; the trigger is
// encoded as TriggerCondition::WheneverPermanentEntersBattlefield { Land + You }.
// See jaddi_offshoot.rs for the canonical simple Landfall template.
use crate::cards::helpers::*;

pub fn card() -> CardDefinition {
    CardDefinition {
        card_id: cid("khalni-heart-expedition"),
        name: "Khalni Heart Expedition".to_string(),
        mana_cost: Some(ManaCost { generic: 1, green: 1, ..Default::default() }),
        types: types(&[CardType::Enchantment]),
        oracle_text: "Landfall — Whenever a land you control enters, you may put a quest counter on this enchantment.\nRemove three quest counters from this enchantment and sacrifice it: Search your library for up to two basic land cards, put them onto the battlefield tapped, then shuffle.".to_string(),
        abilities: vec![
            // Landfall — Whenever a land you control enters, put a quest counter on this.
            // CR 207.2c: "Landfall" is an ability word; trigger uses WheneverPermanentEntersBattlefield.
            // Note: "you may" — bot always takes counter; harmless optimization.
            AbilityDefinition::Triggered {
                trigger_condition: TriggerCondition::WheneverPermanentEntersBattlefield {
                    filter: Some(TargetFilter {
                        has_card_type: Some(CardType::Land),
                        controller: TargetController::You,
                        ..Default::default()
                    }),
                },
                effect: Effect::AddCounter {
                    target: EffectTarget::Source,
                    counter: CounterType::Quest,
                    count: 1,
                },
                intervening_if: None,
                targets: vec![],
                modes: None,
                trigger_zone: None,
            },

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
