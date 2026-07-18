// Blood Seeker — {1}{B}, Creature — Vampire Shaman 1/1
// Whenever a creature an opponent controls enters, you may have that player lose 1 life.
//
// TODO: "that player" — effect should target the entering creature's controller specifically,
//   not all opponents. PlayerTarget lacks "triggering player" reference. Additionally, the
//   effect is optional ("you may") which is not expressible. Wrong multiplayer behavior
//   if implemented with EachOpponent.
use crate::cards::helpers::*;

pub fn card() -> CardDefinition {
    CardDefinition {
        card_id: cid("blood-seeker"),
        name: "Blood Seeker".to_string(),
        mana_cost: Some(ManaCost {
            generic: 1,
            black: 1,
            ..Default::default()
        }),
        types: creature_types(&["Vampire", "Shaman"]),
        oracle_text: "Whenever a creature an opponent controls enters, you may have that player \
                      lose 1 life."
            .to_string(),
        power: Some(1),
        toughness: Some(1),
        abilities: vec![],
        completeness: Completeness::inert(
            "Blocked on the optional no-cost trigger 'you may have that player lose 1 life' — no \
             `optional` field on AbilityDefinition::Triggered and no MayDo/OptionalEffect \
             wrapper; Effect::Choose is a gated stub (always executes choices[0]). Modelling it \
             as a mandatory LoseLife is wrong game state per W6 (declining is a legal, sometimes \
             relevant choice — e.g. an opponent with a life-loss payoff). The rest IS expressible \
             today (WheneverCreatureEntersBattlefield{filter: controller=Opponent} + \
             LoseLife{ControllerOf(TriggeringCreature), 1}); unblock with an \
             optional-triggered-effect primitive (shared with the interactive-choice cohort — see \
             W-EMPTY roster).",
        ),
        ..Default::default()
    }
}
