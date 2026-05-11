import { store, addLog } from '../state/store.js';
import { getConfig, saveConfig, cleanCache, checkUpdates } from '../bridge/tauri.js';

export function renderSettings(container: HTMLElement): void {
  const wrapper = document.createElement('div');
  wrapper.className = 'content-wrapper section';
  container.appendChild(wrapper);

  const header = document.createElement('div');
  header.className = 'flex items-center justify-between mb-lg animate-fade-in';
  header.innerHTML = `
    <div>
      <h2 class="text-xl">Settings</h2>
      <p class="text-sm text-secondary mt-xs">Configure LSP-IO cache behavior and server status checks.</p>
    </div>
  `;
  wrapper.appendChild(header);

  wrapper.appendChild(renderGeneralSettings());
  wrapper.appendChild(renderServerReference());
  wrapper.appendChild(renderUpdatesSection());
  wrapper.appendChild(renderSaveActions());

  loadConfig();
}

function renderGeneralSettings(): HTMLElement {
  const panel = document.createElement('div');
  panel.className = 'panel animate-fade-in';

  const settings = store.getState().settings;

  const sectionHeader = document.createElement('div');
  sectionHeader.className = 'section-header';
  sectionHeader.innerHTML = `
    <span class="section-header__title">General</span>
    <div class="section-header__line"></div>
  `;
  panel.appendChild(sectionHeader);

  const grid = document.createElement('div');
  grid.className = 'grid grid-2 gap-lg';

  const pathField = document.createElement('div');
  pathField.className = 'form-field';
  pathField.innerHTML = `
    <label class="form-label">Prefer System PATH</label>
    <div class="flex items-center gap-sm">
      <label class="chip ${settings.preferPath ? 'chip--selected' : ''}" id="setting-prefer-path-chip">
        <span class="chip__checkbox"></span>
        <span class="chip__label">${settings.preferPath ? 'Enabled' : 'Disabled'}</span>
      </label>
      <span class="form-hint">Show existing system language servers before managed installs</span>
    </div>
  `;
  grid.appendChild(pathField);

  const timeoutField = document.createElement('div');
  timeoutField.className = 'form-field';
  timeoutField.innerHTML = `
    <label class="form-label">Install Timeout (seconds)</label>
    <input class="input" type="number" id="setting-timeout" value="${settings.timeout}" min="30" max="3600" placeholder="300" />
    <span class="form-hint">Maximum time per package manager operation</span>
  `;
  grid.appendChild(timeoutField);

  const cacheField = document.createElement('div');
  cacheField.className = 'form-field';
  cacheField.innerHTML = `
    <label class="form-label">Cache Directory</label>
    <div class="flex gap-sm">
      <input class="input flex-1" type="text" id="setting-cache-dir" value="${escapeAttr(settings.cacheDir)}" placeholder="(default system cache)" />
      <button class="btn btn--ghost btn--sm" id="btn-browse-cache">Browse</button>
    </div>
    <span class="form-hint">Where app-owned language servers are stored</span>
  `;
  grid.appendChild(cacheField);

  panel.appendChild(grid);

  setTimeout(() => {
    bindChip('setting-prefer-path-chip', 'preferPath');

    const browseCache = document.getElementById('btn-browse-cache');
    browseCache?.addEventListener('click', handleBrowseCache);
  }, 0);

  return panel;
}

function renderServerReference(): HTMLElement {
  const panel = document.createElement('div');
  panel.className = 'panel panel--flush animate-fade-in stagger-2';

  const headerRow = document.createElement('div');
  headerRow.className = 'panel__header px-xl pt-lg';
  headerRow.innerHTML = `
    <div>
      <div class="panel__title">Recommendation Reference</div>
      <div class="panel__subtitle">Current server choices and install ownership</div>
    </div>
  `;
  panel.appendChild(headerRow);

  const tableContainer = document.createElement('div');
  tableContainer.className = 'table-container';

  const table = document.createElement('table');
  table.className = 'table table--striped';
  table.innerHTML = `
    <thead>
      <tr>
        <th>Language</th>
        <th>Server</th>
        <th>Install</th>
        <th>Footprint</th>
      </tr>
    </thead>
  `;

  const tbody = document.createElement('tbody');
  store.getState().servers.forEach((server) => {
    const tr = document.createElement('tr');
    tr.innerHTML = `
      <td>${escapeHtml(server.languageDisplay)}</td>
      <td class="text-primary font-medium">${escapeHtml(server.name)}</td>
      <td class="text-cyan mono text-sm">${escapeHtml(server.installMethod)}</td>
      <td>${escapeHtml(server.footprint)}</td>
    `;
    tbody.appendChild(tr);
  });

  table.appendChild(tbody);
  tableContainer.appendChild(table);
  panel.appendChild(tableContainer);

  return panel;
}

