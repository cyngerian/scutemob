// Voldaren Epicure — {R}, Creature — Vampire 1/1
// When Voldaren Epicure enters the battlefield, create a Blood token.
// (It's an artifact with "{1}, {T}, Discard a card, Sacrifice this token: Draw a card.")
//
// CR 111.10g: Blood is a predefined artifact token type.
// CR 603.3: ETB trigger creates the token.
use crate::cards::helpers::*;

pub fn card() -> CardDefinition {
    CardDefinition {
        card_id: cid("voldaren-epicure"),
        name: "Voldaren Epicure".to_string(),
        mana_cost: Some(ManaCost { red: 1, ..Default::default() }),
        types: creature_types(&["Vampire"]),
        oracle_text: "When Voldaren Epicure enters the battlefield, create a Blood token. \
(It's an artifact with \"{1}, {T}, Discard a card, Sacrifice this token: Draw a card.\")"
            .to_string(),
        power: Some(1),
        toughness: Some(1),
        abilities: vec![
            // CR 603.3: ETB trigger — create one Blood token.
            AbilityDefinition::Triggered {
                trigger_condition: TriggerCondition::WhenEntersBattlefield,
                effect: Effect::CreateToken { spec: blood_token_spec(1) },
                intervening_if: None,
                targets: vec![],

                modes: None,
                trigger_zone: None,
            },
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
    }
}
