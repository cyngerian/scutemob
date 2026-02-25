# Replay Viewer (Stepper) Server

Manage the axum replay viewer HTTP server at `tools/replay-viewer/`.

**IMPORTANT**: The binary must be run from the **workspace root**
(`/home/airbaggie/scutemob/`), not from `tools/replay-viewer/`. The
default `--scripts-dir` is `test-data/generated-scripts`, which is
relative to the working directory — running from the wrong directory
means no scripts are found.

The pre-built binary is at `target/release/replay-viewer`. If it does
not exist, build it first.

---

## Determine what the user wants

Check the user's message for one of:
- **start** — start the server (default if no verb given)
- **stop** — kill the running server
- **restart** — stop then start

---

## Stop

Kill any running instance:

```bash
fuser -k 3030/tcp 2>/dev/null; echo "stopped"
```

If the user only wanted to stop, confirm it's stopped and exit.

---

## Start

1. **Check for the binary:**
```bash
ls /home/airbaggie/scutemob/target/release/replay-viewer 2>/dev/null && echo "exists" || echo "missing"
```

2. **If missing, build it:**
```bash
cd /home/airbaggie/scutemob/tools/replay-viewer && ~/.cargo/bin/cargo build --release 2>&1 | tail -5
```

3. **Kill any existing instance on port 3030:**
```bash
fuser -k 3030/tcp 2>/dev/null; sleep 0.3; echo "cleared"
```

4. **Start from workspace root with `run_in_background: true`:**
```bash
/home/airbaggie/scutemob/target/release/replay-viewer --host 0.0.0.0
```
Use `run_in_background: true` on the Bash call.

5. **Verify it's up** (wait ~2 s, then probe the API):
```bash
sleep 2 && curl -s http://localhost:3030/api/scripts | python3 -c "import sys,json; d=json.load(sys.stdin); print(f'OK — {d[\"total\"]} scripts in {len(d[\"groups\"])} groups')" 2>/dev/null || echo "server not responding"
```

6. **Report** to the user:
   - URL: `http://localhost:3030/`
   - Number of scripts loaded
   - If accessed remotely: the host's LAN IP (run `hostname -I | awk '{print $1}'` to find it)

---

## Restart

Run **Stop** then **Start** in sequence.

---

## Notes

- Default port: **3030**. Pass `--port NNNN` to override.
- Default scripts dir: `test-data/generated-scripts` (relative to CWD).
  Override with `--scripts-dir /absolute/path`.
- Only **approved** scripts appear in the test suite but **all** scripts
  (including `pending_review`) are visible in the stepper UI.
- The frontend is pre-built and served from `tools/replay-viewer/dist/`.
  If the UI looks stale, rebuild: `cd tools/replay-viewer && npm run build`.
