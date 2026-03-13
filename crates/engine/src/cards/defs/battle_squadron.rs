// Battle Squadron — {3}{R}{R}, Creature — Goblin 2/2 (*/* characteristic-defining ability)
// Flying; power and toughness each equal to the number of creatures you control.
use crate::cards::helpers::*;

pub fn card() -> CardDefinition {
    CardDefinition {
        card_id: cid("battle-squadron"),
        name: "Battle Squadron".to_string(),
        mana_cost: Some(ManaCost { generic: 3, red: 2, ..Default::default() }),
        types: creature_types(&["Goblin"]),
        oracle_text: "Flying\nBattle Squadron's power and toughness are each equal to the number of creatures you control.".to_string(),
        power: Some(0),
        toughness: Some(0),
        abilities: vec![
            AbilityDefinition::Keyword(KeywordAbility::Flying),
            // TODO: DSL gap — characteristic-defining ability (CDA) setting P/T equal to number
            // of creatures you control requires a Layer 7b continuous effect; not expressible.
        ],
        ..Default::default()
    }
}
