// Path of Ancestry — Land; enters tapped.
// "{T}: Add one mana of any color in your commander's color identity.
// When that mana is spent to cast a spell that shares a creature type with
// your commander, scry 1."
// TODO: DSL gap — creature-type comparison + conditional scry on mana spend
// not expressible. Modeled as ETB tapped + any-color mana (like Command Tower
// but tapped).
use crate::cards::helpers::*;

pub fn card() -> CardDefinition {
    CardDefinition {
        card_id: cid("path-of-ancestry"),
        name: "Path of Ancestry".to_string(),
        mana_cost: None,
        types: types(&[CardType::Land]),
        oracle_text: "Path of Ancestry enters the battlefield tapped.\n{T}: Add one mana of any \
                      color in your commander's color identity. When that mana is spent to cast a \
                      spell that shares a creature type with your commander, scry 1."
            .to_string(),
        abilities: vec![
            AbilityDefinition::Replacement {
                trigger: ReplacementTrigger::WouldEnterBattlefield {
                    filter: ObjectFilter::Any,
                },
                modification: ReplacementModification::EntersTapped,
                is_self: true,
                unless_condition: None,
            },
            AbilityDefinition::Activated {
                cost: Cost::Tap,
                effect: Effect::AddManaAnyColor {
                    player: PlayerTarget::Controller,
                },
                // TODO: conditional scry on creature-type match
                timing_restriction: None,
                targets: vec![],
                activation_condition: None,
                activation_zone: None,
                once_per_turn: false,
                modes: None,
            },
        ],
        completeness: Completeness::known_wrong(
            "Two issues, unchanged by PB-EF12: (1) blocked on mana-spend provenance tracking — no \
             way to trigger on how produced mana is later spent, so the conditional scry is \
             omitted; (2) the mana ability's colour choice is now real (PB-EF12 / EF-W-PB2-3 \
             fixed the colorless-stub half), but it is still offered from all five colours, more \
             permissive than 'any color in your commander's color identity' — the engine has no \
             runtime mechanism to restrict the option set to the commander's identity (same gap \
             as command_tower.rs). Reclassified partial -> known_wrong: with the colour channel \
             now real, this card can produce a colour genuinely outside its printed legal set, \
             which is wrong game state, not merely an omitted clause. Filed as OOS-EF12-1.",
        ),
        ..Default::default()
    }
}
