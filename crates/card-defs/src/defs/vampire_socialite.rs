// Vampire Socialite — {B}{R}, Creature — Vampire Noble 2/2
// Menace
// When this creature enters, if an opponent lost life this turn, put a +1/+1 counter on
// each other Vampire you control.
// As long as an opponent lost life this turn, each other Vampire you control enters with
// an additional +1/+1 counter on it.
use crate::cards::helpers::*;

pub fn card() -> CardDefinition {
    CardDefinition {
        card_id: cid("vampire-socialite"),
        name: "Vampire Socialite".to_string(),
        mana_cost: Some(ManaCost {
            black: 1,
            red: 1,
            ..Default::default()
        }),
        types: creature_types(&["Vampire", "Noble"]),
        oracle_text: "Menace\nWhen this creature enters, if an opponent lost life this turn, put \
                      a +1/+1 counter on each other Vampire you control.\nAs long as an opponent \
                      lost life this turn, each other Vampire you control enters with an \
                      additional +1/+1 counter on it."
            .to_string(),
        power: Some(2),
        toughness: Some(2),
        abilities: vec![
            AbilityDefinition::Keyword(KeywordAbility::Menace),
            // TODO: DSL gap — intervening-if "if an opponent lost life this turn"
            // (Condition::OpponentLostLifeThisTurn) does not exist.
            // TODO: DSL gap — replacement effect for "enters with an additional +1/+1 counter"
            // conditional on opponent life loss. Needs conditional ETB replacement.
        ],
        // PB-DX3b (OOS-DX3-1, 2026-08-01): RE-VERIFIED, still blocked, re-dated. Checked
        // against the current Condition enum (`card-types/src/cards/card_definition.rs`)
        // and `AbilityDefinition::Replacement` (`unless_condition: Option<Condition>` is
        // an opt-OUT gate, not an "active only if" gate — the wrong polarity for "as long
        // as an opponent lost life this turn" even setting the missing variant aside).
        // `Condition::OpponentLostLifeThisTurn` still does not exist — the nearest sibling,
        // `ControllerGainedLifeThisTurn`, is the wrong side of the interaction (own life
        // gain, not opponent life loss). Both TODOs stand; DEFERRED, not fixed this batch.
        completeness: Completeness::partial(
            "DSL gap (a) — intervening-if 'if an opponent lost life this turn' \
             (Condition::OpponentLostLifeThisTurn) does not exist. DSL gap (b) — the printed \
             static ability 'as long as an opponent lost life this turn, each other Vampire you \
             control enters with an additional +1/+1 counter' needs a CONDITIONAL ETB replacement \
             effect; AbilityDefinition::Replacement's unless_condition is an opt-OUT gate (active \
             unless the condition holds), the wrong polarity for an 'active only if' gate, so it \
             cannot express this even setting the missing Condition variant aside. Neither (a) \
             nor (b) is implemented; this def is unauthored.",
        ),
        ..Default::default()
    }
}
