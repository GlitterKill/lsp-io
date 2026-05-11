type Listener<T> = (state: T) => void;

export class Store<T> {
  private state: T;
  private listeners: Set<Listener<T>> = new Set();

  constructor(initial: T) {
    this.state = initial;
  }

  getState(): T {
    return this.state;
  }

  setState(partial: Partial<T>) {
    this.state = { ...this.state, ...partial };
    this.listeners.forEach((fn) => fn(this.state));
  }

  subscribe(fn: Listener<T>): () => void {
    this.listeners.add(fn);
    return () => this.listeners.delete(fn);
  }
}

export type Screen = 'dashboard' | 'installing' | 'results' | 'settings';
export type ServerSortKey = 'server' | 'language' | 'detected' | 'category' | 'install' | 'footprint' | 'status' | 'action';
export type SortDirection = 'asc' | 'desc';

export interface LogEntry {
  timestamp: string;
  level: 'info' | 'success' | 'error' | 'warning';
  message: string;
}

export interface LanguageSelection {
  name: string;
  displayName: string;
  category: string;
  categoryDisplay: string;
  confidence: string;
  evidence: string;
  selected: boolean;
}

export interface ServerStatus {
  id: string;
  name: string;
  language: string;
  languageDisplay: string;
  languageCategory: string;
  languageCategoryDisplay: string;
  version: string;
  binaryName: string;
  installMethod: string;
  installed: boolean;
  installState: 'managed' | 'system' | 'missing';
  installedPath: string | null;
  canInstall: boolean;
  canRemove: boolean;
  footprint: string;
  maturity: string;
  sourceUrl: string;
  rationale: string;
  manualInstructions: string | null;
  installWarning: string | null;
}

export interface ServerProgress {
  server: string;
  status: 'queued' | 'installing' | 'done' | 'failed';
  progress: number;
  message: string;
}

export interface InstallOutcome {
  id: string;
  name: string;
  path: string | null;
  status: 'installed' | 'already_installed' | 'removed' | 'failed';
  message: string;
}

export interface InstallResult {
  outcomes: InstallOutcome[];
  totalDuration: number;
}

export interface AppState {
  screen: Screen;
  projectPath: string;
  languages: LanguageSelection[];
  servers: ServerStatus[];
  serverCategoryFilter: string;
  serverSort: {
    key: ServerSortKey;
    direction: SortDirection;
  };
  isInstalling: boolean;
  serverProgress: Map<string, ServerProgress>;
  pipelineStep: 'resolve' | 'install' | 'verify' | 'done';
  overallProgress: number;
  logs: LogEntry[];
  results: InstallResult | null;
  settings: {
    preferPath: boolean;
    timeout: number;
    cacheDir: string;
  };
}

export const store = new Store<AppState>({
  screen: 'dashboard',
  projectPath: '.',
  languages: [],
  servers: [],
  serverCategoryFilter: 'all',
  serverSort: {
    key: 'language',
    direction: 'asc',
  },
  isInstalling: false,
  serverProgress: new Map(),
  pipelineStep: 'resolve',
  overallProgress: 0,
  logs: [],
  results: null,
  settings: {
    preferPath: true,
    timeout: 300,
    cacheDir: '',
  },
});

export function addLog(level: LogEntry['level'], message: string) {
  const state = store.getState();
  const entry: LogEntry = {
    timestamp: new Date().toLocaleTimeString(),
    level,
    message,
  };
  store.setState({ logs: [...state.logs, entry] });
}
