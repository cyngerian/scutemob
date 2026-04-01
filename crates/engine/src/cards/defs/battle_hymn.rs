// Battle Hymn — {1}{R}, Instant
// Add {R} for each creature you control.
use crate::cards::helpers::*;

pub fn card() -> CardDefinition {
    CardDefinition {
        card_id: cid("battle-hymn"),
        name: "Battle Hymn".to_string(),
        mana_cost: Some(ManaCost { generic: 1, red: 1, ..Default::default() }),
        types: types(&[CardType::Instant]),
        oracle_text: "Add {R} for each creature you control.".to_string(),
        abilities: vec![
            AbilityDefinition::Spell {
                effect: Effect::AddManaScaled {
                    player: PlayerTarget::Controller,
                    color: ManaColor::Red,
                    count: EffectAmount::PermanentCount {
                        filter: TargetFilter {
                            has_card_type: Some(CardType::Creature),
                            ..Default::default()
                        },
                        controller: PlayerTarget::Controller,
                    },
                },
                targets: vec![],
                modes: None,
                cant_be_countered: false,
            },
        ],
        ..Default::default()
    }
}
