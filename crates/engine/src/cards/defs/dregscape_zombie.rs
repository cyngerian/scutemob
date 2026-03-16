// 74. Dregscape Zombie — {1}{B}, Creature — Zombie 2/1;
// Unearth {B} (Pay {B}: Return this card from your graveyard to the battlefield.
// It gains haste. Exile it at the beginning of the next end step or if it would
// leave the battlefield. Unearth only as a sorcery.)
use crate::cards::helpers::*;

pub fn card() -> CardDefinition {
    CardDefinition {
        card_id: cid("dregscape-zombie"),
        name: "Dregscape Zombie".to_string(),
        mana_cost: Some(ManaCost { generic: 1, black: 1, ..Default::default() }),
        types: creature_types(&["Zombie"]),
        oracle_text: "Unearth {B} (Pay {B}: Return this card from your graveyard to the battlefield. It gains haste. Exile it at the beginning of the next end step or if it would leave the battlefield. Unearth only as a sorcery.)".to_string(),
        power: Some(2),
        toughness: Some(1),
        abilities: vec![
            // CR 702.84a: Unearth keyword marker for quick presence-check.
            AbilityDefinition::Keyword(KeywordAbility::Unearth),
            // CR 702.84a: Unearth cost ({B}).
            AbilityDefinition::AltCastAbility {
                kind: AltCostKind::Unearth,
                cost: ManaCost { black: 1, ..Default::default() },
                details: None,
            },
        ],
        color_indicator: None,
        back_face: None,
        spell_cost_modifiers: vec![],
        self_cost_reduction: None,
        starting_loyalty: None,
        meld_pair: None,
    }
}
