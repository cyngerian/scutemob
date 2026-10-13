# Play surface v3 C3 (`scutemob-268`) — task list

<!-- last_updated: 2026-10-12 -->

Change class: row 3 (tooling) + row 4 per new source gate. Row 1 only if an engine / card-types
file changes (then STOP: wire prediction first). PROTOCOL 44 / HASH 85 expected untouched.

## A. Server DTOs (`crates/view-model`, `tools/play-server/src`) — criterion 8615

- [ ] A1 `ZonesView.exile_by_owner` (owner-keyed, every player a key; shared `exile` kept for v1/v2)
- [ ] A2 redaction case: face-down exile not owned by the seat is redacted in `exile_by_owner` too (test)
- [ ] A3 `PlayerView.commander_tax` (per commander name; unit documented from the engine)
- [ ] A4 redaction case / disposition for `commander_tax` in `redact.rs` table + test
- [ ] A5 `StackItemView.target_refs` (index-aligned with `targets`; object / player / stack_object)
- [ ] A6 redaction case: a target id the seat may not hold (another seat's hand / library, gone object) is withheld (test)
- [ ] A7 `GameSummary.game_id` (opaque, per deal, NOT seed-derived — MR-M11-01) + test
- [ ] A8 pass-until as a server parameter on `POST /api/game/action` (until end of turn / my turn / step; "unless" stops) + tests
- [ ] A9 revealed cards: grep the engine for a concept; record as follow-up with seed if none

## B. Card-image store — criterion 8615

- [ ] B1 store module (layout, three sizes `small` / `normal` / `large`, path by flag / env, gitignored)
- [ ] B2 `GET /api/card-image?name=&version=` — store only, miss = 404, no live fetch (tests, no socket)
- [ ] B3 prefetch when a game's decks are resolved (`new_game` → `dealt_decks`), polite (≥100 ms, 429 backoff), off the request path
- [ ] B4 CLI `play-server --prefetch-images <decklist|names>`
- [ ] B5 README routes + flags

## C. Frontend (`lib/v3/`, additive `lib/v2/images.js`, `lib/stores.js`)

- [ ] C1 v3 image URLs → the route (never Scryfall on v3); `hover.js` 404 shows the chip
- [ ] C2 per-player exile in the 2×2 boxes / strips / popovers (drop the `EX*` shared note)
- [ ] C3 commander tax on the CMD pile
- [ ] C4 `game_id` as the order key (`order.js`)
- [ ] C5 `targeted` outline + arrows to what a stack item already targets; hook on stack items
- [ ] C6 auto-pass uses the server parameter; client loop stays as fallback

## D. Probes — criterion 8616

- [ ] D1 seed 3 to game over on v3 headless (`v3game.py` or `v3sweep.py --turns`): game-over screen, 0 page errors, every decision answered
- [ ] D2 `verify.py` flow re-expressed for v3 in `v3interact.py` / `v3scenario.py` (picker, stack, combat each appear; usability criteria asserted)
- [ ] D3 probe checks for A1–A8, B2, C1–C6 on real decisions
- [ ] D4 `v3sweep.py` matrix (seeds 0, 3 × 2560×1340, 1707×880) green
- [ ] D5 `v3interact.py` (122 at HEAD) still green + new checks

## E. Gates

- [ ] E1 new source gate(s) (v3 never names Scryfall; others as needed), each with one executed defeat recorded in the test doc
- [ ] E2 existing `test_frontend_v3_card_images_are_lazy_and_fall_back` + `--block-scryfall` still green
- [ ] E3 `cargo test --workspace --no-fail-fast` to a file; delta vs 5,356 itemised by NAME
- [ ] E4 `cargo clippy --workspace --all-targets -- -D warnings`
- [ ] E5 `cargo fmt --check` + `tools/check-defs-fmt.sh`
- [ ] E6 `npm run build`

## F. Owner-playtest half — criterion 8617 (worker's half only)

- [ ] F1 `docs/play-v3-playtest-guide.md` (start, what to try, how to report)
- [ ] F2 `memory/play-v3-playtest-findings.md` template

## G. Close

- [ ] G1 `/review` — HIGH / MEDIUM fixed in-cycle, LOW logged
- [ ] G2 notes `memory/play-v3-c3-2026-10-12.md`, CHANGELOG entry (≤10 lines), README
- [ ] G3 follow-up notes: CR 509.1b double-block picker limit; revealed cards; friction-log suggestion (owner ruling pending)
- [ ] G4 scratch (probe image store included) deleted; criteria satisfied; signal-ready
