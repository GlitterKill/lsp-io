import { invoke } from '@tauri-apps/api/core';
import { listen } from '@tauri-apps/api/event';

export interface LanguageInfo {
  name: string;
  display_name: string;
  kind: string;
  category: string;
  category_display: string;
  confidence: string;
  evidence: string;
}

export interface ServerStatusInfo {
  id: string;
  name: string;
  language: string;
  language_display: string;
  language_category: string;
  language_category_display: string;
  version: string;
  binary_name: string;
  install_method: string;
  installed: boolean;
  install_state: 'managed' | 'system' | 'missing';
  installed_path: string | null;
  can_install: boolean;
  can_remove: boolean;
  footprint: string;
  maturity: string;
  source_url: string;
  rationale: string;
  manual_instructions: string | null;
  install_warning: string | null;
}

export interface InstallOutcomeInfo {
  id: string;
  name: string;
  path: string | null;
  status: 'installed' | 'already_installed' | 'removed' | 'failed';
  message: string;
}

export interface UpdateInfo {
  id: string;
  name: string;
  language: string;
  current_version: string;
  latest_version: string;
  update_available: boolean;
}

export interface SdlMcpExportInfo {
  fragment_json: string;
  server_count: number;
  skipped: string[];
}

export interface ProgressEvent {
  [key: string]: unknown;
}

export async function detectLanguages(path: string): Promise<LanguageInfo[]> {
  return invoke('detect_languages', { path });
}

export async function getServerStatus(path: string): Promise<ServerStatusInfo[]> {
  return invoke('get_server_status', { path });
}

export async function installServers(serverIds: string[], path: string): Promise<InstallOutcomeInfo[]> {
  return invoke('install_servers', { serverIds, path });
}

export async function installOneServer(serverId: string, path: string): Promise<InstallOutcomeInfo> {
  return invoke('install_one_server', { serverId, path });
}

export async function removeOneServer(serverId: string, path: string): Promise<InstallOutcomeInfo> {
  return invoke('remove_one_server', { serverId, path });
}

export async function getConfig(path: string): Promise<unknown> {
  return invoke('get_config', { path });
}

export async function saveConfig(path: string, config: unknown): Promise<void> {
  return invoke('save_config', { path, config });
}

export async function cleanCache(path: string, serverId?: string): Promise<string> {
  return invoke('clean_cache', { serverId: serverId || null, path });
}

export async function checkUpdates(): Promise<UpdateInfo[]> {
  return invoke('check_updates');
}

export async function exportSdlMcpConfig(
  path: string,
  includeMissing = false,
  validateLaunch = false,
): Promise<SdlMcpExportInfo> {
  return invoke('export_sdl_mcp_config', { path, includeMissing, validateLaunch });
}

export async function writeSdlMcpConfig(
  path: string,
  configPath: string,
  includeMissing = false,
  validateLaunch = false,
  enableSemanticEnrichment = false,
): Promise<SdlMcpExportInfo> {
  return invoke('write_sdl_mcp_config', {
    path,
    configPath,
    includeMissing,
    validateLaunch,
    enableSemanticEnrichment,
  });
}

export function onProgress(callback: (event: ProgressEvent) => void): Promise<() => void> {
  return listen('progress', (event) => {
    callback(event.payload as ProgressEvent);
  }).then(unlisten => unlisten);
}
