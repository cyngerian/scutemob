// Insatiable Avarice — {B}, Sorcery (Spree)
// + {2} — Search your library for a card, then shuffle and put that card on top.
// + {B}{B} — Target player draws three cards and loses 3 life.
// TODO: Spree mode 2 ({B}{B}) requires targeting a player (target player draws 3,
// loses 3). The DSL has no way to attach a TargetRequirement to an individual Spree
// mode. Implementing mode 1 alone would allow illegal casts (mode 2 chosen = no-op)
// producing wrong game state. Full card deferred per W5 policy.
use crate::cards::helpers::*;

pub fn card() -> CardDefinition {
    CardDefinition {
        card_id: cid("insatiable-avarice"),
        name: "Insatiable Avarice".to_string(),
        mana_cost: Some(ManaCost { black: 1, ..Default::default() }),
        types: types(&[CardType::Sorcery]),
        oracle_text:
            "Spree (Choose one or more additional costs.)\n\
             + {2} — Search your library for a card, then shuffle and put that card on top.\n\
             + {B}{B} — Target player draws three cards and loses 3 life."
                .to_string(),
        abilities: vec![],
        ..Default::default()
    }
}
