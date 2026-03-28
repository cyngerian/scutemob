// Earthquake Dragon — {14}{G}, Creature — Elemental Dragon 10/10
// This spell costs {X} less to cast, where X is the total mana value of Dragons you control.
// Flying, trample
// {2}{G}, Sacrifice a land: Return this card from your graveyard to your hand.
//
// CR 602.2 / PB-35: Graveyard-activated ability. Sacrifice a land (Cost::Sacrifice with
// land filter) + mana cost, activated from the graveyard zone.
use crate::cards::helpers::*;

pub fn card() -> CardDefinition {
    CardDefinition {
        card_id: cid("earthquake-dragon"),
        name: "Earthquake Dragon".to_string(),
        mana_cost: Some(ManaCost { generic: 14, green: 1, ..Default::default() }),
        types: creature_types(&["Elemental", "Dragon"]),
        oracle_text: "This spell costs {X} less to cast, where X is the total mana value of Dragons you control.\nFlying, trample\n{2}{G}, Sacrifice a land: Return this card from your graveyard to your hand.".to_string(),
        power: Some(10),
        toughness: Some(10),
        abilities: vec![
            AbilityDefinition::Keyword(KeywordAbility::Flying),
            AbilityDefinition::Keyword(KeywordAbility::Trample),
            // CR 602.2 / PB-35: "{2}{G}, Sacrifice a land: Return this card from your
            // graveyard to your hand." — activated from the graveyard zone.
            AbilityDefinition::Activated {
                cost: Cost::Sequence(vec![
                    Cost::Mana(ManaCost { generic: 2, green: 1, ..Default::default() }),
                    Cost::Sacrifice(TargetFilter {
                        has_card_type: Some(CardType::Land),
                        ..Default::default()
                    }),
                ]),
                effect: Effect::MoveZone {
                    target: EffectTarget::Source,
                    to: ZoneTarget::Hand { owner: PlayerTarget::Controller },
                    controller_override: None,
                },
                timing_restriction: None,
                targets: vec![],
                activation_condition: None,
                activation_zone: Some(ActivationZone::Graveyard),
            },
        ],
        self_cost_reduction: Some(SelfCostReduction::TotalManaValue {
            filter: TargetFilter {
                has_subtype: Some(SubType("Dragon".to_string())),
                ..Default::default()
            },
        }),
        ..Default::default()
    }
}
