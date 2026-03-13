// Zealous Conscripts — {4}{R}, Creature — Human Warrior 3/3
// Haste; ETB: gain control of target permanent until end of turn, untap it, give it haste
// TODO: ETB gain-control targeted trigger not in DSL (targeted_trigger gap)
use crate::cards::helpers::*;

pub fn card() -> CardDefinition {
    CardDefinition {
        card_id: cid("zealous-conscripts"),
        name: "Zealous Conscripts".to_string(),
        mana_cost: Some(ManaCost {
            generic: 4,
            red: 1,
            ..Default::default()
        }),
        types: creature_types(&["Human", "Warrior"]),
        oracle_text: "Haste\nWhen this creature enters, gain control of target permanent until end of turn. Untap that permanent. It gains haste until end of turn.".to_string(),
        power: Some(3),
        toughness: Some(3),
        abilities: vec![
            AbilityDefinition::Keyword(KeywordAbility::Haste),
            // TODO: ETB triggered ability targets a permanent, grants temporary control +
            // untap + haste until end of turn — requires targeted_trigger which is not
            // currently in the DSL.
        ],
        ..Default::default()
    }
}
