// Deep Gnome Terramancer — {1}{W}, Creature — Gnome Wizard 2/2
// Flash
// Mold Earth — Whenever one or more lands enter under an opponent's control without
// being played, you may search your library for a Plains card, put it onto the
// battlefield tapped, then shuffle. Do this only once each turn.
// TODO: "lands enter under opponent's control without being played" trigger condition
// not expressible in current DSL — no TriggerCondition variant for
// "opponent gets a non-played land". Requires a new trigger condition
// (e.g. WheneverOpponentGetsLandWithoutPlaying).
use crate::cards::helpers::*;

pub fn card() -> CardDefinition {
    CardDefinition {
        card_id: cid("deep-gnome-terramancer"),
        name: "Deep Gnome Terramancer".to_string(),
        mana_cost: Some(ManaCost { generic: 1, white: 1, ..Default::default() }),
        types: creature_types(&["Gnome", "Wizard"]),
        oracle_text: "Flash\nMold Earth — Whenever one or more lands enter under an opponent's control without being played, you may search your library for a Plains card, put it onto the battlefield tapped, then shuffle. Do this only once each turn.".to_string(),
        power: Some(2),
        toughness: Some(2),
        abilities: vec![
            AbilityDefinition::Keyword(KeywordAbility::Flash),
            // TODO: Mold Earth triggered ability — needs TriggerCondition::WheneverOpponentGetsLandWithoutPlaying (or equivalent)
        ],
        ..Default::default()
    }
}
