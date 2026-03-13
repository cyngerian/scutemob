// Nezumi Prowler — {1}{B}, Artifact Creature — Rat Ninja 3/1
// Ninjutsu {1}{B}
// When this creature enters, target creature you control gains deathtouch and lifelink
// until end of turn.
//
// TODO: DSL gap — ETB pump omitted.
// "When this creature enters, target creature you control gains deathtouch and lifelink
// until end of turn." Requires a targeted ETB triggered ability granting two keywords
// (Deathtouch + Lifelink) to a creature you control until end of turn. DSL gap:
// no activated_ability_targets field on triggered abilities; granting multiple keywords
// as a continuous effect to a declared target is not currently expressible.
use crate::cards::helpers::*;

pub fn card() -> CardDefinition {
    CardDefinition {
        card_id: cid("nezumi-prowler"),
        name: "Nezumi Prowler".to_string(),
        mana_cost: Some(ManaCost { generic: 1, black: 1, ..Default::default() }),
        types: types_sub(&[CardType::Artifact, CardType::Creature], &["Rat", "Ninja"]),
        oracle_text: "Ninjutsu {1}{B} ({1}{B}, Return an unblocked attacker you control to hand: Put this card onto the battlefield from your hand tapped and attacking.)\nWhen this creature enters, target creature you control gains deathtouch and lifelink until end of turn.".to_string(),
        power: Some(3),
        toughness: Some(1),
        abilities: vec![
            AbilityDefinition::Keyword(KeywordAbility::Ninjutsu),
            AbilityDefinition::Ninjutsu {
                cost: ManaCost { generic: 1, black: 1, ..Default::default() },
            },
        ],
        ..Default::default()
    }
}
