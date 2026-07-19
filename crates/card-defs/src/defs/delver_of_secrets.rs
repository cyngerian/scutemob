// Delver of Secrets // Insectile Aberration — DFC with Transform (CR 701.28)
// Front: {U} Human Wizard 1/1, at beginning of your upkeep look at top card,
//        if instant or sorcery transform Delver of Secrets
// Back:  Insectile Aberration, Human Insect 3/2 flying (blue via color indicator)
//
// PB-OS6(a): upkeep trigger modeled as an unconditional AtBeginningOfYourUpkeep
// trigger whose effect is Effect::Conditional on Condition::TopCardIsInstantOrSorcery
// (mirrors heralds_horn.rs). Faithfully mandatory-if-true: revealing to transform is
// beneficial in effectively all realistic board states (a 1/1 becomes a 3/2 flier), so
// optimal play reveals -- unlike Herald's Horn (known_wrong, since "put into hand" can be
// undesirable). The rare cases where declining is correct (dodging flier/power-based
// removal, keeping the Human Wizard subtype for tribal, hidden-info bluff) don't affect
// the modeled game state; the mandatory model matches the engine's peek-conditional
// convention. Complete.
use crate::cards::helpers::*;

pub fn card() -> CardDefinition {
    CardDefinition {
        card_id: cid("delver-of-secrets-insectile-aberration"),
        name: "Delver of Secrets".to_string(),
        mana_cost: Some(ManaCost {
            blue: 1,
            ..Default::default()
        }),
        types: creature_types(&["Human", "Wizard"]),
        oracle_text: "At the beginning of your upkeep, look at the top card of your library. You \
                      may reveal that card. If an instant or sorcery card is revealed this way, \
                      transform Delver of Secrets."
            .to_string(),
        power: Some(1),
        toughness: Some(1),
        abilities: vec![
            AbilityDefinition::Keyword(KeywordAbility::Transform),
            // CR 400.2/614.1c: "At the beginning of your upkeep, look at the top card of
            // your library. You may reveal that card. If an instant or sorcery card is
            // revealed this way, transform Delver of Secrets."
            AbilityDefinition::Triggered {
                once_per_turn: false,
                trigger_condition: TriggerCondition::AtBeginningOfYourUpkeep,
                effect: Effect::Conditional {
                    condition: Condition::TopCardIsInstantOrSorcery,
                    if_true: Box::new(Effect::TransformSelf),
                    if_false: Box::new(Effect::Nothing),
                },
                intervening_if: None,
                targets: vec![],
                modes: None,
                trigger_zone: None,
            },
        ],
        color_indicator: None,
        back_face: Some(CardFace {
            name: "Insectile Aberration".to_string(),
            mana_cost: None,
            types: creature_types(&["Human", "Insect"]),
            oracle_text: "Flying".to_string(),
            power: Some(3),
            toughness: Some(2),
            abilities: vec![AbilityDefinition::Keyword(KeywordAbility::Flying)],
            color_indicator: Some(vec![Color::Blue]),
        }),
        spell_cost_modifiers: vec![],
        self_cost_reduction: None,
        starting_loyalty: None,
        adventure_face: None,
        meld_pair: None,
        spell_additional_costs: vec![],
        activated_ability_cost_reductions: vec![],
        cant_be_countered: false,
        self_exile_on_resolution: false,
        self_shuffle_on_resolution: false,
        completeness: Completeness::Complete,
    }
}
