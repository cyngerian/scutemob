# Primitive Batch Plan: PB-RS3 — card-def `AtBeginningOfCombat` sweep (OOS-OS9-1)

<!-- last_updated: 2026-07-20 -->

**Generated**: 2026-07-20
**Task**: `scutemob-145`
**Branch**: `feat/pb-rs3-atbeginningofcombat-card-def-sweep-begincombat-collec`
**Primitive**: no new DSL surface. A missing **dispatch site**: `begin_combat` never scans card
definitions for `TriggerCondition::AtBeginningOfCombat`, so every card-def combat trigger in the
corpus is inert.
**Class**: CORRECTNESS (Invariant #9 — `helm_of_the_host` is `Complete`, passes `validate_deck`,
and its only real ability silently never fires)
**CR Rules**: 506.1, 507.1/507.2, 603.2, 603.3/603.3a/603.3b, 603.4, 101.4, 114.4, 903.3d
**Cards affected**: **6** defs carry the trigger (roster verified below). **2 flips**
(`loyal_apprentice`, `siege_gang_lieutenant`) + **1 integrity repair** (`helm_of_the_host`) +
3 that stay non-`Complete` on independent blockers.
**Dependencies**: PB-OS9 (`Condition::YouControlYourCommander`) — SHIPPED, verified present.
**Wire expectation**: **NO PROTOCOL bump, NO HASH bump.** See §9.
**Deferred items from prior PBs**: none carried into this PB. PB-RS2 filed OOS-RS2-1
(`TurnFaceUp` raw cost) — unrelated, stays queued.

---

## 0. Headline for the implementer

Two things in this plan are load-bearing and neither is obvious from the brief:

1. **The chain notes are CONFIRMED.** Exactly one broken hop, as filed. No re-scope. (§1)
2. **There are two mutually incompatible `ability_index` namespaces in this engine, and the
   sibling template uses the one that looks wrong.** `PendingTrigger.ability_index` means
   *"dense index into `characteristics.triggered_abilities`"* for `PendingTriggerKind::Normal`
   and *"index into `CardDefinition::effective_abilities()`"* for
   `PendingTriggerKind::CardDefETB`. The sweep uses `CardDefETB`, so it must enumerate
   `effective_abilities()` — **do not "fix" it to match `collect_triggers_for_event`.** Doing so
   silently breaks `loyal_apprentice`. Full evidence in §3. This is the single most likely defect
   in this PB and there is a test designed to catch it (§6, Test 2).

---

## 1. Chain verification — hop by hop, with file:line evidence

The brief asked for confirmation or falsification. **All five hops verified as filed. No
re-scope signal.**

| Hop | Triage claim | Verdict | Evidence |
| --- | --- | --- | --- |
| 1 | Engine-side `AtBeginningOfCombat` occurrences are two `HashInto` arms + the emblem call | **CONFIRMED** | `state/hash.rs:3175` (`TriggerEvent`, disc. 27), `state/hash.rs:5726` (`TriggerCondition`, disc. 13), `rules/turn_actions.rs:1689-1698`. Corpus-wide grep returns no other engine read site. |
| 2 | **BROKEN**: `begin_combat` collects emblem triggers only, no card-def scan | **CONFIRMED** | `rules/turn_actions.rs:1684-1703`. Body is: init `CombatState` (`:1686-1688`), `collect_emblem_triggers_for_event(..., TriggerEvent::AtBeginningOfCombat, Some(active))` (`:1693-1698`), push those (`:1699-1701`), `Vec::new()` (`:1702`). **No `state.objects` iteration, no `card_registry` read.** |
| 3 | Queue → stack works | **CONFIRMED** | `flush_pending_triggers` at `rules/abilities.rs:6950` drains `state.pending_triggers`, sorts by APNAP (`:6966-6975`), dispatches per `PendingTriggerKind`. `CardDefETB` is an established kind already driven by the four siblings. |
| 4 | Resolution + intervening-if works | **CONFIRMED** | `rules/resolution.rs:2018-2048` is the `CardDefETB` branch; it returns `(effect, intervening_if)` read from `def.effective_abilities(obj.is_transformed).get(ability_index)`. Proven end-to-end by `tests/primitives/pb_os9_lieutenant_commander_control.rs:614-669` and its negative twin at `:673+`, which push the exact `PendingTrigger` this PB will generate. |
| 5 | `Condition::YouControlYourCommander` works | **CONFIRMED** | Shipped by PB-OS9; exercised by both tests named above, both directions. |

**Additional confirmation the triage did not claim**: extra combat phases route through
`Step::BeginningOfCombat` (`rules/turn_structure.rs:62`, `:87`), and `end_combat` clears
`state.combat = None` (`rules/turn_actions.rs:2219`). So the sweep will fire once per combat
phase, which is what CR 506.1 + 603.2 require. See §5 for the trap this creates.

### 1a. One incidental defect found (out of scope, record it)

