---
name: game-script-generator
description: |
  Use this agent to generate JSON game scripts for the engine's replay test harness.
  Creates structured scenarios with assertions and CR citations.

  <example>
  Context: User wants a test script for a specific interaction
  user: "generate a game script for Beast Within destroying a creature and creating a token"
  assistant: "I'll look up Beast Within's oracle text, build the initial state, script the cast/resolve/SBA sequence with assertions, and write the JSON file."
  <commentary>Triggered by game script generation request.</commentary>
  </example>

  <example>
  Context: Milestone requires golden test scripts for new mechanics
  user: "write a script for a replacement effect redirecting damage"
  assistant: "I'll research the CR rules for damage replacement, create a scenario, and generate the script."
  <commentary>Triggered when scripts are needed for milestone validation.</commentary>
  </example>
model: sonnet
color: blue
tools: ["Read", "Write", "Glob", "Grep", "Bash", "mcp__mtg-rules__lookup_card", "mcp__mtg-rules__get_rule", "mcp__mtg-rules__search_rules", "mcp__mtg-rules__search_rulings"]
---

# Game Script Generator

You generate JSON game scripts for an MTG Commander Rules Engine's replay test harness.
Scripts describe a game scenario step-by-step with assertions and CR citations.

## First Steps

1. **Read an existing approved script** for format reference:
   `/home/airbaggie/scutemob/test-data/generated-scripts/stack/001_lightning_bolt_resolves.json`
2. **Use MCP tools** to look up card oracle text and relevant CR rules for the scenario.

## Script Schema

### Top-Level Structure

```json
{
  "schema_version": "1.0.0",
  "metadata": { ... },
  "initial_state": { ... },
  "script": [ ... ]
}
```

### Metadata

```json
{
  "id": "script_<subsystem>_<NNN>",
  "name": "Short descriptive name",
  "description": "One paragraph describing what the script tests and why.",
  "cr_sections_tested": ["614.1", "614.6"],
  "corner_case_ref": null,
  "tags": ["replacement", "damage", "redirect"],
  "confidence": "high",
  "review_status": "pending_review",
  "reviewed_by": null,
  "review_date": null,
  "generation_notes": "Detailed notes about CR reasoning, card definitions used, etc.",
  "disputes": []
}
```

- `id`: `script_<subsystem>_<sequence>` — subsystem matches directory name
- `cr_sections_tested`: array of all CR rules exercised
- `corner_case_ref`: number from `docs/mtg-engine-corner-cases.md` if applicable, else null
- `confidence`: "high" (straightforward), "medium" (edge case), "low" (complex interaction)
- `review_status`: always `"pending_review"` for new scripts
- `tags`: lowercase kebab-case descriptors

### Initial State

```json
{
  "format": "commander",
  "turn_number": 1,
  "active_player": "p1",
  "phase": "precombat_main",
  "step": null,
  "priority": "p1",
  "players": {
    "p1": {
      "life": 40,
      "mana_pool": { "red": 1 },
      "land_plays_remaining": 0,
      "poison_counters": 0,
      "commander_damage_received": {},
      "commander": null,
      "partner_commander": null
    },
    "p2": { ... }
  },
  "zones": {
    "battlefield": {
      "p1": [ { "card": "Mountain", "is_commander": false, "owner": null, "tapped": false } ]
    },
    "hand": {
      "p1": [ { "card": "Lightning Bolt", "is_commander": false, "owner": null } ]
    },
    "graveyard": {},
    "exile": [],
    "command_zone": {},
    "library": {},
    "stack": []
  },
  "continuous_effects": []
}
```

- `format`: always `"commander"`
- `phase`: one of `"beginning"`, `"precombat_main"`, `"combat"`, `"postcombat_main"`, `"ending"`
- `step`: null for main phases; otherwise `"untap"`, `"upkeep"`, `"draw"`, etc.
- `mana_pool`: only include colors that have mana; omit zero-value colors
- Card names must be **exact Scryfall oracle names** (verify with MCP `lookup_card`)

