/// View model types for the replay viewer frontend.
///
/// This module converts `GameState` (engine internals) into a UI-friendly JSON
/// shape. The frontend never sees raw `GameState`. This decoupling layer means
/// engine internal type changes do not break the frontend.
///
/// The same view model shape is reused in the M11 Tauri app.
use std::collections::HashMap;

use mtg_engine::{
    calculate_characteristics, AffinityTarget, AttackTarget, CombatState, CounterType, GameState,
    KeywordAbility, ObjectId, PlayerId, StackObjectKind, ZoneId,
};
use serde::Serialize;

// ── Top-level step view model ─────────────────────────────────────────────────

/// Top-level view model returned by `GET /api/step/:n`.
#[derive(Debug, Serialize)]
pub struct StepViewModel {
    pub index: usize,
    pub total_steps: usize,
    pub script_action: serde_json::Value,
    pub command: Option<serde_json::Value>,
    pub events: Vec<serde_json::Value>,
    pub state: StateViewModel,
    pub assertions: Option<Vec<AssertionResultView>>,
}

/// A single assertion result, UI-friendly.
#[derive(Debug, Serialize)]
pub struct AssertionResultView {
    pub path: String,
    pub expected: serde_json::Value,
    pub actual: serde_json::Value,
    pub passed: bool,
}

// ── State view model ──────────────────────────────────────────────────────────

/// Game state shaped for UI consumption.
#[derive(Debug, Serialize)]
pub struct StateViewModel {
    pub turn: TurnView,
    pub players: HashMap<String, PlayerView>,
    pub zones: ZonesView,
    pub combat: Option<CombatView>,
}

/// Turn/phase/step state for the UI.
#[derive(Debug, Serialize)]
pub struct TurnView {
    pub number: u32,
    pub active_player: String,
    pub phase: String,
    pub step: String,
    pub priority: Option<String>,
}

/// Per-player state for the UI.
#[derive(Debug, Serialize)]
pub struct PlayerView {
    pub life: i32,
    pub poison: u32,
    pub mana_pool: ManaPoolView,
    pub hand_size: usize,
    pub library_size: usize,
    pub graveyard_size: usize,
    /// Outer key: opponent name. Inner key: that opponent's commander card name.
    pub commander_damage_received: HashMap<String, HashMap<String, u32>>,
    pub land_plays_remaining: u32,
    pub has_lost: bool,
    pub has_conceded: bool,
    /// CR 702.131c: the city's blessing is a permanent designation once gained.
    pub has_citys_blessing: bool,
}

/// Mana pool state for the UI.
#[derive(Debug, Serialize)]
pub struct ManaPoolView {
    pub white: u32,
    pub blue: u32,
    pub black: u32,
    pub red: u32,
    pub green: u32,
    pub colorless: u32,
}

/// A permanent on the battlefield.
#[derive(Debug, Serialize)]
pub struct PermanentView {
    pub object_id: u64,
    pub name: String,
    pub card_types: Vec<String>,
    pub subtypes: Vec<String>,
    pub supertypes: Vec<String>,
    pub tapped: bool,
    pub summoning_sick: bool,
    pub power: Option<i32>,
    pub toughness: Option<i32>,
    pub counters: HashMap<String, u32>,
    pub damage_marked: u32,
    pub attached_to: Option<u64>,
    pub attachments: Vec<u64>,
    pub is_commander: bool,
    pub is_token: bool,
    pub keywords: Vec<String>,
    pub controller: String,
}

/// An item on the stack.
#[derive(Debug, Serialize)]
pub struct StackItemView {
    pub id: u64,
    pub controller: String,
    /// "spell", "activated_ability", "triggered_ability", "cascade_trigger", "storm_trigger"
    pub kind: String,
    /// Source card name, looked up from objects map.
    pub source_name: String,
    /// Human-readable target descriptions.
    pub targets: Vec<String>,
    pub is_copy: bool,
}

/// A card in a non-battlefield zone (hand, graveyard, exile, command).
#[derive(Debug, Serialize)]
pub struct CardInZoneView {
    pub object_id: u64,
    pub name: String,
    pub card_types: Vec<String>,
}

