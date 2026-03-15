// Toothy, Imaginary Friend — {3}{U}, Legendary Creature — Illusion 1/1
// Partner with Pir, Imaginative Rascal (ETB trigger handled by PartnerWith keyword).
// "Whenever you draw a card, put a +1/+1 counter on Toothy." — triggered ability.
// "When Toothy leaves the battlefield, draw a card for each +1/+1 counter on it." — triggered.
// TODO: Draw-triggered counter ability — TriggerCondition::WheneverYouDrawACard does not yet
// exist in the DSL. Requires a new TriggerCondition variant and Effect::PutCountersOnSelf
// (or targeting self). Until those variants exist, this ability is omitted.
// TODO: Leaves-the-battlefield draw trigger — TriggerCondition::WhenLeavesBattlefield does
// not yet exist in the DSL. Requires a new TriggerCondition variant and
// Effect::DrawCards { count: EffectAmount::CounterCountOnSelf { kind: CounterKind::PlusOnePlusOne } }.
// Until those variants exist, this ability is omitted.
use crate::cards::helpers::*;

pub fn card() -> CardDefinition {
    CardDefinition {
        card_id: cid("toothy-imaginary-friend"),
        name: "Toothy, Imaginary Friend".to_string(),
        mana_cost: Some(ManaCost { generic: 3, blue: 1, ..Default::default() }),
        types: full_types(&[SuperType::Legendary], &[CardType::Creature], &["Illusion"]),
        oracle_text:
            "Partner with Pir, Imaginative Rascal (When this creature enters the battlefield, \
             target player may search their library for a card named Pir, Imaginative Rascal, \
             reveal it, put it into their hand, then shuffle.)\n\
             Whenever you draw a card, put a +1/+1 counter on Toothy, Imaginary Friend.\n\
             When Toothy leaves the battlefield, draw a card for each +1/+1 counter on it."
                .to_string(),
        power: Some(1),
        toughness: Some(1),
        abilities: vec![
            // CR 702.124j: "Partner with [name]" — ETB trigger searches for named partner.
            AbilityDefinition::Keyword(KeywordAbility::PartnerWith(
                "Pir, Imaginative Rascal".to_string(),
            )),
            // TODO: Draw-triggered +1/+1 counter — requires TriggerCondition::WheneverYouDrawACard
            // and Effect::PutCountersOnSelf (not yet in DSL).
            // TODO: Leaves-the-battlefield draw trigger — requires
            // TriggerCondition::WhenLeavesBattlefield and
            // Effect::DrawCards with EffectAmount::CounterCountOnSelf (not yet in DSL).
        ],
        color_indicator: None,
        back_face: None,
        spell_cost_modifiers: vec![],
        self_cost_reduction: None,
    }
}
