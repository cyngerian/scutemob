// Memory Lapse — {1}{U}, Instant
// Counter target spell. If that spell is countered this way, put it on top of its owner's
// library instead of into that player's graveyard.
use crate::cards::helpers::*;

pub fn card() -> CardDefinition {
    CardDefinition {
        card_id: cid("memory-lapse"),
        name: "Memory Lapse".to_string(),
        mana_cost: Some(ManaCost { generic: 1, blue: 1, ..Default::default() }),
        types: types(&[CardType::Instant]),
        oracle_text: "Counter target spell. If that spell is countered this way, put it on top of its owner's library instead of into that player's graveyard.".to_string(),
        abilities: vec![AbilityDefinition::Spell {
            // TODO: "put on top of library instead of graveyard" — requires counter-to-top
            // variant. CounterSpell currently sends to graveyard (default CR 701.5a).
            effect: Effect::CounterSpell {
                target: EffectTarget::DeclaredTarget { index: 0 },
            },
            targets: vec![TargetRequirement::TargetSpell],
            modes: None,
            cant_be_countered: false,
        }],
        ..Default::default()
    }
}
