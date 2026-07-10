// Noble Hierarch — {G}, Creature — Human Druid 0/1
// Exalted; {T}: Add {G}, {W}, or {U}.
use crate::cards::helpers::*;

pub fn card() -> CardDefinition {
    CardDefinition {
        card_id: cid("noble-hierarch"),
        name: "Noble Hierarch".to_string(),
        mana_cost: Some(ManaCost { green: 1, ..Default::default() }),
        types: creature_types(&["Human", "Druid"]),
        oracle_text: "Exalted (Whenever a creature you control attacks alone, that creature gets +1/+1 until end of turn.)\n{T}: Add {G}, {W}, or {U}.".to_string(),
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
                activation_zone: None,
            once_per_turn: false,
            },
        ],
        ..Default::default()
    }
}
