// Crypt of Agadeem — Land
// This land enters tapped.
// {T}: Add {B}.
// {2}, {T}: Add {B} for each black creature card in your graveyard.
use crate::cards::helpers::*;

pub fn card() -> CardDefinition {
    CardDefinition {
        card_id: cid("crypt-of-agadeem"),
        name: "Crypt of Agadeem".to_string(),
        mana_cost: None,
        types: types(&[CardType::Land]),
        oracle_text: "This land enters tapped.\n{T}: Add {B}.\n{2}, {T}: Add {B} for each black creature card in your graveyard.".to_string(),
        abilities: vec![
            // CR 614.1c: self-replacement — this land enters tapped.
            AbilityDefinition::Replacement {
                trigger: ReplacementTrigger::WouldEnterBattlefield {
                    filter: ObjectFilter::Any,
                },
                modification: ReplacementModification::EntersTapped,
                is_self: true,
                unless_condition: None,
            },
            // {T}: Add {B}.
            AbilityDefinition::Activated {
                cost: Cost::Tap,
                effect: Effect::AddMana {
                    player: PlayerTarget::Controller,
                    mana: mana_pool(0, 0, 1, 0, 0, 0),
                },
                timing_restriction: None,
                targets: vec![],
                activation_condition: None,
            },
            // {2}, {T}: Add {B} for each black creature card in your graveyard.
            AbilityDefinition::Activated {
                cost: Cost::Sequence(vec![
                    Cost::Mana(ManaCost { generic: 2, ..Default::default() }),
                    Cost::Tap,
                ]),
                effect: Effect::AddManaScaled {
                    player: PlayerTarget::Controller,
                    color: ManaColor::Black,
                    count: EffectAmount::CardCount {
                        zone: ZoneTarget::Graveyard {
                            owner: PlayerTarget::Controller,
                        },
                        player: PlayerTarget::Controller,
                        filter: Some(TargetFilter {
                            has_card_type: Some(CardType::Creature),
                            colors: Some(im::ordset![Color::Black]),
                            ..Default::default()
                        }),
                    },
                },
                timing_restriction: None,
                targets: vec![],
                activation_condition: None,
            },
        ],
        ..Default::default()
    }
}
