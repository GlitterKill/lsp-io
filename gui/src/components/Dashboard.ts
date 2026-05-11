import { store, addLog } from '../state/store.js';
import type { ServerStatus, ServerSortKey } from '../state/store.js';
import {
  detectLanguages,
  getServerStatus,
  installOneServer,
  installServers,
  removeOneServer,
  type ServerStatusInfo,
} from '../bridge/tauri.js';
import { renderStatusBadge } from './StatusBadge.js';

export function initApp(_container: HTMLElement): void {
  // Legacy stub; the real entry point is main.ts.
}

export function renderDashboard(container: HTMLElement): void {
  const wrapper = document.createElement('div');
  wrapper.className = 'content-wrapper section';
  container.appendChild(wrapper);

  const header = document.createElement('div');
  header.className = 'flex items-center justify-between mb-xl';
  header.innerHTML = `
    <div>
      <h1 class="text-2xl animate-neon">LSP-IO</h1>
      <p class="text-sm text-secondary mt-xs">Language Server Install Manager</p>
    </div>
  `;
  wrapper.appendChild(header);

  wrapper.appendChild(renderProjectPathSection());

  const langSection = document.createElement('div');
  langSection.id = 'languages-section';
  wrapper.appendChild(langSection);
  renderLanguagesSection(langSection);

  const serverSection = document.createElement('div');
  serverSection.id = 'server-section';
  wrapper.appendChild(serverSection);
  renderServerTable(serverSection);

  const dashboardUnsub = store.subscribe((state) => {
    const langEl = wrapper.querySelector('#languages-section');
    if (langEl) {
      langEl.innerHTML = '';
      renderLanguagesSection(langEl as HTMLElement);
    }

    const serverEl = wrapper.querySelector('#server-section');
    if (serverEl) {
      serverEl.innerHTML = '';
      renderServerTable(serverEl as HTMLElement);
    }

    const installBtn = wrapper.querySelector('#btn-install-selected') as HTMLButtonElement | null;
    if (installBtn) {
      installBtn.disabled = state.isInstalling;
    }
  });

  const screenUnsub = store.subscribe((state) => {
    if (state.screen !== 'dashboard') {
      dashboardUnsub();
      screenUnsub();
    }
  });

  handleDetect();
  handleFetchServers();
}

function renderProjectPathSection(): HTMLElement {
  const panel = document.createElement('div');
  panel.className = 'panel animate-fade-in';

  const sectionHeader = document.createElement('div');
  sectionHeader.className = 'section-header';
  sectionHeader.innerHTML = `
    <span class="section-header__title">Project Path</span>
    <div class="section-header__line"></div>
  `;
  panel.appendChild(sectionHeader);

  const row = document.createElement('div');
  row.className = 'flex flex-wrap gap-sm items-end';

  const field = document.createElement('div');
  field.className = 'form-field flex-1';
  field.innerHTML = `
    <label class="form-label">Working Directory</label>
    <input class="input" type="text" id="project-path-input"
           value="${escapeHtml(store.getState().projectPath)}"
           placeholder="Path to project root..." />
  `;
  row.appendChild(field);

  const browseBtn = document.createElement('button');
  browseBtn.className = 'btn btn--secondary';
  browseBtn.textContent = 'Browse';
  browseBtn.addEventListener('click', handleBrowse);
  row.appendChild(browseBtn);

  panel.appendChild(row);

  const actions = document.createElement('div');
  actions.className = 'flex flex-wrap gap-sm mt-lg';

  const detectBtn = document.createElement('button');
  detectBtn.className = 'btn btn--secondary';
  detectBtn.textContent = 'Detect Languages';
  detectBtn.addEventListener('click', handleDetect);
  actions.appendChild(detectBtn);

  const refreshBtn = document.createElement('button');
  refreshBtn.className = 'btn btn--ghost';
  refreshBtn.textContent = 'Refresh Status';
  refreshBtn.addEventListener('click', handleFetchServers);
  actions.appendChild(refreshBtn);

  const installBtn = document.createElement('button');
  installBtn.className = 'btn btn--primary-filled btn--lg';
  installBtn.id = 'btn-install-selected';
  installBtn.textContent = 'Install Selected';
  installBtn.disabled = store.getState().isInstalling;
  installBtn.addEventListener('click', handleInstallSelected);
  actions.appendChild(installBtn);

  panel.appendChild(actions);

  return panel;
}

function renderLanguagesSection(container: HTMLElement): void {
  const state = store.getState();
  const panel = document.createElement('div');
  panel.className = 'panel animate-fade-in stagger-2';

  const sectionHeader = document.createElement('div');
  sectionHeader.className = 'section-header';
  sectionHeader.innerHTML = `
    <span class="section-header__title">Detected Languages</span>
    ${state.languages.length > 0 ? `<span class="section-header__count">${state.languages.length}</span>` : ''}
    <div class="section-header__line"></div>
  `;
  panel.appendChild(sectionHeader);

  if (state.languages.length === 0) {
    const empty = document.createElement('div');
    empty.className = 'empty-state';
    empty.innerHTML = `
      <div class="empty-state__icon text-muted">?</div>
      <div class="empty-state__title">No languages detected</div>
      <div class="empty-state__description">Scan a project to map it to recommended language servers.</div>
    `;
    panel.appendChild(empty);
  } else {
    const chipGroup = document.createElement('div');
    chipGroup.className = 'chip-group';

    state.languages.forEach((lang, index) => {
      const chip = document.createElement('div');
      chip.className = `chip${lang.selected ? ' chip--selected' : ''} animate-fade-in stagger-${Math.min(index + 1, 8)}`;
      chip.innerHTML = `
        <span class="chip__checkbox"></span>
        <span class="chip__label">${escapeHtml(lang.displayName)}</span>
      `;
      chip.title = lang.evidence;
      chip.addEventListener('click', () => {
        const langs = store.getState().languages.map((l) =>
          l.name === lang.name ? { ...l, selected: !l.selected } : l
        );
        store.setState({ languages: langs });
      });
      chipGroup.appendChild(chip);
    });

    panel.appendChild(chipGroup);
  }

  container.appendChild(panel);
}

