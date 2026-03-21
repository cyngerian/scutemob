// Abstergo Entertainment — Legendary Land, {T}: Add {C}; {1},{T}: Add any color; sacrifice exile ability (TODO)
use crate::cards::helpers::*;

pub fn card() -> CardDefinition {
    CardDefinition {
        card_id: cid("abstergo-entertainment"),
        name: "Abstergo Entertainment".to_string(),
        mana_cost: None,
        types: supertypes(&[SuperType::Legendary], &[CardType::Land]),
        oracle_text: "{T}: Add {C}.\n{1}, {T}: Add one mana of any color.\n{3}, {T}, Exile Abstergo Entertainment: Return up to one target historic card from your graveyard to your hand, then exile all graveyards. (Artifacts, legendaries, and Sagas are historic.)".to_string(),
        abilities: vec![
            // {T}: Add {C}.
            AbilityDefinition::Activated {
                cost: Cost::Tap,
                effect: Effect::AddMana {
                    player: PlayerTarget::Controller,
                    mana: mana_pool(0, 0, 0, 0, 0, 1),
                },
                timing_restriction: None,
                targets: vec![],
                activation_condition: None,
            },
            // {1}, {T}: Add one mana of any color.
            AbilityDefinition::Activated {
                cost: Cost::Sequence(vec![
                    Cost::Mana(ManaCost { generic: 1, ..Default::default() }),
                    Cost::Tap,
                ]),
                effect: Effect::AddManaAnyColor {
                    player: PlayerTarget::Controller,
                },
                timing_restriction: None,
                targets: vec![],
                activation_condition: None,
            },
            // TODO: {3}, {T}, Exile Abstergo Entertainment: Return up to one target historic card
            // from your graveyard to your hand, then exile all graveyards.
            // DSL gaps: return_from_graveyard with historic filter; exile-self cost;
            // exile all graveyards effect not expressible.
        ],
        ..Default::default()
    }
}
