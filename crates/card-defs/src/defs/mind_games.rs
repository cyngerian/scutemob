// Mind Games — {U}, Instant
// Buyback {2}{U}
// Tap target artifact, creature, or land.
use crate::cards::helpers::*;

pub fn card() -> CardDefinition {
    CardDefinition {
        card_id: cid("mind-games"),
        name: "Mind Games".to_string(),
        mana_cost: Some(ManaCost {
            blue: 1,
            ..Default::default()
        }),
        types: types(&[CardType::Instant]),
        oracle_text: "Buyback {2}{U} (You may pay an additional {2}{U} as you cast this spell. If \
                      you do, put this card into your hand as it resolves.)\nTap target artifact, \
                      creature, or land."
            .to_string(),
        abilities: vec![
            // CR 702.27a: Buyback {2}{U}.
            AbilityDefinition::Buyback {
                cost: ManaCost {
                    generic: 2,
                    blue: 1,
                    ..Default::default()
                },
            },
            AbilityDefinition::Spell {
                // Tap target artifact, creature, or land.
                effect: Effect::TapPermanent {
                    target: EffectTarget::DeclaredTarget { index: 0 },
                },
                targets: vec![TargetRequirement::TargetPermanentWithFilter(TargetFilter {
                    has_card_types: vec![CardType::Artifact, CardType::Creature, CardType::Land],
                    ..Default::default()
                })],
                modes: None,
                cant_be_countered: false,
            },
        ],
        completeness: Completeness::Complete,
        ..Default::default()
    }
}
