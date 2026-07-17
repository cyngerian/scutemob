// Outcaster Trailblazer — {2}{G}, Creature — Human Druid 4/2
// When this creature enters, add one mana of any color.
// Whenever another creature you control with power 4 or greater enters, draw a card.
// Plot {2}{G}
use crate::cards::helpers::*;

pub fn card() -> CardDefinition {
    CardDefinition {
        card_id: cid("outcaster-trailblazer"),
        name: "Outcaster Trailblazer".to_string(),
        mana_cost: Some(ManaCost {
            generic: 2,
            green: 1,
            ..Default::default()
        }),
        types: creature_types(&["Human", "Druid"]),
        oracle_text: "When this creature enters, add one mana of any color.\nWhenever another \
                      creature you control with power 4 or greater enters, draw a card.\nPlot \
                      {2}{G}"
            .to_string(),
        power: Some(4),
        toughness: Some(2),
        abilities: vec![
            AbilityDefinition::Triggered {
                once_per_turn: false,
                trigger_condition: TriggerCondition::WhenEntersBattlefield,
                effect: Effect::AddManaAnyColor {
                    player: PlayerTarget::Controller,
                },
                intervening_if: None,
                targets: vec![],

                modes: None,
                trigger_zone: None,
            },
            AbilityDefinition::Triggered {
                once_per_turn: false,
                trigger_condition: TriggerCondition::WheneverCreatureEntersBattlefield {
                    filter: Some(TargetFilter {
                        controller: TargetController::You,
                        min_power: Some(4),
                        ..Default::default()
                    }),
                    exclude_self: true,
                },
                effect: Effect::DrawCards {
                    player: PlayerTarget::Controller,
                    count: EffectAmount::Fixed(1),
                },
                intervening_if: None,
                targets: vec![],

                modes: None,
                trigger_zone: None,
            },
            AbilityDefinition::Keyword(KeywordAbility::Plot),
        ],
        completeness: Completeness::known_wrong(
            "SF-11 (CR 106.1a/106.1b): this card's \"any color\" mana resolves to one COLORLESS \
             mana (Effect::AddManaAnyColor family; effects/mod.rs and handle_tap_for_mana step 8 \
             both add ManaColor::Colorless). Colorless is a mana TYPE, not a color, so {C} is \
             outside the option set \"any color\" offers — wrong game state, not an omitted \
             clause. Un-mark when a color channel for any-color mana lands (SR-37 / scutemob-93).",
        ),
        ..Default::default()
    }
}
