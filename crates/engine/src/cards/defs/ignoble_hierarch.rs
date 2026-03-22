// Ignoble Hierarch — {G}, Creature — Goblin Shaman 0/1
// Exalted; {T}: Add {B}, {R}, or {G}.
use crate::cards::helpers::*;

pub fn card() -> CardDefinition {
    CardDefinition {
        card_id: cid("ignoble-hierarch"),
        name: "Ignoble Hierarch".to_string(),
        mana_cost: Some(ManaCost { green: 1, ..Default::default() }),
        types: creature_types(&["Goblin", "Shaman"]),
        oracle_text: "Exalted (Whenever a creature you control attacks alone, that creature gets +1/+1 until end of turn.)\n{T}: Add {B}, {R}, or {G}.".to_string(),
        power: Some(0),
        toughness: Some(1),
        abilities: vec![
            AbilityDefinition::Keyword(KeywordAbility::Exalted),
            AbilityDefinition::Activated {
                cost: Cost::Tap,
                effect: Effect::AddManaChoice { player: PlayerTarget::Controller, count: EffectAmount::Fixed(1) },
                timing_restriction: None,
                targets: vec![],
                activation_condition: None,
            },
        ],
        ..Default::default()
    }
}