function renderServerTable(container: HTMLElement): void {
  const state = store.getState();
  const panel = document.createElement('div');
  panel.className = 'panel panel--flush animate-fade-in stagger-3';

  const detectedLanguages = new Set(state.languages.map((language) => language.name));
  const categoryFiltered = state.serverCategoryFilter === 'all'
    ? state.servers
    : state.servers.filter((server) => server.languageCategory === state.serverCategoryFilter);
  const recommended = sortServers(categoryFiltered, detectedLanguages, state.serverSort);
  const categories = Array.from(
    new Map(state.servers.map((server) => [server.languageCategory, server.languageCategoryDisplay]))
  ).sort((a, b) => a[1].localeCompare(b[1]));

  const headerRow = document.createElement('div');
  headerRow.className = 'panel__header px-xl pt-lg';
  headerRow.innerHTML = `
    <div>
      <div class="panel__title">Language Server Status</div>
      <div class="panel__subtitle">Full registry; detected language chips only control Install Selected</div>
    </div>
  `;
  panel.appendChild(headerRow);

  if (state.servers.length === 0) {
    const empty = document.createElement('div');
    empty.className = 'empty-state';
    empty.innerHTML = `
      <div class="empty-state__icon text-muted">!</div>
      <div class="empty-state__title">No server status loaded</div>
      <div class="empty-state__description">Refresh status to query the backend for recommended language servers.</div>
    `;
    panel.appendChild(empty);
  } else {
    panel.appendChild(renderCategoryFilters(categories, state.serverCategoryFilter));

    const note = document.createElement('div');
    note.className = 'manual-note mx-xl';
    note.textContent = 'Manual/system rows are tracked on PATH and never removed by LSP-IO; managed rows are installed into the app cache.';
    panel.appendChild(note);

    const tableContainer = document.createElement('div');
    tableContainer.className = 'table-container';

    const table = document.createElement('table');
    table.className = 'table table--striped table--hover';
    table.appendChild(renderServerTableHead(state.serverSort.key, state.serverSort.direction));

    const tbody = document.createElement('tbody');
    recommended.forEach((server) => tbody.appendChild(renderServerRow(server, detectedLanguages.has(server.language))));

    table.appendChild(tbody);
    tableContainer.appendChild(table);
    panel.appendChild(tableContainer);
  }

  container.appendChild(panel);
}

function renderServerTableHead(activeKey: ServerSortKey, direction: 'asc' | 'desc'): HTMLTableSectionElement {
  const thead = document.createElement('thead');
  const tr = document.createElement('tr');

  const columns: Array<{ key: ServerSortKey; label: string; className?: string }> = [
    { key: 'server', label: 'Server' },
    { key: 'language', label: 'Language' },
    { key: 'detected', label: 'Detected' },
    { key: 'category', label: 'Category' },
    { key: 'install', label: 'Install' },
    { key: 'footprint', label: 'Footprint' },
    { key: 'status', label: 'Status', className: 'col-status' },
    { key: 'action', label: 'Action', className: 'col-status' },
  ];

  columns.forEach((column) => {
    const th = document.createElement('th');
    if (column.className) th.className = column.className;
    th.appendChild(renderSortButton(column.key, column.label, activeKey, direction));
    tr.appendChild(th);
  });

  thead.appendChild(tr);
  return thead;
}

function renderSortButton(
  key: ServerSortKey,
  label: string,
  activeKey: ServerSortKey,
  direction: 'asc' | 'desc',
): HTMLElement {
  const btn = document.createElement('button');
  const isActive = key === activeKey;
  btn.className = `table-sort${isActive ? ' table-sort--active' : ''}`;
  btn.type = 'button';
  btn.setAttribute('aria-label', `Sort by ${label}`);
  btn.setAttribute('aria-sort', isActive ? (direction === 'asc' ? 'ascending' : 'descending') : 'none');
  btn.innerHTML = `
    <span>${escapeHtml(label)}</span>
    <span class="table-sort__indicator">${isActive ? (direction === 'asc' ? '^' : 'v') : '-'}</span>
  `;
  btn.addEventListener('click', () => {
    const current = store.getState().serverSort;
    store.setState({
      serverSort: {
        key,
        direction: current.key === key && current.direction === 'asc' ? 'desc' : 'asc',
      },
    });
  });
  return btn;
}

