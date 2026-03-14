import { invoke } from '@tauri-apps/api/core';

export interface WatcherEvent {
	id: number;
	timestamp: Date;
	latency: number | null;
}

function createWatcherStore() {
	let watching = $state(false);
	let repoPath = $state('');
	let events = $state<WatcherEvent[]>([]);
	let error = $state<string | null>(null);
	let eventCounter = $state(0);

	async function startWatching(path: string): Promise<void> {
		try {
			error = null;
			await invoke('watch_repo', { path });
			repoPath = path;
			watching = true;
		} catch (e) {
			error = String(e);
			watching = false;
		}
	}

	async function stopWatching(): Promise<void> {
		try {
			error = null;
			await invoke('stop_watching');
			watching = false;
		} catch (e) {
			error = String(e);
		}
	}

	function addEvent(): void {
		eventCounter += 1;
		const evt: WatcherEvent = {
			id: eventCounter,
			timestamp: new Date(),
			latency: null
		};
		events = [evt, ...events].slice(0, 100);
	}

	function clearEvents(): void {
		events = [];
	}

	return {
		get watching() {
			return watching;
		},
		get repoPath() {
			return repoPath;
		},
		get events() {
			return events;
		},
		get error() {
			return error;
		},
		startWatching,
		stopWatching,
		addEvent,
		clearEvents
	};
}

export const watcherStore = createWatcherStore();
