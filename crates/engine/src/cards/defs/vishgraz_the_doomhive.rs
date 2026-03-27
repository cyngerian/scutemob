// Vishgraz, the Doomhive — {2}{W}{B}{G}, Legendary Creature — Phyrexian Insect 3/3
// Menace, toxic 1. ETB: create three 1/1 colorless Phyrexian Mite artifact creature tokens
// with toxic 1 and "This token can't block."
// Gets +1/+1 for each poison counter opponents have (CDA — TODO).
use crate::cards::helpers::*;

pub fn card() -> CardDefinition {
    CardDefinition {
        card_id: cid("vishgraz-the-doomhive"),
        name: "Vishgraz, the Doomhive".to_string(),
        mana_cost: Some(ManaCost { white: 1, black: 1, green: 1, generic: 2, ..Default::default() }),
        types: full_types(&[SuperType::Legendary], &[CardType::Creature], &["Phyrexian", "Insect"]),
        oracle_text: "Menace, toxic 1 (Players dealt combat damage by this creature also get a poison counter.)\nWhen Vishgraz enters, create three 1/1 colorless Phyrexian Mite artifact creature tokens with toxic 1 and \"This token can't block.\"\nVishgraz gets +1/+1 for each poison counter your opponents have.".to_string(),
        power: Some(3),
        toughness: Some(3),
        abilities: vec![
            AbilityDefinition::Keyword(KeywordAbility::Menace),
            AbilityDefinition::Keyword(KeywordAbility::Toxic(1)),
            AbilityDefinition::Triggered {
                trigger_condition: TriggerCondition::WhenEntersBattlefield,
                effect: Effect::CreateToken {
                    spec: TokenSpec {
                        name: "Phyrexian Mite".to_string(),
                        card_types: [CardType::Artifact, CardType::Creature].into_iter().collect(),
                        subtypes: [SubType("Phyrexian".to_string()), SubType("Mite".to_string())].into_iter().collect(),
                        colors: im::OrdSet::new(),
                        supertypes: im::OrdSet::new(),
                        power: 1,
                        toughness: 1,
                        count: 3,
                        // Note: "This token can't block" is a static restriction.
                        // KeywordAbility::CantBlock does not exist in DSL — omitted.
                        keywords: [KeywordAbility::Toxic(1)].into_iter().collect(),
                        tapped: false,
                        enters_attacking: false,
                        mana_color: None,
                        mana_abilities: vec![],
                        activated_abilities: vec![],
                        ..Default::default()
                    },
                },
                intervening_if: None,
                targets: vec![],
            },
            // TODO: "Vishgraz gets +1/+1 for each poison counter your opponents have."
            // CDA based on opponents' poison counters — no EffectAmount::CountOpponentPoisonCounters
            // variant in DSL. Wrong game state if approximated.
        ],
        ..Default::default()
    }
}
