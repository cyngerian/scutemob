// Delver of Secrets // Insectile Aberration — DFC with Transform (CR 701.28)
// Front: {U} Human Wizard 1/1, at beginning of your upkeep look at top card,
//        if instant or sorcery transform Delver of Secrets
// Back:  Insectile Aberration, Human Insect 3/2 flying (blue via color indicator)
//
// DSL gap: upkeep trigger with "look at top card, if instant/sorcery transform"
// requires TopOfLibraryIsType condition + TransformSelf effect (not yet in DSL).
// Transform keyword and back_face are faithfully represented.
use crate::cards::helpers::*;

pub fn card() -> CardDefinition {
    CardDefinition {
        card_id: cid("delver-of-secrets-insectile-aberration"),
        name: "Delver of Secrets".to_string(),
        mana_cost: Some(ManaCost { blue: 1, ..Default::default() }),
        types: creature_types(&["Human", "Wizard"]),
        oracle_text: "At the beginning of your upkeep, look at the top card of your library. You may reveal that card. If an instant or sorcery card is revealed this way, transform Delver of Secrets.".to_string(),
        power: Some(1),
        toughness: Some(1),
        abilities: vec![
            AbilityDefinition::Keyword(KeywordAbility::Transform),
            // DSL gap: upkeep trigger needs TransformSelf effect + TopOfLibraryIsType condition
        ],
        color_indicator: None,
        back_face: Some(CardFace {
            name: "Insectile Aberration".to_string(),
            mana_cost: None,
            types: creature_types(&["Human", "Insect"]),
            oracle_text: "Flying".to_string(),
            power: Some(3),
            toughness: Some(2),
            abilities: vec![
                AbilityDefinition::Keyword(KeywordAbility::Flying),
            ],
            color_indicator: Some(vec![Color::Blue]),
        }),
        spell_cost_modifiers: vec![],
        self_cost_reduction: None,
        starting_loyalty: None,
        meld_pair: None,
    }
}
