// Legolas's Quick Reflexes — {G}, Instant
// Split second
// Untap target creature. Until end of turn, it gains hexproof, reach, and
// "Whenever this creature becomes tapped, it deals damage equal to its power
// to up to one target creature."
//
// Split second is implemented as a keyword.
// TODO: DSL gap — the main spell effect (untap + grant hexproof/reach + tap
// trigger) is a complex instant-speed effect with multiple parts including a
// temporary "whenever this creature becomes tapped" triggered ability. The
// DSL has no way to grant temporary triggered abilities to a target.
// Abilities are omitted.
use crate::cards::helpers::*;

pub fn card() -> CardDefinition {
    CardDefinition {
        card_id: cid("legolasquick-reflexes"),
        name: "Legolas's Quick Reflexes".to_string(),
        mana_cost: Some(ManaCost { green: 1, ..Default::default() }),
        types: types(&[CardType::Instant]),
        oracle_text: "Split second (As long as this spell is on the stack, players can't cast spells or activate abilities that aren't mana abilities.)\nUntap target creature. Until end of turn, it gains hexproof, reach, and \"Whenever this creature becomes tapped, it deals damage equal to its power to up to one target creature.\"".to_string(),
        abilities: vec![
            AbilityDefinition::Keyword(KeywordAbility::SplitSecond),
            // TODO: DSL gap — untap + grant hexproof/reach + temporary tapped trigger not expressible.
        ],
        ..Default::default()
    }
}