`rules/turn_actions.rs:689-690` — the end-step sweep's comment says it fires
`AtBeginningOfYourEndStep` **"or `AtBeginningOfEachEndStep` (for all players' permanents)"**, but
the code at `:720-723` matches **only** `AtBeginningOfYourEndStep`. Per
`memory/conventions.md` §"Aspirationally-wrong code comments are correctness hazards", this
comment is a lie. `TriggerCondition` has no `AtBeginningOfEachEndStep` variant
(`card_definition.rs:3351-3358` lists only `...EachUpkeep`), so the code is correct and the
**comment** is wrong. **Action**: fix the comment in this PB (one-line, zero behavior change,
same file the PB already touches). Do not add the variant.

---

## 2. CR rule text

**CR 506.1** — "The combat phase has five steps, which proceed in order: beginning of combat,
declare attackers, declare blockers, combat damage, and end of combat. The declare blockers and
combat damage steps are skipped if no creatures are declared as attackers or put onto the
battlefield attacking (see rule 508.8). There are two combat damage steps if any attacking or
blocking creature has first strike (see rule 702.7) or double strike (see rule 702.4)."

**CR 507.1** — "First, if the game being played is a multiplayer game in which the active
player's opponents don't all automatically become defending players, the active player chooses
one of their opponents. That player becomes the defending player. This turn-based action doesn't
use the stack. (See rule 506.2.)"

**CR 507.2** — "Second, the active player gets priority. (See rule 117, 'Timing and Priority.')"

> **Reading for this PB**: 507.1 is the *only* turn-based action of the beginning of combat step.
> "At the beginning of combat" triggered abilities are **not** turn-based actions — they are
> ordinary triggered abilities (CR 603.2) that trigger on entering the step and go on the stack
> under CR 603.3 before the active player receives priority under 507.2. The engine's existing
> shape (queue into `pending_triggers`, flush before priority) is therefore correct; the sweep
> just has to populate the queue.

**CR 603.3** — "Once an ability has triggered, its controller puts it on the stack as an object
that's not a card the next time a player would receive priority. See rule 117, 'Timing and
Priority.' The ability becomes the topmost object on the stack. It has the text of the ability
that created it, and no other characteristics. It remains on the stack until it's countered, it
resolves, a rule causes it to be removed from the stack, or an effect moves it elsewhere."

**CR 603.3a** — "A triggered ability is controlled by the player who controlled its source at the
time it triggered, unless it's a delayed triggered ability. To determine the controller of a
delayed triggered ability, see rules 603.7d–f."

**CR 603.3b** — "If multiple abilities have triggered since the last time a player received
priority, the abilities are placed on the stack in a two-part process. First, each player, in
APNAP order, puts each triggered ability they control with a trigger condition that isn't another
ability triggering on the stack in any order they choose. (See rule 101.4.) Second, each player,
in APNAP order, puts all remaining triggered abilities they control on the stack in any order they
choose. Then the game once again checks for and performs state-based actions until none are
performed, then abilities that triggered during this process go on the stack. This process repeats
until no new state-based actions are performed and no abilities trigger. Then the appropriate
player gets priority."

**CR 603.4** — "A triggered ability may read 'When/Whenever/At [trigger event], if [condition],
[effect].' When the trigger event occurs, the ability checks whether the stated condition is true.
The ability triggers only if it is; otherwise it does nothing. If the ability triggers, it checks
the stated condition again as it resolves. If the condition isn't true at that time, the ability
is removed from the stack and does nothing. Note that this mirrors the check for legal targets.
This rule is referred to as the 'intervening "if" clause' rule. (The word 'if' has only its normal
English meaning anywhere else in the text of a card; this rule only applies to an 'if' that
immediately follows a trigger condition.)"

