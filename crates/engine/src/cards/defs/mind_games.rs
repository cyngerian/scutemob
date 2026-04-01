// Mind Games — {U}, Instant
// Buyback {2}{U}
// Tap target artifact, creature, or land.
//
// TODO: Oracle text says "tap target artifact, creature, or land" — this is a tap, not a
//   choice between tap/untap. TargetPermanent used as filter (artifact, creature, or land).
//   TargetFilter doesn't have a compound "artifact OR creature OR land" filter — using
//   TargetPermanent as approximation (allows any permanent, slightly broader than oracle).
use crate::cards::helpers::*;

pub fn card() -> CardDefinition {
    CardDefinition {
        card_id: cid("mind-games"),
        name: "Mind Games".to_string(),
        mana_cost: Some(ManaCost { blue: 1, ..Default::default() }),
        types: types(&[CardType::Instant]),
        oracle_text: "Buyback {2}{U} (You may pay an additional {2}{U} as you cast this spell. If you do, put this card into your hand as it resolves.)\nTap target artifact, creature, or land.".to_string(),
        abilities: vec![
            // CR 702.27a: Buyback {2}{U}.
            AbilityDefinition::Buyback {
                cost: ManaCost { generic: 2, blue: 1, ..Default::default() },
            },
            AbilityDefinition::Spell {
                // Tap target artifact, creature, or land.
                // TODO: TargetFilter lacks "artifact OR creature OR land" compound type filter.
                //   Using TargetPermanent as approximation (includes planeswalkers/enchantments too).
                effect: Effect::TapPermanent {
                    target: EffectTarget::DeclaredTarget { index: 0 },
                },
                targets: vec![TargetRequirement::TargetPermanent],
                modes: None,
                cant_be_countered: false,
            },
        ],
        ..Default::default()
    }
}
