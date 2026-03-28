// Ramos, Dragon Engine — {6}, Legendary Artifact Creature — Dragon 4/4
// Flying
// Whenever you cast a spell, put a +1/+1 counter on Ramos for each of that spell's colors.
// Remove five +1/+1 counters from Ramos: Add {W}{W}{U}{U}{B}{B}{R}{R}{G}{G}. Activate
// only once each turn.
//
// Note: "Whenever you cast a spell" trigger with SpellColorCount is deferred to PB-37.
// "Activate only once each turn" restriction is deferred to PB-37 (TimingRestriction::OncePerTurn).
use crate::cards::helpers::*;

pub fn card() -> CardDefinition {
    CardDefinition {
        card_id: cid("ramos-dragon-engine"),
        name: "Ramos, Dragon Engine".to_string(),
        mana_cost: Some(ManaCost { generic: 6, ..Default::default() }),
        types: full_types(
            &[SuperType::Legendary],
            &[CardType::Artifact, CardType::Creature],
            &["Dragon"],
        ),
        oracle_text: "Flying\nWhenever you cast a spell, put a +1/+1 counter on Ramos for each of that spell's colors.\nRemove five +1/+1 counters from Ramos: Add {W}{W}{U}{U}{B}{B}{R}{R}{G}{G}. Activate only once each turn.".to_string(),
        power: Some(4),
        toughness: Some(4),
        abilities: vec![
            AbilityDefinition::Keyword(KeywordAbility::Flying),
            // TODO (PB-37): "Whenever you cast a spell" trigger — needs WheneverYouCastASpell
            // trigger + EffectAmount::SpellColorCount to count a spell's colors.
            // CR 602.2: Remove five +1/+1 counters from Ramos: Add {W}{W}{U}{U}{B}{B}{R}{R}{G}{G}.
            // Note: Technically a mana ability (CR 605.1). Implemented as regular activated
            // ability for this batch. Mana-ability classification deferred to PB-37.
            // Once-per-turn restriction deferred to PB-37.
            AbilityDefinition::Activated {
                cost: Cost::RemoveCounter { counter: CounterType::PlusOnePlusOne, count: 5 },
                effect: Effect::AddMana {
                    player: PlayerTarget::Controller,
                    mana: ManaPool {
                        white: 2,
                        blue: 2,
                        black: 2,
                        red: 2,
                        green: 2,
                        ..Default::default()
                    },
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
