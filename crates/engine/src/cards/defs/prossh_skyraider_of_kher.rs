// Prossh, Skyraider of Kher — {3}{B}{R}{G}, Legendary Creature — Dragon 5/5
// When you cast this spell, create X 0/1 red Kobold creature tokens named Kobolds of
// Kher Keep, where X is the amount of mana spent to cast it.
// Flying
// Sacrifice another creature: Prossh gets +1/+0 until end of turn.
use crate::cards::helpers::*;

pub fn card() -> CardDefinition {
    CardDefinition {
        card_id: cid("prossh-skyraider-of-kher"),
        name: "Prossh, Skyraider of Kher".to_string(),
        mana_cost: Some(ManaCost { generic: 3, black: 1, red: 1, green: 1, ..Default::default() }),
        types: full_types(&[SuperType::Legendary], &[CardType::Creature], &["Dragon"]),
        oracle_text: "When you cast this spell, create X 0/1 red Kobold creature tokens named Kobolds of Kher Keep, where X is the amount of mana spent to cast it.\nFlying\nSacrifice another creature: Prossh gets +1/+0 until end of turn.".to_string(),
        power: Some(5),
        toughness: Some(5),
        abilities: vec![
            // TODO: "When you cast this spell" cast trigger creating X tokens where X = mana spent
            // — cast triggers with X-equals-mana-spent not in DSL
            AbilityDefinition::Keyword(KeywordAbility::Flying),
            // TODO: "Sacrifice another creature: Prossh gets +1/+0 until end of turn"
            // — pump effect (Effect for +N/+M until EOT) not in DSL
        ],
        ..Default::default()
    }
}