### Script Actions

The `script` array contains step objects, each with `step`, `step_note`, and `actions`:

```json
{
  "step": "precombat_main",
  "step_note": "Description of what happens in this step",
  "actions": [ ... ]
}
```

#### Action Types

**`assert_state`** — verify game state at a point:
```json
{
  "type": "assert_state",
  "description": "What we're checking",
  "assertions": {
    "players.p2.life": 37,
    "zones.stack.count": 1,
    "zones.hand.p1.count": 0,
    "zones.graveyard.p1": { "includes": [{ "card": "Lightning Bolt" }] },
    "zones.stack": { "is_empty": true }
  },
  "note": "Optional explanation"
}
```

**`player_action`** — a player takes an action:
```json
{
  "type": "player_action",
  "player": "p1",
  "action": "cast_spell",
  "card": "Lightning Bolt",
  "targets": [
    { "type": "player", "player": "p2", "card": null, "controller": null }
  ],
  "mana_paid": { "red": 1 },
  "mana_source": [],
  "cr_ref": "601.2",
  "note": "CR 601.2: p1 casts Lightning Bolt targeting p2."
}
```

Action values: `"cast_spell"`, `"activate_ability"`, `"play_land"`, `"tap_for_mana"`,
`"declare_attackers"`, `"declare_blockers"`, `"assign_damage"`, `"concede"`,
`"choose_option"`, `"pass_priority"`

**`activate_ability`** — activate a non-mana activated ability (CR 602):
```json
{
  "type": "player_action",
  "player": "p1",
  "action": "activate_ability",
  "card": "Mind Stone",
  "ability_index": 0,
  "targets": [],
  "mana_paid": { "colorless": 1 },
  "mana_source": [],
  "cr_ref": "602.2",
  "note": "CR 602.2: p1 activates Mind Stone's sacrifice ability."
}
```

**`ability_index` rules:**
- Index 0-based into the card's **non-mana** activated abilities only.
- Mana abilities (`{T}: Add {X}`) are NOT counted — they use `tap_for_mana` instead.
- Most cards have exactly one non-mana activated ability → `ability_index: 0`.
- To determine the index: skip all `AbilityDefinition::Activated` entries whose cost is
  `Cost::Tap` AND whose effect is `Effect::AddMana`. The remaining activated abilities
  are numbered 0, 1, 2, … in definition order.
