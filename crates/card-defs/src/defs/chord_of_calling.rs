// Chord of Calling — {X}{G}{G}{G}, Instant
// Convoke
// Search your library for a creature card with mana value X or less, put it
// onto the battlefield, then shuffle.
use crate::cards::helpers::*;

pub fn card() -> CardDefinition {
    CardDefinition {
        card_id: cid("chord-of-calling"),
        name: "Chord of Calling".to_string(),
        mana_cost: Some(ManaCost {
            green: 3,
            x_count: 1,
            ..Default::default()
        }),
        types: types(&[CardType::Instant]),
        oracle_text: "Convoke (Your creatures can help cast this spell. Each creature you tap \
                      while casting this spell pays for {1} or one mana of that creature's \
                      color.)\nSearch your library for a creature card with mana value X or less, \
                      put it onto the battlefield, then shuffle."
            .to_string(),
        abilities: vec![
            AbilityDefinition::Keyword(KeywordAbility::Convoke),
            AbilityDefinition::Spell {
                // CR 202.3/608.2h: "mana value X or less" is the runtime cap
                // `TargetFilter.max_cmc_amount`, resolved by the `Effect::SearchLibrary`
                // executor (`effects/mod.rs:3707-3710`) against `ctx.x_value`
                // (`effects/mod.rs:8264`). Precedents: eldritch_evolution, birthing_pod,
                // birthing_ritual. PB-DX27 (2026-08-13): the previous
                // `// TODO: max_cmc should be XValue` claimed a DSL gap that had already
                // closed — the field shipped with PB-EF10.
                effect: Effect::SearchLibrary {
                    filter: TargetFilter {
                        has_card_type: Some(CardType::Creature),
                        max_cmc_amount: Some(Box::new(EffectAmount::XValue)),
                        ..Default::default()
                    },
                    destination: ZoneTarget::Battlefield { tapped: false },
                    reveal: false,
                    player: PlayerTarget::Controller,
                    also_search_graveyard: false,
                    shuffle_before_placing: false,
                },
                targets: vec![],
                modes: None,
                cant_be_countered: false,
            },
        ],
        // PB-DX27: promoted from `partial`. Every printed clause is now authored —
        // Convoke (KeywordAbility::Convoke, M6) and the X-capped creature tutor.
        // Declared EXPLICITLY rather than left to the `#[default]` derive: OOS-RR3-1
        // records that derive as a repeatedly-demonstrated silent-defect generator,
        // so a promotion should be a ruling somebody made, not a field somebody deleted.
        completeness: Completeness::Complete,
        ..Default::default()
    }
}
