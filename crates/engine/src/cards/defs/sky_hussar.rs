// Sky Hussar — {3}{W}{U}, Creature — Human Knight 4/3; Flying; cast-during-upkeep trigger;
// Forecast — {W}{U}: Untap all creatures you control.
use crate::cards::helpers::*;

pub fn card() -> CardDefinition {
    CardDefinition {
        card_id: cid("sky-hussar"),
        name: "Sky Hussar".to_string(),
        mana_cost: Some(ManaCost { generic: 3, white: 1, blue: 1, ..Default::default() }),
        types: creature_types(&["Human", "Knight"]),
        oracle_text: "Flying\nWhen this creature enters, untap all creatures you control.\nForecast — Tap two untapped white and/or blue creatures you control, Reveal this card from your hand: Draw a card. (Activate only during your upkeep and only once each turn.)".to_string(),
        power: Some(4),
        toughness: Some(3),
        abilities: vec![
            AbilityDefinition::Keyword(KeywordAbility::Flying),
            // TODO: ETB trigger — "When this creature enters, untap all creatures you control."
            // Needs Effect::UntapAll { controller_only: true } which does not yet exist in the DSL.
            // UntapPermanent only untaps a single target (EffectTarget). Use DrawCards(0) as
            // structural placeholder until UntapAll is added.
            AbilityDefinition::Triggered {
                trigger_condition: TriggerCondition::WhenEntersBattlefield,
                effect: Effect::DrawCards {
                    player: PlayerTarget::Controller,
                    count: EffectAmount::Fixed(0),
                    // TODO: replace with Effect::UntapAll { filter: TargetFilter for creatures
                    // you control } once that Effect variant exists.
                },
                intervening_if: None,
                targets: vec![],
            },
            AbilityDefinition::Keyword(KeywordAbility::Forecast),
            // Forecast — {W}{U}: Untap all creatures you control.
            // TODO: replace placeholder effect with Effect::UntapAll { controller_only: true }
            // once that variant is added to the Effect enum.
            AbilityDefinition::Forecast {
                cost: ManaCost { white: 1, blue: 1, ..Default::default() },
                effect: Effect::DrawCards {
                    player: PlayerTarget::Controller,
                    count: EffectAmount::Fixed(0),
                    // TODO: replace with Effect::UntapAll { filter: creatures you control }
                },
            },
        ],
        color_indicator: None,
        back_face: None,
        spell_cost_modifiers: vec![],
        self_cost_reduction: None,
    }
}
