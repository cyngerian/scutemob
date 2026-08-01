// Jadar, Ghoulcaller of Nephalia — {1}{B}, Legendary Creature — Human Wizard 1/1
// At the beginning of your end step, if you control no creatures with decayed,
// create a 2/2 black Zombie creature token with decayed.
// (It can't block. When it attacks, sacrifice it at end of combat.)
//
// CR 702.147a: Decayed — can't block; sacrifice at end of combat after attacking.
// CR 603.1: Triggered ability fires at beginning of controller's end step with intervening-if.
// CR 111.10: Token is created with the Decayed keyword in its characteristics.
//
// PB-DX3b (OOS-DX3-1, 2026-08-01): the stored `oracle_text` and this file comment were
// WRONG, not merely blocked — they said "no tokens named Shambling Ghast" (a filter the
// printed card never had). MCP-verified printed text is "if you control no creatures with
// decayed." That filter is expressible today: `Condition::YouControlNOrMoreWithFilter` with
// a Creature + Decayed-keyword TargetFilter, negated. Both PB-DP6 (queue-time, end-step
// CardDef sweep, `rules/turn_actions.rs:781`) and PB-DX1 (resolution-time re-check,
// `InterveningIf::CardDef`) already gate this trigger moment. See the fixed intervening_if
// below.
use crate::cards::helpers::*;

pub fn card() -> CardDefinition {
    CardDefinition {
        card_id: cid("jadar-ghoulcaller-of-nephalia"),
        name: "Jadar, Ghoulcaller of Nephalia".to_string(),
        mana_cost: Some(ManaCost {
            generic: 1,
            black: 1,
            ..Default::default()
        }),
        types: full_types(
            &[SuperType::Legendary],
            &[CardType::Creature],
            &["Human", "Wizard"],
        ),
        oracle_text: "At the beginning of your end step, if you control no creatures with \
                      decayed, create a 2/2 black Zombie creature token with decayed. (It can't \
                      block. When it attacks, sacrifice it at end of combat.)"
            .to_string(),
        power: Some(1),
        toughness: Some(1),
        abilities: vec![
            // CR 603.1/603.4: End step trigger, intervening-if "you control no creatures
            // with decayed". Checked at queue time (rules/turn_actions.rs:781, comment
            // names this exact card) and re-checked at resolution
            // (InterveningIf::CardDef, PB-DX1). Checked against LAYER-RESOLVED
            // characteristics (effects/mod.rs's `expect_characteristics` call inside the
            // YouControlNOrMoreWithFilter evaluator), so a Humility-style effect that strips
            // Decayed correctly re-enables the trigger.
            AbilityDefinition::Triggered {
                once_per_turn: false,
                trigger_condition: TriggerCondition::AtBeginningOfYourEndStep,
                effect: Effect::CreateToken {
                    spec: zombie_decayed_token_spec(1),
                },
                // NOTE: TargetFilter.controller is left at its default (TargetController::Any)
                // deliberately, not set to `You`. The YouControlNOrMoreWithFilter evaluator
                // does its own `obj.controller == controller` check
                // (effects/mod.rs::check_static_condition) using ctx.controller — it does not
                // read TargetFilter.controller at all (matches_filter takes only
                // &Characteristics, which has no controller field). Setting it here would
                // imply a restriction the predicate does not enforce.
                intervening_if: Some(Condition::Not(Box::new(
                    Condition::YouControlNOrMoreWithFilter {
                        count: 1,
                        filter: TargetFilter {
                            has_card_type: Some(CardType::Creature),
                            has_keywords: [KeywordAbility::Decayed].into_iter().collect(),
                            ..Default::default()
                        },
                    },
                ))),
                targets: vec![],

                modes: None,
                trigger_zone: None,
            },
        ],
        color_indicator: None,
        back_face: None,
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
