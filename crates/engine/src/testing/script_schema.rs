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
///
/// # Unknown keys are rejected on the structural spine (SR-22)
///
/// The structural "spine" types — `GameScript`, `ScriptMetadata`, `InitialState`,
/// `PlayerInitState`, `CommanderInitState`, `ZonesInitState`, `CardInZone`,
/// `ScriptStep`, and the small supporting records — carry
/// `#[serde(deny_unknown_fields)]`, so a JSON key that no field claims is a hard
/// deserialization error rather than a silently-dropped value. This caught the bug
/// that motivated SR-22: `stack/135` carried a stray *top-level* `review_status`
/// (a copy of `metadata.review_status`) that serde had been ignoring — the script
/// looked approved at a glance while being retired.
///
/// Three types are **deliberately left permissive**, each documented at its
/// definition: `PermanentInitState` (every battlefield permanent in the corpus
/// carries a dead `owner: null` template key), `ResolutionEffect` (the
/// documentation-only `resolution` block is annotated with undeclared descriptive
/// keys), and `Dispute` (freeform annotations carry an extra `cr_ref`). Enforcing
/// on those would require stripping dead keys from ~190 golden scripts — a corpus
/// migration with no behaviour change — so it is noted as an SR-22 follow-up
/// rather than done here.
///
/// One type *cannot* be covered at all: [`ScriptAction`] is an internally-tagged
/// enum (`#[serde(tag = "type")]`), and serde rejects `deny_unknown_fields` in
/// combination with internal tagging at compile time. Its (large) `PlayerAction`
/// variant therefore still tolerates unknown keys.
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
// ── Top-level ────────────────────────────────────────────────────────────────
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
#[serde(deny_unknown_fields)]
pub struct GameScript {
    pub schema_version: String,
    pub metadata: ScriptMetadata,
    pub initial_state: InitialState,
    pub script: Vec<ScriptStep>,
}
// ── Metadata ─────────────────────────────────────────────────────────────────
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
#[serde(deny_unknown_fields)]
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
    /// Why this script is [`ReviewStatus::Retired`] — the scenario it describes is
    /// one the harness or the engine cannot express, and the script was withdrawn
    /// rather than fixed.
    ///
    /// **Required for, and only for, `Retired`.** `tests/scripts/run_all_scripts.rs`
    /// (`retired_scripts_carry_a_reason`) fails the suite if a retired script omits
    /// it or a non-retired script carries one. A retired script is excluded from the
    /// run — the reason is the difference between exclusion and silent absence.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub retirement_reason: Option<String>,
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
    /// Generated but not yet triaged. **Not a resting state**: SR-9c's
    /// `no_script_is_awaiting_triage` gate fails while any script sits here,
    /// because a `pending_review` script is one the corpus neither runs nor
    /// accounts for.
    PendingReview,
    /// Replays clean and its assertions were checked against the CR. Run on every
    /// `cargo test`.
    Approved,
    /// Withdrawn from the corpus with a recorded
    /// [`ScriptMetadata::retirement_reason`]. Not run, but counted and printed.
    Retired,
    Disputed,
    Corrected,
}
// SR-22: **not** `deny_unknown_fields`. `disputes[]` is a freeform annotation
// block, and existing corpus disputes carry an extra `cr_ref` key (dead — the
// harness never reads a dispute) alongside the declared fields (e.g.
// `layers/012`, `stack/061`, `stack/095`, `stack/140`). Enforcing here would
// redden those without any correctness gain; the strictness that matters is on
// the structural spine (`GameScript`/`InitialState`/…). See the module header.
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
#[serde(deny_unknown_fields)]
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
#[serde(deny_unknown_fields)]
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
#[serde(deny_unknown_fields)]
pub struct CommanderInitState {
    pub card: String,
    pub zone: String,
    pub times_cast_from_command_zone: u32,
}
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
#[serde(deny_unknown_fields)]
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
// SR-22: **not** `deny_unknown_fields`. Every battlefield permanent in the corpus
// carries a dead `"owner": null` key (~190 files) — a template artifact copied
// from `CardInZone` (which legitimately has `owner`). A battlefield permanent's
// owner is derived from its controller map key, so this struct has no `owner`
// field and the value is silently dropped. Enforcing here would require stripping
// that key from ~190 golden scripts (a corpus migration, not a stray-key fix) for
// no behaviour change. Noted as SR-22 follow-up. The spine structs above/below,
// where a mistyped key is a real mistake, are strict.
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
#[serde(deny_unknown_fields)]
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
#[serde(deny_unknown_fields)]
pub struct ContinuousEffectInitState {
    pub source: String,
    pub effect: String,
    pub layer: u8,
    pub timestamp: u64,
    pub duration: String,
}
// ── Script steps and actions ──────────────────────────────────────────────────
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
#[serde(deny_unknown_fields)]
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
        /// `special_action`, `concede`, `mulligan_decision`, `search_library`
        /// (documentation marker — no Command issued; engine resolves SearchLibrary
        /// effects deterministically by minimum ObjectId; M10 will add interactive search).
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
        /// CR 701.43d / CR 508.1g: For `declare_attackers` with exert. Names of declared
        /// attackers the player chooses to exert as an optional attack cost. Each name
        /// must be a creature with the "you may exert this creature as it attacks"
        /// static ability that is not already exerted this turn.
        /// Example: ["Combat Celebrant"]
        #[serde(default)]
        exert: Vec<String>,
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
        /// CR 702.166a: For `cast_spell_bargain`. Name of the artifact,
        /// enchantment, or token to sacrifice as the bargain additional cost.
        /// `None` means the player chose not to bargain.
        /// Example: "Clue Token"
        #[serde(default)]
        bargain_sacrifice: Option<String>,
        /// CR 702.119a: For `cast_spell_emerge`. Name of the creature on the
        /// battlefield to sacrifice as the emerge alternative cost.
        /// Required when action is `cast_spell_emerge`.
        /// Example: "Llanowar Elves"
        #[serde(default)]
        emerge_sacrifice: Option<String>,
        /// CR 702.153a: For `cast_spell_casualty`. Name of the creature on the
        /// battlefield to sacrifice as the casualty additional cost.
        /// `None` means the player chose not to pay the casualty cost (casualty is optional).
        /// Example: "Llanowar Elves"
        #[serde(default)]
        casualty_sacrifice: Option<String>,
        /// CR 702.132a: For `cast_spell_assist`. Name of the player who pays generic
        /// mana as part of the assist cost.
        /// `None` means the caster chose not to use assist (assist is optional).
        /// Example: "p2"
        #[serde(default)]
        assist_player: Option<String>,
        /// CR 702.132a: For `cast_spell_assist`. Amount of generic mana the assisting
        /// player pays. Must be <= the generic component of the spell's total cost.
        /// Ignored when `assist_player` is `None`. Defaults to 0.
        #[serde(default)]
        assist_amount: u32,
        /// CR 702.56a: For `cast_spell_replicate`. Number of times the replicate cost
        /// was paid as an additional cost during casting.
        /// 0 = not paid (no copies). N = paid N times → N copies created by trigger.
        /// Defaults to 0.
        #[serde(default)]
        replicate_count: u32,
        /// CR 702.47a: For `cast_spell_splice`. Names of cards in the caster's hand to
        /// splice onto the spell. Each card must have the Splice ability and the spell
        /// must have the matching subtype (e.g., Arcane). Empty by default.
        /// Example: ["Glacial Ray"]
        #[serde(default)]
        splice_card_names: Vec<String>,
        /// CR 702.120a: For `cast_spell_escalate`. Number of additional modes beyond the
        /// first for which the escalate cost is paid. 0 = only mode[0] executes (no extra
        /// cost). N = pay escalate cost N times and execute modes 0..=N at resolution.
        /// Defaults to 0.
        #[serde(default)]
        escalate_modes: u32,
        /// CR 700.2a / 601.2b: For `cast_spell_modal`. Explicit mode indices (0-indexed) to
        /// choose at cast time. Empty = non-modal spell or auto-select mode[0].
        /// Example: [0] for mode 0; [1, 2] for modes 1 and 2.
        #[serde(default)]
        modes: Vec<usize>,
        /// CR 702.97a: For `scavenge_card`. Name of the creature on the battlefield
        /// to receive +1/+1 counters equal to the scavenged card's power.
        /// Required when action is `scavenge_card`.
        /// Example: "Grizzly Bears"
        #[serde(default)]
        target_creature: Option<String>,
        /// CR 107.3m: For `cast_spell` (and variants) with X in the mana cost.
        /// The value chosen for X at cast time. 0 for non-X spells (default).
        /// Example: 5 means X=5 for a spell like {X}{G}.
        #[serde(default)]
        x_value: u32,
        /// CR 701.59a: For `cast_spell_collect_evidence`. Names of cards in the caster's
        /// graveyard to exile as the collect evidence additional cost.
        /// Empty = player chose not to collect evidence (optional cost) or not applicable.
        /// Total mana value of named cards must be >= N (the evidence threshold).
        /// Example: ["Lightning Bolt", "Grizzly Bears"]
        #[serde(default)]
        collect_evidence_cards: Vec<String>,
        /// CR 702.157a: For `cast_spell_squad`, the number of times the squad additional
        /// cost was paid. 0 = not paid. N = paid N times -> N token copies created on ETB.
        /// Ignored for all other action types. `#[serde(default)]` means 0 if absent from JSON.
        #[serde(default)]
        squad_count: u32,
        /// CR 702.140a: For `cast_spell_mutate`. When true, the mutating spell is placed
        /// on top of the merged permanent (topmost characteristics from the spell's card).
        /// When false, placed underneath (topmost characteristics from the existing target).
        /// Ignored for all other action types.
        #[serde(default)]
        mutate_on_top: bool,
        /// CR 602.2: For `activate_ability` with sacrifice-another-permanent cost.
        /// The name of the permanent on the battlefield to sacrifice as part of
        /// the ability's activation cost. `None` for abilities that don't require
        /// sacrificing another permanent.
        /// Example: "Llanowar Elves"
        #[serde(default)]
        sacrifice_card: Option<String>,
        /// CR 702.174a: For `cast_spell` with gift. The name of the opponent chosen to
        /// receive the gift benefit (e.g. `"p2"`). `None` means the gift was not promised;
        /// the conditional downside clause will apply at resolution (CR 702.174j).
        /// Ignored for all other action types.
        #[serde(default)]
        gift_opponent: Option<String>,
        /// CR 118.9: For `cast_spell_pitch`. Name of the card in the caster's hand to
        /// exile as (part of) the pitch alternative cost (Force of Will, Force of Vigor,
        /// Force of Negation). `None` for pitch spells with no `ExileFromHand` component
        /// or all other action types.
        /// Example: "Island"
        #[serde(default)]
        pitch_exile_card: Option<String>,
        /// CR 702.37e / CR 701.40b / CR 701.58b: For `turn_face_up`. Which turn-face-up
        /// method to use. One of: "morph_cost" (default), "disguise_cost", "mana_cost".
        /// "morph_cost" uses the card's Morph or Megamorph cost.
        /// "disguise_cost" uses the card's Disguise cost.
        /// "mana_cost" is for manifested/cloaked permanents (pays the card's mana cost).
        /// Ignored for all other action types.
        #[serde(default)]
        method: Option<String>,
        /// PB-EF12 (CR 605.3b / CR 106.1b): For `tap_for_mana` on an `any_color: true`
        /// mana ability. One of "white", "blue", "black", "red", "green" (case-insensitive).
        /// Required when the source's ability is any-color; `None` for fixed-colour sources.
        /// Example: "green"
        #[serde(default)]
        chosen_color: Option<String>,
        /// PB-RS2 (CR 107.4e via CR 602.2b/605.1a): For `activate_ability` or
        /// `tap_for_mana` on a source with a hybrid pip in its activation cost. One
        /// entry per hybrid pip, in cost order: `"white"`/`"blue"`/`"black"`/`"red"`/
        /// `"green"`/`"colorless"` to pay with that color, or `"generic"` to pay a
        /// monocolored hybrid (`{2/W}`) with 2 generic mana. Empty = default to the
        /// first color option for each pip.
        /// Example: ["black"]
        #[serde(default)]
        hybrid_choices: Vec<String>,
        /// PB-RS2 (CR 107.4f via CR 602.2b/605.1a): For `activate_ability` or
        /// `tap_for_mana` on a source with a Phyrexian pip in its activation cost.
        /// One entry per Phyrexian pip, in cost order: `true` = pay 2 life, `false` =
        /// pay mana. Empty = default to paying with mana for each pip.
        /// Example: [true]
        #[serde(default)]
        phyrexian_life_payments: Vec<bool>,
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
        /// `discard_to_hand_size`.
        ///
        /// **Empty-string contract:** `#[serde(default)]` makes this field optional;
        /// when absent from JSON (or explicitly `""`) it deserializes to the empty
        /// string. An empty `action` is the canonical sentinel for a synthetic or
        /// informational `TurnBasedAction` that names no concrete turn-based action —
        /// e.g. the replay viewer's step-0 "initial game state" snapshot. No replay
        /// driver (`replay_harness`, `script_replay`, replay-viewer) currently reads
        /// this field; all `TurnBasedAction` entries are treated as informational and
        /// dispatch no engine `Command`. If a driver is ever wired to dispatch on
        /// `action`, it MUST treat the empty string as "no action" explicitly.
        #[serde(default)]
        action: String,
        player: Option<String>,
        cr_ref: Option<String>,
        note: Option<String>,
    },
}
// ── Supporting types ──────────────────────────────────────────────────────────
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
#[serde(deny_unknown_fields)]
pub struct ActionTarget {
    #[serde(rename = "type")]
    pub target_type: String,
    pub card: Option<String>,
    pub controller: Option<String>,
    pub player: Option<String>,
}
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
#[serde(deny_unknown_fields)]
pub struct ManaSource {
    pub card: String,
    pub tap: bool,
}
// SR-22: **not** `deny_unknown_fields`. The `resolution` block on `StackResolve`
// is pure documentation — the harness never dispatches on a `ResolutionEffect` —
// and corpus authors annotate it with extra descriptive keys the schema does not
// declare (`controller`, `choice`, `found`, `criteria`, `token`, `mana_paid`;
// e.g. `stack/012`, `stack/027`, `stack/045`, `baseline/009`). These are dead by
// definition; enforcing would redden dozens of scripts for no behaviour change.
// Noted as SR-22 follow-up.
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
#[serde(deny_unknown_fields)]
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
#[serde(deny_unknown_fields)]
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
#[serde(deny_unknown_fields)]
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
#[serde(deny_unknown_fields)]
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
#[serde(deny_unknown_fields)]
pub struct BlockerDeclaration {
    /// Name of the blocking creature (must be on the battlefield under the player's control).
    pub card: String,
    /// Name of the attacking creature being blocked.
    pub blocking: String,
}
