// Legolas's Quick Reflexes — {G}, Instant
// Split second; untap target creature, grant hexproof + reach + temporary tap trigger
// TODO: DSL gap — untap + grant hexproof/reach + temporary "whenever tapped" triggered ability
// not expressible in the DSL. `LayerModification`'s variants include AddKeyword and
// AddActivatedAbility but no AddTriggeredAbility, so the granted "whenever this creature
// becomes tapped" clause has no expression. Split second alone is deliberately NOT declared:
// a castable do-nothing is worse than an uncastable card (W5 policy, consolidated-fix-list M2).
//
// CARDS-2 (scutemob-181): this card was defined TWICE — this file and a
// `legolasquick_reflexes.rs` carrying card_id "legolasquick-reflexes". `CardRegistry::try_new`
// rejects a duplicate CardId but says nothing about a duplicate *name*, so both shipped, and
// the W5 fix above reached only one of them. The marker sweep spotted it on 2026-07-16
// (`memory/card-authoring/marker-sweep-2026-07-16.md:582-583`, "one of the two should be
// deleted") and nothing happened for seventeen days, because no gate could fail. The twin is
// deleted; `core::cards2_printed_field_fidelity::r5_no_two_definitions_share_a_name` is the
// gate that now makes the finding un-ignorable. This file is the survivor because its
// card_id is the one `test-data/test-cards/edhrec_all_commanders.json` records.
use crate::cards::helpers::*;

pub fn card() -> CardDefinition {
    CardDefinition {
        card_id: cid("legolass-quick-reflexes"),
        name: "Legolas's Quick Reflexes".to_string(),
        mana_cost: Some(ManaCost {
            green: 1,
            ..Default::default()
        }),
        types: types(&[CardType::Instant]),
        oracle_text: "Split second (As long as this spell is on the stack, players can't cast \
                      spells or activate abilities that aren't mana abilities.)\nUntap target \
                      creature. Until end of turn, it gains hexproof, reach, and \"Whenever this \
                      creature becomes tapped, it deals damage equal to its power to up to one \
                      target creature.\""
            .to_string(),
        abilities: vec![
            // TODO: DSL gap — Split second is a keyword but the card's core effect
            // (untap + grant hexproof/reach until EOT + temporary tap trigger) is not
            // expressible. Card left uncastable per W5 policy to avoid do-nothing behavior.
        ],
        completeness: Completeness::inert(
            "DSL gap — untap + grant hexproof/reach + temporary 'whenever tapped' triggered \
             ability not expressible in the DSL. Only...",
        ),
        ..Default::default()
    }
}
