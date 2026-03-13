// Qarsi Revenant — {1}{B}{B}, Creature — Vampire 3/3
// Flying, deathtouch, lifelink
// Renew — {2}{B}, Exile this card from your graveyard: Put a flying counter, a deathtouch counter,
// and a lifelink counter on target creature. Activate only as a sorcery.
//
// TODO: Renew activated ability cannot be expressed — it activates from the graveyard
// (not battlefield), costs exile-self, and puts multiple keyword counters on a target.
// None of these are supported in the current DSL.
use crate::cards::helpers::*;

pub fn card() -> CardDefinition {
    CardDefinition {
        card_id: cid("qarsi-revenant"),
        name: "Qarsi Revenant".to_string(),
        mana_cost: Some(ManaCost { generic: 1, black: 2, ..Default::default() }),
        types: creature_types(&["Vampire"]),
        oracle_text: "Flying, deathtouch, lifelink\nRenew — {2}{B}, Exile this card from your graveyard: Put a flying counter, a deathtouch counter, and a lifelink counter on target creature. Activate only as a sorcery.".to_string(),
        power: Some(3),
        toughness: Some(3),
        abilities: vec![
            AbilityDefinition::Keyword(KeywordAbility::Flying),
            AbilityDefinition::Keyword(KeywordAbility::Deathtouch),
            AbilityDefinition::Keyword(KeywordAbility::Lifelink),
            // TODO: Renew — graveyard activated ability with exile-self cost and keyword counter
            // placement not supported in DSL.
        ],
        ..Default::default()
    }
}
