// Feed the Swarm — {1}{B} Sorcery
// Destroy target creature or enchantment an opponent controls.
// You lose life equal to that permanent's mana value.
use crate::cards::helpers::*;

pub fn card() -> CardDefinition {
    CardDefinition {
        card_id: cid("feed-the-swarm"),
        name: "Feed the Swarm".to_string(),
        mana_cost: Some(ManaCost { generic: 1, black: 1, ..Default::default() }),
        types: types(&[CardType::Sorcery]),
        oracle_text: "Destroy target creature or enchantment an opponent controls. You lose life equal to that permanent's mana value.".to_string(),
        abilities: vec![AbilityDefinition::Spell {
            // CR 701.8: Destroy target creature or enchantment an opponent controls.
            // CR 119: Lose life equal to that permanent's mana value (EffectAmount::ManaValueOf).
            effect: Effect::Sequence(vec![
                Effect::DestroyPermanent {
                    target: EffectTarget::DeclaredTarget { index: 0 },
                    cant_be_regenerated: false,
                },
                Effect::LoseLife {
                    player: PlayerTarget::Controller,
                    amount: EffectAmount::ManaValueOf(EffectTarget::DeclaredTarget { index: 0 }),
                },
            ]),
            // Target: creature or enchantment an opponent controls.
            // has_card_types = OR semantics (Creature OR Enchantment), controller = Opponent.
            targets: vec![TargetRequirement::TargetPermanentWithFilter(TargetFilter {
                has_card_types: vec![CardType::Creature, CardType::Enchantment],
                controller: TargetController::Opponent,
                ..Default::default()
            })],
            modes: None,
            cant_be_countered: false,
        }],
        ..Default::default()
    }
}
