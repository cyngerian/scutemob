// Vishgraz, the Doomhive — {2}{W}{B}{G}, Legendary Creature — Phyrexian Insect 3/3
// Menace, toxic 1. ETB: create three 1/1 colorless Phyrexian Mite artifact creature tokens
// with toxic 1 and "This token can't block."
// Gets +1/+1 for each poison counter opponents have (CDA — deferred, see TODO below).
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
                        // CR 509.1b: "This token can't block."
                        keywords: [KeywordAbility::Toxic(1), KeywordAbility::CantBlock]
                            .into_iter()
                            .collect(),
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

                modes: None,
                trigger_zone: None,
            },
            // "Vishgraz gets +1/+1 for each poison counter your opponents have."
            // CR 611.3a: static ability, not locked-in — continuously re-evaluates.
            // CR 613.4c: Layer 7c modify (not a set — must not use CdaPowerToughness).
            // CR 122.1 + Vishgraz 2023-02-04 ruling: EachOpponent sums poison counters
            // across all opponents (3 opponents with 1/2/5 = 8, NOT count-of-poisoned=3).
            // PB-CC-C-followup ships AbilityDefinition::CdaModifyPowerToughness to
            // register a ContinuousEffect with ModifyBothDynamic + is_cda=true at Layer 7c.
            AbilityDefinition::CdaModifyPowerToughness {
                power: Some(EffectAmount::PlayerCounterCount {
                    player: PlayerTarget::EachOpponent,
                    counter: CounterType::Poison,
                }),
                toughness: Some(EffectAmount::PlayerCounterCount {
                    player: PlayerTarget::EachOpponent,
                    counter: CounterType::Poison,
                }),
            },
        ],
        ..Default::default()
    }
}
