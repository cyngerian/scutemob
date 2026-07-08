// Mishra, Claimed by Gix
use crate::cards::helpers::*;

pub fn card() -> CardDefinition {
    CardDefinition {
        card_id: cid("mishra-claimed-by-gix"),
        name: "Mishra, Claimed by Gix".to_string(),
        mana_cost: Some(ManaCost { generic: 2, black: 1, red: 1, ..Default::default() }),
        types: full_types(&[SuperType::Legendary], &[CardType::Creature], &["Human", "Artificer", "Phyrexian"]),
        oracle_text: "Whenever you attack, each opponent loses X life and you gain X life, where X is the number of attacking creatures. If Mishra, Claimed by Gix and a creature named Phyrexian Dragon Engine are attacking, and you both own and control them, exile them, then meld them into Mishra, Lost to Phyrexia. It enters tapped and attacking.".to_string(),
        abilities: vec![
            // Whenever you attack, each opponent loses 1 life and you gain 1 life.
            // PB-AC3: "X = number of attacking creatures" is now expressible via
            // EffectAmount::AttackingCreatureCount, but this card stays PARTIAL/blocked
            // under W6 no-partials policy — the Meld clause below is unimplemented, and
            // the card's TODO list is left in place (rather than wiring the correct X here
            // in isolation) so authoring-report continues to flag this file for the
            // remaining Meld gap. Fixed(1) below is a KNOWN-WRONG placeholder pending Meld.
            AbilityDefinition::Triggered {
                once_per_turn: false,
                trigger_condition: TriggerCondition::WheneverYouAttack,
                effect: Effect::Sequence(vec![
                    Effect::ForEach {
                        over: ForEachTarget::EachOpponent,
                        effect: Box::new(Effect::LoseLife {
                            player: PlayerTarget::DeclaredTarget { index: 0 },
                            amount: EffectAmount::Fixed(1),
                        }),
                    },
                    Effect::GainLife {
                        player: PlayerTarget::Controller,
                        amount: EffectAmount::Fixed(1),
                    },
                ]),
                intervening_if: None,
                targets: vec![],

                modes: None,
                trigger_zone: None,
            },
            // TODO: Meld trigger (Phyrexian Dragon Engine + Mishra meld) — Meld not yet in
            // DSL. This is the sole remaining blocker for this card after PB-AC3 (which
            // otherwise unblocks the AttackingCreatureCount X above).
        ],
        power: Some(3),
        toughness: Some(5),
        ..Default::default()
    }
}
