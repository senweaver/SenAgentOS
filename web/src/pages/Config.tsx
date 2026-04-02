import { useState, useEffect, useRef, useCallback } from 'react';
import {
  Settings,
  Save,
  CheckCircle,
  AlertTriangle,
  ShieldAlert,
  Bot,
  KeyRound,
  Globe,
  ChevronDown,
  Network,
  ChevronRight,
  ChevronUp,
  Eye,
  EyeOff,
} from 'lucide-react';
import { getConfig, putConfig, getProvider, putProvider, getChannels, putChannels, type ChannelEntry } from '@/lib/api';
import { t } from '@/lib/i18n';


// ---------------------------------------------------------------------------
// TOML syntax highlighter
// ---------------------------------------------------------------------------
function highlightToml(raw: string): string {
  const lines = raw.split('\n');
  const result: string[] = [];

  for (const line of lines) {
    const escaped = line
      .replace(/&/g, '&amp;')
      .replace(/</g, '&lt;')
      .replace(/>/g, '&gt;');

    if (/^\s*\[/.test(escaped)) {
      result.push(`<span style="color:#67e8f9;font-weight:600">${escaped}</span>`);
      continue;
    }
    if (/^\s*#/.test(escaped)) {
      result.push(`<span style="color:#52525b;font-style:italic">${escaped}</span>`);
      continue;
    }

    const kvMatch = escaped.match(/^(\s*)([A-Za-z0-9_\-.]+)(\s*=\s*)(.*)$/);
    if (kvMatch) {
      const [, indent, key, eq, rawValue] = kvMatch;
      result.push(
        `${indent}<span style="color:#a78bfa">${key}</span>`
        + `<span style="color:#71717a">${eq}</span>${colorValue(rawValue ?? '')}`
      );
      continue;
    }
    result.push(escaped);
  }
  return result.join('\n') + '\n';
}

function colorValue(v: string): string {
  const t2 = v.trim();
  const ci = findUnquotedHash(t2);
  if (ci !== -1) {
    return colorScalar(t2.slice(0, ci).trimEnd())
      + ` <span style="color:#52525b;font-style:italic">${t2.slice(ci)}</span>`;
  }
  return colorScalar(v);
}

function findUnquotedHash(s: string): number {
  let inSingle = false, inDouble = false;
  for (let i = 0; i < s.length; i++) {
    const c = s[i];
    if (c === "'" && !inDouble) inSingle = !inSingle;
    else if (c === '"' && !inSingle) inDouble = !inDouble;
    else if (c === '#' && !inSingle && !inDouble) return i;
  }
  return -1;
}

function colorScalar(v: string): string {
  const t2 = v.trim();
  if (t2 === 'true' || t2 === 'false') return `<span style="color:#34d399">${v}</span>`;
  if (/^-?\d[\d_]*(\.[\d_]*)?([eE][+-]?\d+)?$/.test(t2)) return `<span style="color:#fbbf24">${v}</span>`;
  if (t2.startsWith('"') || t2.startsWith("'")) return `<span style="color:#86efac">${v}</span>`;
  if (t2.startsWith('[') || t2.startsWith('{')) return `<span style="color:#e2e8f0">${v}</span>`;
  return v;
}


// ---------------------------------------------------------------------------
// Shared feedback banner
// ---------------------------------------------------------------------------
function FeedbackBanner({
  saved, error,
}: {
  saved: boolean;
  error: string | null;
}) {
  if (!saved && !error) return null;
  return (
    <div className="flex items-start gap-3 rounded-2xl p-4 border animate-fade-in"
      style={saved
        ? { borderColor: 'rgba(0,230,138,0.2)', background: 'rgba(0,230,138,0.06)' }
        : { borderColor: 'rgba(239,68,68,0.2)', background: 'rgba(239,68,68,0.06)' }
      }>
      {saved
        ? <CheckCircle className="h-5 w-5 flex-shrink-0 mt-0.5" style={{ color: 'var(--color-status-success)' }} />
        : <AlertTriangle className="h-5 w-5 flex-shrink-0 mt-0.5" style={{ color: 'var(--color-status-error)' }} />
      }
      <div className="flex-1">
        <p className="text-sm font-medium" style={saved ? { color: 'var(--color-status-success)' } : { color: 'var(--color-status-error)' }}>
          {saved ? t('config.save_success') : error}
        </p>
        {!saved && (
          <p className="text-xs mt-0.5" style={{ color: 'rgba(239,68,68,0.7)' }}>
            {t('config.save_error')}
          </p>
        )}
      </div>
    </div>
  );
}


// ---------------------------------------------------------------------------
// Model / Provider form  (P1-1: now includes gateway settings)
// ---------------------------------------------------------------------------
function ProviderForm() {
  const [provider, setProvider]           = useState('');
  const [model, setModel]               = useState('');
  const [apiKey, setApiKey]             = useState('');
  const [apiUrl, setApiUrl]             = useState('');
  const [gatewayPort, setGatewayPort]   = useState('');
  const [gatewayHost, setGatewayHost]   = useState('');
  const [requirePairing, setRequirePairing] = useState(true);
  const [loading, setLoading]           = useState(true);
  const [saving, setSaving]             = useState(false);
  const [saved, setSaved]               = useState(false);
  const [error, setError]               = useState<string | null>(null);
  const [showKey, setShowKey]           = useState(false);
  const [gatewayOpen, setGatewayOpen]    = useState(false);

  useEffect(() => {
    getProvider()
      .then((cfg) => {
        setProvider(cfg.provider ?? '');
        setModel(cfg.model ?? '');
        setApiKey(cfg.api_key ?? '');
        setApiUrl(cfg.api_url ?? '');
        setGatewayPort(String(cfg.gateway_port ?? 42617));
        setGatewayHost(cfg.gateway_host ?? '127.0.0.1');
        setRequirePairing(cfg.gateway_require_pairing ?? true);
      })
      .catch((e: unknown) => setError(e instanceof Error ? e.message : String(e)))
      .finally(() => setLoading(false));
  }, []);

  const handleSave = async () => {
    setSaving(true);
    setError(null);
    setSaved(false);
    try {
      const body: Record<string, string | number | boolean> = {};
      if (provider.trim()) body.provider = provider.trim();
      if (model.trim())   body.model    = model.trim();
      if (apiKey.trim())  body.api_key  = apiKey.trim();
      if (apiUrl.trim())  body.api_url  = apiUrl.trim();
      const port = parseInt(gatewayPort, 10);
      if (!isNaN(port) && port > 0) body.gateway_port = port;
      if (gatewayHost.trim()) body.gateway_host = gatewayHost.trim();
      body.gateway_require_pairing = requirePairing;
      await putProvider(body);
      setSaved(true);
      const refreshed = await getProvider();
      setApiKey(refreshed.api_key ?? '');
    } catch (e: unknown) {
      setError(e instanceof Error ? e.message : String(e));
    } finally {
      setSaving(false);
    }
  };

  useEffect(() => {
    if (!saved) return;
    const id = setTimeout(() => setSaved(false), 4000);
    return () => clearTimeout(id);
  }, [saved]);

  if (loading) return (
    <div className="flex items-center justify-center h-48">
      <div className="h-6 w-6 border-2 rounded-full animate-spin"
        style={{ borderColor: 'var(--pc-border)', borderTopColor: 'var(--pc-accent)' }} />
    </div>
  );

  return (
    <div className="flex flex-col gap-6 max-w-2xl">

      {/* AI Provider section */}
      <section className="flex flex-col gap-4">
        <div className="flex items-center gap-2">
          <Bot className="h-4 w-4" style={{ color: 'var(--pc-accent)' }} />
          <span className="text-sm font-medium" style={{ color: 'var(--pc-text-primary)' }}>
            AI 模型
          </span>
        </div>
        <div className="flex flex-col gap-4 pl-1">

          {/* Provider */}
          <div className="space-y-1.5">
            <label className="flex items-center gap-2 text-sm font-medium" style={{ color: 'var(--pc-text-primary)' }}>
              <Globe className="h-3.5 w-3.5" style={{ color: 'var(--pc-text-muted)' }} />
              {t('config.provider_label')}
            </label>
            <div className="relative">
              <select
                value={provider}
                onChange={(e) => setProvider(e.target.value)}
                className="input-electric w-full pr-10 appearance-none text-sm"
              >
                <option value="">— 选择供应商 —</option>
                <optgroup label="推荐">
                  <option value="senweaver">SenWeaver（推荐）</option>
                </optgroup>
                <optgroup label="其他">
                  <option value="openai">OpenAI</option>
                  <option value="anthropic">Anthropic (Claude)</option>
                  <option value="openrouter">OpenRouter</option>
                  <option value="deepseek">DeepSeek</option>
                  <option value="groq">Groq</option>
                  <option value="mistral">Mistral</option>
                  <option value="gemini">Google Gemini</option>
                  <option value="ollama">Ollama（本地）</option>
                  <option value="xai">xAI Grok</option>
                  <option value="together">Together AI</option>
                  <option value="fireworks">Fireworks AI</option>
                  <option value="cohere">Cohere</option>
                  <option value="nvidia">NVIDIA NIM</option>
                  <option value="vercel">Vercel AI</option>
                </optgroup>
              </select>
              <ChevronDown className="absolute right-3 top-1/2 -translate-y-1/2 h-4 w-4 pointer-events-none"
                style={{ color: 'var(--pc-text-muted)' }} />
            </div>
          </div>

          {/* Model */}
          <div className="space-y-1.5">
            <label className="flex items-center gap-2 text-sm font-medium" style={{ color: 'var(--pc-text-primary)' }}>
              <Bot className="h-3.5 w-3.5" style={{ color: 'var(--pc-text-muted)' }} />
              {t('config.model_label')}
            </label>
            <input
              type="text"
              value={model}
              onChange={(e) => setModel(e.target.value)}
              placeholder={t('config.model_placeholder')}
              className="input-electric w-full text-sm"
            />
          </div>

          {/* API Key */}
          <div className="space-y-1.5">
            <label className="flex items-center gap-2 text-sm font-medium" style={{ color: 'var(--pc-text-primary)' }}>
              <KeyRound className="h-3.5 w-3.5" style={{ color: 'var(--pc-text-muted)' }} />
              {t('config.api_key_label')}
            </label>
            <div className="relative">
              <input
                type={showKey ? 'text' : 'password'}
                value={apiKey}
                onChange={(e) => setApiKey(e.target.value)}
                placeholder={t('config.api_key_placeholder')}
                className="input-electric w-full pr-20 text-sm font-mono"
              />
              <button
                type="button"
                onClick={() => setShowKey((v) => !v)}
                className="absolute right-2 top-1/2 -translate-y-1/2 text-xs px-2 py-1 rounded border"
                style={{ borderColor: 'var(--pc-border)', color: 'var(--pc-text-muted)' }}
              >
                {showKey ? '隐藏' : '显示'}
              </button>
            </div>
            <p className="text-xs" style={{ color: 'var(--pc-text-muted)' }}>{t('config.api_key_hint')}</p>
          </div>

          {/* API URL */}
          <div className="space-y-1.5">
            <label className="flex items-center gap-2 text-sm font-medium" style={{ color: 'var(--pc-text-primary)' }}>
              <Globe className="h-3.5 w-3.5" style={{ color: 'var(--pc-text-muted)' }} />
              {t('config.api_url_label')}
            </label>
            <input
              type="text"
              value={apiUrl}
              onChange={(e) => setApiUrl(e.target.value)}
              placeholder={t('config.api_url_placeholder')}
              className="input-electric w-full text-sm"
            />
            <p className="text-xs" style={{ color: 'var(--pc-text-muted)' }}>{t('config.api_url_hint')}</p>
          </div>

        </div>
      </section>

      {/* Gateway section (collapsible) */}
      <section className="flex flex-col gap-4">
        <button
          type="button"
          className="flex items-center gap-2 text-sm font-medium w-full text-left"
          style={{ color: 'var(--pc-text-primary)' }}
          onClick={() => setGatewayOpen((v) => !v)}
        >
          <Network className="h-4 w-4" style={{ color: 'var(--pc-accent)' }} />
          {t('config.gateway_label') ?? '网关设置'}
          <span className="ml-auto">
            {gatewayOpen
              ? <ChevronUp className="h-4 w-4" style={{ color: 'var(--pc-text-muted)' }} />
              : <ChevronRight className="h-4 w-4" style={{ color: 'var(--pc-text-muted)' }} />
            }
          </span>
        </button>

        {gatewayOpen && (
          <div className="flex flex-col gap-4 pl-1">

            {/* Gateway Port */}
            <div className="space-y-1.5">
              <label className="flex items-center gap-2 text-sm font-medium" style={{ color: 'var(--pc-text-primary)' }}>
                <Globe className="h-3.5 w-3.5" style={{ color: 'var(--pc-text-muted)' }} />
                {t('config.gateway_port_label') ?? '网关端口'}
              </label>
              <input
                type="number"
                value={gatewayPort}
                onChange={(e) => setGatewayPort(e.target.value)}
                placeholder="42617"
                className="input-electric w-full text-sm"
                min={1}
                max={65535}
              />
              <p className="text-xs" style={{ color: 'var(--pc-text-muted)' }}>
                {t('config.gateway_port_hint') ?? '服务监听端口，默认为 42617。修改后需重启服务。'}
              </p>
            </div>

            {/* Gateway Host */}
            <div className="space-y-1.5">
              <label className="flex items-center gap-2 text-sm font-medium" style={{ color: 'var(--pc-text-primary)' }}>
                <Globe className="h-3.5 w-3.5" style={{ color: 'var(--pc-text-muted)' }} />
                {t('config.gateway_host_label') ?? '监听地址'}
              </label>
              <input
                type="text"
                value={gatewayHost}
                onChange={(e) => setGatewayHost(e.target.value)}
                placeholder="127.0.0.1"
                className="input-electric w-full text-sm"
              />
              <p className="text-xs" style={{ color: 'var(--pc-text-muted)' }}>
                {t('config.gateway_host_hint') ?? '服务监听地址，127.0.0.1 仅本地访问。'}
              </p>
            </div>

            {/* Require Pairing */}
            <div className="flex items-center gap-3">
              <label className="relative inline-flex items-center cursor-pointer">
                <input
                  type="checkbox"
                  checked={requirePairing}
                  onChange={(e) => setRequirePairing(e.target.checked)}
                  className="sr-only peer"
                />
                <div className="w-9 h-5 rounded-full peer transition-colors"
                  style={{
                    backgroundColor: requirePairing ? 'var(--pc-accent)' : 'var(--pc-border)',
                  }}
                />
                <div className="absolute left-0.5 top-0.5 w-4 h-4 bg-white rounded-full transition-transform"
                  style={{ transform: requirePairing ? 'translateX(16px)' : 'translateX(0)' }}
                />
              </label>
              <div>
                <p className="text-sm font-medium" style={{ color: 'var(--pc-text-primary)' }}>
                  {t('config.require_pairing_label') ?? '启用设备配对'}
                </p>
                <p className="text-xs" style={{ color: 'var(--pc-text-muted)' }}>
                  {t('config.require_pairing_hint') ?? '开启后，新设备必须输入配对码才能访问。'}
                </p>
              </div>
            </div>

          </div>
        )}
      </section>

      <FeedbackBanner saved={saved} error={error} />

      <button
        onClick={handleSave}
        disabled={saving}
        className="btn-electric self-start flex items-center gap-2 text-sm px-4 py-2"
      >
        <Save className="h-4 w-4" />
        {saving ? t('config.saving') : t('config.save')}
      </button>
    </div>
  );
}


// ---------------------------------------------------------------------------
// Channel Config form  (P1-2)
// ---------------------------------------------------------------------------
const CHANNEL_META: Record<string, { label: string; tokenField: string; tokenLabel: string; extraFields?: Array<{ key: string; label: string; type: string; placeholder: string }> }> = {
  telegram: {
    label: 'Telegram',
    tokenField: 'bot_token',
    tokenLabel: 'Bot Token',
    extraFields: [
      { key: 'allowed_users', label: '允许的用户（逗号分隔）', type: 'text', placeholder: '@username1, @username2' },
    ],
  },
  discord: {
    label: 'Discord',
    tokenField: 'bot_token',
    tokenLabel: 'Bot Token',
    extraFields: [
      { key: 'guild_id', label: '服务器 ID（可选）', type: 'text', placeholder: '123456789' },
    ],
  },
  slack: {
    label: 'Slack',
    tokenField: 'bot_token',
    tokenLabel: 'Bot Token (xoxb-...)',
    extraFields: [
      { key: 'app_token', label: 'App Token (xapp-..., 可选)', type: 'text', placeholder: 'xapp-...' },
      { key: 'channel_id', label: '频道 ID（可选）', type: 'text', placeholder: 'C012ABCDEF' },
    ],
  },
  mattermost: {
    label: 'Mattermost',
    tokenField: 'bot_token',
    tokenLabel: 'Bot Token',
    extraFields: [
      { key: 'url', label: '服务器 URL', type: 'text', placeholder: 'https://mattermost.example.com' },
    ],
  },
  webhook: {
    label: 'Webhook',
    tokenField: 'secret',
    tokenLabel: 'Webhook Secret',
    extraFields: [],
  },
  matrix: {
    label: 'Matrix',
    tokenField: 'access_token',
    tokenLabel: 'Access Token',
    extraFields: [
      { key: 'homeserver', label: 'Homeserver', type: 'text', placeholder: 'https://matrix.example.com' },
      { key: 'user_id', label: 'User ID', type: 'text', placeholder: '@user:example.com' },
    ],
  },
  whatsapp: {
    label: 'WhatsApp',
    tokenField: 'access_token',
    tokenLabel: 'Access Token',
    extraFields: [],
  },
  linq: {
    label: 'Linq',
    tokenField: 'api_token',
    tokenLabel: 'API Token',
    extraFields: [],
  },
  nextcloud_talk: {
    label: 'Nextcloud Talk',
    tokenField: 'app_token',
    tokenLabel: 'App Token',
    extraFields: [
      { key: 'base_url', label: '服务器 URL', type: 'text', placeholder: 'https://cloud.example.com' },
    ],
  },
  wati: {
    label: 'Wati',
    tokenField: 'api_endpoint',
    tokenLabel: 'API Endpoint',
    extraFields: [],
  },
};

function ChannelForm() {
  const [channels, setChannels]   = useState<ChannelEntry[]>([]);
  const [loading, setLoading]      = useState(true);
  const [saving, setSaving]        = useState(false);
  const [saved, setSaved]          = useState(false);
  const [error, setError]          = useState<string | null>(null);
  const [showTokens, setShowTokens] = useState<Record<string, boolean>>({});
  const [expanded, setExpanded]    = useState<Record<string, boolean>>({});

  useEffect(() => {
    getChannels()
      .then((data) => setChannels(data.channels ?? []))
      .catch((e: unknown) => setError(e instanceof Error ? e.message : String(e)))
      .finally(() => setLoading(false));
  }, []);

  const updateChannel = (name: string, updates: Record<string, unknown>) => {
    setChannels((prev) =>
      prev.map((ch) =>
        ch.name === name ? { ...ch, config: { ...ch.config, ...updates } } : ch
      )
    );
  };

  const toggleExpand = (name: string) =>
    setExpanded((prev) => ({ ...prev, [name]: !prev[name] }));

  const handleSave = async () => {
    setSaving(true);
    setError(null);
    setSaved(false);
    try {
      await putChannels({ channels });
      setSaved(true);
    } catch (e: unknown) {
      setError(e instanceof Error ? e.message : String(e));
    } finally {
      setSaving(false);
    }
  };

  useEffect(() => {
    if (!saved) return;
    const id = setTimeout(() => setSaved(false), 4000);
    return () => clearTimeout(id);
  }, [saved]);

  if (loading) return (
    <div className="flex items-center justify-center h-48">
      <div className="h-6 w-6 border-2 rounded-full animate-spin"
        style={{ borderColor: 'var(--pc-border)', borderTopColor: 'var(--pc-accent)' }} />
    </div>
  );

  return (
    <div className="flex flex-col gap-6 max-w-2xl">

      <div className="flex items-center gap-2">
        <Network className="h-4 w-4" style={{ color: 'var(--pc-accent)' }} />
        <span className="text-sm" style={{ color: 'var(--pc-text-secondary)' }}>
          {t('config.channels_hint') ?? '配置消息频道的 Bot Token，保存后自动启用对应频道。'}
        </span>
      </div>

      {channels.length === 0 && !loading && (
        <div className="rounded-2xl p-6 border text-center"
          style={{ borderColor: 'var(--pc-border)', color: 'var(--pc-text-muted)' }}>
          <p className="text-sm">{t('dashboard.no_channels') ?? '暂无已配置的频道。'}</p>
          <p className="text-xs mt-1" style={{ color: 'var(--pc-text-faint)' }}>
            {'在 TOML 编辑器中添加 [channels_config.telegram] 等配置块来启用频道。'}
          </p>
        </div>
      )}

      <div className="flex flex-col gap-3">
        {channels.map((channel) => {
          const meta = CHANNEL_META[channel.name];
          if (!meta) return null;
          const isOpen = !!expanded[channel.name];
          const tokenVal = String(channel.config[meta.tokenField] ?? '');
          const isMasked = tokenVal.startsWith('***') || tokenVal === '';
          const showing = !!showTokens[channel.name];

          return (
            <div key={channel.name} className="rounded-2xl border overflow-hidden"
              style={{ borderColor: 'var(--pc-border)', background: 'var(--pc-bg-elevated)' }}>

              {/* Channel header */}
              <button
                type="button"
                className="w-full flex items-center justify-between px-4 py-3 text-left"
                onClick={() => toggleExpand(channel.name)}
              >
                <div className="flex items-center gap-3">
                  <span
                    className="status-dot"
                    style={{
                      background: channel.enabled ? '#34d399' : 'var(--pc-border)',
                      boxShadow: channel.enabled ? '0 0 6px #34d399' : 'none',
                    }}
                  />
                  <span className="text-sm font-medium capitalize" style={{ color: 'var(--pc-text-primary)' }}>
                    {meta.label}
                  </span>
                </div>
                <div className="flex items-center gap-2">
                  <span className="text-xs" style={{ color: 'var(--pc-text-muted)' }}>
                    {isMasked ? '未配置' : '已配置'}
                  </span>
                  {isOpen
                    ? <ChevronUp className="h-4 w-4" style={{ color: 'var(--pc-text-muted)' }} />
                    : <ChevronRight className="h-4 w-4" style={{ color: 'var(--pc-text-muted)' }} />
                  }
                </div>
              </button>

              {/* Expanded fields */}
              {isOpen && (
                <div className="px-4 pb-4 pt-1 flex flex-col gap-3 border-t"
                  style={{ borderColor: 'var(--pc-border)' }}>

                  {/* Token field */}
                  <div className="space-y-1.5 mt-3">
                    <label className="flex items-center gap-2 text-sm font-medium"
                      style={{ color: 'var(--pc-text-primary)' }}>
                      <KeyRound className="h-3.5 w-3.5" style={{ color: 'var(--pc-text-muted)' }} />
                      {meta.tokenLabel}
                    </label>
                    <div className="relative">
                      <input
                        type={showing ? 'text' : 'password'}
                        value={tokenVal && isMasked ? '' : tokenVal}
                        onChange={(e) => updateChannel(channel.name, { [meta.tokenField]: e.target.value })}
                        placeholder={t('config.api_key_placeholder') ?? ''}
                        className="input-electric w-full pr-16 text-sm font-mono"
                      />
                      {tokenVal && !isMasked && (
                        <button
                          type="button"
                          onClick={() => setShowTokens((p) => ({ ...p, [channel.name]: !p[channel.name] }))}
                          className="absolute right-2 top-1/2 -translate-y-1/2 text-xs px-2 py-1 rounded border"
                          style={{ borderColor: 'var(--pc-border)', color: 'var(--pc-text-muted)' }}
                        >
                          {showing ? <EyeOff className="h-3.5 w-3.5" /> : <Eye className="h-3.5 w-3.5" />}
                        </button>
                      )}
                    </div>
                  </div>

                  {/* Extra fields */}
                  {(meta.extraFields ?? []).map((field) => (
                    <div key={field.key} className="space-y-1.5">
                      <label className="text-sm font-medium" style={{ color: 'var(--pc-text-primary)' }}>
                        {field.label}
                      </label>
                      <input
                        type="text"
                        value={String(channel.config[field.key] ?? '')}
                        onChange={(e) => updateChannel(channel.name, { [field.key]: e.target.value })}
                        placeholder={field.placeholder}
                        className="input-electric w-full text-sm"
                      />
                    </div>
                  ))}

                </div>
              )}
            </div>
          );
        })}
      </div>

      <FeedbackBanner saved={saved} error={error} />

      {channels.length > 0 && (
        <button
          onClick={handleSave}
          disabled={saving}
          className="btn-electric self-start flex items-center gap-2 text-sm px-4 py-2"
        >
          <Save className="h-4 w-4" />
          {saving ? t('config.saving') : t('config.save')}
        </button>
      )}
    </div>
  );
}


// ---------------------------------------------------------------------------
// TOML Editor
// ---------------------------------------------------------------------------
function TomlEditor({ onSave }: { onSave: () => void }) {
  const [config, setConfig]   = useState('');
  const [loading, setLoading] = useState(true);
  const textareaRef = useRef<HTMLTextAreaElement>(null);
  const preRef = useRef<HTMLPreElement>(null);

  const syncScroll = useCallback(() => {
    if (preRef.current && textareaRef.current) {
      preRef.current.scrollTop = textareaRef.current.scrollTop;
      preRef.current.scrollLeft = textareaRef.current.scrollLeft;
    }
  }, []);

  useEffect(() => {
    getConfig()
      .then((data) => setConfig(typeof data === 'string' ? data : JSON.stringify(data, null, 2)))
      .finally(() => setLoading(false));
  }, []);

  if (loading) return (
    <div className="flex items-center justify-center h-48">
      <div className="h-6 w-6 border-2 rounded-full animate-spin"
        style={{ borderColor: 'var(--pc-border)', borderTopColor: 'var(--pc-accent)' }} />
    </div>
  );

  return (
    <div className="flex flex-col gap-4 flex-1 min-h-0">
      <div className="card overflow-hidden rounded-2xl flex flex-col flex-1 min-h-[400px]">
        <div className="flex items-center justify-between px-4 py-2.5 border-b shrink-0"
          style={{ borderColor: 'var(--pc-border)', background: 'var(--pc-accent-glow)' }}>
          <span className="text-[10px] font-semibold uppercase tracking-wider"
            style={{ color: 'var(--pc-text-muted)' }}>{t('config.toml_label')}</span>
          <span className="text-[10px]" style={{ color: 'var(--pc-text-faint)' }}>
            {config.split('\n').length} {t('config.lines')}
          </span>
        </div>
        <div className="relative flex-1 min-h-0 overflow-hidden">
          <pre
            ref={preRef}
            aria-hidden="true"
            className="absolute inset-0 text-sm p-4 font-mono overflow-auto whitespace-pre pointer-events-none m-0"
            style={{ background: 'var(--pc-bg-base)', tabSize: 4 }}
            dangerouslySetInnerHTML={{ __html: highlightToml(config) }}
          />
          <textarea
            ref={textareaRef}
            value={config}
            onChange={(e) => setConfig(e.target.value)}
            onScroll={syncScroll}
            onKeyDown={(e) => {
              if (e.key === 'Tab') {
                e.preventDefault();
                const el = e.currentTarget;
                const s = el.selectionStart, en = el.selectionEnd;
                setConfig(config.slice(0, s) + '  ' + config.slice(en));
                requestAnimationFrame(() => { el.selectionStart = el.selectionEnd = s + 2; });
              }
              if (e.key === 's' && (e.ctrlKey || e.metaKey)) {
                e.preventDefault();
                onSave();
              }
            }}
            spellCheck={false}
            className="absolute inset-0 w-full h-full text-sm p-4 resize-none focus:outline-none font-mono caret-white"
            style={{ background: 'transparent', color: 'transparent', tabSize: 4 }}
          />
        </div>
      </div>
    </div>
  );
}


// ---------------------------------------------------------------------------
// Main Config page
// ---------------------------------------------------------------------------
type TabId = 'provider' | 'channels' | 'toml';

export default function Config() {
  const [activeTab, setActiveTab] = useState<TabId>('provider');
  const [tomlSaving, setTomlSaving]   = useState(false);
  const [tomlSaved, setTomlSaved]     = useState(false);
  const [tomlError, setTomlError]     = useState<string | null>(null);
  const [tomlConfig]   = useState('');

  const handleTomlSave = async () => {
    setTomlSaving(true);
    setTomlError(null);
    setTomlSaved(false);
    try {
      await putConfig(tomlConfig);
      setTomlSaved(true);
    } catch (e: unknown) {
      setTomlError(e instanceof Error ? e.message : String(e));
    } finally {
      setTomlSaving(false);
    }
  };

  useEffect(() => {
    if (!tomlSaved) return;
    const id = setTimeout(() => setTomlSaved(false), 4000);
    return () => clearTimeout(id);
  }, [tomlSaved]);

  const tabs: { id: TabId; label: string; icon: React.ComponentType<{ className?: string; style?: React.CSSProperties }> }[] = [
    { id: 'provider', label: '模型配置',  icon: Bot },
    { id: 'channels', label: '频道配置', icon: Network },
    { id: 'toml',     label: 'TOML 编辑', icon: Settings },
  ];

  return (
    <div className="flex flex-col h-full p-6 gap-5 animate-fade-in overflow-hidden">
      {/* Header */}
      <div className="flex items-center justify-between">
        <div className="flex items-center gap-2">
          <Settings className="h-5 w-5" style={{ color: 'var(--pc-accent)' }} />
          <h2 className="text-sm font-semibold uppercase tracking-wider"
            style={{ color: 'var(--pc-text-primary)' }}>{t('config.configuration_title')}</h2>
        </div>
        {activeTab === 'toml' && (
          <button
            onClick={handleTomlSave}
            disabled={tomlSaving}
            className="btn-electric flex items-center gap-2 text-sm px-4 py-2"
          >
            <Save className="h-4 w-4" />
            {tomlSaving ? t('config.saving') : t('config.save')}
          </button>
        )}
      </div>

      {/* TOML sensitive note */}
      {activeTab === 'toml' && (
        <div className="flex items-start gap-3 rounded-2xl p-4 border"
          style={{ borderColor: 'rgba(255,170,0,0.2)', background: 'rgba(255,170,0,0.05)' }}>
          <ShieldAlert className="h-5 w-5 flex-shrink-0 mt-0.5" style={{ color: 'var(--color-status-warning)' }} />
          <div>
            <p className="text-sm font-medium" style={{ color: 'var(--color-status-warning)' }}>{t('config.sensitive_title')}</p>
            <p className="text-sm mt-0.5" style={{ color: 'rgba(255,170,0,0.7)' }}>{t('config.sensitive_hint')}</p>
          </div>
        </div>
      )}

      {/* TOML feedback */}
      {activeTab === 'toml' && (tomlSaved || tomlError) && (
        tomlSaved ? (
          <div className="flex items-center gap-2 rounded-xl p-3 border animate-fade-in"
            style={{ borderColor: 'rgba(0,230,138,0.2)', background: 'rgba(0,230,138,0.06)' }}>
            <CheckCircle className="h-4 w-4" style={{ color: 'var(--color-status-success)' }} />
            <span className="text-sm" style={{ color: 'var(--color-status-success)' }}>{t('config.save_success')}</span>
          </div>
        ) : (
          <div className="flex items-center gap-2 rounded-xl p-3 border animate-fade-in"
            style={{ borderColor: 'rgba(239,68,68,0.2)', background: 'rgba(239,68,68,0.06)' }}>
            <AlertTriangle className="h-4 w-4" style={{ color: 'var(--color-status-error)' }} />
            <span className="text-sm" style={{ color: 'var(--color-status-error)' }}>{tomlError}</span>
          </div>
        )
      )}

      {/* Tab bar */}
      <div className="flex items-center gap-1 p-1 rounded-xl w-fit"
        style={{ background: 'var(--pc-bg-elevated)', border: '1px solid var(--pc-border)' }}>
        {tabs.map(({ id, label, icon: Icon }) => (
          <button
            key={id}
            onClick={() => setActiveTab(id)}
            className="flex items-center gap-2 px-4 py-2 rounded-lg text-sm font-medium transition-all"
            style={
              activeTab === id
                ? { background: 'var(--pc-accent)', color: 'white', boxShadow: '0 0 12px rgba(0,0,0,0.3)' }
                : { color: 'var(--pc-text-secondary)' }
            }
          >
            <Icon className="h-4 w-4" style={activeTab === id ? {} : { color: 'var(--pc-text-muted)' }} />
            {label}
          </button>
        ))}
      </div>

      {/* Content */}
      <div className="flex-1 min-h-0 overflow-auto">
        {activeTab === 'provider' ? <ProviderForm /> :
         activeTab === 'channels' ? <ChannelForm /> :
         <TomlEditor onSave={handleTomlSave} />}
      </div>
    </div>
  );
}
