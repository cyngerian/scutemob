/**
 * API fetch wrapper for the replay viewer backend.
 *
 * All functions return the parsed JSON response or throw on HTTP error.
 * The base URL is empty (relative) so both dev (Vite proxy) and production
 * (axum static serving) work without configuration.
 */

const BASE = '';

async function apiFetch(path, options = {}) {
  const response = await fetch(`${BASE}${path}`, options);
  if (!response.ok) {
    const text = await response.text().catch(() => response.statusText);
    throw new Error(`API ${path} failed (${response.status}): ${text}`);
  }
  return response.json();
}

/** GET /api/session — returns session metadata */
export function fetchSession() {
  return apiFetch('/api/session');
}

/** GET /api/step/:n — returns full StepViewModel including state */
export function fetchStep(n) {
  return apiFetch(`/api/step/${n}`);
}

/** GET /api/step/:n/state — returns only StateViewModel (lighter payload) */
export function fetchStepState(n) {
  return apiFetch(`/api/step/${n}/state`);
}

/** GET /api/scripts — returns { groups, total } */
export function fetchScripts() {
  return apiFetch('/api/scripts');
}

/** POST /api/load — load a script by path, returns new session metadata */
export function loadScript(path) {
  return apiFetch('/api/load', {
    method: 'POST',
    headers: { 'Content-Type': 'application/json' },
    body: JSON.stringify({ path }),
  });
}

/** POST /api/scripts/run — run a script through the harness, returns RunResult (no side effects) */
export function runScript(path) {
  return apiFetch('/api/scripts/run', {
    method: 'POST',
    headers: { 'Content-Type': 'application/json' },
    body: JSON.stringify({ path }),
  });
}

/** POST /api/scripts/approve — approve a script by id, returns { ok: true } */
export function approveScript(id) {
  return apiFetch('/api/scripts/approve', {
    method: 'POST',
    headers: { 'Content-Type': 'application/json' },
    body: JSON.stringify({ id }),
  });
}
