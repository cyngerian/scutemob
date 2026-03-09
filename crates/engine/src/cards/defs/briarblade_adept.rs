// Briarblade Adept — {4}{B}, Creature — Elf Assassin 3/4; Encore {3}{B}
// TODO(targeted_trigger): "Whenever this creature attacks, target creature an opponent
// controls gets -1/-1 until end of turn" requires targeted triggered abilities, which
// is a known DSL gap (W5 worklist: targeted_trigger). Omitted from abilities vec per
// W5 policy until the DSL supports TargetRequirement on AbilityDefinition::Triggered.
use crate::cards::helpers::*;

pub fn card() -> CardDefinition {
    CardDefinition {
        card_id: cid("briarblade-adept"),
        name: "Briarblade Adept".to_string(),
        mana_cost: Some(ManaCost { generic: 4, black: 1, ..Default::default() }),
        types: creature_types(&["Elf", "Assassin"]),
        oracle_text: "Whenever this creature attacks, target creature an opponent controls gets -1/-1 until end of turn.\nEncore {3}{B} ({3}{B}, Exile this card from your graveyard: For each opponent, create a token copy that attacks that opponent this turn if able. They gain haste. Sacrifice them at the beginning of the next end step. Activate only as a sorcery.)".to_string(),
        abilities: vec![
            // Attack trigger omitted — targeted_trigger DSL gap; see TODO above.
            AbilityDefinition::Keyword(KeywordAbility::Encore),
            AbilityDefinition::Encore {
                cost: ManaCost { generic: 3, black: 1, ..Default::default() },
            },
        ],
        power: Some(3),
        toughness: Some(4),
        back_face: None,
    }
}
