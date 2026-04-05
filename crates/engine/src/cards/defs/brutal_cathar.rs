// Brutal Cathar // Moonrage Brute — DFC with Daybound/Nightbound (CR 702.146)
// Front: {2}{W} Human Soldier Werewolf 2/2, Daybound,
//        when ETB exile target creature opponent controls until this leaves battlefield
// Back:  Moonrage Brute, Werewolf 3/3, Nightbound, first strike, ward {pay 3 life}
use crate::cards::helpers::*;

pub fn card() -> CardDefinition {
    CardDefinition {
        card_id: cid("brutal-cathar-moonrage-brute"),
        name: "Brutal Cathar".to_string(),
        mana_cost: Some(ManaCost { generic: 2, white: 1, ..Default::default() }),
        types: creature_types(&["Human", "Soldier", "Werewolf"]),
        oracle_text: "When this creature enters the battlefield, exile target creature an opponent controls until this creature leaves the battlefield.\nDaybound".to_string(),
        power: Some(2),
        toughness: Some(2),
        abilities: vec![
            AbilityDefinition::Keyword(KeywordAbility::Daybound),
            // When this creature enters, exile target creature an opponent controls until
            // this creature leaves the battlefield (CR 610.3).
            AbilityDefinition::Triggered {
                trigger_condition: TriggerCondition::WhenEntersBattlefield,
                effect: Effect::ExileWithDelayedReturn {
                    target: EffectTarget::DeclaredTarget { index: 0 },
                    return_timing: crate::state::stubs::DelayedTriggerTiming::WhenSourceLeavesBattlefield,
                    return_tapped: false,
                    return_to: crate::cards::card_definition::DelayedReturnDestination::Battlefield,
                },
                intervening_if: None,
                targets: vec![TargetRequirement::TargetPermanentWithFilter(TargetFilter {
                    has_card_type: Some(CardType::Creature),
                    controller: TargetController::Opponent,
                    ..Default::default()
                })],

                modes: None,
                trigger_zone: None,
            },
        ],
        color_indicator: None,
        back_face: Some(CardFace {
            name: "Moonrage Brute".to_string(),
            mana_cost: None,
            types: creature_types(&["Werewolf"]),
            oracle_text: "First strike\nWard—Pay 3 life.\nNightbound".to_string(),
            power: Some(3),
            toughness: Some(3),
            abilities: vec![
                AbilityDefinition::Keyword(KeywordAbility::Nightbound),
                AbilityDefinition::Keyword(KeywordAbility::FirstStrike),
                // DSL gap: Ward—Pay 3 life (Ward(u32) only supports mana costs)
            ],
            color_indicator: Some(vec![Color::White]),
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
    }
}
