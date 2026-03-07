// Steel Guardian — SYNTHETIC TEST CARD (no real MTG card with this name and stats)
//
// Purpose: Validates the Living Metal (CR 702.161) engine implementation.
// All real Living Metal cards (Optimus Prime, Jetfire, etc.) are Transformers
// double-faced cards, which require the blocked DFC subsystem (deferred). This
// synthetic 3/3 Artifact Vehicle with Living Metal stands in for testing until
// DFC support is implemented.
//
// Synthetic card parameters:
//   Name: Steel Guardian
//   Cost: {2}
//   Type: Artifact — Vehicle
//   P/T: 3/3
//   Ability: Living Metal (CR 702.161a: during your turn, this permanent is an
//            artifact creature in addition to its other types)
//   Crew: 2 (standard Vehicle; allows crewing as an alternative to Living Metal)
use crate::cards::helpers::*;

pub fn card() -> CardDefinition {
    CardDefinition {
        card_id: cid("steel-guardian"),
        name: "Steel Guardian".to_string(),
        mana_cost: Some(ManaCost { generic: 2, ..Default::default() }),
        types: types_sub(&[CardType::Artifact], &["Vehicle"]),
        oracle_text: "Living Metal (During your turn, this Vehicle is also a creature.)\nCrew 2 (Tap any number of creatures you control with total power 2 or greater: This Vehicle becomes an artifact creature until end of turn.)".to_string(),
        abilities: vec![
            AbilityDefinition::Keyword(KeywordAbility::LivingMetal),
            AbilityDefinition::Keyword(KeywordAbility::Crew(2)),
        ],
        power: Some(3),
        toughness: Some(3),
    }
}
