// Torch the Tower — {R}, Instant; Bargain (WOE)
// Deals 2 damage to target creature or planeswalker; if bargained, deals 3 instead
// and the controller can't gain life this turn.
// CR 702.166: Bargain — optional additional cost: sacrifice an artifact, enchantment, or token.
// CR 702.166b: "was bargained" — the player paid the bargain cost at cast time.
use crate::cards::helpers::*;

pub fn card() -> CardDefinition {
    CardDefinition {
        card_id: cid("torch-the-tower"),
        name: "Torch the Tower".to_string(),
        mana_cost: Some(ManaCost { red: 1, ..Default::default() }),
        types: types(&[CardType::Instant]),
        oracle_text: "Bargain (You may sacrifice an artifact, enchantment, or token as you cast this spell.)\nTorch the Tower deals 2 damage to target creature or planeswalker. If this spell was bargained, it deals 3 damage instead and the controller can't gain life this turn.".to_string(),
        abilities: vec![
            AbilityDefinition::Keyword(KeywordAbility::Bargain),
            AbilityDefinition::Spell {
                effect: Effect::Conditional {
                    condition: Condition::WasBargained,
                    if_true: Box::new(Effect::DealDamage {
                        target: EffectTarget::DeclaredTarget { index: 0 },
                        amount: EffectAmount::Fixed(3),
                        // TODO: also apply "controller can't gain life this turn" —
                        // no CantGainLife effect variant exists in the DSL yet.
                        // When Effect::CantGainLife { player, duration } is added,
                        // wrap this in Effect::Sequence(vec![DealDamage{3}, CantGainLife{...}]).
                    }),
                    if_false: Box::new(Effect::DealDamage {
                        target: EffectTarget::DeclaredTarget { index: 0 },
                        amount: EffectAmount::Fixed(2),
                    }),
                },
                targets: vec![TargetRequirement::TargetPermanentWithFilter(TargetFilter {
                    has_card_types: vec![CardType::Creature, CardType::Planeswalker],
                    ..Default::default()
                })],
                modes: None,
                cant_be_countered: false,
            },
        ],
        ..Default::default()
    }
}
