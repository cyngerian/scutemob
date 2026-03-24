// Purphoros, God of the Forge — {3}{R}, Legendary Enchantment Creature — God 6/5
// Indestructible
// As long as your devotion to red is less than five, Purphoros isn't a creature.
// Whenever another creature you control enters, Purphoros deals 2 damage to each opponent.
// {2}{R}: Creatures you control get +1/+0 until end of turn.
//
// CR 700.5 / CR 604.2 / CR 613.1d (Layer 4): "As long as your devotion to red is less than
// five, Purphoros isn't a creature." Implemented as a conditional RemoveCardTypes static.
//
// TODO: "{2}{R}: Creatures you control get +1/+0 until end of turn." Requires an activated
// ability that applies a transient continuous effect to all your creatures. DSL gap for
// activated abilities that apply effects to all creatures you control.
use crate::cards::helpers::*;

pub fn card() -> CardDefinition {
    CardDefinition {
        card_id: cid("purphoros-god-of-the-forge"),
        name: "Purphoros, God of the Forge".to_string(),
        mana_cost: Some(ManaCost { generic: 3, red: 1, ..Default::default() }),
        types: full_types(
            &[SuperType::Legendary],
            &[CardType::Enchantment, CardType::Creature],
            &["God"],
        ),
        oracle_text: "Indestructible\nAs long as your devotion to red is less than five, Purphoros isn't a creature.\nWhenever another creature you control enters, Purphoros deals 2 damage to each opponent.\n{2}{R}: Creatures you control get +1/+0 until end of turn.".to_string(),
        power: Some(6),
        toughness: Some(5),
        abilities: vec![
            AbilityDefinition::Keyword(KeywordAbility::Indestructible),
            // CR 700.5 / CR 613.1d (Layer 4): "As long as your devotion to red is less than
            // five, Purphoros isn't a creature." Removes the Creature card type conditionally.
            // The threshold is devotion < 5 (i.e., 4 or fewer red mana symbols in mana costs
            // of permanents you control).
            AbilityDefinition::Static {
                continuous_effect: ContinuousEffectDef {
                    layer: EffectLayer::TypeChange,
                    modification: LayerModification::RemoveCardTypes(
                        [CardType::Creature].into_iter().collect(),
                    ),
                    filter: EffectFilter::Source,
                    duration: EffectDuration::WhileSourceOnBattlefield,
                    condition: Some(Condition::DevotionToColorsLessThan {
                        colors: vec![Color::Red],
                        threshold: 5,
                    }),
                },
            },
            // Whenever another creature you control enters, Purphoros deals 2 damage to each opponent.
            AbilityDefinition::Triggered {
                trigger_condition: TriggerCondition::WheneverCreatureEntersBattlefield {
                    filter: Some(TargetFilter {
                        controller: TargetController::You,
                        ..Default::default()
                    }),
                },
                effect: Effect::ForEach {
                    over: ForEachTarget::EachOpponent,
                    effect: Box::new(Effect::DealDamage {
                        target: EffectTarget::DeclaredTarget { index: 0 },
                        amount: EffectAmount::Fixed(2),
                    }),
                },
                intervening_if: None,
                targets: vec![],
            },
            // TODO: "{2}{R}: Creatures you control get +1/+0 until end of turn."
            // Requires activated ability applying a transient effect to all your creatures.
        ],
        ..Default::default()
    }
}
