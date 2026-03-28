// Spelunking — {2}{G}, Enchantment
// When this enchantment enters, draw a card, then you may put a land card from your
// hand onto the battlefield. If you put a Cave onto the battlefield this way, you
// gain 4 life.
// Lands you control enter untapped.
//
// TODO: "Put land from hand" + Cave detection + "lands enter untapped" not expressible.
use crate::cards::helpers::*;

pub fn card() -> CardDefinition {
    CardDefinition {
        card_id: cid("spelunking"),
        name: "Spelunking".to_string(),
        mana_cost: Some(ManaCost { generic: 2, green: 1, ..Default::default() }),
        types: types(&[CardType::Enchantment]),
        oracle_text: "When this enchantment enters, draw a card, then you may put a land card from your hand onto the battlefield. If you put a Cave onto the battlefield this way, you gain 4 life.\nLands you control enter untapped.".to_string(),
        abilities: vec![
            AbilityDefinition::Triggered {
                trigger_condition: TriggerCondition::WhenEntersBattlefield,
                effect: Effect::DrawCards {
                    player: PlayerTarget::Controller,
                    count: EffectAmount::Fixed(1),
                },
                // TODO: "Then put a land from hand" not expressible.
                intervening_if: None,
                targets: vec![],

                modes: None,
                trigger_zone: None,
            },
            // TODO: "Lands enter untapped" replacement effect not expressible.
        ],
        ..Default::default()
    }
}
