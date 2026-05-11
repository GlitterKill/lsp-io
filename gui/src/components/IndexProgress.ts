import { store, addLog } from '../state/store.js';
import type { ServerProgress, InstallOutcome, LogEntry, AppState } from '../state/store.js';
import { onProgress } from '../bridge/tauri.js';
import { renderProgressBar } from './ProgressBar.js';
import { renderLogViewer } from './LogViewer.js';

const PIPELINE_STEPS = ['resolve', 'install', 'verify', 'done'] as const;
const PIPELINE_LABELS: Record<string, string> = {
  resolve: 'Resolve Servers',
  install: 'Install Binaries',
  verify: 'Verify Status',
  done: 'Complete',
};

let progressUnlisten: (() => void) | null = null;

export function renderInstallProgress(container: HTMLElement): void {
  const wrapper = document.createElement('div');
  wrapper.className = 'content-wrapper section';
  container.appendChild(wrapper);

  const header = document.createElement('div');
  header.className = 'flex items-center justify-between mb-lg';
  header.innerHTML = `
    <div>
      <h2 class="text-xl">Installing Language Servers</h2>
      <p class="text-sm text-secondary mt-xs">Managing app-owned server binaries...</p>
    </div>
  `;
  wrapper.appendChild(header);

  const overallPanel = document.createElement('div');
  overallPanel.className = 'panel animate-fade-in';
  overallPanel.id = 'overall-progress-panel';
  wrapper.appendChild(overallPanel);
  renderOverallProgress(overallPanel);

  const pipelinePanel = document.createElement('div');
  pipelinePanel.className = 'panel animate-fade-in stagger-2';
  pipelinePanel.id = 'pipeline-panel';
  wrapper.appendChild(pipelinePanel);
  renderPipelineSteps(pipelinePanel);

  const serverProgressContainer = document.createElement('div');
  serverProgressContainer.className = 'grid grid-auto-md gap-md';
  serverProgressContainer.id = 'server-progress-container';
  wrapper.appendChild(serverProgressContainer);
  renderServerCards(serverProgressContainer);

  const logSection = document.createElement('div');
  logSection.className = 'animate-fade-in stagger-4';
  logSection.id = 'install-log-section';
  wrapper.appendChild(logSection);
  const logUnsub = renderLogViewer(logSection);

  const unsub = store.subscribe((state) => {
    const op = document.getElementById('overall-progress-panel');
    if (op) {
      op.innerHTML = '';
      renderOverallProgress(op);
    }

    const pp = document.getElementById('pipeline-panel');
    if (pp) {
      pp.innerHTML = '';
      renderPipelineSteps(pp);
    }

    const sp = document.getElementById('server-progress-container');
    if (sp) {
      sp.innerHTML = '';
      renderServerCards(sp);
    }

    if (!state.isInstalling && state.pipelineStep === 'done' && state.results) {
      store.setState({ screen: 'results' });
    }
  });

  setupProgressListener();

  const screenUnsub = store.subscribe((state) => {
    if (state.screen !== 'installing') {
      unsub();
      logUnsub();
      screenUnsub();
      teardownProgressListener();
    }
  });
}

function renderOverallProgress(container: HTMLElement): void {
  const state = store.getState();

  const sectionHeader = document.createElement('div');
  sectionHeader.className = 'section-header';
  sectionHeader.innerHTML = `
    <span class="section-header__title">Overall Progress</span>
    <div class="section-header__line"></div>
  `;
  container.appendChild(sectionHeader);

  const bar = renderProgressBar({
    label: `Step: ${PIPELINE_LABELS[state.pipelineStep] || state.pipelineStep}`,
    value: state.overallProgress,
    size: 'lg',
    indeterminate: state.isInstalling && state.overallProgress === 0,
  });
  container.appendChild(bar);
}

