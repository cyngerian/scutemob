// Decadent Dragon // Expensive Taste — {2}{R}{R} Creature — Dragon 4/4
// Oracle (front): "Flying, trample\nWhenever this creature attacks, create a Treasure token."
// Oracle (adventure): "Exile the top two cards of target player's library face down. You may
// look at and play those cards for as long as they remain exiled, and you may spend mana as
// though it were mana of any color to cast them."
use crate::cards::helpers::*;

pub fn card() -> CardDefinition {
    CardDefinition {
        card_id: cid("decadent-dragon"),
        name: "Decadent Dragon // Expensive Taste".to_string(),
        mana_cost: Some(ManaCost { generic: 2, red: 2, ..Default::default() }),
        types: creature_types(&["Dragon"]),
        oracle_text: "Flying, trample\nWhenever this creature attacks, create a Treasure token.".to_string(),
        power: Some(4),
        toughness: Some(4),
        abilities: vec![
            AbilityDefinition::Keyword(KeywordAbility::Flying),
            AbilityDefinition::Keyword(KeywordAbility::Trample),
            // CR 603.1: Attack trigger — create a Treasure token.
            AbilityDefinition::Triggered {
                trigger_condition: TriggerCondition::WhenAttacks,
                effect: Effect::CreateToken { spec: treasure_token_spec(1) },
                intervening_if: None,
                targets: vec![],
            },
        ],
        color_indicator: None,
        back_face: None,
        adventure_face: Some(CardFace {
            name: "Expensive Taste".to_string(),
            mana_cost: Some(ManaCost { generic: 2, black: 1, ..Default::default() }),
            types: TypeLine {
                card_types: [CardType::Instant].iter().copied().collect(),
                subtypes: [SubType("Adventure".to_string())]
                    .iter()
                    .cloned()
                    .collect(),
                supertypes: Default::default(),
            },
            oracle_text: "Exile the top two cards of target player's library face down. You may look at and play those cards for as long as they remain exiled, and you may spend mana as though it were mana of any color to cast them.".to_string(),
            power: None,
            toughness: None,
            color_indicator: None,
            // TODO: DSL gap — exile-face-down + play-from-exile + any-color mana spending.
            // Complex exile-play mechanic not expressible.
            abilities: vec![],
        }),
        spell_cost_modifiers: vec![],
        self_cost_reduction: None,
        starting_loyalty: None,
        meld_pair: None,
        spell_additional_costs: vec![],
        activated_ability_cost_reductions: vec![],
    }
}
