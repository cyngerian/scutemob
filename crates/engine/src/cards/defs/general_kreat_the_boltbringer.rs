// General Kreat, the Boltbringer — {2}{R}, Legendary Creature — Goblin Soldier 2/2
// Whenever one or more Goblins you control attack, create a 1/1 red Goblin creature
// token that's tapped and attacking.
// Whenever another creature you control enters, General Kreat deals 1 damage to each opponent.
use crate::cards::helpers::*;

pub fn card() -> CardDefinition {
    CardDefinition {
        card_id: cid("general-kreat-the-boltbringer"),
        name: "General Kreat, the Boltbringer".to_string(),
        mana_cost: Some(ManaCost { generic: 2, red: 1, ..Default::default() }),
        types: full_types(
            &[SuperType::Legendary],
            &[CardType::Creature],
            &["Goblin", "Soldier"],
        ),
        oracle_text: "Whenever one or more Goblins you control attack, create a 1/1 red Goblin creature token that's tapped and attacking.\nWhenever another creature you control enters, General Kreat deals 1 damage to each opponent.".to_string(),
        power: Some(2),
        toughness: Some(2),
        abilities: vec![
            // TODO: "Whenever one or more Goblins you control attack" — needs subtype-filtered
            // attack trigger + tapped-and-attacking token creation.
            // Whenever another creature you control enters, deal 1 to each opponent.
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
                        amount: EffectAmount::Fixed(1),
                    }),
                },
                intervening_if: None,
                targets: vec![],

                modes: None,
                trigger_zone: None,
            },
        ],
        ..Default::default()
    }
}
