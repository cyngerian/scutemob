// Tyvar Kell — {2}{G}{G}, Legendary Planeswalker — Tyvar
// Elves you control have "{T}: Add {B}."
// +1: Put a +1/+1 counter on up to one target Elf. Untap it. It gains deathtouch until end of turn.
// 0: Create a 1/1 green Elf Warrior creature token.
// −6: You get an emblem with "Whenever you cast an Elf spell, it gains haste until end of turn
//     and you draw two cards."
use crate::cards::helpers::*;

pub fn card() -> CardDefinition {
    CardDefinition {
        card_id: cid("tyvar-kell"),
        name: "Tyvar Kell".to_string(),
        mana_cost: Some(ManaCost {
            generic: 2,
            green: 2,
            ..Default::default()
        }),
        types: full_types(
            &[SuperType::Legendary],
            &[CardType::Planeswalker],
            &["Tyvar"],
        ),
        oracle_text: "Elves you control have \"{T}: Add {B}.\"\n+1: Put a +1/+1 counter on up to one target Elf. Untap it. It gains deathtouch until end of turn.\n0: Create a 1/1 green Elf Warrior creature token.\n\u{2212}6: You get an emblem with \"Whenever you cast an Elf spell, it gains haste until end of turn and you draw two cards.\"".to_string(),
        abilities: vec![
            // Static ability: "Elves you control have '{T}: Add {B}.'"
            // TODO: Granting mana abilities to other creatures is a complex DSL pattern
            // (requires a continuous effect that adds a mana ability to Elf creatures).
            // This is a known DSL gap.

            // +1: Put a +1/+1 counter on up to one target Elf. Untap it. Gains deathtouch until EOT.
            // TODO: Untap target and grant deathtouch until EOT are additional effects
            // beyond AddCounter. Partial implementation: counter placement only.
            AbilityDefinition::LoyaltyAbility {
                cost: LoyaltyCost::Plus(1),
                effect: Effect::AddCounter {
                    target: EffectTarget::DeclaredTarget { index: 0 },
                    counter: CounterType::PlusOnePlusOne,
                    count: 1,
                },
                targets: vec![TargetRequirement::TargetCreatureWithFilter(TargetFilter {
                    has_subtype: Some(SubType("Elf".to_string())),
                    ..Default::default()
                })],
            },
            // 0: Create a 1/1 green Elf Warrior creature token.
            AbilityDefinition::LoyaltyAbility {
                cost: LoyaltyCost::Zero,
                effect: Effect::CreateToken {
                    spec: TokenSpec {
                        name: "Elf Warrior".to_string(),
                        power: 1,
                        toughness: 1,
                        colors: [Color::Green].into_iter().collect(),
                        card_types: [CardType::Creature].into_iter().collect(),
                        subtypes: [
                            SubType("Elf".to_string()),
                            SubType("Warrior".to_string()),
                        ]
                        .into_iter()
                        .collect(),
                        count: 1,
                        ..Default::default()
                    },
                },
                targets: vec![],
            },
            // −6: You get an emblem with "Whenever you cast an Elf spell, it gains haste
            // until end of turn and you draw two cards." (CR 114.1-114.4)
            // NOTE: Elf-spell filtering requires spell-type/subtype checks not yet
            // supported in TriggeredAbilityDef. This uses AnySpellCast as a placeholder.
            // The full effect (grant haste to the cast spell + draw 2) is also simplified
            // to just drawing 2 cards (granting haste to a spell on the stack requires
            // stack object modification, a known gap).
            AbilityDefinition::LoyaltyAbility {
                cost: LoyaltyCost::Minus(6),
                effect: Effect::CreateEmblem {
                    triggered_abilities: vec![
                        TriggeredAbilityDef {
                            trigger_on: TriggerEvent::AnySpellCast,
                            intervening_if: None,
                            description: "Whenever you cast an Elf spell, it gains haste until end of turn and you draw two cards.".to_string(),
                            // TODO: Only trigger on Elf spells (requires spell-subtype filter).
                            // TODO: Grant haste until EOT to the cast spell.
                            // Partial implementation: draw 2 cards on any spell cast.
                            effect: Some(Effect::DrawCards {
                                player: PlayerTarget::Controller,
                                count: EffectAmount::Fixed(2),
                            }),
                            etb_filter: None,
                            death_filter: None,
                combat_damage_filter: None,
                            targets: vec![],
                        },
                    ],
                    static_effects: vec![],
                },
                targets: vec![],
            },
        ],
        starting_loyalty: Some(3),
        adventure_face: None,
        meld_pair: None,
        activated_ability_cost_reductions: vec![],
        ..Default::default()
    }
}
