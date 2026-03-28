// Junji, the Midnight Sky — {3}{B}{B}, Legendary Creature — Dragon Spirit 5/5
// Flying, menace
// When Junji dies, choose one —
// • Each opponent discards two cards and loses 2 life.
// • Put target non-Dragon creature card from a graveyard onto the battlefield
//   under your control. You lose 2 life.
//
// CR 700.2b / PB-35: Modal death trigger. Bot fallback: mode 0 (discard + lose life).
// Note: Mode 1 ("put target non-Dragon creature card from a graveyard onto the
// battlefield") requires TargetRequirement::TargetCardInGraveyard with non-Dragon
// filter — approximated with TargetCardInGraveyard (no Dragon exclusion filter).
use crate::cards::helpers::*;

pub fn card() -> CardDefinition {
    CardDefinition {
        card_id: cid("junji-the-midnight-sky"),
        name: "Junji, the Midnight Sky".to_string(),
        mana_cost: Some(ManaCost { generic: 3, black: 2, ..Default::default() }),
        types: full_types(
            &[SuperType::Legendary],
            &[CardType::Creature],
            &["Dragon", "Spirit"],
        ),
        oracle_text: "Flying, menace\nWhen Junji, the Midnight Sky dies, choose one —\n• Each opponent discards two cards and loses 2 life.\n• Put target non-Dragon creature card from a graveyard onto the battlefield under your control. You lose 2 life.".to_string(),
        power: Some(5),
        toughness: Some(5),
        abilities: vec![
            AbilityDefinition::Keyword(KeywordAbility::Flying),
            AbilityDefinition::Keyword(KeywordAbility::Menace),
            // CR 700.2b / PB-35: Modal death trigger.
            // Mode 0: Each opponent discards two cards and loses 2 life.
            // Mode 1: Reanimate target non-Dragon creature card from any graveyard;
            //         you lose 2 life.
            AbilityDefinition::Triggered {
                trigger_condition: TriggerCondition::WhenDies,
                effect: Effect::Nothing,
                intervening_if: None,
                targets: vec![
                    // Mode 1 target: any creature card in any graveyard.
                    TargetRequirement::TargetCardInGraveyard(TargetFilter {
                        has_card_type: Some(CardType::Creature),
                        ..Default::default()
                    }),
                ],
                modes: Some(ModeSelection {
                    min_modes: 1,
                    max_modes: 1,
                    modes: vec![
                        // Mode 0: Each opponent discards two cards and loses 2 life.
                        Effect::Sequence(vec![
                            Effect::ForEach {
                                over: ForEachTarget::EachOpponent,
                                effect: Box::new(Effect::DiscardCards {
                                    player: PlayerTarget::DeclaredTarget { index: 0 },
                                    count: EffectAmount::Fixed(2),
                                }),
                            },
                            Effect::ForEach {
                                over: ForEachTarget::EachOpponent,
                                effect: Box::new(Effect::LoseLife {
                                    player: PlayerTarget::DeclaredTarget { index: 0 },
                                    amount: EffectAmount::Fixed(2),
                                }),
                            },
                        ]),
                        // Mode 1: Put target non-Dragon creature card from a graveyard
                        // onto the battlefield under your control. You lose 2 life.
                        // Note: Dragon filter not expressible; any creature card is used.
                        Effect::Sequence(vec![
                            Effect::MoveZone {
                                target: EffectTarget::DeclaredTarget { index: 0 },
                                to: ZoneTarget::Battlefield { tapped: false },
                                controller_override: Some(PlayerTarget::Controller),
                            },
                            Effect::LoseLife {
                                player: PlayerTarget::Controller,
                                amount: EffectAmount::Fixed(2),
                            },
                        ]),
                    ],
                    allow_duplicate_modes: false,
                    mode_costs: None,
                }),
                trigger_zone: None,
            },
        ],
        ..Default::default()
    }
}
