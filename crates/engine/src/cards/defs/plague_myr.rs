// Plague Myr — {2}, Artifact Creature — Phyrexian Myr 1/1
// Infect; {T}: Add {C}.
use crate::cards::helpers::*;

pub fn card() -> CardDefinition {
    CardDefinition {
        card_id: cid("plague-myr"),
        name: "Plague Myr".to_string(),
        mana_cost: Some(ManaCost { generic: 2, ..Default::default() }),
        types: types_sub(&[CardType::Artifact, CardType::Creature], &["Phyrexian", "Myr"]),
        oracle_text: "Infect (This creature deals damage to creatures in the form of -1/-1 counters and to players in the form of poison counters.)\n{T}: Add {C}.".to_string(),
        power: Some(1),
        toughness: Some(1),
        abilities: vec![
            AbilityDefinition::Keyword(KeywordAbility::Infect),
            AbilityDefinition::Activated {
                cost: Cost::Tap,
                effect: Effect::AddMana {
                    player: PlayerTarget::Controller,
                    mana: mana_pool(0, 0, 0, 0, 0, 1),
                },
                timing_restriction: None,
                targets: vec![],
                activation_condition: None,
                activation_zone: None,
            },
        ],
        ..Default::default()
    }
}
