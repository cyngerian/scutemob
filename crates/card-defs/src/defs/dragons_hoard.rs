// Dragon's Hoard — {3}, Artifact
// Whenever a Dragon you control enters, put a gold counter on Dragon's Hoard.
// {T}, Remove a gold counter from Dragon's Hoard: Draw a card.
// {T}: Add one mana of any color.
use crate::cards::helpers::*;

pub fn card() -> CardDefinition {
    CardDefinition {
        card_id: cid("dragons-hoard"),
        name: "Dragon's Hoard".to_string(),
        mana_cost: Some(ManaCost {
            generic: 3,
            ..Default::default()
        }),
        types: types(&[CardType::Artifact]),
        oracle_text: "Whenever a Dragon you control enters, put a gold counter on Dragon's \
                      Hoard.\n{T}, Remove a gold counter from Dragon's Hoard: Draw a card.\n{T}: \
                      Add one mana of any color."
            .to_string(),
        abilities: vec![
            // Whenever a Dragon you control enters, put a gold counter on this.
            AbilityDefinition::Triggered {
                once_per_turn: false,
                trigger_condition: TriggerCondition::WheneverCreatureEntersBattlefield {
                    filter: Some(TargetFilter {
                        has_subtype: Some(SubType("Dragon".to_string())),
                        controller: TargetController::You,
                        ..Default::default()
                    }),
                    exclude_self: false,
                },
                effect: Effect::AddCounter {
                    target: EffectTarget::Source,
                    counter: CounterType::Custom("gold".to_string()),
                    count: 1,
                },
                intervening_if: None,
                targets: vec![],

                modes: None,
                trigger_zone: None,
            },
            // CR 602.2: {T}, Remove a gold counter from Dragon's Hoard: Draw a card.
            AbilityDefinition::Activated {
                cost: Cost::Sequence(vec![
                    Cost::Tap,
                    Cost::RemoveCounter {
                        counter: CounterType::Custom("gold".to_string()),
                        count: 1,
                    },
                ]),
                effect: Effect::DrawCards {
                    player: PlayerTarget::Controller,
                    count: EffectAmount::Fixed(1),
                },
                timing_restriction: None,
                targets: vec![],
                activation_condition: None,
                activation_zone: None,
                once_per_turn: false,
                modes: None,
            },
            // {T}: Add one mana of any color.
            AbilityDefinition::Activated {
                cost: Cost::Tap,
                effect: Effect::AddManaAnyColor {
                    player: PlayerTarget::Controller,
                },
                timing_restriction: None,
                targets: vec![],
                activation_condition: None,
                activation_zone: None,
                once_per_turn: false,
                modes: None,
            },
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
