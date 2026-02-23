# Engine Performance Benchmarks — Baseline Results

**Run date**: 2026-02-23
**Benchmark file**: `crates/engine/benches/engine_perf.rs`
**Profile**: `--release` (criterion optimized build)
**Hardware**: Linux 6.1.0-31-amd64

---

## Results

| Benchmark          | Mean       | 95% CI (low–high)      | Status |
|--------------------|------------|------------------------|--------|
| `priority_cycle_4p` | 23.4 µs   | 22.8 µs – 24.1 µs     | PASS   |
| `priority_cycle_6p` | 36.6 µs   | 35.7 µs – 37.6 µs     | PASS   |
| `sba_check`         | 13.8 µs   | 13.6 µs – 14.0 µs     | PASS   |
| `full_turn_4p`      | 205 µs    | 200 µs – 211 µs        | PASS   |
| `full_turn_6p`      | 303 µs    | 295 µs – 312 µs        | PASS   |

---

## Red-Flag Thresholds

| Threshold                    | Target    | Actual (worst case)  | Status |
|------------------------------|-----------|----------------------|--------|
| Priority cycle >10ms         | <10ms     | ~37 µs (6p)          | **270× under target** |
| SBA check >1ms               | <1ms      | ~14 µs               | **70× under target**  |

All benchmarks are well within acceptable bounds.

---

## Analysis

### Priority Cycle (4p vs 6p)

- 4-player priority cycle: **~23 µs**
- 6-player priority cycle: **~37 µs**
- Scaling: 6p is ~1.57× slower than 4p for a 50% larger player count — near-linear.
- No hotspot. The priority loop is O(N) in players as expected.

### SBA Check (20 permanents, no SBAs firing)

- **~14 µs** per check on a board with 20 permanents.
- This measures scan + fixed-point termination cost, not SBA consequence application.
- With complex boards (100+ permanents with counters and layers), SBA cost would scale.
- No hotspot at this board size; no further optimization needed before M10.

### Full Turn Processing

- 4-player full turn (Upkeep → Cleanup): **~205 µs**
- 6-player full turn (Upkeep → Cleanup): **~303 µs**
- These include: draw step, priority rounds for each step, SBA checks, and turn-order advancement.
- Scaling: 6p is ~1.48× slower than 4p — near-linear, as expected.
- At 303 µs per turn, a 100-turn Commander game would take ~30ms of pure engine time — completely negligible.

### Outlier Note

- `priority_cycle_6p`: 11 outliers (8 high mild, 3 high severe) — typical for short benchmarks on shared hardware; median is stable.
- `sba_check`: 13 outliers — also typical; mean is tight (13.6–14.0 µs).

---

## Hotspot Identification

**No hotspots detected.** All measurements are well under their red-flag thresholds:
- Priority cycle: 270× under the 10ms threshold
- SBA check: 70× under the 1ms threshold

The engine has substantial headroom for complex board states and longer games.
Even if board complexity added 100× overhead to SBA checks, the result would be
~1.4ms — still near the target. Priority cycles are dominated by state cloning
(O(1) with `im-rs` structural sharing) and are unlikely to become bottlenecks.

---

## Criterion Raw Output (2026-02-23)

```
priority_cycle_4p  time:  [22.792 µs 23.428 µs 24.142 µs]
priority_cycle_6p  time:  [35.679 µs 36.612 µs 37.649 µs]
sba_check          time:  [13.582 µs 13.762 µs 13.971 µs]
full_turn_4p       time:  [200.09 µs 205.27 µs 211.35 µs]
full_turn_6p       time:  [294.52 µs 302.76 µs 311.83 µs]
```

Format: [low median high] — 100 samples per benchmark, 3s warmup.
