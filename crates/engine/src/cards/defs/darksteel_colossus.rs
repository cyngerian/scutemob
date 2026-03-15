// 54. Darksteel Colossus — {11}, Artifact Creature — Golem 11/11.
// Trample, indestructible. If Darksteel Colossus would be put into a
// graveyard from anywhere, reveal it and shuffle it into its owner's library
// instead.
//
// The self-replacement trigger uses ObjectFilter::Any as a placeholder;
// register_permanent_replacement_abilities substitutes SpecificObject(new_id)
// at registration time so the effect only fires for this specific Colossus.
// "Shuffle into library" is simplified to RedirectToZone(Library) (no shuffle).
use crate::cards::helpers::*;

pub fn card() -> CardDefinition {
    CardDefinition {
        card_id: cid("darksteel-colossus"),
        name: "Darksteel Colossus".to_string(),
        mana_cost: Some(ManaCost { generic: 11, ..Default::default() }),
        types: types_sub(&[CardType::Artifact, CardType::Creature], &["Golem"]),
        oracle_text:
            "Trample, indestructible.\n\
             If Darksteel Colossus would be put into a graveyard from anywhere, reveal it \
             and shuffle it into its owner's library instead."
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
    }
}
