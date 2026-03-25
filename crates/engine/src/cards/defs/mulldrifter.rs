// 66. Mulldrifter — {4}{U}, Creature — Elemental 2/2; Flying; ETB draw two cards;
// Evoke {2}{U}.
use crate::cards::helpers::*;

pub fn card() -> CardDefinition {
    CardDefinition {
        card_id: cid("mulldrifter"),
        name: "Mulldrifter".to_string(),
        mana_cost: Some(ManaCost { generic: 4, blue: 1, ..Default::default() }),
        types: creature_types(&["Elemental"]),
        oracle_text: "Flying\nWhen this creature enters, draw two cards.\nEvoke {2}{U} (You may cast this spell for its evoke cost. If you do, it's sacrificed when it enters.)".to_string(),
        power: Some(2),
        toughness: Some(2),
        abilities: vec![
            AbilityDefinition::Keyword(KeywordAbility::Flying),
            AbilityDefinition::Triggered {
                trigger_condition: TriggerCondition::WhenEntersBattlefield,
                effect: Effect::DrawCards {
                    player: PlayerTarget::Controller,
                    count: EffectAmount::Fixed(2),
                },
                intervening_if: None,
                targets: vec![],
            },
            AbilityDefinition::Evoke {
                cost: ManaCost { generic: 2, blue: 1, ..Default::default() },
            },
        ],
        color_indicator: None,
        back_face: None,
        spell_cost_modifiers: vec![],
        self_cost_reduction: None,
        starting_loyalty: None,
        adventure_face: None,
        meld_pair: None,
        activated_ability_cost_reductions: vec![],
    }
}