function renderCategoryFilters(categories: [string, string][], active: string): HTMLElement {
  const wrapper = document.createElement('div');
  wrapper.className = 'server-filters px-xl';

  const allBtn = renderCategoryFilterButton('all', 'All', active);
  wrapper.appendChild(allBtn);

  categories.forEach(([name, label]) => {
    wrapper.appendChild(renderCategoryFilterButton(name, label, active));
  });

  return wrapper;
}

function renderCategoryFilterButton(name: string, label: string, active: string): HTMLElement {
  const btn = document.createElement('button');
  btn.className = `filter-chip${active === name ? ' filter-chip--active' : ''}`;
  btn.textContent = label;
  btn.addEventListener('click', () => store.setState({ serverCategoryFilter: name }));
  return btn;
}

function renderServerRow(server: ServerStatus, detected: boolean): HTMLElement {
  const tr = document.createElement('tr');
  tr.title = server.rationale;

  const tdName = document.createElement('td');
  tdName.innerHTML = `
    <div class="text-primary font-medium">${escapeHtml(server.name)}</div>
    <div class="text-xs text-muted mono">${escapeHtml(server.binaryName)}</div>
  `;
  tr.appendChild(tdName);

  const tdLang = document.createElement('td');
  tdLang.textContent = server.languageDisplay;
  tr.appendChild(tdLang);

  const tdDetected = document.createElement('td');
  tdDetected.appendChild(renderDetectedBadge(detected));
  tr.appendChild(tdDetected);

  const tdCategory = document.createElement('td');
  tdCategory.textContent = server.languageCategoryDisplay;
  tr.appendChild(tdCategory);

  const tdInstall = document.createElement('td');
  tdInstall.innerHTML = `
    <span class="text-cyan mono text-sm">${escapeHtml(server.installMethod)}</span>
    ${server.maturity !== 'Stable' ? `<span class="badge badge--outdated ml-sm"><span class="badge__dot"></span>${escapeHtml(server.maturity)}</span>` : ''}
    ${server.installWarning ? `<span class="badge badge--outdated ml-sm" title="${escapeHtml(server.installWarning)}"><span class="badge__dot"></span>Warning</span>` : ''}
  `;
  tr.appendChild(tdInstall);

  const tdFootprint = document.createElement('td');
  tdFootprint.textContent = server.footprint;
  tr.appendChild(tdFootprint);

  const tdStatus = document.createElement('td');
  tdStatus.className = 'col-status';
  tdStatus.appendChild(renderStatusBadge(statusBadge(server)));
  tr.appendChild(tdStatus);

  const tdAction = document.createElement('td');
  tdAction.className = 'col-status';
  tdAction.appendChild(renderServerAction(server));
  tr.appendChild(tdAction);

  return tr;
}

function renderDetectedBadge(detected: boolean): HTMLElement {
  const badge = document.createElement('span');
  badge.className = `badge ${detected ? 'badge--running' : 'badge--not-installed'}`;
  badge.innerHTML = `
    <span class="badge__dot"></span>
    <span>${detected ? 'Detected' : 'Undetected'}</span>
  `;
  return badge;
}

function renderServerAction(server: ServerStatus): HTMLElement {
  const btn = document.createElement('button');
  btn.className = 'btn btn--ghost btn--sm';

  if (server.canRemove) {
    btn.textContent = 'Remove';
    btn.addEventListener('click', () => handleRemoveServer(server));
  } else if (server.canInstall && !server.installed) {
    btn.textContent = 'Install';
    btn.addEventListener('click', () => handleInstallSingle(server));
  } else if (server.manualInstructions) {
    btn.textContent = 'Guide';
    btn.addEventListener('click', () => {
      addLog('info', `${server.name}: ${server.manualInstructions}`);
      window.open(server.sourceUrl, '_blank');
    });
  } else {
    btn.textContent = 'Ready';
    btn.disabled = true;
  }

  return btn;
}

function sortServers(
  servers: ServerStatus[],
  detectedLanguages: Set<string>,
  sort: { key: ServerSortKey; direction: 'asc' | 'desc' },
): ServerStatus[] {
  const direction = sort.direction === 'asc' ? 1 : -1;
  return [...servers].sort((a, b) => {
    const result = compareSortValues(
      sortValue(a, sort.key, detectedLanguages),
      sortValue(b, sort.key, detectedLanguages),
    );
    if (result !== 0) return result * direction;

    return compareSortValues(
      `${a.languageDisplay} ${a.name}`.toLowerCase(),
      `${b.languageDisplay} ${b.name}`.toLowerCase(),
    );
  });
}

function sortValue(server: ServerStatus, key: ServerSortKey, detectedLanguages: Set<string>): string | number {
  switch (key) {
    case 'server':
      return server.name.toLowerCase();
    case 'language':
      return server.languageDisplay.toLowerCase();
    case 'detected':
      return detectedLanguages.has(server.language) ? 0 : 1;
    case 'category':
      return server.languageCategoryDisplay.toLowerCase();
    case 'install':
      return server.installMethod.toLowerCase();
    case 'footprint':
      return footprintRank(server.footprint);
    case 'status':
      return statusRank(server);
    case 'action':
      return actionLabel(server).toLowerCase();
  }
}

function compareSortValues(a: string | number, b: string | number): number {
  if (typeof a === 'number' && typeof b === 'number') return a - b;
  return String(a).localeCompare(String(b), undefined, { numeric: true, sensitivity: 'base' });
}

