// Korvold, Fae-Cursed King — {2}{B}{R}{G}, Legendary Creature — Dragon Noble 4/4
// Flying
// Whenever Korvold enters or attacks, sacrifice another permanent.
// Whenever you sacrifice a permanent, put a +1/+1 counter on Korvold and draw a card.
//
// PB-EF1 (scutemob-99): the enters/attacks "sacrifice another permanent" is now
// expressible — Effect::SacrificePermanents honors TargetFilter.exclude_self (CR 109.1).
use crate::cards::helpers::*;

pub fn card() -> CardDefinition {
    CardDefinition {
        card_id: cid("korvold-fae-cursed-king"),
        name: "Korvold, Fae-Cursed King".to_string(),
        mana_cost: Some(ManaCost {
            generic: 2,
            black: 1,
            red: 1,
            green: 1,
            ..Default::default()
        }),
        types: full_types(
            &[SuperType::Legendary],
            &[CardType::Creature],
            &["Dragon", "Noble"],
        ),
        oracle_text: "Flying\nWhenever Korvold, Fae-Cursed King enters or attacks, sacrifice \
                      another permanent.\nWhenever you sacrifice a permanent, put a +1/+1 counter \
                      on Korvold and draw a card."
            .to_string(),
        power: Some(4),
        toughness: Some(4),
        abilities: vec![
            AbilityDefinition::Keyword(KeywordAbility::Flying),
            // "Whenever Korvold enters or attacks, sacrifice another permanent."
            // Split into two separate triggers (there is no combined enters-or-attacks
            // TriggerCondition); each is an exact translation of one half of the clause.
            // PB-EF1 (CR 109.1): "another permanent" — Effect::SacrificePermanents now honors
            // TargetFilter.exclude_self (source ObjectId threaded into eligible_sacrifice_targets),
            // so Korvold cannot sacrifice itself. Forced (not "may").
            AbilityDefinition::Triggered {
                once_per_turn: false,
                trigger_condition: TriggerCondition::WhenEntersBattlefield,
                effect: Effect::SacrificePermanents {
                    player: PlayerTarget::Controller,
                    count: EffectAmount::Fixed(1),
                    filter: Some(TargetFilter {
                        controller: TargetController::You,
                        exclude_self: true,
                        ..Default::default()
                    }),
                },
                intervening_if: None,
                targets: vec![],
                modes: None,
                trigger_zone: None,
            },
            AbilityDefinition::Triggered {
                once_per_turn: false,
                // CR 508.1: "Whenever Korvold attacks".
                trigger_condition: TriggerCondition::WhenAttacks,
                effect: Effect::SacrificePermanents {
                    player: PlayerTarget::Controller,
                    count: EffectAmount::Fixed(1),
                    filter: Some(TargetFilter {
                        controller: TargetController::You,
                        exclude_self: true,
                        ..Default::default()
                    }),
                },
                intervening_if: None,
                targets: vec![],
                modes: None,
                trigger_zone: None,
            },
            // Whenever you sacrifice a permanent, put +1/+1 counter on Korvold and draw a card.
            AbilityDefinition::Triggered {
                once_per_turn: false,
                trigger_condition: TriggerCondition::WheneverYouSacrifice {
                    filter: None,
                    player_filter: None,
                },
                effect: Effect::Sequence(vec![
                    Effect::AddCounter {
                        target: EffectTarget::Source,
                        counter: CounterType::PlusOnePlusOne,
                        count: 1,
                    },
                    Effect::DrawCards {
                        player: PlayerTarget::Controller,
                        count: EffectAmount::Fixed(1),
                    },
                ]),
                intervening_if: None,
                targets: vec![],

                modes: None,
                trigger_zone: None,
            },
        ],
        // PB-EF1 (scutemob-99): Effect::SacrificePermanents now honors
        // TargetFilter.exclude_self (source threaded into eligible_sacrifice_targets, CR
        // 109.1), so the "another permanent" restriction is enforced. Both the enters/attacks
        // forced sacrifice and the "whenever you sacrifice" reward are implemented. Complete.
        ..Default::default()
    }
}
