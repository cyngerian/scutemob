// Dragonstorm Globe — {3}, Artifact
// Each Dragon you control enters with an additional +1/+1 counter on it.
// {T}: Add one mana of any color.
//
// PB-EWC-D (2026-05-15): authored the counter half via
// `ReplacementModification::EntersWithCounters` with `ObjectFilter::CreatureControlledByOfSubtype
// { controller: PlayerId(0), subtype: SubType("Dragon") }`. The placeholder is bound to the
// actual controller by `bind_object_filter` at registration time. Layer-resolved creature type
// AND subtype check (CR 613.1d) performed in `object_matches_filter`.
//
// Why entering a Dragon simultaneously with Dragonstorm Globe does NOT get the counter:
// `register_permanent_replacement_abilities` runs AFTER `apply_etb_replacements` for the same
// ETB. Globe's replacement is not yet registered when Globe itself enters, and any subsequent
// Dragon ETB is a distinct object that matches the filter. (Same property as Master Biomancer.)
use crate::cards::helpers::*;

pub fn card() -> CardDefinition {
    CardDefinition {
        card_id: cid("dragonstorm-globe"),
        name: "Dragonstorm Globe".to_string(),
        mana_cost: Some(ManaCost {
            generic: 3,
            ..Default::default()
        }),
        types: types(&[CardType::Artifact]),
        oracle_text: "Each Dragon you control enters with an additional +1/+1 counter on \
                      it.\n{T}: Add one mana of any color."
            .to_string(),
        abilities: vec![
            // CR 614.1c / CR 613.1d — replacement effect: "Each Dragon you control enters
            // with an additional +1/+1 counter on it."
            //
            // `CreatureControlledByOfSubtype { controller: PlayerId(0), subtype: SubType("Dragon") }`
            // is the placeholder bound to the actual controller by `bind_object_filter` at
            // registration. Layer-resolved creature-type AND subtype check (CR 613.1d) is
            // performed in `object_matches_filter`.
            AbilityDefinition::Replacement {
                trigger: ReplacementTrigger::WouldEnterBattlefield {
                    filter: ObjectFilter::CreatureControlledByOfSubtype {
                        controller: PlayerId(0),
                        subtype: SubType("Dragon".to_string()),
                    },
                },
                modification: ReplacementModification::EntersWithCounters {
                    counter: CounterType::PlusOnePlusOne,
                    count: Box::new(EffectAmount::Fixed(1)),
                },
                is_self: false,
                unless_condition: None,
            },
            AbilityDefinition::Activated {
                cost: Cost::Tap,
                effect: Effect::AddManaAnyColor {
                    player: PlayerTarget::Controller,
                },
                timing_restriction: None,
                targets: vec![],
                activation_condition: None,
                activation_zone: None,
                once_per_turn: false,
            },
        ],
        ..Default::default()
    }
}
