// Ganax, Astral Hunter — {4}{R}, Legendary Creature — Dragon 3/4
// Flying
// Whenever Ganax or another Dragon you control enters, create a Treasure token.
// Choose a Background
use crate::cards::helpers::*;

pub fn card() -> CardDefinition {
    CardDefinition {
        card_id: cid("ganax-astral-hunter"),
        name: "Ganax, Astral Hunter".to_string(),
        mana_cost: Some(ManaCost { generic: 4, red: 1, ..Default::default() }),
        types: full_types(&[SuperType::Legendary], &[CardType::Creature], &["Dragon"]),
        oracle_text: "Flying\nWhenever Ganax or another Dragon you control enters, create a Treasure token.\nChoose a Background".to_string(),
        power: Some(3),
        toughness: Some(4),
        abilities: vec![
            AbilityDefinition::Keyword(KeywordAbility::Flying),
            // TODO: ENGINE-BLOCKED — "Whenever Ganax or another Dragon you control enters,
            // create a Treasure token." The WheneverCreatureEntersBattlefield trigger is
            // converted to an ETBTriggerFilter (state/game_object.rs:560) which has NO
            // subtype field — only creature_only/controller_you/exclude_self/color_filter/
            // card_type_filter. The TargetFilter.has_subtype set here is silently dropped at
            // replay_harness.rs:2371, so the trigger would fire for EVERY creature you control
            // entering, not just Dragons. Needs ETBTriggerFilter to carry a subtype filter
            // (or the creature-ETB path to forward triggering_creature_filter like the
            // death-trigger path does). Authoring-only batch — cannot make the engine change.
            AbilityDefinition::Keyword(KeywordAbility::ChooseABackground),
        ],
        ..Default::default()
    }
}