function footprintRank(footprint: string): number {
  switch (footprint) {
    case 'Low': return 0;
    case 'Medium': return 1;
    case 'High': return 2;
    case 'Toolchain-bound': return 3;
    default: return 4;
  }
}

function statusRank(server: ServerStatus): number {
  if (server.installState === 'managed') return 0;
  if (server.installState === 'system') return 1;
  if (!server.canInstall) return 2;
  return 3;
}

function actionLabel(server: ServerStatus): string {
  if (server.canRemove) return 'Remove';
  if (server.canInstall && !server.installed) return 'Install';
  if (server.manualInstructions) return 'Guide';
  return 'Ready';
}

export function selectInstallTargetsForLanguages(
  servers: ServerStatus[],
  selectedLanguages: Set<string>,
): string[] {
  const serversByLanguage = new Map<string, ServerStatus[]>();
  for (const server of servers) {
    if (!selectedLanguages.has(server.language)) continue;
    const group = serversByLanguage.get(server.language) ?? [];
    group.push(server);
    serversByLanguage.set(server.language, group);
  }

  const targets: string[] = [];
  for (const candidates of serversByLanguage.values()) {
    if (candidates.some((server) => server.installed)) continue;
    const preferred = candidates.find((server) => server.canInstall && !server.installed);
    if (preferred) targets.push(preferred.id);
  }
  return targets;
}

function statusBadge(server: ServerStatus) {
  if (server.installState === 'managed') return 'installed';
  if (server.installState === 'system') return 'system';
  if (!server.canInstall) return 'manual';
  return 'not-installed';
}

async function handleBrowse() {
  if (isTauriRuntime()) {
    try {
      const { open } = await import('@tauri-apps/plugin-dialog');
      const selected = await open({ directory: true, multiple: false, title: 'Select Project Directory' });
      if (selected && typeof selected === 'string') {
        applyProjectPath(selected);
      }
      return;
    } catch {
      // Fall through to the browser fallback.
    }
  }

  showPathModal();
}

function showPathModal() {
  document.getElementById('path-modal')?.remove();

  const overlay = document.createElement('div');
  overlay.id = 'path-modal';
  overlay.className = 'dialog-overlay';

  const dialog = document.createElement('div');
  dialog.className = 'dialog animate-fade-in';
  dialog.innerHTML = `
    <div class="dialog__header">Enter Project Path</div>
    <div class="dialog__body">
      <div class="form-field">
        <label class="form-label">Project Directory</label>
        <input class="input" type="text" id="modal-path-input"
               value="${escapeHtml(store.getState().projectPath)}"
               placeholder="/path/to/project" autofocus />
      </div>
    </div>
    <div class="dialog__footer flex gap-sm justify-end">
      <button class="btn btn--ghost" id="modal-cancel">Cancel</button>
      <button class="btn btn--primary" id="modal-confirm">Confirm</button>
    </div>
  `;

  overlay.appendChild(dialog);
  document.body.appendChild(overlay);

  const input = document.getElementById('modal-path-input') as HTMLInputElement;
  input?.focus();
  input?.select();

  const close = () => overlay.remove();
  overlay.addEventListener('click', (e) => {
    if (e.target === overlay) close();
  });
  document.getElementById('modal-cancel')?.addEventListener('click', close);
  document.getElementById('modal-confirm')?.addEventListener('click', () => {
    const val = input?.value?.trim();
    if (val) applyProjectPath(val);
    close();
  });
  input?.addEventListener('keydown', (e) => {
    if (e.key === 'Enter') {
      const val = input.value.trim();
      if (val) applyProjectPath(val);
      close();
    } else if (e.key === 'Escape') {
      close();
    }
  });
}

function applyProjectPath(path: string) {
  store.setState({ projectPath: path });
  const input = document.getElementById('project-path-input') as HTMLInputElement | null;
  if (input) input.value = path;
  addLog('info', `Project path set to: ${path}`);
  handleDetect();
}

async function handleDetect() {
  const input = document.getElementById('project-path-input') as HTMLInputElement | null;
  if (input) {
    store.setState({ projectPath: input.value });
  }

  const path = store.getState().projectPath;
  addLog('info', `Detecting languages in: ${path}`);

  try {
    const result = await detectLanguages(path);
    const languages = result.map((lang) => ({
      name: lang.name,
      displayName: lang.display_name,
      category: lang.category,
      categoryDisplay: lang.category_display,
      confidence: lang.confidence,
      evidence: lang.evidence,
      selected: true,
    }));
    store.setState({ languages });
    addLog('success', `Detected ${languages.length} language(s): ${languages.map((l) => l.displayName).join(', ')}`);
  } catch (err) {
    const message = err instanceof Error ? err.message : String(err);
    addLog('error', `Detection failed: ${message}`);
    if (isTauriRuntime()) {
      store.setState({ languages: [] });
    } else if (!store.getState().languages.length) {
      store.setState({
        languages: [
          { name: 'typescript', displayName: 'TypeScript', category: 'programming', categoryDisplay: 'Programming', confidence: 'High', evidence: 'tsconfig.json', selected: true },
          { name: 'rust', displayName: 'Rust', category: 'programming', categoryDisplay: 'Programming', confidence: 'High', evidence: 'Cargo.toml', selected: true },
          { name: 'go', displayName: 'Go', category: 'programming', categoryDisplay: 'Programming', confidence: 'High', evidence: 'go.mod', selected: false },
        ],
      });
    }
  }
}