/// All zones in a UI-friendly format.
#[derive(Debug, Serialize)]
pub struct ZonesView {
    /// Per-player battlefield: player_name -> list of permanents they control.
    pub battlefield: HashMap<String, Vec<PermanentView>>,
    /// Per-player hand: player_name -> list of cards.
    pub hand: HashMap<String, Vec<CardInZoneView>>,
    /// Per-player graveyard: player_name -> list of cards (top first).
    pub graveyard: HashMap<String, Vec<CardInZoneView>>,
    /// Shared exile pile.
    pub exile: Vec<CardInZoneView>,
    /// Per-player command zone: player_name -> list of cards.
    pub command_zone: HashMap<String, Vec<CardInZoneView>>,
    /// Stack in LIFO order (last element = top of stack).
    pub stack: Vec<StackItemView>,
}

/// Combat state for the UI (only present when in combat).
#[derive(Debug, Serialize)]
pub struct CombatView {
    pub attacking_player: String,
    pub attackers: Vec<AttackerView>,
}

/// An attacking creature with its target.
#[derive(Debug, Serialize)]
pub struct AttackerView {
    pub object_id: u64,
    pub name: String,
    /// "player:<name>" or "planeswalker:<id>"
    pub target: String,
    pub blockers: Vec<BlockerView>,
}

/// A blocking creature.
#[derive(Debug, Serialize)]
pub struct BlockerView {
    pub object_id: u64,
    pub name: String,
}

// ── StateViewModel::from_game_state ───────────────────────────────────────────

impl StateViewModel {
    /// Convert a `GameState` to a UI-friendly `StateViewModel`.
    pub fn from_game_state(state: &GameState, player_names: &HashMap<PlayerId, String>) -> Self {
        let turn = build_turn_view(state, player_names);
        let players = build_players_view(state, player_names);
        let zones = build_zones_view(state, player_names);
        let combat = state
            .combat
            .as_ref()
            .map(|c| build_combat_view(c, state, player_names));

        StateViewModel {
            turn,
            players,
            zones,
            combat,
        }
    }
}

// ── Helper: resolve PlayerId → name ──────────────────────────────────────────

fn player_name(pid: PlayerId, player_names: &HashMap<PlayerId, String>) -> String {
    player_names
        .get(&pid)
        .cloned()
        .unwrap_or_else(|| format!("player_{}", pid.0))
}

// ── Helper: TurnView ─────────────────────────────────────────────────────────

fn build_turn_view(state: &GameState, player_names: &HashMap<PlayerId, String>) -> TurnView {
    TurnView {
        number: state.turn.turn_number,
        active_player: player_name(state.turn.active_player, player_names),
        phase: format!("{:?}", state.turn.phase),
        step: format!("{:?}", state.turn.step),
        priority: state
            .turn
            .priority_holder
            .map(|p| player_name(p, player_names)),
    }
}

// ── Helper: PlayersView ───────────────────────────────────────────────────────

fn build_players_view(
    state: &GameState,
    player_names: &HashMap<PlayerId, String>,
) -> HashMap<String, PlayerView> {
    let mut result = HashMap::new();

    for (pid, player) in &state.players {
        let name = player_name(*pid, player_names);

        // Count objects in each zone for this player.
        let hand_size = state
            .objects
            .values()
            .filter(|o| o.zone == ZoneId::Hand(*pid))
            .count();
        let library_size = state
            .zones
            .get(&ZoneId::Library(*pid))
            .map(|z| z.len())
            .unwrap_or(0);
        let graveyard_size = state
            .zones
            .get(&ZoneId::Graveyard(*pid))
            .map(|z| z.len())
            .unwrap_or(0);

        // Commander damage: convert PlayerId keys → player name strings,
        // and CardId keys → card name strings (look up via objects).
        let mut commander_damage: HashMap<String, HashMap<String, u32>> = HashMap::new();
        for (opponent_pid, by_card) in &player.commander_damage_received {
            let opp_name = player_name(*opponent_pid, player_names);
            let mut by_card_view: HashMap<String, u32> = HashMap::new();
            for (card_id, &damage) in by_card {
                // Try to find the card's name from the objects map using card_id.
                let card_name = state
                    .objects
                    .values()
                    .find(|o| o.card_id.as_ref() == Some(card_id))
                    .map(|o| o.characteristics.name.clone())
                    .unwrap_or_else(|| card_id.0.clone());
                by_card_view.insert(card_name, damage);
            }
            commander_damage.insert(opp_name, by_card_view);
        }

        let view = PlayerView {
            life: player.life_total,
            poison: player.poison_counters,
            mana_pool: ManaPoolView {
                white: player.mana_pool.white,
                blue: player.mana_pool.blue,
                black: player.mana_pool.black,
                red: player.mana_pool.red,
                green: player.mana_pool.green,
                colorless: player.mana_pool.colorless,
            },
            hand_size,
            library_size,
            graveyard_size,
            commander_damage_received: commander_damage,
            land_plays_remaining: player.land_plays_remaining,
            has_lost: player.has_lost,
            has_conceded: player.has_conceded,
            has_citys_blessing: player.has_citys_blessing,
        };

        result.insert(name, view);
    }

    result
}

