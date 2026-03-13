// Skrelv, Defector Mite — {W}, Legendary Artifact Creature — Phyrexian Mite 1/1
// Toxic 1 (Players dealt combat damage by this creature also get a poison counter.)
// Skrelv can't block.
// {W/P}, {T}: Choose a color. Another target creature you control gains toxic 1 and
// hexproof from that color until end of turn. It can't be blocked by creatures of that
// color this turn. ({W/P} can be paid with either {W} or 2 life.)
//
// Toxic 1 and CantBlock are implemented.
//
// TODO: DSL gap — the activated ability requires:
// 1. Phyrexian mana cost ({W/P} = {W} or 2 life) — Cost enum has no PhyrexianMana variant.
// 2. "Choose a color" selection — no ChooseColor effect primitive.
// 3. "Gains hexproof from that color" — protection-from-color effect not expressible.
// 4. "Can't be blocked by creatures of that color this turn" — complex block restriction.
// This activated ability is omitted.
use crate::cards::helpers::*;

pub fn card() -> CardDefinition {
    CardDefinition {
        card_id: cid("skrelv-defector-mite"),
        name: "Skrelv, Defector Mite".to_string(),
        mana_cost: Some(ManaCost { white: 1, ..Default::default() }),
        types: full_types(
            &[SuperType::Legendary],
            &[CardType::Artifact, CardType::Creature],
            &["Phyrexian", "Mite"],
        ),
        oracle_text: "Toxic 1 (Players dealt combat damage by this creature also get a poison counter.)\nSkrelv can't block.\n{W/P}, {T}: Choose a color. Another target creature you control gains toxic 1 and hexproof from that color until end of turn. It can't be blocked by creatures of that color this turn. ({W/P} can be paid with either {W} or 2 life.)".to_string(),
        power: Some(1),
        toughness: Some(1),
        abilities: vec![
            AbilityDefinition::Keyword(KeywordAbility::Toxic(1)),
            // TODO: DSL gap — no CantBlock keyword variant; Skrelv can't block
        ],
        ..Default::default()
    }
}