**CR 101.4** — "If multiple players would make choices and/or take actions at the same time, the
active player (the player whose turn it is) makes any choices required, then the next player in
turn order (usually the player seated to the active player's left) makes any choices required,
followed by the remaining nonactive players in turn order. Then the actions happen simultaneously.
This rule is often referred to as the 'Active Player, Nonactive Player (APNAP) order' rule."

---

## 3. The sibling template — which one, and the index-space trap

### 3a. The four siblings

| # | Site | Step | Controller scope | Conditions matched |
| --- | --- | --- | --- | --- |
| S1 | `turn_actions.rs:267-320` (predicate `:294-301`) | Upkeep | **mixed** — `controller == active` for `...YourUpkeep`, unscoped for `...EachUpkeep` | 2 |
| S2 | `turn_actions.rs:426-475` (predicate `:442-458`) | PreCombatMain | **active only** — early `return None` at `:436-438` | 1 |
| S3 | `turn_actions.rs:490-540` (predicate `:506-522`) | PostCombatMain | **active only** — early `return None` at `:500-502` | 1 |
| S4 | `turn_actions.rs:696-742` (predicate `:709-726`) | End step | `controller == active` inside the predicate | 1 |

### 3b. Chosen template: **S3 (`postcombat_main_actions`)**

**Why S3 and not the others:**

- **Single condition, single controller scope.** `AtBeginningOfCombat` is documented as
  *"At the beginning of combat on your turn."* (`card_definition.rs:3357-3358`) — one variant,
  active-player-only. S1 is the wrong shape (it carries a two-condition disjunction with mixed
  scoping that this PB does not need).
- **Early-return controller filter.** S2/S3 reject non-active controllers *before* walking the
  ability list (`:500-502`), which is both cheaper and clearer than S4's in-predicate
  `&& controller == active`. Prefer S3's shape.
- **S3 is the only sibling whose step can occur more than once per turn** (CR 505.1a extra main
  phases) and its doc comment at `:478-487` already reasons explicitly about why no per-turn dedup
  bookkeeping is needed. `BeginningOfCombat` has the identical property (extra combats route
  through it — `turn_structure.rs:62`, `:87`), so S3's comment is the one to adapt. **S2's
  "fires ONCE per turn" reasoning must NOT be copied** — it is false for combat.
- S3 uses `obj.is_phased_in()` (`:495`); S1/S4 use the equivalent-but-longhand
  `!obj.status.phased_out`. Use `is_phased_in()`.

### 3c. THE INDEX-SPACE TRAP — read this before writing code

The siblings build `ability_index` by enumerating **`def.effective_abilities(is_transformed)`** —
the *raw* ability list, which also contains `Keyword`, `Static`, and `Activated` entries. That
looks like the exact bug PB-AC7 fixed, and an implementer who "corrects" it will break this PB.

**It is not a bug here.** The engine has two `ability_index` namespaces, selected by
`PendingTriggerKind`:

| Kind | `ability_index` means | Resolution reads |
| --- | --- | --- |
| `Normal` | dense index into `characteristics.triggered_abilities` | `resolution.rs:2185-2219` → `resolved.triggered_abilities.get(ability_index)` |
| **`CardDefETB`** | **index into `def.effective_abilities(is_transformed)`** | `resolution.rs:2018-2048` — comment at `:2019-2020` reads *"CardDefETB path: ability_index is into CardDef::abilities. Always use the card registry — never runtime triggered_abilities."* |

The sweep uses `PendingTriggerKind::CardDefETB`, so `effective_abilities()` enumeration is
**correct and required**. Corroborating evidence that the dense namespace would be wrong:

- `characteristics.triggered_abilities` is built by `build_ability_vectors`
  (`testing/replay_harness.rs:2109-2300+`) as a series of **per-`TriggerCondition` opt-in loops**
  (`WhenDies`→`SelfDies`, `WhenAttacks`→`SelfAttacks`, …). There is **no lowering arm for any
  step-based `TriggerCondition`** — no `AtBeginningOfYourUpkeep`, no `AtBeginningOfCombat`.
  So for all six roster cards, `characteristics.triggered_abilities` is **empty** with respect to
  this trigger. A dense-index sweep would find nothing at all.
- PB-AC7's regression file (`tests/primitives/pb_ac7_ability_index_desync.rs:1-23`) documents the
  *other* direction: a `Normal`-path consumer that wrongly used the raw namespace.

**`loyal_apprentice` is the discriminating card.** Its abilities are
`[Keyword(Haste), Triggered{AtBeginningOfCombat}]` (`loyal_apprentice.rs:54-90`). Correct
`ability_index` is **1**. If an implementer uses the dense namespace they get **0**, resolution
looks up `effective_abilities()[0]` = `Keyword(Haste)`, hits the `_ => None` arm at
`resolution.rs:2041`, and the trigger resolves as a **silent no-op** — fires, does nothing, no
error. `siege_gang_lieutenant` and `helm_of_the_host` both have the trigger at index 0 and would
**not** catch this. Test 2 in §6 exists solely to catch it.

---

## 4. Engine change (the only production edit)

### Change 1 — card-def sweep in `begin_combat`

**File**: `crates/engine/src/rules/turn_actions.rs`
**Function**: `begin_combat` (`:1684-1703`)
**Action**: insert the sweep **between** the `CombatState` init block (`:1686-1688`) and the
existing emblem block (`:1689-1701`). Adapt S3 verbatim, substituting
`TriggerCondition::AtBeginningOfCombat`.

Required properties, each traceable to a CR rule or a verified site:

1. **Scope**: `obj.zone == ZoneId::Battlefield && obj.is_phased_in()`. Matches all four siblings.
   CR 702.26d — phased-out permanents' abilities don't function. **Command zone is not scanned**
   by the card-def sweep; emblems there are the existing block's job (CR 114.4).
2. **Controller scope**: early `return None` when `controller != active`. CR 603.2 + the printed
   text "on your turn" on all three in-scope cards (§5, oracle-verified via MCP).
3. **Face-awareness**: `def.effective_abilities(obj.is_transformed)`. CR 712.8d/e — carry S3's
   PB-OS4b comment forward; a transformed permanent's combat triggers come from its back face.
4. **Index**: dense `enumerate()` index over `effective_abilities()`, per §3c.
5. **Two-phase collect-then-push**: build `Vec<(ObjectId, PlayerId, Vec<usize>)>` inside a block
   that borrows `&state.card_registry`, then push after the borrow ends. Non-negotiable — this is
   why all four siblings are shaped that way (borrowck).
6. **Push shape (SR-7)**: `PendingTrigger { ability_index, ..PendingTrigger::blank(obj_id, controller, PendingTriggerKind::CardDefETB) }`.
   Never construct `PendingTrigger` literally.
7. **Return**: `begin_combat` still returns `Vec::new()`. Queueing a trigger emits no
   `GameEvent` here — `flush_pending_triggers` emits the stack-placement events. Adding a new
   event would be a wire change (§9).

**Interleaving with the existing emblem block — no doubling, no drops:**

The two collections are **disjoint by zone and by mechanism**, so coexistence is structural
rather than something the code has to arbitrate:

- `collect_emblem_triggers_for_event` scans the **command zone** for emblems
  (CR 114.4) and matches on **`TriggerEvent::AtBeginningOfCombat`**.
- The new sweep scans the **battlefield** and matches on
  **`TriggerCondition::AtBeginningOfCombat`**.

No object is in both zones, and the two enums are distinct types with distinct dispatch. An
emblem cannot be picked up by the battlefield sweep and a battlefield permanent cannot be picked
up by the emblem scan. **Both push into the same `state.pending_triggers` deque**, which
`flush_pending_triggers` then APNAP-sorts as one batch (CR 603.3b) — correct and desirable.

**Ordering**: put the card-def sweep **before** the emblem block so that `pending_triggers`
receives battlefield triggers first. This is cosmetic (CR 603.3b lets the controller order their
own triggers freely, and `sort_by_key` at `abilities.rs:6970-6975` is a *stable* sort keyed only
on controller), but it must be pinned to keep replay hashes deterministic. Test 4 (§6) locks it.

**Do NOT put the sweep inside the `if state.combat.is_none()` guard.** The guard exists only to
avoid clobbering a `CombatState` already built by `combat.rs:59-61`. Nesting the sweep inside it
would silently drop all combat triggers on any combat phase where `state.combat` was already
`Some` — the exact extra-combat regression Test 5 checks for.

### Change 2 — comment repair

**File**: `crates/engine/src/rules/turn_actions.rs:689-690`
**Action**: correct the end-step sweep's comment per §1a. No behavior change.

### Change 3 — exhaustive-match sites

**None.** This PB adds no enum variant, no struct field, and no `GameEvent`. The standing
hazard list (`state/hash.rs` `HashInto`, `tools/replay-viewer/src/view_model.rs`,
`tools/tui/src/play/panels/stack_view.rs`) is **not** triggered — both
`TriggerEvent::AtBeginningOfCombat` and `TriggerCondition::AtBeginningOfCombat` already have
`HashInto` arms (`hash.rs:3175`, `:5726`), and `PendingTriggerKind::CardDefETB` /
`StackObjectKind` are pre-existing. `cargo build --workspace` remains the gate that proves it.

---

## 5. Scope: zones, controllers, and the oracle check

### 5a. Controller scoping — researched, not assumed

The brief correctly flagged this as the most likely correctness defect. **Answer:
active-player-only, and this is not a simplification.**

Three independent sources agree:

1. **The DSL variant's own doc**: `card_definition.rs:3357-3358` — *"At the beginning of combat on
   your turn."* The variant *is* the "your turn" form; there is no `AtBeginningOfEachCombat`.
2. **MCP oracle text for every in-scope card** (§5b) — all three read "on your turn".
3. **The existing emblem call** already passes `Some(active)` (`turn_actions.rs:1697`), so the
   card-def sweep matching that scope keeps the two halves consistent.

Corpus check: all six roster defs (§7) use the plain `AtBeginningOfCombat` variant, and every one
of their printed cards says "on your turn". **No card in the corpus needs an each-combat form.**
If a future card does (e.g. "at the beginning of each opponent's combat"), that is a **new
`TriggerCondition` variant and therefore a wire change** — out of scope here, and a stop-and-flag
condition under §9.

### 5b. Oracle verification of the three named targets (MCP `lookup_card`)

**`helm_of_the_host`** — `{4}` Legendary Artifact — Equipment.
> "At the beginning of combat on your turn, create a token that's a copy of equipped creature,
> except the token isn't legendary. That token gains haste.
> Equip {5}"

Current def (`helm_of_the_host.rs`): `Triggered { trigger_condition: AtBeginningOfCombat, effect:
CreateTokenCopy { source: EquippedCreature, except_not_legendary: true, gains_haste: true, .. },
intervening_if: None }` at `:27-42`, plus `Equip {5}` at `:44-59`. **The DSL is a faithful
translation and needs no change.** It has **no `completeness:` field** ⇒ `Complete` via
`#[default]` (`card_definition.rs:196-200`). This is the live-wrong card: it is deck-legal today
and does nothing.
**Required**: (a) the sweep makes it fire; (b) add an **explicit
`completeness: Completeness::Complete,`** line. The explicit form is already used by ≥20 defs in
the corpus, and making it explicit converts a silent default into a reviewed assertion.
**Edge case to assert, not assume**: `EffectTarget::EquippedCreature` with the Helm **unattached**
must resolve to nothing and create no token. Test 6 covers it.

**`loyal_apprentice`** — `{1}{R}` Creature — Human Artificer 2/1.
> "Haste
> Lieutenant — At the beginning of combat on your turn, if you control your commander, create a
> 1/1 colorless Thopter artifact creature token with flying. That token gains haste until end of
> turn."

Def matches (`loyal_apprentice.rs:54-90`): `Keyword(Haste)` at `abilities[0]`, `Triggered
{ AtBeginningOfCombat, CreateToken{Thopter 1/1, Flying+Haste}, intervening_if:
YouControlYourCommander }` at `abilities[1]`.
**Required**: flip `Completeness::partial(...)` (`:92-101`) → `Complete`, and delete the
"STILL BLOCKED" block at `:17-35` (it describes a gap this PB closes). The permanent-haste
`TokenSpec.keywords` fallback for "gains haste until end of turn" is the accepted,
already-reviewed approximation (rationale preserved in the file's `:7-15` comment — **keep that
half**, it is still true).

**`siege_gang_lieutenant`** — `{3}{R}` Creature — Goblin 2/2.
> "Lieutenant — At the beginning of combat on your turn, if you control your commander, create two
> 1/1 red Goblin creature tokens. Those tokens gain haste until end of turn.
> {2}, Sacrifice a Goblin: This creature deals 1 damage to any target."

Def matches (`siege_gang_lieutenant.rs:41-97`): `Triggered{...}` at `abilities[0]` with
`count: EffectAmount::Fixed(2)` and `intervening_if: YouControlYourCommander`; activated ability
at `abilities[1]`.
**Required**: flip `Completeness::partial(...)` (`:99-108`) → `Complete`; delete the "STILL
BLOCKED" block at `:13-21`; keep the haste-fallback rationale at `:6-11`.

### 5c. The other three roster members — do NOT flip, and one is a regression risk

- **`goblin_rabblemaster`** (`:28`) — trigger is a correct 1/1 Goblin token. Stays `partial`; the
  surviving blocker is the subtype-filtered forced-attack `GameRestriction` (`:91-97`).
  **Behavior strictly improves.** Update the note only if it claims the trigger doesn't fire.
- **`legion_warboss`** (`:26-54`) — trigger is correct. Stays `partial("Mentor keyword not in
  DSL")` (`:56`). The token's "attacks this combat if able" is still unexpressible (`:48`).
  **Behavior strictly improves.**
- **`mirage_phalanx`** (`:39-57`) — ⚠️ **REGRESSION RISK.** Its `AtBeginningOfCombat`
  `CreateTokenCopy` is authored **unconditionally** on Mirage Phalanx itself, but the oracle grants
  it only **while paired** (CR 702.94a). Today the def is protected by accident: the trigger never
  fires. After this PB it will fire **every combat, paired or not** — creating a token copy the
  card should not create.
  **Containment**: the def is already `Completeness::known_wrong(...)` (`:59-66`), so
  `validate_deck` rejects it and it cannot enter a real game (SR-2). **Exposure is zero, but the
  reasoning changes from "inert" to "gated".**
  **Required**: amend the `known_wrong` note to record that the trigger now *fires* and
  over-produces, so the next reader does not mistake `known_wrong` for `harmless`. Do not attempt
  the Soulbond while-paired ability grant — that is a separate primitive.

---

## 6. Mandatory tests

**File**: `crates/engine/tests/primitives/pb_rs3_at_beginning_of_combat_sweep.rs`
**Registration**: add `mod pb_rs3_at_beginning_of_combat_sweep;` to
`crates/engine/tests/primitives/main.rs` (alphabetical, after `mod pb_rs2_activated_pip_payment;`
at `:46`).
**SR-9a**: never create a top-level `crates/engine/tests/*.rs` —
`tests/no_stray_test_binaries.rs` fails the suite if one reappears. A dropped `mod` line silently
deletes coverage.
**Pattern to follow**: `tests/primitives/pb_os9_lieutenant_commander_control.rs` — same cards,
same helpers (`all_cards`, `commander_def`, `load_defs_from`, `enrich_spec_from_def`,
`GameStateBuilder`, `find_object`, `pass_all`). Reuse its scaffolding wholesale.

**Step 0 (before any production edit)**: write Test 1 and **verify it FAILS against pre-fix
HEAD**. Record the observed failure in the implement commit message. A probe that passes before
the fix is not a probe.

| # | Test | What it proves | CR |
| --- | --- | --- | --- |
| 1 | `test_helm_of_the_host_creates_token_copy_at_beginning_of_combat` | **The probe.** Helm attached to a vanilla creature, p1 active, advance into `Step::BeginningOfCombat` **through the real step transition** (not a manual `pending_triggers` push — that is what PB-OS9 already did and it cannot detect this bug). Assert a non-legendary hasty token copy exists. Must fail pre-fix (0 tokens), pass post-fix. | 603.2, 603.3 |
| 2 | `test_loyal_apprentice_trigger_uses_carddef_ability_index_namespace` | **The index-space discriminator (§3c).** Loyal Apprentice (`Triggered` at `abilities[1]`, behind `Keyword(Haste)`) + commander on battlefield. Assert exactly one Thopter token. Fails if the implementer used the dense `triggered_abilities` namespace, because index 0 lands on `Keyword(Haste)` → `_ => None` → silent no-op. **Name the hazard in the doc comment** so a future reader cannot "simplify" it away. | 603.3a |
| 3a | `test_siege_gang_lieutenant_intervening_if_holds_creates_two_goblins` | Intervening-if **true** direction, driven end-to-end through the step transition. Commander on battlefield at both trigger time and resolution → 2 Goblin tokens. | 603.4, 903.3d |
| 3b | `test_siege_gang_lieutenant_intervening_if_fails_when_commander_removed` | Intervening-if **false** direction. Commander leaves the battlefield after the trigger queues but before it resolves → **0** tokens. Mirrors `pb_os9_lieutenant_commander_control.rs:673+` but via the real sweep. | 603.4 |
| 4 | `test_at_beginning_of_combat_multiplayer_only_active_player_triggers` | **APNAP / controller scoping.** 4-player game; give p1 (active) **and** p2 (non-active) each a Loyal Apprentice, both controlling their commanders. Assert **only p1's** trigger queues and resolves (1 Thopter, controlled by p1). **Honest framing**: because `AtBeginningOfCombat` is active-player-only, APNAP ordering is *trivially* satisfied — the batch is single-controller by construction. This test's real content is the controller filter; assert that explicitly rather than claiming to test ordering it cannot exercise. Additionally assert the resulting `PendingTrigger`s all carry `controller == active`, so the APNAP sort at `abilities.rs:6970-6975` is a documented no-op rather than an untested one. | 101.4, 603.3b |
| 5 | `test_emblem_and_carddef_combat_triggers_coexist` | **No doubling, no drops.** Basri Ket emblem in p1's command zone (see `basri_ket.rs:71-85`) **plus** a battlefield Helm of the Host. Assert **both** effects occur, **exactly once each**. Then assert the queue ordering is deterministic (card-def before emblem, per §4) so replay hashes are stable. | 114.4, 603.3b |
| 6 | `test_helm_of_the_host_unattached_creates_no_token` | Negative edge: Helm on the battlefield **unattached**. The trigger fires (correct — no intervening-if), but `EffectTarget::EquippedCreature` resolves to nothing → **0** tokens, no panic, no diagnostic. Guards the `CreateTokenCopy` source-resolution path the sweep now reaches for the first time. | 603.2, 702.6 |
| 7 | `test_at_beginning_of_combat_fires_in_extra_combat_phase` | **Extra-combat behavior with CR citation.** Drive a second combat phase in one turn (extra combats route through `Step::BeginningOfCombat` — `turn_structure.rs:62`, `:87`) and assert the Helm trigger fires **again** — 2 tokens total across the turn. CR 506.1 defines the phase structure and CR 603.2 makes the ability trigger on **each** occurrence of its event; nothing makes it once-per-turn (`once_per_turn: false` on every roster def). **This test also guards the `state.combat.is_none()` trap in §4** — if the sweep is nested inside that guard, the second combat drops its trigger and this test fails. | 506.1, 603.2 |

**Also update**: `tests/primitives/pb_os9_lieutenant_commander_control.rs`. Its file-level doc
(`:17-24`) and the doc comments at `:609-613` state that the `AtBeginningOfCombat` sweep does not
exist and that the tests therefore push the `PendingTrigger` by hand. That becomes false with this
PB. Per `memory/conventions.md` §"Aspirationally-wrong code comments", **fix the comments** and
point them at the PB-RS3 end-to-end tests. Keep the manual-push tests themselves — they are still
valid unit isolation of the resolution path — but they must stop claiming the gap is open.

---

## 7. Roster sweep (SR-36 — enumerate `all_cards()`, never grep)

**File**: `crates/engine/tests/core/pb_rs3_combat_trigger_roster.rs`
**Registration**: add `mod pb_rs3_combat_trigger_roster;` to `crates/engine/tests/core/main.rs`.
**Pattern**: copy `crates/engine/tests/core/pb_rs1_roster_sweep.rs` verbatim and re-point it.
That file's two design choices are both required here:

1. **Enumerate `all_cards()`**, not grep. Grep misses macro-generated / re-exported defs and
   over-counts comment text — and this corpus is full of comment text naming
   `AtBeginningOfCombat` (six of the grep hits below are *comments*, not code).
2. **Walk `serde_json::to_value(&def)` recursively** rather than matching top-level abilities.
   A `TriggerCondition` can sit inside a back face, a mode, or a nested ability list; a shallow
   scan under-counts. `pb_rs1_roster_sweep.rs:19-30`'s `contains_key` helper works directly
   (`TriggerCondition` is externally tagged, so the variant name is an object key).

The test must `eprintln!` the full sorted roster and assert a **non-vacuity floor** so a serde
rename cannot make it silently report nothing (the hazard
`core/effect_choose_gate.rs::stub_gates_are_not_vacuous` guards).

### Predicted roster — **6 cards**, and the triage's "3" was an undercount of the *roster*, not of the *yield*

Grep calibration (code sites only, comments excluded) found exactly six defs:

| Card | Ability index | Post-PB completeness | Note |
| --- | --- | --- | --- |
| `helm_of_the_host` | 0 | **`Complete`** (explicit) | integrity repair — was `Complete` by default and inert |
| `loyal_apprentice` | **1** | **`Complete`** (flip) | index-space discriminator |
| `siege_gang_lieutenant` | 0 | **`Complete`** (flip) | intervening-if both directions |
| `goblin_rabblemaster` | — | stays `partial` | forced-attack `GameRestriction` |
| `legion_warboss` | — | stays `partial` | Mentor keyword |
| `mirage_phalanx` | — | stays `known_wrong` | Soulbond grant; **now over-produces** (§5c) |

**Honesty note for the close-out**: the triage's R3 row said "2 flips + helm repaired", which is
**exactly right** and is confirmed here. The "3 cards" figure in the brief referred to the
*verification targets*, not the roster. The roster is 6; the yield is 2 flips + 1 repair. Per
`feedback_pb_yield_calibration`, **do not inflate the close-out to 6.** Report roster 6 / flips 2
/ repairs 1 as three separate numbers.

`basri_ket.rs` uses `TriggerEvent::AtBeginningOfCombat` (emblem path, `:78`) — it is **not** part
of the card-def roster and must not be counted in it, but it *is* the fixture for Test 5.

---

## 8. Complete-by-default hazard assessment

### 8a. The mechanism, measured

`Completeness` derives `Default` with `#[default] Complete` (`card_definition.rs:196-200`). A def
that omits `completeness:` is therefore **asserted fully correct by silence**. `helm_of_the_host`
is the proof that this is not theoretical.

Measured corpus facts (this session):

- ~**1,136 of 1,804** defs are effectively `Complete` (`docs/authoring-status.md`), so ~668 carry
  an explicit non-`Complete` marker.
- The explicit `Completeness::Complete` form **is** used — grep found ≥20 defs writing it — so the
  corpus is **inconsistent**: some Complete defs assert it, most default into it.
- **567 def files carry a `// TODO:` comment.** That set overlaps both markers.

### 8b. Existing gates and the exact hole

| Gate | Class it guards |
| --- | --- |
| `core/card_registry_gate.rs::test_inert_definitions_are_marked_incomplete` | def has rules text but **zero abilities** |
| `core/completeness_deviation_scan.rs` | def source contains **deviation language** ("simplif", "modeled as", "approximat", "deviation") without a marker |

**Neither would have caught `helm_of_the_host`.** It has abilities, and it contains no deviation
language and no TODO — the def is a *faithful* translation. Its failure was that the **engine had
no dispatch site for the trigger condition it used**. No textual scan can see that.

### 8c. Proposed gate — trigger-condition dispatch coverage (the SR-5 pattern)

The generalizable form of the helm bug is: *a card-def declares a `TriggerCondition` that no
engine code ever dispatches.* Following the `state::keyword_registry::handling` precedent (SR-5),
add an **exhaustive registry** classifying every `TriggerCondition` variant as `Dispatched { site }`
/ `HandledElsewhere { why }` / `Undispatched { seed }`. Adding a variant becomes a compile error
until classified, and a test asserts **no `Complete` def uses an `Undispatched` variant**.

**File** (proposed): `crates/engine/tests/core/trigger_condition_dispatch_coverage.rs`, backed by
a `pub(crate)` registry fn next to the enum.

**Filing thresholds** (state these in the close-out; do not silently skip the measurement):

- **≥1** `Complete` def using a `TriggerCondition` with no dispatch site → **file a seed
  immediately, class CORRECTNESS.** This is a live Invariant #9 violation by definition. (Today,
  before this PB, that count is ≥1 — helm.)
