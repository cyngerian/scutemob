// Rewind — {2}{U}{U}, Instant
// Counter target spell. Untap up to four lands.
use crate::cards::helpers::*;

pub fn card() -> CardDefinition {
    CardDefinition {
        card_id: cid("rewind"),
        name: "Rewind".to_string(),
        mana_cost: Some(ManaCost {
            generic: 2,
            blue: 2,
            ..Default::default()
        }),
        types: types(&[CardType::Instant]),
        oracle_text: "Counter target spell. Untap up to four lands.".to_string(),
        abilities: vec![AbilityDefinition::Spell {
            // "Counter target spell." is a REAL printed target (CR 115.1a) and stays a
            // declared target at slot 0 / index 0. "Untap up to four lands." is printed
            // with no "target" — CR 115.10 (PB-DX28): a resolution-time UNTARGETED
            // choice, carried on the `UntapPermanent` effect's own `ChosenObject`
            // rather than a second `TargetRequirement` slot. No more pooled indexing
            // between the two halves: `targets` has exactly one requirement now.
            effect: Effect::Sequence(vec![
                Effect::CounterSpell {
                    target: EffectTarget::DeclaredTarget { index: 0 },
                    exile_instead: false,
                },
                Effect::UntapPermanent {
                    target: EffectTarget::ChosenObject {
                        zone: ChoiceZone::Battlefield,
                        filter: Box::new(TargetFilter {
                            has_card_type: Some(CardType::Land),
                            ..Default::default()
                        }),
                        count: 4,
                        up_to: true,
                    },
                },
            ]),
            targets: vec![TargetRequirement::TargetSpell],
            modes: None,
            cant_be_countered: false,
        }],
        ..Default::default()
    }
}
