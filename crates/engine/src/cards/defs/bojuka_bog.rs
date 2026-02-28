// Bojuka Bog — Land.
// "Bojuka Bog enters the battlefield tapped."
// "When Bojuka Bog enters the battlefield, exile all cards from target player's graveyard."
// {T}: Add {B}.
//
// Simplification: "target player's graveyard" → all graveyards. Triggered abilities
// cannot declare targets in the current DSL, so this exiles every player's graveyard.
// In practice this is more powerful than the card (hits all players), but correct for
// engine testing purposes.
use crate::cards::helpers::*;

pub fn card() -> CardDefinition {
    CardDefinition {
        card_id: cid("bojuka-bog"),
        name: "Bojuka Bog".to_string(),
        mana_cost: None,
        types: types(&[CardType::Land]),
        oracle_text: "Bojuka Bog enters the battlefield tapped.\nWhen Bojuka Bog enters the battlefield, exile all cards from target player's graveyard.\n{T}: Add {B}.".to_string(),
        abilities: vec![
            // CR 614.1c: self-replacement — enters tapped.
            AbilityDefinition::Replacement {
                trigger: ReplacementTrigger::WouldEnterBattlefield {
                    filter: ObjectFilter::Any,
                },
                modification: ReplacementModification::EntersTapped,
                is_self: true,
            },
            // ETB triggered: exile all graveyards (simplified from "target player's graveyard").
            AbilityDefinition::Triggered {
                trigger_condition: TriggerCondition::WhenEntersBattlefield,
                effect: Effect::ForEach {
                    over: ForEachTarget::EachCardInAllGraveyards,
                    effect: Box::new(Effect::ExileObject {
                        target: EffectTarget::DeclaredTarget { index: 0 },
                    }),
                },
                intervening_if: None,
            },
            // {T}: Add {B}.
            AbilityDefinition::Activated {
                cost: Cost::Tap,
                effect: Effect::AddMana {
                    player: PlayerTarget::Controller,
                    mana: mana_pool(0, 0, 1, 0, 0, 0),
                },
                timing_restriction: None,
            },
        ],
        ..Default::default()
    }
}
