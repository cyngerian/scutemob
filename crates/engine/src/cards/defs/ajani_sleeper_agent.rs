// Ajani, Sleeper Agent — {1}{G}{G/W/P}{W}, Legendary Planeswalker — Ajani
// Compleated ({G/W/P} can be paid with {G}, {W}, or 2 life. If life was paid, this
// planeswalker enters with two fewer loyalty counters.)
// +1: Reveal the top card of your library. If it's a creature or planeswalker card,
//     put it into your hand. Otherwise, you may put it on the bottom of your library.
// −3: Distribute three +1/+1 counters among up to three target creatures. They gain
//     vigilance until end of turn.
// −6: You get an emblem with "Whenever you cast a creature or planeswalker spell,
//     target opponent gets two poison counters."
use crate::cards::helpers::*;

pub fn card() -> CardDefinition {
    CardDefinition {
        card_id: cid("ajani-sleeper-agent"),
        name: "Ajani, Sleeper Agent".to_string(),
        // Mana cost: {1}{G}{G/W/P}{W}
        mana_cost: Some(ManaCost {
            generic: 1,
            green: 1,
            white: 1,
            phyrexian: vec![PhyrexianMana::Hybrid(ManaColor::Green, ManaColor::White)],
            ..Default::default()
        }),
        types: full_types(
            &[SuperType::Legendary],
            &[CardType::Planeswalker],
            &["Ajani"],
        ),
        oracle_text: "Compleated ({G/W/P} can be paid with {G}, {W}, or 2 life. If life was paid, this planeswalker enters with two fewer loyalty counters.)\n+1: Reveal the top card of your library. If it's a creature or planeswalker card, put it into your hand. Otherwise, you may put it on the bottom of your library.\n\u{2212}3: Distribute three +1/+1 counters among up to three target creatures. They gain vigilance until end of turn.\n\u{2212}6: You get an emblem with \"Whenever you cast a creature or planeswalker spell, target opponent gets two poison counters.\"".to_string(),
        abilities: vec![
            // TODO: Compleated keyword — 2 fewer loyalty if Phyrexian life was paid.
            AbilityDefinition::LoyaltyAbility {
                cost: LoyaltyCost::Plus(1),
                // +1: Reveal top card, put in hand if creature/planeswalker, else bottom of library.
                // TODO: RevealTop + conditional hand/library placement.
                effect: Effect::Sequence(vec![]),
                targets: vec![],
            },
            AbilityDefinition::LoyaltyAbility {
                cost: LoyaltyCost::Minus(3),
                // −3: Distribute three +1/+1 counters among up to three targets + vigilance.
                // TODO: distributed counter placement + grant vigilance until EOT.
                effect: Effect::Sequence(vec![]),
                targets: vec![],
            },
            AbilityDefinition::LoyaltyAbility {
                cost: LoyaltyCost::Minus(6),
                // −6: You get an emblem with "Whenever you cast a creature or planeswalker
                // spell, target opponent gets two poison counters." (CR 114.1-114.4)
                // NOTE: The emblem trigger fires on AnySpellCast for the controller.
                // TODO: Spell-type filter (creature/planeswalker only) is not enforced —
                // TriggeredAbilityDef lacks a spell_type_filter field. The emblem fires
                // on any spell cast by the controller (instants, sorceries, etc. also
                // trigger it). Known DSL gap — MEDIUM severity per PB-22 S6 review.
                //
                // TODO: Target should be TargetRequirement::TargetOpponent (i.e., only
                // opponents may be chosen). TargetRequirement has no Opponent variant;
                // TargetPlayer allows targeting oneself. Known DSL gap — MEDIUM severity.
                effect: Effect::CreateEmblem {
                    triggered_abilities: vec![
                        TriggeredAbilityDef {
                            trigger_on: TriggerEvent::AnySpellCast,
                            intervening_if: None,
                            description: "Whenever you cast a creature or planeswalker spell, target opponent gets two poison counters.".to_string(),
                            effect: Some(Effect::AddCounter {
                                target: EffectTarget::DeclaredTarget { index: 0 },
                                counter: CounterType::Poison,
                                count: 2,
                            }),
                            etb_filter: None,
                            death_filter: None,
                combat_damage_filter: None,
                            // TODO: Should be TargetOpponent; TargetPlayer is an approximation.
                            targets: vec![TargetRequirement::TargetPlayer],
                        },
                    ],
                    static_effects: vec![],
                },
                targets: vec![],
            },
        ],
        starting_loyalty: Some(4),
        adventure_face: None,
        meld_pair: None,
        spell_additional_costs: vec![],
        activated_ability_cost_reductions: vec![],
        ..Default::default()
    }
}
