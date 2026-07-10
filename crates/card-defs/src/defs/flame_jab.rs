// Flame Jab — {R}, Sorcery; deal 1 damage to any target; Retrace
use crate::cards::helpers::*;

pub fn card() -> CardDefinition {
    CardDefinition {
        card_id: cid("flame-jab"),
        name: "Flame Jab".to_string(),
        mana_cost: Some(ManaCost { red: 1, ..Default::default() }),
        types: types(&[CardType::Sorcery]),
        oracle_text: "Flame Jab deals 1 damage to any target.\nRetrace (You may cast this card from your graveyard by discarding a land card in addition to paying its other costs.)".to_string(),
        abilities: vec![
            AbilityDefinition::Spell {
                effect: Effect::DealDamage {
                    target: EffectTarget::DeclaredTarget { index: 0 },
                    amount: EffectAmount::Fixed(1),
                },
                targets: vec![TargetRequirement::TargetAny],
                modes: None,
                cant_be_countered: false,
            },
            AbilityDefinition::Keyword(KeywordAbility::Retrace),
        ],
        ..Default::default()
    }
}
