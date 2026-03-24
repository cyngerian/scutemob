// Vito, Thorn of the Dusk Rose — {2}{B}, Legendary Creature — Vampire Cleric 1/3
// Whenever you gain life, target opponent loses that much life.
// {3}{B}{B}: Creatures you control gain lifelink until end of turn.
//
// CR 604.2 / CR 613.1f: Activated ability produces a continuous lifelink grant.
// TODO: DSL gap — "Whenever you gain life, target opponent loses that much life." requires
// TriggerCondition::WhenYouGainLife and a way to track "that much life" as EffectAmount;
// neither exists in the DSL. Triggered ability omitted until those primitives are added.
use crate::cards::helpers::*;

pub fn card() -> CardDefinition {
    CardDefinition {
        card_id: cid("vito-thorn-of-the-dusk-rose"),
        name: "Vito, Thorn of the Dusk Rose".to_string(),
        mana_cost: Some(ManaCost { generic: 2, black: 1, ..Default::default() }),
        types: full_types(
            &[SuperType::Legendary],
            &[CardType::Creature],
            &["Vampire", "Cleric"],
        ),
        oracle_text: "Whenever you gain life, target opponent loses that much life.\n{3}{B}{B}: Creatures you control gain lifelink until end of turn.".to_string(),
        power: Some(1),
        toughness: Some(3),
        abilities: vec![
            // Whenever you gain life, target opponent loses 1 life.
            // TODO: "that much life" — needs EffectAmount::TriggeringAmount. Using Fixed(1) as partial.
            AbilityDefinition::Triggered {
                trigger_condition: TriggerCondition::WheneverYouGainLife,
                effect: Effect::LoseLife {
                    player: PlayerTarget::EachOpponent,
                    amount: EffectAmount::Fixed(1),
                },
                intervening_if: None,
                targets: vec![],
            },
            // "{3}{B}{B}: Creatures you control gain lifelink until end of turn."
            AbilityDefinition::Activated {
                cost: Cost::Mana(ManaCost { generic: 3, black: 2, ..Default::default() }),
                effect: Effect::ApplyContinuousEffect {
                    effect_def: Box::new(ContinuousEffectDef {
                        layer: EffectLayer::Ability,
                        modification: LayerModification::AddKeyword(KeywordAbility::Lifelink),
                        filter: EffectFilter::CreaturesYouControl,
                        duration: EffectDuration::UntilEndOfTurn,
                        condition: None,
                    }),
                },
                timing_restriction: None,
                targets: vec![],
                activation_condition: None,
            },
        ],
        ..Default::default()
    }
}
