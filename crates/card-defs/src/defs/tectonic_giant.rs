// Tectonic Giant — {2}{R}{R}, Creature — Elemental Giant 3/4
// Whenever this creature attacks or becomes the target of a spell an opponent controls, choose one —
// • This creature deals 3 damage to each opponent.
// • Exile the top two cards of your library. Choose one of them. Until the end of your
//   next turn, you may play that card.
use crate::cards::helpers::*;

pub fn card() -> CardDefinition {
    CardDefinition {
        card_id: cid("tectonic-giant"),
        name: "Tectonic Giant".to_string(),
        mana_cost: Some(ManaCost {
            generic: 2,
            red: 2,
            ..Default::default()
        }),
        types: creature_types(&["Elemental", "Giant"]),
        oracle_text: "Whenever this creature attacks or becomes the target of a spell an opponent \
                      controls, choose one —\n• This creature deals 3 damage to each opponent.\n• \
                      Exile the top two cards of your library. Choose one of them. Until the end \
                      of your next turn, you may play that card."
            .to_string(),
        power: Some(3),
        toughness: Some(4),
        abilities: vec![
            // ENGINE-BLOCKED: mode 1 ("Exile the top two cards of your library. Choose one of
            // them. Until the end of your next turn, you may play that card.") has no DSL
            // representation — there is no exile-top-N / choose-one / impulse-play effect.
            //
            // The rest of this ability IS expressible: modal triggered abilities resolve
            // (CR 700.2b, resolution.rs handles `AbilityDefinition::Triggered { modes, .. }`),
            // mode 0 works, and PB-AC6 supplies the becomes-target half of the dual trigger as
            // TriggerCondition::WhenBecomesTarget { scope: None, by_opponent: true,
            // include_abilities: false }.
            //
            // It is left UNAUTHORED anyway, deliberately. Authoring it two ways both produce
            // wrong game state: with `Effect::Nothing` as mode 1, a player who chooses mode 1
            // gets nothing (this is what the code did before — a live defect); with mode 1
            // dropped, the player is forced into mode 0 and loses a choice the oracle grants.
            // A partial modal ability is worse than an absent one (W6 policy).
        ],
        completeness: Completeness::partial(
            "mode 1 ('Exile the top two cards of your library. Choose one of them. Until the end \
             of your next turn, you may play...",
        ),
        ..Default::default()
    }
}
