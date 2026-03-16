/**
 * Theme store — Svelte 5 runes-based theme management.
 *
 * Exports:
 *   theme        — reactive $state<'system' | 'light' | 'dark'>
 *   resolvedTheme — $derived<'light' | 'dark'> (resolves 'system' via matchMedia)
 *   setTheme()   — update theme, persist to localStorage, sync data-theme attr
 */

const STORAGE_KEY = "mongit-theme";

type ThemeMode = "system" | "light" | "dark";
type ResolvedTheme = "light" | "dark";

/** Read the stored theme from localStorage, defaulting to 'system'. */
function readStoredTheme(): ThemeMode {
	try {
		const stored = localStorage.getItem(STORAGE_KEY);
		if (stored === "light" || stored === "dark" || stored === "system") {
			return stored;
		}
	} catch {
		// localStorage unavailable — fall back to system
	}
	return "system";
}

/** Detect current system preference. */
function getSystemPreference(): ResolvedTheme {
	if (typeof window !== "undefined" && window.matchMedia) {
		return window.matchMedia("(prefers-color-scheme: dark)").matches
			? "dark"
			: "light";
	}
	return "dark"; // default for Tauri desktop
}

/** Sync the data-theme attribute on <html>. */
function syncAttribute(mode: ThemeMode): void {
	if (typeof document !== "undefined") {
		document.documentElement.setAttribute("data-theme", mode);
	}
}

// --- Reactive state ---

let theme = $state<ThemeMode>(readStoredTheme());

/** Track system preference reactively so resolvedTheme updates on OS change. */
let systemIsDark = $state<boolean>(getSystemPreference() === "dark");

const resolvedTheme = $derived<ResolvedTheme>(
	theme === "system" ? (systemIsDark ? "dark" : "light") : theme,
);

// --- System preference listener ---

if (typeof window !== "undefined" && window.matchMedia) {
	const mql = window.matchMedia("(prefers-color-scheme: dark)");
	mql.addEventListener("change", (e: MediaQueryListEvent) => {
		systemIsDark = e.matches;
	});
}

// --- Public API ---

/**
 * Set the active theme mode.
 * Persists to localStorage and updates the data-theme attribute on <html>.
 */
function setTheme(mode: ThemeMode): void {
	theme = mode;
	syncAttribute(mode);
	try {
		localStorage.setItem(STORAGE_KEY, mode);
	} catch {
		// localStorage unavailable — attribute is still set
	}
}

export type { ResolvedTheme, ThemeMode };
export { resolvedTheme, setTheme, theme };
