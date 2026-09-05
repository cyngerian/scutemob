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
            // An earlier version of this file carried a comment asserting that no engine
            // primitive could express a count-based attack condition and citing a Condition
            // variant that never existed under that name. That claim was already false at the
            // time this file was authored: an attack-count Condition variant has existed since
            // PB-OS6 (2026-07-19). PB-DX53 gives the per-turn scope its own correctly-named
            // variant, which is what this ability now reads.
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
