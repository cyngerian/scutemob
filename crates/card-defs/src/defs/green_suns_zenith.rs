// Green Sun's Zenith — {X}{G}, Sorcery
// Search your library for a green creature card with mana value X or less, put
// it onto the battlefield, then shuffle. Shuffle Green Sun's Zenith into its
// owner's library.
//
// PB-DX27 (2026-08-13): the trailing "instead of putting it anywhere else" was a
// PHANTOM CLAUSE — it is not on the printed card (MCP-verified). It has been
// dropped from both this comment and `oracle_text`.
use crate::cards::helpers::*;

pub fn card() -> CardDefinition {
    CardDefinition {
        card_id: cid("green-suns-zenith"),
        name: "Green Sun's Zenith".to_string(),
        mana_cost: Some(ManaCost {
            green: 1,
            x_count: 1,
            ..Default::default()
        }),
        types: types(&[CardType::Sorcery]),
        oracle_text: "Search your library for a green creature card with mana value X or less, \
                      put it onto the battlefield, then shuffle. Shuffle Green Sun's Zenith into \
                      its owner's library."
            .to_string(),
        abilities: vec![AbilityDefinition::Spell {
            // CR 202.3/608.2h: "mana value X or less" is the runtime cap
            // `TargetFilter.max_cmc_amount`, resolved by the `Effect::SearchLibrary`
            // executor (`effects/mod.rs:3707-3710`) against `ctx.x_value`
            // (`effects/mod.rs:8264`). Precedents: eldritch_evolution, birthing_pod,
            // birthing_ritual. PB-DX27 (2026-08-13): the previous
            // `// TODO: max_cmc should be XValue, not fixed` claimed a DSL gap that had
            // already closed — the field shipped with PB-EF10.
            //
            // The replacement half is `self_shuffle_on_resolution` (below), the same
            // shape nexus_of_fate.rs uses.
            effect: Effect::SearchLibrary {
                filter: TargetFilter {
                    has_card_type: Some(CardType::Creature),
                    colors: Some([Color::Green].into_iter().collect()),
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
        }],
        // PB-DX27: promoted from `partial`. Both printed clauses are now authored —
        // the X-capped green creature tutor and the self-shuffle replacement.
        // Declared EXPLICITLY rather than left to the `#[default]` derive, per OOS-RR3-1.
        completeness: Completeness::Complete,
        self_shuffle_on_resolution: true,
        ..Default::default()
    }
}
