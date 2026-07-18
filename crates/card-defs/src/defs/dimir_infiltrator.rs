// Dimir Infiltrator — {U}{B}, Creature — Spirit 1/3
// This creature can't be blocked.
// Transmute {1}{U}{B} — search for card with same mana value, reveal, to hand.
// PB-AC5: Transmute implemented via KeywordAbility::Transmute (marker) + a normal
// Cost::Sequence([Mana, DiscardSelf]) activated ability (CR 702.53). Dimir Infiltrator's
// own mana value ({U}{B} = 2) is a fixed property, so the search filter hardcodes
// min_cmc = max_cmc = 2 (faithful for this card; a general "same MV as source, dynamic"
// filter is out of PB-AC5 scope — see plan Risks).
use crate::cards::helpers::*;

pub fn card() -> CardDefinition {
    CardDefinition {
        card_id: cid("dimir-infiltrator"),
        name: "Dimir Infiltrator".to_string(),
        mana_cost: Some(ManaCost {
            blue: 1,
            black: 1,
            ..Default::default()
        }),
        types: creature_types(&["Spirit"]),
        oracle_text: "This creature can't be blocked.\nTransmute {1}{U}{B} ({1}{U}{B}, Discard \
                      this card: Search your library for a card with the same mana value as this \
                      card, reveal it, put it into your hand, then shuffle. Transmute only as a \
                      sorcery.)"
            .to_string(),
        power: Some(1),
        toughness: Some(3),
        abilities: vec![
            AbilityDefinition::Keyword(KeywordAbility::CantBeBlocked),
            AbilityDefinition::Keyword(KeywordAbility::Transmute),
            AbilityDefinition::Activated {
                cost: Cost::Sequence(vec![
                    Cost::Mana(ManaCost {
                        generic: 1,
                        blue: 1,
                        black: 1,
                        ..Default::default()
                    }),
                    Cost::DiscardSelf,
                ]),
                effect: Effect::SearchLibrary {
                    player: PlayerTarget::Controller,
                    filter: TargetFilter {
                        min_cmc: Some(2),
                        max_cmc: Some(2),
                        ..Default::default()
                    },
                    reveal: true,
                    destination: ZoneTarget::Hand {
                        owner: PlayerTarget::Controller,
                    },
                    shuffle_before_placing: false,
                    also_search_graveyard: false,
                },
                timing_restriction: Some(TimingRestriction::SorcerySpeed),
                targets: vec![],
                activation_condition: None,
                activation_zone: None,
                once_per_turn: false,
                modes: None,
            },
        ],
        ..Default::default()
    }
}
