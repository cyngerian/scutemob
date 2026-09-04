// 54. Darksteel Colossus — {11}, Artifact Creature — Golem 11/11.
// Trample, indestructible. If Darksteel Colossus would be put into a
// graveyard from anywhere, reveal it and shuffle it into its owner's library
// instead.
//
// The self-replacement trigger uses ObjectFilter::Any as a placeholder;
// register_permanent_replacement_abilities substitutes SpecificObject(new_id)
// at registration time so the effect only fires for this specific Colossus.
// PB-DX18 (`OOS-DP2-7`, 2026-09-04): this header used to say *"'Shuffle into library'
// is simplified to RedirectToZone(Library) (no shuffle)"*. That described the DEF's
// variant wrongly (the def uses `ShuffleIntoOwnerLibrary`, not `RedirectToZone`) and
// described the ENGINE correctly and by accident: both `ShuffleIntoOwnerLibrary` sites
// in `rules/replacement.rs` emitted `GameEvent::LibraryShuffled` and never shuffled, so
// the card landed on the library TOP and a Colossus that died was redrawn next turn.
// The engine now performs a real seeded shuffle after the redirect move
// (`GameState::finish_redirect_shuffle`), so the variant does what its name says.
use crate::cards::helpers::*;

pub fn card() -> CardDefinition {
    CardDefinition {
        card_id: cid("darksteel-colossus"),
        name: "Darksteel Colossus".to_string(),
        mana_cost: Some(ManaCost {
            generic: 11,
            ..Default::default()
        }),
        types: types_sub(&[CardType::Artifact, CardType::Creature], &["Golem"]),
        oracle_text: "Trample, indestructible.\nIf Darksteel Colossus would be put into a \
                      graveyard from anywhere, reveal it and shuffle it into its owner's library \
                      instead."
            .to_string(),
        power: Some(11),
        toughness: Some(11),
        abilities: vec![
            AbilityDefinition::Keyword(KeywordAbility::Trample),
            AbilityDefinition::Keyword(KeywordAbility::Indestructible),
            // CR 614.1a / 614.15 / 701.20: Self-replacement effect — if this specific
            // Colossus would go to a graveyard, shuffle it into its owner's library.
            // ObjectFilter::Any is replaced with SpecificObject at registration time.
            AbilityDefinition::Replacement {
                trigger: ReplacementTrigger::WouldChangeZone {
                    from: None,
                    to: ZoneType::Graveyard,
                    filter: ObjectFilter::Any,
                },
                modification: ReplacementModification::ShuffleIntoOwnerLibrary,
                is_self: true,
                unless_condition: None,
            },
        ],
        color_indicator: None,
        back_face: None,
        spell_cost_modifiers: vec![],
        self_cost_reduction: None,
        starting_loyalty: None,
        adventure_face: None,
        meld_pair: None,
        spell_additional_costs: vec![],
        activated_ability_cost_reductions: vec![],
        cant_be_countered: false,
        self_exile_on_resolution: false,
        self_shuffle_on_resolution: false,
        completeness: Completeness::known_wrong(
            // PB-DX18 (`OOS-DP2-7`): this note asserted the replacement "itself is
            // correct" while the engine's `ShuffleIntoOwnerLibrary` arm shuffled nothing
            // and put the card on the library top. It was a claim, and it was false —
            // PB-DX27's rule that a blocker note is a claim, arriving on the note that
            // covered this seed. The engine half is now true; the surviving blocker is
            // the printed "reveal it" clause, which is what keeps this def known_wrong.
            "the 'reveal it' clause is not modelled (CR 701.15). The shuffle-into-owner's-library \
             replacement itself is correct as of PB-DX18 \
             (`ReplacementModification::ShuffleIntoOwnerLibrary` now performs a real seeded \
             shuffle after the redirect move; before it emitted a phantom `LibraryShuffled` and \
             left the card on top)",
        ),
    }
}
