// Scapeshift — {2}{G}{G} Sorcery.
// "Sacrifice any number of lands. Search your library for up to that many land
// cards, put them onto the battlefield tapped, then shuffle."
//
// TODO: "Sacrifice any number of lands" as a spell cost is not expressible in
// the DSL. There is no Cost::SacrificeAnyNumberWithType or SpellAdditionalCost
// variant for "sacrifice any number of [type]". The number sacrificed must be
// tracked to gate the SearchLibrary count ("up to that many"). Both halves of
// the ability depend on this variable-count sacrifice mechanic. W5: wrong
// implementation omitted — abilities: vec![].
use crate::cards::helpers::*;

pub fn card() -> CardDefinition {
    CardDefinition {
        card_id: cid("scapeshift"),
        name: "Scapeshift".to_string(),
        mana_cost: Some(ManaCost { generic: 2, green: 2, ..Default::default() }),
        types: types(&[CardType::Sorcery]),
        oracle_text: "Sacrifice any number of lands. Search your library for up to that many land cards, put them onto the battlefield tapped, then shuffle.".to_string(),
        abilities: vec![
            // TODO: Requires Cost::SacrificeAnyNumber(TargetFilter { has_card_type: Land, .. })
            // and EffectAmount::NumberSacrificed (or similar) to cap the SearchLibrary count.
            // Neither primitive exists in the current DSL.
        ],
        ..Default::default()
    }
}