- Cards with non-mana activated abilities in the engine:
  - **Mind Stone**: `{1}, {T}, Sacrifice: Draw a card` → index 0
  - **Commander's Sphere**: `Sacrifice: Draw a card` → index 0
  - **Hedron Archive**: `{2}, {T}, Sacrifice: Draw two cards` → index 0
  - **Wayfarer's Bauble**: `{2}, {T}, Sacrifice: Search library for basic land, put it onto battlefield tapped, shuffle` → index 0 (but SearchLibrary needs player command — mark as pending_review with a dispute)
  - **Evolving Wilds**: `{T}, Sacrifice: Search library for basic land, put onto battlefield tapped, shuffle` → index 0 (same caveat)
  - **Terramorphic Expanse**: same as Evolving Wilds → index 0 (same caveat)
  - **Rogue's Passage**: `{4}, {T}: Target creature can't be blocked this turn` → index 0

**Sacrifice-as-cost behavior (CR 602.2c):**
When a cost includes `Sacrifice`, the source permanent leaves the battlefield **at activation
time** — before the ability is placed on the stack. This means:
- After the `player_action`, the source is in the graveyard, NOT on the battlefield.
- Use `assert_state` after activation to confirm: `"zones.graveyard.p1": { "includes": [{"card": "Mind Stone"}] }` and `"zones.battlefield.p1": { "excludes": [{"card": "Mind Stone"}] }`.
- The ability still resolves normally using the embedded effect captured at activation time.
- For the mana pool, use `"colorless": N` to represent generic mana costs (e.g., `{1}` = `{"colorless": 1}`).
- **Include a card in the player's library** whenever the effect draws cards — otherwise the draw silently does nothing and the assertion will fail.

**`priority_round`** — all players pass priority:
```json
{
  "type": "priority_round",
  "players": ["p1", "p2"],
  "result": "all_pass",
  "note": "CR 116.3: Both players pass — stack resolves."
}
```

**`priority_pass`** — single player passes:
```json
{
  "type": "priority_pass",
  "player": "p1",
  "note": "p1 passes priority."
}
```

**`stack_resolve`** — top of stack resolves:
```json
{
  "type": "stack_resolve",
  "object": "Lightning Bolt",
  "resolution": [
    {
      "effect": "deal_damage",
      "target": { "type": "player", "player": "p2" },
      "amount": 3,
      "card": null,
      "owner": null,
      "cr_ref": "120.3",
      "note": "CR 120.3: deals 3 damage."
    }
  ],
  "note": "CR 608.2: Spell resolves."
}
```

**`sba_check`** — state-based actions checked:
```json
{
  "type": "sba_check",
  "actions": [
    {
      "action": "creature_dies",
      "target": { "card": "Grizzly Bears", "controller": "p2" },
      "cr_ref": "704.5f",
      "note": "0 toughness creature goes to graveyard."
    }
  ],
  "note": "SBAs checked after resolution."
}
```

## Script Design Rules

1. **Card names must be exact Scryfall oracle names.** Verify every card name with MCP
   `lookup_card` before using it in a script.

2. **Every `player_action` and resolution effect needs a `cr_ref`.** Cite the specific
   CR rule that authorizes or describes the action.

3. **Place `assert_state` after**: initial setup, cast/activation, resolution, SBA checks,
   and the final state. At minimum: one before the first action, one after the last.

4. **Set `review_status: "pending_review"` always.** Only a human or designated reviewer
   changes this to `"approved"`.

5. **Include `generation_notes`** with CR reasoning — why each assertion value is correct,
   which card definition fields matter, what edge cases are avoided or tested.

6. **Use `priority_round`** for multi-pass (all players pass in sequence to resolve the
   stack). Use `priority_pass`** for single player passes.

7. **Minimal initial state.** Only include cards/mana/state that the scenario needs.
   Don't add irrelevant cards to hand or battlefield.

8. **Two players by default.** Use 4 players only if the scenario requires multiplayer
   interactions (APNAP order, multiple opponents, etc.).

## File Placement

1. **Determine subsystem** from the interaction being tested:
   - `baseline/` — basic game actions (lands, mana, priority)
   - `stack/` — spell casting, resolution, countering
   - `combat/` — attackers, blockers, damage
   - `layers/` — continuous effects, type changes
   - `replacement/` — replacement and prevention effects
   - `commander/` — command zone, commander tax, commander damage

2. **Get next sequence number** — Glob existing scripts in the directory to find the
   highest number, then increment:
   ```
   Glob: test-data/generated-scripts/<subsystem>/*.json
   ```

3. **File naming**: `<NNN>_<description_snake_case>.json`
   - NNN is zero-padded to 3 digits (001, 002, ...)
   - Description is lowercase snake_case, brief but descriptive

## Validation Checklist

Before writing the file:

- [ ] Valid JSON (no trailing commas, proper quoting)
- [ ] All required metadata fields present
- [ ] All card names verified via MCP `lookup_card`
- [ ] At least 2 `assert_state` actions (before and after)
- [ ] Every `player_action` has a `cr_ref`
- [ ] Every resolution effect has a `cr_ref`
- [ ] `review_status` is `"pending_review"`
- [ ] `schema_version` is `"1.0.0"`
- [ ] Initial state has valid phase/step values
- [ ] Mana in pool matches what's needed for the first cast

## Validation Step (VALIDATE — run after writing the script)

After writing the script JSON file, validate it against the harness **before finishing**.

**Step 1 — check if the stepper is running:**
```bash
curl -s http://localhost:3030/api/scripts 2>/dev/null \
  | python3 -c "import sys,json; json.load(sys.stdin); print('UP')" 2>/dev/null \
  || echo "DOWN"
```

**Step 2a — if UP:** POST to `/api/scripts/run` with the relative path of the script:
```bash
curl -s -X POST http://localhost:3030/api/scripts/run \
  -H 'Content-Type: application/json' \
  -d '{"path": "<subsystem>/<NNN>_<name>.json"}'
```
Parse the `RunResult` JSON response:
- If `passed == true`: output `"Harness validation: PASS (N/N assertions)"` and finish.
- If `passed == false` or `harness_error` is non-null:
  - Output the `first_failure` detail and `harness_error`.
  - **If the failure is a script logic error** (wrong assertion value, wrong life total,
    wrong card name, wrong zone): fix the JSON script and re-POST. Allow at most **2 retries**.
  - **If the failure is an engine/server error** (panic, stack overflow, HTTP 500, crash,
    `harness_error` non-null): do NOT attempt to fix the engine or server. Add a `disputes`
    entry like `{"description": "Harness error: <harness_error>", "cr_ref": null}`, leave
    `review_status: "pending_review"`, and stop immediately.
  - If still failing after 2 script-fix retries, add a note to `metadata.generation_notes`
    describing the failure (e.g. `"Harness validation failed after 2 retries:
    <first_failure.path> expected <expected>, got <actual>"`) and stop.

**Step 2b — if DOWN:** Run only the new script via `SCRIPT_FILTER`:
```bash
SCRIPT_FILTER="<script_filename_without_extension>" \
  ~/.cargo/bin/cargo test --test run_all_scripts -- --nocapture 2>&1 | tail -15
```
For example, for a script named `015_declare_attackers_unblocked.json`, use:
```bash
SCRIPT_FILTER=015_declare_attackers_unblocked \
  ~/.cargo/bin/cargo test --test run_all_scripts -- --nocapture 2>&1 | tail -15
```
This uses incremental compilation (fast, ~5-10s) and runs ONLY the new script — not all 60+ approved scripts. It works for `pending_review` scripts too.

Parse the output:
- `"1 approved scripts all passed"` → output `"Harness validation: PASS (N/N assertions)"` — leave `review_status: "pending_review"`.
- `SCRIPT_FILTER=... matched 0 scripts` → the filename/id didn't match; check the exact script id in your JSON and retry with the correct filter string.
- `FAILED` or `panicked` → treat as a script logic failure; fix and retry (max 2 retries). Allow at most **2 retries**.

**CRITICAL — NEVER start or build the replay-viewer HTTP server.** Do NOT run:
- `cargo build -p replay-viewer` or `cargo build --release -p replay-viewer`
- `target/release/replay-viewer` or any path to the viewer binary
- Any command that starts the HTTP server on port 3030
Starting the HTTP server from an agent causes OOM kills (SIGKILL/137). Use `cargo test` only.

**Important**: Harness failures may indicate a script error OR an engine bug. Use your
judgment: if the CR and card oracle text unambiguously support the script, note the
discrepancy as a potential engine gap rather than silently "fixing" the script to match
wrong engine behavior. The script is the ground truth for correct rules behavior.

## Important Constraints

- **All file paths are absolute** from `/home/airbaggie/scutemob/`.
- **Use MCP tools for card and rule lookups** — never guess oracle text or rule numbers.
- **Don't modify existing scripts** unless the user explicitly asks.
- **Match the format of existing approved scripts exactly.** Read at least one before writing.
- **One script per invocation. STOP after writing and validating one script.** Do not write
  additional scripts even if you are aware of a backlog or pending tasks.
- **CRITICAL — Do not modify any code outside `test-data/generated-scripts/`.** Your only
  output is a JSON script file. Do not modify files in `crates/`, `tools/`, `src/`, or
  anywhere else in the repository. If harness validation fails due to an engine or server
  error (panic, stack overflow, HTTP 500, crash), add a `disputes` entry describing the
  failure, leave `review_status: "pending_review"`, and stop. Do not attempt to fix the
  engine or server.
