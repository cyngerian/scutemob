// Devilish Valet — {2}{R}, Creature — Devil Warrior 1/3; Trample, Haste.
// Alliance — Whenever another creature you control enters, double this creature's power
// until end of turn.
// TODO: DSL gap — "double this creature's power" continuous effect not expressible
// (no LayerModification for power doubling).
use crate::cards::helpers::*;

pub fn card() -> CardDefinition {
    CardDefinition {
        card_id: cid("devilish-valet"),
        name: "Devilish Valet".to_string(),
        mana_cost: Some(ManaCost { generic: 2, red: 1, ..Default::default() }),
        types: creature_types(&["Devil", "Warrior"]),
        oracle_text: "Trample, haste\nAlliance \u{2014} Whenever another creature you control enters, double this creature's power until end of turn.".to_string(),
        power: Some(1),
        toughness: Some(3),
        abilities: vec![
            AbilityDefinition::Keyword(KeywordAbility::Trample),
            AbilityDefinition::Keyword(KeywordAbility::Haste),
        ],
        // TODO: Alliance trigger — "double this creature's power until EOT"
        // (requires LayerModification for multiplicative power change)
        ..Default::default()
    }
}
