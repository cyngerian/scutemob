// Siren Stormtamer — {U}, Creature — Siren Pirate Wizard 1/1
// Flying
// {U}, Sacrifice this creature: Counter target spell or ability that targets you or a
// creature you control.
use crate::cards::helpers::*;

pub fn card() -> CardDefinition {
    CardDefinition {
        card_id: cid("siren-stormtamer"),
        name: "Siren Stormtamer".to_string(),
        mana_cost: Some(ManaCost { blue: 1, ..Default::default() }),
        types: creature_types(&["Siren", "Pirate", "Wizard"]),
        oracle_text: "Flying\n{U}, Sacrifice this creature: Counter target spell or ability that targets you or a creature you control.".to_string(),
        power: Some(1),
        toughness: Some(1),
        abilities: vec![
            AbilityDefinition::Keyword(KeywordAbility::Flying),
            // TODO: "{U}, Sacrifice ~: Counter target spell or ability that targets you or a
            // creature you control." — requires sacrifice-as-cost + counter spell/ability with
            // targeting restriction ("targets you or a creature you control"). TargetSpell
            // doesn't support this filter. Complex interaction.
        ],
        ..Default::default()
    }
}
