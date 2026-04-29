// Armorcraft Judge — {3}{G}, Creature — Elf Artificer 3/3
// When this creature enters, draw a card for each creature you control with a
// +1/+1 counter on it.
//
// CR 122.1: Counters are artifacts of game state tracked on the object.
// CR 122.6: Counters on permanents are tracked in GameObject.counters.
// Ruling 2020-11-10 (Armorcraft Judge): counts CREATURES with one or more
// +1/+1 counters (not the total number of counters); threshold is >= 1 counter.
use crate::cards::helpers::*;

pub fn card() -> CardDefinition {
    CardDefinition {
        card_id: cid("armorcraft-judge"),
        name: "Armorcraft Judge".to_string(),
        mana_cost: Some(ManaCost { generic: 3, green: 1, ..Default::default() }),
        types: creature_types(&["Elf", "Artificer"]),
        oracle_text: "When this creature enters, draw a card for each creature you control with a +1/+1 counter on it.".to_string(),
        power: Some(3),
        toughness: Some(3),
        abilities: vec![
            AbilityDefinition::Triggered {
                trigger_condition: TriggerCondition::WhenEntersBattlefield,
                effect: Effect::DrawCards {
                    player: PlayerTarget::Controller,
                    count: EffectAmount::PermanentCount {
                        filter: TargetFilter {
                            has_card_type: Some(CardType::Creature),
                            controller: TargetController::You,
                            // CR 122.6 + Ruling 2020-11-10: count creatures that have at least
                            // one +1/+1 counter on them (not total counters). The
                            // has_counter_type field is checked at runtime against
                            // GameObject.counters by check_has_counter_type(), NOT via
                            // matches_filter() (which only sees Characteristics).
                            has_counter_type: Some(CounterType::PlusOnePlusOne),
                            ..Default::default()
                        },
                        controller: PlayerTarget::Controller,
                    },
                },
                intervening_if: None,
                targets: vec![],
                modes: None,
                trigger_zone: None,
            },
        ],
        ..Default::default()
    }
}
