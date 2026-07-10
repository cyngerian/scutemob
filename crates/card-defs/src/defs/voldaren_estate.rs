// Voldaren Estate — Land
// {T}: Add {C}.
// {T}, Pay 1 life: Add one mana of any color. Spend this mana only to cast a Vampire spell.
// {5}, {T}: Create a Blood token. This ability costs {1} less for each Vampire you control.
use crate::cards::helpers::*;

pub fn card() -> CardDefinition {
    CardDefinition {
        card_id: cid("voldaren-estate"),
        name: "Voldaren Estate".to_string(),
        mana_cost: None,
        types: types(&[CardType::Land]),
        oracle_text: "{T}: Add {C}.\n{T}, Pay 1 life: Add one mana of any color. Spend this mana only to cast a Vampire spell.\n{5}, {T}: Create a Blood token. This ability costs {1} less to activate for each Vampire you control.".to_string(),
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
                activation_zone: None,
            once_per_turn: false,
            },
            // {T}, Pay 1 life: Add one mana of any color. Spend this mana only to cast a Vampire spell.
            AbilityDefinition::Activated {
                cost: Cost::Sequence(vec![Cost::Tap, Cost::PayLife(1)]),
                effect: Effect::AddManaAnyColorRestricted {
                    player: PlayerTarget::Controller,
                    restriction: ManaRestriction::SubtypeOnly(SubType("Vampire".to_string())),
                },
                timing_restriction: None,
                targets: vec![],
                activation_condition: None,
                activation_zone: None,
            once_per_turn: false,
            },
            // {5}, {T}: Create a Blood token. This ability costs {1} less for each Vampire you control.
            AbilityDefinition::Activated {
                cost: Cost::Sequence(vec![
                    Cost::Mana(ManaCost { generic: 5, ..Default::default() }),
                    Cost::Tap,
                ]),
                effect: Effect::CreateToken {
                    spec: blood_token_spec(1),
                },
                timing_restriction: None,
                targets: vec![],
                activation_condition: None,
                activation_zone: None,
            once_per_turn: false,
            },
        ],
        // CR 602.2b + 601.2f: Blood token ability (activated_ability index 1) costs {1} less
        // per Vampire controller has. The {T}:Add{C} tap ability goes to mana_abilities
        // (index excluded). The {T},pay 1 life ability is activated_ability index 0.
        // The blood token {5}{T} ability is activated_ability index 1.
        activated_ability_cost_reductions: vec![(
            1,
            SelfActivatedCostReduction::PerPermanent {
                per: 1,
                filter: TargetFilter {
                    has_subtype: Some(SubType("Vampire".to_string())),
                    ..Default::default()
                },
                controller: PlayerTarget::Controller,
            },
        )],
        ..Default::default()
    }
}
