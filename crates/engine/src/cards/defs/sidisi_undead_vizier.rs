// Sidisi, Undead Vizier — {3}{B}{B}, Legendary Creature — Zombie Snake 4/6
// Deathtouch
// Exploit (When this creature enters, you may sacrifice a creature.)
// When Sidisi exploits a creature, you may search your library for a card,
// put it into your hand, then shuffle.
// TODO: TriggerCondition::WhenThisExploitsACreature does not exist in the DSL.
// The "when this creature exploits a creature" trigger (fired after exploit sacrifice
// resolves) cannot be expressed — no exploit-specific trigger condition exists.
// The Exploit keyword marker is implemented; the tutor trigger is deferred per W5 policy.
use crate::cards::helpers::*;

pub fn card() -> CardDefinition {
    CardDefinition {
        card_id: cid("sidisi-undead-vizier"),
        name: "Sidisi, Undead Vizier".to_string(),
        mana_cost: Some(ManaCost { generic: 3, black: 2, ..Default::default() }),
        types: full_types(
            &[SuperType::Legendary],
            &[CardType::Creature],
            &["Zombie", "Snake"],
        ),
        oracle_text:
            "Deathtouch\n\
             Exploit (When this creature enters, you may sacrifice a creature.)\n\
             When Sidisi exploits a creature, you may search your library for a card, \
             put it into your hand, then shuffle."
                .to_string(),
        power: Some(4),
        toughness: Some(6),
        abilities: vec![
            AbilityDefinition::Keyword(KeywordAbility::Deathtouch),
            AbilityDefinition::Keyword(KeywordAbility::Exploit),
            // TODO: "When this creature exploits a creature" — no TriggerCondition variant for
            // exploit-specific triggers. The search-library effect tied to exploit sacrifice
            // cannot be expressed in the DSL.
        ],
        ..Default::default()
    }
}
