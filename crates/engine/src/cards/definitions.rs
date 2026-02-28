//! Hand-authored card definitions (57 cards as of Flashback ability implementation).
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
    AffinityTarget, CardId, CardType, Color, EnchantTarget, KeywordAbility, LandwalkType, ManaCost,
    ManaPool, SubType, SuperType,
};

use super::card_definition::{
    food_token_spec, treasure_token_spec, AbilityDefinition, CardDefinition, Condition,
    ContinuousEffectDef, Cost, Effect, EffectAmount, EffectTarget, ForEachTarget, PlayerTarget,
    TargetFilter, TargetRequirement, TimingRestriction, TriggerCondition, TypeLine, ZoneTarget,
};
use crate::state::continuous_effect::{
    EffectDuration, EffectFilter, EffectLayer, LayerModification,
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
        //    CR 402.2: KeywordAbility::NoMaxHandSize signals the engine to skip the
        //    cleanup discard for the controller.
        CardDefinition {
            card_id: cid("thought-vessel"),
            name: "Thought Vessel".to_string(),
            mana_cost: Some(ManaCost { generic: 2, ..Default::default() }),
            types: types(&[CardType::Artifact]),
            oracle_text: "You have no maximum hand size.\n{T}: Add {C}.".to_string(),
            abilities: vec![
                // CR 402.2: no maximum hand size for controller.
                AbilityDefinition::Keyword(KeywordAbility::NoMaxHandSize),
                AbilityDefinition::Activated {
                    cost: Cost::Tap,
                    effect: Effect::AddMana {
                        player: PlayerTarget::Controller,
                        mana: mana_pool(0, 0, 0, 0, 0, 1),
                    },
                    timing_restriction: None,
                },
            ],
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
        //    CR 402.2: KeywordAbility::NoMaxHandSize signals the engine to skip the
        //    cleanup discard for the controller.
        CardDefinition {
            card_id: cid("reliquary-tower"),
            name: "Reliquary Tower".to_string(),
            mana_cost: None,
            types: types(&[CardType::Land]),
            oracle_text: "You have no maximum hand size.\n{T}: Add {C}.".to_string(),
            abilities: vec![
                // CR 402.2: no maximum hand size for controller.
                AbilityDefinition::Keyword(KeywordAbility::NoMaxHandSize),
                AbilityDefinition::Activated {
                    cost: Cost::Tap,
                    effect: Effect::AddMana {
                        player: PlayerTarget::Controller,
                        mana: mana_pool(0, 0, 0, 0, 0, 1),
                    },
                    timing_restriction: None,
                },
            ],
            ..Default::default()
        },

        // 13. Rogue's Passage — Land; {T}: add {C}; {4}, {T}: target creature can't be
        //     blocked this turn. (CR 509.1: creature with CantBeBlocked keyword can't
        //     be declared as a blocker's attack target.)
        CardDefinition {
            card_id: cid("rogues-passage"),
            name: "Rogue's Passage".to_string(),
            mana_cost: None,
            types: types(&[CardType::Land]),
            oracle_text: "{T}: Add {C}.\n{4}, {T}: Target creature can't be blocked this turn.".to_string(),
            abilities: vec![
                AbilityDefinition::Activated {
                    cost: Cost::Tap,
                    effect: Effect::AddMana {
                        player: PlayerTarget::Controller,
                        mana: mana_pool(0, 0, 0, 0, 0, 1),
                    },
                    timing_restriction: None,
                },
                // CR 509.1 / CR 702 (CantBeBlocked): {4}, {T}: target creature can't be
                // blocked this turn. Applies a UntilEndOfTurn continuous effect granting
                // KeywordAbility::CantBeBlocked in layer 6.
                AbilityDefinition::Activated {
                    cost: Cost::Sequence(vec![
                        Cost::Mana(ManaCost { generic: 4, ..Default::default() }),
                        Cost::Tap,
                    ]),
                    effect: Effect::ApplyContinuousEffect {
                        effect_def: Box::new(super::card_definition::ContinuousEffectDef {
                            layer: crate::state::EffectLayer::Ability,
                            modification: crate::state::LayerModification::AddKeyword(
                                KeywordAbility::CantBeBlocked,
                            ),
                            filter: crate::state::EffectFilter::DeclaredTarget { index: 0 },
                            duration: crate::state::EffectDuration::UntilEndOfTurn,
                        }),
                    },
                    timing_restriction: None,
                },
            ],
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
                // {T}: Add {U} or {B} (CR 106.6: player chooses color).
                // M9.4: uses Effect::Choose between AddMana blue and AddMana black.
                // Deterministic fallback executes the first option (blue).
                AbilityDefinition::Activated {
                    cost: Cost::Tap,
                    effect: Effect::Choose {
                        prompt: "Add {U} or {B}?".to_string(),
                        choices: vec![
                            Effect::AddMana {
                                player: PlayerTarget::Controller,
                                mana: mana_pool(0, 1, 0, 0, 0, 0),
                            },
                            Effect::AddMana {
                                player: PlayerTarget::Controller,
                                mana: mana_pool(0, 0, 1, 0, 0, 0),
                            },
                        ],
                    },
                    timing_restriction: None,
                },
            ],
            ..Default::default()
        },

        // 19b. Lonely Sandbar — Land — Island; enters tapped; cycling {U}.
        CardDefinition {
            card_id: cid("lonely-sandbar"),
            name: "Lonely Sandbar".to_string(),
            mana_cost: None,
            types: types_sub(&[CardType::Land], &["Island"]),
            oracle_text: "This land enters tapped.\n{T}: Add {U}.\nCycling {U} ({U}, Discard this card: Draw a card.)".to_string(),
            abilities: vec![
                // CR 614.1c: self-replacement effect — this permanent enters tapped.
                AbilityDefinition::Replacement {
                    trigger: ReplacementTrigger::WouldEnterBattlefield {
                        filter: ObjectFilter::Any,
                    },
                    modification: ReplacementModification::EntersTapped,
                    is_self: true,
                },
                // {T}: Add {U} (Island subtype grants this implicitly, but explicit here).
                AbilityDefinition::Activated {
                    cost: Cost::Tap,
                    effect: Effect::AddMana {
                        player: PlayerTarget::Controller,
                        mana: mana_pool(0, 1, 0, 0, 0, 0),
                    },
                    timing_restriction: None,
                },
                // CR 702.29: Cycling {U} — pay {U} and discard this card to draw a card.
                AbilityDefinition::Keyword(KeywordAbility::Cycling),
                AbilityDefinition::Cycling {
                    cost: ManaCost { blue: 1, ..Default::default() },
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
        //     CR 701.19: "may search" is modelled via MayPayOrElse with zero cost.
        //     M9.4 deterministic fallback: payer does not pay → or_else (search) fires.
        //     The exiled creature's controller is the payer (ControllerOf target).
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
                    // "May search" — modelled as MayPayOrElse with zero cost.
                    // or_else = search (fires when player declines to "pay" the
                    // zero cost, i.e. chooses NOT to search in interactive play).
                    // Deterministic fallback: always fires or_else (always searches).
                    Effect::MayPayOrElse {
                        cost: super::card_definition::Cost::Mana(
                            ManaCost { ..Default::default() }
                        ),
                        payer: PlayerTarget::ControllerOf(Box::new(
                            EffectTarget::DeclaredTarget { index: 0 },
                        )),
                        or_else: Box::new(Effect::Sequence(vec![
                            Effect::SearchLibrary {
                                player: PlayerTarget::ControllerOf(Box::new(
                                    EffectTarget::DeclaredTarget { index: 0 },
                                )),
                                filter: basic_land_filter(),
                                reveal: false,
                                destination: super::card_definition::ZoneTarget::Battlefield {
                                    tapped: true,
                                },
                            },
                            Effect::Shuffle {
                                player: PlayerTarget::ControllerOf(Box::new(
                                    EffectTarget::DeclaredTarget { index: 0 },
                                )),
                            },
                        ])),
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
                            mana_abilities: vec![],
                            activated_abilities: vec![],
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
                            mana_abilities: vec![],
                            activated_abilities: vec![],
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

        // 26a. Krosan Grip — {2G}, Instant; split second; destroy target artifact
        //      or enchantment.
        //      TODO: No TargetArtifactOrEnchantment variant exists; using TargetPermanent
        //      as an approximation. A combined variant (or has_any_of_card_types on
        //      TargetFilter) would be needed for precise targeting enforcement.
        CardDefinition {
            card_id: cid("krosan-grip"),
            name: "Krosan Grip".to_string(),
            mana_cost: Some(ManaCost { green: 1, generic: 2, ..Default::default() }),
            types: types(&[CardType::Instant]),
            oracle_text: "Split second (As long as this spell is on the stack, players can't cast spells or activate abilities that aren't mana abilities.)\nDestroy target artifact or enchantment.".to_string(),
            abilities: vec![
                AbilityDefinition::Keyword(KeywordAbility::SplitSecond),
                AbilityDefinition::Spell {
                    // TODO: target should be TargetArtifactOrEnchantment (no such variant);
                    // TargetPermanent is used as the closest approximation.
                    effect: Effect::DestroyPermanent {
                        target: EffectTarget::DeclaredTarget { index: 0 },
                    },
                    targets: vec![TargetRequirement::TargetPermanent],
                    modes: None,
                    cant_be_countered: false,
                },
            ],
            ..Default::default()
        },

        // Bake into a Pie — {2BB}, Instant; destroy target creature, create a Food token.
        CardDefinition {
            card_id: cid("bake-into-a-pie"),
            name: "Bake into a Pie".to_string(),
            mana_cost: Some(ManaCost { generic: 2, black: 2, ..Default::default() }),
            types: types(&[CardType::Instant]),
            oracle_text: "Destroy target creature. Create a Food token. (It's an artifact with \"{2}, {T}, Sacrifice this token: You gain 3 life.\")".to_string(),
            abilities: vec![AbilityDefinition::Spell {
                effect: Effect::Sequence(vec![
                    Effect::DestroyPermanent {
                        target: EffectTarget::DeclaredTarget { index: 0 },
                    },
                    Effect::CreateToken { spec: food_token_spec(1) },
                ]),
                targets: vec![TargetRequirement::TargetCreature],
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
                            mana_abilities: vec![],
                            activated_abilities: vec![],
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

        // 35. Treasure Cruise — {7U}, Sorcery; delve, draw 3 cards.
        CardDefinition {
            card_id: cid("treasure-cruise"),
            name: "Treasure Cruise".to_string(),
            mana_cost: Some(ManaCost { blue: 1, generic: 7, ..Default::default() }),
            types: types(&[CardType::Sorcery]),
            oracle_text: "Delve (Each card you exile from your graveyard while casting this spell pays for {1}.)\nDraw three cards.".to_string(),
            abilities: vec![
                AbilityDefinition::Keyword(KeywordAbility::Delve),
                AbilityDefinition::Spell {
                    effect: Effect::DrawCards {
                        player: PlayerTarget::Controller,
                        count: EffectAmount::Fixed(3),
                    },
                    targets: vec![],
                    modes: None,
                    cant_be_countered: false,
                },
            ],
            ..Default::default()
        },

        // Reverse Engineer — {3UU}, Sorcery; improvise, draw 3 cards.
        CardDefinition {
            card_id: cid("reverse-engineer"),
            name: "Reverse Engineer".to_string(),
            mana_cost: Some(ManaCost { blue: 2, generic: 3, ..Default::default() }),
            types: types(&[CardType::Sorcery]),
            oracle_text: "Improvise (Your artifacts can help cast this spell. Each artifact you tap after you're done activating mana abilities pays for {1}.)\nDraw three cards.".to_string(),
            abilities: vec![
                AbilityDefinition::Keyword(KeywordAbility::Improvise),
                AbilityDefinition::Spell {
                    effect: Effect::DrawCards {
                        player: PlayerTarget::Controller,
                        count: EffectAmount::Fixed(3),
                    },
                    targets: vec![],
                    modes: None,
                    cant_be_countered: false,
                },
            ],
            ..Default::default()
        },

        // 36. Night's Whisper — {1B}, Sorcery; you draw 2 cards and lose 2 life.
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

        // 37. Read the Bones — {2B}, Sorcery; scry 2, draw 2 cards, lose 2 life.
        //     CR 701.18: Scry 2 implemented via Effect::Scry { count: Fixed(2) }.
        CardDefinition {
            card_id: cid("read-the-bones"),
            name: "Read the Bones".to_string(),
            mana_cost: Some(ManaCost { black: 1, generic: 2, ..Default::default() }),
            types: types(&[CardType::Sorcery]),
            oracle_text: "Scry 2, then draw two cards. You lose 2 life.".to_string(),
            abilities: vec![AbilityDefinition::Spell {
                effect: Effect::Sequence(vec![
                    // CR 701.18: Scry 2 before drawing.
                    Effect::Scry {
                        player: PlayerTarget::Controller,
                        count: EffectAmount::Fixed(2),
                    },
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

        // Consider — {U}, Instant; surveil 1, then draw a card.
        //     CR 701.25: Surveil 1 before drawing.
        CardDefinition {
            card_id: cid("consider"),
            name: "Consider".to_string(),
            mana_cost: Some(ManaCost { blue: 1, ..Default::default() }),
            types: types(&[CardType::Instant]),
            oracle_text: "Surveil 1. (Look at the top card of your library. You may put it into your graveyard.)\nDraw a card.".to_string(),
            abilities: vec![AbilityDefinition::Spell {
                effect: Effect::Sequence(vec![
                    Effect::Surveil {
                        player: PlayerTarget::Controller,
                        count: EffectAmount::Fixed(1),
                    },
                    Effect::DrawCards {
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

        // 40. Rhystic Study — {2U}, Enchantment; whenever an opponent casts a spell,
        //     you may draw a card unless that player pays {1}.
        //     M9.4: payer is DeclaredTarget { index: 0 } — the opponent who cast the spell.
        //     The triggering opponent is expected to be passed as target 0 when the
        //     card-def trigger dispatch system is wired up (currently deferred).
        //     In the interim the draw always fires (payment never collected) because
        //     triggered abilities resolve with targets: vec![].
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
                    // DeclaredTarget { index: 0 } = the specific opponent who cast the spell.
                    // This is the correct model (CR 603.1): only "that player" pays, not all
                    // opponents. Resolves to an empty list at runtime until trigger context
                    // wiring passes the casting opponent as target 0.
                    payer: PlayerTarget::DeclaredTarget { index: 0 },
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
        //     CR 702.6a: Equipment static ability grants keywords to equipped creature.
        //     CR 604.2: Static ability functions while on the battlefield.
        CardDefinition {
            card_id: cid("lightning-greaves"),
            name: "Lightning Greaves".to_string(),
            mana_cost: Some(ManaCost { generic: 2, ..Default::default() }),
            types: types_sub(&[CardType::Artifact], &["Equipment"]),
            oracle_text: "Equipped creature has haste and shroud. (It can't be the target of spells or abilities your opponents control.)\nEquip {0}".to_string(),
            abilities: vec![
                // CR 702.6a: Equipped creature has Haste and Shroud (layer 6 ability grant).
                AbilityDefinition::Static {
                    continuous_effect: ContinuousEffectDef {
                        layer: EffectLayer::Ability,
                        modification: LayerModification::AddKeywords(
                            [KeywordAbility::Haste, KeywordAbility::Shroud]
                                .into_iter()
                                .collect(),
                        ),
                        filter: EffectFilter::AttachedCreature,
                        duration: EffectDuration::WhileSourceOnBattlefield,
                    },
                },
                // Equip {0}: attach this Equipment to target creature you control.
                // CR 702.6b: Equip is an activated ability; sorcery speed (CR 702.6d).
                AbilityDefinition::Activated {
                    cost: Cost::Mana(ManaCost { ..Default::default() }), // Equip {0}
                    effect: Effect::AttachEquipment {
                        equipment: EffectTarget::Source,
                        target: EffectTarget::DeclaredTarget { index: 0 },
                    },
                    timing_restriction: Some(TimingRestriction::SorcerySpeed),
                },
            ],
            ..Default::default()
        },

        // 46. Swiftfoot Boots — {2}, Artifact — Equipment; Equipped creature has
        //     haste and hexproof. Equip {1}.
        //     CR 702.6a: Equipment static ability grants keywords to equipped creature.
        //     CR 604.2: Static ability functions while on the battlefield.
        CardDefinition {
            card_id: cid("swiftfoot-boots"),
            name: "Swiftfoot Boots".to_string(),
            mana_cost: Some(ManaCost { generic: 2, ..Default::default() }),
            types: types_sub(&[CardType::Artifact], &["Equipment"]),
            oracle_text: "Equipped creature has haste and hexproof.\nEquip {1}".to_string(),
            abilities: vec![
                // CR 702.6a: Equipped creature has Haste and Hexproof (layer 6 ability grant).
                AbilityDefinition::Static {
                    continuous_effect: ContinuousEffectDef {
                        layer: EffectLayer::Ability,
                        modification: LayerModification::AddKeywords(
                            [KeywordAbility::Haste, KeywordAbility::Hexproof]
                                .into_iter()
                                .collect(),
                        ),
                        filter: EffectFilter::AttachedCreature,
                        duration: EffectDuration::WhileSourceOnBattlefield,
                    },
                },
                // Equip {1}: attach this Equipment to target creature you control.
                // CR 702.6b: Equip is an activated ability; sorcery speed (CR 702.6d).
                AbilityDefinition::Activated {
                    cost: Cost::Mana(ManaCost { generic: 1, ..Default::default() }), // Equip {1}
                    effect: Effect::AttachEquipment {
                        equipment: EffectTarget::Source,
                        target: EffectTarget::DeclaredTarget { index: 0 },
                    },
                    timing_restriction: Some(TimingRestriction::SorcerySpeed),
                },
            ],
            ..Default::default()
        },

        // 47. Whispersilk Cloak — {3}, Artifact — Equipment; Equipped creature has
        //     shroud and can't be blocked. Equip {2}.
        //     CR 702.6a: Equipment static ability grants keyword to equipped creature.
        //     CR 604.2: Static ability functions while on the battlefield.
        CardDefinition {
            card_id: cid("whispersilk-cloak"),
            name: "Whispersilk Cloak".to_string(),
            mana_cost: Some(ManaCost { generic: 3, ..Default::default() }),
            types: types_sub(&[CardType::Artifact], &["Equipment"]),
            oracle_text: "Equipped creature can't be blocked and has shroud. (It can't be the target of spells or abilities.)\nEquip {2}".to_string(),
            abilities: vec![
                // CR 702.6a: Equipped creature has Shroud (layer 6 ability grant).
                AbilityDefinition::Static {
                    continuous_effect: ContinuousEffectDef {
                        layer: EffectLayer::Ability,
                        modification: LayerModification::AddKeyword(KeywordAbility::Shroud),
                        filter: EffectFilter::AttachedCreature,
                        duration: EffectDuration::WhileSourceOnBattlefield,
                    },
                },
                // CR 509.1b: Equipped creature can't be blocked.
                // Grants CantBeBlocked in layer 6 (ability) while this Equipment is attached.
                AbilityDefinition::Static {
                    continuous_effect: ContinuousEffectDef {
                        layer: EffectLayer::Ability,
                        modification: LayerModification::AddKeyword(KeywordAbility::CantBeBlocked),
                        filter: EffectFilter::AttachedCreature,
                        duration: EffectDuration::WhileSourceOnBattlefield,
                    },
                },
                // Equip {2}: attach this Equipment to target creature you control.
                // CR 702.6b: Equip is an activated ability; sorcery speed (CR 702.6d).
                AbilityDefinition::Activated {
                    cost: Cost::Mana(ManaCost { generic: 2, ..Default::default() }), // Equip {2}
                    effect: Effect::AttachEquipment {
                        equipment: EffectTarget::Source,
                        target: EffectTarget::DeclaredTarget { index: 0 },
                    },
                    timing_restriction: Some(TimingRestriction::SorcerySpeed),
                },
            ],
            ..Default::default()
        },

        // 48. Batterskull — {5}, Artifact — Equipment; Living weapon. Equipped creature gets
        //     +4/+4 and has vigilance and lifelink. Equip {5}.
        //     CR 702.92a: Living weapon ETB trigger creates 0/0 black Phyrexian Germ, attaches.
        //     CR 702.6a: Equipment static ability grants keywords to equipped creature.
        CardDefinition {
            card_id: cid("batterskull"),
            name: "Batterskull".to_string(),
            mana_cost: Some(ManaCost { generic: 5, ..Default::default() }),
            types: types_sub(&[CardType::Artifact], &["Equipment"]),
            oracle_text:
                "Living weapon (When this Equipment enters, create a 0/0 black Phyrexian Germ \
                 creature token, then attach this Equipment to it.)\n\
                 Equipped creature gets +4/+4 and has vigilance and lifelink.\n\
                 Equip {5}"
                    .to_string(),
            abilities: vec![
                // CR 702.92a: Living weapon — ETB trigger handled by builder.rs keyword wiring.
                AbilityDefinition::Keyword(KeywordAbility::LivingWeapon),
                // CR 702.6a: Equipped creature gets +4/+4 (layer 7c).
                AbilityDefinition::Static {
                    continuous_effect: ContinuousEffectDef {
                        layer: EffectLayer::PtModify,
                        modification: LayerModification::ModifyBoth(4),
                        filter: EffectFilter::AttachedCreature,
                        duration: EffectDuration::WhileSourceOnBattlefield,
                    },
                },
                // CR 702.6a: Equipped creature has vigilance and lifelink (layer 6).
                AbilityDefinition::Static {
                    continuous_effect: ContinuousEffectDef {
                        layer: EffectLayer::Ability,
                        modification: LayerModification::AddKeywords(
                            [KeywordAbility::Vigilance, KeywordAbility::Lifelink]
                                .into_iter()
                                .collect(),
                        ),
                        filter: EffectFilter::AttachedCreature,
                        duration: EffectDuration::WhileSourceOnBattlefield,
                    },
                },
                // Equip {5}: attach this Equipment to target creature you control.
                // CR 702.6b/d: Equip is a sorcery-speed activated ability.
                AbilityDefinition::Activated {
                    cost: Cost::Mana(ManaCost { generic: 5, ..Default::default() }),
                    effect: Effect::AttachEquipment {
                        equipment: EffectTarget::Source,
                        target: EffectTarget::DeclaredTarget { index: 0 },
                    },
                    timing_restriction: Some(TimingRestriction::SorcerySpeed),
                },
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

        // 55. Monastery Swiftspear — {R}, Creature — Human Monk 1/2; Haste. Prowess.
        CardDefinition {
            card_id: cid("monastery-swiftspear"),
            name: "Monastery Swiftspear".to_string(),
            mana_cost: Some(ManaCost { red: 1, ..Default::default() }),
            types: creature_types(&["Human", "Monk"]),
            oracle_text: "Haste\nProwess (Whenever you cast a noncreature spell, this creature gets +1/+1 until end of turn.)".to_string(),
            power: Some(1),
            toughness: Some(2),
            abilities: vec![
                AbilityDefinition::Keyword(KeywordAbility::Haste),
                // CR 702.108a: Prowess — builder.rs auto-generates the triggered ability from this keyword.
                AbilityDefinition::Keyword(KeywordAbility::Prowess),
            ],
        },

        // 56. Bladetusk Boar — {3R}, Creature — Boar 3/2; Intimidate.
        CardDefinition {
            card_id: cid("bladetusk-boar"),
            name: "Bladetusk Boar".to_string(),
            mana_cost: Some(ManaCost { generic: 3, red: 1, ..Default::default() }),
            types: creature_types(&["Boar"]),
            oracle_text: "Intimidate (This creature can't be blocked except by artifact creatures and/or creatures that share a color with it.)".to_string(),
            power: Some(3),
            toughness: Some(2),
            abilities: vec![
                AbilityDefinition::Keyword(KeywordAbility::Intimidate),
            ],
        },

        // 57. Fiery Temper — {1RR}, Instant; Fiery Temper deals 3 damage to any target.
        //     Madness {R} (If you discard this card, discard it into exile. When you do,
        //     cast it for its madness cost or put it into your graveyard.)
        //     CR 702.35: Madness — exile replacement + triggered cast opportunity.
        CardDefinition {
            card_id: cid("fiery-temper"),
            name: "Fiery Temper".to_string(),
            mana_cost: Some(ManaCost { generic: 1, red: 2, ..Default::default() }),
            types: types(&[CardType::Instant]),
            oracle_text:
                "Fiery Temper deals 3 damage to any target.\n\
                 Madness {R} (If you discard this card, discard it into exile. When you do, \
                 cast it for its madness cost or put it into your graveyard.)"
                    .to_string(),
            abilities: vec![
                AbilityDefinition::Spell {
                    effect: Effect::DealDamage {
                        target: EffectTarget::DeclaredTarget { index: 0 },
                        amount: EffectAmount::Fixed(3),
                    },
                    targets: vec![TargetRequirement::TargetAny],
                    modes: None,
                    cant_be_countered: false,
                },
                AbilityDefinition::Keyword(KeywordAbility::Madness),
                AbilityDefinition::Madness { cost: ManaCost { red: 1, ..Default::default() } },
            ],
            ..Default::default()
        },

        // 58. Bog Raiders — {2B}, Creature — Zombie 2/2; Swampwalk.
        CardDefinition {
            card_id: cid("bog-raiders"),
            name: "Bog Raiders".to_string(),
            mana_cost: Some(ManaCost { generic: 2, black: 1, ..Default::default() }),
            types: creature_types(&["Zombie"]),
            oracle_text: "Swampwalk (This creature can't be blocked as long as defending player controls a Swamp.)".to_string(),
            power: Some(2),
            toughness: Some(2),
            abilities: vec![
                AbilityDefinition::Keyword(KeywordAbility::Landwalk(
                    LandwalkType::BasicType(SubType("Swamp".to_string())),
                )),
            ],
        },

        // 58. Severed Legion — {1BB}, Creature — Zombie 2/2; Fear.
        CardDefinition {
            card_id: cid("severed-legion"),
            name: "Severed Legion".to_string(),
            mana_cost: Some(ManaCost { generic: 1, black: 2, ..Default::default() }),
            types: creature_types(&["Zombie"]),
            oracle_text: "Fear (This creature can't be blocked except by artifact creatures and/or black creatures.)".to_string(),
            power: Some(2),
            toughness: Some(2),
            abilities: vec![
                AbilityDefinition::Keyword(KeywordAbility::Fear),
            ],
        },

        // 59. Boon Satyr — {2G}, Creature — Satyr 4/2; Flash; Bestow {4GG}.
        CardDefinition {
            card_id: cid("boon-satyr"),
            name: "Boon Satyr".to_string(),
            mana_cost: Some(ManaCost { generic: 2, green: 1, ..Default::default() }),
            types: creature_types(&["Satyr"]),
            oracle_text: "Flash\nBestow {4}{G}{G} (If you cast this card for its bestow cost, it's an Aura spell with enchant creature. It becomes a creature again if it's not attached to a creature.)".to_string(),
            power: Some(4),
            toughness: Some(2),
            abilities: vec![
                AbilityDefinition::Keyword(KeywordAbility::Flash),
                AbilityDefinition::Keyword(KeywordAbility::Bestow),
                AbilityDefinition::Bestow { cost: ManaCost { generic: 4, green: 2, ..Default::default() } },
            ],
        },

        // 60. Audacious Thief — {2B}, Creature — Human Rogue 2/2;
        //     Whenever this creature attacks, you draw a card and you lose 1 life.
        CardDefinition {
            card_id: cid("audacious-thief"),
            name: "Audacious Thief".to_string(),
            mana_cost: Some(ManaCost { generic: 2, black: 1, ..Default::default() }),
            types: creature_types(&["Human", "Rogue"]),
            oracle_text: "Whenever Audacious Thief attacks, you draw a card and you lose 1 life."
                .to_string(),
            power: Some(2),
            toughness: Some(2),
            abilities: vec![AbilityDefinition::Triggered {
                trigger_condition: TriggerCondition::WhenAttacks,
                effect: Effect::Sequence(vec![
                    Effect::DrawCards {
                        player: PlayerTarget::Controller,
                        count: EffectAmount::Fixed(1),
                    },
                    Effect::LoseLife {
                        player: PlayerTarget::Controller,
                        amount: EffectAmount::Fixed(1),
                    },
                ]),
                intervening_if: None,
            }],
        },

        // 59. Scroll Thief — {2U}, Creature — Merfolk Rogue 1/3;
        //     Whenever this creature deals combat damage to a player, draw a card.
        CardDefinition {
            card_id: cid("scroll-thief"),
            name: "Scroll Thief".to_string(),
            mana_cost: Some(ManaCost { generic: 2, blue: 1, ..Default::default() }),
            types: creature_types(&["Merfolk", "Rogue"]),
            oracle_text: "Whenever this creature deals combat damage to a player, draw a card."
                .to_string(),
            power: Some(1),
            toughness: Some(3),
            abilities: vec![AbilityDefinition::Triggered {
                trigger_condition: TriggerCondition::WhenDealsCombatDamageToPlayer,
                effect: Effect::DrawCards {
                    player: PlayerTarget::Controller,
                    count: EffectAmount::Fixed(1),
                },
                intervening_if: None,
            }],
        },

        // 60. Siege Wurm — {5GG}, Creature — Wurm 5/5; Convoke. Trample.
        CardDefinition {
            card_id: cid("siege-wurm"),
            name: "Siege Wurm".to_string(),
            mana_cost: Some(ManaCost { generic: 5, green: 2, ..Default::default() }),
            types: creature_types(&["Wurm"]),
            oracle_text: "Convoke (Your creatures can help cast this spell. Each creature you tap while casting this spell pays for {1} or one mana of that creature's color.)\nTrample".to_string(),
            power: Some(5),
            toughness: Some(5),
            abilities: vec![
                AbilityDefinition::Keyword(KeywordAbility::Convoke),
                AbilityDefinition::Keyword(KeywordAbility::Trample),
            ],
        },

        // 61. Akrasan Squire — {W}, Creature — Human Soldier 1/1; Exalted (CR 702.83).
        CardDefinition {
            card_id: cid("akrasan-squire"),
            name: "Akrasan Squire".to_string(),
            mana_cost: Some(ManaCost { white: 1, ..Default::default() }),
            types: creature_types(&["Human", "Soldier"]),
            oracle_text: "Exalted (Whenever a creature you control attacks alone, that creature gets +1/+1 until end of turn.)".to_string(),
            power: Some(1),
            toughness: Some(1),
            abilities: vec![AbilityDefinition::Keyword(KeywordAbility::Exalted)],
        },

        // 62. Ulamog's Crusher — {8}, Creature — Eldrazi 8/8.
        //     Annihilator 2 (CR 702.86a): whenever this creature attacks, the defending
        //     player sacrifices two permanents. Engine support: builder.rs registers the
        //     WhenAttacks triggered ability from KeywordAbility::Annihilator(n).
        //     "This creature attacks each combat if able." — no DSL variant exists for
        //     the compelled-attack static ability; tracked below as a TODO.
        CardDefinition {
            card_id: cid("ulamogs-crusher"),
            name: "Ulamog's Crusher".to_string(),
            mana_cost: Some(ManaCost { generic: 8, ..Default::default() }),
            types: creature_types(&["Eldrazi"]),
            oracle_text: "Annihilator 2 (Whenever this creature attacks, defending player sacrifices two permanents.)\nThis creature attacks each combat if able.".to_string(),
            power: Some(8),
            toughness: Some(8),
            abilities: vec![
                AbilityDefinition::Keyword(KeywordAbility::Annihilator(2)),
                // TODO: Add a KeywordAbility::AttacksEachCombatIfAble variant (or a Static
                // ContinuousEffectDef) to enforce the compelled-attack rule (CR 508.1d).
                // Until then the "attacks each combat if able" text is cosmetic only.
            ],
        },

        // 63. Kitchen Finks — {1}{G/W}{G/W}, Creature — Ouphe 3/2; ETB gain 2 life. Persist.
        //     Oracle cost is {1}{G/W}{G/W} (hybrid); simplified here to {1}{W}{G} because
        //     the ManaCost struct does not support hybrid mana symbols.
        CardDefinition {
            card_id: cid("kitchen-finks"),
            name: "Kitchen Finks".to_string(),
            mana_cost: Some(ManaCost { generic: 1, white: 1, green: 1, ..Default::default() }),
            types: creature_types(&["Ouphe"]),
            oracle_text: "When this creature enters, you gain 2 life.\nPersist (When this creature dies, if it had no -1/-1 counters on it, return it to the battlefield under its owner's control with a -1/-1 counter on it.)".to_string(),
            power: Some(3),
            toughness: Some(2),
            abilities: vec![
                AbilityDefinition::Triggered {
                    trigger_condition: TriggerCondition::WhenEntersBattlefield,
                    effect: Effect::GainLife {
                        player: PlayerTarget::Controller,
                        amount: EffectAmount::Fixed(2),
                    },
                    intervening_if: None,
                },
                AbilityDefinition::Keyword(KeywordAbility::Persist),
            ],
        },

        // 64. Young Wolf — {G}, Creature — Wolf 1/1; Undying.
        CardDefinition {
            card_id: cid("young-wolf"),
            name: "Young Wolf".to_string(),
            mana_cost: Some(ManaCost { green: 1, ..Default::default() }),
            types: creature_types(&["Wolf"]),
            oracle_text: "Undying (When this creature dies, if it had no +1/+1 counters on it, return it to the battlefield under its owner's control with a +1/+1 counter on it.)".to_string(),
            power: Some(1),
            toughness: Some(1),
            abilities: vec![
                AbilityDefinition::Keyword(KeywordAbility::Undying),
            ],
        },

        // 65. Universal Automaton — {1}, Artifact Creature — Shapeshifter 1/1; Changeling.
        CardDefinition {
            card_id: cid("universal-automaton"),
            name: "Universal Automaton".to_string(),
            mana_cost: Some(ManaCost { generic: 1, ..Default::default() }),
            types: types_sub(&[CardType::Artifact, CardType::Creature], &["Shapeshifter"]),
            oracle_text: "Changeling (This card is every creature type.)".to_string(),
            power: Some(1),
            toughness: Some(1),
            abilities: vec![
                AbilityDefinition::Keyword(KeywordAbility::Changeling),
            ],
        },

        // 66. Mulldrifter — {4}{U}, Creature — Elemental 2/2; Flying; ETB draw two cards;
        //     Evoke {2}{U}.
        CardDefinition {
            card_id: cid("mulldrifter"),
            name: "Mulldrifter".to_string(),
            mana_cost: Some(ManaCost { generic: 4, blue: 1, ..Default::default() }),
            types: creature_types(&["Elemental"]),
            oracle_text: "Flying\nWhen this creature enters, draw two cards.\nEvoke {2}{U} (You may cast this spell for its evoke cost. If you do, it's sacrificed when it enters.)".to_string(),
            power: Some(2),
            toughness: Some(2),
            abilities: vec![
                AbilityDefinition::Keyword(KeywordAbility::Flying),
                AbilityDefinition::Triggered {
                    trigger_condition: TriggerCondition::WhenEntersBattlefield,
                    effect: Effect::DrawCards {
                        player: PlayerTarget::Controller,
                        count: EffectAmount::Fixed(2),
                    },
                    intervening_if: None,
                },
                AbilityDefinition::Evoke {
                    cost: ManaCost { generic: 2, blue: 1, ..Default::default() },
                },
            ],
        },

        // 67. Smuggler's Copter — {2}, Artifact — Vehicle 3/3; Flying; Crew 1;
        //     Whenever it attacks or blocks, you may draw a card. If you do, discard a card.
        CardDefinition {
            card_id: cid("smugglers-copter"),
            name: "Smuggler's Copter".to_string(),
            mana_cost: Some(ManaCost { generic: 2, ..Default::default() }),
            types: TypeLine {
                card_types: [CardType::Artifact].iter().copied().collect(),
                subtypes: ["Vehicle".to_string()].iter().cloned().map(SubType).collect(),
                ..Default::default()
            },
            oracle_text: "Flying\nCrew 1 (Tap any number of creatures you control with total power 1 or more: This Vehicle becomes an artifact creature until end of turn.)\nWhenever Smuggler's Copter attacks or blocks, you may draw a card. If you do, discard a card.".to_string(),
            power: Some(3),
            toughness: Some(3),
            abilities: vec![
                AbilityDefinition::Keyword(KeywordAbility::Flying),
                AbilityDefinition::Keyword(KeywordAbility::Crew(1)),
                AbilityDefinition::Triggered {
                    trigger_condition: TriggerCondition::WhenAttacks,
                    effect: Effect::Sequence(vec![
                        Effect::DrawCards { player: PlayerTarget::Controller, count: EffectAmount::Fixed(1) },
                        Effect::DiscardCards { player: PlayerTarget::Controller, count: EffectAmount::Fixed(1) },
                    ]),
                    intervening_if: None,
                },
                AbilityDefinition::Triggered {
                    trigger_condition: TriggerCondition::WhenBlocks,
                    effect: Effect::Sequence(vec![
                        Effect::DrawCards { player: PlayerTarget::Controller, count: EffectAmount::Fixed(1) },
                        Effect::DiscardCards { player: PlayerTarget::Controller, count: EffectAmount::Fixed(1) },
                    ]),
                    intervening_if: None,
                },
            ],
        },

        // 68. Signal Pest — {1}, Artifact Creature — Pest 0/1;
        //     Battle cry (CR 702.91). Blocking restriction (flying/reach only) deferred — no DSL variant.
        CardDefinition {
            card_id: cid("signal-pest"),
            name: "Signal Pest".to_string(),
            mana_cost: Some(ManaCost { generic: 1, ..Default::default() }),
            types: types_sub(&[CardType::Artifact, CardType::Creature], &["Pest"]),
            oracle_text: "Battle cry (Whenever this creature attacks, each other attacking creature gets +1/+0 until end of turn.)\nThis creature can't be blocked except by creatures with flying or reach."
                .to_string(),
            power: Some(0),
            toughness: Some(1),
            abilities: vec![
                AbilityDefinition::Keyword(KeywordAbility::BattleCry),
                // TODO: blocking restriction ("can't be blocked except by flying/reach") deferred
            ],
        },

        // 69. Ministrant of Obligation — {2}{W}, Creature — Human Cleric 2/1; Afterlife 2.
        CardDefinition {
            card_id: cid("ministrant-of-obligation"),
            name: "Ministrant of Obligation".to_string(),
            mana_cost: Some(ManaCost { generic: 2, white: 1, ..Default::default() }),
            types: creature_types(&["Human", "Cleric"]),
            oracle_text: "Afterlife 2 (When this creature dies, create two 1/1 white and black Spirit creature tokens with flying.)".to_string(),
            power: Some(2),
            toughness: Some(1),
            abilities: vec![
                AbilityDefinition::Keyword(KeywordAbility::Afterlife(2)),
            ],
        },

        // 70. Syndic of Tithes — {1}{W}, Creature — Human Cleric 2/2; Extort.
        CardDefinition {
            card_id: cid("syndic-of-tithes"),
            name: "Syndic of Tithes".to_string(),
            mana_cost: Some(ManaCost { generic: 1, white: 1, ..Default::default() }),
            types: creature_types(&["Human", "Cleric"]),
            oracle_text: "Extort (Whenever you cast a spell, you may pay {W/B}. If you do, each opponent loses 1 life and you gain that much life.)".to_string(),
            power: Some(2),
            toughness: Some(2),
            abilities: vec![
                AbilityDefinition::Keyword(KeywordAbility::Extort),
            ],
        },

        // 71. Boggart Ram-Gang — {R/G}{R/G}{R/G}, Creature — Goblin Warrior 3/3;
        //     Haste. Wither.
        //     Oracle cost is {R/G}{R/G}{R/G} (hybrid); simplified here to {R}{R}{R} because
        //     the ManaCost struct does not support hybrid mana symbols.
        CardDefinition {
            card_id: cid("boggart-ram-gang"),
            name: "Boggart Ram-Gang".to_string(),
            mana_cost: Some(ManaCost { red: 3, ..Default::default() }),
            types: creature_types(&["Goblin", "Warrior"]),
            oracle_text: "Haste\nWither (This deals damage to creatures in the form of -1/-1 counters.)".to_string(),
            power: Some(3),
            toughness: Some(3),
            abilities: vec![
                AbilityDefinition::Keyword(KeywordAbility::Haste),
                // TODO: Add KeywordAbility::Wither variant (CR 702.77a): damage dealt to
                // creatures is in the form of -1/-1 counters instead of marked damage.
            ],
        },

        // 72. Cloudfin Raptor — {U}, Creature — Bird Mutant 0/1;
        //     Flying. Evolve (CR 702.100a).
        CardDefinition {
            card_id: cid("cloudfin-raptor"),
            name: "Cloudfin Raptor".to_string(),
            mana_cost: Some(ManaCost { blue: 1, ..Default::default() }),
            types: creature_types(&["Bird", "Mutant"]),
            oracle_text: "Flying\nEvolve (Whenever a creature with greater power and/or toughness enters the battlefield under your control, put a +1/+1 counter on this creature.)".to_string(),
            power: Some(0),
            toughness: Some(1),
            abilities: vec![
                AbilityDefinition::Keyword(KeywordAbility::Flying),
                AbilityDefinition::Keyword(KeywordAbility::Evolve),
            ],
        },

        // 73. Raffine's Informant — {1}{W}, Creature — Human Wizard 2/1;
        //     When Raffine's Informant enters the battlefield, it connives.
        //     (CR 701.50a)
        CardDefinition {
            card_id: cid("raffines-informant"),
            name: "Raffine's Informant".to_string(),
            mana_cost: Some(ManaCost { generic: 1, white: 1, ..Default::default() }),
            types: creature_types(&["Human", "Wizard"]),
            oracle_text: "When Raffine's Informant enters the battlefield, it connives. (Draw a card, then discard a card. If you discarded a nonland card, put a +1/+1 counter on this creature.)".to_string(),
            power: Some(2),
            toughness: Some(1),
            abilities: vec![
                AbilityDefinition::Triggered {
                    trigger_condition: TriggerCondition::WhenEntersBattlefield,
                    effect: Effect::Connive {
                        target: EffectTarget::Source,
                        count: EffectAmount::Fixed(1),
                    },
                    intervening_if: None,
                },
            ],
        },

        // 74. Wayward Swordtooth — {2}{G}, Creature — Dinosaur 5/5;
        //     Ascend. You may play an additional land on each of your turns.
        //     Wayward Swordtooth can't attack or block unless you have the city's blessing.
        //     (Additional land play and attack/block restriction noted in oracle_text;
        //      Ascend keyword fully modeled.)
        CardDefinition {
            card_id: cid("wayward-swordtooth"),
            name: "Wayward Swordtooth".to_string(),
            mana_cost: Some(ManaCost { generic: 2, green: 1, ..Default::default() }),
            types: creature_types(&["Dinosaur"]),
            oracle_text: "Ascend (If you control ten or more permanents, you get the city's blessing for the rest of the game.)\nYou may play an additional land on each of your turns.\nWayward Swordtooth can't attack or block unless you have the city's blessing.".to_string(),
            power: Some(5),
            toughness: Some(5),
            abilities: vec![
                AbilityDefinition::Keyword(KeywordAbility::Ascend),
            ],
        },

        // ── Replacement-effect cards (M8 Session 6) ──────────────────────────

        // 55. Alela, Cunning Conqueror — {2UB}, Legendary Creature — Faerie Warlock 2/4;
        //     Flying. Whenever you cast your first spell during each opponent's turn,
        //     create a 1/1 black Faerie Rogue creature token with flying. Whenever one or
        //     more Faeries you control deal combat damage to a player, goad target creature
        //     that player controls.
        //
        //     M9.4 improvements from M8 simplifications:
        //     - Trigger 1: WheneverYouCastSpell { during_opponent_turn: true } restricts
        //       the token trigger to opponent turns only (CR 603.1).
        //       "First spell per turn" tracking deferred (requires per-turn state counter).
        //     - Trigger 2: Effect::Goad now implemented; target is the nearest creature
        //       the damaged player controls. Faerie-filtered trigger remains approximated
        //       as WhenDealsCombatDamageToPlayer (fires when Alela deals combat damage).
        //       TODO: Add TriggerCondition::WheneverCreatureTypeYouControlDealsCombatDamage
        //       with a creature-type filter (Session 1 item 6 plan note).
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
                // CR 603.1: fires only during opponent turns (during_opponent_turn: true).
                // "First spell per turn" tracking deferred to later session.
                AbilityDefinition::Triggered {
                    trigger_condition: TriggerCondition::WheneverYouCastSpell {
                        during_opponent_turn: true,
                    },
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
                            mana_abilities: vec![],
                            activated_abilities: vec![],
                        },
                    },
                    intervening_if: None,
                },
                // CR 701.38: Effect::Goad — goad target creature that the damaged player controls.
                // Trigger approximated as WhenDealsCombatDamageToPlayer (fires when Alela
                // itself deals combat damage); Faerie-filtered variant deferred.
                AbilityDefinition::Triggered {
                    trigger_condition: TriggerCondition::WhenDealsCombatDamageToPlayer,
                    effect: Effect::Goad {
                        target: EffectTarget::DeclaredTarget { index: 0 },
                    },
                    intervening_if: None,
                },
            ],
        },


        // 52. Rest in Peace — {1W}, Enchantment.
        //     "When Rest in Peace enters the battlefield, exile all cards from all
        //      graveyards. If a card would be put into a graveyard from anywhere,
        //      exile it instead."
        //
        //     ETB trigger: fires inline via fire_when_enters_triggered_effects at ETB
        //     site (not via the stack), exiling all cards currently in all graveyards.
        //     Ongoing replacement: registered via register_permanent_replacement_abilities.
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
                // CR 603.2: ETB triggered ability — exile all cards from all graveyards.
                // Executed inline (non-interactively) via fire_when_enters_triggered_effects.
                AbilityDefinition::Triggered {
                    trigger_condition: TriggerCondition::WhenEntersBattlefield,
                    effect: Effect::ForEach {
                        over: ForEachTarget::EachCardInAllGraveyards,
                        effect: Box::new(Effect::ExileObject {
                            target: EffectTarget::DeclaredTarget { index: 0 },
                        }),
                    },
                    intervening_if: None,
                },
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
                // CR 113.6b: If Leyline of the Void is in your opening hand, you may begin
                // the game with it on the battlefield. Handled by start_game pre-game check.
                AbilityDefinition::OpeningHand,
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
                // CR 614.1a / 614.15 / 701.20: Self-replacement effect — if this specific
                // Colossus would go to a graveyard, shuffle it into its owner's library.
                // ObjectFilter::Any is replaced with SpecificObject at registration time.
                AbilityDefinition::Replacement {
                    trigger: ReplacementTrigger::WouldChangeZone {
                        from: None,
                        to: ZoneType::Graveyard,
                        filter: ObjectFilter::Any,
                    },
                    modification: ReplacementModification::ShuffleIntoOwnerLibrary,
                    is_self: true,
                },
            ],
        },


        // 55 (additional). Adrix and Nev, Twincasters — {2}{G}{U}, Legendary Creature — Merfolk
        //     Wizard 2/2. Ward {2}. If one or more tokens would be created under your control,
        //     twice that many of those tokens are created instead.
        //
        //     Ward {2} is encoded as KeywordAbility::Ward(2); the triggered ability that counters
        //     spells/abilities targeting this creature unless the opponent pays {2} is generated
        //     automatically from the keyword by state/builder.rs.
        //
        //     The token-doubling ability is a replacement effect (CR 614.1): "If one or more tokens
        //     would be created under your control, twice that many of those tokens are created
        //     instead." ReplacementTrigger::WouldCreateToken does not yet exist in the DSL.
        //     TODO: Add ReplacementTrigger::WouldCreateToken { player_filter: PlayerFilter }
        //     and ReplacementModification::DoubleTokens to replacement_effect.rs, then replace
        //     this TODO with AbilityDefinition::Replacement using those variants.
        CardDefinition {
            card_id: cid("adrix-and-nev-twincasters"),
            name: "Adrix and Nev, Twincasters".to_string(),
            mana_cost: Some(ManaCost { generic: 2, green: 1, blue: 1, ..Default::default() }),
            types: full_types(&[SuperType::Legendary], &[CardType::Creature], &["Merfolk", "Wizard"]),
            oracle_text:
                "Ward {2} (Whenever this creature becomes the target of a spell or ability an opponent controls, counter it unless that player pays {2}.)\n\
                 If one or more tokens would be created under your control, twice that many of those tokens are created instead."
                    .to_string(),
            power: Some(2),
            toughness: Some(2),
            abilities: vec![
                // CR 702.21a: Ward {2} — generates a triggered ability at object-construction
                // time that counters any spell or ability an opponent controls that targets this
                // creature, unless that opponent pays {2}.
                AbilityDefinition::Keyword(KeywordAbility::Ward(2)),
                // TODO: Token-doubling replacement effect — requires ReplacementTrigger::WouldCreateToken
                // and ReplacementModification::DoubleTokens (not yet in DSL). See note above.
            ],
        },

        // 56. Think Twice — {1}{U}, Instant; draw a card. Flashback {2}{U}.
        //
        //     CR 702.34a: "Flashback [cost]" means the card may be cast from its owner's
        //     graveyard by paying [cost] rather than its mana cost. If the flashback cost was
        //     paid, exile this card instead of putting it anywhere else when it leaves the stack.
        //
        //     Two abilities encode flashback:
        //     1. AbilityDefinition::Keyword(KeywordAbility::Flashback) — marker for quick
        //        presence-checking in casting.rs (zone validation, cost lookup).
        //     2. AbilityDefinition::Flashback { cost } — stores the alternative cost {2}{U}.
        CardDefinition {
            card_id: cid("think-twice"),
            name: "Think Twice".to_string(),
            mana_cost: Some(ManaCost { generic: 1, blue: 1, ..Default::default() }),
            types: types(&[CardType::Instant]),
            oracle_text: "Draw a card.\nFlashback {2}{U} (You may cast this card from your graveyard for its flashback cost. Then exile it.)".to_string(),
            abilities: vec![
                // CR 702.34a: Flashback marker — enables casting from graveyard in casting.rs.
                AbilityDefinition::Keyword(KeywordAbility::Flashback),
                // CR 702.34a: The flashback cost itself ({2}{U}).
                AbilityDefinition::Flashback {
                    cost: ManaCost { generic: 2, blue: 1, ..Default::default() },
                },
                // The spell effect: draw a card for the controller.
                AbilityDefinition::Spell {
                    effect: Effect::DrawCards {
                        player: PlayerTarget::Controller,
                        count: EffectAmount::Fixed(1),
                    },
                    targets: vec![],
                    modes: None,
                    cant_be_countered: false,
                },
            ],
            ..Default::default()
        },

        // 57. Faithless Looting — {R}, Sorcery; draw two cards, then discard two cards.
        //     Flashback {2}{R}.
        //
        //     CR 702.34a: Sorcery with flashback — can be cast from graveyard at sorcery speed
        //     by paying {2}{R}. Exiled on any stack departure when cast via flashback.
        //
        //     Note: Faithless Looting is banned in Modern but legal in Commander. It is a
        //     Commander staple and an ideal test card for sorcery-speed flashback.
        CardDefinition {
            card_id: cid("faithless-looting"),
            name: "Faithless Looting".to_string(),
            mana_cost: Some(ManaCost { red: 1, ..Default::default() }),
            types: types(&[CardType::Sorcery]),
            oracle_text: "Draw two cards, then discard two cards.\nFlashback {2}{R} (You may cast this card from your graveyard for its flashback cost. Then exile it.)".to_string(),
            abilities: vec![
                // CR 702.34a: Flashback marker — enables casting from graveyard in casting.rs.
                AbilityDefinition::Keyword(KeywordAbility::Flashback),
                // CR 702.34a: The flashback cost itself ({2}{R}).
                AbilityDefinition::Flashback {
                    cost: ManaCost { generic: 2, red: 1, ..Default::default() },
                },
                // The spell effect: draw 2 cards, then discard 2 cards.
                AbilityDefinition::Spell {
                    effect: Effect::Sequence(vec![
                        Effect::DrawCards {
                            player: PlayerTarget::Controller,
                            count: EffectAmount::Fixed(2),
                        },
                        Effect::DiscardCards {
                            player: PlayerTarget::Controller,
                            count: EffectAmount::Fixed(2),
                        },
                    ]),
                    targets: vec![],
                    modes: None,
                    cant_be_countered: false,
                },
            ],
            ..Default::default()
        },

        // 60. Golgari Grave-Troll — {4G}, Creature — Troll Skeleton 0/4.
        //     "This creature enters with a +1/+1 counter on it for each creature card in
        //      your graveyard. {1}, Remove a +1/+1 counter from this creature: Regenerate
        //      this creature. Dredge 6."
        //
        //     Simplifications for this milestone:
        //     - Power is fixed at 0 (real card is 0/0 and tracks counters; counter-based
        //       P/T requires a continuous layer-3 effect, deferred).
        //     - ETB counter placement (one per creature card in graveyard) deferred — needs
        //       a TriggeredEffect that counts graveyard contents at resolution.
        //     - Regeneration cost deferred — requires AbilityDefinition::Activated with a
        //       RemoveCounter cost variant that does not yet exist in the DSL.
        //
        //     CR 702.52a: Dredge N — if you would draw a card, you may instead mill N cards
        //     and return this card from your graveyard to your hand. Functions only while
        //     this card is in the graveyard. Requires >= N cards in library (CR 702.52b).
        //
        //     TODO: AbilityDefinition::Dredge { amount } does not exist in card_definition.rs;
        //     once added (paired with KeywordAbility::Dredge marker, following the Flashback
        //     pattern), replace this keyword-only encoding with the full two-ability form.
        CardDefinition {
            card_id: cid("golgari-grave-troll"),
            name: "Golgari Grave-Troll".to_string(),
            mana_cost: Some(ManaCost { generic: 4, green: 1, ..Default::default() }),
            types: creature_types(&["Troll", "Skeleton"]),
            oracle_text:
                "This creature enters with a +1/+1 counter on it for each creature card in your graveyard.\n\
                 {1}, Remove a +1/+1 counter from this creature: Regenerate this creature.\n\
                 Dredge 6 (If you would draw a card, you may mill six cards instead. If you do, return this card from your graveyard to your hand.)"
                    .to_string(),
            power: Some(0),
            toughness: Some(4),
            abilities: vec![
                // CR 702.52a: Dredge 6 marker — checked in drawing logic to offer the
                // draw-replacement option when this card is in its owner's graveyard.
                AbilityDefinition::Keyword(KeywordAbility::Dredge(6)),
            ],
        },

        // 61. Rancor — {G}, Enchantment — Aura.
        //     "Enchant creature. Enchanted creature gets +2/+0 and has trample.
        //      When Rancor is put into a graveyard from the battlefield, return
        //      Rancor to its owner's hand."
        //
        //     CR 702.5a: Enchant creature — restricts casting target and legal attachments.
        //     CR 613.4c: +2/+0 is a layer-7c P/T-modifying effect.
        //     CR 702.19a: Trample keyword granted in layer 6.
        //     CR 603.1: "When Rancor is put into a graveyard from the battlefield" is a
        //     triggered ability that fires on the zone-change event.
        CardDefinition {
            card_id: cid("rancor"),
            name: "Rancor".to_string(),
            mana_cost: Some(ManaCost { green: 1, ..Default::default() }),
            types: types_sub(&[CardType::Enchantment], &["Aura"]),
            oracle_text: "Enchant creature\nEnchanted creature gets +2/+0 and has trample.\nWhen Rancor is put into a graveyard from the battlefield, return Rancor to its owner's hand.".to_string(),
            abilities: vec![
                // CR 702.5a: Enchant creature — defines legal targets and attachment restriction.
                AbilityDefinition::Keyword(KeywordAbility::Enchant(EnchantTarget::Creature)),
                // CR 613.4c: Enchanted creature gets +2/+0 (layer 7c, P/T modify).
                AbilityDefinition::Static {
                    continuous_effect: ContinuousEffectDef {
                        layer: EffectLayer::PtModify,
                        modification: LayerModification::ModifyPower(2),
                        filter: EffectFilter::AttachedCreature,
                        duration: EffectDuration::WhileSourceOnBattlefield,
                    },
                },
                // CR 702.19a: Enchanted creature has trample (layer 6, ability grant).
                AbilityDefinition::Static {
                    continuous_effect: ContinuousEffectDef {
                        layer: EffectLayer::Ability,
                        modification: LayerModification::AddKeyword(KeywordAbility::Trample),
                        filter: EffectFilter::AttachedCreature,
                        duration: EffectDuration::WhileSourceOnBattlefield,
                    },
                },
                // CR 603.1: When Rancor is put into a graveyard from the battlefield,
                // return it to its owner's hand. This trigger fires on the WhenDies
                // zone-change event (battlefield → graveyard).
                AbilityDefinition::Triggered {
                    trigger_condition: TriggerCondition::WhenDies,
                    effect: Effect::MoveZone {
                        target: EffectTarget::Source,
                        to: ZoneTarget::Hand { owner: PlayerTarget::Controller },
                    },
                    intervening_if: None,
                },
            ],
            ..Default::default()
        },

        // ── Kicker cards ─────────────────────────────────────────────────────────

        // Burst Lightning {R}
        // Instant — Kicker {4}
        // Burst Lightning deals 2 damage to any target. If this spell was kicked,
        // it deals 4 damage instead.
        // CR 702.33a: Kicker [cost] — optional additional cost for enhanced effect.
        // CR 702.33d: "kicked" means the player paid the kicker cost at cast time.
        CardDefinition {
            card_id: cid("burst-lightning"),
            name: "Burst Lightning".to_string(),
            mana_cost: Some(ManaCost { red: 1, ..Default::default() }),
            types: types(&[CardType::Instant]),
            oracle_text: "Kicker {4}\nBurst Lightning deals 2 damage to any target. If this spell was kicked, it deals 4 damage instead.".to_string(),
            abilities: vec![
                AbilityDefinition::Keyword(KeywordAbility::Kicker),
                AbilityDefinition::Kicker {
                    cost: ManaCost { generic: 4, ..Default::default() },
                    is_multikicker: false,
                },
                AbilityDefinition::Spell {
                    effect: Effect::Conditional {
                        condition: Condition::WasKicked,
                        if_true: Box::new(Effect::DealDamage {
                            target: EffectTarget::DeclaredTarget { index: 0 },
                            amount: EffectAmount::Fixed(4),
                        }),
                        if_false: Box::new(Effect::DealDamage {
                            target: EffectTarget::DeclaredTarget { index: 0 },
                            amount: EffectAmount::Fixed(2),
                        }),
                    },
                    targets: vec![TargetRequirement::TargetAny],
                    modes: None,
                    cant_be_countered: false,
                },
            ],
            ..Default::default()
        },

        // Torch Slinger {2}{R}
        // Creature — Goblin Shaman, 2/2
        // Kicker {1}{R}
        // When Torch Slinger enters, if it was kicked, it deals 2 damage to
        // target creature an opponent controls.
        // CR 702.33e: Linked abilities — the ETB trigger is linked to the kicker
        // and only fires when the permanent was kicked.
        CardDefinition {
            card_id: cid("torch-slinger"),
            name: "Torch Slinger".to_string(),
            mana_cost: Some(ManaCost { generic: 2, red: 1, ..Default::default() }),
            types: TypeLine {
                card_types: [CardType::Creature].into_iter().collect(),
                subtypes: [SubType("Goblin".to_string()), SubType("Shaman".to_string())].into_iter().collect(),
                ..Default::default()
            },
            power: Some(2),
            toughness: Some(2),
            oracle_text: "Kicker {1}{R}\nWhen Torch Slinger enters, if it was kicked, it deals 2 damage to target creature.".to_string(),
            abilities: vec![
                AbilityDefinition::Keyword(KeywordAbility::Kicker),
                AbilityDefinition::Kicker {
                    cost: ManaCost { generic: 1, red: 1, ..Default::default() },
                    is_multikicker: false,
                },
                AbilityDefinition::Triggered {
                    trigger_condition: TriggerCondition::WhenEntersBattlefield,
                    effect: Effect::Conditional {
                        condition: Condition::WasKicked,
                        if_true: Box::new(Effect::DealDamage {
                            target: EffectTarget::DeclaredTarget { index: 0 },
                            amount: EffectAmount::Fixed(2),
                        }),
                        if_false: Box::new(Effect::Sequence(vec![])),
                    },
                    intervening_if: None,
                },
            ],
        },

        // ── Flashback cards ───────────────────────────────────────────────────────────

        // Strike It Rich {R}
        // Sorcery
        // Create a Treasure token. (It's an artifact with "{T}, Sacrifice this token: Add one mana of any color.")
        // Flashback {2}{R} (You may cast this card from your graveyard for its flashback cost. Then exile it.)
        //
        // CR 702.34a: Flashback marker enables casting from graveyard. The flashback cost {2}{R}
        // replaces the mana cost when cast via flashback. The card is exiled on any stack departure
        // when cast via flashback.
        // CR 111.10a: Treasure token — colorless Artifact — Treasure with mana ability.

        // 71. Ox of Agonas — {3}{R}{R}, Creature — Ox 4/2; ETB: discard hand, draw 3.
        //     Escape — {R}{R}, Exile eight other cards from your graveyard.
        //
        //     ETB effect approximated: DiscardCards up to 7 then DrawCards 3 (interactive
        //     hand discard deferred). Escape cost and exile count accurate per oracle text.
        CardDefinition {
            card_id: cid("ox-of-agonas"),
            name: "Ox of Agonas".to_string(),
            mana_cost: Some(ManaCost { generic: 3, red: 2, ..Default::default() }),
            types: creature_types(&["Ox"]),
            oracle_text: "When Ox of Agonas enters the battlefield, discard your hand, then draw three cards.\nEscape — {R}{R}, Exile eight other cards from your graveyard. (You may cast this card from your graveyard for its escape cost.)".to_string(),
            power: Some(4),
            toughness: Some(2),
            abilities: vec![
                // CR 702.138a: Escape keyword marker for quick presence-check.
                AbilityDefinition::Keyword(KeywordAbility::Escape),
                // CR 702.138a: Escape cost ({R}{R}) and exile count (8).
                AbilityDefinition::Escape {
                    cost: ManaCost { red: 2, ..Default::default() },
                    exile_count: 8,
                },
                // ETB: discard hand then draw 3.
                AbilityDefinition::Triggered {
                    trigger_condition: TriggerCondition::WhenEntersBattlefield,
                    intervening_if: None,
                    effect: Effect::Sequence(vec![
                        Effect::DiscardCards {
                            player: PlayerTarget::Controller,
                            count: EffectAmount::Fixed(7),
                        },
                        Effect::DrawCards {
                            player: PlayerTarget::Controller,
                            count: EffectAmount::Fixed(3),
                        },
                    ]),
                },
            ],
        },

        // 72. Terminus — {4}{W}{W}, Sorcery; Put all creatures on the bottom of their owners'
        //     libraries. Miracle {W} (You may cast this card for its miracle cost. Cast it only
        //     as the first card you drew this turn.)
        //
        //     Effect approximation: destroys all creatures (engine does not yet support
        //     "put on bottom of owner's library" as a ForEach with per-creature owner
        //     routing). Terminus is included primarily to validate the Miracle keyword
        //     (CR 702.94). The full oracle text and mana cost are correct.
        CardDefinition {
            card_id: cid("terminus"),
            name: "Terminus".to_string(),
            mana_cost: Some(ManaCost { generic: 4, white: 2, ..Default::default() }),
            types: types(&[CardType::Sorcery]),
            oracle_text: "Put all creatures on the bottom of their owners' libraries.\nMiracle {W} (You may cast this card for its miracle cost. Cast it only as the first card you drew this turn.)".to_string(),
            abilities: vec![
                // CR 702.94a: Miracle keyword marker.
                AbilityDefinition::Keyword(KeywordAbility::Miracle),
                // CR 702.94a: The miracle alternative cost ({W}).
                AbilityDefinition::Miracle {
                    cost: ManaCost { white: 1, ..Default::default() },
                },
                // The spell effect: destroy all creatures (approximates "put on bottom of
                // their owners' libraries" — owners' library routing deferred to M10+).
                AbilityDefinition::Spell {
                    effect: Effect::DestroyPermanent {
                        target: EffectTarget::AllCreatures,
                    },
                    targets: vec![],
                    modes: None,
                    cant_be_countered: false,
                },
            ],
            ..Default::default()
        },

        CardDefinition {
            card_id: cid("strike-it-rich"),
            name: "Strike It Rich".to_string(),
            mana_cost: Some(ManaCost { red: 1, ..Default::default() }),
            types: types(&[CardType::Sorcery]),
            oracle_text: "Create a Treasure token. (It's an artifact with \"{T}, Sacrifice this token: Add one mana of any color.\")\nFlashback {2}{R} (You may cast this card from your graveyard for its flashback cost. Then exile it.)".to_string(),
            abilities: vec![
                // CR 702.34a: Flashback marker — enables casting from graveyard in casting.rs.
                AbilityDefinition::Keyword(KeywordAbility::Flashback),
                // CR 702.34a: The flashback cost itself ({2}{R}).
                AbilityDefinition::Flashback {
                    cost: ManaCost { generic: 2, red: 1, ..Default::default() },
                },
                // The spell effect: create one Treasure token.
                AbilityDefinition::Spell {
                    effect: Effect::CreateToken {
                        spec: treasure_token_spec(1),
                    },
                    targets: vec![],
                    modes: None,
                    cant_be_countered: false,
                },
            ],
            ..Default::default()
        },

        // 73. Saw It Coming — {2}{U}{U}, Instant; Counter target spell.
        //     Foretell {1}{U} (During your turn, you may pay {2} and exile this card from
        //     your hand face down. Cast it on a future turn for its foretell cost.)
        CardDefinition {
            card_id: cid("saw-it-coming"),
            name: "Saw It Coming".to_string(),
            mana_cost: Some(ManaCost { generic: 2, blue: 2, ..Default::default() }),
            types: types(&[CardType::Instant]),
            oracle_text: "Counter target spell.\nForetell {1}{U} (During your turn, you may pay {2} and exile this card from your hand face down. Cast it on a future turn for its foretell cost.)".to_string(),
            abilities: vec![
                // CR 702.143a: Foretell keyword marker.
                AbilityDefinition::Keyword(KeywordAbility::Foretell),
                // CR 702.143a: Foretell cost ({1}{U}).
                AbilityDefinition::Foretell {
                    cost: ManaCost { generic: 1, blue: 1, ..Default::default() },
                },
                // Spell effect: counter target spell.
                AbilityDefinition::Spell {
                    effect: Effect::CounterSpell {
                        target: EffectTarget::DeclaredTarget { index: 0 },
                    },
                    targets: vec![TargetRequirement::TargetSpell],
                    modes: None,
                    cant_be_countered: false,
                },
            ],
            ..Default::default()
        },

        // 74. Dregscape Zombie — {1}{B}, Creature — Zombie 2/1;
        //     Unearth {B} (Pay {B}: Return this card from your graveyard to the battlefield.
        //     It gains haste. Exile it at the beginning of the next end step or if it would
        //     leave the battlefield. Unearth only as a sorcery.)
        CardDefinition {
            card_id: cid("dregscape-zombie"),
            name: "Dregscape Zombie".to_string(),
            mana_cost: Some(ManaCost { generic: 1, black: 1, ..Default::default() }),
            types: creature_types(&["Zombie"]),
            oracle_text: "Unearth {B} (Pay {B}: Return this card from your graveyard to the battlefield. It gains haste. Exile it at the beginning of the next end step or if it would leave the battlefield. Unearth only as a sorcery.)".to_string(),
            power: Some(2),
            toughness: Some(1),
            abilities: vec![
                // CR 702.84a: Unearth keyword marker for quick presence-check.
                AbilityDefinition::Keyword(KeywordAbility::Unearth),
                // CR 702.84a: Unearth cost ({B}).
                AbilityDefinition::Unearth {
                    cost: ManaCost { black: 1, ..Default::default() },
                },
            ],
        },

        // 75. Frogmite — {4}, Artifact Creature — Frog 2/2; Affinity for artifacts.
        //     (This spell costs {1} less to cast for each artifact you control.)
        //     With 4 artifacts controlled, Frogmite costs {0} to cast.
        CardDefinition {
            card_id: cid("frogmite"),
            name: "Frogmite".to_string(),
            mana_cost: Some(ManaCost { generic: 4, ..Default::default() }),
            types: {
                let mut tl = creature_types(&["Frog"]);
                tl.card_types.insert(CardType::Artifact);
                tl
            },
            oracle_text: "Affinity for artifacts (This spell costs {1} less to cast for each artifact you control.)".to_string(),
            power: Some(2),
            toughness: Some(2),
            abilities: vec![
                // CR 702.41a: Affinity for artifacts — costs {1} less for each artifact controlled.
                AbilityDefinition::Keyword(KeywordAbility::Affinity(AffinityTarget::Artifacts)),
            ],
        },

        // 79. Qarsi Sadist — {1}{B}, Creature — Human Cleric 1/3; Exploit.
        //     CR 702.110a: When this enters, you may sacrifice a creature.
        CardDefinition {
            card_id: cid("qarsi-sadist"),
            name: "Qarsi Sadist".to_string(),
            mana_cost: Some(ManaCost { generic: 1, black: 1, ..Default::default() }),
            types: types_sub(&[CardType::Creature], &["Human", "Cleric"]),
            oracle_text: "Exploit (When this creature enters the battlefield, you may sacrifice a creature.)".to_string(),
            power: Some(1),
            toughness: Some(3),
            abilities: vec![
                AbilityDefinition::Keyword(KeywordAbility::Exploit),
            ],
        },

        // 78. Zhur-Taa Goblin — {R}{G}, Creature — Goblin Berserker 2/2; Riot.
        //     CR 702.136a: As this enters, choose +1/+1 counter OR haste.
        CardDefinition {
            card_id: cid("zhur-taa-goblin"),
            name: "Zhur-Taa Goblin".to_string(),
            mana_cost: Some(ManaCost { red: 1, green: 1, ..Default::default() }),
            types: types_sub(&[CardType::Creature], &["Goblin", "Berserker"]),
            oracle_text: "Riot (As this enters, choose to have it enter with a +1/+1 counter or gain haste.)".to_string(),
            power: Some(2),
            toughness: Some(2),
            abilities: vec![
                AbilityDefinition::Keyword(KeywordAbility::Riot),
            ],
        },

        // 77. Marchesa's Emissary — {3}{U}, Creature — Human Rogue 2/2; Hexproof, Dethrone.
        //     Dethrone: CR 702.105 — whenever this attacks the player with most life (or tied),
        //     put a +1/+1 counter on it.
        CardDefinition {
            card_id: cid("marchesas-emissary"),
            name: "Marchesa's Emissary".to_string(),
            mana_cost: Some(ManaCost { generic: 3, blue: 1, ..Default::default() }),
            types: types_sub(&[CardType::Creature], &["Human", "Rogue"]),
            oracle_text: "Hexproof\nDethrone".to_string(),
            power: Some(2),
            toughness: Some(2),
            abilities: vec![
                AbilityDefinition::Keyword(KeywordAbility::Hexproof),
                AbilityDefinition::Keyword(KeywordAbility::Dethrone),
            ],
        },

        // 76. Sublime Exhalation — {6}{W}, Sorcery; Undaunted (This spell costs {1} less to
        //     cast for each of your opponents.)
        //     Destroy all creatures.
        //     In a 4-player Commander game (3 opponents), costs {3}{W} instead of {6}{W}.
        CardDefinition {
            card_id: cid("sublime-exhalation"),
            name: "Sublime Exhalation".to_string(),
            mana_cost: Some(ManaCost { generic: 6, white: 1, ..Default::default() }),
            types: types(&[CardType::Sorcery]),
            oracle_text: "Undaunted (This spell costs {1} less to cast for each of your opponents.)\nDestroy all creatures.".to_string(),
            abilities: vec![
                // CR 702.125a: Undaunted keyword marker.
                AbilityDefinition::Keyword(KeywordAbility::Undaunted),
                // Spell effect: destroy all creatures.
                AbilityDefinition::Spell {
                    effect: Effect::DestroyPermanent {
                        target: EffectTarget::AllCreatures,
                    },
                    targets: vec![],
                    modes: None,
                    cant_be_countered: false,
                },
            ],
            ..Default::default()
        },

        // ── Buyback cards ────────────────────────────────────────────────────────────

        // Searing Touch — {R}, Instant; Buyback {4}; deals 1 damage to any target.
        //
        // CR 702.27a: Buyback — you may pay an additional {4} as you cast this spell.
        // If you do, put this card into your hand as it resolves instead of the graveyard.
        //
        // TODO: KeywordAbility::Buyback does not yet exist in state/mod.rs; add it and
        // pair AbilityDefinition::Keyword(KeywordAbility::Buyback) here for quick
        // presence-checking (see card_definition.rs line ~243).
        CardDefinition {
            card_id: cid("searing-touch"),
            name: "Searing Touch".to_string(),
            mana_cost: Some(ManaCost { red: 1, ..Default::default() }),
            types: types(&[CardType::Instant]),
            oracle_text: "Buyback {4} (You may pay an additional {4} as you cast this spell. If you do, put this card into your hand as it resolves.)\nSearing Touch deals 1 damage to any target.".to_string(),
            abilities: vec![
                // CR 702.27a: Buyback cost ({4}).
                AbilityDefinition::Buyback {
                    cost: ManaCost { generic: 4, ..Default::default() },
                },
                // Spell effect: deal 1 damage to any target.
                AbilityDefinition::Spell {
                    effect: Effect::DealDamage {
                        target: EffectTarget::DeclaredTarget { index: 0 },
                        amount: EffectAmount::Fixed(1),
                    },
                    targets: vec![TargetRequirement::TargetAny],
                    modes: None,
                    cant_be_countered: false,
                },
            ],
            ..Default::default()
        },

    ]
}
