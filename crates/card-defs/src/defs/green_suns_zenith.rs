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
            // PB-DX27 /review (HIGH): "then shuffle" is a SEPARATE printed clause and must
            // be authored explicitly. `Effect::SearchLibrary`'s only shuffle is the
            // `shuffle_before_placing` branch (`effects/mod.rs:3839-3844`), which shuffles
            // BEFORE placing, not after. `eldritch_evolution.rs:12-14` says so in-source,
            // and the first draft of this def cited that file as precedent while omitting
            // exactly what its comment warns about.
            effect: Effect::Sequence(vec![
                Effect::SearchLibrary {
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
                Effect::Shuffle {
                    player: PlayerTarget::Controller,
                },
            ]),
            targets: vec![],
            modes: None,
            cant_be_countered: false,
        }],
        // PB-DX27 /review: this def was promoted to `Complete` by the implement phase and
        // is DEMOTED BACK by the review, which is the honest outcome.
        //
        // The first printed clause IS now fully authored — the X-capped green creature
        // tutor plus the explicit `Effect::Shuffle` for "then shuffle" (see above).
        //
        // The SECOND clause is not. `self_shuffle_on_resolution` does not shuffle: it
        // selects `ZoneId::Library(owner)` and then plain `move_object_to_zone`s the card
        // there, and `resolution.rs:2023-2025` says so in its own comment — "Engine uses
        // deterministic library placement (top of library). Proper shuffling requires
        // external randomization outside engine scope." So the card lands on TOP of the
        // library, fully known, where the printed card shuffles it in.
        //
        // `nexus_of_fate` is the corpus's only other user of the flag and is `partial` for
        // exactly this reason. This def claiming `Complete` on the identical mechanism was
        // the same shape PB-DX27 demoted `qarsi_sadist` for — an outlier nobody had ruled
        // on — reproduced inside the batch that filed it. The blocker is engine-side: a
        // shuffle-in placement for the `self_shuffle` branch, not a card-def change.
        completeness: Completeness::partial(
            "Clause 2 only: 'Shuffle Green Sun's Zenith into its owner's library' is a \
             deterministic TOP-of-library placement, not a shuffle — resolution.rs:2023-2025 \
             documents the deviation in-source, and nexus_of_fate (the flag's only other user) is \
             partial for the same reason. Clause 1 (the X-capped green creature tutor + 'then \
             shuffle') is fully authored: TargetFilter.max_cmc_amount = EffectAmount::XValue plus \
             an explicit Effect::Shuffle.",
        ),
        self_shuffle_on_resolution: true,
        ..Default::default()
    }
}
