// Nethergoyf — {B}, Creature — Lhurgoyf 0/1+*
// Nethergoyf's power is equal to the number of card types among cards in your
// graveyard and its toughness is equal to that number plus 1.
// Escape — {2}{B}, Exile three other cards from your graveyard.
use crate::cards::helpers::*;

pub fn card() -> CardDefinition {
    CardDefinition {
        card_id: cid("nethergoyf"),
        name: "Nethergoyf".to_string(),
        mana_cost: Some(ManaCost { black: 1, ..Default::default() }),
        types: creature_types(&["Lhurgoyf"]),
        oracle_text: "Nethergoyf's power is equal to the number of card types among cards in your graveyard and its toughness is equal to that number plus 1.\nEscape—{2}{B}, Exile three other cards from your graveyard. (You may cast this card from your graveyard for its escape cost.)".to_string(),
        power: None,
        toughness: None,
        abilities: vec![
            // TODO: CDA for P/T = card types in graveyard. Needs EffectAmount::CardTypesInGraveyard.
            // Using CdaPowerToughness stub.
            AbilityDefinition::Keyword(KeywordAbility::Escape),
            AbilityDefinition::AltCastAbility {
                kind: AltCostKind::Escape,
                details: Some(AltCastDetails::Escape { exile_count: 3 }),
                cost: ManaCost { generic: 2, black: 1, ..Default::default() },
            },
        ],
        ..Default::default()
    }
}
