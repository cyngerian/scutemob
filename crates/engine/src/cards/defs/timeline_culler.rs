// Timeline Culler — {B}{B}, Creature — Drix Warlock 2/2
// Haste
// You may cast this card from your graveyard using its warp ability.
// Warp—{B}, Pay 2 life.
// TODO: DSL gap — Warp is not an implemented AltCostKind variant. The "cast from exile on
// a later turn" loop and the "exile at next end step" replacement have no DSL support.
use crate::cards::helpers::*;

pub fn card() -> CardDefinition {
    CardDefinition {
        card_id: cid("timeline-culler"),
        name: "Timeline Culler".to_string(),
        mana_cost: Some(ManaCost { black: 2, ..Default::default() }),
        types: creature_types(&["Drix", "Warlock"]),
        oracle_text: "Haste\nYou may cast this card from your graveyard using its warp ability.\nWarp\u{2014}{B}, Pay 2 life. (You may cast this card from your hand or graveyard for its warp cost. If you do, exile this creature at the beginning of the next end step, then you may cast it from exile on a later turn.)".to_string(),
        power: Some(2),
        toughness: Some(2),
        abilities: vec![
            AbilityDefinition::Keyword(KeywordAbility::Haste),
            // TODO: Warp {B}, Pay 2 life — not an implemented AltCostKind
        ],
        ..Default::default()
    }
}
