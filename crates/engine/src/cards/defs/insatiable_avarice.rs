// Insatiable Avarice — {B}, Sorcery (Spree)
// + {2} — Search your library for a card, then shuffle and put that card on top.
// + {B}{B} — Target player draws three cards and loses 3 life.
use crate::cards::helpers::*;

pub fn card() -> CardDefinition {
    CardDefinition {
        card_id: cid("insatiable-avarice"),
        name: "Insatiable Avarice".to_string(),
        mana_cost: Some(ManaCost { black: 1, ..Default::default() }),
        types: types(&[CardType::Sorcery]),
        oracle_text:
            "Spree (Choose one or more additional costs.)\n\
             + {2} — Search your library for a card, then shuffle and put that card on top.\n\
             + {B}{B} — Target player draws three cards and loses 3 life."
                .to_string(),
        abilities: vec![
            AbilityDefinition::Keyword(KeywordAbility::Spree),
            AbilityDefinition::Spell {
                // CR 702.172a: Spree — at least one mode must be chosen; each chosen
                // mode's additional cost is paid on top of the card's base mana cost.
                effect: Effect::Sequence(vec![]),
                targets: vec![],
                modes: Some(ModeSelection {
                    // CR 702.172a: Spree requires at least 1 mode; no upper limit beyond mode count.
                    min_modes: 1,
                    max_modes: 2,
                    allow_duplicate_modes: false,
                    // CR 700.2h / 702.172a: per-mode additional costs.
                    // Mode 0: +{2}; Mode 1: +{B}{B}
                    mode_costs: Some(vec![
                        ManaCost { generic: 2, ..Default::default() },
                        ManaCost { black: 2, ..Default::default() },
                    ]),
                    modes: vec![
                        // Mode 0 (+{2}): Search your library for a card, then shuffle and
                        // put that card on top. (CR 701.23 — "shuffle and put on top" means
                        // shuffle first, then place the chosen card on top.)
                        Effect::SearchLibrary {
                            player: PlayerTarget::Controller,
                            filter: TargetFilter::default(),
                            reveal: false,
                            destination: ZoneTarget::Library {
                                owner: PlayerTarget::Controller,
                                position: LibraryPosition::Top,
                            },
                            shuffle_before_placing: true,
                            also_search_graveyard: false,
                        },

                        // Mode 1 (+{B}{B}): Target player draws three cards and loses 3 life.
                        // TODO: the DSL has no way to attach a per-mode TargetRequirement to
                        // an individual Spree mode. A spell-level target would wrongly require
                        // a target even when only mode 0 is chosen. This mode is a no-op
                        // placeholder, mirroring the same DSL gap documented in final_showdown.rs.
                        Effect::Sequence(vec![]),
                    ],
                    mode_targets: None,
                }),
                cant_be_countered: false,
            },
        ],
        completeness: Completeness::partial("the DSL has no way to attach a per-mode TargetRequirement to an individual Spree mode. A spell-level target would..."),
        ..Default::default()
    }
}
