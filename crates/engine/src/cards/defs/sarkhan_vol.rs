// Sarkhan Vol — {2}{R}{G}, Legendary Planeswalker — Sarkhan
// +1: Creatures you control get +1/+1 and gain haste until end of turn.
// −2: Gain control of target creature until end of turn. Untap that creature. It gains
//     haste until end of turn.
// −6: Create five 4/4 red Dragon creature tokens with flying.
use crate::cards::helpers::*;

pub fn card() -> CardDefinition {
    CardDefinition {
        card_id: cid("sarkhan-vol"),
        name: "Sarkhan Vol".to_string(),
        mana_cost: Some(ManaCost { generic: 2, red: 1, green: 1, ..Default::default() }),
        types: full_types(
            &[SuperType::Legendary],
            &[CardType::Planeswalker],
            &["Sarkhan"],
        ),
        oracle_text: "+1: Creatures you control get +1/+1 and gain haste until end of turn.\n\u{2212}2: Gain control of target creature until end of turn. Untap that creature. It gains haste until end of turn.\n\u{2212}6: Create five 4/4 red Dragon creature tokens with flying.".to_string(),
        starting_loyalty: Some(4),
        abilities: vec![
            // +1: Pump + haste
            // TODO: ApplyContinuousEffect to all creatures you control not wired to DSL.
            AbilityDefinition::LoyaltyAbility {
                cost: LoyaltyCost::Plus(1),
                effect: Effect::Nothing,
                targets: vec![],
            },
            // −2: Threaten
            // TODO: Gain control + untap + haste until EOT not expressible.
            AbilityDefinition::LoyaltyAbility {
                cost: LoyaltyCost::Minus(2),
                effect: Effect::Nothing,
                targets: vec![],
            },
            // −6: Create 5 Dragon tokens
            AbilityDefinition::LoyaltyAbility {
                cost: LoyaltyCost::Minus(6),
                effect: Effect::CreateToken {
                    spec: TokenSpec {
                        name: "Dragon".to_string(),
                        card_types: [CardType::Creature].into_iter().collect(),
                        subtypes: [SubType("Dragon".to_string())].into_iter().collect(),
                        colors: [Color::Red].into_iter().collect(),
                        power: 4,
                        toughness: 4,
                        count: 5,
                        supertypes: im::OrdSet::new(),
                        keywords: [KeywordAbility::Flying].into_iter().collect(),
                        tapped: false,
                        enters_attacking: false,
                        mana_color: None,
                        mana_abilities: vec![],
                        activated_abilities: vec![],
                    },
                },
                targets: vec![],
            },
        ],
        ..Default::default()
    }
}
