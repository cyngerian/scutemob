# PB-DX24 — Stage 0: premise re-verified at HEAD

**Task**: `scutemob-202` · **Branch**: `feat/pb-dx24-the-lowering-drops-triggerzone-the-two-index-spaces-`
**Seeds**: `OOS-DX1-3` (the lowering drops `trigger_zone`) + `OOS-DX1-4` (the two index spaces
disagree) · **Queue**: `memory/primitives/seed-rerank-2026-08-02.md` §4 rank 6
**HEAD at stage 0**: `cceda74c` (merge-base with `main`; branch had no commits before this file)

Every line number below was re-cited **at this HEAD**, not copied from the v3 census. The census
cited `replay_harness.rs:3034` and `:3249-3288`; PB-DX20/DX21 edited files since, so the numbers
below supersede those.

---

## 1. Baseline, measured BEFORE any edit

`cargo test --workspace --no-fail-fast` to a file
(`scratchpad/baseline-preedit.txt`, 4,945 lines):

| metric | value |
|---|---|
| passed | **4,413** |
| failed | **0** |
| ignored | **5** |

Matches the CLAUDE.md pin for `scutemob-201` exactly. Version constants read from source and
confirmed executed green by that run:

- `PROTOCOL_VERSION` = **35** (`crates/engine/src/rules/protocol.rs:360`)
- `HASH_SCHEMA_VERSION` = **73** (`crates/engine/src/state/hash.rs:757`)

---

## 2. OOS-DX1-3 — the premise is TRUE and the card is live-wrong

### 2.1 The def

`crates/card-defs/src/defs/nether_traitor.rs`:

- `:35` — `AbilityDefinition::Triggered {`
- `:41` — `trigger_condition: TriggerCondition::WheneverCreatureDies { controller: Some(You), exclude_self: true, nontoken_only: false, filter: None }`
- `:60` — `trigger_zone: Some(TriggerZone::Graveyard)`
- `:63` — `completeness: Completeness::Complete` → **deck-legal**, `validate_deck` accepts it

Printed text: *"Whenever another creature is put into your graveyard from the battlefield, you may
pay {B}. If you do, return this card from your graveyard to the battlefield."* The ability
functions **only from the graveyard** — a returned-to-battlefield Nether Traitor must not be able
to return itself again.

### 2.2 The whole corpus population of `trigger_zone: Some(_)` is 3 defs

`grep -rln "trigger_zone: Some" crates/card-defs/src/defs/`:

| def | trigger condition | `completeness` | lowered by `build_face_ability_vectors`? |
|---|---|---|---|
| `bloodghast.rs:50/:67` | `WheneverPermanentEntersBattlefield` | `partial(...)` — deck-illegal | **skipped** (the one arm that checks) |
| `squee_goblin_nabob.rs:29/:40` | `AtBeginningOfYourUpkeep` | `known_wrong(...)` — deck-illegal | no arm exists for this condition |
| **`nether_traitor.rs:41/:60`** | **`WheneverCreatureDies`** | **`Complete` — deck-legal** | **lowered, `trigger_zone` dropped** |

So exactly **one deck-legal def** is affected, and it is live-wrong. The v3 §2 correction is
confirmed: the "latent" classification in the audit registry row is false.

### 2.3 The loss is non-uniform — re-cited at HEAD

`crates/engine/src/testing/replay_harness.rs`, `build_face_ability_vectors` (`:2449`–`:3871`):

- **The one arm that skips**: `WheneverPermanentEntersBattlefield` — comment at **`:3029-3032`**,
  guard `if trigger_zone.is_some() { continue; }` at **`:3049-3051`**.
- **The arm that does not**: `WheneverCreatureDies` — match at **`:3261-3275`**, push at
  **`:3282-3302`**. `trigger_zone` is swallowed by the `..` rest pattern at `:3273`.
- **Every other trigger arm** in the function also swallows it via `..`. Arm census by
  `trigger_condition:` match, inside `:2449`–`:3871`: **40 arms**, of which **1** checks
  `trigger_zone`.

### 2.4 The runtime type genuinely has no home for it

