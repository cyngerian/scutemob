// Freyalise, Llanowar's Fury — {3}{G}{G}, Legendary Planeswalker — Freyalise
// +2: Create a 1/1 green Elf Druid creature token with "{T}: Add {G}."
// −2: Destroy target artifact or enchantment.
// −6: Draw a card for each green creature you control.
// Freyalise can be your commander.
use crate::cards::helpers::*;

pub fn card() -> CardDefinition {
    CardDefinition {
        card_id: cid("freyalise-llanowars-fury"),
        name: "Freyalise, Llanowar's Fury".to_string(),
        mana_cost: Some(ManaCost { generic: 3, green: 2, ..Default::default() }),
        types: full_types(&[SuperType::Legendary], &[CardType::Planeswalker], &["Freyalise"]),
        oracle_text: "+2: Create a 1/1 green Elf Druid creature token with \"{T}: Add {G}.\"\n\u{2212}2: Destroy target artifact or enchantment.\n\u{2212}6: Draw a card for each green creature you control.\nFreyalise, Llanowar's Fury can be your commander.".to_string(),
        starting_loyalty: Some(3),
        abilities: vec![
            // +2: Create a 1/1 green Elf Druid token with "{T}: Add {G}."
            AbilityDefinition::LoyaltyAbility {
                cost: LoyaltyCost::Plus(2),
                effect: Effect::CreateToken {
                    spec: TokenSpec {
                        name: "Elf Druid".to_string(),
                        card_types: [CardType::Creature].into_iter().collect(),
                        subtypes: [SubType("Elf".to_string()), SubType("Druid".to_string())].into_iter().collect(),
                        colors: [Color::Green].into_iter().collect(),
                        power: 1,
                        toughness: 1,
                        supertypes: im::OrdSet::new(),
                        keywords: im::OrdSet::new(),
                        count: 1,
                        tapped: false,
                        enters_attacking: false,
                        mana_color: None,
                        mana_abilities: vec![ManaAbility::tap_for(ManaColor::Green)],
                        activated_abilities: vec![],
                        ..Default::default()
                    },
                },
                targets: vec![],
            },
            // −2: Destroy target artifact or enchantment.
            AbilityDefinition::LoyaltyAbility {
                cost: LoyaltyCost::Minus(2),
                effect: Effect::DestroyPermanent {
                    target: EffectTarget::DeclaredTarget { index: 0 },
                    cant_be_regenerated: false,
                },
                targets: vec![TargetRequirement::TargetPermanentWithFilter(TargetFilter {
                    has_card_types: vec![CardType::Artifact, CardType::Enchantment],
                    ..Default::default()
                })],
            },
            // −6: Draw a card for each green creature you control.
            AbilityDefinition::LoyaltyAbility {
                cost: LoyaltyCost::Minus(6),
                effect: Effect::DrawCards {
                    player: PlayerTarget::Controller,
                    count: EffectAmount::PermanentCount {
                        filter: TargetFilter {
                            has_card_type: Some(CardType::Creature),
                            colors: Some(im::ordset![Color::Green]),
                            ..Default::default()
                        },
                        controller: PlayerTarget::Controller,
                    },
                },
                targets: vec![],
            },
        ],
        ..Default::default()
    }
}
