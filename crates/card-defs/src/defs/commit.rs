// Commit // Memory — Put target spell or nonland permanent into its owner's library second
use crate::cards::helpers::*;

pub fn card() -> CardDefinition {
    CardDefinition {
        card_id: cid("commit"),
        name: "Commit // Memory".to_string(),
        mana_cost: Some(ManaCost { generic: 3, blue: 1, ..Default::default() }),
        types: types(&[CardType::Instant]),
        oracle_text: "Put target spell or nonland permanent into its owner's library second from the top.".to_string(),
        abilities: vec![],
        completeness: Completeness::inert("Blocked on: (1) LibraryPosition has no Nth-from-top variant — 'into its owner's library second from the top' is inexpressible (Top/Bottom/ShuffledIn only). (2) The Memory half of this split card ({4}{U}{U} sorcery) is not authored. Note that targeting a spell OR a nonland permanent is expressible today (TargetPermanentWithFilter{non_land: true} / TargetSpell)."),
        ..Default::default()
    }
}
