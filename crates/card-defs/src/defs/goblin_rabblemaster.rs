// Goblin Rabblemaster — {2}{R}, Creature — Goblin Warrior 2/2
// Other Goblin creatures you control attack each combat if able.
// At the beginning of combat on your turn, create a 1/1 red Goblin creature token with haste.
// Whenever this creature attacks, it gets +1/+0 until end of turn for each other attacking Goblin.
use crate::cards::helpers::*;

pub fn card() -> CardDefinition {
    CardDefinition {
        card_id: cid("goblin-rabblemaster"),
        name: "Goblin Rabblemaster".to_string(),
        mana_cost: Some(ManaCost {
            generic: 2,
            red: 1,
            ..Default::default()
        }),
        types: creature_types(&["Goblin", "Warrior"]),
        oracle_text: "Other Goblin creatures you control attack each combat if able.\nAt the \
                      beginning of combat on your turn, create a 1/1 red Goblin creature token \
                      with haste.\nWhenever Goblin Rabblemaster attacks, it gets +1/+0 until end \
                      of turn for each other attacking Goblin."
            .to_string(),
        power: Some(2),
        toughness: Some(2),
        abilities: vec![
            // CR 508.1d / 613.1f: "Other Goblin creatures you control attack each combat
            // if able." Layer 6 static grant, exactly the galadhrim_brigade.rs /
            // camellia_the_seedmiser.rs OtherCreaturesYouControlWithSubtype idiom, swapping
            // the modification for AddKeyword(MustAttackEachCombat). combat.rs's must-attack
            // enforcement (:378-390) reads layer-resolved characteristics
            // (expect_characteristics) for every battlefield object the active player
            // controls, not just the source's own printed keyword list, so this composes
            // cleanly for non-source objects -- probe-verified in
            // tests/primitives/pb_rs3_rabblemaster_mustattack_probe.rs (PB-RS3 / F-Rabble).
            //
            // Accepted engine-wide limitation (PB-RS3 review Finding 1, inherited, not
            // introduced here): combat.rs:421-424's must-attack "able" test computes
            // `cannot_attack` from only tapped/vigilance, summoning-sickness/haste,
            // Defender, and CantAttackOwner -- it never reads GameRestriction::
            // CantAttackYouUnlessPay, even though that restriction IS fully enforced at
            // combat.rs:185-224 (a declaration the player can't pay the tax for is
            // rejected there). The precondition is narrower than "a Ghostly Prison is
            // out": tax_per_attacker is keyed PER DEFENDING PLAYER (combat.rs:~200), so in
            // 4-player Commander the forced token can simply attack an untaxed opponent.
            // A deadlock needs EVERY remaining viable opponent taxed and the attacking
            // player unable to pay -- realistic late-game, but not the common case. When
            // that holds, declaring this creature's forced Goblin token is illegal (tax
            // check) AND omitting it is illegal (must-attack check) simultaneously, which
            // is a genuine deadlock (CR 508.1d + the 2014-07-18 Rabblemaster
            // ruling: "If there's a cost associated with having a creature attack, you're
            // not forced to pay that cost"). This is shared by every already-shipped
            // MustAttackEachCombat card, not new to this def, but this card manufactures a
            // forced attacker every single combat, so the gap is reachable every turn
            // rather than only when a fixed forced-attacker happens to be present. Filed as
            // OOS-RS3-4 (memory/primitives/rider-seed-triage-2026-07-19.md); not fixed
            // here -- engine change is out of scope for this PB.
            AbilityDefinition::Static {
                continuous_effect: ContinuousEffectDef {
                    layer: EffectLayer::Ability,
                    modification: LayerModification::AddKeyword(
                        KeywordAbility::MustAttackEachCombat,
                    ),
                    filter: EffectFilter::OtherCreaturesYouControlWithSubtype(SubType(
                        "Goblin".to_string(),
                    )),
                    duration: EffectDuration::WhileSourceOnBattlefield,
                    condition: None,
                },
            },
            AbilityDefinition::Triggered {
                once_per_turn: false,
                trigger_condition: TriggerCondition::AtBeginningOfCombat,
                effect: Effect::CreateToken {
                    spec: TokenSpec {
                        name: "Goblin".to_string(),
                        card_types: [CardType::Creature].into_iter().collect(),
                        subtypes: [SubType("Goblin".to_string())].into_iter().collect(),
                        colors: [Color::Red].into_iter().collect(),
                        power: 1,
                        toughness: 1,
                        count: EffectAmount::Fixed(1),
                        supertypes: imbl::OrdSet::new(),
                        keywords: [KeywordAbility::Haste].into_iter().collect(),
                        tapped: false,
                        enters_attacking: false,
                        mana_color: None,
                        mana_abilities: vec![],
                        activated_abilities: vec![],
                        ..Default::default()
                    },
                },
                intervening_if: None,
                targets: vec![],

                modes: None,
                trigger_zone: None,
            },
            // CR 508.1m: "Whenever Goblin Rabblemaster attacks, it gets +1/+0 until end of
            // turn for each other attacking Goblin." Single (x1, no Sum) analogue of Goblin
            // Piledriver's x2 pump — see goblin_piledriver.rs for the shape rationale.
            // controller: EachPlayer (not Controller) is the CR-correct "each other attacking
            // Goblin" scope (any controller), matching the Commissar Severina Raine precedent;
            // identical to Controller-only in normal single-attacker combat, but EachPlayer is
            // the safe general reading. exclude_self excludes Rabblemaster itself via
            // ctx.source (WhenAttacks -> source is this creature).
            AbilityDefinition::Triggered {
                once_per_turn: false,
                trigger_condition: TriggerCondition::WhenAttacks,
                effect: Effect::ApplyContinuousEffect {
                    effect_def: Box::new(ContinuousEffectDef {
                        layer: EffectLayer::PtModify,
                        modification: LayerModification::ModifyPowerDynamic {
                            amount: Box::new(EffectAmount::AttackingCreatureCount {
                                controller: PlayerTarget::EachPlayer,
                                filter: Some(TargetFilter {
                                    has_subtype: Some(SubType("Goblin".to_string())),
                                    exclude_self: true,
                                    ..Default::default()
                                }),
                            }),
                            negate: false,
                        },
                        filter: EffectFilter::Source,
                        duration: EffectDuration::UntilEndOfTurn,
                        condition: None,
                    }),
                },
                intervening_if: None,
                targets: vec![],

                modes: None,
                trigger_zone: None,
            },
        ],
        completeness: Completeness::Complete,
        ..Default::default()
    }
}
