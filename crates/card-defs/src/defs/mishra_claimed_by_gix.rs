// Mishra, Claimed by Gix
use crate::cards::helpers::*;

pub fn card() -> CardDefinition {
    CardDefinition {
        card_id: cid("mishra-claimed-by-gix"),
        name: "Mishra, Claimed by Gix".to_string(),
        mana_cost: Some(ManaCost {
            generic: 2,
            black: 1,
            red: 1,
            ..Default::default()
        }),
        types: full_types(
            &[SuperType::Legendary],
            &[CardType::Creature],
            &["Human", "Artificer", "Phyrexian"],
        ),
        oracle_text: "Whenever you attack, each opponent loses X life and you gain X life, where \
                      X is the number of attacking creatures. If Mishra, Claimed by Gix and a \
                      creature named Phyrexian Dragon Engine are attacking, and you both own and \
                      control them, exile them, then meld them into Mishra, Lost to Phyrexia. It \
                      enters tapped and attacking."
            .to_string(),
        abilities: vec![
            // Whenever you attack, each opponent loses X life and you gain X life,
            // where X is the number of attacking creatures.
            // CR 508.1/509: PB-AC3 AttackingCreatureCount (EachPlayer, unrestricted by
            // controller — mirrors throne_of_the_god_pharaoh.rs / keep_watch.rs pattern).
            AbilityDefinition::Triggered {
                once_per_turn: false,
                trigger_condition: TriggerCondition::WheneverYouAttack,
                effect: Effect::Sequence(vec![
                    Effect::LoseLife {
                        player: PlayerTarget::EachOpponent,
                        amount: EffectAmount::AttackingCreatureCount {
                            controller: PlayerTarget::EachPlayer,
                            filter: None,
                        },
                    },
                    Effect::GainLife {
                        player: PlayerTarget::Controller,
                        amount: EffectAmount::AttackingCreatureCount {
                            controller: PlayerTarget::EachPlayer,
                            filter: None,
                        },
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
        completeness: Completeness::partial(
            "Meld is NOT a blocker — Effect::Meld + CardDefinition::meld_pair shipped and are \
             used by hanweir_battlements.rs. The live blockers are (1) no Condition gating the \
             meld on 'Mishra and a creature named Phyrexian Dragon Engine are attacking' \
             (Effect::Meld only checks battlefield + same owner/controller, so an unguarded call \
             would meld outside combat — wrong game state), and (2) no expression for the melded \
             permanent entering tapped and attacking. The WheneverYouAttack drain/gain half is \
             implemented via PB-AC3 AttackingCreatureCount.",
        ),
        ..Default::default()
    }
}
