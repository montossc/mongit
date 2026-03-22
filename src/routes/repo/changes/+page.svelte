<script lang="ts">
	import { onMount } from 'svelte';
	import { goto } from '$app/navigation';
	import { repoStore } from '$lib/stores/repo.svelte';
	import { changesStore, type FileChangeKind } from '$lib/stores/changes.svelte';
	import { conflictStore } from '$lib/stores/conflict.svelte';
	import { diffStore, type DiffHunkInfo } from '$lib/stores/diff.svelte';
	import { listen } from '@tauri-apps/api/event';

	let unlisten: (() => void) | null = null;
	let mounted = true;

	onMount(() => {
		// Load files and diff when the route mounts
		if (repoStore.activeRepoPath) {
			changesStore.loadFiles(repoStore.activeRepoPath);
			diffStore.fetchDiff(repoStore.activeRepoPath);
		}

		// Listen for file system changes and refresh
		const setupListener = async () => {
			const cb = await listen('repo-changed', () => {
				changesStore.refresh();
				diffStore.refresh();
			});
			if (mounted) {
				unlisten = cb;
			} else {
				cb(); // Already unmounted — clean up immediately
			}
		};
		setupListener();

		return () => {
			mounted = false;
			if (unlisten) unlisten();
		};
	});

	/** Handle file selection — update both stores. */
	function handleFileSelect(path: string) {
		changesStore.selectFile(path);
		diffStore.selectFile(path);
	}

	/** Handle staging a hunk, then refresh both stores. */
	async function handleStageHunk(filePath: string, hunkIndex: number) {
		const success = await diffStore.stageHunk(filePath, hunkIndex);
		if (success) {
			await Promise.all([diffStore.refresh(), changesStore.refresh()]);
		}
	}

	/** Handle unstaging a hunk, then refresh both stores. */
	async function handleUnstageHunk(filePath: string, hunkIndex: number) {
		const success = await diffStore.unstageHunk(filePath, hunkIndex);
		if (success) {
			await Promise.all([diffStore.refresh(), changesStore.refresh()]);
		}
	}

	/** Handle staging selected lines from a hunk. */
	async function handleStageLines(filePath: string, hunkIndex: number) {
		const selected = diffStore.getSelectedLines('unstaged', hunkIndex);
		if (selected.size === 0) return;
		const success = await diffStore.stageLines(filePath, hunkIndex, [...selected]);
		if (success) {
			await Promise.all([diffStore.refresh(), changesStore.refresh()]);
		}
	}

	/** Handle unstaging selected lines from a hunk. */
	async function handleUnstageLines(filePath: string, hunkIndex: number) {
		const selected = diffStore.getSelectedLines('staged', hunkIndex);
		if (selected.size === 0) return;
		const success = await diffStore.unstageLines(filePath, hunkIndex, [...selected]);
		if (success) {
			await Promise.all([diffStore.refresh(), changesStore.refresh()]);
		}
	}

	/** Handle clicking a diff line to toggle selection. */
	function handleLineClick(side: 'unstaged' | 'staged', hunkIndex: number, lineIndex: number, origin: string) {
		// Only allow selecting change lines (+ or -), not context
		if (origin !== '+' && origin !== '-') return;
		diffStore.toggleLineSelection(side, hunkIndex, lineIndex);
	}

	/** Check if a line is selected. */
	function isLineSelected(side: 'unstaged' | 'staged', hunkIndex: number, lineIndex: number): boolean {
		return diffStore.getSelectedLines(side, hunkIndex).has(lineIndex);
	}

	/** Map a FileChangeKind to a short display label. */
	function kindLabel(kind: FileChangeKind): string {
		switch (kind) {
			case 'Added': return 'A';
			case 'Modified': return 'M';
			case 'Deleted': return 'D';
			case 'Renamed': return 'R';
			case 'Typechange': return 'T';
			case 'Conflicted': return 'C';
		}
	}

	/** Map a FileChangeKind to a CSS modifier class. */
	function kindClass(kind: FileChangeKind): string {
		switch (kind) {
			case 'Added': return 'added';
			case 'Modified': return 'modified';
			case 'Deleted': return 'deleted';
			case 'Renamed': return 'renamed';
			case 'Typechange': return 'typechange';
			case 'Conflicted': return 'conflicted';
		}
	}

	/** Get the filename from a path. */
	function fileName(path: string): string {
		const parts = path.split('/');
		return parts[parts.length - 1];
	}

	/** Get the directory from a path, or empty string. */
	function fileDir(path: string): string {
		const lastSlash = path.lastIndexOf('/');
		return lastSlash > 0 ? path.substring(0, lastSlash + 1) : '';
	}

	/** Format diff line origin for display. */
	function lineClass(origin: string): string {
		switch (origin) {
			case '+': return 'line-add';
			case '-': return 'line-del';
			default: return 'line-ctx';
		}
	}

	/** Format hunk header for display. */
	function formatHunkHeader(hunk: DiffHunkInfo): string {
		return `@@ -${hunk.old_start},${hunk.old_lines} +${hunk.new_start},${hunk.new_lines} @@`;
	}

	// Reactive: whether we have any hunks to show
	const hasUnstagedHunks = $derived(diffStore.selectedFileUnstagedHunks.length > 0);
	const hasStagedHunks = $derived(diffStore.selectedFileStagedHunks.length > 0);
	const hasAnyHunks = $derived(hasUnstagedHunks || hasStagedHunks);
