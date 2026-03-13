// Overwhelming Stampede — {3}{G}{G}, Sorcery
// Until end of turn, creatures you control gain trample and get +X/+X,
// where X is the greatest power among creatures you control.
use crate::cards::helpers::*;

pub fn card() -> CardDefinition {
    CardDefinition {
        card_id: cid("overwhelming-stampede"),
        name: "Overwhelming Stampede".to_string(),
        mana_cost: Some(ManaCost { generic: 3, green: 2, ..Default::default() }),
        types: types(&[CardType::Sorcery]),
        oracle_text: "Until end of turn, creatures you control gain trample and get +X/+X, where X is the greatest power among creatures you control.".to_string(),
        abilities: vec![
            // TODO: Spell effect — grant trample and +X/+X to all creatures you control until end
            // of turn, where X = greatest power among your creatures.
            // DSL gap: no dynamic X value based on max power among permanents you control.
        ],
        ..Default::default()
    }
}
