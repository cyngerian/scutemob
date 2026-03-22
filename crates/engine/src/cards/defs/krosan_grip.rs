// 26a. Krosan Grip — {2G}, Instant; split second; destroy target artifact
// or enchantment.
// TODO: No TargetArtifactOrEnchantment variant exists; using TargetPermanent
// as an approximation. A combined variant (or has_any_of_card_types on
// TargetFilter) would be needed for precise targeting enforcement.
use crate::cards::helpers::*;

pub fn card() -> CardDefinition {
    CardDefinition {
        card_id: cid("krosan-grip"),
        name: "Krosan Grip".to_string(),
        mana_cost: Some(ManaCost { green: 1, generic: 2, ..Default::default() }),
        types: types(&[CardType::Instant]),
        oracle_text: "Split second (As long as this spell is on the stack, players can't cast spells or activate abilities that aren't mana abilities.)\nDestroy target artifact or enchantment.".to_string(),
        abilities: vec![
            AbilityDefinition::Keyword(KeywordAbility::SplitSecond),
            AbilityDefinition::Spell {
                // TODO: target should be TargetArtifactOrEnchantment (no such variant);
                // TargetPermanent is used as the closest approximation.
                effect: Effect::DestroyPermanent {
                    target: EffectTarget::DeclaredTarget { index: 0 },
                    cant_be_regenerated: false,
                },
                targets: vec![TargetRequirement::TargetPermanent],
                modes: None,
                cant_be_countered: false,
            },
        ],
        ..Default::default()
    }
}
