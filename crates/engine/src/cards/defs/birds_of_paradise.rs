// 49. Birds of Paradise — {G}, Creature — Bird 0/1; Flying; {T}: add one mana
// of any color.
use crate::cards::helpers::*;

pub fn card() -> CardDefinition {
    CardDefinition {
        card_id: cid("birds-of-paradise"),
        name: "Birds of Paradise".to_string(),
        mana_cost: Some(ManaCost { green: 1, ..Default::default() }),
        types: creature_types(&["Bird"]),
        oracle_text: "Flying\n{T}: Add one mana of any color.".to_string(),
        power: Some(0),
        toughness: Some(1),
        abilities: vec![
            AbilityDefinition::Keyword(KeywordAbility::Flying),
            AbilityDefinition::Activated {
                cost: Cost::Tap,
                effect: Effect::AddManaAnyColor { player: PlayerTarget::Controller },
                timing_restriction: None,
                targets: vec![],
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