// ── Helper: ZonesView ────────────────────────────────────────────────────────

fn build_zones_view(state: &GameState, player_names: &HashMap<PlayerId, String>) -> ZonesView {
    // Determine which card_ids are commanders (any player's commander_ids).
    let commander_card_ids: std::collections::HashSet<_> = state
        .players
        .values()
        .flat_map(|p| p.commander_ids.iter().cloned())
        .collect();

    // ── Battlefield ────────────────────────────────────────────────────────
    let mut battlefield: HashMap<String, Vec<PermanentView>> = HashMap::new();
    for (_, obj) in &state.objects {
        if obj.zone != ZoneId::Battlefield {
            continue;
        }
        let controller_name = player_name(obj.controller, player_names);
        let is_commander = obj
            .card_id
            .as_ref()
            .map(|cid| commander_card_ids.contains(cid))
            .unwrap_or(false);

        // Use post-layer calculated characteristics (CR 613) so that continuous
        // effects (e.g. Humility, Glorious Anthem) are reflected in the viewer.
        let calc = calculate_characteristics(state, obj.id);
        let chars = calc.as_ref().unwrap_or(&obj.characteristics);

        let permanent = PermanentView {
            object_id: obj.id.0,
            name: chars.name.clone(),
            card_types: chars.card_types.iter().map(|t| format!("{t:?}")).collect(),
            subtypes: chars.subtypes.iter().map(|s| s.0.clone()).collect(),
            supertypes: chars.supertypes.iter().map(|s| format!("{s:?}")).collect(),
            tapped: obj.status.tapped,
            summoning_sick: obj.has_summoning_sickness,
            power: chars.power,
            toughness: chars.toughness,
            counters: obj
                .counters
                .iter()
                .map(|(ct, &n)| (format_counter_type(ct), n))
                .collect(),
            damage_marked: obj.damage_marked,
            attached_to: obj.attached_to.map(|id| id.0),
            attachments: obj.attachments.iter().map(|id| id.0).collect(),
            is_commander,
            is_token: obj.is_token,
            keywords: chars.keywords.iter().map(format_keyword).collect(),
            controller: controller_name.clone(),
        };

        battlefield
            .entry(controller_name)
            .or_default()
            .push(permanent);
    }

    // ── Hands ──────────────────────────────────────────────────────────────
    let mut hand: HashMap<String, Vec<CardInZoneView>> = HashMap::new();
    for pid in state.players.keys() {
        let pname = player_name(*pid, player_names);
        let cards = objects_in_zone_as_card_views(state, &ZoneId::Hand(*pid));
        hand.insert(pname, cards);
    }

    // ── Graveyards ─────────────────────────────────────────────────────────
    let mut graveyard: HashMap<String, Vec<CardInZoneView>> = HashMap::new();
    for pid in state.players.keys() {
        let pname = player_name(*pid, player_names);
        let cards = objects_in_zone_as_card_views(state, &ZoneId::Graveyard(*pid));
        graveyard.insert(pname, cards);
    }

    // ── Exile ──────────────────────────────────────────────────────────────
    let exile = objects_in_zone_as_card_views(state, &ZoneId::Exile);

    // ── Command zones ──────────────────────────────────────────────────────
    let mut command_zone: HashMap<String, Vec<CardInZoneView>> = HashMap::new();
    for pid in state.players.keys() {
        let pname = player_name(*pid, player_names);
        let cards = objects_in_zone_as_card_views(state, &ZoneId::Command(*pid));
        command_zone.insert(pname, cards);
    }

    // ── Stack ─────────────────────────────────────────────────────────────
    let stack: Vec<StackItemView> = state
        .stack_objects
        .iter()
        .map(|so| {
            let controller_name = player_name(so.controller, player_names);
            let (kind, source_id) = stack_kind_info(&so.kind);
            let source_name = source_id
                .and_then(|oid| state.objects.get(&oid))
                .map(|o| o.characteristics.name.clone())
                .unwrap_or_else(|| "unknown".to_string());

            let targets: Vec<String> = so
                .targets
                .iter()
                .map(|t| format_target(&t.target, state, player_names))
                .collect();

            StackItemView {
                id: so.id.0,
                controller: controller_name,
                kind: kind.to_string(),
                source_name,
                targets,
                is_copy: so.is_copy,
            }
        })
        .collect();

    ZonesView {
        battlefield,
        hand,
        graveyard,
        exile,
        command_zone,
        stack,
    }
}

