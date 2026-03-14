<script lang="ts">
  import type { CommitNode, RefData } from './types';

  interface Props {
    node: CommitNode | null;
    onNavigateToCommit?: (commitId: string) => void;
  }

  let { node, onNavigateToCommit }: Props = $props();

  let shortHashCopied = $state(false);
  let fullHashCopied = $state(false);
  let bodyExpanded = $state(false);

  const titleLine = $derived(node?.data.message.split('\n')[0]?.trim() || '');
  const bodyLines = $derived(node ? node.data.message.split('\n').slice(1) : []);
  const bodyText = $derived(bodyLines.join('\n').trim());
  const shouldTruncateBody = $derived(bodyText.length > 280 || bodyLines.length > 4);
  const displayedBodyText = $derived(
    !shouldTruncateBody || bodyExpanded ? bodyText : bodyText.slice(0, 280).trimEnd()
  );

  const shortHash = $derived(node?.data.id.slice(0, 7) ?? '');

  const absoluteDate = $derived(
    node
      ? new Intl.DateTimeFormat(undefined, {
          month: 'short',
          day: 'numeric',
          year: 'numeric',
          hour: 'numeric',
          minute: '2-digit'
        }).format(new Date(node.data.time * 1000))
      : ''
  );

  const relativeDate = $derived(node ? formatRelativeTime(node.data.time * 1000) : '');

  const avatarInitials = $derived.by(() => {
    const name = node?.data.author_name.trim() ?? '';
    if (!name) return '?';

    const parts = name.split(/\s+/).filter(Boolean);
    if (parts.length === 1) return parts[0]?.slice(0, 2).toUpperCase() ?? '?';

    return `${parts[0]?.[0] ?? ''}${parts[1]?.[0] ?? ''}`.toUpperCase();
  });

  const branchColorStyle = $derived(
    node
      ? `background: var(--graph-color-${node.color}, var(--color-accent));`
      : 'background: var(--color-accent);'
  );

  $effect(() => {
    node?.data.id;
    bodyExpanded = false;
    shortHashCopied = false;
    fullHashCopied = false;
  });

  function formatRelativeTime(timestampMs: number): string {
    const rtf = new Intl.RelativeTimeFormat(undefined, { numeric: 'auto' });
    const diffMs = timestampMs - Date.now();
    const absSeconds = Math.abs(diffMs / 1000);

    if (absSeconds < 60) return rtf.format(Math.round(diffMs / 1000), 'second');
    if (absSeconds < 3600) return rtf.format(Math.round(diffMs / 60000), 'minute');
    if (absSeconds < 86400) return rtf.format(Math.round(diffMs / 3600000), 'hour');
    if (absSeconds < 604800) return rtf.format(Math.round(diffMs / 86400000), 'day');
    if (absSeconds < 2629800) return rtf.format(Math.round(diffMs / 604800000), 'week');
    if (absSeconds < 31557600) return rtf.format(Math.round(diffMs / 2629800000), 'month');
    return rtf.format(Math.round(diffMs / 31557600000), 'year');
  }

  async function copyText(value: string, target: 'short' | 'full'): Promise<void> {
    if (!value) return;

    try {
      await navigator.clipboard.writeText(value);
      if (target === 'short') {
        shortHashCopied = true;
        window.setTimeout(() => {
          shortHashCopied = false;
        }, 1500);
      } else {
        fullHashCopied = true;
        window.setTimeout(() => {
          fullHashCopied = false;
        }, 1500);
      }
    } catch {
      // Clipboard copy can fail in restricted environments.
    }
  }

  function toggleBodyExpanded(): void {
    bodyExpanded = !bodyExpanded;
  }

  function refClass(ref: RefData): string {
    switch (ref.ref_type) {
      case 'LocalBranch':
        return 'ref-badge local';
      case 'RemoteBranch':
        return 'ref-badge remote';
      case 'Tag':
        return 'ref-badge tag';
      case 'Head':
        return 'ref-badge head';
      default:
        return 'ref-badge';
    }
  }
</script>

