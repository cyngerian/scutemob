// Encroaching Dragonstorm — {3}{G}, Enchantment
// When this enchantment enters, search your library for up to two basic land cards,
// put them onto the battlefield tapped, then shuffle.
// When a Dragon you control enters, return this enchantment to its owner's hand.
//
// TODO (second trigger): Effect::ReturnToHand does not exist as a DSL variant.
// When a Dragon you control enters, return this enchantment to its owner's hand.
// Requires Effect::ReturnToHand { target: EffectTarget::Source } or similar.
use crate::cards::helpers::*;

pub fn card() -> CardDefinition {
    CardDefinition {
        card_id: cid("encroaching-dragonstorm"),
        name: "Encroaching Dragonstorm".to_string(),
        mana_cost: Some(ManaCost { generic: 3, green: 1, ..Default::default() }),
        types: types(&[CardType::Enchantment]),
        oracle_text: "When this enchantment enters, search your library for up to two basic land cards, put them onto the battlefield tapped, then shuffle.\nWhen a Dragon you control enters, return this enchantment to its owner's hand.".to_string(),
        abilities: vec![
            // When this enchantment enters, search for up to two basic lands tapped, then shuffle.
            AbilityDefinition::Triggered {
                trigger_condition: TriggerCondition::WhenEntersBattlefield,
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
                intervening_if: None,
                targets: vec![],
                modes: None,
                trigger_zone: None,
            },
            // TODO: Second trigger — when a Dragon you control enters, return this to owner's hand.
            // Effect::ReturnToHand { target: EffectTarget::Source } does not exist yet.
            // TriggerCondition::WheneverCreatureEntersBattlefield { filter: Some(TargetFilter {
            //     has_subtype: Some(SubType("Dragon".to_string())),
            //     controller: TargetController::You, ..Default::default() }) }
            // → Effect::ReturnToHand { target: EffectTarget::Source }
        ],
        ..Default::default()
    }
}