/// Extract (kind_str, source_object_id) from a StackObjectKind.
fn stack_kind_info(kind: &StackObjectKind) -> (&'static str, Option<ObjectId>) {
    match kind {
        StackObjectKind::Spell { source_object } => ("spell", Some(*source_object)),
        StackObjectKind::ActivatedAbility { source_object, .. } => {
            ("activated_ability", Some(*source_object))
        }
        StackObjectKind::TriggeredAbility { source_object, .. } => {
            ("triggered_ability", Some(*source_object))
        }
        StackObjectKind::CascadeTrigger { source_object, .. } => {
            ("cascade_trigger", Some(*source_object))
        }
        StackObjectKind::StormTrigger { source_object, .. } => {
            ("storm_trigger", Some(*source_object))
        }
        StackObjectKind::EvokeSacrificeTrigger { source_object } => {
            ("evoke_sacrifice_trigger", Some(*source_object))
        }
        StackObjectKind::MadnessTrigger { source_object, .. } => {
            ("madness_trigger", Some(*source_object))
        }
        StackObjectKind::MiracleTrigger { source_object, .. } => {
            ("miracle_trigger", Some(*source_object))
        }
        StackObjectKind::UnearthAbility { source_object } => {
            ("unearth_ability", Some(*source_object))
        }
        StackObjectKind::UnearthTrigger { source_object } => {
            ("unearth_trigger", Some(*source_object))
        }
        StackObjectKind::ExploitTrigger { source_object } => {
            ("exploit_trigger", Some(*source_object))
        }
        StackObjectKind::ModularTrigger { source_object, .. } => {
            ("modular_trigger", Some(*source_object))
        }
        StackObjectKind::EvolveTrigger { source_object, .. } => {
            ("evolve_trigger", Some(*source_object))
        }
        StackObjectKind::MyriadTrigger { source_object, .. } => {
            ("myriad_trigger", Some(*source_object))
        }
        StackObjectKind::SuspendCounterTrigger { source_object, .. } => {
            ("suspend_counter_trigger", Some(*source_object))
        }
        StackObjectKind::SuspendCastTrigger { source_object, .. } => {
            ("suspend_cast_trigger", Some(*source_object))
        }
        StackObjectKind::HideawayTrigger { source_object, .. } => {
            ("hideaway_trigger", Some(*source_object))
        }
        StackObjectKind::PartnerWithTrigger { source_object, .. } => {
            ("partner_with_trigger", Some(*source_object))
        }
        StackObjectKind::IngestTrigger { source_object, .. } => {
            ("ingest_trigger", Some(*source_object))
        }
        StackObjectKind::FlankingTrigger { source_object, .. } => {
            ("flanking_trigger", Some(*source_object))
        }
        StackObjectKind::RampageTrigger { source_object, .. } => {
            ("rampage_trigger", Some(*source_object))
        }
        StackObjectKind::ProvokeTrigger { source_object, .. } => {
            ("provoke_trigger", Some(*source_object))
        }
        StackObjectKind::RenownTrigger { source_object, .. } => {
            ("renown_trigger", Some(*source_object))
        }
        StackObjectKind::MeleeTrigger { source_object, .. } => {
            ("melee_trigger", Some(*source_object))
        }
        StackObjectKind::PoisonousTrigger { source_object, .. } => {
            ("poisonous_trigger", Some(*source_object))
        }
        StackObjectKind::EnlistTrigger { source_object, .. } => {
            ("enlist_trigger", Some(*source_object))
        }
        StackObjectKind::NinjutsuAbility { source_object, .. } => {
            ("ninjutsu_ability", Some(*source_object))
        }
        StackObjectKind::EmbalmAbility { .. } => {
            // No source_object -- card was already exiled as cost (CR 702.128a, CR 400.7).
            ("embalm_ability", None)
        }
        StackObjectKind::EternalizeAbility { .. } => {
            // No source_object -- card was already exiled as cost (CR 702.129a, CR 400.7).
            ("eternalize_ability", None)
        }
        StackObjectKind::EncoreAbility { .. } => {
            // No source_object -- card was already exiled as cost (CR 702.141a, CR 400.7).
            ("encore_ability", None)
        }
        StackObjectKind::EncoreSacrificeTrigger { source_object, .. } => {
            ("encore_sacrifice_trigger", Some(*source_object))
        }
        StackObjectKind::DashReturnTrigger { source_object } => {
            ("dash_return_trigger", Some(*source_object))
        }
        StackObjectKind::BlitzSacrificeTrigger { source_object } => {
            ("blitz_sacrifice_trigger", Some(*source_object))
        }
        StackObjectKind::ImpendingCounterTrigger {
            impending_permanent,
            ..
        } => ("impending_counter_trigger", Some(*impending_permanent)),
        StackObjectKind::CasualtyTrigger { source_object, .. } => {
            ("casualty_trigger", Some(*source_object))
        }
        StackObjectKind::ReplicateTrigger { source_object, .. } => {
            ("replicate_trigger", Some(*source_object))
        }
        StackObjectKind::GravestormTrigger { source_object, .. } => {
            ("gravestorm_trigger", Some(*source_object))
        }
        StackObjectKind::VanishingCounterTrigger {
            vanishing_permanent,
            ..
        } => ("vanishing_counter_trigger", Some(*vanishing_permanent)),
        StackObjectKind::VanishingSacrificeTrigger {
            vanishing_permanent,
            ..
        } => ("vanishing_sacrifice_trigger", Some(*vanishing_permanent)),
        StackObjectKind::FadingTrigger {
            fading_permanent, ..
        } => ("fading_trigger", Some(*fading_permanent)),
        StackObjectKind::EchoTrigger { echo_permanent, .. } => {
            ("echo_trigger", Some(*echo_permanent))
        }
        StackObjectKind::CumulativeUpkeepTrigger { cu_permanent, .. } => {
            ("cumulative_upkeep_trigger", Some(*cu_permanent))
        }
        StackObjectKind::RecoverTrigger { recover_card, .. } => {
            ("recover_trigger", Some(*recover_card))
        }
        StackObjectKind::ForecastAbility { source_object, .. } => {
            ("forecast_ability", Some(*source_object))
        }
        StackObjectKind::GraftTrigger { source_object, .. } => {
            ("graft_trigger", Some(*source_object))
        }
        StackObjectKind::ScavengeAbility { .. } => {
            // No source_object -- card was already exiled as cost (CR 702.97a, CR 400.7).
            ("scavenge_ability", None)
        }
        StackObjectKind::BackupTrigger { source_object, .. } => {
            ("backup_trigger", Some(*source_object))
        }
        StackObjectKind::ChampionETBTrigger { source_object, .. } => {
            ("champion_etb_trigger", Some(*source_object))
        }
        StackObjectKind::ChampionLTBTrigger { source_object, .. } => {
            ("champion_ltb_trigger", Some(*source_object))
        }
        StackObjectKind::SoulbondTrigger { source_object, .. } => {
            ("soulbond_trigger", Some(*source_object))
        }
    }
}

