// Arixmethes, Slumbering Isle
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
            // Note: oracle says "enters tapped WITH five slumber counters" — counters are a
            // DSL gap but enters-tapped is expressible and reduces wrong game state.
            AbilityDefinition::Replacement {
                trigger: ReplacementTrigger::WouldEnterBattlefield {
                    filter: ObjectFilter::Any,
                },
                modification: ReplacementModification::EntersTapped,
                is_self: true,
                unless_condition: None,
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
            // TODO: Static — "as long as it has a slumber counter, it's a land (not a creature)"
            // DSL gap: conditional type-change (Layer 4) based on counter presence.
            // TODO: Triggered — "whenever you cast a spell, may remove a slumber counter"
            // DSL gap: WheneverYouCastASpell trigger condition.
        ],
        power: Some(12),
        toughness: Some(12),
        ..Default::default()
    }
}
