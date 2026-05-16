// Lathliss, Dragon Queen — {4}{R}{R}, Legendary Creature — Dragon 6/6
// Flying
// Whenever another nontoken Dragon you control enters, create a 5/5 red Dragon creature
// token with flying.
// {1}{R}: Dragons you control get +1/+0 until end of turn.
use crate::cards::helpers::*;

pub fn card() -> CardDefinition {
    CardDefinition {
        card_id: cid("lathliss-dragon-queen"),
        name: "Lathliss, Dragon Queen".to_string(),
        mana_cost: Some(ManaCost { generic: 4, red: 2, ..Default::default() }),
        types: full_types(&[SuperType::Legendary], &[CardType::Creature], &["Dragon"]),
        oracle_text: "Flying\nWhenever another nontoken Dragon you control enters, create a 5/5 red Dragon creature token with flying.\n{1}{R}: Dragons you control get +1/+0 until end of turn.".to_string(),
        power: Some(6),
        toughness: Some(6),
        abilities: vec![
            AbilityDefinition::Keyword(KeywordAbility::Flying),
            // TODO: ENGINE-BLOCKED — "Whenever another nontoken Dragon you control enters,
            // create a 5/5 red Dragon creature token with flying." The
            // WheneverCreatureEntersBattlefield trigger converts to an ETBTriggerFilter
            // (state/game_object.rs:560), which carries NO subtype field and NO token field —
            // only creature_only/controller_you/exclude_self/color_filter/card_type_filter.
            // Both the `has_subtype: Dragon` and the `nontoken` constraints would be silently
            // dropped at replay_harness.rs:2371, so the trigger would fire for every creature
            // (token or not) entering and mint a 5/5 Dragon each time. Needs ETBTriggerFilter
            // to carry subtype + nontoken filters (or the creature-ETB path to forward
            // triggering_creature_filter like the death-trigger path does). Authoring-only
            // batch — cannot make the engine change. The activated pump ability below IS
            // expressible and is implemented.
            // CR 613.4c: {1}{R}: Dragons you control get +1/+0 until end of turn.
            AbilityDefinition::Activated {
                cost: Cost::Mana(ManaCost { generic: 1, red: 1, ..Default::default() }),
                effect: Effect::ApplyContinuousEffect {
                    effect_def: Box::new(ContinuousEffectDef {
                        layer: EffectLayer::PtModify,
                        modification: LayerModification::ModifyPower(1),
                        filter: EffectFilter::CreaturesYouControlWithSubtype(
                            SubType("Dragon".to_string()),
                        ),
                        duration: EffectDuration::UntilEndOfTurn,
                        condition: None,
                    }),
                },
                timing_restriction: None,
                targets: vec![],
                activation_condition: None,
                activation_zone: None,
                once_per_turn: false,
            },
        ],
        ..Default::default()
    }
}
