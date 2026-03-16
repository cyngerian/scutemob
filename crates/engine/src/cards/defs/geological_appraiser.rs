// Geological Appraiser — {2}{R}{R}, Creature — Human Artificer 3/2
// ETB: if you cast it, discover 3 (CR 701.57).
use crate::cards::helpers::*;

pub fn card() -> CardDefinition {
    CardDefinition {
        card_id: cid("geological-appraiser"),
        name: "Geological Appraiser".to_string(),
        mana_cost: Some(ManaCost { generic: 2, red: 2, ..Default::default() }),
        types: creature_types(&["Human", "Artificer"]),
        oracle_text: "When this creature enters, if you cast it, discover 3. (Exile cards from the top of your library until you exile a nonland card with mana value 3 or less. Cast it without paying its mana cost or put it into your hand. Put the rest on the bottom in a random order.)"
            .to_string(),
        power: Some(3),
        toughness: Some(2),
        abilities: vec![
            AbilityDefinition::Triggered {
                trigger_condition: TriggerCondition::WhenEntersBattlefield,
                // TODO: intervening_if should be Condition::WasCast ("if you cast it"),
                // but no such Condition variant exists yet. Until it is added, this trigger
                // fires unconditionally (including when the creature enters without being cast).
                intervening_if: None,
                targets: vec![],
                effect: Effect::Discover {
                    player: PlayerTarget::Controller,
                    n: 3,
                },
            },
        ],
        color_indicator: None,
        back_face: None,
        spell_cost_modifiers: vec![],
        self_cost_reduction: None,
        starting_loyalty: None,
        meld_pair: None,
    }
}
