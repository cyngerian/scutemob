// Ramos, Dragon Engine — {6}, Legendary Artifact Creature — Dragon 4/4
// Flying
// Whenever you cast a spell, put a +1/+1 counter on Ramos for each of that spell's colors.
// Remove five +1/+1 counters from Ramos: Add {W}{W}{U}{U}{B}{B}{R}{R}{G}{G}. Activate
// only once each turn.
//
// CR 602.5b: "Activate only once each turn" restriction implemented via once_per_turn: true.
// TODO: "Whenever you cast a spell" trigger — needs WheneverYouCastASpell trigger
//        + EffectAmount::SpellColorCount to count a spell's colors. Deferred.
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
            // CR 602.2: Remove five +1/+1 counters from Ramos: Add {W}{W}{U}{U}{B}{B}{R}{R}{G}{G}.
            // CR 602.5b: "Activate only once each turn" restriction.
            // Note: Technically a mana ability (CR 605.1). Implemented as regular activated
            // ability. Mana-ability classification deferred to BF-1.
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
                once_per_turn: true,
            },
        ],
        ..Default::default()
    }
}
