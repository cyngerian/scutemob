// General Kreat, the Boltbringer — {2}{R}, Legendary Creature — Goblin Soldier 2/2
// Whenever one or more Goblins you control attack, create a 1/1 red Goblin creature
// token that's tapped and attacking.
// Whenever another creature you control enters, General Kreat deals 1 damage to each opponent.
//
// PB-OS11 (forced add, self-identified TODO): "Whenever one or more Goblins you control
// attack" is a BATCH trigger (CR 508.1m) -- fires ONCE per combat if at least one Goblin
// attacked, not once per matching attacker. TriggerCondition::WheneverYouAttack{filter}
// (PB-OS11) expresses this directly via has_subtype: Goblin on the declared-attacker set.
use crate::cards::helpers::*;

pub fn card() -> CardDefinition {
    CardDefinition {
        card_id: cid("general-kreat-the-boltbringer"),
        name: "General Kreat, the Boltbringer".to_string(),
        mana_cost: Some(ManaCost {
            generic: 2,
            red: 1,
            ..Default::default()
        }),
        types: full_types(
            &[SuperType::Legendary],
            &[CardType::Creature],
            &["Goblin", "Soldier"],
        ),
        oracle_text: "Whenever one or more Goblins you control attack, create a 1/1 red Goblin \
                      creature token that's tapped and attacking.\nWhenever another creature you \
                      control enters, General Kreat deals 1 damage to each opponent."
            .to_string(),
        power: Some(2),
        toughness: Some(2),
        abilities: vec![
            // "Whenever one or more Goblins you control attack, create a 1/1 red Goblin
            // creature token that's tapped and attacking." (CR 508.1m batch trigger.)
            AbilityDefinition::Triggered {
                once_per_turn: false,
                trigger_condition: TriggerCondition::WheneverYouAttack {
                    filter: Some(TargetFilter {
                        has_subtype: Some(SubType("Goblin".to_string())),
                        controller: TargetController::You,
                        ..Default::default()
                    }),
                },
                effect: Effect::CreateToken {
                    spec: TokenSpec {
                        name: "Goblin".to_string(),
                        card_types: [CardType::Creature].into_iter().collect(),
                        subtypes: [SubType("Goblin".to_string())].into_iter().collect(),
                        colors: [Color::Red].into_iter().collect(),
                        power: 1,
                        toughness: 1,
                        count: EffectAmount::Fixed(1),
                        tapped: true,
                        enters_attacking: true,
                        ..Default::default()
                    },
                },
                intervening_if: None,
                targets: vec![],
                modes: None,
                trigger_zone: None,
            },
            // Whenever another creature you control enters, General Kreat deals 1 damage to each opponent.
            AbilityDefinition::Triggered {
                once_per_turn: false,
                trigger_condition: TriggerCondition::WheneverCreatureEntersBattlefield {
                    filter: Some(TargetFilter {
                        controller: TargetController::You,
                        ..Default::default()
                    }),
                    exclude_self: true,
                },
                effect: Effect::ForEach {
                    over: ForEachTarget::EachOpponent,
                    effect: Box::new(Effect::DealDamage {
                        source: None,
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
        completeness: Completeness::Complete,
        ..Default::default()
    }
}
