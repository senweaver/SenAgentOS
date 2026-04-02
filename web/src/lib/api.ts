import type {
  StatusResponse,
  ToolSpec,
  CronJob,
  CronRun,
  Integration,
  DiagResult,
  MemoryEntry,
  CostSummary,
  CliTool,
  HealthSnapshot,
  Session,
  SessionMessagesResponse,
  HardwareBoardsResponse,
  HardwareContextResponse,
} from '../types/api';
import { clearToken, getToken, setToken } from './auth';
import { apiOrigin, basePath } from './basePath';

// ---------------------------------------------------------------------------
// Base fetch wrapper
// ---------------------------------------------------------------------------

export class UnauthorizedError extends Error {
  constructor() {
    super('Unauthorized');
    this.name = 'UnauthorizedError';
  }
}

export async function apiFetch<T = unknown>(
  path: string,
  options: RequestInit = {},
): Promise<T> {
  const token = getToken();
  const headers = new Headers(options.headers);

  if (token) {
    headers.set('Authorization', `Bearer ${token}`);
  }

  if (
    options.body &&
    typeof options.body === 'string' &&
    !headers.has('Content-Type')
  ) {
    headers.set('Content-Type', 'application/json');
  }

  const url = apiOrigin ? `${apiOrigin}${basePath}${path}` : `${basePath}${path}`;
  console.log('[API]', options.method || 'GET', url);
  
  const response = await fetch(url, { ...options, headers });

  if (response.status === 401) {
    clearToken();
    window.dispatchEvent(new Event('senagent-unauthorized'));
    throw new UnauthorizedError();
  }

  if (!response.ok) {
    const text = await response.text().catch(() => '');
    throw new Error(`API ${response.status}: ${text || response.statusText}`);
  }

  // Some endpoints may return 204 No Content
  if (response.status === 204) {
    return undefined as unknown as T;
  }

  return response.json() as Promise<T>;
}

function unwrapField<T>(value: T | Record<string, T>, key: string): T {
  if (value !== null && typeof value === 'object' && !Array.isArray(value) && key in value) {
    const unwrapped = (value as Record<string, T | undefined>)[key];
    if (unwrapped !== undefined) {
      return unwrapped;
    }
  }
  return value as T;
}

// ---------------------------------------------------------------------------
// Pairing
// ---------------------------------------------------------------------------

export async function pair(code: string): Promise<{ token: string }> {
  const url = apiOrigin ? `${apiOrigin}${basePath}/pair` : `${basePath}/pair`;
  const response = await fetch(url, {
    method: 'POST',
    headers: { 'X-Pairing-Code': code },
  });

  if (!response.ok) {
    const text = await response.text().catch(() => '');
    throw new Error(`Pairing failed (${response.status}): ${text || response.statusText}`);
  }

  const data = (await response.json()) as { token: string };
  setToken(data.token);
  return data;
}

export async function getAdminPairCode(): Promise<{ pairing_code: string | null; pairing_required: boolean }> {
  // Try the public /pair/code endpoint first
  const publicUrl = apiOrigin ? `${apiOrigin}${basePath}/pair/code` : `${basePath}/pair/code`;
  const publicResp = await fetch(publicUrl);
  if (publicResp.ok) {
    return publicResp.json() as Promise<{ pairing_code: string | null; pairing_required: boolean }>;
  }

  // Fallback to admin endpoint
  const adminUrl = apiOrigin ? `${apiOrigin}/admin/paircode` : '/admin/paircode';
  const response = await fetch(adminUrl);
  if (!response.ok) {
    throw new Error(`Failed to fetch pairing code (${response.status})`);
  }
  return response.json() as Promise<{ pairing_code: string | null; pairing_required: boolean }>;
}

// ---------------------------------------------------------------------------
// Public health (no auth required)
// ---------------------------------------------------------------------------

export async function getPublicHealth(): Promise<{ require_pairing: boolean; paired: boolean }> {
  const url = apiOrigin ? `${apiOrigin}${basePath}/health` : `${basePath}/health`;
  const response = await fetch(url);
  if (!response.ok) {
    throw new Error(`Health check failed (${response.status})`);
  }
  return response.json() as Promise<{ require_pairing: boolean; paired: boolean }>;
}

// ---------------------------------------------------------------------------
// Status / Health
// ---------------------------------------------------------------------------

export function getStatus(): Promise<StatusResponse> {
  return apiFetch<StatusResponse>('/api/status');
}

export function getHealth(): Promise<HealthSnapshot> {
  return apiFetch<HealthSnapshot | { health: HealthSnapshot }>('/api/health').then((data) =>
    unwrapField(data, 'health'),
  );
}

// ---------------------------------------------------------------------------
// Config
// ---------------------------------------------------------------------------

