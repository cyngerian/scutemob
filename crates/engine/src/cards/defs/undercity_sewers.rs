// Undercity Sewers
use crate::cards::helpers::*;

pub fn card() -> CardDefinition {
    CardDefinition {
        card_id: cid("undercity-sewers"),
        name: "Undercity Sewers".to_string(),
        mana_cost: None,
        types: types_sub(&[CardType::Land], &["Island", "Swamp"]),
        oracle_text: "({T}: Add {U} or {B}.)\nThis land enters tapped.\nWhen this land enters, surveil 1. (Look at the top card of your library. You may put it into your graveyard.)".to_string(),
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
            // CR 701.25: Surveil 1 on ETB.
            AbilityDefinition::Triggered {
                trigger_condition: TriggerCondition::WhenEntersBattlefield,
                effect: Effect::Surveil {
                    player: PlayerTarget::Controller,
                    count: EffectAmount::Fixed(1),
                },
                intervening_if: None,
                targets: vec![],
            },
            // Mana production handled by basic land subtypes Island/Swamp (CR 305.6).
        ],
        ..Default::default()
    }
}
