// Fumigate — {3}{W}{W} Sorcery
// Destroy all creatures. You gain 1 life for each creature destroyed this way.
use crate::cards::helpers::*;

pub fn card() -> CardDefinition {
    CardDefinition {
        card_id: cid("fumigate"),
        name: "Fumigate".to_string(),
        mana_cost: Some(ManaCost { generic: 3, white: 2, ..Default::default() }),
        types: types(&[CardType::Sorcery]),
        oracle_text:
            "Destroy all creatures. You gain 1 life for each creature destroyed this way."
                .to_string(),
        abilities: vec![AbilityDefinition::Spell {
            // CR 701.8: Destroy all creatures, then gain life equal to the count.
            // EffectAmount::LastEffectCount reads ctx.last_effect_count set by DestroyAll.
            effect: Effect::Sequence(vec![
                Effect::DestroyAll {
                    filter: TargetFilter {
                        has_card_type: Some(CardType::Creature),
                        ..Default::default()
                    },
                    cant_be_regenerated: false,
                },
                Effect::GainLife {
                    player: PlayerTarget::Controller,
                    amount: EffectAmount::LastEffectCount,
                },
            ]),
            targets: vec![],
            modes: None,
            cant_be_countered: false,
        }],
        ..Default::default()
    }
}
