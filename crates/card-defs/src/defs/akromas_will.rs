// Akroma's Will — {3}{W}, Instant; modal spell with commander bonus.
// TODO: DSL gap — conditional modal choice ("if you control a commander, may choose both"),
// mass flying/vigilance/double strike grant, mass lifelink/indestructible/protection grant
// not expressible.
use crate::cards::helpers::*;

pub fn card() -> CardDefinition {
    CardDefinition {
        card_id: cid("akromas-will"),
        name: "Akroma's Will".to_string(),
        mana_cost: Some(ManaCost {
            generic: 3,
            white: 1,
            ..Default::default()
        }),
        types: types(&[CardType::Instant]),
        oracle_text: "Choose one. If you control a commander as you cast this spell, you may \
                      choose both instead.\n\u{2022} Creatures you control gain flying, \
                      vigilance, and double strike until end of turn.\n\u{2022} Creatures you \
                      control gain lifelink, indestructible, and protection from each color until \
                      end of turn."
            .to_string(),
        abilities: vec![],
        // TODO: conditional modal bonus; mass keyword grants; protection from each color
        completeness: Completeness::inert(
            "Blocked on commander-gated modal widening: ModeSelection has fixed \
             min_modes/max_modes and cannot express 'if you control a commander as you cast this \
             spell, you may choose both instead' (CR 700.2). The two mode bodies themselves are \
             expressible (ApplyContinuousEffect + AddKeywords over CreaturesYouControl, \
             UntilEndOfTurn); 'protection from each color' needs a protection-quality grant \
             check. Unblock with a condition-gated max_modes on ModeSelection. Shares this \
             blocker with jeskas_will.rs.",
        ),
        ..Default::default()
    }
}
