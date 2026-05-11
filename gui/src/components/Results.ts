import { exportSdlMcpConfig, writeSdlMcpConfig } from '../bridge/tauri.js';
import { addLog, store } from '../state/store.js';

export function renderResults(container: HTMLElement): void {
  const wrapper = document.createElement('div');
  wrapper.className = 'content-wrapper section';
  container.appendChild(wrapper);

  const state = store.getState();
  const results = state.results;

  const header = document.createElement('div');
  header.className = 'flex items-center justify-between mb-lg animate-fade-in';
  header.innerHTML = `
    <div>
      <h2 class="text-xl text-success">Install Operation Complete</h2>
      <p class="text-sm text-secondary mt-xs">Managed language server state has been updated.</p>
    </div>
  `;
  wrapper.appendChild(header);

  if (!results) {
    const empty = document.createElement('div');
    empty.className = 'panel';
    empty.innerHTML = `
      <div class="empty-state">
        <div class="empty-state__icon text-muted">!</div>
        <div class="empty-state__title">No results available</div>
        <div class="empty-state__description">Install or remove a language server to see results here.</div>
      </div>
    `;
    wrapper.appendChild(empty);
    wrapper.appendChild(renderBackButton());
    return;
  }

  wrapper.appendChild(renderSummaryStats(results));
  wrapper.appendChild(renderOutcomeTable(results));
  wrapper.appendChild(renderActions());
}

function renderSummaryStats(results: NonNullable<ReturnType<typeof store.getState>['results']>): HTMLElement {
  const panel = document.createElement('div');
  panel.className = 'panel animate-fade-in';

  const sectionHeader = document.createElement('div');
  sectionHeader.className = 'section-header';
  sectionHeader.innerHTML = `
    <span class="section-header__title">Summary</span>
    <div class="section-header__line"></div>
  `;
  panel.appendChild(sectionHeader);

  const installed = results.outcomes.filter((o) => o.status === 'installed').length;
  const already = results.outcomes.filter((o) => o.status === 'already_installed').length;
  const removed = results.outcomes.filter((o) => o.status === 'removed').length;
  const failed = results.outcomes.filter((o) => o.status === 'failed').length;

  const grid = document.createElement('div');
  grid.className = 'grid grid-4 gap-lg';

  const stats = [
    { label: 'Installed', value: installed.toLocaleString(), color: 'text-success' },
    { label: 'Already Present', value: already.toLocaleString(), color: 'text-cyan' },
    { label: 'Removed', value: removed.toLocaleString(), color: 'text-primary' },
    { label: 'Failed', value: failed.toLocaleString(), color: failed ? 'text-error' : 'text-primary' },
  ];

  stats.forEach((stat, idx) => {
    const card = document.createElement('div');
    card.className = `panel panel--inset panel--compact animate-fade-in stagger-${idx + 1}`;
    card.innerHTML = `
      <div class="text-xs text-muted uppercase mb-xs mono">${stat.label}</div>
      <div class="text-lg font-semibold ${stat.color} tabular-nums">${stat.value}</div>
    `;
    grid.appendChild(card);
  });

  panel.appendChild(grid);

  if (results.totalDuration > 0) {
    const duration = document.createElement('div');
    duration.className = 'flex items-center gap-sm mt-lg text-sm text-secondary';
    duration.innerHTML = `
      <span>Duration:</span>
      <span class="text-cyan mono">${formatDuration(results.totalDuration)}</span>
      <span class="separator--vertical"></span>
      <span>Operations:</span>
      <span class="text-cyan mono">${results.outcomes.length}</span>
    `;
    panel.appendChild(duration);
  }

  return panel;
}

function renderOutcomeTable(results: NonNullable<ReturnType<typeof store.getState>['results']>): HTMLElement {
  const panel = document.createElement('div');
  panel.className = 'panel panel--flush animate-fade-in stagger-3';

  const headerRow = document.createElement('div');
  headerRow.className = 'panel__header px-xl pt-lg';
  headerRow.innerHTML = `
    <div>
      <div class="panel__title">Server Results</div>
      <div class="panel__subtitle">Per-server outcome from the last operation</div>
    </div>
  `;
  panel.appendChild(headerRow);

  const tableContainer = document.createElement('div');
  tableContainer.className = 'table-container';

  const table = document.createElement('table');
  table.className = 'table table--striped table--hover';
  table.innerHTML = `
    <thead>
      <tr>
        <th>Server</th>
        <th>Status</th>
        <th>Path / Message</th>
      </tr>
    </thead>
  `;

  const tbody = document.createElement('tbody');
  results.outcomes.forEach((outcome) => {
    const tr = document.createElement('tr');
    tr.innerHTML = `
      <td class="text-primary font-medium">${escapeHtml(outcome.name)}</td>
      <td>${formatOutcome(outcome.status)}</td>
      <td class="text-sm text-secondary mono">${escapeHtml(outcome.path || outcome.message)}</td>
    `;
    tbody.appendChild(tr);
  });

  table.appendChild(tbody);
  tableContainer.appendChild(table);
  panel.appendChild(tableContainer);

  return panel;
}

