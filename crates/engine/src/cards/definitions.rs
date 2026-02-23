//! Hand-authored card definitions (51 cards as of M8).
//!
//! These are Commander staples covering a range of effects that the engine
//! can execute. Each definition encodes the card's behavior using the Effect DSL
//! from `card_definition.rs`.
//!
//! Card IDs use lowercase kebab-case of the English card name.
//!
//! Organisation:
//!   - Mana rocks & ramp (Sol Ring, Arcane Signet, …)
//!   - Lands (Command Tower, Evolving Wilds, …)
//!   - Removal — targeted (Swords to Plowshares, Path to Exile, …)
//!   - Removal — mass (Wrath of God, Damnation, …)
//!   - Counterspells (Counterspell, Negate, …)
//!   - Card draw (Harmonize, Night's Whisper, …)
//!   - Ramp spells (Cultivate, Rampant Growth, …)
//!   - Equipment (Lightning Greaves, Swiftfoot Boots)
//!   - Utility creatures (Llanowar Elves, Solemn Simulacrum, …)

use im::OrdSet;

use crate::state::player::PlayerId;
use crate::state::{
    CardId, CardType, Color, KeywordAbility, ManaCost, ManaPool, SubType, SuperType,
};

use super::card_definition::{
    AbilityDefinition, CardDefinition, Cost, Effect, EffectAmount, EffectTarget, PlayerTarget,
    TargetFilter, TargetRequirement, TriggerCondition, TypeLine,
};
use crate::state::replacement_effect::{ObjectFilter, ReplacementModification, ReplacementTrigger};
use crate::state::zone::ZoneType;

// ── Helper macros & functions ─────────────────────────────────────────────────

fn cid(s: &str) -> CardId {
    CardId(s.to_string())
}

fn types(card_types: &[CardType]) -> TypeLine {
    TypeLine {
        card_types: card_types.iter().copied().collect(),
        ..Default::default()
    }
}

#[allow(dead_code)]
fn supertypes(supers: &[SuperType], card_types: &[CardType]) -> TypeLine {
    TypeLine {
        supertypes: supers.iter().copied().collect(),
        card_types: card_types.iter().copied().collect(),
        ..Default::default()
    }
}

fn types_sub(card_types: &[CardType], subtypes: &[&str]) -> TypeLine {
    TypeLine {
        card_types: card_types.iter().copied().collect(),
        subtypes: subtypes.iter().map(|s| SubType(s.to_string())).collect(),
        ..Default::default()
    }
}

fn full_types(supers: &[SuperType], card_types: &[CardType], subtypes: &[&str]) -> TypeLine {
    TypeLine {
        supertypes: supers.iter().copied().collect(),
        card_types: card_types.iter().copied().collect(),
        subtypes: subtypes.iter().map(|s| SubType(s.to_string())).collect(),
    }
}

fn creature_types(subtypes: &[&str]) -> TypeLine {
    types_sub(&[CardType::Creature], subtypes)
}

fn mana_pool(c: u32, u: u32, b: u32, r: u32, g: u32, colorless: u32) -> ManaPool {
    ManaPool {
        white: c,
        blue: u,
        black: b,
        red: r,
        green: g,
        colorless,
    }
}

fn basic_land_filter() -> TargetFilter {
    TargetFilter {
        basic: true,
        has_card_type: Some(CardType::Land),
        ..Default::default()
    }
}

// ── Card list ─────────────────────────────────────────────────────────────────

