// Braided Net // Braided Quipu — DFC with Craft (CR 702.167)
// Front: {2} Artifact, when ETB tap target creature an opponent controls.
//        Craft with artifact {2}{U}
// Back:  Braided Quipu, Artifact, when ETB tap target creature,
//        whenever you cast a spell draw a card
//
// DSL gap: TapTarget effect not in DSL. ETB trigger effect and cast-spell trigger
// cannot be expressed. Craft keyword and back_face are faithfully represented.
use crate::cards::helpers::*;

pub fn card() -> CardDefinition {
    CardDefinition {
        card_id: cid("braided-net-braided-quipu"),
        name: "Braided Net".to_string(),
        mana_cost: Some(ManaCost { generic: 2, ..Default::default() }),
        types: types(&[CardType::Artifact]),
        oracle_text: "When Braided Net enters the battlefield, tap target creature an opponent controls.\nCraft with artifact {2}{U}".to_string(),
        power: None,
        toughness: None,
        abilities: vec![
            AbilityDefinition::Keyword(KeywordAbility::Craft),
            AbilityDefinition::Craft {
                cost: ManaCost { generic: 2, blue: 1, ..Default::default() },
                materials: CraftMaterials::Artifacts(1),
            },
            // DSL gap: ETB tap target creature needs TapTarget effect
        ],
        color_indicator: None,
        back_face: Some(CardFace {
            name: "Braided Quipu".to_string(),
            mana_cost: None,
            types: types(&[CardType::Artifact]),
            oracle_text: "When Braided Quipu enters the battlefield, tap target creature an opponent controls.\nWhenever you cast a spell, draw a card.".to_string(),
            power: None,
            toughness: None,
            abilities: vec![
                // DSL gap: ETB tap target + cast-spell trigger
            ],
            color_indicator: Some(vec![Color::Blue]),
        }),
        spell_cost_modifiers: vec![],
        self_cost_reduction: None,
        starting_loyalty: None,
        adventure_face: None,
        meld_pair: None,
    }
}