/// Convert a `Target` to a human-readable string.
fn format_target(
    target: &mtg_engine::state::targeting::Target,
    state: &GameState,
    player_names: &HashMap<PlayerId, String>,
) -> String {
    use mtg_engine::state::targeting::Target;
    match target {
        Target::Player(pid) => format!("player:{}", player_name(*pid, player_names)),
        Target::Object(oid) => {
            let name = state
                .objects
                .get(oid)
                .map(|o| o.characteristics.name.clone())
                .unwrap_or_else(|| format!("object_{}", oid.0));
            format!("object:{name}")
        }
    }
}

/// Collect all objects in a zone as `CardInZoneView`s, in zone order.
fn objects_in_zone_as_card_views(state: &GameState, zone_id: &ZoneId) -> Vec<CardInZoneView> {
    let ids = match state.zones.get(zone_id) {
        Some(zone) => zone.object_ids(),
        None => return vec![],
    };
    ids.iter()
        .filter_map(|id| state.objects.get(id))
        .map(|obj| CardInZoneView {
            object_id: obj.id.0,
            name: obj.characteristics.name.clone(),
            card_types: obj
                .characteristics
                .card_types
                .iter()
                .map(|t| format!("{t:?}"))
                .collect(),
        })
        .collect()
}

// ── Helper: CombatView ────────────────────────────────────────────────────────