async function handleFetchServers() {
  try {
    const result = await getServerStatus(store.getState().projectPath);
    store.setState({ servers: result.map(mapServerStatus) });
  } catch (err) {
    const message = err instanceof Error ? err.message : String(err);
    addLog('warning', `Could not fetch language server status: ${message}`);
    if (isTauriRuntime()) {
      store.setState({ servers: [] });
    } else if (!store.getState().servers.length) {
      store.setState({ servers: mockServers() });
    }
  }
}

async function handleInstallSelected() {
  const state = store.getState();
  const selected = new Set(state.languages.filter((l) => l.selected).map((l) => l.name));
  const targets = selectInstallTargetsForLanguages(state.servers, selected);

  if (targets.length === 0) {
    addLog('warning', 'No selected missing servers can be managed automatically');
    return;
  }

  const started = performance.now();
  store.setState({
    isInstalling: true,
    screen: 'installing',
    pipelineStep: 'resolve',
    overallProgress: 0,
    logs: [],
    serverProgress: new Map(
      targets.map((id) => {
        const server = state.servers.find((s) => s.id === id);
        return [server?.name || id, { server: server?.name || id, status: 'queued', progress: 0, message: 'Queued' }];
      })
    ),
  });
  state.servers
    .filter((server) => targets.includes(server.id) && server.installWarning)
    .forEach((server) => addLog('warning', `${server.name}: ${server.installWarning}`));

  try {
    const outcomes = await installServers(targets, state.projectPath);
    store.setState({
      isInstalling: false,
      pipelineStep: 'done',
      overallProgress: 100,
      results: { outcomes: outcomes.map(mapOutcome), totalDuration: performance.now() - started },
    });
    await handleFetchServers();
  } catch (err) {
    const message = err instanceof Error ? err.message : String(err);
    addLog('error', `Install failed: ${message}`);
    store.setState({ isInstalling: false });
  }
}

async function handleInstallSingle(server: ServerStatus) {
  const started = performance.now();
  store.setState({
    isInstalling: true,
    screen: 'installing',
    pipelineStep: 'install',
    overallProgress: 20,
    logs: [],
    serverProgress: new Map([
      [server.name, { server: server.name, status: 'queued', progress: 0, message: 'Queued' }],
    ]),
  });
  if (server.installWarning) {
    addLog('warning', `${server.name}: ${server.installWarning}`);
  }

  try {
    const outcome = await installOneServer(server.id, store.getState().projectPath);
    store.setState({
      isInstalling: false,
      pipelineStep: 'done',
      overallProgress: 100,
      results: { outcomes: [mapOutcome(outcome)], totalDuration: performance.now() - started },
    });
    await handleFetchServers();
  } catch (err) {
    const message = err instanceof Error ? err.message : String(err);
    addLog('error', `Install failed: ${message}`);
    store.setState({ isInstalling: false });
  }
}

async function handleRemoveServer(server: ServerStatus) {
  try {
    const outcome = await removeOneServer(server.id, store.getState().projectPath);
    addLog('success', outcome.message);
    await handleFetchServers();
  } catch (err) {
    const message = err instanceof Error ? err.message : String(err);
    addLog('error', `Remove failed: ${message}`);
  }
}

function mapServerStatus(info: ServerStatusInfo): ServerStatus {
  return {
    id: info.id,
    name: info.name,
    language: info.language,
    languageDisplay: info.language_display,
    languageCategory: info.language_category,
    languageCategoryDisplay: info.language_category_display,
    version: info.version,
    binaryName: info.binary_name,
    installMethod: info.install_method,
    installed: info.installed,
    installState: info.install_state,
    installedPath: info.installed_path,
    canInstall: info.can_install,
    canRemove: info.can_remove,
    footprint: info.footprint,
    maturity: info.maturity,
    sourceUrl: info.source_url,
    rationale: info.rationale,
    manualInstructions: info.manual_instructions,
    installWarning: info.install_warning,
  };
}

function mapOutcome(outcome: { id: string; name: string; path: string | null; status: string; message: string }) {
  return {
    id: outcome.id,
    name: outcome.name,
    path: outcome.path,
    status: outcome.status as 'installed' | 'already_installed' | 'removed' | 'failed',
    message: outcome.message,
  };
}