- **Secondary measurement**, cheap to compute in the same test: count defs that are effectively
  `Complete` **and** carry `// TODO:` / `ENGINE-BLOCKED` text. If **>10**, file a separate
  MEDIUM seed for a marker-consistency sweep.
- **Tertiary, advisory only**: the ~1,116 Complete-by-omission defs. Do **not** file a seed to
  make `completeness:` mandatory on all of them — that is a 1,800-file mechanical churn with no
  correctness content, and the two thresholds above catch the cases that actually bite.

**Scope discipline**: this PB should **measure and file**, not build. Implementing the registry is
a micro-PB of its own (`memory/conventions.md` §"Implement-phase default-to-defer"). The
deliverable here is the number plus a filed seed.

---

## 9. Wire expectation

**NO `PROTOCOL_VERSION` bump. NO `HASH_SCHEMA_VERSION` bump.** Current live values: PROTOCOL
**27**, HASH **63**.

Justification: this PB adds a **collection call inside an existing turn-based action**. It adds no
`Command` field, no `Effect` variant, no `GameEvent` variant, no `PendingTriggerKind`, no
`TriggerCondition` variant, and reshapes no serialized struct. Both relevant `HashInto` arms
already exist (`hash.rs:3175`, `:5726`). Scenario/state **hash values** will move for any script
involving a combat trigger — that is expected and correct; the pinned artifacts are schema
fingerprints, not state digests.

