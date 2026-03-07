// Wear // Tear — Split card with Fuse (Dragon's Maze)
// Wear: {1}{R} Instant — Destroy target artifact.
// Tear: {W} Instant — Destroy target enchantment.
// Fuse (You may cast one or both halves of this card from your hand.)
//
// CR 702.102: Fuse — both halves may be cast together from hand, paying combined cost.
// CR 702.102c: Fused MV = {1}{R} + {W} = 3.
// CR 702.102d: Left half (Wear) resolves before right half (Tear).
use crate::cards::helpers::*;

pub fn card() -> CardDefinition {
    CardDefinition {
        card_id: cid("wear-tear"),
        name: "Wear // Tear".to_string(),
        // Wear half: {1}{R}
        mana_cost: Some(ManaCost { generic: 1, red: 1, ..Default::default() }),
        types: types(&[CardType::Instant]),
        oracle_text: "Wear — Destroy target artifact.\nTear — Destroy target enchantment.\nFuse (You may cast one or both halves of this card from your hand.)".to_string(),
        abilities: vec![
            // Fuse keyword marker (CR 702.102)
            AbilityDefinition::Keyword(KeywordAbility::Fuse),

            // Wear (left half): destroy target artifact.
            // Target index 0: artifact.
            AbilityDefinition::Spell {
                effect: Effect::DestroyPermanent {
                    target: EffectTarget::DeclaredTarget { index: 0 },
                },
                targets: vec![TargetRequirement::TargetArtifact],
                modes: None,
                cant_be_countered: false,
            },

            // Tear (right half): destroy target enchantment.
            // Target index 1 (right-half target follows left-half targets — CR 702.102d index contract).
            AbilityDefinition::Fuse {
                name: "Tear".to_string(),
                cost: ManaCost { white: 1, ..Default::default() },
                card_type: CardType::Instant,
                effect: Effect::DestroyPermanent {
                    target: EffectTarget::DeclaredTarget { index: 1 },
                },
                targets: vec![TargetRequirement::TargetEnchantment],
            },
        ],
        ..Default::default()
    }
}
