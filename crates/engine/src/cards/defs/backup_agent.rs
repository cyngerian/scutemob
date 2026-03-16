// Backup Agent — {2}{W}, Creature — Human Soldier 2/3; Backup 1, Lifelink.
// CR 702.165: Backup 1 grants Lifelink to the target if it is another creature.
use crate::cards::helpers::*;

pub fn card() -> CardDefinition {
    CardDefinition {
        card_id: cid("backup-agent"),
        name: "Backup Agent".to_string(),
        mana_cost: Some(ManaCost { generic: 2, white: 1, ..Default::default() }),
        types: creature_types(&["Human", "Soldier"]),
        oracle_text: "Backup 1 (When this creature enters the battlefield, put a +1/+1 counter on target creature. If that creature is another creature, it gains the following abilities until end of turn.)\nLifelink".to_string(),
        power: Some(2),
        toughness: Some(3),
        abilities: vec![
            // CR 702.165a: Backup trigger fires on ETB; abilities listed below Backup are
            // granted to the target if it is a different creature than this one.
            AbilityDefinition::Keyword(KeywordAbility::Backup(1)),
            // Lifelink is below Backup in the definition — engine snapshots this as the
            // granted ability set (CR 702.165d).
            AbilityDefinition::Keyword(KeywordAbility::Lifelink),
        ],
        color_indicator: None,
        back_face: None,
        spell_cost_modifiers: vec![],
        self_cost_reduction: None,
        starting_loyalty: None,
        meld_pair: None,
    }
}