**What WOULD force a bump — stop and re-scope if you find yourself doing any of these:**

1. Adding a `TriggerCondition` variant (e.g. an each-combat / opponent-combat form). §5a explains
   why no corpus card needs one.
2. Adding a `PendingTriggerKind` variant instead of reusing `CardDefETB`.
3. Adding a field to `PendingTrigger` (SR-7 — per-kind payload belongs in `TriggerData`).
4. Emitting a new `GameEvent` from `begin_combat`.
5. Any `TriggerData` variant.

**If the machine gate demands a fingerprint re-pin anyway, that contradicts this section and is a
stop-and-flag condition (AC 5127). Do not silently re-pin.** Report it to the coordinator with the
diff that caused it.

---

## 10. Verification checklist

- [ ] Test 1 (helm probe) written **first** and observed **FAILING** on pre-fix HEAD; failure
      recorded in the implement commit message
- [ ] Sweep added to `begin_combat`, S3-shaped, **outside** the `state.combat.is_none()` guard
- [ ] `ability_index` enumerates `def.effective_abilities(obj.is_transformed)` (§3c) and the
      push uses `PendingTrigger::blank(..., PendingTriggerKind::CardDefETB)` (SR-7)
- [ ] Emblem block preserved and still runs; card-def sweep ordered before it
- [ ] `helm_of_the_host` given an explicit `completeness: Completeness::Complete,`
- [ ] `loyal_apprentice` + `siege_gang_lieutenant` flipped `partial` → `Complete`; stale
      "STILL BLOCKED" comment blocks deleted; haste-fallback rationale kept
