// Brallin, Skyshark Rider — {3}{R}, Legendary Creature — Human Shaman 3/3
// Partner with Shabraz, the Skyshark
// Whenever you discard a card, put a +1/+1 counter on Brallin and it deals 1 damage to each opponent.
// {R}: Target Shark gains trample until end of turn.
use crate::cards::helpers::*;

pub fn card() -> CardDefinition {
    CardDefinition {
        card_id: cid("brallin-skyshark-rider"),
        name: "Brallin, Skyshark Rider".to_string(),
        mana_cost: Some(ManaCost { generic: 3, red: 1, ..Default::default() }),
        types: full_types(
            &[SuperType::Legendary],
            &[CardType::Creature],
            &["Human", "Shaman"],
        ),
        oracle_text: "Partner with Shabraz, the Skyshark (When this creature enters, target player may put Shabraz into their hand from their library, then shuffle.)\nWhenever you discard a card, put a +1/+1 counter on Brallin and it deals 1 damage to each opponent.\n{R}: Target Shark gains trample until end of turn.".to_string(),
        power: Some(3),
        toughness: Some(3),
        abilities: vec![
            AbilityDefinition::Keyword(KeywordAbility::PartnerWith(
                "Shabraz, the Skyshark".to_string(),
            )),
            // Whenever you discard a card, put a +1/+1 counter on Brallin and deal 1 to each opponent.
            AbilityDefinition::Triggered {
                trigger_condition: TriggerCondition::WheneverYouDiscard,
                effect: Effect::Sequence(vec![
                    Effect::AddCounter {
                        target: EffectTarget::Source,
                        counter: CounterType::PlusOnePlusOne,
                        count: 1,
                    },
                    Effect::ForEach {
                        over: ForEachTarget::EachOpponent,
                        effect: Box::new(Effect::DealDamage {
                            target: EffectTarget::DeclaredTarget { index: 0 },
                            amount: EffectAmount::Fixed(1),
                        }),
                    },
                ]),
                intervening_if: None,
                targets: vec![],
            },
            // TODO: "{R}: Target Shark gains trample until end of turn."
            // Requires subtype-filtered targeting + grant keyword continuous effect.
        ],
        ..Default::default()
    }
}
