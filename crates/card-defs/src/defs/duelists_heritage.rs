// Duelist's Heritage — {2}{W}, Enchantment
// TODO: DSL gap — triggered ability "Whenever one or more creatures attack, you may have
//   target attacking creature gain double strike until end of turn."
//   (targeted trigger that grants a keyword until end of turn not supported; requires
//   ApplyContinuousEffect with EffectDuration::UntilEndOfTurn targeting a declared attacker)
use crate::cards::helpers::*;

pub fn card() -> CardDefinition {
    CardDefinition {
        card_id: cid("duelists-heritage"),
        name: "Duelist's Heritage".to_string(),
        mana_cost: Some(ManaCost {
            generic: 2,
            white: 1,
            ..Default::default()
        }),
        types: types(&[CardType::Enchantment]),
        oracle_text: "Whenever one or more creatures attack, you may have target attacking creature gain double strike until end of turn.".to_string(),
        abilities: vec![],
        completeness: Completeness::inert("Blocked on a global attack trigger: 'Whenever one or more creatures attack' fires on ANY player's declare-attackers, and TriggerEvent has only controller-scoped variants (ControllerAttacks, AnyCreatureYouControlAttacks, ControllerCreatureAttacksAlone). Using a controller-scoped trigger would silently drop the ability on opponents' turns (W5: wrong game state). The effect body IS expressible today (Triggered.targets + TargetCreatureWithFilter{is_attacking} + ApplyContinuousEffect{AddKeyword(DoubleStrike), UntilEndOfTurn}); the 'you may' clause has no `optional` field on Triggered."),
        ..Default::default()
    }
}
