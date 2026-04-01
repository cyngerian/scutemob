// Force of Vigor — {2}{G}{G}, Instant
// If it's not your turn, you may exile a green card from your hand rather than
// pay this spell's mana cost.
// Destroy up to two target artifacts and/or enchantments.
use crate::cards::helpers::*;

pub fn card() -> CardDefinition {
    CardDefinition {
        card_id: cid("force-of-vigor"),
        name: "Force of Vigor".to_string(),
        mana_cost: Some(ManaCost { generic: 2, green: 2, ..Default::default() }),
        types: types(&[CardType::Instant]),
        oracle_text: "If it's not your turn, you may exile a green card from your hand rather than pay this spell's mana cost.\nDestroy up to two target artifacts and/or enchantments.".to_string(),
        abilities: vec![
            // TODO: Pitch alt cost (exile green card from hand) not in DSL.
            AbilityDefinition::Spell {
                // TODO: "up to two" targets — DSL declares fixed targets.
                // Using two declared targets as approximation.
                effect: Effect::Sequence(vec![
                    Effect::DestroyPermanent {
                        target: EffectTarget::DeclaredTarget { index: 0 },
                        cant_be_regenerated: false,
                    },
                    Effect::DestroyPermanent {
                        target: EffectTarget::DeclaredTarget { index: 1 },
                        cant_be_regenerated: false,
                    },
                ]),
                targets: vec![
                    TargetRequirement::TargetPermanentWithFilter(TargetFilter {
                        has_card_types: vec![CardType::Artifact, CardType::Enchantment],
                        ..Default::default()
                    }),
                    TargetRequirement::TargetPermanentWithFilter(TargetFilter {
                        has_card_types: vec![CardType::Artifact, CardType::Enchantment],
                        ..Default::default()
                    }),
                ],
                modes: None,
                cant_be_countered: false,
            },
        ],
        ..Default::default()
    }
}
