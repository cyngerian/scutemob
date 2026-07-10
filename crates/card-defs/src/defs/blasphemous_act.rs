// Blasphemous Act — {8}{R} Sorcery; deals 13 damage to each creature.
// This spell costs {1} less to cast for each creature on the battlefield.
use crate::cards::helpers::*;

pub fn card() -> CardDefinition {
    CardDefinition {
        card_id: cid("blasphemous-act"),
        name: "Blasphemous Act".to_string(),
        mana_cost: Some(ManaCost { generic: 8, red: 1, ..Default::default() }),
        types: types(&[CardType::Sorcery]),
        oracle_text: "This spell costs {1} less to cast for each creature on the battlefield.\nBlasphemous Act deals 13 damage to each creature.".to_string(),
        abilities: vec![AbilityDefinition::Spell {
            effect: Effect::DealDamage {
                target: EffectTarget::AllCreatures,
                amount: EffectAmount::Fixed(13),
            },
            targets: vec![],
            modes: None,
            cant_be_countered: false,
        }],
        self_cost_reduction: Some(SelfCostReduction::PerPermanent {
            per: 1,
            filter: TargetFilter {
                has_card_type: Some(CardType::Creature),
                ..Default::default()
            },
            controller: PlayerTarget::EachPlayer,
        }),
        ..Default::default()
    }
}
