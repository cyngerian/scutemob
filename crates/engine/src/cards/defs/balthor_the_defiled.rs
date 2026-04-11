// Balthor the Defiled — {2}{B}{B} Legendary Creature — Zombie Dwarf 2/2
// Minion creatures get +1/+1.
// {B}{B}{B}, Exile Balthor the Defiled: Each player returns all black and all red creature
// cards from their graveyard to the battlefield.
use crate::cards::helpers::*;

pub fn card() -> CardDefinition {
    CardDefinition {
        card_id: cid("balthor-the-defiled"),
        name: "Balthor the Defiled".to_string(),
        mana_cost: Some(ManaCost { generic: 2, black: 2, ..Default::default() }),
        types: full_types(
            &[SuperType::Legendary],
            &[CardType::Creature],
            &["Zombie", "Dwarf"],
        ),
        oracle_text: "Minion creatures get +1/+1.\n{B}{B}{B}, Exile Balthor the Defiled: Each player returns all black and all red creature cards from their graveyard to the battlefield.".to_string(),
        power: Some(2),
        toughness: Some(2),
        abilities: vec![
            // CR 613.1c / Layer 7c: Static "+1/+1 to all Minion creatures" (any controller).
            // AllCreaturesWithSubtype applies across all battlefields.
            AbilityDefinition::Static {
                continuous_effect: ContinuousEffectDef {
                    layer: EffectLayer::PtModify,
                    modification: LayerModification::ModifyBoth(1),
                    filter: EffectFilter::AllCreaturesWithSubtype(SubType("Minion".to_string())),
                    duration: EffectDuration::WhileSourceOnBattlefield,
                    condition: None,
                },
            },
            // CR 602.2 / CR 701.10: {B}{B}{B}, Exile Balthor — activated ability.
            // Cost::ExileSelf moves the source to exile before the ability resolves.
            // LKI: the effect is captured into embedded_effect in abilities.rs before exile,
            // so resolution succeeds even though the source is gone.
            // CR 400.7: after exile, Balthor's static ability ceases to apply.
            AbilityDefinition::Activated {
                cost: Cost::Sequence(vec![
                    Cost::Mana(ManaCost { black: 3, ..Default::default() }),
                    Cost::ExileSelf,
                ]),
                // CR 401.1: "each player returns ... from their graveyard to the battlefield"
                // EachPlayer graveyards + creature filter with colors Black or Red.
                // controller_override: None means permanents enter under their owners' control.
                effect: Effect::ReturnAllFromGraveyardToBattlefield {
                    graveyards: PlayerTarget::EachPlayer,
                    filter: TargetFilter {
                        has_card_type: Some(CardType::Creature),
                        colors: Some(
                            [Color::Black, Color::Red].into_iter().collect(),
                        ),
                        ..Default::default()
                    },
                    tapped: false,
                    controller_override: None,
                    unique_names: false,
                    permanent_cards_only: false,
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
