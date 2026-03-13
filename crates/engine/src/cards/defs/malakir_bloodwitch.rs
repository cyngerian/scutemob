// Malakir Bloodwitch — {3}{B}{B}, Creature — Vampire Shaman 4/4
// Flying, protection from white
// When this creature enters, each opponent loses life equal to the number of
// Vampires you control. You gain life equal to the life lost this way.
//
// Flying and protection from white are implemented.
// TODO: DSL gap — the ETB triggered ability requires counting creatures you
// control with a specific subtype (Vampire) and using that count as a
// variable amount for ForEach/LoseLife + GainLife. No subtype-count
// EffectAmount exists in the DSL.
use crate::cards::helpers::*;
use crate::state::types::ProtectionQuality;

pub fn card() -> CardDefinition {
    CardDefinition {
        card_id: cid("malakir-bloodwitch"),
        name: "Malakir Bloodwitch".to_string(),
        mana_cost: Some(ManaCost { generic: 3, black: 2, ..Default::default() }),
        types: creature_types(&["Vampire", "Shaman"]),
        oracle_text: "Flying, protection from white\nWhen this creature enters, each opponent loses life equal to the number of Vampires you control. You gain life equal to the life lost this way.".to_string(),
        power: Some(4),
        toughness: Some(4),
        abilities: vec![
            AbilityDefinition::Keyword(KeywordAbility::Flying),
            AbilityDefinition::Keyword(KeywordAbility::ProtectionFrom(
                ProtectionQuality::FromColor(Color::White),
            )),
            // TODO: DSL gap — ETB drain equal to Vampire count not expressible.
        ],
        ..Default::default()
    }
}
