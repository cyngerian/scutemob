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
                activation_zone: None,
                once_per_turn: false,
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
                            colors: Some(imbl::ordset![Color::Black]),
                            ..Default::default()
                        }),
                    },
                },
                timing_restriction: None,
                targets: vec![],
                activation_condition: None,
                activation_zone: None,
                once_per_turn: false,
            },
        ],
        completeness: Completeness::partial("CR 605.1a/605.3b: the '{T}: Add {B}' ability IS a correctly registered mana ability, but '{2},{T}: Add {B} for each black creature card in your graveyard' is registered as a stack-using activated ability. The AMOUNT is correct (probed: 3 black creature cards in graveyard -> +3 black). Same Effect::AddManaScaled exclusion as Cabal Coffers; blocked on SF-8."),
        ..Default::default()
    }
}