<aside class="commit-detail" aria-label="Commit details">
  {#if !node}
    <div class="empty-state">Select a commit to view details</div>
  {:else}
    <section class="section header">
      <div class="header-top">
        <span class="color-dot" style={branchColorStyle} aria-hidden="true"></span>
        <span class="short-hash">{shortHash}</span>
        <button type="button" class="copy-button" onclick={() => copyText(node.data.id, 'short')}>
          {shortHashCopied ? 'Copied!' : 'Copy'}
        </button>
      </div>
    </section>

    <section class="section">
      <h2 class="message-title">{titleLine || '(no commit message)'}</h2>
      {#if bodyText}
        <p class="message-body">{displayedBodyText}{!bodyExpanded && shouldTruncateBody ? '…' : ''}</p>
        {#if shouldTruncateBody}
          <button type="button" class="text-button" onclick={toggleBodyExpanded}>
            {bodyExpanded ? 'Show less' : 'Show more'}
          </button>
        {/if}
      {/if}
    </section>

    <section class="section author">
      <div class="avatar" style={branchColorStyle} aria-hidden="true">{avatarInitials}</div>
      <div class="author-meta">
        <div class="author-name">{node.data.author_name}</div>
        <div class="author-email">{node.data.author_email}</div>
        <div class="author-date">{absoluteDate} · {relativeDate}</div>
      </div>
    </section>

    {#if node.refs.length > 0}
      <section class="section">
        <h3 class="section-title">Refs</h3>
        <div class="ref-list">
          {#each node.refs as ref (`${ref.ref_type}:${ref.name}`)}
            <span class={refClass(ref)}>{ref.name}</span>
          {/each}
        </div>
      </section>
    {/if}

    <section class="section">
      <h3 class="section-title">Parents</h3>
      {#if node.data.parent_ids.length === 0}
        <p class="muted">No parents (root commit)</p>
      {:else}
        <ul class="parent-list">
          {#each node.data.parent_ids as parentId}
            <li>
              <button type="button" class="parent-link" onclick={() => onNavigateToCommit?.(parentId)}>
                {parentId.slice(0, 7)}
              </button>
            </li>
          {/each}
        </ul>
      {/if}
    </section>

    <section class="section">
      <h3 class="section-title">Full hash</h3>
      <button type="button" class="full-hash" onclick={() => copyText(node.data.id, 'full')}>
        <span>{node.data.id}</span>
        <span class="copy-indicator">{fullHashCopied ? 'Copied!' : 'Click to copy'}</span>
      </button>
    </section>
  {/if}
</aside>

<style>
  .commit-detail {
    width: 100%;
    height: 100%;
    overflow: auto;
    padding: var(--space-6);
    border-left: 1px solid var(--color-border);
    background: var(--color-bg-surface);
    font-family: var(--font-sans);
  }

  .empty-state {
    display: grid;
    place-items: center;
    height: 100%;
    color: var(--color-text-muted);
  }

  .section {
    padding: var(--space-5) 0;
    border-top: 1px solid var(--color-border-subtle);
  }

  .section:first-child {
    padding-top: 0;
    border-top: 0;
  }

  .header-top {
    display: flex;
    align-items: center;
    gap: var(--space-4);
  }

  .color-dot {
    width: var(--space-4);
    height: var(--space-4);
    border-radius: var(--radius-full);
    flex: 0 0 auto;
  }

  .short-hash,
  .author-email,
  .parent-link,
  .full-hash {
    font-family: var(--font-mono);
  }

  .short-hash {
    color: var(--color-text-primary);
    font-size: 14px;
    letter-spacing: 0.02em;
  }

  .copy-button,
  .text-button,
  .parent-link,
  .full-hash {
    border: 1px solid var(--color-border);
    border-radius: var(--radius-sm);
    background: var(--color-bg-elevated);
    color: var(--color-text-primary);
  }

  .copy-button,
  .text-button,
  .parent-link {
    padding: var(--space-2) var(--space-4);
    cursor: pointer;
  }

  .copy-button {
    margin-left: auto;
  }

  .copy-button:hover,
  .text-button:hover,
  .parent-link:hover,
  .full-hash:hover {
    background: var(--color-bg-hover);
  }

  .message-title {
    font-size: 15px;
    font-weight: 600;
    color: var(--color-text-primary);
    margin-bottom: var(--space-4);
  }

  .message-body {
    color: var(--color-text-secondary);
    white-space: pre-wrap;
    line-height: 1.45;
    margin-bottom: var(--space-4);
  }

  .author {
    display: flex;
    gap: var(--space-5);
    align-items: flex-start;
  }

  .avatar {
    width: var(--space-10);
    height: var(--space-10);
    border-radius: var(--radius-full);
    display: grid;
    place-items: center;
    font-weight: 600;
    color: var(--color-bg);
    flex: 0 0 auto;
  }

  .author-meta {
    display: flex;
    flex-direction: column;
    gap: var(--space-2);
  }

  .author-name {
    color: var(--color-text-primary);
    font-weight: 500;
  }

  .author-email {
    color: var(--color-text-secondary);
  }

  .author-date {
    color: var(--color-text-muted);
  }

  .section-title {
    font-size: 12px;
    font-weight: 600;
    text-transform: uppercase;
    letter-spacing: 0.04em;
    color: var(--color-text-muted);
    margin-bottom: var(--space-4);
  }

  .ref-list {
    display: flex;
    flex-wrap: wrap;
    gap: var(--space-3);
  }

  .ref-badge {
    display: inline-flex;
    align-items: center;
    padding: var(--space-2) var(--space-4);
    border-radius: var(--radius-full);
    font-size: 12px;
    border: 1px solid var(--color-border);
    color: var(--color-text-primary);
    background: var(--color-bg-elevated);
  }

  .ref-badge.local {
    border-color: var(--color-accent);
    color: var(--color-accent);
    background: var(--color-accent-muted);
  }

  .ref-badge.remote {
    border-color: var(--color-border);
    color: var(--color-text-secondary);
    background: var(--color-accent-muted);
  }

  .ref-badge.tag {
    border-color: var(--color-warning);
    color: var(--color-warning);
    background: var(--color-warning-muted);
  }

  .ref-badge.head {
    border-width: 2px;
    font-weight: 600;
    border-color: var(--color-text-primary);
  }

  .parent-list {
    list-style: none;
    display: flex;
    flex-direction: column;
    gap: var(--space-3);
  }

  .parent-link {
    text-align: left;
    color: var(--color-accent);
    width: fit-content;
  }

  .muted {
    color: var(--color-text-muted);
  }

  .full-hash {
    width: 100%;
    text-align: left;
    padding: var(--space-4);
    cursor: pointer;
    display: flex;
    flex-direction: column;
    gap: var(--space-2);
  }

  .copy-indicator {
    font-family: var(--font-sans);
    color: var(--color-text-secondary);
    font-size: 12px;
  }
</style>
