// Multani, Yavimaya's Avatar — {4}{G}{G}, Legendary Creature — Elemental Avatar 0/0
// Reach, trample; gets +1/+1 for each land you control and each land in graveyard (static pump, not CDA)
// PB-AC3: the +1/+1-per-land pump is expressible via the already-shipped
// AbilityDefinition::CdaModifyPowerToughness{amount: Sum(PermanentCount{Land},
// CardCount{Graveyard, Land})} — see Abomination of Llanowar for the Sum(...) pattern.
// NOT a new PB-AC3 primitive and NOT the blocker for this card.
// TODO: "{1}{G}, Return two lands you control to their owner's hand: Return this card from
// your graveyard to your hand." — needs an ActivationZone::Graveyard activated ability plus
// a "return N lands you control" additional cost; no DSL equivalent exists for the
// return-lands cost. This is the remaining blocker; card stays blocked under W6
// no-partials policy rather than authoring the CDA pump in isolation.
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
        oracle_text: "Reach, trample\nMultani gets +1/+1 for each land you control and each land card in your graveyard.\n{1}{G}, Return two lands you control to their owner's hand: Return this card from your graveyard to your hand.".to_string(),
        power: Some(0),
        toughness: Some(0),
        abilities: vec![
            AbilityDefinition::Keyword(KeywordAbility::Reach),
            AbilityDefinition::Keyword(KeywordAbility::Trample),
            // TODO: Activated ability to return from graveyard requires ActivationZone::Graveyard
            // and a cost of returning lands ("return two lands you control to hand"), neither
            // of which is in the DSL. See file-header comment for full PB-AC3 disposition.
        ],
        ..Default::default()
    }
}
