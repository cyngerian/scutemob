// Elvish Dreadlord — {3}{B}{B}, Creature — Zombie Elf 3/3
// Deathtouch
// When this creature dies, non-Elf creatures get -3/-3 until end of turn.
// Encore {5}{B}{B}
use crate::cards::helpers::*;

pub fn card() -> CardDefinition {
    CardDefinition {
        card_id: cid("elvish-dreadlord"),
        name: "Elvish Dreadlord".to_string(),
        mana_cost: Some(ManaCost { generic: 3, black: 2, ..Default::default() }),
        types: creature_types(&["Zombie", "Elf"]),
        oracle_text: "Deathtouch\nWhen this creature dies, non-Elf creatures get -3/-3 until end of turn.\nEncore {5}{B}{B} ({5}{B}{B}, Exile this card from your graveyard: For each opponent, create a token copy that attacks that opponent this turn if able. They gain haste. Sacrifice them at the beginning of the next end step. Activate only as a sorcery.)".to_string(),
        power: Some(3),
        toughness: Some(3),
        abilities: vec![
            AbilityDefinition::Keyword(KeywordAbility::Deathtouch),
            AbilityDefinition::Keyword(KeywordAbility::Encore),
            // TODO: "When this creature dies, non-Elf creatures get -3/-3 until end of turn" —
            // WhenDies trigger targeting all non-Elf creatures (EffectTarget with subtype
            // exclusion filter) not in DSL.
        ],
        ..Default::default()
    }
}
