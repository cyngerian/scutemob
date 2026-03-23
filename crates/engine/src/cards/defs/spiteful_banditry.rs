// Spiteful Banditry — {X}{R}{R}, Enchantment
// When this enchantment enters, it deals X damage to each creature.
// Whenever one or more creatures your opponents control die, you create a Treasure
// token. This ability triggers only once each turn.
use crate::cards::helpers::*;

pub fn card() -> CardDefinition {
    CardDefinition {
        card_id: cid("spiteful-banditry"),
        name: "Spiteful Banditry".to_string(),
        // TODO: {X}{R}{R} mana cost — X costs not representable in ManaCost struct.
        mana_cost: Some(ManaCost { red: 2, ..Default::default() }),
        types: types(&[CardType::Enchantment]),
        oracle_text: "When this enchantment enters, it deals X damage to each creature.\nWhenever one or more creatures your opponents control die, you create a Treasure token. This ability triggers only once each turn.".to_string(),
        abilities: vec![
            // TODO: ETB "deals X damage to each creature" — requires X value from casting
            //   cost. DSL has no EffectAmount::XCost variant. W5 policy: omitted.
            // TODO: "Whenever one or more creatures your opponents control die" —
            //   WheneverCreatureDies fires per creature death, but has no opponent-only
            //   filter. The "only once each turn" throttle is also not in DSL.
            //   W5 policy: no approximation.
        ],
        ..Default::default()
    }
}