export function getConfig(): Promise<string> {
  return apiFetch<string | { format?: string; content: string }>('/api/config').then((data) =>
    typeof data === 'string' ? data : data.content,
  );
}

export function putConfig(toml: string): Promise<void> {
  return apiFetch<void>('/api/config', {
    method: 'PUT',
    headers: { 'Content-Type': 'application/toml' },
    body: toml,
  });
}

// ---------------------------------------------------------------------------
// Provider / Model
// ---------------------------------------------------------------------------

export interface ProviderConfig {
  provider: string | null;
  model: string | null;
  api_key: string | null; // masked (***...) or null
  api_url: string | null;
  gateway_port: number;
  gateway_host: string;
  gateway_require_pairing: boolean;
}

export function getProvider(): Promise<ProviderConfig> {
  return apiFetch<ProviderConfig>('/api/provider');
}

export function putProvider(body: {
  provider?: string;
  model?: string;
  api_key?: string;
  api_url?: string;
  gateway_port?: number;
  gateway_host?: string;
  gateway_require_pairing?: boolean;
}): Promise<void> {
  return apiFetch<void>('/api/provider', {
    method: 'PUT',
    body: JSON.stringify(body),
  });
}

export interface ChannelEntry {
  name: string;
  enabled: boolean;
  type?: string;
  status?: 'active' | 'inactive' | 'error';
  health?: 'healthy' | 'degraded' | 'down';
  message_count?: number;
  last_message_at?: string | null;
  config: Record<string, unknown>;
}

export function getChannels(): Promise<{ channels: ChannelEntry[] }> {
  return apiFetch<{ channels: ChannelEntry[] }>('/api/channels');
}

export function putChannels(body: { channels: ChannelEntry[] }): Promise<void> {
  return apiFetch<void>('/api/channels', {
    method: 'PUT',
    body: JSON.stringify(body),
  });
}

// ---------------------------------------------------------------------------
// Tools
// ---------------------------------------------------------------------------

export function getTools(): Promise<ToolSpec[]> {
  return apiFetch<ToolSpec[] | { tools: ToolSpec[] }>('/api/tools').then((data) =>
    unwrapField(data, 'tools'),
  );
}

// ---------------------------------------------------------------------------
// Skills (workspace + open-skills; `[skills].disabled_skills`)
// ---------------------------------------------------------------------------

export interface SkillRow {
  name: string;
  description: string;
  version: string;
  author?: string | null;
  tags: string[];
  tools_count: number;
  prompts_count: number;
  enabled: boolean;
  path?: string | null;
}

export interface SkillsResponse {
  workspace_skills_dir: string;
  open_skills_enabled: boolean;
  allow_scripts: boolean;
  disabled_skills: string[];
  skills: SkillRow[];
}

export function getSkills(): Promise<SkillsResponse> {
  return apiFetch<SkillsResponse>('/api/skills');
}

export function putSkills(body: { disabled_skills: string[] }): Promise<void> {
  return apiFetch<void>('/api/skills', {
    method: 'PUT',
    body: JSON.stringify(body),
  });
}

// ---------------------------------------------------------------------------
// Cron
// ---------------------------------------------------------------------------

export function getCronJobs(): Promise<CronJob[]> {
  return apiFetch<CronJob[] | { jobs: CronJob[] }>('/api/cron').then((data) =>
    unwrapField(data, 'jobs'),
  );
}

export function addCronJob(body: {
  name?: string;
  command: string;
  schedule: string;
  enabled?: boolean;
}): Promise<CronJob> {
  return apiFetch<CronJob | { status: string; job: CronJob }>('/api/cron', {
    method: 'POST',
    body: JSON.stringify(body),
  }).then((data) => (typeof (data as { job?: CronJob }).job === 'object' ? (data as { job: CronJob }).job : (data as CronJob)));
}

export function deleteCronJob(id: string): Promise<void> {
  return apiFetch<void>(`/api/cron/${encodeURIComponent(id)}`, {
    method: 'DELETE',
  });
}
export function patchCronJob(
  id: string,
  patch: { name?: string; schedule?: string; command?: string },
): Promise<CronJob> {
  return apiFetch<CronJob | { status: string; job: CronJob }>(
    `/api/cron/${encodeURIComponent(id)}`,
    {
      method: 'PATCH',
      body: JSON.stringify(patch),
    },
  ).then((data) => (typeof (data as { job?: CronJob }).job === 'object' ? (data as { job: CronJob }).job : (data as CronJob)));
}


export function getCronRuns(
  jobId: string,
  limit: number = 20,
): Promise<CronRun[]> {
  const params = new URLSearchParams({ limit: String(limit) });
  return apiFetch<CronRun[] | { runs: CronRun[] }>(
    `/api/cron/${encodeURIComponent(jobId)}/runs?${params}`,
  ).then((data) => unwrapField(data, 'runs'));
}

