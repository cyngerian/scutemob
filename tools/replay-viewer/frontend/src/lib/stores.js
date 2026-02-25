/**
 * Svelte stores for the replay viewer.
 *
 * Data flow:
 *   fetchSession() -> session store
 *   currentStepIndex store -> fetchStep(n) -> stepData store
 *   prevStepData + stepData -> stateDiff (Set of changed path strings)
 *
 * Components read from these stores. Only App.svelte triggers fetches.
 */
import { writable, derived } from 'svelte/store';
import { fetchStep, fetchSession } from './api.js';
import { diffState } from './diff.js';

/** Current session metadata (from GET /api/session or POST /api/load). */
export const session = writable(null);

/** RunResult for the currently loaded script (null when no script is loaded). */
export const runResult = writable(null);

/** The current step index (0 = initial state). */
export const currentStepIndex = writable(0);

/** The full StepViewModel for the current step (from GET /api/step/:n). */
export const stepData = writable(null);

/** The full StepViewModel for the PREVIOUS step (used for diff highlighting). */
export const prevStepData = writable(null);

/** True while a fetch is in progress (prevents double-fetches). */
export const loading = writable(false);

/**
 * Derived store: Set<string> of changed state paths between prevStepData and stepData.
 * Components use this to add a `changed` CSS class to affected fields.
 */
export const stateDiff = derived(
  [prevStepData, stepData],
  ([$prev, $curr]) => diffState($prev?.state ?? null, $curr?.state ?? null)
);

/**
 * Load session metadata from the backend and reset to step 0.
 * Call after POST /api/load or on initial page load.
 */
export async function initSession() {
  loading.set(true);
  try {
    const meta = await fetchSession();
    session.set(meta);
    runResult.set(meta.run_result ?? null);
    currentStepIndex.set(0);
    prevStepData.set(null);
    if (meta.loaded) {
      const step = await fetchStep(0);
      stepData.set(step);
    } else {
      stepData.set(null);
    }
  } catch (err) {
    console.error('initSession failed:', err);
  } finally {
    loading.set(false);
  }
}

/**
 * Navigate to a specific step index.
 * Fetches the step data and updates stepData store.
 * Saves the current stepData as prevStepData for diff computation.
 */
export async function goToStep(n) {
  loading.set(true);
  try {
    // Snapshot current data as prev before overwriting
    let currentData;
    stepData.subscribe((v) => { currentData = v; })();
    prevStepData.set(currentData);

    currentStepIndex.set(n);
    const step = await fetchStep(n);
    stepData.set(step);
  } catch (err) {
    console.error(`goToStep(${n}) failed:`, err);
  } finally {
    loading.set(false);
  }
}
