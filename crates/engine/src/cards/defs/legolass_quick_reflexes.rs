// Legolas's Quick Reflexes — {G}, Instant
// Split second; untap target creature, grant hexproof + reach + temporary tap trigger
// TODO: DSL gap — untap + grant hexproof/reach + temporary "whenever tapped" triggered ability
// not expressible in the DSL. Only Split second keyword implemented.
use crate::cards::helpers::*;

pub fn card() -> CardDefinition {
    CardDefinition {
        card_id: cid("legolass-quick-reflexes"),
        name: "Legolas's Quick Reflexes".to_string(),
        mana_cost: Some(ManaCost { green: 1, ..Default::default() }),
        types: types(&[CardType::Instant]),
        oracle_text: "Split second (As long as this spell is on the stack, players can't cast spells or activate abilities that aren't mana abilities.)\nUntap target creature. Until end of turn, it gains hexproof, reach, and \"Whenever this creature becomes tapped, it deals damage equal to its power to up to one target creature.\"".to_string(),
        abilities: vec![
            AbilityDefinition::Keyword(KeywordAbility::SplitSecond),
            // TODO: DSL gap — untap effect, grant hexproof/reach until EOT, and temporary
            // "whenever this creature becomes tapped" triggered ability are not expressible.
        ],
        ..Default::default()
    }
}
