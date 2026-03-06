// Glacial Ray — {1}{R}, Instant — Arcane; deals 2 damage to any target.
// Splice onto Arcane {1}{R}: add this card's damage effect to any Arcane spell.
//
// CR 702.47a: Splice onto [subtype] [cost] — while in hand, may be revealed when
// casting an Arcane spell; pay splice cost as additional cost to add this effect.
// The spliced card stays in the player's hand after resolution (CR 702.47c).
use crate::cards::helpers::*;

pub fn card() -> CardDefinition {
    CardDefinition {
        card_id: cid("glacial-ray"),
        name: "Glacial Ray".to_string(),
        mana_cost: Some(ManaCost { generic: 1, red: 1, ..Default::default() }),
        types: types_sub(&[CardType::Instant], &["Arcane"]),
        oracle_text: "Glacial Ray deals 2 damage to any target.\nSplice onto Arcane {1}{R} (As you cast an Arcane spell, you may reveal this card from your hand and pay its splice cost. If you do, add this card's effects to that spell.)".to_string(),
        abilities: vec![
            // CR 702.47a: Splice keyword marker for quick presence-checking.
            AbilityDefinition::Keyword(KeywordAbility::Splice),
            // CR 702.47a: Splice onto Arcane {1}{R} — adds DealDamage 2 to any target.
            AbilityDefinition::Splice {
                cost: ManaCost { generic: 1, red: 1, ..Default::default() },
                onto_subtype: SubType("Arcane".to_string()),
                effect: Box::new(Effect::DealDamage {
                    target: EffectTarget::DeclaredTarget { index: 0 },
                    amount: EffectAmount::Fixed(2),
                }),
            },
            // Primary spell: deal 2 damage to any target.
            AbilityDefinition::Spell {
                effect: Effect::DealDamage {
                    target: EffectTarget::DeclaredTarget { index: 0 },
                    amount: EffectAmount::Fixed(2),
                },
                targets: vec![TargetRequirement::TargetAny],
                modes: None,
                cant_be_countered: false,
            },
        ],
        ..Default::default()
    }
}
