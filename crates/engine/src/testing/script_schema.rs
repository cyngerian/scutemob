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
    pub step_index: usize,
    pub action_index: usize,
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

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct CardInZone {
    pub card: String,
    #[serde(default)]
    pub is_commander: bool,
    pub owner: Option<String>,
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
