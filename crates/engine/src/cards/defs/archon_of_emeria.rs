// Archon of Emeria — {2}{W}, Creature — Archon 2/3
// Flying; each player can't cast more than one spell each turn;
// nonbasic lands opponents control enter tapped.
use crate::cards::helpers::*;

pub fn card() -> CardDefinition {
    CardDefinition {
        card_id: cid("archon-of-emeria"),
        name: "Archon of Emeria".to_string(),
        mana_cost: Some(ManaCost { generic: 2, white: 1, ..Default::default() }),
        types: creature_types(&["Archon"]),
        oracle_text: "Flying\nEach player can't cast more than one spell each turn.\nNonbasic lands your opponents control enter tapped.".to_string(),
        power: Some(2),
        toughness: Some(3),
        abilities: vec![
            AbilityDefinition::Keyword(KeywordAbility::Flying),
            // TODO: DSL gap — "each player can't cast more than one spell each turn" is a
            // global spell-frequency restriction; no CastRestriction continuous effect type exists.
            // TODO: DSL gap — ETB-tapped replacement on nonbasic lands opponents control requires
            // a conditional ReplacementTrigger scoped to opponent-controlled nonbasic lands; not supported.
        ],
        ..Default::default()
    }
}
