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
            // PB-18: "Each player can't cast more than one spell each turn."
            AbilityDefinition::StaticRestriction {
                restriction: GameRestriction::MaxSpellsPerTurn { max: 1 },
            },
            // DEFERRED (PB-18 review Finding 5): "Nonbasic lands your opponents control enter tapped."
            // This requires a conditional ETB-tapped replacement scoped to opponent-controlled
            // nonbasic lands. The replacement framework currently has no ETBTappedFilter that
            // checks (a) controller != self and (b) non-basic land type. Until a
            // ReplacementEffect::ETBTapped variant with OpponentNonbasicLand filter is added,
            // this ability produces wrong game state (nonbasic lands opponents control don't
            // enter tapped). Defer to PB-2 or a dedicated replacement-framework fix session.
        ],
        ..Default::default()
    }
}
