// Beloved Beggar // Generous Soul — DFC with Disturb (CR 702.146)
// Front: {1}{W} Human Peasant 0/4, when dies gain 3 life
// Back:  {4}{W}{W} Spirit 3/4 flying, vigilance; exile if would go to graveyard
use crate::cards::helpers::*;

pub fn card() -> CardDefinition {
    CardDefinition {
        card_id: cid("beloved-beggar-generous-soul"),
        name: "Beloved Beggar".to_string(),
        mana_cost: Some(ManaCost { generic: 1, white: 1, ..Default::default() }),
        types: creature_types(&["Human", "Peasant"]),
        oracle_text: "When Beloved Beggar dies, you gain 3 life.\nDisturb {4}{W}{W} (You may cast this card transformed from your graveyard for its disturb cost.)".to_string(),
        power: Some(0),
        toughness: Some(4),
        abilities: vec![
            // CR 702.146a: Disturb keyword marker for presence-checking.
            AbilityDefinition::Keyword(KeywordAbility::Disturb),
            // CR 702.146a: Disturb cost {4}{W}{W}.
            AbilityDefinition::Disturb {
                cost: ManaCost { generic: 4, white: 2, ..Default::default() },
            },
            // "When Beloved Beggar dies, you gain 3 life."
            AbilityDefinition::Triggered {
                trigger_condition: TriggerCondition::WhenDies,
                effect: Effect::GainLife {
                    player: PlayerTarget::Controller,
                    amount: EffectAmount::Fixed(3),
                },
                intervening_if: None,
                targets: vec![],
            },
        ],
        color_indicator: None,
        back_face: Some(CardFace {
            name: "Generous Soul".to_string(),
            // Back face has a printed mana cost {4}{W}{W} (used for Disturb casting).
            mana_cost: Some(ManaCost { generic: 4, white: 2, ..Default::default() }),
            types: creature_types(&["Spirit"]),
            oracle_text: "Flying\nVigilance\nIf Generous Soul would be put into a graveyard from anywhere, exile it instead.".to_string(),
            power: Some(3),
            toughness: Some(4),
            abilities: vec![
                AbilityDefinition::Keyword(KeywordAbility::Flying),
                AbilityDefinition::Keyword(KeywordAbility::Vigilance),
                // CR 702.146 ruling: exile-if-graveyard replacement enforced by engine
                // via was_cast_disturbed flag; the keyword marker here is for display/reference.
            ],
            color_indicator: None,
        }),
        spell_cost_modifiers: vec![],
        self_cost_reduction: None,
        starting_loyalty: None,
        adventure_face: None,
        meld_pair: None,
    }
}