export interface CronSettings {
  enabled: boolean;
  catch_up_on_startup: boolean;
  max_run_history: number;
}

export function getCronSettings(): Promise<CronSettings> {
  return apiFetch<CronSettings>('/api/cron/settings');
}

export function patchCronSettings(
  patch: Partial<CronSettings>,
): Promise<CronSettings> {
  return apiFetch<CronSettings & { status: string }>('/api/cron/settings', {
    method: 'PATCH',
    body: JSON.stringify(patch),
  });
}

// ---------------------------------------------------------------------------
// Integrations
// ---------------------------------------------------------------------------

export function getIntegrations(): Promise<Integration[]> {
  return apiFetch<Integration[] | { integrations: Integration[] }>('/api/integrations').then(
    (data) => unwrapField(data, 'integrations'),
  );
}

// ---------------------------------------------------------------------------
// Doctor / Diagnostics
// ---------------------------------------------------------------------------

export function runDoctor(): Promise<DiagResult[]> {
  return apiFetch<DiagResult[] | { results: DiagResult[]; summary?: unknown }>('/api/doctor', {
    method: 'POST',
    body: JSON.stringify({}),
  }).then((data) => (Array.isArray(data) ? data : data.results));
}

// ---------------------------------------------------------------------------
// Memory
// ---------------------------------------------------------------------------

export function getMemory(
  query?: string,
  category?: string,
): Promise<MemoryEntry[]> {
  const params = new URLSearchParams();
  if (query) params.set('query', query);
  if (category) params.set('category', category);
  const qs = params.toString();
  return apiFetch<MemoryEntry[] | { entries: MemoryEntry[] }>(`/api/memory${qs ? `?${qs}` : ''}`).then(
    (data) => unwrapField(data, 'entries'),
  );
}

export function storeMemory(
  key: string,
  content: string,
  category?: string,
): Promise<void> {
  return apiFetch<unknown>('/api/memory', {
    method: 'POST',
    body: JSON.stringify({ key, content, category }),
  }).then(() => undefined);
}

export function deleteMemory(key: string): Promise<void> {
  return apiFetch<void>(`/api/memory/${encodeURIComponent(key)}`, {
    method: 'DELETE',
  });
}

// ---------------------------------------------------------------------------
// Cost
// ---------------------------------------------------------------------------

export function getCost(): Promise<CostSummary> {
  return apiFetch<CostSummary | { cost: CostSummary }>('/api/cost').then((data) =>
    unwrapField(data, 'cost'),
  );
}

// ---------------------------------------------------------------------------
// Sessions
// ---------------------------------------------------------------------------

export function getSessions(): Promise<Session[]> {
  return apiFetch<Session[] | { sessions: Session[] }>('/api/sessions').then((data) =>
    unwrapField(data, 'sessions'),
  );
}

export function getSession(id: string): Promise<Session> {
  return apiFetch<Session>(`/api/sessions/${encodeURIComponent(id)}`);
}

/** Load persisted gateway WebSocket chat transcript for the dashboard Agent Chat. */
export function getSessionMessages(id: string): Promise<SessionMessagesResponse> {
  return apiFetch<SessionMessagesResponse>(
    `/api/sessions/${encodeURIComponent(id)}/messages`,
  );
}

// ---------------------------------------------------------------------------
// CLI Tools
// ---------------------------------------------------------------------------

export function getCliTools(): Promise<CliTool[]> {
  return apiFetch<CliTool[] | { cli_tools: CliTool[] }>('/api/cli-tools').then((data) =>
    unwrapField(data, 'cli_tools'),
  );
}

// ---------------------------------------------------------------------------
// Hardware
// ---------------------------------------------------------------------------

export function getHardwareBoards(): Promise<HardwareBoardsResponse> {
  return apiFetch<HardwareBoardsResponse>('/api/hardware/boards');
}

export function getHardwareContext(): Promise<HardwareContextResponse> {
  return apiFetch<HardwareContextResponse>('/api/hardware/context');
}

export function registerGpioPin(body: {
  device?: string;
  pin: number;
  component: string;
  notes?: string;
}): Promise<{ ok: boolean; message: string }> {
  return apiFetch<{ ok: boolean; message: string }>('/api/hardware/pin', {
    method: 'POST',
    body: JSON.stringify(body),
  });
}

export function reloadHardwareContext(): Promise<{
  ok: boolean;
  tools: number;
  context_length: number;
}> {
  return apiFetch<{ ok: boolean; tools: number; context_length: number }>(
    '/api/hardware/reload',
    { method: 'POST' },
  );
}

