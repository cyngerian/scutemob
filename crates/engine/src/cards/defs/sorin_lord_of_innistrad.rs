// Sorin, Lord of Innistrad — {2}{W}{B}, Legendary Planeswalker — Sorin
// +1: Create a 1/1 black Vampire creature token with lifelink.
// −2: You get an emblem with "Creatures you control get +1/+0."
// −6: Destroy up to three target creatures and/or other planeswalkers. Return each card
//     put into a graveyard this way to the battlefield under your control.
use crate::cards::helpers::*;

pub fn card() -> CardDefinition {
    CardDefinition {
        card_id: cid("sorin-lord-of-innistrad"),
        name: "Sorin, Lord of Innistrad".to_string(),
        mana_cost: Some(ManaCost { generic: 2, white: 1, black: 1, ..Default::default() }),
        types: full_types(
            &[SuperType::Legendary],
            &[CardType::Planeswalker],
            &["Sorin"],
        ),
        oracle_text: "+1: Create a 1/1 black Vampire creature token with lifelink.\n\u{2212}2: You get an emblem with \"Creatures you control get +1/+0.\"\n\u{2212}6: Destroy up to three target creatures and/or other planeswalkers. Return each card put into a graveyard this way to the battlefield under your control.".to_string(),
        starting_loyalty: Some(3),
        abilities: vec![
            // +1: Create a 1/1 black Vampire token with lifelink.
            AbilityDefinition::LoyaltyAbility {
                cost: LoyaltyCost::Plus(1),
                effect: Effect::CreateToken {
                    spec: TokenSpec {
                        name: "Vampire".to_string(),
                        card_types: [CardType::Creature].into_iter().collect(),
                        subtypes: [SubType("Vampire".to_string())].into_iter().collect(),
                        colors: [Color::Black].into_iter().collect(),
                        supertypes: im::OrdSet::new(),
                        power: 1,
                        toughness: 1,
                        count: EffectAmount::Fixed(1),
                        keywords: [KeywordAbility::Lifelink].into_iter().collect(),
                        tapped: false,
                        enters_attacking: false,
                        mana_color: None,
                        mana_abilities: vec![],
                        activated_abilities: vec![],
                        ..Default::default()
                    },
                },
                targets: vec![],
            },
            // −2: You get an emblem with "Creatures you control get +1/+0."
            // TODO: Emblem with static P/T modification (all creatures +1/+0) — the emblem
            // creates a static continuous effect. EmblemSpec static_effects DSL not yet
            // wired to produce a working lord-style effect from an emblem.
            AbilityDefinition::LoyaltyAbility {
                cost: LoyaltyCost::Minus(2),
                effect: Effect::Nothing,
                targets: vec![],
            },
            // −6: Destroy up to three target creatures and/or other planeswalkers. Return each
            //     card put into a graveyard this way to the battlefield under your control.
            //     (CR 601.2c / CR 701.8 / CR 400.7)
            // DeclaredTarget{index} resolves exactly one target slot, so we list all three
            // indices explicitly to cover the full up-to-3 declared set (PB-LS6).
            AbilityDefinition::LoyaltyAbility {
                cost: LoyaltyCost::Minus(6),
                effect: Effect::DestroyAndReanimate {
                    targets: vec![
                        EffectTarget::DeclaredTarget { index: 0 },
                        EffectTarget::DeclaredTarget { index: 1 },
                        EffectTarget::DeclaredTarget { index: 2 },
                    ],
                    cant_be_regenerated: false,
                },
                targets: vec![TargetRequirement::UpToN {
                    count: 3,
                    inner: Box::new(TargetRequirement::TargetPermanentWithFilter(TargetFilter {
                        has_card_types: vec![CardType::Creature, CardType::Planeswalker],
                        ..Default::default()
                    })),
                }],
            },
        ],
        ..Default::default()
    }
}
