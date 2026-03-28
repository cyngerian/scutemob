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
            // TODO: "Whenever another nontoken Dragon enters" — nontoken subtype-filtered ETB
            // trigger not in DSL (blocked on PB-26).
            // CR 613.4c: "{1}{R}: Dragons you control get +1/+0 until end of turn."
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
            },
        ],
        ..Default::default()
    }
}
