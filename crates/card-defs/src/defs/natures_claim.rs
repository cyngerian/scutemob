// Nature's Claim — {G} Instant; destroy target artifact or enchantment. Its controller gains 4 life.
use crate::cards::helpers::*;

pub fn card() -> CardDefinition {
    CardDefinition {
        card_id: cid("natures-claim"),
        name: "Nature's Claim".to_string(),
        mana_cost: Some(ManaCost { green: 1, ..Default::default() }),
        types: types(&[CardType::Instant]),
        oracle_text: "Destroy target artifact or enchantment. Its controller gains 4 life.".to_string(),
        abilities: vec![AbilityDefinition::Spell {
            // Destroy the artifact or enchantment, then its controller gains 4 life.
            // Using ControllerOf(DeclaredTarget{0}) to correctly give life to the
            // destroyed permanent's controller (not the spell's controller).
            effect: Effect::Sequence(vec![
                Effect::DestroyPermanent {
                    target: EffectTarget::DeclaredTarget { index: 0 },
                    cant_be_regenerated: false,
                },
                Effect::GainLife {
                    player: PlayerTarget::ControllerOf(Box::new(EffectTarget::DeclaredTarget { index: 0 })),
                    amount: EffectAmount::Fixed(4),
                },
            ]),
            targets: vec![TargetRequirement::TargetPermanentWithFilter(TargetFilter {
                has_card_types: vec![CardType::Artifact, CardType::Enchantment],
                ..Default::default()
            })],
            modes: None,
            cant_be_countered: false,
        }],
        ..Default::default()
    }
}