function renderPipelineSteps(container: HTMLElement): void {
  const state = store.getState();
  const currentIdx = PIPELINE_STEPS.indexOf(state.pipelineStep);

  const sectionHeader = document.createElement('div');
  sectionHeader.className = 'section-header';
  sectionHeader.innerHTML = `
    <span class="section-header__title">Pipeline</span>
    <div class="section-header__line"></div>
  `;
  container.appendChild(sectionHeader);

  const stepsRow = document.createElement('div');
  stepsRow.className = 'flex items-center gap-md';

  PIPELINE_STEPS.forEach((step, idx) => {
    const stepEl = document.createElement('div');
    stepEl.className = 'flex items-center gap-xs';

    const indicator = document.createElement('div');
    const allDone = state.pipelineStep === 'done';
    if (idx < currentIdx || allDone) {
      indicator.className = 'flex items-center justify-center rounded-full';
      indicator.style.width = '24px';
      indicator.style.height = '24px';
      indicator.style.background = 'var(--color-success-dim)';
      indicator.style.border = '1px solid var(--color-success)';
      indicator.innerHTML = `<svg width="12" height="12" viewBox="0 0 12 12" fill="none"><path d="M2 6l3 3 5-5" stroke="var(--color-success)" stroke-width="1.5" stroke-linecap="round" stroke-linejoin="round"/></svg>`;
    } else if (idx === currentIdx) {
      indicator.className = 'flex items-center justify-center rounded-full animate-glow';
      indicator.style.width = '24px';
      indicator.style.height = '24px';
      indicator.style.background = 'var(--accent-cyan-dim)';
      indicator.style.border = '1px solid var(--accent-cyan)';
      indicator.innerHTML = `<span class="spinner spinner--sm"></span>`;
    } else {
      indicator.className = 'flex items-center justify-center rounded-full';
      indicator.style.width = '24px';
      indicator.style.height = '24px';
      indicator.style.background = 'var(--bg-surface)';
      indicator.style.border = '1px solid var(--border-default)';
    }

    stepEl.appendChild(indicator);

    const label = document.createElement('span');
    label.className = idx === currentIdx ? 'text-sm text-cyan font-medium' : 'text-sm text-muted';
    label.textContent = PIPELINE_LABELS[step];
    stepEl.appendChild(label);

    stepsRow.appendChild(stepEl);

    if (idx < PIPELINE_STEPS.length - 1) {
      const line = document.createElement('div');
      line.className = 'flex-1';
      line.style.height = '1px';
      line.style.background = idx < currentIdx ? 'var(--color-success)' : 'var(--border-default)';
      line.style.minWidth = '20px';
      stepsRow.appendChild(line);
    }
  });

  container.appendChild(stepsRow);
}

function renderServerCards(container: HTMLElement): void {
  const entries: ServerProgress[] = Array.from(store.getState().serverProgress.values());
  for (const entry of entries) {
    container.appendChild(createServerCard(entry));
  }
}

function createServerCard(entry: ServerProgress): HTMLElement {
  const card = document.createElement('div');
  const isActive = entry.status === 'installing';
  const isDone = entry.status === 'done';
  const isFailed = entry.status === 'failed';

  card.className = `panel panel--compact animate-fade-in${isActive ? ' panel--active' : ''}`;

  const headerRow = document.createElement('div');
  headerRow.className = 'flex items-center justify-between mb-sm';

  const serverName = document.createElement('span');
  serverName.className = 'font-medium text-primary';
  serverName.textContent = entry.server;
  headerRow.appendChild(serverName);

  const statusBadge = document.createElement('span');
  statusBadge.className = `badge badge--${getStatusBadgeClass(entry.status)}`;
  statusBadge.innerHTML = `<span class="badge__dot"></span> ${formatStatus(entry.status)}`;
  headerRow.appendChild(statusBadge);

  card.appendChild(headerRow);

  const bar = renderProgressBar({
    value: entry.progress,
    size: 'sm',
    showValue: entry.status !== 'queued',
    indeterminate: entry.status === 'installing' && entry.progress < 100,
  });
  card.appendChild(bar);

  const msg = document.createElement('div');
  msg.className = `text-xs mt-sm ${isFailed ? 'text-error' : isDone ? 'text-success' : 'text-muted'}`;
  msg.textContent = entry.message;
  card.appendChild(msg);

  return card;
}

