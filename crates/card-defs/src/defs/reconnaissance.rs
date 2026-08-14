// Reconnaissance — {W}, Enchantment
// {0}: Remove target attacking creature you control from combat and untap it.
use crate::cards::helpers::*;

pub fn card() -> CardDefinition {
    CardDefinition {
        card_id: cid("reconnaissance"),
        name: "Reconnaissance".to_string(),
        mana_cost: Some(ManaCost {
            white: 1,
            ..Default::default()
        }),
        types: types(&[CardType::Enchantment]),
        oracle_text: "{0}: Remove target attacking creature you control from combat and untap it. \
                      (If you activate during end of combat, the creature will untap after it \
                      deals combat damage.)"
            .to_string(),
        abilities: vec![
            // CR 506.4/506.4b/701.21 (PB-DX27, closing a stale blocker note): "{0}: Remove
            // target attacking creature you control from combat and untap it." Both
            // Effect::RemoveFromCombat and Effect::UntapPermanent exist and are paired in a
            // Sequence, matching the thaumatic_compass.rs (Spires of Orazca) precedent — CR
            // 506.4b means untapping alone would NOT remove the creature from combat, so both
            // effects are required regardless of their relative order (the two mutations are
            // independent of one another).
            AbilityDefinition::Activated {
                cost: Cost::Mana(ManaCost::default()),
                effect: Effect::Sequence(vec![
                    Effect::RemoveFromCombat {
                        target: EffectTarget::DeclaredTarget { index: 0 },
                    },
                    Effect::UntapPermanent {
                        target: EffectTarget::DeclaredTarget { index: 0 },
                    },
                ]),
                timing_restriction: None,
                targets: vec![TargetRequirement::TargetCreatureWithFilter(TargetFilter {
                    controller: TargetController::You,
                    is_attacking: true,
                    ..Default::default()
                })],
                activation_condition: None,
                activation_zone: None,
                once_per_turn: false,
                modes: None,
            },
        ],
        completeness: Completeness::Complete,
        ..Default::default()
    }
}
