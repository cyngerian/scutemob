// Skyhunter Strike Force — {2}{W}, Creature — Cat Knight 2/2
// Flying
// Melee (Whenever this creature attacks, it gets +1/+1 until end of turn for each
// opponent you attacked this combat.)
// Lieutenant — As long as you control your commander, other creatures you control
// have melee.
//
// ENGINE-BLOCKED (Lieutenant grant): "As long as you control your commander, other
// creatures you control have melee" needs a "you control your commander" condition
// on a continuous-effect grant. No `Condition` variant nor `TargetFilter` field
// expresses "is commander" (card_definition.rs Condition/TargetFilter enums audited,
// PB-EF3b recon) — `ContinuousEffectDef.condition` has nowhere to hang this check.
// Flying + printed Melee are modeled and correct; the Lieutenant anthem is omitted
// (not wrong game state — see Completeness note). Blocker filed as OOS-EF3b-1.
use crate::cards::helpers::*;

pub fn card() -> CardDefinition {
    CardDefinition {
        card_id: cid("skyhunter-strike-force"),
        name: "Skyhunter Strike Force".to_string(),
        mana_cost: Some(ManaCost {
            generic: 2,
            white: 1,
            ..Default::default()
        }),
        types: creature_types(&["Cat", "Knight"]),
        oracle_text: "Flying\nMelee (Whenever this creature attacks, it gets +1/+1 until end of \
                      turn for each opponent you attacked this combat.)\nLieutenant — As long as \
                      you control your commander, other creatures you control have melee."
            .to_string(),
        power: Some(2),
        toughness: Some(2),
        abilities: vec![
            AbilityDefinition::Keyword(KeywordAbility::Flying),
            // CR 702.121a: printed Melee (self).
            AbilityDefinition::Keyword(KeywordAbility::Melee),
            // Lieutenant clause ENGINE-BLOCKED (see top-of-file comment) — omitted,
            // not modeled wrong.
        ],
        completeness: Completeness::partial(
            "Lieutenant clause 'As long as you control your commander, other creatures you \
             control have melee' is unrepresentable: no Condition variant nor TargetFilter field \
             expresses 'you control your commander' for a continuous-effect grant \
             (ContinuousEffectDef.condition). Flying + printed Melee are modeled and correct; the \
             anthem is omitted (not wrong game state). Blocker filed as OOS-EF3b-1.",
        ),
        ..Default::default()
    }
}