function getStatusBadgeClass(status: ServerProgress['status']): string {
  switch (status) {
    case 'queued': return 'not-installed';
    case 'installing': return 'running';
    case 'done': return 'installed';
    case 'failed': return 'error';
    default: return 'not-installed';
  }
}

function formatStatus(status: ServerProgress['status']): string {
  switch (status) {
    case 'queued': return 'Queued';
    case 'installing': return 'Installing';
    case 'done': return 'Done';
    case 'failed': return 'Failed';
    default: return status;
  }
}

async function setupProgressListener() {
  try {
    progressUnlisten = await onProgress((event) => {
      handleProgressEvent(event);
    });
  } catch {
    console.log('[dev] Progress listener not available outside Tauri');
    simulateProgress();
  }
}

function teardownProgressListener() {
  if (progressUnlisten) {
    progressUnlisten();
    progressUnlisten = null;
  }
}

function handleProgressEvent(event: Record<string, unknown>) {
  const kind = event.kind as string | undefined;
  const server = event.server as string | undefined;

  if (kind === 'pipeline_step') {
    const step = event.step as string;
    const progress = event.progress as number;
    store.setState({
      pipelineStep: step as AppState['pipelineStep'],
      overallProgress: progress,
    });
    addLog('info', `Pipeline step: ${PIPELINE_LABELS[step] || step}`);
  }

  if (kind === 'server_progress' && server) {
    const map = new Map(store.getState().serverProgress);
    map.set(server, {
      server,
      status: event.status as ServerProgress['status'],
      progress: (event.progress as number) || 0,
      message: (event.message as string) || '',
    });
    store.setState({
      serverProgress: map,
      pipelineStep: (event.status === 'done' ? 'verify' : 'install') as AppState['pipelineStep'],
      overallProgress: Math.max(store.getState().overallProgress, (event.progress as number) || 0),
    });
  }

  if (kind === 'install_complete') {
    const outcomes = ((event.outcomes as InstallOutcome[]) || []).map((outcome) => ({
      ...outcome,
      path: outcome.path || null,
    }));
    store.setState({
      isInstalling: false,
      pipelineStep: 'done',
      overallProgress: 100,
      results: {
        outcomes,
        totalDuration: 0,
      },
    });
    addLog('success', 'Install operation complete');
  }

  if (kind === 'log') {
    const level = (event.level as string) || 'info';
    const message = (event.message as string) || '';
    addLog(level as LogEntry['level'], message);
  }
}

function simulateProgress() {
  const state = store.getState();
  if (!state.isInstalling) return;

  const servers = Array.from(state.serverProgress.keys());
  let step = 0;
  const steps: Array<[AppState['pipelineStep'], number]> = [
    ['resolve', 15],
    ['install', 60],
    ['verify', 90],
    ['done', 100],
  ];

  const interval = setInterval(() => {
    if (!store.getState().isInstalling || step >= steps.length) {
      clearInterval(interval);
      return;
    }

    const [stepName, progress] = steps[step];
    store.setState({ pipelineStep: stepName, overallProgress: progress });
    addLog('info', `[sim] Pipeline: ${PIPELINE_LABELS[stepName]}`);

    const map = new Map(store.getState().serverProgress);
    servers.forEach((server) => {
      map.set(server, {
        server,
        status: progress >= 100 ? 'done' : progress > 15 ? 'installing' : 'queued',
        progress,
        message: progress >= 100 ? 'Verified' : progress > 15 ? 'Installing package...' : 'Waiting...',
      });
    });
    store.setState({ serverProgress: map });

    step++;
    if (stepName === 'done') {
      clearInterval(interval);
      store.setState({
        isInstalling: false,
        results: {
          outcomes: servers.map((server) => ({
            id: server,
            name: server,
            path: `C:\\Users\\you\\AppData\\Local\\lsp-io\\${server}`,
            status: 'installed',
            message: 'Installed in browser simulation',
          })),
          totalDuration: 3200,
        },
      });
      addLog('success', '[sim] Install complete');
    }
  }, 1000);
}
