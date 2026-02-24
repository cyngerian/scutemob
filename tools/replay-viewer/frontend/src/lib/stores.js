/**
 * Svelte stores for the replay viewer.
 *
 * Data flow:
 *   fetchSession() -> session store
 *   currentStepIndex store -> fetchStep(n) -> stepData store
 *
 * Components read from these stores. Only App.svelte triggers fetches.
 */
import { writable, derived } from 'svelte/store';
import { fetchStep, fetchSession } from './api.js';

/** Current session metadata (from GET /api/session or POST /api/load). */
export const session = writable(null);

/** The current step index (0 = initial state). */
export const currentStepIndex = writable(0);

/** The full StepViewModel for the current step (from GET /api/step/:n). */
export const stepData = writable(null);

/** True while a fetch is in progress (prevents double-fetches). */
export const loading = writable(false);

/**
 * Load session metadata from the backend and reset to step 0.
 * Call after POST /api/load or on initial page load.
 */
export async function initSession() {
  loading.set(true);
  try {
    const meta = await fetchSession();
    session.set(meta);
    currentStepIndex.set(0);
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
 */
export async function goToStep(n) {
  loading.set(true);
  try {
    currentStepIndex.set(n);
    const step = await fetchStep(n);
    stepData.set(step);
  } catch (err) {
    console.error(`goToStep(${n}) failed:`, err);
  } finally {
    loading.set(false);
  }
}
