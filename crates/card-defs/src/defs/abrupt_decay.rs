// Abrupt Decay — {B}{G} Instant; can't be countered; destroy target nonland permanent with MV 3 or less.
use crate::cards::helpers::*;

pub fn card() -> CardDefinition {
    CardDefinition {
        card_id: cid("abrupt-decay"),
        name: "Abrupt Decay".to_string(),
        mana_cost: Some(ManaCost { black: 1, green: 1, ..Default::default() }),
        types: types(&[CardType::Instant]),
        oracle_text: "This spell can't be countered.\nDestroy target nonland permanent with mana value 3 or less.".to_string(),
        abilities: vec![AbilityDefinition::Spell {
            // CR 702.x: This spell can't be countered (cant_be_countered: true).
            // Target: nonland permanent with mana value 3 or less.
            effect: Effect::DestroyPermanent {
                target: EffectTarget::DeclaredTarget { index: 0 },
                    cant_be_regenerated: false,
            },
            targets: vec![TargetRequirement::TargetPermanentWithFilter(TargetFilter {
                non_land: true,
                max_cmc: Some(3),
                ..Default::default()
            })],
            modes: None,
            cant_be_countered: true,
        }],
        ..Default::default()
    }
}
