// Throne of Eldraine — {5}, Legendary Artifact
// As Throne of Eldraine enters, choose a color.
// {T}: Add four mana of the chosen color. Spend this mana only to cast monocolored
// spells of that color.
// {3}, {T}: Draw two cards. Spend only mana of the chosen color to activate this ability.
//
// TODO (PB-Q2 / PB-spending-restriction): "Spend this mana only to cast monocolored
// spells of that color" — mana-spending restriction primitive not yet in DSL.
// TODO (PB-Q2 / PB-spending-restriction): "{3},{T}: Draw two cards. Spend only mana of
// the chosen color to activate this ability" — activation mana restriction not yet in DSL.
use crate::cards::helpers::*;

pub fn card() -> CardDefinition {
    CardDefinition {
        card_id: cid("throne-of-eldraine"),
        name: "Throne of Eldraine".to_string(),
        mana_cost: Some(ManaCost { generic: 5, ..Default::default() }),
        types: supertypes(&[SuperType::Legendary], &[CardType::Artifact]),
        oracle_text: "As Throne of Eldraine enters, choose a color.\n{T}: Add four mana of the chosen color. Spend this mana only to cast monocolored spells of that color.\n{3}, {T}: Draw two cards. Spend only mana of the chosen color to activate this ability.".to_string(),
        abilities: vec![
            // CR 614.12 / CR 614.12a: "As this enters, choose a color."
            // Replacement effect — NOT a triggered ability (PB-X C1 lesson).
            // Default: White (arbitrary; deterministic fallback overrides at ETB time).
            AbilityDefinition::Replacement {
                trigger: ReplacementTrigger::WouldEnterBattlefield {
                    filter: ObjectFilter::Any,
                },
                modification: ReplacementModification::ChooseColor(Color::White),
                is_self: true,
                unless_condition: None,
            },
            // {T}: Add four mana of the chosen color.
            // Effect::AddManaOfChosenColor reads chosen_color from this object at execution time.
            AbilityDefinition::Activated {
                cost: Cost::Tap,
                effect: Effect::AddManaOfChosenColor {
                    player: PlayerTarget::Controller,
                    amount: 4,
                },
                timing_restriction: None,
                targets: vec![],
                activation_condition: None,
                activation_zone: None,
                once_per_turn: false,
            },
            // {3},{T}: Draw two cards.
            // TODO (PB-spending-restriction): "Spend only mana of the chosen color to activate".
            AbilityDefinition::Activated {
                cost: Cost::Sequence(vec![
                    Cost::Mana(ManaCost { generic: 3, ..Default::default() }),
                    Cost::Tap,
                ]),
                effect: Effect::DrawCards {
                    player: PlayerTarget::Controller,
                    count: EffectAmount::Fixed(2),
                },
                timing_restriction: None,
                targets: vec![],
                activation_condition: None,
                activation_zone: None,
                once_per_turn: false,
            },
        ],
        ..Default::default()
    }
}