/// Return all hand-authored card definitions.
pub fn all_cards() -> Vec<CardDefinition> {
    vec![
        // ── Mana rocks ──────────────────────────────────────────────────────

        // 1. Sol Ring — {1}, Artifact, tap: add {C}{C}
        CardDefinition {
            card_id: cid("sol-ring"),
            name: "Sol Ring".to_string(),
            mana_cost: Some(ManaCost { generic: 1, ..Default::default() }),
            types: types(&[CardType::Artifact]),
            oracle_text: "{T}: Add {C}{C}.".to_string(),
            abilities: vec![AbilityDefinition::Activated {
                cost: Cost::Tap,
                effect: Effect::AddMana {
                    player: PlayerTarget::Controller,
                    mana: mana_pool(0, 0, 0, 0, 0, 2),
                },
                timing_restriction: None,
            }],
            ..Default::default()
        },

        // 2. Arcane Signet — {2}, Artifact, tap: add one mana of any color in your
        //    commander's color identity. Modelled as AddManaAnyColor (simplified).
        CardDefinition {
            card_id: cid("arcane-signet"),
            name: "Arcane Signet".to_string(),
            mana_cost: Some(ManaCost { generic: 2, ..Default::default() }),
            types: types(&[CardType::Artifact]),
            oracle_text: "{T}: Add one mana of any color in your commander's color identity."
                .to_string(),
            abilities: vec![AbilityDefinition::Activated {
                cost: Cost::Tap,
                effect: Effect::AddManaAnyColor { player: PlayerTarget::Controller },
                timing_restriction: None,
            }],
            ..Default::default()
        },

        // 3. Commander's Sphere — {3}, Artifact, tap: add one mana of any color;
        //    sacrifice: draw a card.
        CardDefinition {
            card_id: cid("commanders-sphere"),
            name: "Commander's Sphere".to_string(),
            mana_cost: Some(ManaCost { generic: 3, ..Default::default() }),
            types: types(&[CardType::Artifact]),
            oracle_text: "{T}: Add one mana of any color in your commander's color identity.\nSacrifice Commander's Sphere: Draw a card.".to_string(),
            abilities: vec![
                AbilityDefinition::Activated {
                    cost: Cost::Tap,
                    effect: Effect::AddManaAnyColor { player: PlayerTarget::Controller },
                    timing_restriction: None,
                },
                AbilityDefinition::Activated {
                    cost: Cost::Sacrifice(TargetFilter::default()),
                    effect: Effect::DrawCards {
                        player: PlayerTarget::Controller,
                        count: EffectAmount::Fixed(1),
                    },
                    timing_restriction: None,
                },
            ],
            ..Default::default()
        },

        // 4. Thought Vessel — {2}, Artifact, tap: add {C}; you have no maximum hand size.
        //    (The no-max-hand-size continuous effect is not yet implemented; tap ability is.)
        CardDefinition {
            card_id: cid("thought-vessel"),
            name: "Thought Vessel".to_string(),
            mana_cost: Some(ManaCost { generic: 2, ..Default::default() }),
            types: types(&[CardType::Artifact]),
            oracle_text: "You have no maximum hand size.\n{T}: Add {C}.".to_string(),
            abilities: vec![AbilityDefinition::Activated {
                cost: Cost::Tap,
                effect: Effect::AddMana {
                    player: PlayerTarget::Controller,
                    mana: mana_pool(0, 0, 0, 0, 0, 1),
                },
                timing_restriction: None,
            }],
            ..Default::default()
        },

        // 5. Mind Stone — {2}, Artifact, tap: add {C}; {1}, tap, sacrifice: draw a card.
        CardDefinition {
            card_id: cid("mind-stone"),
            name: "Mind Stone".to_string(),
            mana_cost: Some(ManaCost { generic: 2, ..Default::default() }),
            types: types(&[CardType::Artifact]),
            oracle_text: "{T}: Add {C}.\n{1}, {T}, Sacrifice Mind Stone: Draw a card.".to_string(),
            abilities: vec![
                AbilityDefinition::Activated {
                    cost: Cost::Tap,
                    effect: Effect::AddMana {
                        player: PlayerTarget::Controller,
                        mana: mana_pool(0, 0, 0, 0, 0, 1),
                    },
                    timing_restriction: None,
                },
                AbilityDefinition::Activated {
                    cost: Cost::Sequence(vec![
                        Cost::Mana(ManaCost { generic: 1, ..Default::default() }),
                        Cost::Tap,
                        Cost::Sacrifice(TargetFilter::default()),
                    ]),
                    effect: Effect::DrawCards {
                        player: PlayerTarget::Controller,
                        count: EffectAmount::Fixed(1),
                    },
                    timing_restriction: None,
                },
            ],
            ..Default::default()
        },

        // 6. Darksteel Ingot — {3}, Artifact (Indestructible), tap: add one mana of any color.
        CardDefinition {
            card_id: cid("darksteel-ingot"),
            name: "Darksteel Ingot".to_string(),
            mana_cost: Some(ManaCost { generic: 3, ..Default::default() }),
            types: types(&[CardType::Artifact]),
            oracle_text: "Indestructible\n{T}: Add one mana of any color.".to_string(),
            abilities: vec![
                AbilityDefinition::Keyword(KeywordAbility::Indestructible),
                AbilityDefinition::Activated {
                    cost: Cost::Tap,
                    effect: Effect::AddManaAnyColor { player: PlayerTarget::Controller },
                    timing_restriction: None,
                },
            ],
            ..Default::default()
        },

        // 7. Wayfarer's Bauble — {1}, Artifact, {2}, tap, sacrifice: search your library for
        //    a basic land, put it onto the battlefield tapped, then shuffle.
        CardDefinition {
            card_id: cid("wayfarers-bauble"),
            name: "Wayfarer's Bauble".to_string(),
            mana_cost: Some(ManaCost { generic: 1, ..Default::default() }),
            types: types(&[CardType::Artifact]),
            oracle_text: "{2}, {T}, Sacrifice Wayfarer's Bauble: Search your library for a basic land card and put it onto the battlefield tapped. Then shuffle.".to_string(),
            abilities: vec![AbilityDefinition::Activated {
                cost: Cost::Sequence(vec![
                    Cost::Mana(ManaCost { generic: 2, ..Default::default() }),
                    Cost::Tap,
                    Cost::Sacrifice(TargetFilter::default()),
                ]),
                effect: Effect::Sequence(vec![
                    Effect::SearchLibrary {
                        player: PlayerTarget::Controller,
                        filter: basic_land_filter(),
                        reveal: false,
                        destination: super::card_definition::ZoneTarget::Battlefield { tapped: true },
                    },
                    Effect::Shuffle { player: PlayerTarget::Controller },
                ]),
                timing_restriction: None,
            }],
            ..Default::default()
        },

        // 8. Hedron Archive — {4}, Artifact, tap: add {C}{C};
        //    {2}, tap, sacrifice: draw 2 cards.
        CardDefinition {
            card_id: cid("hedron-archive"),
            name: "Hedron Archive".to_string(),
            mana_cost: Some(ManaCost { generic: 4, ..Default::default() }),
            types: types(&[CardType::Artifact]),
            oracle_text: "{T}: Add {C}{C}.\n{2}, {T}, Sacrifice Hedron Archive: Draw two cards.".to_string(),
            abilities: vec![
                AbilityDefinition::Activated {
                    cost: Cost::Tap,
                    effect: Effect::AddMana {
                        player: PlayerTarget::Controller,
                        mana: mana_pool(0, 0, 0, 0, 0, 2),
                    },
                    timing_restriction: None,
                },
                AbilityDefinition::Activated {
                    cost: Cost::Sequence(vec![
                        Cost::Mana(ManaCost { generic: 2, ..Default::default() }),
                        Cost::Tap,
                        Cost::Sacrifice(TargetFilter::default()),
                    ]),
                    effect: Effect::DrawCards {
                        player: PlayerTarget::Controller,
                        count: EffectAmount::Fixed(2),
                    },
                    timing_restriction: None,
                },
            ],
            ..Default::default()
        },

        // ── Lands ───────────────────────────────────────────────────────────

        // 9. Command Tower — Land, tap: add one mana of any color in your commander's
        //    color identity.
        CardDefinition {
            card_id: cid("command-tower"),
            name: "Command Tower".to_string(),
            mana_cost: None,
            types: types(&[CardType::Land]),
            oracle_text: "{T}: Add one mana of any color in your commander's color identity.".to_string(),
            abilities: vec![AbilityDefinition::Activated {
                cost: Cost::Tap,
                effect: Effect::AddManaAnyColor { player: PlayerTarget::Controller },
                timing_restriction: None,
            }],
            ..Default::default()
        },

        // 10. Evolving Wilds — Land, {T}, sacrifice: search library for a basic land,
        //     put it onto battlefield tapped, then shuffle.
        CardDefinition {
            card_id: cid("evolving-wilds"),
            name: "Evolving Wilds".to_string(),
            mana_cost: None,
            types: types(&[CardType::Land]),
            oracle_text: "{T}, Sacrifice Evolving Wilds: Search your library for a basic land card and put it onto the battlefield tapped. Then shuffle.".to_string(),
            abilities: vec![AbilityDefinition::Activated {
                cost: Cost::Sequence(vec![
                    Cost::Tap,
                    Cost::Sacrifice(TargetFilter::default()),
                ]),
                effect: Effect::Sequence(vec![
                    Effect::SearchLibrary {
                        player: PlayerTarget::Controller,
                        filter: basic_land_filter(),
                        reveal: false,
                        destination: super::card_definition::ZoneTarget::Battlefield { tapped: true },
                    },
                    Effect::Shuffle { player: PlayerTarget::Controller },
                ]),
                timing_restriction: None,
            }],
            ..Default::default()
        },

        // 11. Terramorphic Expanse — same as Evolving Wilds.
        CardDefinition {
            card_id: cid("terramorphic-expanse"),
            name: "Terramorphic Expanse".to_string(),
            mana_cost: None,
            types: types(&[CardType::Land]),
            oracle_text: "{T}, Sacrifice Terramorphic Expanse: Search your library for a basic land card and put it onto the battlefield tapped. Then shuffle.".to_string(),
            abilities: vec![AbilityDefinition::Activated {
                cost: Cost::Sequence(vec![
                    Cost::Tap,
                    Cost::Sacrifice(TargetFilter::default()),
                ]),
                effect: Effect::Sequence(vec![
                    Effect::SearchLibrary {
                        player: PlayerTarget::Controller,
                        filter: basic_land_filter(),
                        reveal: false,
                        destination: super::card_definition::ZoneTarget::Battlefield { tapped: true },
                    },
                    Effect::Shuffle { player: PlayerTarget::Controller },
                ]),
                timing_restriction: None,
            }],
            ..Default::default()
        },

        // 12. Reliquary Tower — Land; tap: add {C}; you have no maximum hand size.
        CardDefinition {
            card_id: cid("reliquary-tower"),
            name: "Reliquary Tower".to_string(),
            mana_cost: None,
            types: types(&[CardType::Land]),
            oracle_text: "You have no maximum hand size.\n{T}: Add {C}.".to_string(),
            abilities: vec![AbilityDefinition::Activated {
                cost: Cost::Tap,
                effect: Effect::AddMana {
                    player: PlayerTarget::Controller,
                    mana: mana_pool(0, 0, 0, 0, 0, 1),
                },
                timing_restriction: None,
            }],
            ..Default::default()
        },

        // 13. Rogue's Passage — Land; {T}: add {C}; {4}, {T}: target creature can't be
        //     blocked this turn. (Unblockable effect not implemented; tap ability only.)
        CardDefinition {
            card_id: cid("rogues-passage"),
            name: "Rogue's Passage".to_string(),
            mana_cost: None,
            types: types(&[CardType::Land]),
            oracle_text: "{T}: Add {C}.\n{4}, {T}: Target creature can't be blocked this turn.".to_string(),
            abilities: vec![AbilityDefinition::Activated {
                cost: Cost::Tap,
                effect: Effect::AddMana {
                    player: PlayerTarget::Controller,
                    mana: mana_pool(0, 0, 0, 0, 0, 1),
                },
                timing_restriction: None,
            }],
            ..Default::default()
        },

        // 14-18: Basic lands (each produces one mana of its color).
        CardDefinition {
            card_id: cid("plains"),
            name: "Plains".to_string(),
            mana_cost: None,
            types: full_types(&[SuperType::Basic], &[CardType::Land], &["Plains"]),
            oracle_text: "{T}: Add {W}.".to_string(),
            abilities: vec![AbilityDefinition::Activated {
                cost: Cost::Tap,
                effect: Effect::AddMana {
                    player: PlayerTarget::Controller,
                    mana: mana_pool(1, 0, 0, 0, 0, 0),
                },
                timing_restriction: None,
            }],
            ..Default::default()
        },
        CardDefinition {
            card_id: cid("island"),
            name: "Island".to_string(),
            mana_cost: None,
            types: full_types(&[SuperType::Basic], &[CardType::Land], &["Island"]),
            oracle_text: "{T}: Add {U}.".to_string(),
            abilities: vec![AbilityDefinition::Activated {
                cost: Cost::Tap,
                effect: Effect::AddMana {
                    player: PlayerTarget::Controller,
                    mana: mana_pool(0, 1, 0, 0, 0, 0),
                },
                timing_restriction: None,
            }],
            ..Default::default()
        },
        CardDefinition {
            card_id: cid("swamp"),
            name: "Swamp".to_string(),
            mana_cost: None,
            types: full_types(&[SuperType::Basic], &[CardType::Land], &["Swamp"]),
            oracle_text: "{T}: Add {B}.".to_string(),
            abilities: vec![AbilityDefinition::Activated {
                cost: Cost::Tap,
                effect: Effect::AddMana {
                    player: PlayerTarget::Controller,
                    mana: mana_pool(0, 0, 1, 0, 0, 0),
                },
                timing_restriction: None,
            }],
            ..Default::default()
        },
        CardDefinition {
            card_id: cid("mountain"),
            name: "Mountain".to_string(),
            mana_cost: None,
            types: full_types(&[SuperType::Basic], &[CardType::Land], &["Mountain"]),
            oracle_text: "{T}: Add {R}.".to_string(),
            abilities: vec![AbilityDefinition::Activated {
                cost: Cost::Tap,
                effect: Effect::AddMana {
                    player: PlayerTarget::Controller,
                    mana: mana_pool(0, 0, 0, 1, 0, 0),
                },
                timing_restriction: None,
            }],
            ..Default::default()
        },
        CardDefinition {
            card_id: cid("forest"),
            name: "Forest".to_string(),
            mana_cost: None,
            types: full_types(&[SuperType::Basic], &[CardType::Land], &["Forest"]),
            oracle_text: "{T}: Add {G}.".to_string(),
            abilities: vec![AbilityDefinition::Activated {
                cost: Cost::Tap,
                effect: Effect::AddMana {
                    player: PlayerTarget::Controller,
                    mana: mana_pool(0, 0, 0, 0, 1, 0),
                },
                timing_restriction: None,
            }],
            ..Default::default()
        },

        // ── ETB-tapped dual lands (M8 Session 4) ────────────────────────────

        // 19a. Dimir Guildgate — Land — Gate; enters the battlefield tapped.
        //      {T}: Add {U} or {B}.
        CardDefinition {
            card_id: cid("dimir-guildgate"),
            name: "Dimir Guildgate".to_string(),
            mana_cost: None,
            types: types_sub(&[CardType::Land], &["Gate"]),
            oracle_text: "Dimir Guildgate enters the battlefield tapped.\n{T}: Add {U} or {B}."
                .to_string(),
            abilities: vec![
                // CR 614.1c: self-replacement effect — this permanent enters tapped.
                AbilityDefinition::Replacement {
                    trigger: ReplacementTrigger::WouldEnterBattlefield {
                        filter: ObjectFilter::Any,
                    },
                    modification: ReplacementModification::EntersTapped,
                    is_self: true,
                },
                // {T}: Add {U} or {B} (simplified as colorless for M8; full modal
                // color choice is M9+ interactive).
                AbilityDefinition::Activated {
                    cost: Cost::Tap,
                    effect: Effect::AddManaAnyColor {
                        player: PlayerTarget::Controller,
                    },
                    timing_restriction: None,
                },
            ],
            ..Default::default()
        },

        // ── Targeted removal ─────────────────────────────────────────────────

        // 20. Swords to Plowshares — {W}, Instant
        CardDefinition {
            card_id: cid("swords-to-plowshares"),
            name: "Swords to Plowshares".to_string(),
            mana_cost: Some(ManaCost { white: 1, ..Default::default() }),
            types: types(&[CardType::Instant]),
            oracle_text: "Exile target creature. Its controller gains life equal to its power."
                .to_string(),
            abilities: vec![AbilityDefinition::Spell {
                effect: Effect::Sequence(vec![
                    Effect::ExileObject {
                        target: EffectTarget::DeclaredTarget { index: 0 },
                    },
                    Effect::GainLife {
                        player: PlayerTarget::ControllerOf(Box::new(
                            EffectTarget::DeclaredTarget { index: 0 },
                        )),
                        amount: EffectAmount::PowerOf(EffectTarget::DeclaredTarget { index: 0 }),
                    },
                ]),
                targets: vec![TargetRequirement::TargetCreature],
                modes: None,
                cant_be_countered: false,
            }],
            ..Default::default()
        },

        // 21. Path to Exile — {W}, Instant; exile target creature, its controller may
        //     search for a basic land and put it into play tapped.
        CardDefinition {
            card_id: cid("path-to-exile"),
            name: "Path to Exile".to_string(),
            mana_cost: Some(ManaCost { white: 1, ..Default::default() }),
            types: types(&[CardType::Instant]),
            oracle_text: "Exile target creature. Its controller may search their library for a basic land card, put that card onto the battlefield tapped, then shuffle.".to_string(),
            abilities: vec![AbilityDefinition::Spell {
                effect: Effect::Sequence(vec![
                    Effect::ExileObject {
                        target: EffectTarget::DeclaredTarget { index: 0 },
                    },
                    // Optional land fetch: modelled as an unconditional search (simplified).
                    Effect::SearchLibrary {
                        player: PlayerTarget::ControllerOf(Box::new(
                            EffectTarget::DeclaredTarget { index: 0 },
                        )),
                        filter: basic_land_filter(),
                        reveal: false,
                        destination: super::card_definition::ZoneTarget::Battlefield { tapped: true },
                    },
                    Effect::Shuffle {
                        player: PlayerTarget::ControllerOf(Box::new(
                            EffectTarget::DeclaredTarget { index: 0 },
                        )),
                    },
                ]),
                targets: vec![TargetRequirement::TargetCreature],
                modes: None,
                cant_be_countered: false,
            }],
            ..Default::default()
        },

        // 22. Beast Within — {2G}, Instant; destroy target permanent, its controller
        //     creates a 3/3 green Beast creature token.
        CardDefinition {
            card_id: cid("beast-within"),
            name: "Beast Within".to_string(),
            mana_cost: Some(ManaCost { green: 1, generic: 2, ..Default::default() }),
            types: types(&[CardType::Instant]),
            oracle_text: "Destroy target permanent. Its controller creates a 3/3 green Beast creature token.".to_string(),
            abilities: vec![AbilityDefinition::Spell {
                effect: Effect::Sequence(vec![
                    Effect::DestroyPermanent {
                        target: EffectTarget::DeclaredTarget { index: 0 },
                    },
                    Effect::CreateToken {
                        spec: super::card_definition::TokenSpec {
                            name: "Beast".to_string(),
                            power: 3,
                            toughness: 3,
                            colors: [Color::Green].into_iter().collect(),
                            card_types: [CardType::Creature].into_iter().collect(),
                            subtypes: [SubType("Beast".to_string())].into_iter().collect(),
                            keywords: OrdSet::new(),
                            count: 1,
                            tapped: false,
                            mana_color: None,
                        },
                    },
                ]),
                targets: vec![TargetRequirement::TargetPermanent],
                modes: None,
                cant_be_countered: false,
            }],
            ..Default::default()
        },

        // 23. Generous Gift — {2W}, Instant; destroy target permanent, its controller
        //     creates a 3/3 green Elephant creature token.
        CardDefinition {
            card_id: cid("generous-gift"),
            name: "Generous Gift".to_string(),
            mana_cost: Some(ManaCost { white: 1, generic: 2, ..Default::default() }),
            types: types(&[CardType::Instant]),
            oracle_text: "Destroy target permanent. Its controller creates a 3/3 green Elephant creature token.".to_string(),
            abilities: vec![AbilityDefinition::Spell {
                effect: Effect::Sequence(vec![
                    Effect::DestroyPermanent {
                        target: EffectTarget::DeclaredTarget { index: 0 },
                    },
                    Effect::CreateToken {
                        spec: super::card_definition::TokenSpec {
                            name: "Elephant".to_string(),
                            power: 3,
                            toughness: 3,
                            colors: [Color::Green].into_iter().collect(),
                            card_types: [CardType::Creature].into_iter().collect(),
                            subtypes: [SubType("Elephant".to_string())].into_iter().collect(),
                            keywords: OrdSet::new(),
                            count: 1,
                            tapped: false,
                            mana_color: None,
                        },
                    },
                ]),
                targets: vec![TargetRequirement::TargetPermanent],
                modes: None,
                cant_be_countered: false,
            }],
            ..Default::default()
        },

        // 24. Lightning Bolt — {R}, Instant; deal 3 damage to any target.
        CardDefinition {
            card_id: cid("lightning-bolt"),
            name: "Lightning Bolt".to_string(),
            mana_cost: Some(ManaCost { red: 1, ..Default::default() }),
            types: types(&[CardType::Instant]),
            oracle_text: "Lightning Bolt deals 3 damage to any target.".to_string(),
            abilities: vec![AbilityDefinition::Spell {
                effect: Effect::DealDamage {
                    target: EffectTarget::DeclaredTarget { index: 0 },
                    amount: EffectAmount::Fixed(3),
                },
                targets: vec![TargetRequirement::TargetAny],
                modes: None,
                cant_be_countered: false,
            }],
            ..Default::default()
        },

        // 25. Doom Blade — {1B}, Instant; destroy target non-black creature.
        CardDefinition {
            card_id: cid("doom-blade"),
            name: "Doom Blade".to_string(),
            mana_cost: Some(ManaCost { black: 1, generic: 1, ..Default::default() }),
            types: types(&[CardType::Instant]),
            oracle_text: "Destroy target non-black creature.".to_string(),
            abilities: vec![AbilityDefinition::Spell {
                effect: Effect::DestroyPermanent {
                    target: EffectTarget::DeclaredTarget { index: 0 },
                },
                targets: vec![TargetRequirement::TargetCreatureWithFilter(TargetFilter {
                    exclude_colors: Some([Color::Black].into_iter().collect()),
                    ..Default::default()
                })],
                modes: None,
                cant_be_countered: false,
            }],
            ..Default::default()
        },

        // ── Mass removal ─────────────────────────────────────────────────────

        // 26. Wrath of God — {2WW}, Sorcery; destroy all creatures. They can't be
        //     regenerated.
        CardDefinition {
            card_id: cid("wrath-of-god"),
            name: "Wrath of God".to_string(),
            mana_cost: Some(ManaCost { white: 2, generic: 2, ..Default::default() }),
            types: types(&[CardType::Sorcery]),
            oracle_text: "Destroy all creatures. They can't be regenerated.".to_string(),
            abilities: vec![AbilityDefinition::Spell {
                effect: Effect::ForEach {
                    over: super::card_definition::ForEachTarget::EachCreature,
                    effect: Box::new(Effect::DestroyPermanent {
                        target: EffectTarget::DeclaredTarget { index: 0 },
                    }),
                },
                targets: vec![],
                modes: None,
                cant_be_countered: false,
            }],
            ..Default::default()
        },

        // 27. Damnation — {2BB}, Sorcery; destroy all creatures. They can't be
        //     regenerated.
        CardDefinition {
            card_id: cid("damnation"),
            name: "Damnation".to_string(),
            mana_cost: Some(ManaCost { black: 2, generic: 2, ..Default::default() }),
            types: types(&[CardType::Sorcery]),
            oracle_text: "Destroy all creatures. They can't be regenerated.".to_string(),
            abilities: vec![AbilityDefinition::Spell {
                effect: Effect::ForEach {
                    over: super::card_definition::ForEachTarget::EachCreature,
                    effect: Box::new(Effect::DestroyPermanent {
                        target: EffectTarget::DeclaredTarget { index: 0 },
                    }),
                },
                targets: vec![],
                modes: None,
                cant_be_countered: false,
            }],
            ..Default::default()
        },

        // 28. Supreme Verdict — {1WWU}, Sorcery; destroy all creatures. It can't be
        //     countered.
        CardDefinition {
            card_id: cid("supreme-verdict"),
            name: "Supreme Verdict".to_string(),
            mana_cost: Some(ManaCost { white: 2, blue: 1, generic: 1, ..Default::default() }),
            types: types(&[CardType::Sorcery]),
            oracle_text: "This spell can't be countered.\nDestroy all creatures.".to_string(),
            abilities: vec![AbilityDefinition::Spell {
                effect: Effect::ForEach {
                    over: super::card_definition::ForEachTarget::EachCreature,
                    effect: Box::new(Effect::DestroyPermanent {
                        target: EffectTarget::DeclaredTarget { index: 0 },
                    }),
                },
                targets: vec![],
                modes: None,
                cant_be_countered: true,
            }],
            ..Default::default()
        },

        // ── Counterspells ────────────────────────────────────────────────────

        // 29. Counterspell — {UU}, Instant; counter target spell.
        CardDefinition {
            card_id: cid("counterspell"),
            name: "Counterspell".to_string(),
            mana_cost: Some(ManaCost { blue: 2, ..Default::default() }),
            types: types(&[CardType::Instant]),
            oracle_text: "Counter target spell.".to_string(),
            abilities: vec![AbilityDefinition::Spell {
                effect: Effect::CounterSpell {
                    target: EffectTarget::DeclaredTarget { index: 0 },
                },
                targets: vec![TargetRequirement::TargetSpell],
                modes: None,
                cant_be_countered: false,
            }],
            ..Default::default()
        },

        // 30. Negate — {1U}, Instant; counter target non-creature spell.
        CardDefinition {
            card_id: cid("negate"),
            name: "Negate".to_string(),
            mana_cost: Some(ManaCost { blue: 1, generic: 1, ..Default::default() }),
            types: types(&[CardType::Instant]),
            oracle_text: "Counter target noncreature spell.".to_string(),
            abilities: vec![AbilityDefinition::Spell {
                effect: Effect::CounterSpell {
                    target: EffectTarget::DeclaredTarget { index: 0 },
                },
                targets: vec![TargetRequirement::TargetSpellWithFilter(TargetFilter {
                    non_creature: true,
                    ..Default::default()
                })],
                modes: None,
                cant_be_countered: false,
            }],
            ..Default::default()
        },

        // 31. Swan Song — {U}, Instant; counter target instant, sorcery, or enchantment
        //     spell. Its controller creates a 2/2 blue Bird creature token with flying.
        CardDefinition {
            card_id: cid("swan-song"),
            name: "Swan Song".to_string(),
            mana_cost: Some(ManaCost { blue: 1, ..Default::default() }),
            types: types(&[CardType::Instant]),
            oracle_text: "Counter target instant, sorcery, or enchantment spell. Its controller creates a 2/2 blue Bird creature token with flying.".to_string(),
            abilities: vec![AbilityDefinition::Spell {
                effect: Effect::Sequence(vec![
                    Effect::CounterSpell {
                        target: EffectTarget::DeclaredTarget { index: 0 },
                    },
                    Effect::CreateToken {
                        spec: super::card_definition::TokenSpec {
                            name: "Bird".to_string(),
                            power: 2,
                            toughness: 2,
                            colors: [Color::Blue].into_iter().collect(),
                            card_types: [CardType::Creature].into_iter().collect(),
                            subtypes: [SubType("Bird".to_string())].into_iter().collect(),
                            keywords: [KeywordAbility::Flying].into_iter().collect(),
                            count: 1,
                            tapped: false,
                            mana_color: None,
                        },
                    },
                ]),
                targets: vec![TargetRequirement::TargetSpell],
                modes: None,
                cant_be_countered: false,
            }],
            ..Default::default()
        },

        // 32. Arcane Denial — {1U}, Instant; counter target spell. Its controller may
        //     draw up to two cards. You draw a card.
        CardDefinition {
            card_id: cid("arcane-denial"),
            name: "Arcane Denial".to_string(),
            mana_cost: Some(ManaCost { blue: 1, generic: 1, ..Default::default() }),
            types: types(&[CardType::Instant]),
            oracle_text: "Counter target spell. Its controller may draw up to two cards at the beginning of the next turn's upkeep.\nYou draw a card at the beginning of the next turn's upkeep.".to_string(),
            abilities: vec![AbilityDefinition::Spell {
                // Simplified: counter the spell and draw a card immediately.
                effect: Effect::Sequence(vec![
                    Effect::CounterSpell {
                        target: EffectTarget::DeclaredTarget { index: 0 },
                    },
                    Effect::DrawCards {
                        player: PlayerTarget::Controller,
                        count: EffectAmount::Fixed(1),
                    },
                ]),
                targets: vec![TargetRequirement::TargetSpell],
                modes: None,
                cant_be_countered: false,
            }],
            ..Default::default()
        },

        // ── Card draw ────────────────────────────────────────────────────────

        // 33. Harmonize — {2GG}, Sorcery; draw 3 cards.
        CardDefinition {
            card_id: cid("harmonize"),
            name: "Harmonize".to_string(),
            mana_cost: Some(ManaCost { green: 2, generic: 2, ..Default::default() }),
            types: types(&[CardType::Sorcery]),
            oracle_text: "Draw three cards.".to_string(),
            abilities: vec![AbilityDefinition::Spell {
                effect: Effect::DrawCards {
                    player: PlayerTarget::Controller,
                    count: EffectAmount::Fixed(3),
                },
                targets: vec![],
                modes: None,
                cant_be_countered: false,
            }],
            ..Default::default()
        },

        // 34. Divination — {2U}, Sorcery; draw 2 cards.
        CardDefinition {
            card_id: cid("divination"),
            name: "Divination".to_string(),
            mana_cost: Some(ManaCost { blue: 1, generic: 2, ..Default::default() }),
            types: types(&[CardType::Sorcery]),
            oracle_text: "Draw two cards.".to_string(),
            abilities: vec![AbilityDefinition::Spell {
                effect: Effect::DrawCards {
                    player: PlayerTarget::Controller,
                    count: EffectAmount::Fixed(2),
                },
                targets: vec![],
                modes: None,
                cant_be_countered: false,
            }],
            ..Default::default()
        },

        // 35. Night's Whisper — {1B}, Sorcery; you draw 2 cards and lose 2 life.
        CardDefinition {
            card_id: cid("nights-whisper"),
            name: "Night's Whisper".to_string(),
            mana_cost: Some(ManaCost { black: 1, generic: 1, ..Default::default() }),
            types: types(&[CardType::Sorcery]),
            oracle_text: "You draw two cards and you lose 2 life.".to_string(),
            abilities: vec![AbilityDefinition::Spell {
                effect: Effect::Sequence(vec![
                    Effect::DrawCards {
                        player: PlayerTarget::Controller,
                        count: EffectAmount::Fixed(2),
                    },
                    Effect::LoseLife {
                        player: PlayerTarget::Controller,
                        amount: EffectAmount::Fixed(2),
                    },
                ]),
                targets: vec![],
                modes: None,
                cant_be_countered: false,
            }],
            ..Default::default()
        },

        // 36. Sign in Blood — {BB}, Sorcery; target player draws 2 cards and loses 2 life.
        CardDefinition {
            card_id: cid("sign-in-blood"),
            name: "Sign in Blood".to_string(),
            mana_cost: Some(ManaCost { black: 2, ..Default::default() }),
            types: types(&[CardType::Sorcery]),
            oracle_text: "Target player draws two cards and loses 2 life.".to_string(),
            abilities: vec![AbilityDefinition::Spell {
                effect: Effect::Sequence(vec![
                    Effect::DrawCards {
                        player: PlayerTarget::DeclaredTarget { index: 0 },
                        count: EffectAmount::Fixed(2),
                    },
                    Effect::LoseLife {
                        player: PlayerTarget::DeclaredTarget { index: 0 },
                        amount: EffectAmount::Fixed(2),
                    },
                ]),
                targets: vec![TargetRequirement::TargetPlayer],
                modes: None,
                cant_be_countered: false,
            }],
            ..Default::default()
        },

        // 37. Read the Bones — {2B}, Sorcery; draw 2 cards, lose 2 life.
        //     (Scry 2 simplified as draw without scry for now.)
        CardDefinition {
            card_id: cid("read-the-bones"),
            name: "Read the Bones".to_string(),
            mana_cost: Some(ManaCost { black: 1, generic: 2, ..Default::default() }),
            types: types(&[CardType::Sorcery]),
            oracle_text: "Scry 2, then draw two cards. You lose 2 life.".to_string(),
            abilities: vec![AbilityDefinition::Spell {
                effect: Effect::Sequence(vec![
                    // Scry 2 simplified: not yet implemented, skip.
                    Effect::DrawCards {
                        player: PlayerTarget::Controller,
                        count: EffectAmount::Fixed(2),
                    },
                    Effect::LoseLife {
                        player: PlayerTarget::Controller,
                        amount: EffectAmount::Fixed(2),
                    },
                ]),
                targets: vec![],
                modes: None,
                cant_be_countered: false,
            }],
            ..Default::default()
        },

        // 38. Pull from Tomorrow — {X}{U}{U}, Instant; draw X cards, discard a card.
        //     (X is simplified as 3 for now.)
        CardDefinition {
            card_id: cid("pull-from-tomorrow"),
            name: "Pull from Tomorrow".to_string(),
            mana_cost: Some(ManaCost { blue: 2, ..Default::default() }),
            types: types(&[CardType::Instant]),
            oracle_text: "Draw X cards, then discard a card.".to_string(),
            abilities: vec![AbilityDefinition::Spell {
                effect: Effect::Sequence(vec![
                    Effect::DrawCards {
                        player: PlayerTarget::Controller,
                        count: EffectAmount::XValue,
                    },
                    Effect::DiscardCards {
                        player: PlayerTarget::Controller,
                        count: EffectAmount::Fixed(1),
                    },
                ]),
                targets: vec![],
                modes: None,
                cant_be_countered: false,
            }],
            ..Default::default()
        },

        // 39. Brainstorm — {U}, Instant; draw 3, then put 2 cards from hand on top of library.
        //     (CR 701.20 "put on top": deterministic M7 — takes first 2 by ObjectId ascending.)
        CardDefinition {
            card_id: cid("brainstorm"),
            name: "Brainstorm".to_string(),
            mana_cost: Some(ManaCost { blue: 1, ..Default::default() }),
            types: types(&[CardType::Instant]),
            oracle_text: "Draw three cards, then put two cards from your hand on top of your library in any order.".to_string(),
            abilities: vec![AbilityDefinition::Spell {
                effect: Effect::Sequence(vec![
                    Effect::DrawCards {
                        player: PlayerTarget::Controller,
                        count: EffectAmount::Fixed(3),
                    },
                    Effect::PutOnLibrary {
                        player: PlayerTarget::Controller,
                        count: EffectAmount::Fixed(2),
                        from: super::card_definition::ZoneTarget::Hand {
                            owner: PlayerTarget::Controller,
                        },
                    },
                ]),
                targets: vec![],
                modes: None,
                cant_be_countered: false,
            }],
            ..Default::default()
        },

        // 40. Rhystic Study — {2U}, Enchantment; whenever an opponent casts a spell,
        //     you may draw a card unless that player pays {1}.
        //     MayPayOrElse: opponent is the payer (EachOpponent); or_else draws for controller.
        //     In M7 the payment is never made — the draw always fires. Interactive
        //     payment choice is deferred to M9+ (interactive priority pass).
        CardDefinition {
            card_id: cid("rhystic-study"),
            name: "Rhystic Study".to_string(),
            mana_cost: Some(ManaCost { blue: 1, generic: 2, ..Default::default() }),
            types: types(&[CardType::Enchantment]),
            oracle_text: "Whenever an opponent casts a spell, you may draw a card unless that player pays {1}.".to_string(),
            abilities: vec![AbilityDefinition::Triggered {
                trigger_condition: TriggerCondition::WheneverOpponentCastsSpell,
                effect: Effect::MayPayOrElse {
                    cost: Cost::Mana(ManaCost { generic: 1, ..Default::default() }),
                    payer: PlayerTarget::EachOpponent,
                    or_else: Box::new(Effect::DrawCards {
                        player: PlayerTarget::Controller,
                        count: EffectAmount::Fixed(1),
                    }),
                },
                intervening_if: None,
            }],
            ..Default::default()
        },

        // ── Ramp spells ──────────────────────────────────────────────────────

        // 41. Cultivate — {2G}, Sorcery; search for 2 basic lands, one to battlefield
        //     tapped, one to hand, then shuffle.
        CardDefinition {
            card_id: cid("cultivate"),
            name: "Cultivate".to_string(),
            mana_cost: Some(ManaCost { green: 1, generic: 2, ..Default::default() }),
            types: types(&[CardType::Sorcery]),
            oracle_text: "Search your library for up to two basic land cards, reveal those cards, and put one onto the battlefield tapped and the other into your hand. Then shuffle.".to_string(),
            abilities: vec![AbilityDefinition::Spell {
                effect: Effect::Sequence(vec![
                    Effect::SearchLibrary {
                        player: PlayerTarget::Controller,
                        filter: basic_land_filter(),
                        reveal: true,
                        destination: super::card_definition::ZoneTarget::Battlefield { tapped: true },
                    },
                    Effect::SearchLibrary {
                        player: PlayerTarget::Controller,
                        filter: basic_land_filter(),
                        reveal: true,
                        destination: super::card_definition::ZoneTarget::Hand { owner: PlayerTarget::Controller },
                    },
                    Effect::Shuffle { player: PlayerTarget::Controller },
                ]),
                targets: vec![],
                modes: None,
                cant_be_countered: false,
            }],
            ..Default::default()
        },

        // 42. Kodama's Reach — {2G}, Sorcery; same as Cultivate.
        CardDefinition {
            card_id: cid("kodamas-reach"),
            name: "Kodama's Reach".to_string(),
            mana_cost: Some(ManaCost { green: 1, generic: 2, ..Default::default() }),
            types: types(&[CardType::Sorcery]),
            oracle_text: "Search your library for up to two basic land cards, reveal those cards, and put one onto the battlefield tapped and the other into your hand. Then shuffle.".to_string(),
            abilities: vec![AbilityDefinition::Spell {
                effect: Effect::Sequence(vec![
                    Effect::SearchLibrary {
                        player: PlayerTarget::Controller,
                        filter: basic_land_filter(),
                        reveal: true,
                        destination: super::card_definition::ZoneTarget::Battlefield { tapped: true },
                    },
                    Effect::SearchLibrary {
                        player: PlayerTarget::Controller,
                        filter: basic_land_filter(),
                        reveal: true,
                        destination: super::card_definition::ZoneTarget::Hand { owner: PlayerTarget::Controller },
                    },
                    Effect::Shuffle { player: PlayerTarget::Controller },
                ]),
                targets: vec![],
                modes: None,
                cant_be_countered: false,
            }],
            ..Default::default()
        },

        // 43. Rampant Growth — {1G}, Sorcery; search for a basic land, put it onto
        //     battlefield tapped, then shuffle.
        CardDefinition {
            card_id: cid("rampant-growth"),
            name: "Rampant Growth".to_string(),
            mana_cost: Some(ManaCost { green: 1, generic: 1, ..Default::default() }),
            types: types(&[CardType::Sorcery]),
            oracle_text: "Search your library for a basic land card, put it onto the battlefield tapped, then shuffle.".to_string(),
            abilities: vec![AbilityDefinition::Spell {
                effect: Effect::Sequence(vec![
                    Effect::SearchLibrary {
                        player: PlayerTarget::Controller,
                        filter: basic_land_filter(),
                        reveal: false,
                        destination: super::card_definition::ZoneTarget::Battlefield { tapped: true },
                    },
                    Effect::Shuffle { player: PlayerTarget::Controller },
                ]),
                targets: vec![],
                modes: None,
                cant_be_countered: false,
            }],
            ..Default::default()
        },

        // 44. Explosive Vegetation — {3G}, Sorcery; search for up to two basic lands,
        //     put them onto battlefield tapped, then shuffle.
        CardDefinition {
            card_id: cid("explosive-vegetation"),
            name: "Explosive Vegetation".to_string(),
            mana_cost: Some(ManaCost { green: 1, generic: 3, ..Default::default() }),
            types: types(&[CardType::Sorcery]),
            oracle_text: "Search your library for up to two basic land cards and put them onto the battlefield tapped. Then shuffle.".to_string(),
            abilities: vec![AbilityDefinition::Spell {
                effect: Effect::Sequence(vec![
                    Effect::SearchLibrary {
                        player: PlayerTarget::Controller,
                        filter: basic_land_filter(),
                        reveal: false,
                        destination: super::card_definition::ZoneTarget::Battlefield { tapped: true },
                    },
                    Effect::SearchLibrary {
                        player: PlayerTarget::Controller,
                        filter: basic_land_filter(),
                        reveal: false,
                        destination: super::card_definition::ZoneTarget::Battlefield { tapped: true },
                    },
                    Effect::Shuffle { player: PlayerTarget::Controller },
                ]),
                targets: vec![],
                modes: None,
                cant_be_countered: false,
            }],
            ..Default::default()
        },

        // ── Equipment ────────────────────────────────────────────────────────

        // 45. Lightning Greaves — {2}, Artifact — Equipment; Equipped creature has
        //     haste and shroud. Equip {0}.
        CardDefinition {
            card_id: cid("lightning-greaves"),
            name: "Lightning Greaves".to_string(),
            mana_cost: Some(ManaCost { generic: 2, ..Default::default() }),
            types: types_sub(&[CardType::Artifact], &["Equipment"]),
            oracle_text: "Equipped creature has haste and shroud. (It can't be the target of spells or abilities your opponents control.)\nEquip {0}".to_string(),
            abilities: vec![
                // Static ability (granting haste and shroud) not yet modelled via ContinuousEffectDef.
                // Keywords listed for reference; enforcement via layer system (M8+).
            ],
            ..Default::default()
        },

        // 46. Swiftfoot Boots — {2}, Artifact — Equipment; Equipped creature has
        //     haste and hexproof. Equip {1}.
        CardDefinition {
            card_id: cid("swiftfoot-boots"),
            name: "Swiftfoot Boots".to_string(),
            mana_cost: Some(ManaCost { generic: 2, ..Default::default() }),
            types: types_sub(&[CardType::Artifact], &["Equipment"]),
            oracle_text: "Equipped creature has haste and hexproof.\nEquip {1}".to_string(),
            abilities: vec![
                // Static ability; not yet modelled.
            ],
            ..Default::default()
        },

        // ── Utility creatures ────────────────────────────────────────────────

        // 47. Llanowar Elves — {G}, Creature — Elf Druid 1/1; {T}: add {G}.
        CardDefinition {
            card_id: cid("llanowar-elves"),
            name: "Llanowar Elves".to_string(),
            mana_cost: Some(ManaCost { green: 1, ..Default::default() }),
            types: creature_types(&["Elf", "Druid"]),
            oracle_text: "{T}: Add {G}.".to_string(),
            power: Some(1),
            toughness: Some(1),
            abilities: vec![AbilityDefinition::Activated {
                cost: Cost::Tap,
                effect: Effect::AddMana {
                    player: PlayerTarget::Controller,
                    mana: mana_pool(0, 0, 0, 0, 1, 0),
                },
                timing_restriction: None,
            }],
        },

        // 48. Elvish Mystic — {G}, Creature — Elf Druid 1/1; same as Llanowar Elves.
        CardDefinition {
            card_id: cid("elvish-mystic"),
            name: "Elvish Mystic".to_string(),
            mana_cost: Some(ManaCost { green: 1, ..Default::default() }),
            types: creature_types(&["Elf", "Druid"]),
            oracle_text: "{T}: Add {G}.".to_string(),
            power: Some(1),
            toughness: Some(1),
            abilities: vec![AbilityDefinition::Activated {
                cost: Cost::Tap,
                effect: Effect::AddMana {
                    player: PlayerTarget::Controller,
                    mana: mana_pool(0, 0, 0, 0, 1, 0),
                },
                timing_restriction: None,
            }],
        },

        // 49. Birds of Paradise — {G}, Creature — Bird 0/1; Flying; {T}: add one mana
        //     of any color.
        CardDefinition {
            card_id: cid("birds-of-paradise"),
            name: "Birds of Paradise".to_string(),
            mana_cost: Some(ManaCost { green: 1, ..Default::default() }),
            types: creature_types(&["Bird"]),
            oracle_text: "Flying\n{T}: Add one mana of any color.".to_string(),
            power: Some(0),
            toughness: Some(1),
            abilities: vec![
                AbilityDefinition::Keyword(KeywordAbility::Flying),
                AbilityDefinition::Activated {
                    cost: Cost::Tap,
                    effect: Effect::AddManaAnyColor { player: PlayerTarget::Controller },
                    timing_restriction: None,
                },
            ],
        },

        // 50. Wall of Omens — {1W}, Creature — Wall 0/4; Defender; When Wall of Omens
        //     enters the battlefield, draw a card.
        CardDefinition {
            card_id: cid("wall-of-omens"),
            name: "Wall of Omens".to_string(),
            mana_cost: Some(ManaCost { white: 1, generic: 1, ..Default::default() }),
            types: creature_types(&["Wall"]),
            oracle_text: "Defender\nWhen Wall of Omens enters the battlefield, draw a card."
                .to_string(),
            power: Some(0),
            toughness: Some(4),
            abilities: vec![
                AbilityDefinition::Keyword(KeywordAbility::Defender),
                AbilityDefinition::Triggered {
                    trigger_condition: TriggerCondition::WhenEntersBattlefield,
                    effect: Effect::DrawCards {
                        player: PlayerTarget::Controller,
                        count: EffectAmount::Fixed(1),
                    },
                    intervening_if: None,
                },
            ],
        },

        // 51. Solemn Simulacrum — {4}, Artifact Creature — Golem 2/2;
        //     When ~ enters the battlefield, search for a basic land, put it onto the
        //     battlefield tapped. When ~ dies, you may draw a card.
        CardDefinition {
            card_id: cid("solemn-simulacrum"),
            name: "Solemn Simulacrum".to_string(),
            mana_cost: Some(ManaCost { generic: 4, ..Default::default() }),
            types: types_sub(&[CardType::Artifact, CardType::Creature], &["Golem"]),
            power: Some(2),
            toughness: Some(2),
            oracle_text: "When Solemn Simulacrum enters the battlefield, you may search your library for a basic land card, put that card onto the battlefield tapped, then shuffle.\nWhen Solemn Simulacrum dies, you may draw a card.".to_string(),
            abilities: vec![
                AbilityDefinition::Triggered {
                    trigger_condition: TriggerCondition::WhenEntersBattlefield,
                    effect: Effect::Sequence(vec![
                        Effect::SearchLibrary {
                            player: PlayerTarget::Controller,
                            filter: basic_land_filter(),
                            reveal: false,
                            destination: super::card_definition::ZoneTarget::Battlefield { tapped: true },
                        },
                        Effect::Shuffle { player: PlayerTarget::Controller },
                    ]),
                    intervening_if: None,
                },
                AbilityDefinition::Triggered {
                    trigger_condition: TriggerCondition::WhenDies,
                    effect: Effect::DrawCards {
                        player: PlayerTarget::Controller,
                        count: EffectAmount::Fixed(1),
                    },
                    intervening_if: None,
                },
            ],
        },

        // ── Replacement-effect cards (M8 Session 6) ──────────────────────────

        // 52. Rest in Peace — {1W}, Enchantment.
        //     "When Rest in Peace enters the battlefield, exile all cards from all
        //      graveyards. If a card would be put into a graveyard from anywhere,
        //      exile it instead."
        //
        //     Simplification: The ETB "exile all graveyard cards" effect is deferred
        //     (requires targeting non-battlefield objects, not yet modelled). The
        //     ongoing replacement — all cards to graveyard → exile instead — is fully
        //     implemented. Registered via register_permanent_replacement_abilities when
        //     Rest in Peace enters the battlefield.
        CardDefinition {
            card_id: cid("rest-in-peace"),
            name: "Rest in Peace".to_string(),
            mana_cost: Some(ManaCost { white: 1, generic: 1, ..Default::default() }),
            types: types(&[CardType::Enchantment]),
            oracle_text:
                "When Rest in Peace enters the battlefield, exile all cards from all graveyards.\n\
                 If a card would be put into a graveyard from anywhere, exile it instead."
                    .to_string(),
            abilities: vec![
                // CR 614.1a: Replacement — any card going to any graveyard → exile instead.
                // is_self: false — global effect, not tied to Rest in Peace itself.
                AbilityDefinition::Replacement {
                    trigger: ReplacementTrigger::WouldChangeZone {
                        from: None,
                        to: ZoneType::Graveyard,
                        filter: ObjectFilter::Any,
                    },
                    modification: ReplacementModification::RedirectToZone(ZoneType::Exile),
                    is_self: false,
                },
            ],
            ..Default::default()
        },

        // 53. Leyline of the Void — {2BB}, Enchantment.
        //     "If Leyline of the Void is in your opening hand, you may begin the game
        //      with it on the battlefield. If a card an opponent owns would be put into
        //      that player's graveyard from anywhere, exile it instead."
        //
        //     Simplification: The "opening hand" leyline rule is not modelled — Leyline
        //     enters play normally when cast. The opponent-only filter uses
        //     ObjectFilter::OwnedByOpponentsOf with a placeholder PlayerId(0); the
        //     registration function (register_permanent_replacement_abilities) binds the
        //     actual controller's PlayerId at registration time (MR-M8-09).
        CardDefinition {
            card_id: cid("leyline-of-the-void"),
            name: "Leyline of the Void".to_string(),
            mana_cost: Some(ManaCost { black: 2, generic: 2, ..Default::default() }),
            types: types(&[CardType::Enchantment]),
            oracle_text:
                "If Leyline of the Void is in your opening hand, you may begin the game with it on the battlefield.\n\
                 If a card an opponent owns would be put into that player's graveyard from anywhere, exile it instead."
                    .to_string(),
            abilities: vec![
                // CR 614.1a: Replacement — opponent-owned cards going to graveyard → exile.
                // PlayerId(0) is a placeholder bound to the actual controller at
                // registration time by register_permanent_replacement_abilities.
                AbilityDefinition::Replacement {
                    trigger: ReplacementTrigger::WouldChangeZone {
                        from: None,
                        to: ZoneType::Graveyard,
                        filter: ObjectFilter::OwnedByOpponentsOf(PlayerId(0)),
                    },
                    modification: ReplacementModification::RedirectToZone(ZoneType::Exile),
                    is_self: false,
                },
            ],
            ..Default::default()
        },


        // 55. Alela, Cunning Conqueror — {2UB}, Legendary Creature — Faerie Warlock 2/4;
        //     Flying. Whenever you cast your first spell during each opponent's turn,
        //     create a 1/1 black Faerie Rogue creature token with flying. Whenever one or
        //     more Faeries you control deal combat damage to a player, goad target creature
        //     that player controls.
        //
        //     Simplification 1: TriggerCondition has no "during each opponent's turn" scoping.
        //     WheneverYouCastSpell is used as the closest approximation; it will also fire on
        //     your own turn.
        //     TODO: Add TriggerCondition::WheneverYouCastSpellDuringOpponentTurn (or a
        //     general "during opponent's turn" scope modifier) to restrict the token creation
        //     trigger to opponent turns only, and to track "first spell" per turn.
        //
        //     Simplification 2: WhenDealsCombatDamageToPlayer is self-referential (this
        //     creature). The oracle text fires for "one or more Faeries you control" which
        //     requires a controller-filtered creature-type watcher. Approximated as
        //     WhenDealsCombatDamageToPlayer (fires when Alela itself deals combat damage).
        //     TODO: Add TriggerCondition::WheneverCreatureYouControlDealsCombatDamageToPlayer
        //     with a creature-type filter.
        //
        //     Simplification 3: Effect::Goad does not exist. The second triggered ability
        //     is recorded as a no-op Effect::DrawCards(0) placeholder.
        //     TODO: Add Effect::Goad { target: EffectTarget } for the goad mechanic.
        CardDefinition {
            card_id: cid("alela-cunning-conqueror"),
            name: "Alela, Cunning Conqueror".to_string(),
            mana_cost: Some(ManaCost { generic: 2, blue: 1, black: 1, ..Default::default() }),
            types: full_types(&[SuperType::Legendary], &[CardType::Creature], &["Faerie", "Warlock"]),
            oracle_text:
                "Flying\n\
                 Whenever you cast your first spell during each opponent's turn, create a 1/1 black Faerie Rogue creature token with flying.\n\
                 Whenever one or more Faeries you control deal combat damage to a player, goad target creature that player controls."
                    .to_string(),
            power: Some(2),
            toughness: Some(4),
            abilities: vec![
                AbilityDefinition::Keyword(KeywordAbility::Flying),
                // TODO: restrict to opponent's turns only and track "first spell per turn".
                // Uses WheneverYouCastSpell as an approximation (fires on all turns).
                AbilityDefinition::Triggered {
                    trigger_condition: TriggerCondition::WheneverYouCastSpell,
                    effect: Effect::CreateToken {
                        spec: super::card_definition::TokenSpec {
                            name: "Faerie Rogue".to_string(),
                            power: 1,
                            toughness: 1,
                            colors: [Color::Black].into_iter().collect(),
                            card_types: [CardType::Creature].into_iter().collect(),
                            subtypes: [SubType("Faerie".to_string()), SubType("Rogue".to_string())]
                                .into_iter()
                                .collect(),
                            keywords: [KeywordAbility::Flying].into_iter().collect(),
                            count: 1,
                            tapped: false,
                            mana_color: None,
                        },
                    },
                    intervening_if: None,
                },
                // TODO: replace WhenDealsCombatDamageToPlayer with a Faerie-filtered
                // controller-creature trigger; replace placeholder effect with Effect::Goad.
                AbilityDefinition::Triggered {
                    trigger_condition: TriggerCondition::WhenDealsCombatDamageToPlayer,
                    // TODO: Effect::Goad { target: EffectTarget::DeclaredTarget { index: 0 } }
                    // Placeholder: no-op draw of 0 cards until Goad effect is implemented.
                    effect: Effect::DrawCards {
                        player: PlayerTarget::Controller,
                        count: EffectAmount::Fixed(0),
                    },
                    intervening_if: None,
                },
            ],
        },

        // 54. Darksteel Colossus — {11}, Artifact Creature — Golem 11/11.
        //     Trample, indestructible. If Darksteel Colossus would be put into a
        //     graveyard from anywhere, reveal it and shuffle it into its owner's library
        //     instead.
        //
        //     The self-replacement trigger uses ObjectFilter::Any as a placeholder;
        //     register_permanent_replacement_abilities substitutes SpecificObject(new_id)
        //     at registration time so the effect only fires for this specific Colossus.
        //     "Shuffle into library" is simplified to RedirectToZone(Library) (no shuffle).
        CardDefinition {
            card_id: cid("darksteel-colossus"),
            name: "Darksteel Colossus".to_string(),
            mana_cost: Some(ManaCost { generic: 11, ..Default::default() }),
            types: types_sub(&[CardType::Artifact, CardType::Creature], &["Golem"]),
            oracle_text:
                "Trample, indestructible.\n\
                 If Darksteel Colossus would be put into a graveyard from anywhere, reveal it \
                 and shuffle it into its owner's library instead."
                    .to_string(),
            power: Some(11),
            toughness: Some(11),
            abilities: vec![
                AbilityDefinition::Keyword(KeywordAbility::Trample),
                AbilityDefinition::Keyword(KeywordAbility::Indestructible),
                // CR 614.1a / 614.15: Self-replacement effect — if this specific Colossus
                // would go to a graveyard, send it to the library instead.
                // ObjectFilter::Any is replaced with SpecificObject at registration time.
                AbilityDefinition::Replacement {
                    trigger: ReplacementTrigger::WouldChangeZone {
                        from: None,
                        to: ZoneType::Graveyard,
                        filter: ObjectFilter::Any,
                    },
                    modification: ReplacementModification::RedirectToZone(ZoneType::Library),
                    is_self: true,
                },
            ],
        },


    ]
}
