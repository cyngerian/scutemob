// Tyvar Kell — {2}{G}{G}, Legendary Planeswalker — Tyvar
// Elves you control have "{T}: Add {B}."
// +1: Put a +1/+1 counter on up to one target Elf. Untap it. It gains deathtouch until
//     end of turn.
// 0: Create a 1/1 green Elf Warrior creature token.
// −6: You get an emblem with "Whenever you cast an Elf spell, it gains haste until end
//     of turn and you draw two cards."
//
// ENGINE-BLOCKED (static ability): "Elves you control have '{T}: Add {B}'" requires
// granting a mana ability via a continuous layer effect. No LayerModification::AddManaAbility
// exists in the DSL.
//
// ENGINE-BLOCKED (emblem trigger): The emblem's "Whenever you cast an Elf spell" requires
// a spell-subtype filter on WheneverYouCastSpell. Only CardType filters exist; Elf is a
// subtype. Also: "it gains haste" means the spell on the stack gains haste, which requires
// stack object modification. Both clauses are ENGINE-BLOCKED.
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
            // ENGINE-BLOCKED: "Elves you control have '{T}: Add {B}.'" — granting mana
            // abilities via a continuous effect is not in DSL.

            // +1: Put a +1/+1 counter on up to one target Elf. Untap it. It gains
            // deathtouch until end of turn. (CR 601.2c / 115.1b)
            AbilityDefinition::LoyaltyAbility {
                cost: LoyaltyCost::Plus(1),
                effect: Effect::Sequence(vec![
                    Effect::AddCounter {
                        target: EffectTarget::DeclaredTarget { index: 0 },
                        counter: CounterType::PlusOnePlusOne,
                        count: 1,
                    },
                    Effect::UntapPermanent { target: EffectTarget::DeclaredTarget { index: 0 } },
                    Effect::ApplyContinuousEffect {
                        effect_def: Box::new(ContinuousEffectDef {
                            layer: EffectLayer::Ability,
                            modification: LayerModification::AddKeyword(KeywordAbility::Deathtouch),
                            filter: EffectFilter::DeclaredTarget { index: 0 },
                            duration: EffectDuration::UntilEndOfTurn,
                            condition: None,
                        }),
                    },
                ]),
                targets: vec![TargetRequirement::UpToN {
                    count: 1,
                    inner: Box::new(TargetRequirement::TargetCreatureWithFilter(TargetFilter {
                        has_subtype: Some(SubType("Elf".to_string())),
                        ..Default::default()
                    })),
                }],
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
                        count: EffectAmount::Fixed(1),
                        ..Default::default()
                    },
                },
                targets: vec![],
            },
            // −6: You get an emblem with "Whenever you cast an Elf spell, it gains haste
            // until end of turn and you draw two cards."
            // ENGINE-BLOCKED: Elf is a spell subtype; WheneverYouCastSpell has no subtype
            // filter. Granting haste to a spell on the stack is also not expressible.
            // Effect::Nothing preserves the loyalty ability structure without wrong behavior.
            AbilityDefinition::LoyaltyAbility {
                cost: LoyaltyCost::Minus(6),
                effect: Effect::Nothing,
                targets: vec![],
            },
        ],
        starting_loyalty: Some(3),
        ..Default::default()
    }
}
