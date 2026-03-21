// Kaito, Bane of Nightmares — {2}{U}{B}, Legendary Planeswalker — Kaito
// Ninjutsu {1}{U}{B}
// During your turn, as long as Kaito has one or more loyalty counters on him, he's a
// 3/4 Ninja creature and has hexproof.
// +1: You get an emblem with "Ninjas you control get +1/+1."
// 0: Surveil 2. Then draw a card for each opponent who lost life this turn.
// −2: Tap target creature. Put two stun counters on it.
use crate::cards::helpers::*;

pub fn card() -> CardDefinition {
    CardDefinition {
        card_id: cid("kaito-bane-of-nightmares"),
        name: "Kaito, Bane of Nightmares".to_string(),
        mana_cost: Some(ManaCost {
            generic: 2,
            blue: 1,
            black: 1,
            ..Default::default()
        }),
        types: full_types(
            &[SuperType::Legendary],
            &[CardType::Planeswalker],
            &["Kaito"],
        ),
        oracle_text: "Ninjutsu {1}{U}{B}\nDuring your turn, as long as Kaito has one or more loyalty counters on him, he's a 3/4 Ninja creature and has hexproof.\n+1: You get an emblem with \"Ninjas you control get +1/+1.\"\n0: Surveil 2. Then draw a card for each opponent who lost life this turn.\n\u{2212}2: Tap target creature. Put two stun counters on it.".to_string(),
        abilities: vec![
            // Ninjutsu keyword marker
            AbilityDefinition::Keyword(KeywordAbility::Ninjutsu),
            // Ninjutsu cost ability
            // TODO: Ninjutsu cost activated ability — ninjutsu is handled by the keyword
            // engine and doesn't need a separate activated ability definition here.
            // The Keyword(Ninjutsu) above registers the keyword marker.
            // +1: You get an emblem with "Ninjas you control get +1/+1."
            // The emblem applies a static +1/+1 continuous effect to Ninja creatures (CR 114.4).
            // Each additional activation creates a new emblem that stacks independently (CR 113.2c).
            AbilityDefinition::LoyaltyAbility {
                cost: LoyaltyCost::Plus(1),
                effect: Effect::CreateEmblem {
                    triggered_abilities: vec![],
                    static_effects: vec![
                        ContinuousEffectDef {
                            layer: EffectLayer::PtModify,
                            // Layer 7c: +1/+1 to Ninjas you control.
                            // ModifyBoth applies the same value to both P and T.
                            modification: LayerModification::ModifyBoth(1),
                            // OtherCreaturesYouControlWithSubtype("Ninja") resolves correctly
                            // for emblems: the emblem (source) is in the command zone, not a
                            // Ninja creature, so no battlefield creature is excluded. All Ninja
                            // creatures you control receive the bonus (CR 114.4).
                            filter: EffectFilter::OtherCreaturesYouControlWithSubtype(
                                SubType("Ninja".to_string()),
                            ),
                            // Indefinite: emblems never leave the command zone (CR 114.1).
                            duration: EffectDuration::Indefinite,
                        },
                    ],
                },
                targets: vec![],
            },
            // 0: Surveil 2. Then draw a card for each opponent who lost life this turn.
            // TODO: "draw a card for each opponent who lost life this turn" requires tracking
            // per-player life loss history — a known DSL gap. Surveil 2 is implemented.
            AbilityDefinition::LoyaltyAbility {
                cost: LoyaltyCost::Zero,
                effect: Effect::Surveil {
                    player: PlayerTarget::Controller,
                    count: EffectAmount::Fixed(2),
                },
                targets: vec![],
            },
            // −2: Tap target creature. Put two stun counters on it.
            AbilityDefinition::LoyaltyAbility {
                cost: LoyaltyCost::Minus(2),
                effect: Effect::Sequence(vec![
                    Effect::TapPermanent {
                        target: EffectTarget::DeclaredTarget { index: 0 },
                    },
                    Effect::AddCounter {
                        target: EffectTarget::DeclaredTarget { index: 0 },
                        counter: CounterType::Stun,
                        count: 2,
                    },
                ]),
                targets: vec![TargetRequirement::TargetCreature],
            },
        ],
        starting_loyalty: Some(4),
        meld_pair: None,
        ..Default::default()
    }
}
