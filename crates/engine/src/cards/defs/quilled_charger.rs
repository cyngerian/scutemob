// Quilled Charger — {3}{R} Creature — Porcupine Mount 4/3
// Whenever this creature attacks while saddled, it gets +1/+2 and gains menace until end of turn.
// Saddle 2
use crate::cards::helpers::*;

pub fn card() -> CardDefinition {
    CardDefinition {
        card_id: cid("quilled-charger"),
        name: "Quilled Charger".to_string(),
        mana_cost: Some(ManaCost { red: 1, generic: 3, ..Default::default() }),
        types: full_types(
            &[],
            &[CardType::Creature],
            &["Porcupine", "Mount"],
        ),
        oracle_text: "Whenever this creature attacks while saddled, it gets +1/+2 and gains menace until end of turn. (It can't be blocked except by two or more creatures.)\nSaddle 2 (Tap any number of other creatures you control with total power 2 or more: This Mount becomes saddled until end of turn. Saddle only as a sorcery.)".to_string(),
        abilities: vec![
            // CR 702.171: Saddle 2 — tap creatures with total power >= 2.
            AbilityDefinition::Keyword(KeywordAbility::Saddle(2)),
            // TODO: "Whenever this creature attacks while saddled, it gets +1/+2 and gains menace
            // until end of turn." Requires TriggerCondition::WhenAttacksWhileSaddled (not yet
            // implemented). The trigger needs to check is_saddled on the attacker at attack
            // declaration time. Deferred until attack-trigger infrastructure supports saddled check.
        ],
        power: Some(4),
        toughness: Some(3),
        ..Default::default()
    }
}