- [ ] `mirage_phalanx` `known_wrong` note amended to record that it now over-produces (§5c)
- [ ] `turn_actions.rs:689-690` end-step comment corrected (§1a)
- [ ] `pb_os9_lieutenant_commander_control.rs` doc comments corrected (§6)
- [ ] All 8 tests in `tests/primitives/pb_rs3_at_beginning_of_combat_sweep.rs`, registered in
      `primitives/main.rs`; **no** top-level `tests/*.rs` created (SR-9a)
- [ ] Roster sweep in `tests/core/pb_rs3_combat_trigger_roster.rs`, registered in `core/main.rs`;
      prints the full roster, asserts a non-vacuity floor
- [ ] §8 hazard measured; seed filed if either threshold is crossed
- [ ] `PROTOCOL_VERSION == 27` and `HASH_SCHEMA_VERSION == 63` **unchanged**
- [ ] `cargo build --workspace` clean (the exhaustive-match gate)
- [ ] `cargo test --all` green; `cargo clippy -- -D warnings` clean
- [ ] `cargo fmt --check` **and** `tools/check-defs-fmt.sh` (SR-35 — `cargo fmt` checks **zero**
      of the 1,804 defs and still exits 0)
- [ ] Close-out reports roster (6) / flips (2) / repairs (1) as **three separate numbers**

