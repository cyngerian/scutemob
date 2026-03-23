// Drana, Liberator of Malakir — {1}{B}{B}, Legendary Creature — Vampire Ally 2/3
// Flying, first strike
// Whenever Drana deals combat damage to a player, put a +1/+1 counter on each attacking
// creature you control.
use crate::cards::helpers::*;

pub fn card() -> CardDefinition {
    CardDefinition {
        card_id: cid("drana-liberator-of-malakir"),
        name: "Drana, Liberator of Malakir".to_string(),
        mana_cost: Some(ManaCost { generic: 1, black: 2, ..Default::default() }),
        types: full_types(
            &[SuperType::Legendary],
            &[CardType::Creature],
            &["Vampire", "Ally"],
        ),
        oracle_text: "Flying, first strike\nWhenever Drana, Liberator of Malakir deals combat damage to a player, put a +1/+1 counter on each attacking creature you control.".to_string(),
        power: Some(2),
        toughness: Some(3),
        abilities: vec![
            AbilityDefinition::Keyword(KeywordAbility::Flying),
            AbilityDefinition::Keyword(KeywordAbility::FirstStrike),
            // Whenever Drana deals combat damage to a player, +1/+1 on each attacker you control.
            AbilityDefinition::Triggered {
                trigger_condition: TriggerCondition::WhenDealsCombatDamageToPlayer,
                effect: Effect::ForEach {
                    over: ForEachTarget::EachAttackingCreature,
                    effect: Box::new(Effect::AddCounter {
                        target: EffectTarget::DeclaredTarget { index: 0 },
                        counter: CounterType::PlusOnePlusOne,
                        count: 1,
                    }),
                },
                intervening_if: None,
                targets: vec![],
            },
        ],
        ..Default::default()
    }
}
