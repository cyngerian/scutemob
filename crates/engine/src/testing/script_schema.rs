/// Game script schema — Hook 1 from `docs/mtg-engine-game-scripts.md`.
///
/// `GameScript` is the contract between script generation (Claude Code + MCP tools,
/// M7+) and the replay harness (`tests/script_replay.rs`, also M7). Defining the
/// type now lets the schema evolve under the compiler before any scripts exist.
///
/// Scripts live in `test-data/generated-scripts/` organized by subsystem. The
/// replay harness auto-discovers and runs all `review_status: approved` scripts.
///
/// Schema version: 1.0.0
use serde::{Deserialize, Serialize};
use std::collections::HashMap;

// ── Top-level ────────────────────────────────────────────────────────────────

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct GameScript {
    pub schema_version: String,
    pub metadata: ScriptMetadata,
    pub initial_state: InitialState,
    pub script: Vec<ScriptStep>,
}

// ── Metadata ─────────────────────────────────────────────────────────────────

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct ScriptMetadata {
    /// Unique identifier. Format: `script_<topic>_<nnn>`
    pub id: String,
    pub name: String,
    pub description: String,
    pub cr_sections_tested: Vec<String>,
    pub corner_case_ref: Option<u32>,
    pub tags: Vec<String>,
    pub confidence: Confidence,
    pub review_status: ReviewStatus,
    pub reviewed_by: Option<String>,
    pub review_date: Option<String>,
    pub generation_notes: Option<String>,
    #[serde(default)]
    pub disputes: Vec<Dispute>,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
#[serde(rename_all = "snake_case")]
pub enum Confidence {
    /// Simple interaction, well-documented in CR, no ambiguity.
    High,
    /// Multiple subsystems interact, or rules text is ambiguous.
    Medium,
    /// Complex replacement chains, dependency resolution, or gaps in CR examples.
    Low,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
#[serde(rename_all = "snake_case")]
pub enum ReviewStatus {
    PendingReview,
    Approved,
    Disputed,
    Corrected,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct Dispute {
    /// None for script-level disputes not tied to a specific step.
    pub step_index: Option<usize>,
    /// None for script-level disputes not tied to a specific action.
    pub action_index: Option<usize>,
    pub raised_by: String,
    pub description: String,
    pub resolution: Option<String>,
    pub resolved_by: Option<String>,
    pub resolved_date: Option<String>,
}

// ── Initial state ─────────────────────────────────────────────────────────────

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct InitialState {
    pub format: String,
    pub turn_number: u32,
    pub active_player: String,
    /// Phase name, e.g. `"precombat_main"`. Matches `Step` names in the engine.
    pub phase: String,
    pub step: Option<String>,
    pub priority: String,
    pub players: HashMap<String, PlayerInitState>,
    pub zones: ZonesInitState,
    #[serde(default)]
    pub continuous_effects: Vec<ContinuousEffectInitState>,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct PlayerInitState {
    pub life: i32,
    #[serde(default)]
    pub mana_pool: HashMap<String, u32>,
    #[serde(default)]
    pub land_plays_remaining: u32,
    #[serde(default)]
    pub poison_counters: u32,
    #[serde(default)]
    pub commander_damage_received: HashMap<String, i32>,
    pub commander: Option<CommanderInitState>,
    pub partner_commander: Option<CommanderInitState>,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct CommanderInitState {
    pub card: String,
    pub zone: String,
    pub times_cast_from_command_zone: u32,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct ZonesInitState {
    /// `controller → [permanents]`
    #[serde(default)]
    pub battlefield: HashMap<String, Vec<PermanentInitState>>,
    /// `player → [cards]`
    #[serde(default)]
    pub hand: HashMap<String, Vec<CardInZone>>,
    #[serde(default)]
    pub graveyard: HashMap<String, Vec<CardInZone>>,
    #[serde(default)]
    pub exile: Vec<CardInZone>,
    #[serde(default)]
    pub command_zone: HashMap<String, Vec<CardInZone>>,
    /// Ordered top-to-bottom. Omit unless the scenario involves drawing/searching.
    #[serde(default)]
    pub library: HashMap<String, Vec<CardInZone>>,
    /// Rarely pre-populated; use `serde_json::Value` for flexibility.
    #[serde(default)]
    pub stack: Vec<serde_json::Value>,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct PermanentInitState {
    /// Scryfall oracle card name (exact match required for replay harness card DB lookup).
    pub card: String,
    #[serde(default)]
    pub tapped: bool,
    #[serde(default)]
    pub summoning_sick: bool,
    #[serde(default)]
    pub counters: HashMap<String, u32>,
    #[serde(default)]
    pub attached: Vec<String>,
    #[serde(default)]
    pub damage_marked: u32,
    #[serde(default)]
    pub is_commander: bool,
    pub subtypes: Option<Vec<String>>,
    pub is_basic: Option<bool>,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Default)]
pub struct CardInZone {
    pub card: String,
    #[serde(default)]
    pub is_commander: bool,
    pub owner: Option<String>,
    /// CR 702.62: True for cards exiled via suspend (they have time counters ticking down).
    #[serde(default)]
    pub is_suspended: bool,
    /// Counters on the exiled card (e.g., time counters for suspend). Key = counter type string.
    #[serde(default)]
    pub counters: HashMap<String, u32>,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct ContinuousEffectInitState {
    pub source: String,
    pub effect: String,
    pub layer: u8,
    pub timestamp: u64,
    pub duration: String,
}

// ── Script steps and actions ──────────────────────────────────────────────────

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct ScriptStep {
    /// One of the step identifiers from `mtg-engine-game-scripts.md` (e.g. `"precombat_main"`).
    pub step: String,
    pub step_note: Option<String>,
    pub actions: Vec<ScriptAction>,
}

/// Every observable event in a script — player actions, priority passes, SBA checks,
/// stack resolutions, and state assertions.
///
/// Uses `#[serde(tag = "type")]` so JSON objects carry a `"type"` discriminant field
/// matching the variant names in `snake_case`.
// PlayerAction is intentionally large — it holds all script action data in a flat
// struct to avoid nested JSON. Boxing individual fields would break the serde schema.
#[allow(clippy::large_enum_variant)]
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
#[serde(tag = "type", rename_all = "snake_case")]
pub enum ScriptAction {
    /// A player takes a named game action (`cast_spell`, `play_land`, `activate_ability`, …).
    PlayerAction {
        player: String,
        /// One of: `cast_spell`, `activate_ability`, `play_land`, `declare_attackers`,
        /// `declare_blockers`, `assign_damage`, `choose_option`, `order_triggers`,
        /// `special_action`, `concede`, `mulligan_decision`.
        action: String,
        card: Option<String>,
        #[serde(default)]
        targets: Vec<ActionTarget>,
        mana_paid: Option<HashMap<String, u32>>,
        #[serde(default)]
        mana_source: Vec<ManaSource>,
        /// 0-based index into `characteristics.activated_abilities` (non-mana only).
        /// Required for `activate_ability` actions. Defaults to 0.
        #[serde(default)]
        ability_index: u32,
        /// For `declare_attackers`: list of (creature_name, attack_target_player) pairs.
        /// Each entry declares one creature as attacking one player or planeswalker.
        /// Example: [{"card": "Grizzly Bears", "target_player": "p2"}]
        #[serde(default)]
        attackers: Vec<AttackerDeclaration>,
        /// For `declare_blockers`: list of (blocker_name, attacker_name) pairs.
        /// Each entry declares one creature as blocking one attacker.
        /// Example: [{"card": "Llanowar Elves", "blocking": "Grizzly Bears"}]
        #[serde(default)]
        blockers: Vec<BlockerDeclaration>,
        /// CR 702.154a: For `declare_attackers` with enlist. Each entry specifies an
        /// attacker with Enlist and the non-attacking creature to tap as the enlist cost.
        /// Example: [{"attacker": "Coalition Skyknight", "enlisted": "Llanowar Elves"}]
        #[serde(default)]
        enlist: Vec<EnlistDeclaration>,
        /// CR 702.51: For `cast_spell` with convoke. Names of untapped creatures on the
        /// battlefield to tap as part of cost payment. Empty for non-convoke casts.
        /// Example: ["Llanowar Elves", "Saproling Token", "Saproling Token"]
        #[serde(default)]
        convoke: Vec<String>,
        /// CR 702.126: For `cast_spell` with improvise. Names of untapped artifacts on the
        /// battlefield to tap as part of cost payment. Empty for non-improvise casts.
        /// Example: ["Sol Ring", "Mana Vault", "Signet"]
        #[serde(default)]
        improvise: Vec<String>,
        /// CR 702.66: For `cast_spell` with delve. Names of cards in the caster's graveyard
        /// to exile as part of cost payment. Empty for non-delve casts.
        /// Example: ["Lightning Bolt", "Mountain", "Grizzly Bears"]
        #[serde(default)]
        delve: Vec<String>,
        /// CR 702.33: For `cast_spell` with kicker. If true, the kicker cost is paid
        /// once (standard kicker). For multikicker, use `kicker_times` instead.
        /// Defaults to false (not kicked).
        #[serde(default)]
        kicked: bool,
        /// CR 702.138: For `cast_spell_escape`. Names of cards in the caster's graveyard
        /// to exile as part of the escape cost. Empty for non-escape casts.
        /// Example: ["Lightning Bolt", "Mountain", "Grizzly Bears"]
        #[serde(default)]
        escape: Vec<String>,
        /// CR 702.27: For `cast_spell` with buyback. If true, the buyback additional
        /// cost is paid. If the spell resolves, it returns to the owner's hand.
        /// Defaults to false (buyback not paid).
        #[serde(default)]
        buyback: bool,
        /// CR 702.49a: For `activate_ninjutsu`. Name of the unblocked attacking
        /// creature to return to its owner's hand as part of the ninjutsu cost.
        /// Example: "Eager Construct"
        #[serde(default)]
        attacker_name: Option<String>,
        /// CR 702.81a: For `cast_spell_retrace`. Name of the land card in the
        /// player's hand to discard as the retrace additional cost.
        /// Example: "Mountain"
        #[serde(default)]
        discard_land: Option<String>,
        /// CR 702.133a: For `cast_spell_jump_start`. Name of any card in the
        /// player's hand to discard as the jump-start additional cost.
        /// Any card type is accepted (unlike retrace which requires a land).
        /// Example: "Lightning Bolt"
        #[serde(default)]
        discard_card: Option<String>,
        cr_ref: Option<String>,
        note: Option<String>,
    },

    /// A single player passes priority.
    PriorityPass {
        player: String,
        note: Option<String>,
    },

    /// Shorthand: all listed players pass in succession with no action taken.
    PriorityRound {
        players: Vec<String>,
        /// Usually `"all_pass"`.
        result: String,
        note: Option<String>,
    },

    /// The top of the stack resolves.
    StackResolve {
        object: String,
        #[serde(default)]
        resolution: Vec<ResolutionEffect>,
        note: Option<String>,
    },

    /// State-based actions are checked (CR 704.3). `results` may be empty.
    SbaCheck {
        #[serde(default)]
        results: Vec<SbaResult>,
        #[serde(default)]
        triggered_abilities: Vec<TriggeredAbilityEvent>,
        note: Option<String>,
    },

    /// A triggered ability is placed onto the stack (CR 603.3).
    TriggerPlaced {
        source: String,
        controller: String,
        trigger_condition: String,
        #[serde(default)]
        targets: Vec<ActionTarget>,
        ordering_context: Option<String>,
        apnap_position: Option<u32>,
        cr_ref: Option<String>,
        note: Option<String>,
    },

    /// Checkpoint: the replay harness asserts the engine's state matches these expectations.
    /// Keys use the dot-notation path syntax from `mtg-engine-game-scripts.md`.
    AssertState {
        description: String,
        assertions: HashMap<String, serde_json::Value>,
        note: Option<String>,
    },

    /// The game advances from one step/phase to the next.
    PhaseTransition {
        from_step: String,
        to_step: String,
        cr_ref: Option<String>,
        note: Option<String>,
    },

    /// An automatic action that happens at the start of a step (untap, draw, etc.).
    TurnBasedAction {
        /// One of: `untap_all`, `draw_card`, `empty_mana_pool`, `remove_until_eot`,
        /// `discard_to_hand_size`. Optional for informational-only scripts.
        #[serde(default)]
        action: String,
        player: Option<String>,
        cr_ref: Option<String>,
        note: Option<String>,
    },
}

// ── Supporting types ──────────────────────────────────────────────────────────

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct ActionTarget {
    #[serde(rename = "type")]
    pub target_type: String,
    pub card: Option<String>,
    pub controller: Option<String>,
    pub player: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct ManaSource {
    pub card: String,
    pub tap: bool,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct ResolutionEffect {
    pub effect: String,
    pub target: Option<serde_json::Value>,
    pub player: Option<String>,
    pub amount: Option<i32>,
    pub card: Option<String>,
    pub owner: Option<String>,
    pub cr_ref: Option<String>,
    pub note: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct SbaResult {
    /// CR rule number, e.g. `"704.5f"`.
    pub sba: String,
    pub description: String,
    pub object: Option<String>,
    pub controller: Option<String>,
    pub result: String,
    pub cr_ref: String,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct TriggeredAbilityEvent {
    pub trigger: String,
    pub source: String,
    pub controller: String,
    pub cr_ref: Option<String>,
}

/// CR 508.1: One attacker declaration entry for the `declare_attackers` harness action.
///
/// `card` is the name of the attacking creature on the battlefield.
/// `target_player` is the script player name (e.g. `"p2"`) being attacked.
/// `target_planeswalker` is the planeswalker card name on the battlefield (mutually exclusive
/// with `target_player`). If both are absent the harness defaults to the first non-active player.
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct AttackerDeclaration {
    /// Name of the attacking creature (must be on the battlefield under the player's control).
    pub card: String,
    /// Name of the player being attacked (e.g. `"p2"`).
    pub target_player: Option<String>,
    /// Name of the planeswalker being attacked (on the battlefield).
    /// Mutually exclusive with `target_player`.
    pub target_planeswalker: Option<String>,
}

/// CR 702.154a: One enlist declaration entry for the `declare_attackers` harness action.
///
/// The attacking player may tap a non-attacking creature they control (without summoning
/// sickness unless it has haste) to give the attacker +X/+0 until end of turn, where X
/// is the enlisted creature's power.
///
/// `attacker` is the name of the attacking creature with Enlist on the battlefield.
/// `enlisted` is the name of the non-attacking creature being tapped for the enlist cost.
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct EnlistDeclaration {
    /// Name of the attacking creature that has the Enlist keyword.
    pub attacker: String,
    /// Name of the non-attacking creature to tap as the enlist cost.
    pub enlisted: String,
}

/// CR 509.1: One blocker declaration entry for the `declare_blockers` harness action.
///
/// `card` is the name of the blocking creature.
/// `blocking` is the name of the attacker being blocked.
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct BlockerDeclaration {
    /// Name of the blocking creature (must be on the battlefield under the player's control).
    pub card: String,
    /// Name of the attacking creature being blocked.
    pub blocking: String,
}
