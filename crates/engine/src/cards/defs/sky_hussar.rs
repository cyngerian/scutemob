// Sky Hussar — {3}{W}{U}, Creature — Human Knight 4/3
// Flying
// When this creature enters, untap all creatures you control.
// Forecast — Tap two untapped white and/or blue creatures you control, Reveal this card
// from your hand: Draw a card. (Activate only during your upkeep and only once each turn.)
use crate::cards::helpers::*;

pub fn card() -> CardDefinition {
    CardDefinition {
        card_id: cid("sky-hussar"),
        name: "Sky Hussar".to_string(),
        mana_cost: Some(ManaCost { generic: 3, white: 1, blue: 1, ..Default::default() }),
        types: creature_types(&["Human", "Knight"]),
        oracle_text: "Flying\nWhen this creature enters, untap all creatures you control.\nForecast — Tap two untapped white and/or blue creatures you control, Reveal this card from your hand: Draw a card. (Activate only during your upkeep and only once each turn.)".to_string(),
        power: Some(4),
        toughness: Some(3),
        abilities: vec![
            AbilityDefinition::Keyword(KeywordAbility::Flying),
            // CR 603.10a / 502.3: "When this creature enters, untap all creatures you control."
            AbilityDefinition::Triggered {
                once_per_turn: false,
                trigger_condition: TriggerCondition::WhenEntersBattlefield,
                effect: Effect::UntapAll {
                    filter: TargetFilter {
                        has_card_type: Some(CardType::Creature),
                        controller: TargetController::You,
                        ..Default::default()
                    },
                },
                intervening_if: None,
                targets: vec![],
                modes: None,
                trigger_zone: None,
            },
            AbilityDefinition::Keyword(KeywordAbility::Forecast),
            // ENGINE-BLOCKED: Forecast's actual activation cost is "Tap two untapped white
            // and/or blue creatures you control, Reveal this card from your hand" — a
            // non-mana cost (creature-tap + hand-reveal). `AbilityDefinition::Forecast {
            // cost: ManaCost, .. }` only supports mana costs, so this specific Forecast
            // activation cannot be expressed. Substituting a mana cost (e.g. {W}{U}) would
            // be a different ability with a different real cost — wrong game state. The
            // effect itself ("Draw a card") is trivial; the cost shape is the blocker.
        ],
        color_indicator: None,
        back_face: None,
        spell_cost_modifiers: vec![],
        self_cost_reduction: None,
        starting_loyalty: None,
        adventure_face: None,
        meld_pair: None,
        spell_additional_costs: vec![],
        activated_ability_cost_reductions: vec![],
        cant_be_countered: false,
        self_exile_on_resolution: false,
        self_shuffle_on_resolution: false,
    }
}