function renderUpdatesSection(): HTMLElement {
  const panel = document.createElement('div');
  panel.className = 'panel animate-fade-in stagger-3';

  const sectionHeader = document.createElement('div');
  sectionHeader.className = 'section-header';
  sectionHeader.innerHTML = `
    <span class="section-header__title">Maintenance</span>
    <div class="section-header__line"></div>
  `;
  panel.appendChild(sectionHeader);

  const body = document.createElement('div');
  body.className = 'flex items-center gap-md';

  const checkBtn = document.createElement('button');
  checkBtn.className = 'btn btn--secondary';
  checkBtn.textContent = 'Check Versions';
  checkBtn.addEventListener('click', handleCheckUpdates);
  body.appendChild(checkBtn);

  const cleanBtn = document.createElement('button');
  cleanBtn.className = 'btn btn--ghost';
  cleanBtn.textContent = 'Clean Managed Cache';
  cleanBtn.addEventListener('click', handleCleanCache);
  body.appendChild(cleanBtn);

  const resultsArea = document.createElement('div');
  resultsArea.id = 'update-results';
  resultsArea.className = 'flex-1';
  body.appendChild(resultsArea);

  panel.appendChild(body);

  return panel;
}

function renderSaveActions(): HTMLElement {
  const actions = document.createElement('div');
  actions.className = 'flex justify-end gap-sm mt-lg animate-fade-in stagger-4';

  const resetBtn = document.createElement('button');
  resetBtn.className = 'btn btn--ghost';
  resetBtn.textContent = 'Reset to Defaults';
  resetBtn.addEventListener('click', handleReset);
  actions.appendChild(resetBtn);

  const saveBtn = document.createElement('button');
  saveBtn.className = 'btn btn--primary-filled';
  saveBtn.textContent = 'Save Settings';
  saveBtn.addEventListener('click', handleSave);
  actions.appendChild(saveBtn);

  return actions;
}

function bindChip(id: string, key: 'preferPath') {
  const chip = document.getElementById(id);
  chip?.addEventListener('click', () => {
    const current = store.getState().settings;
    const next = { ...current, [key]: !current[key] };
    store.setState({ settings: next });
    chip.classList.toggle('chip--selected');
    const label = chip.querySelector('.chip__label');
    if (label) label.textContent = next[key] ? 'Enabled' : 'Disabled';
  });
}

async function loadConfig() {
  try {
    const config = await getConfig(store.getState().projectPath);
    if (config && typeof config === 'object') {
      const c = config as Record<string, unknown>;
      const current = store.getState().settings;
      store.setState({
        settings: {
          preferPath: typeof c.prefer_path === 'boolean' ? c.prefer_path : current.preferPath,
          timeout: typeof c.timeout === 'number' ? c.timeout : current.timeout,
          cacheDir: typeof c.cache_dir === 'string' ? c.cache_dir : current.cacheDir,
        },
      });
    }
  } catch {
    // Config not available; defaults are fine.
  }
}

async function handleSave() {
  const timeoutInput = document.getElementById('setting-timeout') as HTMLInputElement | null;
  const cacheInput = document.getElementById('setting-cache-dir') as HTMLInputElement | null;

  const current = store.getState().settings;
  const settings = {
    preferPath: current.preferPath,
    timeout: timeoutInput ? parseInt(timeoutInput.value, 10) || 300 : current.timeout,
    cacheDir: cacheInput ? cacheInput.value : current.cacheDir,
  };

  store.setState({ settings });

  const config = {
    prefer_path: settings.preferPath,
    timeout: settings.timeout,
    cache_dir: settings.cacheDir || undefined,
    overrides: [],
  };

  try {
    await saveConfig(store.getState().projectPath, config);
    addLog('success', 'Settings saved successfully');
    showNotification('Settings saved', 'success');
  } catch (err) {
    const message = err instanceof Error ? err.message : String(err);
    addLog('error', `Failed to save settings: ${message}`);
    showNotification('Failed to save settings', 'error');
  }
}