function mockServers(): ServerStatus[] {
  const rows: Array<[string, string, string, string, string, string, string, string, boolean?, string?]> = [
    ['typescript-language-server', 'typescript-language-server', 'typescript', 'TypeScript', 'programming', 'Programming', 'npm', 'Medium'],
    ['javascript-language-server', 'typescript-language-server', 'javascript', 'JavaScript', 'programming', 'Programming', 'npm', 'Medium'],
    ['pyright', 'pyright-langserver', 'python', 'Python', 'programming', 'Programming', 'npm', 'Medium'],
    ['rust-analyzer', 'rust-analyzer', 'rust', 'Rust', 'programming', 'Programming', 'github release', 'Low'],
    ['gopls', 'gopls', 'go', 'Go', 'programming', 'Programming', 'go install', 'Low'],
    ['jdtls', 'Eclipse JDT LS', 'java', 'Java', 'programming', 'Programming', 'system/manual', 'High', false],
    ['csharp-ls', 'csharp-ls', 'csharp', 'C#', 'programming', 'Programming', 'dotnet tool', 'Medium'],
    ['ruby-lsp', 'ruby-lsp', 'ruby', 'Ruby', 'programming', 'Programming', 'gem', 'Medium'],
    ['kotlin-lsp', 'kotlin-lsp', 'kotlin', 'Kotlin', 'programming', 'Programming', 'system/manual', 'High', false],
    ['clangd', 'clangd', 'cpp', 'C / C++', 'programming', 'Programming', 'github release', 'High', true, 'Large LLVM release archive; allow several GB of download and extraction space.'],
    ['metals', 'Metals', 'scala', 'Scala', 'programming', 'Programming', 'system/manual', 'High', false],
    ['lua-language-server', 'lua-language-server', 'lua', 'Lua', 'programming', 'Programming', 'github release', 'Low'],
    ['sourcekit-lsp', 'sourcekit-lsp', 'swift', 'Swift', 'programming', 'Programming', 'system/manual', 'Toolchain-bound', false],
    ['phpactor', 'phpactor', 'php', 'PHP', 'programming', 'Programming', 'system/manual', 'Medium', false],
    ['ada-language-server', 'Ada Language Server', 'ada', 'Ada / SPARK', 'programming', 'Programming', 'github release', 'Medium'],
    ['bash-language-server', 'bash-language-server', 'bash', 'Bash', 'programming', 'Programming', 'npm', 'Low'],
    ['clojure-lsp', 'clojure-lsp', 'clojure', 'Clojure', 'programming', 'Programming', 'github release', 'Medium'],
    ['dart-sdk-lsp', 'Dart SDK language server', 'dart', 'Dart', 'programming', 'Programming', 'system/manual', 'Toolchain-bound', false],
    ['deno-lsp', 'deno lsp', 'deno', 'Deno', 'programming', 'Programming', 'github release', 'Low'],
    ['expert', 'Expert', 'elixir', 'Elixir', 'programming', 'Programming', 'github release', 'Medium'],
    ['elp', 'Erlang Language Platform', 'erlang', 'Erlang', 'programming', 'Programming', 'system/manual', 'Medium', false],
    ['fsautocomplete', 'FsAutoComplete', 'fsharp', 'F#', 'programming', 'Programming', 'dotnet tool', 'Medium'],
    ['fortls', 'fortls', 'fortran', 'Fortran', 'programming', 'Programming', 'pipx', 'Low'],
    ['haskell-language-server', 'Haskell Language Server', 'haskell', 'Haskell', 'programming', 'Programming', 'system/manual', 'Toolchain-bound', false],
    ['julia-language-server', 'LanguageServer.jl', 'julia', 'Julia', 'programming', 'Programming', 'system/manual', 'Toolchain-bound', false],
    ['nimlangserver', 'nimlangserver', 'nim', 'Nim', 'programming', 'Programming', 'github release', 'Low'],
    ['ocamllsp', 'ocamllsp', 'ocaml', 'OCaml / Reason', 'programming', 'Programming', 'system/manual', 'Medium', false],
    ['perl-navigator', 'Perl Navigator', 'perl', 'Perl', 'programming', 'Programming', 'github release', 'Medium'],
    ['powershell-editor-services', 'PowerShell Editor Services', 'powershell', 'PowerShell', 'programming', 'Programming', 'system/manual', 'Toolchain-bound', false],
    ['r-languageserver', 'R languageserver', 'r', 'R', 'programming', 'Programming', 'system/manual', 'Toolchain-bound', false],
    ['racket-langserver', 'racket-langserver', 'racket', 'Racket', 'programming', 'Programming', 'system/manual', 'Medium', false],
    ['raku-navigator', 'Raku Navigator', 'raku', 'Raku', 'programming', 'Programming', 'system/manual', 'Medium', false],
    ['rescript-language-server', 'ReScript language server', 'rescript', 'ReScript', 'programming', 'Programming', 'system/manual', 'Medium', false],
    ['v-analyzer', 'v-analyzer', 'v', 'V', 'programming', 'Programming', 'github release', 'Low'],
    ['vala-language-server', 'vala-language-server', 'vala', 'Vala', 'programming', 'Programming', 'system/manual', 'Low', false],
    ['zls', 'zls', 'zig', 'Zig', 'programming', 'Programming', 'github release', 'Low'],
    ['nil', 'nil', 'nix', 'Nix', 'programming', 'Programming', 'system/manual', 'Low', false],
    ['ballerina-language-server', 'Ballerina Language Server', 'ballerina', 'Ballerina', 'programming', 'Programming', 'system/manual', 'Toolchain-bound', false],
    ['chapel-language-server', 'Chapel Language Server', 'chapel', 'Chapel', 'programming', 'Programming', 'system/manual', 'Toolchain-bound', false],
    ['crystalline', 'Crystalline', 'crystal', 'Crystal', 'programming', 'Programming', 'system/manual', 'Low', false],
    ['serve-d', 'serve-d', 'd', 'D', 'programming', 'Programming', 'github release', 'Medium'],
    ['elm-language-server', 'elm-language-server', 'elm', 'Elm', 'programming', 'Programming', 'npm', 'Medium'],
    ['gleam-lsp', 'Gleam language server', 'gleam', 'Gleam', 'programming', 'Programming', 'github release', 'Low'],
    ['groovy-language-server', 'Groovy Language Server', 'groovy', 'Groovy', 'programming', 'Programming', 'system/manual', 'High', false],
    ['haxe-language-server', 'Haxe Language Server', 'haxe', 'Haxe', 'programming', 'Programming', 'system/manual', 'Medium', false],
    ['idris2-lsp', 'idris2-lsp', 'idris2', 'Idris2', 'programming', 'Programming', 'system/manual', 'Toolchain-bound', false],
    ['lean-language-server', 'Lean 4 language server', 'lean4', 'Lean 4', 'proof', 'Proof', 'system/manual', 'Toolchain-bound', false],
    ['coq-lsp', 'coq-lsp', 'coq', 'Coq', 'proof', 'Proof', 'system/manual', 'Toolchain-bound', false],
    ['cl-lsp', 'cl-lsp', 'common-lisp', 'Common Lisp', 'programming', 'Programming', 'system/manual', 'Medium', false],
    ['millet', 'Millet', 'standard-ml', 'Standard ML', 'programming', 'Programming', 'github release', 'Low'],
    ['html-language-server', 'vscode-html-language-server', 'html', 'HTML', 'web', 'Web', 'npm', 'Medium'],
    ['css-language-server', 'vscode-css-language-server', 'css', 'CSS / Less / Sass', 'web', 'Web', 'npm', 'Medium'],
    ['json-language-server', 'vscode-json-language-server', 'json', 'JSON', 'data', 'Data', 'npm', 'Medium'],
    ['angular-language-server', 'Angular Language Server', 'angular', 'Angular', 'framework', 'Framework', 'npm', 'Medium'],
    ['astro-language-server', 'Astro Language Server', 'astro', 'Astro', 'framework', 'Framework', 'npm', 'Medium'],
    ['svelte-language-server', 'Svelte Language Server', 'svelte', 'Svelte', 'framework', 'Framework', 'npm', 'Medium'],
    ['vue-language-server', 'Vue Language Server', 'vue', 'Vue', 'framework', 'Framework', 'npm', 'Medium'],
    ['mdx-language-server', 'MDX Language Server', 'mdx', 'MDX', 'web', 'Web', 'npm', 'Low'],
    ['tailwindcss-language-server', 'Tailwind CSS Language Server', 'tailwind-css', 'Tailwind CSS', 'framework', 'Framework', 'npm', 'Medium'],
    ['emmet-language-server', 'emmet-language-server', 'emmet', 'Emmet', 'web', 'Web', 'npm', 'Low'],
    ['graphql-language-service-cli', 'GraphQL Language Service', 'graphql', 'GraphQL', 'data', 'Data', 'npm', 'Medium'],
    ['yaml-language-server', 'yaml-language-server', 'yaml', 'YAML', 'data', 'Data', 'npm', 'Medium'],
    ['lemminx', 'LemMinX XML Language Server', 'xml', 'XML', 'data', 'Data', 'system/manual', 'Medium', false],
    ['taplo', 'Taplo', 'toml', 'TOML', 'data', 'Data', 'github release', 'Low'],
    ['docker-language-server', 'Docker Language Server', 'docker', 'Docker / Compose / Bake', 'infra', 'Infra', 'github release', 'Low'],
    ['dockerfile-language-server', 'dockerfile-language-server', 'docker', 'Docker / Compose / Bake', 'infra', 'Infra', 'npm', 'Low'],
    ['terraform-ls', 'terraform-ls', 'terraform', 'Terraform', 'infra', 'Infra', 'system/manual', 'Low', false],
    ['cue-lsp', 'CUE language server', 'cue', 'CUE', 'config', 'Config', 'github release', 'Low'],
    ['jsonnet-language-server', 'jsonnet-language-server', 'jsonnet', 'Jsonnet', 'config', 'Config', 'go install', 'Low'],
    ['kcl-language-server', 'KCL Language Server', 'kcl', 'KCL', 'config', 'Config', 'github release', 'Medium'],
    ['bicep-language-server', 'Bicep Language Server', 'bicep', 'Bicep', 'infra', 'Infra', 'system/manual', 'Medium', false],
    ['ansible-language-server', 'ansible-language-server', 'ansible', 'Ansible', 'infra', 'Infra', 'system/manual', 'High', false],
    ['helm-ls', 'helm-ls', 'helm', 'Helm', 'infra', 'Infra', 'github release', 'Low'],
    ['neocmakelsp', 'neocmakelsp', 'cmake', 'CMake', 'build', 'Build', 'github release', 'Low'],
    ['mesonlsp', 'mesonlsp', 'meson', 'Meson', 'build', 'Build', 'github release', 'Low'],
    ['just-lsp', 'just-lsp', 'just', 'Just', 'build', 'Build', 'github release', 'Low'],
    ['make-lsp', 'make-lsp', 'make', 'Make', 'build', 'Build', 'system/manual', 'Low', false],
    ['nginx-language-server', 'nginx-language-server', 'nginx', 'Nginx', 'infra', 'Infra', 'pipx', 'Low'],
    ['systemd-language-server', 'systemd-language-server', 'systemd', 'systemd', 'infra', 'Infra', 'pipx', 'Low'],
    ['github-actions-language-server', 'GitHub Actions Language Server', 'github-actions', 'GitHub Actions', 'infra', 'Infra', 'system/manual', 'Low', false],
    ['gitlab-ci-ls', 'gitlab-ci-ls', 'gitlab-ci', 'GitLab CI', 'infra', 'Infra', 'system/manual', 'Low', false],
    ['buf-language-server', 'Buf Language Server', 'protobuf', 'Protocol Buffers', 'data', 'Data', 'github release', 'Low'],
    ['thrift-ls', 'thrift-ls', 'thrift', 'Thrift', 'data', 'Data', 'go install', 'Low'],
    ['sqls', 'sqls', 'sql', 'SQL', 'data', 'Data', 'go install', 'Low'],
    ['postgres-language-server', 'Postgres Language Server', 'postgres-sql', 'Postgres SQL', 'data', 'Data', 'github release', 'Low'],
    ['promql-langserver', 'promql-langserver', 'promql', 'PromQL', 'domain-specific', 'Domain-specific', 'go install', 'Low'],
    ['openapi-yaml-language-server', 'yaml-language-server (OpenAPI schemas)', 'openapi', 'OpenAPI', 'data', 'Data', 'npm', 'Medium'],
    ['glsl-analyzer', 'glsl_analyzer', 'glsl', 'GLSL', 'shader', 'Shader', 'github release', 'Low'],
    ['wgsl-analyzer', 'wgsl-analyzer', 'wgsl', 'WGSL', 'shader', 'Shader', 'github release', 'Low'],
    ['hlsl-tools', 'HLSL Tools', 'hlsl', 'HLSL', 'shader', 'Shader', 'system/manual', 'Medium', false],
    ['qmlls', 'qmlls', 'qml', 'QML', 'framework', 'Framework', 'system/manual', 'Toolchain-bound', false],
    ['opencl-language-server', 'opencl-language-server', 'opencl', 'OpenCL', 'shader', 'Shader', 'github release', 'Medium'],
    ['verible', 'Verible Verilog Language Server', 'systemverilog', 'SystemVerilog', 'hardware', 'Hardware', 'github release', 'Medium'],
    ['vhdl-ls', 'vhdl_ls', 'vhdl', 'VHDL', 'hardware', 'Hardware', 'github release', 'Low'],
    ['veryl-ls', 'Veryl Language Server', 'veryl', 'Veryl', 'hardware', 'Hardware', 'github release', 'Low'],
    ['dot-language-server', 'dot-language-server', 'dot', 'Graphviz DOT', 'domain-specific', 'Domain-specific', 'npm', 'Low'],
    ['marksman', 'Marksman', 'markdown', 'Markdown', 'data', 'Data', 'github release', 'Low'],
    ['texlab', 'texlab', 'latex', 'LaTeX', 'domain-specific', 'Domain-specific', 'github release', 'Low'],
    ['tinymist', 'tinymist', 'typst', 'Typst', 'domain-specific', 'Domain-specific', 'github release', 'Low'],
    ['robotcode', 'RobotCode', 'robot-framework', 'Robot Framework', 'domain-specific', 'Domain-specific', 'system/manual', 'Medium', false],
    ['cucumber-language-server', 'Cucumber Language Server', 'gherkin', 'Gherkin / Cucumber', 'domain-specific', 'Domain-specific', 'npm', 'Low'],
    ['regal', 'Regal', 'rego', 'Rego', 'domain-specific', 'Domain-specific', 'github release', 'Low'],
    ['puppet-editor-services', 'Puppet Editor Services', 'puppet', 'Puppet', 'infra', 'Infra', 'system/manual', 'Medium', false],
  ];

  return rows.map(([
    id,
    name,
    language,
    languageDisplay,
    languageCategory,
    languageCategoryDisplay,
    installMethod,
    footprint,
    canInstall = installMethod !== 'system/manual',
    installWarning = null,
  ]) => mockServer(
    id,
    name,
    language,
    languageDisplay,
    languageCategory,
    languageCategoryDisplay,
    installMethod,
    false,
    'missing',
    footprint,
    canInstall,
    installWarning,
  ));
}

function mockServer(
  id: string,
  name: string,
  language: string,
  languageDisplay: string,
  languageCategory: string,
  languageCategoryDisplay: string,
  installMethod: string,
  installed: boolean,
  installState: ServerStatus['installState'],
  footprint: string,
  canInstall = true,
  installWarning: string | null = null,
): ServerStatus {
  return {
    id,
    name,
    language,
    languageDisplay,
    languageCategory,
    languageCategoryDisplay,
    version: 'latest',
    binaryName: name,
    installMethod,
    installed,
    installState,
    installedPath: installed ? `C:\\Users\\you\\AppData\\Local\\lsp-io\\${name}` : null,
    canInstall,
    canRemove: installState === 'managed',
    footprint,
    maturity: 'Stable',
    sourceUrl: 'https://langserver.org/',
    rationale: 'Mock development row for browser preview.',
    manualInstructions: canInstall ? null : 'Install with the upstream project instructions and put the binary on PATH.',
    installWarning,
  };
}

function isTauriRuntime(): boolean {
  return '__TAURI_INTERNALS__' in window;
}

function escapeHtml(str: string): string {
  const div = document.createElement('div');
  div.textContent = str;
  return div.innerHTML;
}
