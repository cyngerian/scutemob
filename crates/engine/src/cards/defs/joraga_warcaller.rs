// Joraga Warcaller — {G}, Creature — Elf Warrior 1/1
// Multikicker {1}{G}
// This creature enters with a +1/+1 counter on it for each time it was kicked.
// Other Elf creatures you control get +1/+1 for each +1/+1 counter on this creature.
use crate::cards::helpers::*;

pub fn card() -> CardDefinition {
    CardDefinition {
        card_id: cid("joraga-warcaller"),
        name: "Joraga Warcaller".to_string(),
        mana_cost: Some(ManaCost { green: 1, ..Default::default() }),
        types: creature_types(&["Elf", "Warrior"]),
        oracle_text: "Multikicker {1}{G}\nThis creature enters with a +1/+1 counter on it for each time it was kicked.\nOther Elf creatures you control get +1/+1 for each +1/+1 counter on this creature.".to_string(),
        power: Some(1),
        toughness: Some(1),
        abilities: vec![
            AbilityDefinition::Keyword(KeywordAbility::Kicker),
            AbilityDefinition::Kicker {
                cost: ManaCost { generic: 1, green: 1, ..Default::default() },
                is_multikicker: true,
            },
            // TODO: DSL gap — "enters with a +1/+1 counter for each time it was kicked"
            // Needs ETB replacement with count = kicker_times_paid.
            // TODO: DSL gap — "Other Elf creatures you control get +1/+1 for each +1/+1
            // counter on this creature." Needs dynamic ModifyBoth based on source counter count.
        ],
        ..Default::default()
    }
}