fn build_combat_view(
    combat: &CombatState,
    state: &GameState,
    player_names: &HashMap<PlayerId, String>,
) -> CombatView {
    let attacking_player = player_name(combat.attacking_player, player_names);

    let attackers: Vec<AttackerView> = combat
        .attackers
        .iter()
        .map(|(attacker_id, attack_target)| {
            let attacker_name = state
                .objects
                .get(attacker_id)
                .map(|o| o.characteristics.name.clone())
                .unwrap_or_else(|| format!("object_{}", attacker_id.0));

            let target_str = match attack_target {
                AttackTarget::Player(pid) => {
                    format!("player:{}", player_name(*pid, player_names))
                }
                AttackTarget::Planeswalker(oid) => {
                    let pw_name = state
                        .objects
                        .get(oid)
                        .map(|o| o.characteristics.name.clone())
                        .unwrap_or_else(|| format!("object_{}", oid.0));
                    format!("planeswalker:{pw_name}")
                }
            };

            // Find blockers assigned to this attacker.
            let blockers: Vec<BlockerView> = combat
                .blockers
                .iter()
                .filter(|(_, &a)| a == *attacker_id)
                .map(|(blocker_id, _)| {
                    let blocker_name = state
                        .objects
                        .get(blocker_id)
                        .map(|o| o.characteristics.name.clone())
                        .unwrap_or_else(|| format!("object_{}", blocker_id.0));
                    BlockerView {
                        object_id: blocker_id.0,
                        name: blocker_name,
                    }
                })
                .collect();

            AttackerView {
                object_id: attacker_id.0,
                name: attacker_name,
                target: target_str,
                blockers,
            }
        })
        .collect();

    CombatView {
        attacking_player,
        attackers,
    }
}

// ── Formatting helpers ─────────────────────────────────────────────────────────

fn format_counter_type(ct: &CounterType) -> String {
    match ct {
        CounterType::PlusOnePlusOne => "+1/+1".to_string(),
        CounterType::MinusOneMinusOne => "-1/-1".to_string(),
        CounterType::Loyalty => "loyalty".to_string(),
        CounterType::Charge => "charge".to_string(),
        CounterType::Energy => "energy".to_string(),
        CounterType::Experience => "experience".to_string(),
        CounterType::Level => "level".to_string(),
        CounterType::Lore => "lore".to_string(),
        CounterType::Oil => "oil".to_string(),
        CounterType::Poison => "poison".to_string(),
        CounterType::Shield => "shield".to_string(),
        CounterType::Stun => "stun".to_string(),
        CounterType::Time => "time".to_string(),
        CounterType::Fade => "fade".to_string(),
        CounterType::Age => "age".to_string(),
        CounterType::Custom(s) => s.clone(),
    }
}