function handleReset() {
  store.setState({
    settings: {
      preferPath: true,
      timeout: 300,
      cacheDir: '',
    },
  });

  const timeoutInput = document.getElementById('setting-timeout') as HTMLInputElement | null;
  const cacheInput = document.getElementById('setting-cache-dir') as HTMLInputElement | null;
  const preferChip = document.getElementById('setting-prefer-path-chip');

  if (timeoutInput) timeoutInput.value = '300';
  if (cacheInput) cacheInput.value = '';
  preferChip?.classList.add('chip--selected');
  const preferLabel = preferChip?.querySelector('.chip__label');
  if (preferLabel) preferLabel.textContent = 'Enabled';

  addLog('info', 'Settings reset to defaults');
  showNotification('Settings reset', 'info');
}

async function handleBrowseCache() {
  try {
    const { open } = await import('@tauri-apps/plugin-dialog');
    const selected = await open({ directory: true, multiple: false, title: 'Select Cache Directory' });
    if (selected && typeof selected === 'string') {
      const input = document.getElementById('setting-cache-dir') as HTMLInputElement | null;
      if (input) input.value = selected;
    }
  } catch {
    const path = prompt('Enter cache directory path:');
    if (path) {
      const input = document.getElementById('setting-cache-dir') as HTMLInputElement | null;
      if (input) input.value = path;
    }
  }
}

async function handleCheckUpdates() {
  const resultsArea = document.getElementById('update-results');
  if (resultsArea) {
    resultsArea.innerHTML = '<span class="spinner spinner--sm"></span> <span class="text-sm text-muted ml-sm">Checking...</span>';
  }

  try {
    const updates = await checkUpdates();
    const managed = updates.filter((u) => u.latest_version !== 'system-managed').length;
    if (resultsArea) {
      resultsArea.innerHTML = `<span class="text-sm text-success">${managed} managed server(s) use package-manager latest or pinned release metadata.</span>`;
    }
    addLog('info', `Version check complete for ${updates.length} server(s)`);
  } catch (err) {
    const message = err instanceof Error ? err.message : String(err);
    if (resultsArea) {
      resultsArea.innerHTML = `<span class="text-sm text-error">Check failed: ${escapeHtml(message)}</span>`;
    }
    addLog('error', `Version check failed: ${message}`);
  }
}

async function handleCleanCache() {
  try {
    const result = await cleanCache(store.getState().projectPath);
    addLog('success', result);
    showNotification('Managed cache cleaned', 'success');
  } catch (err) {
    const message = err instanceof Error ? err.message : String(err);
    addLog('error', `Failed to clean cache: ${message}`);
    showNotification('Failed to clean cache', 'error');
  }
}

function showNotification(message: string, type: 'success' | 'error' | 'info') {
  const existing = document.querySelector('.notification-toast');
  if (existing) existing.remove();

  const toast = document.createElement('div');
  toast.className = 'notification-toast fixed animate-slide-in-right';
  toast.style.bottom = 'var(--space-xl)';
  toast.style.right = 'var(--space-xl)';
  toast.style.zIndex = '1000';

  const colorClass = type === 'success' ? 'text-success' : type === 'error' ? 'text-error' : 'text-cyan';
  const bgClass = type === 'success' ? 'color-success-dim' : type === 'error' ? 'color-error-dim' : 'accent-cyan-dim';

  toast.innerHTML = `
    <div class="panel panel--compact flex items-center gap-sm" style="background: var(--${bgClass}); border-color: var(--${type === 'success' ? 'color-success' : type === 'error' ? 'color-error' : 'accent-cyan'});">
      <span class="${colorClass} text-sm font-medium">${escapeHtml(message)}</span>
    </div>
  `;

  document.body.appendChild(toast);

  setTimeout(() => {
    toast.classList.remove('animate-slide-in-right');
    toast.classList.add('animate-slide-out-right');
    setTimeout(() => toast.remove(), 300);
  }, 3000);
}

function escapeHtml(str: string): string {
  const div = document.createElement('div');
  div.textContent = str;
  return div.innerHTML;
}

function escapeAttr(str: string): string {
  return str.replace(/&/g, '&amp;').replace(/"/g, '&quot;').replace(/</g, '&lt;').replace(/>/g, '&gt;');
}
