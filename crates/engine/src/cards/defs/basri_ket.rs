// Basri Ket — {1}{W}{W}, Legendary Planeswalker — Basri
// +1: Put a +1/+1 counter on up to one target creature. It gains indestructible until end of turn.
// −2: Whenever one or more nontoken creatures attack this turn, create that many 1/1 white
//     Soldier creature tokens that are tapped and attacking.
// −6: You get an emblem with "At the beginning of combat on your turn, create a 1/1 white
//     Soldier creature token, then put a +1/+1 counter on each creature you control."
use crate::cards::helpers::*;

pub fn card() -> CardDefinition {
    CardDefinition {
        card_id: cid("basri-ket"),
        name: "Basri Ket".to_string(),
        mana_cost: Some(ManaCost {
            generic: 1,
            white: 2,
            ..Default::default()
        }),
        types: full_types(
            &[SuperType::Legendary],
            &[CardType::Planeswalker],
            &["Basri"],
        ),
        oracle_text: "+1: Put a +1/+1 counter on up to one target creature. It gains indestructible until end of turn.\n\u{2212}2: Whenever one or more nontoken creatures attack this turn, create that many 1/1 white Soldier creature tokens that are tapped and attacking.\n\u{2212}6: You get an emblem with \"At the beginning of combat on your turn, create a 1/1 white Soldier creature token, then put a +1/+1 counter on each creature you control.\"".to_string(),
        abilities: vec![
            AbilityDefinition::LoyaltyAbility {
                cost: LoyaltyCost::Plus(1),
                // +1: Put a +1/+1 counter on up to one target creature. Gains indestructible until EOT.
                // NOTE: Indestructible grant until EOT is a continuous effect not yet fully
                // supported as a loyalty ability DSL effect. Counter placement is implemented.
                effect: Effect::AddCounter {
                    target: EffectTarget::DeclaredTarget { index: 0 },
                    counter: CounterType::PlusOnePlusOne,
                    count: 1,
                },
                targets: vec![TargetRequirement::TargetCreature],
            },
            AbilityDefinition::LoyaltyAbility {
                cost: LoyaltyCost::Minus(2),
                // −2: Delayed trigger "whenever nontoken creatures attack this turn".
                // TODO: Delayed triggered abilities (scoped to the current turn) are not yet
                // fully supported via DSL. This is a known gap.
                effect: Effect::Sequence(vec![]),
                targets: vec![],
            },
            AbilityDefinition::LoyaltyAbility {
                cost: LoyaltyCost::Minus(6),
                // −6: You get an emblem with "At the beginning of combat on your turn,
                // create a 1/1 white Soldier creature token, then put a +1/+1 counter
                // on each creature you control." (CR 114.1-114.4)
                // NOTE: Uses TriggerEvent::AtBeginningOfCombat. Emblem scanning for
                // step-based trigger events is wired in begin_combat() in turn_actions.rs.
                effect: Effect::CreateEmblem {
                    triggered_abilities: vec![
                        TriggeredAbilityDef {
                            trigger_on: TriggerEvent::AtBeginningOfCombat,
                            intervening_if: None,
                            description: "At the beginning of combat on your turn, create a 1/1 white Soldier creature token, then put a +1/+1 counter on each creature you control.".to_string(),
                            // TODO: Full effect (create token + distribute counters on all creatures)
                            // using AtBeginningOfCombat trigger.
                            effect: Some(Effect::CreateToken {
                                spec: TokenSpec {
                                    name: "Soldier".to_string(),
                                    power: 1,
                                    toughness: 1,
                                    colors: [Color::White].into_iter().collect(),
                                    card_types: [CardType::Creature].into_iter().collect(),
                                    subtypes: [SubType("Soldier".to_string())]
                                        .into_iter()
                                        .collect(),
                                    count: 1,
                                    ..Default::default()
                                },
                            }),
                            etb_filter: None,
                            death_filter: None,
                combat_damage_filter: None,
                            targets: vec![],
                        },
                    ],
                    static_effects: vec![],
                    play_from_graveyard: None,
                },
                targets: vec![],
            },
        ],
        starting_loyalty: Some(3),
        adventure_face: None,
        meld_pair: None,
        spell_additional_costs: vec![],
        activated_ability_cost_reductions: vec![],
        cant_be_countered: false,
        ..Default::default()
    }
}
