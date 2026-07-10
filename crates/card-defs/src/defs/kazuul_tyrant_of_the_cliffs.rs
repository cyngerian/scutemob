// Kazuul, Tyrant of the Cliffs — {3}{R}{R}, Legendary Creature — Ogre Warrior 5/4
// Whenever a creature an opponent controls attacks, if you're the defending player, create
// a 3/3 red Ogre creature token unless that creature's controller pays {3}.
use crate::cards::helpers::*;

pub fn card() -> CardDefinition {
    CardDefinition {
        card_id: cid("kazuul-tyrant-of-the-cliffs"),
        name: "Kazuul, Tyrant of the Cliffs".to_string(),
        mana_cost: Some(ManaCost { generic: 3, red: 2, ..Default::default() }),
        types: full_types(&[SuperType::Legendary], &[CardType::Creature], &["Ogre", "Warrior"]),
        oracle_text: "Whenever a creature an opponent controls attacks, if you're the defending player, create a 3/3 red Ogre creature token unless that creature's controller pays {3}.".to_string(),
        power: Some(5),
        toughness: Some(4),
        abilities: vec![
            // CR 508.1m / CR 603.2: "Whenever a creature an opponent controls attacks, if
            // you're the defending player, create a 3/3 red Ogre token unless that creature's
            // controller pays {3}."
            // TODO: ENGINE-BLOCKED. Two genuine gaps:
            // 1. No TriggerCondition for "whenever a creature an opponent controls attacks."
            //    WheneverCreatureYouControlAttacks only fires for your own attackers, and
            //    WheneverYouAttack is once-per-combat for the active player. Neither fires on
            //    an opponent's attacker. The "if you're the defending player" intervening-if
            //    also needs a per-defending-player check.
            // 2. The "unless that creature's controller pays {3}" clause: Effect::MayPayOrElse
            //    { cost, payer: PlayerTarget, or_else } exists (card_definition.rs:1584), but
            //    PlayerTarget has no variant resolving to "the controller of the triggering
            //    attacking creature" — the payer cannot be addressed. The or_else (create a
            //    3/3 red Ogre) is itself expressible once the payer can be resolved.
        ],
        completeness: Completeness::partial("ENGINE-BLOCKED. Two genuine gaps: 1. No TriggerCondition for 'whenever a creature an opponent controls attacks.'..."),
        ..Default::default()
    }
}
