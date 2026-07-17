// Esper Sentinel — {W}, Artifact Creature — Human Soldier 1/1
// Whenever an opponent casts their first noncreature spell each turn, draw a card
// unless that player pays {X}, where X is Esper Sentinel's power.
//
// TODO: Opponent-cast trigger with noncreature filter, once-per-turn,
//   and conditional pay-or-draw. Not expressible in current DSL.
use crate::cards::helpers::*;

pub fn card() -> CardDefinition {
    CardDefinition {
        card_id: cid("esper-sentinel"),
        name: "Esper Sentinel".to_string(),
        mana_cost: Some(ManaCost { white: 1, ..Default::default() }),
        types: full_types(
            &[],
            &[CardType::Artifact, CardType::Creature],
            &["Human", "Soldier"],
        ),
        oracle_text: "Whenever an opponent casts their first noncreature spell each turn, draw a card unless that player pays {X}, where X is Esper Sentinel's power.".to_string(),
        power: Some(1),
        toughness: Some(1),
        // TODO: Opponent-cast trigger with noncreature filter, once-per-turn,
        //   and conditional pay-or-draw. Not expressible in current DSL.
        abilities: vec![],
        completeness: Completeness::inert("Blocked on two items: (1) dynamic cost — 'unless that player pays {X}, where X is this creature's power' needs an EffectAmount-valued cost; Effect::MayPayOrElse takes only a static Cost::Mana. (2) per-opponent 'first noncreature spell each turn' tracking — Triggered.once_per_turn is a global once-per-turn cap and would be WRONG in multiplayer (this should trigger once per opponent per turn). The noncreature filter is NOT a blocker: WheneverOpponentCastsSpell{noncreature_only: true} exists."),
        ..Default::default()
    }
}
