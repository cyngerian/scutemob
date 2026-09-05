// Minas Tirith
use crate::cards::helpers::*;

pub fn card() -> CardDefinition {
    CardDefinition {
        card_id: cid("minas-tirith"),
        name: "Minas Tirith".to_string(),
        mana_cost: None,
        types: supertypes(&[SuperType::Legendary], &[CardType::Land]),
        oracle_text: "Minas Tirith enters tapped unless you control a legendary creature.\n{T}: \
                      Add {W}.\n{1}{W}, {T}: Draw a card. Activate only if you attacked with two \
                      or more creatures this turn."
            .to_string(),
        abilities: vec![
            // CR 614.1c: enters tapped unless you control a legendary creature.
            AbilityDefinition::Replacement {
                trigger: ReplacementTrigger::WouldEnterBattlefield {
                    filter: ObjectFilter::Any,
                },
                modification: ReplacementModification::EntersTapped,
                is_self: true,
                unless_condition: Some(Condition::ControlLegendaryCreature),
            },
            AbilityDefinition::Activated {
                cost: Cost::Tap,
                effect: Effect::AddMana {
                    player: PlayerTarget::Controller,
                    mana: mana_pool(1, 0, 0, 0, 0, 0),
                },
                timing_restriction: None,
                targets: vec![],
                activation_condition: None,
                activation_zone: None,
                once_per_turn: false,
                modes: None,
            },
            // CR 508.1/CR 508.4, ruling 2007-10-01 (PB-DX53): "{1}{W}, {T}: Draw a card.
            // Activate only if you attacked with two or more creatures this turn." Per-turn,
            // deduplicated attack-count gate -- Condition::YouAttackedWithNOrMoreCreaturesThisTurn
            // reads PlayerState.creatures_declared_as_attackers_this_turn (an OrdSet<ObjectId>,
            // deduplicated by CR 400.7 identity, CR 508.4 entrants excluded by construction).
            // An earlier version of this file carried an ENGINE-BLOCKED comment asserting
            // that no engine primitive could express a count-based attack condition, and
            // naming a `Condition` variant that never existed under that spelling. Stated
            // precisely, because the first replacement for it overcorrected and was itself
            // false: that note was TRUE WHEN WRITTEN -- it is present in `b6f748f8`
            // (2026-07-10) and `Condition::YouAttackedWithNOrMore` was not added until
            // PB-OS6's `bc79a72c` (2026-07-19). It ROTTED, outliving by nine days the commit
            // that falsified it, and was still false at HEAD when PB-DX53 read it
            // (`OOS-DX47-6`'s shape). It was never false at authoring time, and a comment
            // that says so is the same defect one direction over.
            AbilityDefinition::Activated {
                cost: Cost::Sequence(vec![
                    Cost::Mana(ManaCost {
                        generic: 1,
                        white: 1,
                        ..Default::default()
                    }),
                    Cost::Tap,
                ]),
                effect: Effect::DrawCards {
                    player: PlayerTarget::Controller,
                    count: EffectAmount::Fixed(1),
                },
                timing_restriction: None,
                targets: vec![],
                activation_condition: Some(Condition::YouAttackedWithNOrMoreCreaturesThisTurn(2)),
                activation_zone: None,
                once_per_turn: false,
                modes: None,
            },
        ],
        completeness: Completeness::Complete,
        ..Default::default()
    }
}
