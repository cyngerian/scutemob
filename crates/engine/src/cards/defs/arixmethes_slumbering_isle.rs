// Arixmethes, Slumbering Isle — {2}{G}{U}, Legendary Creature — Kraken 12/12
// Arixmethes enters tapped with five slumber counters on it.
// As long as Arixmethes has a slumber counter on it, it's a land. (It's not a creature.)
// Whenever you cast a spell, you may remove a slumber counter from Arixmethes.
// {T}: Add {G}{U}.
//
// CR 614.1c: "Enters tapped" — self-ETB replacement.
// CR 604.2 / CR 613.1d (Layer 4): "As long as Arixmethes has a slumber counter on it,
// it's a land. (It's not a creature.)" Two effects: RemoveCardTypes(Creature) and (implicitly)
// the existing Creature type is suppressed. Arixmethes remains a Creature in its base type
// line, but the conditional static removes Creature when slumber counters are present.
//
// NOTE: "Enters with five slumber counters" — ETB-with-counters replacement not in DSL.
// The enters-tapped replacement IS implemented; the counter placement is a TODO.
//
// TODO: "Whenever you cast a spell, you may remove a slumber counter from Arixmethes."
// DSL gap: no WheneverYouCastASpell trigger condition.
use crate::cards::helpers::*;

pub fn card() -> CardDefinition {
    CardDefinition {
        card_id: cid("arixmethes-slumbering-isle"),
        name: "Arixmethes, Slumbering Isle".to_string(),
        mana_cost: Some(ManaCost { generic: 2, blue: 1, green: 1, ..Default::default() }),
        types: full_types(&[SuperType::Legendary], &[CardType::Creature], &["Kraken"]),
        oracle_text: "Arixmethes enters tapped with five slumber counters on it.\nAs long as Arixmethes has a slumber counter on it, it's a land. (It's not a creature.)\nWhenever you cast a spell, you may remove a slumber counter from Arixmethes.\n{T}: Add {G}{U}.".to_string(),
        abilities: vec![
            // CR 614.1c: self-replacement — enters tapped.
            AbilityDefinition::Replacement {
                trigger: ReplacementTrigger::WouldEnterBattlefield {
                    filter: ObjectFilter::Any,
                },
                modification: ReplacementModification::EntersTapped,
                is_self: true,
                unless_condition: None,
            },
            // CR 604.2 / CR 613.1d (Layer 4): "As long as Arixmethes has a slumber counter
            // on it, it's a land." Adds Land type when slumber counters are present.
            AbilityDefinition::Static {
                continuous_effect: ContinuousEffectDef {
                    layer: EffectLayer::TypeChange,
                    modification: LayerModification::AddCardTypes(
                        [CardType::Land].into_iter().collect(),
                    ),
                    filter: EffectFilter::Source,
                    duration: EffectDuration::WhileSourceOnBattlefield,
                    condition: Some(Condition::SourceHasCounters {
                        counter: CounterType::Slumber,
                        min: 1,
                    }),
                },
            },
            // CR 604.2 / CR 613.1d (Layer 4): "As long as Arixmethes has a slumber counter
            // on it, it's not a creature." Removes Creature type when slumber counters present.
            AbilityDefinition::Static {
                continuous_effect: ContinuousEffectDef {
                    layer: EffectLayer::TypeChange,
                    modification: LayerModification::RemoveCardTypes(
                        [CardType::Creature].into_iter().collect(),
                    ),
                    filter: EffectFilter::Source,
                    duration: EffectDuration::WhileSourceOnBattlefield,
                    condition: Some(Condition::SourceHasCounters {
                        counter: CounterType::Slumber,
                        min: 1,
                    }),
                },
            },
            // {T}: Add {G}{U}.
            AbilityDefinition::Activated {
                cost: Cost::Tap,
                effect: Effect::AddMana {
                    player: PlayerTarget::Controller,
                    mana: mana_pool(0, 1, 0, 0, 1, 0),
                },
                timing_restriction: None,
                targets: vec![],
                activation_condition: None,
            },
            // TODO: ETB — five slumber counters (ETB-with-counters replacement not in DSL).
            // TODO: "Whenever you cast a spell, you may remove a slumber counter from Arixmethes."
            // DSL gap: WheneverYouCastASpell trigger condition.
        ],
        power: Some(12),
        toughness: Some(12),
        ..Default::default()
    }
}
