// 68. Signal Pest — {1}, Artifact Creature — Pest 0/1;
// Battle cry (CR 702.91). Blocking restriction (flying/reach only) deferred — no DSL variant.
use crate::cards::helpers::*;

pub fn card() -> CardDefinition {
    CardDefinition {
        card_id: cid("signal-pest"),
        name: "Signal Pest".to_string(),
        mana_cost: Some(ManaCost { generic: 1, ..Default::default() }),
        types: types_sub(&[CardType::Artifact, CardType::Creature], &["Pest"]),
        oracle_text: "Battle cry (Whenever this creature attacks, each other attacking creature gets +1/+0 until end of turn.)\nThis creature can't be blocked except by creatures with flying or reach."
            .to_string(),
        power: Some(0),
        toughness: Some(1),
        abilities: vec![
            AbilityDefinition::Keyword(KeywordAbility::BattleCry),
            // TODO: blocking restriction ("can't be blocked except by flying/reach") deferred
        ],
        color_indicator: None,
        back_face: None,
        spell_cost_modifiers: vec![],
        self_cost_reduction: None,
        starting_loyalty: None,
        adventure_face: None,
        meld_pair: None,
        activated_ability_cost_reductions: vec![],
    }
}
