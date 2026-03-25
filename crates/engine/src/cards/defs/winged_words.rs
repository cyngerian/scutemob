// Winged Words — {2}{U}, Sorcery
// This spell costs {1} less to cast if you control a creature with flying.
// Draw two cards.
use crate::cards::helpers::*;

pub fn card() -> CardDefinition {
    CardDefinition {
        card_id: cid("winged-words"),
        name: "Winged Words".to_string(),
        mana_cost: Some(ManaCost { generic: 2, blue: 1, ..Default::default() }),
        types: types(&[CardType::Sorcery]),
        oracle_text: "This spell costs {1} less to cast if you control a creature with flying.\nDraw two cards.".to_string(),
        // CR 601.2f: Costs {1} less to cast if caster controls a creature with flying.
        self_cost_reduction: Some(SelfCostReduction::ConditionalKeyword {
            keyword: KeywordAbility::Flying,
            reduction: 1,
        }),
        abilities: vec![AbilityDefinition::Spell {
            effect: Effect::DrawCards {
                player: PlayerTarget::Controller,
                count: EffectAmount::Fixed(2),
            },
            targets: vec![],
            modes: None,
            cant_be_countered: false,
        }],
        ..Default::default()
    }
}
