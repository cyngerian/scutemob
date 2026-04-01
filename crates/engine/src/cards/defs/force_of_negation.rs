// Force of Negation — {1}{U}{U}, Instant
// If it's not your turn, you may exile a blue card from your hand rather than
// pay this spell's mana cost.
// Counter target noncreature spell. If that spell is countered this way, exile
// it instead of putting it into its owner's graveyard.
use crate::cards::helpers::*;

pub fn card() -> CardDefinition {
    CardDefinition {
        card_id: cid("force-of-negation"),
        name: "Force of Negation".to_string(),
        mana_cost: Some(ManaCost { generic: 1, blue: 2, ..Default::default() }),
        types: types(&[CardType::Instant]),
        oracle_text: "If it's not your turn, you may exile a blue card from your hand rather than pay this spell's mana cost.\nCounter target noncreature spell. If that spell is countered this way, exile it instead of putting it into its owner's graveyard.".to_string(),
        abilities: vec![
            // TODO: Pitch alt cost + exile-on-counter replacement not in DSL.
            AbilityDefinition::Spell {
                effect: Effect::CounterSpell {
                    target: EffectTarget::DeclaredTarget { index: 0 },
                },
                targets: vec![TargetRequirement::TargetSpellWithFilter(TargetFilter {
                    non_creature: true,
                    ..Default::default()
                })],
                modes: None,
                cant_be_countered: false,
            },
        ],
        ..Default::default()
    }
}
