// 49. Birds of Paradise — {G}, Creature — Bird 0/1; Flying; {T}: add one mana
// of any color.
use crate::cards::helpers::*;

pub fn card() -> CardDefinition {
    CardDefinition {
        card_id: cid("birds-of-paradise"),
        name: "Birds of Paradise".to_string(),
        mana_cost: Some(ManaCost {
            green: 1,
            ..Default::default()
        }),
        types: creature_types(&["Bird"]),
        oracle_text: "Flying\n{T}: Add one mana of any color.".to_string(),
        power: Some(0),
        toughness: Some(1),
        abilities: vec![
            AbilityDefinition::Keyword(KeywordAbility::Flying),
            AbilityDefinition::Activated {
                cost: Cost::Tap,
                effect: Effect::AddManaAnyColor {
                    player: PlayerTarget::Controller,
                },
                timing_restriction: None,
                targets: vec![],
                activation_condition: None,
                activation_zone: None,
                once_per_turn: false,
                modes: None,
            },
        ],
        color_indicator: None,
        back_face: None,
        spell_cost_modifiers: vec![],
        self_cost_reduction: None,
        starting_loyalty: None,
        adventure_face: None,
        meld_pair: None,
        spell_additional_costs: vec![],
        activated_ability_cost_reductions: vec![],
        cant_be_countered: false,
        self_exile_on_resolution: false,
        self_shuffle_on_resolution: false,
        // PB-EF12 (EF-W-PB2-3): un-marked. `any_color: true` mana abilities now resolve
        // to a real chosen colour (CR 111.10a/605.3b) via `Command::TapForMana.chosen_color`,
        // not ManaColor::Colorless (was SF-11 / SR-37).
        completeness: Completeness::Complete,
    }
}
