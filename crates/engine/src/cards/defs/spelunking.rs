// Spelunking — {2}{G}, Enchantment
// When this enchantment enters, draw a card, then you may put a land card from your
// hand onto the battlefield. If you put a Cave onto the battlefield this way, you
// gain 4 life.
// Lands you control enter untapped.
use crate::cards::helpers::*;

pub fn card() -> CardDefinition {
    CardDefinition {
        card_id: cid("spelunking"),
        name: "Spelunking".to_string(),
        mana_cost: Some(ManaCost { generic: 2, green: 1, ..Default::default() }),
        types: types(&[CardType::Enchantment]),
        oracle_text: "When this enchantment enters, draw a card, then you may put a land card from your hand onto the battlefield. If you put a Cave onto the battlefield this way, you gain 4 life.\nLands you control enter untapped.".to_string(),
        abilities: vec![
            // CR 603.3: ETB trigger — draw a card, then put a land from hand onto battlefield.
            AbilityDefinition::Triggered {
                trigger_condition: TriggerCondition::WhenEntersBattlefield,
                effect: Effect::Sequence(vec![
                    Effect::DrawCards {
                        player: PlayerTarget::Controller,
                        count: EffectAmount::Fixed(1),
                    },
                    Effect::PutLandFromHandOntoBattlefield { tapped: false },
                    // TODO: Cave detection — "if you put a Cave onto the battlefield this way,
                    // you gain 4 life" requires tracking which specific land entered and checking
                    // if it has the Cave subtype. This requires an effect result tracking primitive
                    // (PB-A territory) and is deferred.
                ]),
                intervening_if: None,
                targets: vec![],
                modes: None,
                trigger_zone: None,
            },
            // TODO: "Lands you control enter the battlefield untapped" — ETB replacement effect
            // for lands. Requires a replacement effect that modifies land ETB to remove the
            // tapped condition. This is a global ETB replacement and is deferred (PB-D territory).
        ],
        ..Default::default()
    }
}