fn format_keyword(kw: &KeywordAbility) -> String {
    match kw {
        KeywordAbility::Deathtouch => "Deathtouch".to_string(),
        KeywordAbility::Defender => "Defender".to_string(),
        KeywordAbility::DoubleStrike => "Double Strike".to_string(),
        KeywordAbility::Enchant(target) => format!("Enchant {target:?}"),
        KeywordAbility::Equip => "Equip".to_string(),
        KeywordAbility::FirstStrike => "First Strike".to_string(),
        KeywordAbility::Flash => "Flash".to_string(),
        KeywordAbility::Flying => "Flying".to_string(),
        KeywordAbility::Haste => "Haste".to_string(),
        KeywordAbility::Hexproof => "Hexproof".to_string(),
        KeywordAbility::Indestructible => "Indestructible".to_string(),
        KeywordAbility::Intimidate => "Intimidate".to_string(),
        KeywordAbility::Landwalk(lw) => format!("Landwalk ({lw:?})"),
        KeywordAbility::Lifelink => "Lifelink".to_string(),
        KeywordAbility::Menace => "Menace".to_string(),
        KeywordAbility::ProtectionFrom(q) => format!("Protection ({q:?})"),
        KeywordAbility::Prowess => "Prowess".to_string(),
        KeywordAbility::Reach => "Reach".to_string(),
        KeywordAbility::Shroud => "Shroud".to_string(),
        KeywordAbility::Trample => "Trample".to_string(),
        KeywordAbility::Vigilance => "Vigilance".to_string(),
        KeywordAbility::Ward(n) => format!("Ward {{{n}}}"),
        KeywordAbility::Partner => "Partner".to_string(),
        KeywordAbility::NoMaxHandSize => "No Maximum Hand Size".to_string(),
        KeywordAbility::CantBeBlocked => "Can't Be Blocked".to_string(),
        KeywordAbility::Storm => "Storm".to_string(),
        KeywordAbility::Cascade => "Cascade".to_string(),
        KeywordAbility::Flashback => "Flashback".to_string(),
        KeywordAbility::Cycling => "Cycling".to_string(),
        KeywordAbility::Dredge(n) => format!("Dredge {n}"),
        KeywordAbility::Convoke => "Convoke".to_string(),
        KeywordAbility::Delve => "Delve".to_string(),
        KeywordAbility::Kicker => "Kicker".to_string(),
        KeywordAbility::SplitSecond => "Split Second".to_string(),
        KeywordAbility::Exalted => "Exalted".to_string(),
        KeywordAbility::Annihilator(n) => format!("Annihilator {n}"),
        KeywordAbility::Persist => "Persist".to_string(),
        KeywordAbility::Undying => "Undying".to_string(),
        KeywordAbility::Changeling => "Changeling".to_string(),
        KeywordAbility::Evoke => "Evoke".to_string(),
        KeywordAbility::Crew(n) => format!("Crew {n}"),
        KeywordAbility::BattleCry => "Battle Cry".to_string(),
        KeywordAbility::Afterlife(n) => format!("Afterlife {n}"),
        KeywordAbility::Extort => "Extort".to_string(),
        KeywordAbility::Improvise => "Improvise".to_string(),
        KeywordAbility::Bestow => "Bestow".to_string(),
        KeywordAbility::Fear => "Fear".to_string(),
        KeywordAbility::LivingWeapon => "Living Weapon".to_string(),
        KeywordAbility::Madness => "Madness".to_string(),
        KeywordAbility::Miracle => "Miracle".to_string(),
        KeywordAbility::Escape => "Escape".to_string(),
        KeywordAbility::Foretell => "Foretell".to_string(),
        KeywordAbility::Unearth => "Unearth".to_string(),
        KeywordAbility::Affinity(target) => match target {
            AffinityTarget::Artifacts => "Affinity for artifacts".to_string(),
            AffinityTarget::BasicLandType(st) => format!("Affinity for {}", st.0),
        },
        KeywordAbility::Undaunted => "Undaunted".to_string(),
        KeywordAbility::Dethrone => "Dethrone".to_string(),
        KeywordAbility::Riot => "Riot".to_string(),
        KeywordAbility::Exploit => "Exploit".to_string(),
        KeywordAbility::Wither => "Wither".to_string(),
        KeywordAbility::Modular(n) => format!("Modular {n}"),
        KeywordAbility::Evolve => "Evolve".to_string(),
        KeywordAbility::Buyback => "Buyback".to_string(),
        KeywordAbility::Ascend => "Ascend".to_string(),
        KeywordAbility::Infect => "Infect".to_string(),
        KeywordAbility::Myriad => "Myriad".to_string(),
        KeywordAbility::Suspend => "Suspend".to_string(),
        KeywordAbility::Hideaway(n) => format!("Hideaway {n}"),
        KeywordAbility::Adapt(n) => format!("Adapt {n}"),
        KeywordAbility::Shadow => "Shadow".to_string(),
        KeywordAbility::PartnerWith(name) => format!("Partner with {name}"),
        KeywordAbility::Overload => "Overload".to_string(),
        KeywordAbility::Horsemanship => "Horsemanship".to_string(),
        KeywordAbility::Skulk => "Skulk".to_string(),
        KeywordAbility::Devoid => "Devoid".to_string(),
        KeywordAbility::Decayed => "Decayed".to_string(),
        KeywordAbility::Ingest => "Ingest".to_string(),
        KeywordAbility::Flanking => "Flanking".to_string(),
        KeywordAbility::Bushido(n) => format!("Bushido {n}"),
        KeywordAbility::Rampage(n) => format!("Rampage {n}"),
        KeywordAbility::Provoke => "Provoke".to_string(),
        KeywordAbility::Afflict(n) => format!("Afflict {n}"),
        KeywordAbility::Renown(n) => format!("Renown {n}"),
        KeywordAbility::Training => "Training".to_string(),
        KeywordAbility::Melee => "Melee".to_string(),
        KeywordAbility::Poisonous(n) => format!("Poisonous {n}"),
        KeywordAbility::Toxic(n) => format!("Toxic {n}"),
        KeywordAbility::Enlist => "Enlist".to_string(),
        KeywordAbility::Ninjutsu => "Ninjutsu".to_string(),
        KeywordAbility::CommanderNinjutsu => "Commander Ninjutsu".to_string(),
        KeywordAbility::Retrace => "Retrace".to_string(),
        KeywordAbility::JumpStart => "Jump-Start".to_string(),
        KeywordAbility::Aftermath => "Aftermath".to_string(),
        KeywordAbility::Embalm => "Embalm".to_string(),
        KeywordAbility::Eternalize => "Eternalize".to_string(),
        KeywordAbility::Encore => "Encore".to_string(),
        KeywordAbility::Dash => "Dash".to_string(),
        KeywordAbility::Blitz => "Blitz".to_string(),
        KeywordAbility::Plot => "Plot".to_string(),
        KeywordAbility::Prototype => "Prototype".to_string(),
        KeywordAbility::Impending => "Impending".to_string(),
        KeywordAbility::Bargain => "Bargain".to_string(),
        KeywordAbility::Emerge => "Emerge".to_string(),
        KeywordAbility::Spectacle => "Spectacle".to_string(),
        KeywordAbility::Surge => "Surge".to_string(),
        KeywordAbility::Casualty(n) => format!("Casualty {}", n),
        KeywordAbility::Assist => "Assist".to_string(),
        KeywordAbility::Replicate => "Replicate".to_string(),
        KeywordAbility::Gravestorm => "Gravestorm".to_string(),
        KeywordAbility::Cleave => "Cleave".to_string(),
        KeywordAbility::Splice => "Splice".to_string(),
        KeywordAbility::Entwine => "Entwine".to_string(),
        KeywordAbility::Escalate => "Escalate".to_string(),
        KeywordAbility::Vanishing(n) => {
            if *n == 0 {
                "Vanishing".to_string()
            } else {
                format!("Vanishing {n}")
            }
        }
        KeywordAbility::Fading(n) => format!("Fading {n}"),
        KeywordAbility::Echo(_) => "Echo".to_string(),
        KeywordAbility::CumulativeUpkeep(_) => "Cumulative Upkeep".to_string(),
        KeywordAbility::Recover => "Recover".to_string(),
        KeywordAbility::Forecast => "Forecast".to_string(),
        KeywordAbility::Phasing => "Phasing".to_string(),
        KeywordAbility::Graft(n) => format!("Graft {n}"),
        KeywordAbility::Scavenge => "Scavenge".to_string(),
        KeywordAbility::Outlast => "Outlast".to_string(),
        KeywordAbility::Amplify(n) => format!("Amplify {n}"),
        KeywordAbility::Bloodthirst(n) => format!("Bloodthirst {n}"),
        KeywordAbility::Devour(n) => format!("Devour {n}"),
        KeywordAbility::Backup(n) => format!("Backup {n}"),
        KeywordAbility::Champion => "Champion".to_string(),
        KeywordAbility::UmbraArmor => "Umbra Armor".to_string(),
        KeywordAbility::LivingMetal => "Living Metal".to_string(),
        KeywordAbility::Soulbond => "Soulbond".to_string(),
    }
}
