/**
 * Frontend performance instrumentation utilities.
 *
 * Provides lightweight timing helpers for measuring UI interactions
 * against the budgets defined in docs/specs/editor-performance-budget.md.
 *
 * Timers log to the browser console in development mode and when the
 * `VIGIL_PERF` flag is set on `window`. In production builds, the
 * logging is a no-op but the measurement functions still return
 * accurate durations for programmatic use.
 */

/** Check whether perf logging is enabled. */
function isPerfEnabled(): boolean {
	if (
		typeof window !== 'undefined' &&
		(window as unknown as Record<string, unknown>)['VIGIL_PERF']
	) {
		return true;
	}
	// Always log in dev mode
	if (import.meta.env?.DEV) {
		return true;
	}
	return false;
}

/**
 * A simple scoped timer that logs duration on `stop()`.
 *
 * Usage:
 * ```ts
 * const timer = perfTimer('omnibar-open');
 * // ... do work ...
 * timer.stop(); // logs "[perf] omnibar-open: 42.3ms"
 * ```
 */
export interface PerfTimer {
	/** Stop the timer and log the duration. Returns elapsed milliseconds. */
	stop(): number;
	/** Get elapsed milliseconds without stopping. */
	elapsed(): number;
}

/**
 * Start a named performance timer.
 *
 * @param label  Human-readable label for the measurement (e.g. "editor-mount").
 * @param budgetMs  Optional budget in ms. If exceeded, a warning is logged.
 */
export function perfTimer(label: string, budgetMs?: number): PerfTimer {
	const start = performance.now();

	return {
		elapsed(): number {
			return performance.now() - start;
		},
		stop(): number {
			const duration = performance.now() - start;
			if (isPerfEnabled()) {
				const msg = `[perf] ${label}: ${duration.toFixed(1)}ms`;
				if (budgetMs !== undefined && duration > budgetMs) {
					console.warn(`${msg} (OVER BUDGET: ${budgetMs}ms)`);
				} else {
					console.debug(msg);
				}
			}
			return duration;
		}
	};
}

/**
 * Measure an async operation and return both its result and duration.
 *
 * @param label  Human-readable label.
 * @param fn  Async function to measure.
 * @param budgetMs  Optional budget in ms.
 */
export async function timeAsync<T>(
	label: string,
	fn: () => Promise<T>,
	budgetMs?: number
): Promise<{ result: T; durationMs: number }> {
	const timer = perfTimer(label, budgetMs);
	const result = await fn();
	const durationMs = timer.stop();
	return { result, durationMs };
}

/**
 * Measure a synchronous operation and return both its result and duration.
 *
 * @param label  Human-readable label.
 * @param fn  Synchronous function to measure.
 * @param budgetMs  Optional budget in ms.
 */
export function timeSync<T>(
	label: string,
	fn: () => T,
	budgetMs?: number
): { result: T; durationMs: number } {
	const timer = perfTimer(label, budgetMs);
	const result = fn();
	const durationMs = timer.stop();
	return { result, durationMs };
}