---

## 11. Honest yield estimate

- **Clean flips**: **2** — `loyal_apprentice`, `siege_gang_lieutenant`. Coverage 1,136 → **1,138**
  of 1,804 (63.0% → **63.1%**).
- **Integrity repairs**: **1** — `helm_of_the_host`, a deck-legal `Complete` card whose only real
  ability silently never fired. This is the actual value of the PB; it buys Invariant #9 integrity,
  not coverage.
- **Strict behavior improvements without a flip**: **2** — `goblin_rabblemaster`, `legion_warboss`
  start producing their tokens.
- **Newly-gated wrongness**: **1** — `mirage_phalanx` (contained by `known_wrong`; §5c).

This matches the triage's discounted prediction exactly. Per `feedback_pb_yield_calibration`, no
upward revision: the roster is 6 but the flip count is 2, and conflating them would be the
classic 2-3× overcount.

---

## 12. Risks

| # | Risk | Severity | Mitigation |
| --- | --- | --- | --- |
| R1 | **Index-space "correction"** — implementer switches to the dense `triggered_abilities` namespace to match `collect_triggers_for_event`, silently no-opping `loyal_apprentice` | **HIGH** | §3c; Test 2 is built to fail on exactly this; the doc comment must name the hazard |
| R2 | **Sweep nested inside `if state.combat.is_none()`** — drops all triggers on extra combats | **HIGH** | §4 item; Test 7 |
| R3 | **`mirage_phalanx` starts over-producing** — inert-by-accident becomes actively wrong | MEDIUM | Contained by `known_wrong` + `validate_deck` (SR-2); note amended so the containment is deliberate rather than lucky |
| R4 | **Copying S2's "fires ONCE per turn" comment** into a step that can recur | MEDIUM | §3b; Test 7 |
| R5 | **Controller scope wrong** (sweeping all players) — every opponent's Helm fires on your turn | MEDIUM | §5a: three independent confirmations; Test 4 |
| R6 | **Golden-script drift** — scripts in `test-data/generated-scripts/` that transit a combat phase with a roster card on the battlefield will change behavior | MEDIUM | Run the full script corpus; any diff must be reviewed as a **correction**, with a CR 603.2 citation. SR-9c: a new assertion path must be implemented in `check_assertions`, never skipped |
| R7 | **Unattached Helm panics or emits a diagnostic** — a `CreateTokenCopy` source-resolution path the sweep reaches for the first time | LOW | Test 6; if it does fire a diagnostic, classify per SR-4 (LKI-fizzle vs engine-bug) rather than silencing it |
| R8 | **Test 4 mislabeled as an APNAP-ordering test** when the batch is single-controller by construction | LOW | §6 Test 4 states the honest framing and adds the controller assertion that makes the no-op explicit |
| R9 | Scope creep into the §8c dispatch-coverage registry | LOW | §8c: measure and file, do not build (conventions §default-to-defer) |