`TriggeredAbilityDef` (see `crates/card-types/src/state/game_object.rs`) has no `trigger_zone`
field. The lossy-lowering table on `build_face_ability_vectors` (**`:2469-2478`**) already records
this as the third dropped field, seeded `OOS-DX1-3` — that table is what this batch must make
true.

### 2.5 The dispatch side is a separate registry path, and it is narrow

`collect_graveyard_carddef_triggers` (`crates/engine/src/rules/abilities.rs:7112`, body
`:7118`–`:7222`) is the only reader of `trigger_zone` outside the def crate. Its `fires` match
(`:7147`) handles **exactly one** event shape:

```
GameEvent::PermanentEnteredBattlefield { .. } => match trigger_condition {
    TriggerCondition::WheneverPermanentEntersBattlefield { .. } => …,
    _ => false,
},
_ => false,
```

So a `WheneverCreatureDies` graveyard trigger has **no dispatch path at all** today. Making
Nether Traitor stop firing from the battlefield is a one-line skip; making it fire from the
graveyard requires extending this function. **Both halves are needed** — criterion 6205 demands
the trigger fire from the graveyard *and* not fire from the battlefield.

Its scan is `for (idx, ability) in def.abilities.iter().enumerate()` (`:7135`) and it pushes
`PendingTriggerKind::CardDefETB` with `ability_index: idx` (`:7213`), so resolution reads the
effect back out of the registry.

---

## 3. OOS-DX1-4 — the two index spaces, re-cited at HEAD

**Queue side** — `def.abilities.iter().enumerate()`, i.e. the **front face always**:

| # | site | what it queues | `PendingTriggerKind` |
|---|---|---|---|
| Q1 | `abilities.rs:3147` | Backup (CR 702.165a) — `idx` also slices `def.abilities[idx+1..]` | (backup snapshot, not a `CardDefETB` push) |
| Q2 | `abilities.rs:3764` | `WhenYouCastThisSpell` | `CardDefETB` (`:3830`) |
| Q3 | `abilities.rs:4119` (filter_map) | `WhenExertedAsAttacks` | `CardDefETB` (`:4148`) |
| Q4 | `abilities.rs:5157` (filter_map) | `WhenDealsCombatDamageToPlayer` | `CardDefETB` (`:5199`) |
| Q5 | `abilities.rs:6080` | face-down turn-up triggers | `TurnFaceUp` (`:6127`) |
| Q6 | `abilities.rs:6135` | `WheneverRingTemptsYou` | `CardDefETB` (`:6163`) |
| Q7 | `abilities.rs:7135` | the graveyard sweep (§2.5) | `CardDefETB` (`:7216`) |

**Resolution / read side** — `def.effective_abilities(is_transformed).get(ability_index)`, i.e.
**face-aware**:

`abilities.rs:6390`, `:8134`, `:8212`, `:8340`, `:9229`, `:9745`; `resolution.rs:2184`, `:2216`,
`:2254`; `sba.rs:889`.

**The disagreement**: for a source with `is_transformed == true` and a `back_face`, the queue side
picks an index out of the **front** list and the read side resolves it against the **back** list —
so a different ability (or none) is read. For sources that are never transformed the two lists are
identical by `effective_abilities`' own `(_, _) => &self.abilities` arm, so the disagreement is
invisible.

**Already-aligned counter-examples** (both ends face-aware — the shape the queue sites should
match): `turn_actions.rs:287`, `:400`, `:462`, `:541`, `:762`, `:1935`; `replacement.rs:2013`,
`:2031`, `:2220`; `mana.rs:817`.

**Corpus exposure is not yet measured.** Whether any of Q1–Q7 is reachable on a real DFC back face
must be established by enumerating `all_cards()` (SR-36 — never by grep) before deciding fix vs
re-scope. That measurement is stage 1 work, not a stage-0 claim.

---

## 4. What stage 0 does NOT claim

- No claim that the narrow fix is wire-neutral — that is to be **gate-computed**, not predicted.
  The prediction (PROTOCOL 35 / HASH 73 both unmoved) holds only if `TriggeredAbilityDef` does not
  grow a field; §5 of the plan decides the shape.
- No claim about how many of Q1–Q7 are live vs latent — see §3's last paragraph.
