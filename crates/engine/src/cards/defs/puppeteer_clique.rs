// Puppeteer Clique — {3}{B}{B}, Creature — Faerie Wizard 3/2
// Flying
// When this creature enters, put target creature card from an opponent's graveyard onto
// the battlefield under your control. It gains haste. At the beginning of your next end
// step, exile it.
// Persist
use crate::cards::helpers::*;

pub fn card() -> CardDefinition {
    CardDefinition {
        card_id: cid("puppeteer-clique"),
        name: "Puppeteer Clique".to_string(),
        mana_cost: Some(ManaCost { generic: 3, black: 2, ..Default::default() }),
        types: creature_types(&["Faerie", "Wizard"]),
        oracle_text: "Flying\nWhen Puppeteer Clique enters, put target creature card from an opponent's graveyard onto the battlefield under your control. It gains haste. At the beginning of your next end step, exile it.\nPersist".to_string(),
        power: Some(3),
        toughness: Some(2),
        abilities: vec![
            AbilityDefinition::Keyword(KeywordAbility::Flying),
            // TODO: DSL gap — ETB reanimate from opponent's GY + haste grant + delayed
            // exile trigger at next end step. Multiple DSL gaps.
            AbilityDefinition::Keyword(KeywordAbility::Persist),
        ],
        ..Default::default()
    }
}
