# B16 Fix Session Plan

## Session 1 -- Dungeon error handling and documentation

- [x] MR-B16-01 (MEDIUM) -- Silent error swallowing: replace `unwrap_or_default()` with proper error propagation in `sba.rs:372`, `turn_actions.rs:581,1794`
- [x] MR-B16-02 (MEDIUM) -- Misleading comment about double-counting `dungeons_completed` in `sba.rs:1298-1302`
- [x] MR-B16-03 (LOW) -- Extract duplicated StackObject construction in `engine.rs:2116-2157,2196-2237` into helper
- [x] MR-B16-08 (LOW) -- Initiative upkeep `return venture_events;` should not bypass subsequent upkeep logic in `turn_actions.rs:581-582`

## Notes
- All 4 issues fixed in one pass. Tests pass (1934+), clippy clean, fmt clean.
- MR-B16-01 + MR-B16-08 were fixed together since both touched the same code block in `upkeep_actions()`.
- MR-B16-01: Used explicit `match` with a comment-documented Err branch rather than changing function signatures, since all three callers return `Vec<GameEvent>` and the engine has no logging infrastructure.
- MR-B16-03: Extracted `room_ability_stack_object()` helper immediately before `handle_venture_into_dungeon()`; removed the now-unused `use crate::state::stack::{StackObject, StackObjectKind}` import from inside `handle_venture_into_dungeon`.