function renderActions(): HTMLElement {
  const actions = document.createElement('div');
  actions.className = 'flex gap-sm mt-lg animate-fade-in stagger-5';

  const backBtn = document.createElement('button');
  backBtn.className = 'btn btn--secondary';
  backBtn.textContent = 'Back to Dashboard';
  backBtn.addEventListener('click', () => {
    store.setState({ screen: 'dashboard' });
  });
  actions.appendChild(backBtn);

  const againBtn = document.createElement('button');
  againBtn.className = 'btn btn--primary';
  againBtn.textContent = 'Install More';
  againBtn.addEventListener('click', () => {
    store.setState({ screen: 'dashboard', results: null, logs: [], overallProgress: 0, pipelineStep: 'resolve' });
  });
  actions.appendChild(againBtn);

  const exportBtn = document.createElement('button');
  exportBtn.className = 'btn btn--secondary';
  exportBtn.textContent = 'Export SDL-MCP Config';
  exportBtn.addEventListener('click', handleSdlMcpExport);
  actions.appendChild(exportBtn);

  const writeBtn = document.createElement('button');
  writeBtn.className = 'btn btn--ghost';
  writeBtn.textContent = 'Write SDL-MCP Config';
  writeBtn.addEventListener('click', handleSdlMcpWrite);
  actions.appendChild(writeBtn);

  return actions;
}

async function handleSdlMcpExport(): Promise<void> {
  try {
    const info = await exportSdlMcpConfig(store.getState().projectPath, false, true);
    addLog('success', `Exported ${info.server_count} SDL-MCP LSP server config(s)`);
    info.skipped.forEach((message) => addLog('warning', message));
    showSdlMcpPreview(info.fragment_json, info.skipped);
  } catch (err) {
    const message = err instanceof Error ? err.message : String(err);
    addLog('error', `SDL-MCP export failed: ${message}`);
  }
}

async function handleSdlMcpWrite(): Promise<void> {
  const configPath = window.prompt('SDL-MCP config path');
  if (!configPath?.trim()) return;

  try {
    const info = await writeSdlMcpConfig(store.getState().projectPath, configPath.trim(), false, true, false);
    addLog('success', `Wrote ${info.server_count} SDL-MCP LSP server config(s)`);
    info.skipped.forEach((message) => addLog('warning', message));
    showSdlMcpPreview(info.fragment_json, info.skipped);
  } catch (err) {
    const message = err instanceof Error ? err.message : String(err);
    addLog('error', `SDL-MCP write failed: ${message}`);
  }
}

function showSdlMcpPreview(fragmentJson: string, skipped: string[]): void {
  document.getElementById('sdl-mcp-preview-modal')?.remove();

  const overlay = document.createElement('div');
  overlay.id = 'sdl-mcp-preview-modal';
  overlay.className = 'dialog-overlay';

  const dialog = document.createElement('div');
  dialog.className = 'dialog animate-fade-in';
  dialog.innerHTML = `
    <div class="dialog__header">SDL-MCP Config</div>
    <div class="dialog__body">
      <textarea class="input mono" rows="16" readonly>${escapeHtml(fragmentJson)}</textarea>
      ${skipped.length ? `<div class="manual-note mt-sm">${escapeHtml(skipped.join('\n'))}</div>` : ''}
    </div>
    <div class="dialog__footer flex gap-sm justify-end">
      <button class="btn btn--secondary" id="sdl-mcp-preview-close">Close</button>
    </div>
  `;

  overlay.appendChild(dialog);
  document.body.appendChild(overlay);
  const close = () => overlay.remove();
  overlay.addEventListener('click', (event) => {
    if (event.target === overlay) close();
  });
  document.getElementById('sdl-mcp-preview-close')?.addEventListener('click', close);
}

function renderBackButton(): HTMLElement {
  const actions = document.createElement('div');
  actions.className = 'flex gap-sm mt-lg';
  const backBtn = document.createElement('button');
  backBtn.className = 'btn btn--secondary';
  backBtn.textContent = 'Back to Dashboard';
  backBtn.addEventListener('click', () => {
    store.setState({ screen: 'dashboard' });
  });
  actions.appendChild(backBtn);
  return actions;
}

function formatOutcome(status: string): string {
  switch (status) {
    case 'installed': return 'Installed';
    case 'already_installed': return 'Already present';
    case 'removed': return 'Removed';
    case 'failed': return 'Failed';
    default: return status;
  }
}

function formatDuration(ms: number): string {
  if (ms < 1000) return `${ms.toFixed(0)}ms`;
  const seconds = ms / 1000;
  if (seconds < 60) return `${seconds.toFixed(1)}s`;
  const minutes = Math.floor(seconds / 60);
  const secs = (seconds % 60).toFixed(0);
  return `${minutes}m ${secs}s`;
}

function escapeHtml(str: string): string {
  const div = document.createElement('div');
  div.textContent = str;
  return div.innerHTML;
}
