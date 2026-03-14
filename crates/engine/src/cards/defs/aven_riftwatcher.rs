// Aven Riftwatcher — {2}{W}, Creature — Bird Rebel Soldier 2/3
// Flying, Vanishing 3, ETB/LTB gain 2 life
use crate::cards::helpers::*;

pub fn card() -> CardDefinition {
    CardDefinition {
        card_id: cid("aven-riftwatcher"),
        name: "Aven Riftwatcher".to_string(),
        mana_cost: Some(ManaCost { generic: 2, white: 1, ..Default::default() }),
        types: creature_types(&["Bird", "Rebel", "Soldier"]),
        oracle_text: "Flying\nVanishing 3 (This creature enters with three time counters on it. At the beginning of your upkeep, remove a time counter from it. When the last is removed, sacrifice it.)\nWhen this creature enters or leaves the battlefield, you gain 2 life."
            .to_string(),
        power: Some(2),
        toughness: Some(3),
        abilities: vec![
            AbilityDefinition::Keyword(KeywordAbility::Flying),
            AbilityDefinition::Vanishing { count: 3 },
            AbilityDefinition::Triggered {
                trigger_condition: TriggerCondition::WhenEntersBattlefield,
                effect: Effect::GainLife {
                    player: PlayerTarget::Controller,
                    amount: EffectAmount::Fixed(2),
                },
                intervening_if: None,
                targets: vec![],
            },
            // TODO: WhenLeavesBattlefield trigger condition not yet implemented in DSL.
            // The second half of "When this creature enters or leaves the battlefield,
            // you gain 2 life" cannot be expressed until TriggerCondition::WhenLeavesBattlefield
            // is added. Add a second Triggered ability here when that variant exists.
        ],
        color_indicator: None,
        back_face: None,
    }
}
