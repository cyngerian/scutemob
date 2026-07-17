// Multani, Yavimaya's Avatar — {4}{G}{G}, Legendary Creature — Elemental Avatar 0/0
// Reach, trample; gets +1/+1 for each land you control and each land in graveyard.
// CR 613.4c: PB-AC3 CdaModifyPowerToughness{amount: Sum(PermanentCount{Land},
// CardCount{Graveyard, Land})} — see Abomination of Llanowar for the Sum(...) pattern and
// Wight of the Reliquary for the graveyard CardCount shape.
// TODO: "{1}{G}, Return two lands you control to their owner's hand: Return this card from
// your graveyard to your hand." — needs an ActivationZone::Graveyard activated ability plus
// a "return N lands you control" additional cost; no DSL equivalent exists for the
// return-lands cost. This is the sole remaining blocker (capability gap, not wrong state).
use crate::cards::helpers::*;

pub fn card() -> CardDefinition {
    CardDefinition {
        card_id: cid("multani-yavimayas-avatar"),
        name: "Multani, Yavimaya's Avatar".to_string(),
        mana_cost: Some(ManaCost {
            generic: 4,
            green: 2,
            ..Default::default()
        }),
        types: full_types(
            &[SuperType::Legendary],
            &[CardType::Creature],
            &["Elemental", "Avatar"],
        ),
        oracle_text: "Reach, trample\nMultani gets +1/+1 for each land you control and each land \
                      card in your graveyard.\n{1}{G}, Return two lands you control to their \
                      owner's hand: Return this card from your graveyard to your hand."
            .to_string(),
        power: Some(0),
        toughness: Some(0),
        abilities: vec![
            AbilityDefinition::Keyword(KeywordAbility::Reach),
            AbilityDefinition::Keyword(KeywordAbility::Trample),
            // CR 613.4c: CDA — gets +1/+1 for each land you control and each land card in
            // your graveyard.
            AbilityDefinition::CdaModifyPowerToughness {
                power: Some(EffectAmount::Sum(
                    Box::new(EffectAmount::PermanentCount {
                        filter: TargetFilter {
                            has_card_type: Some(CardType::Land),
                            ..Default::default()
                        },
                        controller: PlayerTarget::Controller,
                    }),
                    Box::new(EffectAmount::CardCount {
                        zone: ZoneTarget::Graveyard {
                            owner: PlayerTarget::Controller,
                        },
                        player: PlayerTarget::Controller,
                        filter: Some(TargetFilter {
                            has_card_type: Some(CardType::Land),
                            ..Default::default()
                        }),
                    }),
                )),
                toughness: Some(EffectAmount::Sum(
                    Box::new(EffectAmount::PermanentCount {
                        filter: TargetFilter {
                            has_card_type: Some(CardType::Land),
                            ..Default::default()
                        },
                        controller: PlayerTarget::Controller,
                    }),
                    Box::new(EffectAmount::CardCount {
                        zone: ZoneTarget::Graveyard {
                            owner: PlayerTarget::Controller,
                        },
                        player: PlayerTarget::Controller,
                        filter: Some(TargetFilter {
                            has_card_type: Some(CardType::Land),
                            ..Default::default()
                        }),
                    }),
                )),
            },
            // TODO: Activated ability to return from graveyard requires ActivationZone::Graveyard
            // and a cost of returning lands ("return two lands you control to hand"), neither
            // of which is in the DSL. See file-header comment for full PB-AC3 disposition.
        ],
        completeness: Completeness::partial(
            "Sole blocker: no Cost variant expresses 'Return two lands you control to their \
             owner's hand' as an activation cost. The graveyard-activation half is NOT a blocker \
             — AbilityDefinition::Activated already has an `activation_zone` field and \
             graveyard-zone abilities are wired via collect_graveyard_carddef_triggers \
             (replay_harness.rs:2732). Reach, trample, and the CDA are implemented.",
        ),
        ..Default::default()
    }
}
