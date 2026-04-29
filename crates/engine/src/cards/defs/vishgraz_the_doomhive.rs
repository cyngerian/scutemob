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
            // TODO: "Vishgraz gets +1/+1 for each poison counter your opponents have."
            //
            // PB-CC-A status (Option B per PB-CC-C precedent): the EffectAmount primitive
            // (`EffectAmount::PlayerCounterCount { player: PlayerTarget::EachOpponent,
            // counter: CounterType::Poison }`) is shipped and produces correct values via
            // `resolve_amount` and `resolve_cda_amount` (sums across opponents per CR 122.1
            // and the Vishgraz 2023-02-04 ruling).
            //
            // The card-def is BLOCKED because there is no Layer-7c CDA-style continuous-
            // re-evaluation primitive yet. Routing this through
            // `AbilityDefinition::Static { continuous_effect: ContinuousEffectDef {
            //     modification: LayerModification::ModifyBothDynamic { amount:
            //         EffectAmount::PlayerCounterCount { ... }, negate: false }, .. } }`
            // would (a) be registered with `is_cda: false` by
            // `register_static_continuous_effects` (replacement.rs) and (b) reach
            // layer-application code as an unsubstituted Dynamic variant, triggering the
            // `debug_assert!` guard in `apply_modification` (rules/layers.rs:1164-1170)
            // and silently no-op'ing in release.
            //
            // Mirroring `exuberant_fuseling.rs`: this needs the deferred PB-CC-C-followup
            // primitive (Layer-7c dynamic-static modification with continuous re-evaluation
            // — CR 611.3a). `CdaPowerToughness` (Layer 7a; abomination_of_llanowar pattern)
            // is the closest existing analogue but applies to a different layer (it sets
            // base P/T rather than modifying it), so a mechanical port using
            // `Sum(Fixed(3), PlayerCounterCount)` would be wrong-game-state under
            // Layer-7b "becomes a 0/2" overrides — forbidden by W6 policy.
            //
            // See: memory/primitives/pb-review-CC-C.md (Option B precedent),
            //      memory/primitives/pb-retriage-CC.md PB-CC-A § (a) "What does NOT work
            //      today" + § (b) "Subtlety on Vishgraz" (sum-semantic note).
        ],
        ..Default::default()
    }
}
