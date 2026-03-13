// Smuggler's Surprise — {G}, Instant
// Spree (Choose one or more additional costs.)
// + {2} — Mill four cards. You may put up to two creature and/or land cards from among the milled cards into your hand.
// + {4}{G} — You may put up to two creature cards from your hand onto the battlefield.
// + {1} — Creatures you control with power 4 or greater gain hexproof and indestructible until end of turn.
use crate::cards::helpers::*;

pub fn card() -> CardDefinition {
    CardDefinition {
        card_id: cid("smugglers-surprise"),
        name: "Smuggler's Surprise".to_string(),
        mana_cost: Some(ManaCost { green: 1, ..Default::default() }),
        types: types(&[CardType::Instant]),
        oracle_text: "Spree (Choose one or more additional costs.)\n+ {2} \u{2014} Mill four cards. You may put up to two creature and/or land cards from among the milled cards into your hand.\n+ {4}{G} \u{2014} You may put up to two creature cards from your hand onto the battlefield.\n+ {1} \u{2014} Creatures you control with power 4 or greater gain hexproof and indestructible until end of turn.".to_string(),
        abilities: vec![
            AbilityDefinition::Keyword(KeywordAbility::Spree),
            // TODO: Spree mode 1 — mill 4, put up to two creature/land cards milled into hand.
            // TODO: Spree mode 2 — put up to two creature cards from hand onto battlefield.
            // TODO: Spree mode 3 — creatures you control with power 4+ gain hexproof and indestructible until end of turn.
            // DSL gap: Spree modes not yet wired to Effect enum; power-filter for keyword grants.
        ],
        ..Default::default()
    }
}