</script>

<div class="changes-workspace">
	{#if conflictStore.isMerging}
		<div class="conflict-banner">
			<span class="conflict-banner-icon">!</span>
			<span class="conflict-banner-text">
				Merge in progress — {conflictStore.conflictCount} conflicted file{conflictStore.conflictCount !== 1 ? 's' : ''}
			</span>
			<button class="conflict-banner-btn" onclick={() => goto('/repo/resolve')}>Resolve Conflicts</button>
		</div>
	{/if}

	{#if changesStore.loading && changesStore.files.length === 0}
		<!-- Loading state (only when no cached files) -->
		<div class="state-message">
			<div class="spinner"></div>
			<p>Loading changed files…</p>
		</div>

	{:else if changesStore.error}
		<!-- Error state -->
		<div class="state-message error">
			<p class="state-label">Error loading changes</p>
			<p class="state-detail">{changesStore.error}</p>
			<button class="retry-btn" onclick={() => changesStore.refresh()}>Retry</button>
		</div>

	{:else if changesStore.files.length === 0}
		<!-- Empty state: clean repo -->
		<div class="state-message">
			<p class="state-label">No changes</p>
			<p class="state-detail">Working tree is clean</p>
		</div>

	{:else}
		<div class="split-layout">
			<!-- File list (left panel) -->
			<div class="file-list-panel">
				<div class="file-list" role="listbox" aria-label="Changed files">
					{#each changesStore.files as file (file.path)}
						<button
							class="file-row"
							class:selected={changesStore.selectedPath === file.path}
							role="option"
							aria-selected={changesStore.selectedPath === file.path}
							onclick={() => handleFileSelect(file.path)}
						>
							<span class="file-path">
								{#if fileDir(file.path)}
									<span class="file-dir">{fileDir(file.path)}</span>
								{/if}
								<span class="file-name">{fileName(file.path)}</span>
							</span>

							<span class="file-badges">
								{#if file.staged}
									<span class="status-badge staged {kindClass(file.staged)}" title="Staged: {file.staged}">
										{kindLabel(file.staged)}
									</span>
								{/if}
								{#if file.unstaged}
									<span class="status-badge unstaged {kindClass(file.unstaged)}" title="Unstaged: {file.unstaged}">
										{kindLabel(file.unstaged)}
									</span>
								{/if}
							</span>
						</button>
					{/each}
				</div>
			</div>

			<!-- Hunk panel (right panel) -->
			<div class="hunk-panel">
				{#if !changesStore.selectedPath}
					<div class="state-message">
						<p class="state-detail">Select a file to view hunks</p>
					</div>

				{:else if diffStore.loading}
					<div class="state-message">
						<div class="spinner"></div>
						<p>Loading diff…</p>
					</div>

				{:else if !hasAnyHunks}
					<div class="state-message">
						<p class="state-label">No hunks</p>
						<p class="state-detail">This file has no renderable text hunks</p>
					</div>

				{:else}
					<div class="hunk-scroll">
						{#if diffStore.stagingError}
							<div class="staging-error">
								<p>{diffStore.stagingError}</p>
							</div>
						{/if}

						<!-- Unstaged hunks -->
						{#if hasUnstagedHunks}
							<div class="hunk-section">
								<h3 class="hunk-section-title">Unstaged Changes</h3>
								{#each diffStore.selectedFileUnstagedHunks as hunk, hunkIndex}
									{@const selectedCount = diffStore.getSelectedChangeCount('unstaged', hunkIndex, hunk)}
									<div class="hunk-block">
										<div class="hunk-header">
											<span class="hunk-header-text">{formatHunkHeader(hunk)}</span>
											<div class="hunk-actions">
												{#if selectedCount > 0}
													<button
														class="hunk-action-btn stage-btn"
														disabled={diffStore.staging}
														onclick={() => handleStageLines(diffStore.selectedPath!, hunkIndex)}
														title="Stage selected lines"
													>
														{#if diffStore.staging}
															<span class="btn-spinner"></span>
														{:else}
															Stage ({selectedCount})
														{/if}
													</button>
												{/if}
												<button
													class="hunk-action-btn stage-btn"
													disabled={diffStore.staging}
													onclick={() => handleStageHunk(diffStore.selectedPath!, hunkIndex)}
													title="Stage this hunk"
												>
													{#if diffStore.staging}
														<span class="btn-spinner"></span>
													{:else}
														Stage Hunk
													{/if}
												</button>
											</div>
										</div>
										<div class="hunk-lines">
											{#each hunk.lines as line, lineIdx}
												{#if line.origin === ' ' || line.origin === '+' || line.origin === '-'}
													<!-- svelte-ignore a11y_click_events_have_key_events -->
													<div
														class="diff-line {lineClass(line.origin)}"
														class:line-selectable={line.origin === '+' || line.origin === '-'}
														class:line-selected={isLineSelected('unstaged', hunkIndex, lineIdx)}
														role={line.origin !== ' ' ? 'option' : undefined}
														aria-selected={line.origin !== ' ' ? isLineSelected('unstaged', hunkIndex, lineIdx) : undefined}
														onclick={() => handleLineClick('unstaged', hunkIndex, lineIdx, line.origin)}
													>
														<span class="line-origin">{line.origin}</span>
														<span class="line-content">{line.content}</span>
													</div>
												{/if}
											{/each}
										</div>
									</div>
								{/each}
							</div>
						{/if}

						<!-- Staged hunks -->
						{#if hasStagedHunks}
							<div class="hunk-section">
								<h3 class="hunk-section-title">Staged Changes</h3>
								{#each diffStore.selectedFileStagedHunks as hunk, hunkIndex}
									{@const selectedCount = diffStore.getSelectedChangeCount('staged', hunkIndex, hunk)}
									<div class="hunk-block">
										<div class="hunk-header">
											<span class="hunk-header-text">{formatHunkHeader(hunk)}</span>
											<div class="hunk-actions">
												{#if selectedCount > 0}
													<button
														class="hunk-action-btn unstage-btn"
														disabled={diffStore.staging}
														onclick={() => handleUnstageLines(diffStore.selectedPath!, hunkIndex)}
														title="Unstage selected lines"
													>
														{#if diffStore.staging}
															<span class="btn-spinner"></span>
														{:else}
															Unstage ({selectedCount})
														{/if}
													</button>
												{/if}
												<button
													class="hunk-action-btn unstage-btn"
													disabled={diffStore.staging}
													onclick={() => handleUnstageHunk(diffStore.selectedPath!, hunkIndex)}
													title="Unstage this hunk"
												>
													{#if diffStore.staging}
														<span class="btn-spinner"></span>
													{:else}
														Unstage Hunk
													{/if}
												</button>
											</div>
										</div>
										<div class="hunk-lines">
											{#each hunk.lines as line, lineIdx}
												{#if line.origin === ' ' || line.origin === '+' || line.origin === '-'}
													<!-- svelte-ignore a11y_click_events_have_key_events -->
													<div
														class="diff-line {lineClass(line.origin)}"
														class:line-selectable={line.origin === '+' || line.origin === '-'}
														class:line-selected={isLineSelected('staged', hunkIndex, lineIdx)}
														role={line.origin !== ' ' ? 'option' : undefined}
														aria-selected={line.origin !== ' ' ? isLineSelected('staged', hunkIndex, lineIdx) : undefined}
														onclick={() => handleLineClick('staged', hunkIndex, lineIdx, line.origin)}
													>
														<span class="line-origin">{line.origin}</span>
														<span class="line-content">{line.content}</span>
													</div>
												{/if}
											{/each}
										</div>
									</div>
								{/each}
							</div>
						{/if}
					</div>
				{/if}
			</div>
		</div>
	{/if}
</div>

<style>
	.changes-workspace {
		height: 100%;
		display: flex;
		flex-direction: column;
		overflow: hidden;
	}

	/* ── Split layout ──────────────────────────────────────────────── */

	.split-layout {
		display: flex;
		flex: 1;
		overflow: hidden;
	}

	.file-list-panel {
		width: 280px;
		min-width: 200px;
		border-right: 1px solid var(--color-border);
		display: flex;
		flex-direction: column;
		overflow: hidden;
	}

	.hunk-panel {
		flex: 1;
		display: flex;
		flex-direction: column;
		overflow: hidden;
	}

	/* ── State messages ─────────────────────────────────────────────── */

	.state-message {
		display: flex;
		flex-direction: column;
		align-items: center;
		justify-content: center;
		flex: 1;
		gap: var(--space-3);
		color: var(--color-text-muted);
		padding: var(--space-8);
	}

	.state-message.error {
		color: var(--color-danger);
	}

	.state-label {
		font-size: var(--text-body-size);
		font-weight: 500;
		margin: 0;
	}

	.state-detail {
		font-size: var(--text-body-sm-size);
		margin: 0;
		max-width: 360px;
		text-align: center;
		word-break: break-word;
	}

	.state-message.error .state-detail {
		color: var(--color-text-secondary);
	}

	.retry-btn {
		margin-top: var(--space-3);
		padding: var(--space-2) var(--space-5);
		font-size: var(--text-body-sm-size);
		background: none;
		border: 1px solid var(--color-border);
		border-radius: var(--radius-sm);
		color: var(--color-text-secondary);
		cursor: pointer;
		transition: background var(--transition-fast);
	}

	.retry-btn:hover {
		background: var(--color-bg-hover);
		color: var(--color-text-primary);
	}

	.spinner {
		width: 20px;
		height: 20px;
		border: 2px solid var(--color-border);
		border-top-color: var(--color-accent);
		border-radius: 50%;
		animation: spin 0.6s linear infinite;
	}

	@keyframes spin {
		to { transform: rotate(360deg); }
	}

	/* ── File list ──────────────────────────────────────────────────── */

	.file-list {
		flex: 1;
		overflow-y: auto;
		padding: var(--space-2) 0;
	}

	.file-row {
		display: flex;
		align-items: center;
		justify-content: space-between;
		width: 100%;
		height: var(--size-row-default);
		padding: 0 var(--space-5);
		background: none;
		border: none;
		color: var(--color-text-primary);
		font-size: var(--text-body-sm-size);
		cursor: pointer;
		text-align: left;
		transition: background var(--transition-fast);
	}

	.file-row:hover {
		background: var(--color-bg-hover);
	}

	.file-row.selected {
		background: var(--color-bg-active);
	}

	.file-row:focus-visible {
		outline: var(--focus-ring-width) solid var(--focus-ring-color);
		outline-offset: calc(-1 * var(--focus-ring-width));
	}

	/* ── File path ──────────────────────────────────────────────────── */

	.file-path {
		display: flex;
		align-items: baseline;
		min-width: 0;
		overflow: hidden;
		font-family: var(--font-mono);
		font-size: var(--text-mono-sm-size);
	}

	.file-dir {
		color: var(--color-text-muted);
		white-space: nowrap;
		overflow: hidden;
		text-overflow: ellipsis;
		flex-shrink: 1;
	}

	.file-name {
		color: var(--color-text-primary);
		white-space: nowrap;
		flex-shrink: 0;
	}

	/* ── Status badges ──────────────────────────────────────────────── */

	.file-badges {
		display: flex;
		gap: var(--space-1);
		flex-shrink: 0;
		margin-left: var(--space-3);
	}

	.status-badge {
		display: inline-flex;
		align-items: center;
		justify-content: center;
		width: 18px;
		height: 18px;
		border-radius: var(--radius-xs, 3px);
		font-family: var(--font-mono);
		font-size: 10px;
		font-weight: 600;
		line-height: 1;
	}

	/* Staged badges: filled background */
	.status-badge.staged.added     { background: var(--color-success); color: white; }
	.status-badge.staged.modified  { background: var(--color-info); color: white; }
	.status-badge.staged.deleted   { background: var(--color-danger); color: white; }
	.status-badge.staged.renamed   { background: var(--color-warning); color: white; }
	.status-badge.staged.typechange { background: var(--color-text-muted); color: white; }
	.status-badge.staged.conflicted { background: var(--color-danger); color: white; }

	/* Unstaged badges: outline style */
	.status-badge.unstaged.added     { border: 1px solid var(--color-success); color: var(--color-success); }
	.status-badge.unstaged.modified  { border: 1px solid var(--color-info); color: var(--color-info); }
	.status-badge.unstaged.deleted   { border: 1px solid var(--color-danger); color: var(--color-danger); }
	.status-badge.unstaged.renamed   { border: 1px solid var(--color-warning); color: var(--color-warning); }
	.status-badge.unstaged.typechange { border: 1px solid var(--color-text-muted); color: var(--color-text-muted); }
	.status-badge.unstaged.conflicted { border: 1px solid var(--color-danger); color: var(--color-danger); }

	/* ── Hunk panel ────────────────────────────────────────────────── */

	.hunk-scroll {
		flex: 1;
		overflow-y: auto;
		padding: var(--space-3);
	}

	.staging-error {
		padding: var(--space-3) var(--space-4);
		margin-bottom: var(--space-3);
		background: color-mix(in srgb, var(--color-danger) 10%, transparent);
		border: 1px solid var(--color-danger);
		border-radius: var(--radius-sm);
		color: var(--color-danger);
		font-size: var(--text-body-sm-size);
		word-break: break-word;
	}

	.staging-error p {
		margin: 0;
	}

	.hunk-section {
		margin-bottom: var(--space-5);
	}

	.hunk-section-title {
		font-size: var(--text-body-sm-size);
		font-weight: 600;
		color: var(--color-text-secondary);
		text-transform: uppercase;
		letter-spacing: 0.05em;
		margin: 0 0 var(--space-3) 0;
		padding: 0 var(--space-2);
	}

	.hunk-block {
		border: 1px solid var(--color-border);
		border-radius: var(--radius-sm);
		margin-bottom: var(--space-3);
		overflow: hidden;
	}

	.hunk-header {
		display: flex;
		align-items: center;
		justify-content: space-between;
		padding: var(--space-2) var(--space-3);
		background: var(--color-bg-hover);
		border-bottom: 1px solid var(--color-border);
		gap: var(--space-3);
	}

	.hunk-header-text {
		font-family: var(--font-mono);
		font-size: var(--text-mono-sm-size);
		color: var(--color-text-muted);
		white-space: nowrap;
		overflow: hidden;
		text-overflow: ellipsis;
	}

	.hunk-action-btn {
		flex-shrink: 0;
		padding: var(--space-1) var(--space-3);
		font-size: var(--text-body-sm-size);
		font-weight: 500;
		border: 1px solid var(--color-border);
		border-radius: var(--radius-sm);
		cursor: pointer;
		transition: background var(--transition-fast), color var(--transition-fast);
		min-width: 64px;
		display: inline-flex;
		align-items: center;
		justify-content: center;
	}

	.hunk-actions {
		display: flex;
		gap: var(--space-2);
		align-items: center;
		flex-shrink: 0;
	}

	.hunk-action-btn:disabled {
		opacity: 0.5;
		cursor: not-allowed;
	}

	.hunk-action-btn:focus-visible {
		outline: var(--focus-ring-width) solid var(--focus-ring-color);
		outline-offset: 1px;
	}

	.stage-btn {
		background: color-mix(in srgb, var(--color-success) 10%, transparent);
		color: var(--color-success);
		border-color: var(--color-success);
	}

	.stage-btn:hover:not(:disabled) {
		background: color-mix(in srgb, var(--color-success) 20%, transparent);
	}

	.unstage-btn {
		background: color-mix(in srgb, var(--color-warning) 10%, transparent);
		color: var(--color-warning);
		border-color: var(--color-warning);
	}

	.unstage-btn:hover:not(:disabled) {
		background: color-mix(in srgb, var(--color-warning) 20%, transparent);
	}

	.btn-spinner {
		width: 12px;
		height: 12px;
		border: 2px solid currentColor;
		border-top-color: transparent;
		border-radius: 50%;
		animation: spin 0.6s linear infinite;
		display: inline-block;
	}

	/* ── Diff lines ────────────────────────────────────────────────── */

	.hunk-lines {
		font-family: var(--font-mono);
		font-size: var(--text-mono-sm-size);
		line-height: 1.5;
	}

	.diff-line {
		display: flex;
		padding: 0 var(--space-3);
		white-space: pre;
	}

	.diff-line.line-add {
		background: color-mix(in srgb, var(--color-success) 12%, transparent);
	}

	.diff-line.line-del {
		background: color-mix(in srgb, var(--color-danger) 12%, transparent);
	}

	.diff-line.line-ctx {
		background: transparent;
	}

	.line-origin {
		width: 16px;
		flex-shrink: 0;
		color: var(--color-text-muted);
		user-select: none;
	}

	.line-content {
		flex: 1;
		overflow-x: auto;
	}

	/* ── Line selection ──────────────────────────────────────────── */

	.diff-line.line-selectable {
		cursor: pointer;
	}

	.diff-line.line-selectable:hover {
		filter: brightness(0.92);
	}

	.diff-line.line-selected.line-add {
		background: color-mix(in srgb, var(--color-success) 30%, transparent);
		box-shadow: inset 3px 0 0 var(--color-success);
	}

	.diff-line.line-selected.line-del {
		background: color-mix(in srgb, var(--color-danger) 30%, transparent);
		box-shadow: inset 3px 0 0 var(--color-danger);
	}

	/* ── Conflict banner ────────────────────────────────────────────── */

	.conflict-banner {
		display: flex;
		align-items: center;
		gap: var(--space-3);
		padding: var(--space-3) var(--space-5);
		background: color-mix(in srgb, var(--color-danger) 10%, transparent);
		border-bottom: 1px solid var(--color-danger);
		flex-shrink: 0;
	}

	.conflict-banner-icon {
		display: inline-flex;
		align-items: center;
		justify-content: center;
		width: 20px;
		height: 20px;
		background: var(--color-danger);
		color: white;
		border-radius: 50%;
		font-weight: 700;
		font-size: 12px;
		flex-shrink: 0;
	}

	.conflict-banner-text {
		flex: 1;
		font-size: var(--text-body-sm-size);
		color: var(--color-text-primary);
		font-weight: 500;
	}

	.conflict-banner-btn {
		padding: var(--space-1) var(--space-4);
		font-size: var(--text-body-sm-size);
		font-weight: 500;
		background: var(--color-danger);
		color: white;
		border: none;
		border-radius: var(--radius-sm);
		cursor: pointer;
		transition: opacity var(--transition-fast);
		flex-shrink: 0;
	}

	.conflict-banner-btn:hover {
		opacity: 0.85;
	}
</style>
