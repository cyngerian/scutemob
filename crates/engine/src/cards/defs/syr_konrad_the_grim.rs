// Syr Konrad, the Grim — {3}{B}{B}, Legendary Creature — Human Knight 5/4
// Whenever another creature dies, or a creature card is put into a graveyard from
// anywhere other than the battlefield, or a creature card leaves your graveyard,
// Syr Konrad deals 1 damage to each opponent.
// {1}{B}: Each player mills a card.
use crate::cards::helpers::*;

pub fn card() -> CardDefinition {
    CardDefinition {
        card_id: cid("syr-konrad-the-grim"),
        name: "Syr Konrad, the Grim".to_string(),
        mana_cost: Some(ManaCost { generic: 3, black: 2, ..Default::default() }),
        types: full_types(
            &[SuperType::Legendary],
            &[CardType::Creature],
            &["Human", "Knight"],
        ),
        oracle_text: "Whenever another creature dies, or a creature card is put into a graveyard from anywhere other than the battlefield, or a creature card leaves your graveyard, Syr Konrad deals 1 damage to each opponent.\n{1}{B}: Each player mills a card.".to_string(),
        power: Some(5),
        toughness: Some(4),
        abilities: vec![
            // Trigger 1: Whenever another creature dies — WheneverCreatureDies approximation.
            // TODO: Full Syr Konrad requires 3 separate trigger conditions:
            // 1. Another creature dies (WheneverCreatureDies covers this but is overbroad — includes self)
            // 2. Creature card put into graveyard from non-battlefield (mill, discard) — no DSL trigger
            // 3. Creature card leaves your graveyard (exile, return) — no DSL trigger
            AbilityDefinition::Triggered {
                trigger_condition: TriggerCondition::WheneverCreatureDies { controller: None, exclude_self: true, nontoken_only: false, filter: None,
},
                effect: Effect::ForEach {
                    over: ForEachTarget::EachOpponent,
                    effect: Box::new(Effect::DealDamage {
                        target: EffectTarget::DeclaredTarget { index: 0 },
                        amount: EffectAmount::Fixed(1),
                    }),
                },
                intervening_if: None,
                targets: vec![],

                modes: None,
                trigger_zone: None,
            },
            // TODO: "{1}{B}: Each player mills a card." — needs Effect::Mill with ForEach::EachPlayer.
        ],
        ..Default::default()
    }
}
