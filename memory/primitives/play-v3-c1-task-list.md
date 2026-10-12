# Play surface v3 C1 (`scutemob-266`) — task list

Live list; reposted as an `esm task comment` at every milestone. Change class: row 3 (tooling)
plus row 4 per new source gate.

## Setup
- [x] 1. Read worker.md, brief.md, `docs/play-surface-v3-brief.md`, mockup 7, v2 sources, conventions table
- [x] 2. ESM session started; `npm install` + `cargo build -p play-server` in the worktree
- [x] 3. Shared contract: `lib/v3/ui.svelte.js` (popover / context menu / peek state, size settings)

## Components (`tools/play-server/frontend/src/lib/v3/`)
- [x] 4. `v3.css` — mockup 7 + `common.css` `.t6`/`.t7` rules ported, scoped under `.v3`
- [x] 5. `CardFace.svelte` + `images.js` normal/large URL
- [x] 6. `hover.js` (one overlay + BroadcastChannel) and `CardWindow.svelte` (`?card-window`)
- [x] 7. `fit.js` — `fitBoard` / `fitAll` / `sizeStrips` ported
- [x] 8. `Pile`, `PilePopover`, `ZoneBox` (2×2), `ZoneStrip` (1×5)
- [x] 9. `SideCell` — mirrored rows, focused grey-out + lift, one-second peek
- [x] 10. `FocusedBoard`, `YourBoard`, `HandRow` (70px, paged)
- [x] 11. `PhaseTrack`
- [x] 12. `StackBox`, `CombatCard`, `CombatFloat`
- [x] 13. `DecisionPanel` — centred float, minimise, card items, hosts v2 `PromptPanel` (pickers, pass-until)
- [x] 14. `ContextMenu` (seat right-click), `TopBar` (size slider, auto toggles, card window, menu)
- [x] 15. `TableV3.svelte` (grid, focus rule, click-through, offered ids) + `main.js` `?v3` / `?card-window` mounts
- [x] 16. `npm run build` green

## Probes
- [x] 17. `tools/play-server/probes/v3sweep.py` written
- [x] 18. Seed A at 2560×1340 and 1707×880: 10 turns, no page scroll, no box scroll, 0 page errors
- [x] 19. Seed B at 2560×1340 and 1707×880: same
- [x] 20. Screenshots turns 1 / 5 / 10 per seed eyeballed against the mockup

## Gates
- [x] 21. Source gate: v3 card images lazy + fall back, URL through `images.js` (+ executed defeat)
- [x] 22. Source gate: `?v3` / `?card-window` mounts, `?v2` and default untouched, v3 never imports `PlayAppV2` (+ defeat)
- [x] 23. Source gate: every `v3.css` rule scoped under `.v3` (+ defeat)
- [x] 24. `cargo test --workspace --no-fail-fast` to a file, count delta by test name (5,352 at HEAD)
- [x] 25. `cargo clippy --workspace --all-targets -- -D warnings`
- [x] 26. `cargo fmt --check` + `tools/check-defs-fmt.sh`

## Acceptance
- [x] 27. Criterion 8605 (table renders) satisfied
- [x] 28. Criterion 8606 (sizing) satisfied
- [x] 29. Criterion 8607 (priority playable) satisfied
- [x] 30. Criterion 8608 (drive + gates) satisfied

## Close
- [ ] 31. `/review`; HIGH and MEDIUM fixed, LOW logged
- [ ] 32. Notes `memory/play-v3-c1-2026-10-12.md`, CHANGELOG entry, README pointer
- [ ] 33. Scratch deleted, committed by explicit path, signal-ready, session end
