import { useState, useEffect, useCallback } from 'react';
import {
  Cpu,
  Wifi,
  Cable,
  Zap,
  RefreshCw,
  Plus,
  Database,
  Info,
  CheckCircle2,
  AlertCircle,
  XCircle,
  ChevronDown,
  ChevronRight,
} from 'lucide-react';
import type {
  HardwareBoardsResponse,
  HardwareContextResponse,
  BoardInfo,
} from '@/types/api';
import {
  getHardwareBoards,
  getHardwareContext,
  registerGpioPin,
  reloadHardwareContext,
} from '@/lib/api';
import { t } from '@/lib/i18n';

// ---------------------------------------------------------------------------
// Helpers
// ---------------------------------------------------------------------------

function transportIcon(transport: string) {
  switch (transport) {
    case 'serial':    return <Cable className="h-5 w-5" />;
    case 'websocket': return <Wifi className="h-5 w-5" />;
    case 'native':    return <Cpu className="h-5 w-5" />;
    default:          return <Cable className="h-5 w-5" />;
  }
}

function transportColor(transport: string): string {
  switch (transport) {
    case 'serial':    return '#a78bfa';
    case 'websocket': return '#34d399';
    case 'native':    return '#fbbf24';
    default:          return 'var(--pc-text-muted)';
  }
}

function transportBg(transport: string): string {
  switch (transport) {
    case 'serial':    return 'rgba(167, 139, 250, 0.08)';
    case 'websocket': return 'rgba(52, 211, 153, 0.08)';
    case 'native':    return 'rgba(251, 191, 36, 0.08)';
    default:          return 'var(--pc-bg-elevated)';
  }
}

function statusForBoard(_board: BoardInfo): { label: string; color: string; bg: string } {
  // Boards without a path or with "native" transport may not be actively connected.
  // Without runtime health data, we treat "serial with path" as connected.
  if (!_board.path && _board.transport !== 'native') {
    return { label: 'No Path', color: 'var(--color-status-warning)', bg: 'rgba(255, 170, 0, 0.08)' };
  }
  return { label: 'Configured', color: 'var(--color-status-success)', bg: 'rgba(0, 230, 138, 0.08)' };
}

function parseGpioLines(content: string): { pin: number; component: string; notes: string }[] {
  const lines = content.split('\n');
  const results: { pin: number; component: string; notes: string }[] = [];
  for (const line of lines) {
    const gpioMatch = line.match(/^- GPIO (\d+): (.+?)(?: — (.*))?$/);
    if (gpioMatch) {
      const pinStr = gpioMatch[1];
      const componentStr = gpioMatch[2];
      if (!pinStr || !componentStr) continue;
      results.push({
        pin: parseInt(pinStr, 10),
        component: componentStr.trim(),
        notes: gpioMatch[3]?.trim() ?? '',
      });
    }
  }
  return results;
}

// ---------------------------------------------------------------------------
// Board Card
// ---------------------------------------------------------------------------

function BoardCard({ board }: { board: BoardInfo }) {
  const { label, color, bg } = statusForBoard(board);

  return (
    <div
      className="card p-5 animate-slide-in-up transition-all"
      style={{ borderColor: `${color}30` }}
      onMouseEnter={(e) => {
        e.currentTarget.style.transform = 'translateY(-2px)';
        e.currentTarget.style.boxShadow = `0 4px 16px ${color}20`;
      }}
      onMouseLeave={(e) => {
        e.currentTarget.style.transform = 'translateY(0)';
        e.currentTarget.style.boxShadow = 'none';
      }}
    >
      {/* Header */}
      <div className="flex items-start gap-3 mb-4">
        <div
          className="p-3 rounded-2xl shrink-0"
          style={{ background: transportBg(board.transport), color: transportColor(board.transport) }}
        >
          {transportIcon(board.transport)}
        </div>
        <div className="flex-1 min-w-0">
          <h3 className="text-base font-semibold truncate capitalize" style={{ color: 'var(--pc-text-primary)' }}>
            {board.board.replace(/-/g, ' ')}
          </h3>
          <p className="text-sm font-mono truncate" style={{ color: 'var(--pc-text-muted)' }}>
            {board.chip}
          </p>
        </div>
        <span
          className="text-[10px] uppercase font-bold px-2.5 py-1 rounded-full shrink-0"
          style={{ background: bg, color }}
        >
          {label}
        </span>
      </div>

      {/* Description */}
      <p className="text-xs mb-4 leading-relaxed" style={{ color: 'var(--pc-text-secondary)' }}>
        {board.description}
      </p>

      {/* Details */}
      <div
        className="rounded-xl p-3 space-y-2"
        style={{ background: 'var(--pc-bg-elevated)' }}
      >
        <div className="flex justify-between text-xs">
          <span style={{ color: 'var(--pc-text-faint)' }}>{t('hardware.transport')}</span>
          <span className="font-medium capitalize" style={{ color: transportColor(board.transport) }}>
            {board.transport}
          </span>
        </div>
        {board.path && (
          <div className="flex justify-between text-xs">
            <span style={{ color: 'var(--pc-text-faint)' }}>{t('hardware.port')}</span>
            <span className="font-mono text-[10px]" style={{ color: 'var(--pc-text-secondary)' }}>
              {board.path}
            </span>
          </div>
        )}
        {board.baud > 0 && (
          <div className="flex justify-between text-xs">
            <span style={{ color: 'var(--pc-text-faint)' }}>{t('hardware.baud')}</span>
            <span className="font-mono" style={{ color: 'var(--pc-text-secondary)' }}>
              {board.baud.toLocaleString()} bps
            </span>
          </div>
        )}
      </div>
    </div>
  );
}

// ---------------------------------------------------------------------------
// Boards Tab
// ---------------------------------------------------------------------------

function BoardsTab() {
  const [data, setData] = useState<HardwareBoardsResponse | null>(null);
  const [loading, setLoading] = useState(true);
  const [error, setError] = useState<string | null>(null);

  const load = useCallback(() => {
    setLoading(true);
    setError(null);
    getHardwareBoards()
      .then((d) => { setData(d); setLoading(false); })
      .catch((e) => { setError(e.message); setLoading(false); });
  }, []);

  useEffect(() => { load(); }, [load]);

  if (loading) {
    return (
      <div className="flex items-center justify-center h-48">
        <RefreshCw className="h-6 w-6 animate-spin" style={{ color: 'var(--pc-accent)' }} />
      </div>
    );
  }

  if (error) {
    return (
      <div className="rounded-2xl border p-4" style={{ background: 'rgba(239, 68, 102, 0.08)', borderColor: 'rgba(239, 68, 102, 0.2)' }}>
        <p style={{ color: 'var(--color-status-error)' }}>{t('hardware.load_boards_error')}: {error}</p>
      </div>
    );
  }

  if (!data) return null;

  if (!data.enabled || data.boards.length === 0) {
    return (
      <div className="card p-8 text-center animate-fade-in">
        <Cpu className="h-12 w-12 mx-auto mb-4" style={{ color: 'var(--pc-text-faint)' }} />
        <h3 className="text-base font-semibold mb-2" style={{ color: 'var(--pc-text-primary)' }}>
          {t('hardware.no_boards_title')}
        </h3>
        <p className="text-sm mb-4" style={{ color: 'var(--pc-text-muted)' }}>
          {t('hardware.no_boards_desc')}
        </p>
        <code
          className="block text-xs font-mono rounded-xl p-4 text-left"
          style={{ background: 'var(--pc-bg-elevated)', color: 'var(--pc-text-secondary)' }}
        >
          {`[peripherals]
enabled = true

[[peripherals.boards]]
board = "nucleo-f401re"
transport = "serial"
path = "/dev/ttyACM0"
baud = 115200`}
        </code>
      </div>
    );
  }

  return (
    <div className="space-y-6">
      <div className="flex items-center justify-between">
        <div className="flex items-center gap-2">
          <Cpu className="h-5 w-5" style={{ color: 'var(--pc-accent)' }} />
          <h2 className="text-sm font-semibold uppercase tracking-wider" style={{ color: 'var(--pc-text-primary)' }}>
            {t('hardware.configured_boards')}
          </h2>
          <span
            className="text-xs font-mono px-2 py-0.5 rounded-full"
            style={{ background: 'rgba(var(--pc-accent-rgb), 0.1)', color: 'var(--pc-accent)' }}
          >
            {data.boards.length}
          </span>
        </div>
        <button
          onClick={load}
          className="flex items-center gap-2 text-xs px-3 py-1.5 rounded-xl border transition-all"
          style={{ borderColor: 'var(--pc-border)', color: 'var(--pc-text-muted)' }}
          onMouseEnter={(e) => {
            e.currentTarget.style.borderColor = 'var(--pc-accent)';
            e.currentTarget.style.color = 'var(--pc-accent)';
          }}
          onMouseLeave={(e) => {
            e.currentTarget.style.borderColor = 'var(--pc-border)';
            e.currentTarget.style.color = 'var(--pc-text-muted)';
          }}
        >
          <RefreshCw className="h-3.5 w-3.5" />
          {t('hardware.refresh')}
        </button>
      </div>

      <div className="grid grid-cols-1 sm:grid-cols-2 lg:grid-cols-3 gap-4 stagger-children">
        {data.boards.map((b) => (
          <BoardCard key={b.board} board={b} />
        ))}
      </div>
    </div>
  );
}

// ---------------------------------------------------------------------------
// GPIO Tab
// ---------------------------------------------------------------------------

function GpioTab() {
  const [context, setContext] = useState<HardwareContextResponse | null>(null);
  const [loading, setLoading] = useState(true);
  const [error, setError] = useState<string | null>(null);

  // Register pin form
  const [device, setDevice] = useState('rpi0');
  const [pin, setPin] = useState('');
  const [component, setComponent] = useState('');
  const [notes, setNotes] = useState('');
  const [submitting, setSubmitting] = useState(false);
  const [submitMsg, setSubmitMsg] = useState<{ ok: boolean; text: string } | null>(null);

  const loadContext = useCallback(() => {
    setLoading(true);
    getHardwareContext()
      .then((d) => { setContext(d); setLoading(false); })
      .catch((e) => { setError(e.message); setLoading(false); });
  }, []);

  useEffect(() => { loadContext(); }, [loadContext]);

  const handleSubmit = async (e: React.FormEvent) => {
    e.preventDefault();
    if (!pin || !component) return;
    setSubmitting(true);
    setSubmitMsg(null);
    try {
      const result = await registerGpioPin({
        device: device || 'rpi0',
        pin: parseInt(pin, 10),
        component,
        notes,
      });
      setSubmitMsg({ ok: true, text: result.message });
      setPin('');
      setComponent('');
      setNotes('');
      loadContext();
    } catch (err: unknown) {
      setSubmitMsg({ ok: false, text: err instanceof Error ? err.message : String(err) });
    } finally {
      setSubmitting(false);
    }
  };

  if (loading) {
    return (
      <div className="flex items-center justify-center h-48">
        <RefreshCw className="h-6 w-6 animate-spin" style={{ color: 'var(--pc-accent)' }} />
      </div>
    );
  }

  if (error) {
    return (
      <div className="rounded-2xl border p-4" style={{ background: 'rgba(239, 68, 102, 0.08)', borderColor: 'rgba(239, 68, 102, 0.2)' }}>
        <p style={{ color: 'var(--color-status-error)' }}>{t('hardware.load_gpio_error')}: {error}</p>
      </div>
    );
  }

  const allPins: { device: string; pin: number; component: string; notes: string }[] = [];
  if (context?.devices) {
    for (const [dev, content] of Object.entries(context.devices)) {
      for (const entry of parseGpioLines(content)) {
        allPins.push({ device: dev, ...entry });
      }
    }
  }

  return (
    <div className="space-y-6">
      {/* Register Pin Form */}
      <div className="card p-5 animate-slide-in-up">
        <div className="flex items-center gap-2 mb-4">
          <Zap className="h-5 w-5" style={{ color: 'var(--pc-accent)' }} />
          <h2 className="text-sm font-semibold uppercase tracking-wider" style={{ color: 'var(--pc-text-primary)' }}>
            {t('hardware.register_pin')}
          </h2>
        </div>

        <form onSubmit={handleSubmit} className="space-y-3">
          <div className="grid grid-cols-2 sm:grid-cols-4 gap-3">
            <div>
              <label className="block text-xs mb-1" style={{ color: 'var(--pc-text-faint)' }}>
                {t('hardware.device')}
              </label>
              <input
                type="text"
                value={device}
                onChange={(e) => setDevice(e.target.value)}
                className="input-electric w-full px-3 py-2 text-sm"
                placeholder="rpi0"
              />
            </div>
            <div>
              <label className="block text-xs mb-1" style={{ color: 'var(--pc-text-faint)' }}>
                {t('hardware.gpio_pin')}
              </label>
              <input
                type="number"
                value={pin}
                onChange={(e) => setPin(e.target.value)}
                className="input-electric w-full px-3 py-2 text-sm"
                placeholder="17"
                min="0"
                max="27"
                required
              />
            </div>
            <div>
              <label className="block text-xs mb-1" style={{ color: 'var(--pc-text-faint)' }}>
                {t('hardware.component')}
              </label>
              <input
                type="text"
                value={component}
                onChange={(e) => setComponent(e.target.value)}
                className="input-electric w-full px-3 py-2 text-sm"
                placeholder="LED, Button..."
                required
              />
            </div>
            <div>
              <label className="block text-xs mb-1" style={{ color: 'var(--pc-text-faint)' }}>
                {t('hardware.notes_optional')}
              </label>
              <input
                type="text"
                value={notes}
                onChange={(e) => setNotes(e.target.value)}
                className="input-electric w-full px-3 py-2 text-sm"
                placeholder="red, active HIGH"
              />
            </div>
          </div>

          <div className="flex items-center gap-3">
            <button
              type="submit"
              disabled={submitting || !pin || !component}
              className="btn-electric flex items-center gap-2 px-4 py-2 text-sm"
            >
              {submitting ? (
                <RefreshCw className="h-4 w-4 animate-spin" />
              ) : (
                <Plus className="h-4 w-4" />
              )}
              {t('hardware.register')}
            </button>
            {submitMsg && (
              <span
                className="text-xs flex items-center gap-1"
                style={{ color: submitMsg.ok ? 'var(--color-status-success)' : 'var(--color-status-error)' }}
              >
                {submitMsg.ok
                  ? <CheckCircle2 className="h-4 w-4" />
                  : <XCircle className="h-4 w-4" />}
                {submitMsg.text}
              </span>
            )}
          </div>
        </form>
      </div>

      {/* Pin List */}
      <div className="card p-5 animate-slide-in-up">
        <div className="flex items-center justify-between mb-4">
          <div className="flex items-center gap-2">
            <Zap className="h-5 w-5" style={{ color: 'var(--pc-accent)' }} />
            <h2 className="text-sm font-semibold uppercase tracking-wider" style={{ color: 'var(--pc-text-primary)' }}>
              {t('hardware.gpio_pins')}
            </h2>
            <span
              className="text-xs font-mono px-2 py-0.5 rounded-full"
              style={{ background: 'rgba(var(--pc-accent-rgb), 0.1)', color: 'var(--pc-accent)' }}
            >
              {allPins.length}
            </span>
          </div>
          <button
            onClick={loadContext}
            className="flex items-center gap-1.5 text-xs px-2.5 py-1 rounded-xl border transition-all"
            style={{ borderColor: 'var(--pc-border)', color: 'var(--pc-text-muted)' }}
          >
            <RefreshCw className="h-3 w-3" />
            {t('hardware.refresh')}
          </button>
        </div>

        {allPins.length === 0 ? (
          <p className="text-sm py-6 text-center" style={{ color: 'var(--pc-text-faint)' }}>
            {t('hardware.no_pins')}
          </p>
        ) : (
          <div className="space-y-2">
            {allPins.map((p, i) => (
              <div
                key={i}
                className="flex items-center gap-3 py-3 px-4 rounded-xl transition-all"
                style={{ background: 'var(--pc-bg-elevated)' }}
              >
                <span
                  className="text-xs font-mono font-bold w-10 text-center py-1 rounded-full shrink-0"
                  style={{ background: 'rgba(var(--pc-accent-rgb), 0.08)', color: 'var(--pc-accent)' }}
                >
                  {p.pin}
                </span>
                <span className="text-sm font-medium" style={{ color: 'var(--pc-text-primary)' }}>
                  {p.component}
                </span>
                {p.notes && (
                  <span className="text-xs" style={{ color: 'var(--pc-text-muted)' }}>
                    — {p.notes}
                  </span>
                )}
                <span
                  className="ml-auto text-[10px] uppercase font-medium px-2 py-0.5 rounded-full shrink-0"
                  style={{ background: 'var(--pc-bg-base)', color: 'var(--pc-text-faint)' }}
                >
                  {p.device}
                </span>
              </div>
            ))}
          </div>
        )}
      </div>
    </div>
  );
}

// ---------------------------------------------------------------------------
// Context Tab
// ---------------------------------------------------------------------------

function ContextTab() {
  const [context, setContext] = useState<HardwareContextResponse | null>(null);
  const [loading, setLoading] = useState(true);
  const [error, setError] = useState<string | null>(null);
  const [reloading, setReloading] = useState(false);
  const [reloadResult, setReloadResult] = useState<{ ok: boolean; text: string } | null>(null);
  const [expandedDevice, setExpandedDevice] = useState<string | null>(null);

  const load = useCallback(() => {
    setLoading(true);
    getHardwareContext()
      .then((d) => { setContext(d); setLoading(false); })
      .catch((e) => { setError(e.message); setLoading(false); });
  }, []);

  useEffect(() => { load(); }, [load]);

  const handleReload = async () => {
    setReloading(true);
    setReloadResult(null);
    try {
      const r = await reloadHardwareContext();
      setReloadResult({
        ok: true,
        text: `${t('hardware.reload_success')} — ${r.tools} ${t('hardware.tools')} · ${r.context_length} chars`,
      });
      load();
    } catch (err: unknown) {
      setReloadResult({
        ok: false,
        text: err instanceof Error ? err.message : String(err),
      });
    } finally {
      setReloading(false);
    }
  };

  if (loading) {
    return (
      <div className="flex items-center justify-center h-48">
        <RefreshCw className="h-6 w-6 animate-spin" style={{ color: 'var(--pc-accent)' }} />
      </div>
    );
  }

  if (error) {
    return (
      <div className="rounded-2xl border p-4" style={{ background: 'rgba(239, 68, 102, 0.08)', borderColor: 'rgba(239, 68, 102, 0.2)' }}>
        <p style={{ color: 'var(--color-status-error)' }}>{t('hardware.load_context_error')}: {error}</p>
      </div>
    );
  }

  const devices = context?.devices ?? {};
  const hasHardwareMd = context?.hardware_md && context.hardware_md.trim().length > 0;
  const hasDevices = Object.keys(devices).length > 0;

  if (!hasHardwareMd && !hasDevices) {
    return (
      <div className="card p-8 text-center animate-fade-in">
        <Database className="h-12 w-12 mx-auto mb-4" style={{ color: 'var(--pc-text-faint)' }} />
        <h3 className="text-base font-semibold mb-2" style={{ color: 'var(--pc-text-primary)' }}>
          {t('hardware.no_context_title')}
        </h3>
        <p className="text-sm" style={{ color: 'var(--pc-text-muted)' }}>
          {t('hardware.no_context_desc')}
        </p>
      </div>
    );
  }

  return (
    <div className="space-y-6">
      {/* HARDWARE.md */}
      {hasHardwareMd && (
        <div className="card p-5 animate-slide-in-up">
          <div className="flex items-center gap-2 mb-4">
            <Info className="h-5 w-5" style={{ color: 'var(--pc-accent)' }} />
            <h2 className="text-sm font-semibold uppercase tracking-wider" style={{ color: 'var(--pc-text-primary)' }}>
              HARDWARE.md
            </h2>
          </div>
          <pre
            className="text-xs font-mono whitespace-pre-wrap leading-relaxed rounded-xl p-4 overflow-x-auto"
            style={{ background: 'var(--pc-bg-elevated)', color: 'var(--pc-text-secondary)' }}
          >
            {context?.hardware_md}
          </pre>
        </div>
      )}

      {/* Device Files */}
      {hasDevices && (
        <div className="card p-5 animate-slide-in-up">
          <div className="flex items-center gap-2 mb-4">
            <Cpu className="h-5 w-5" style={{ color: 'var(--pc-accent)' }} />
            <h2 className="text-sm font-semibold uppercase tracking-wider" style={{ color: 'var(--pc-text-primary)' }}>
              {t('hardware.device_files')}
            </h2>
            <span
              className="text-xs font-mono px-2 py-0.5 rounded-full"
              style={{ background: 'rgba(var(--pc-accent-rgb), 0.1)', color: 'var(--pc-accent)' }}
            >
              {Object.keys(devices).length}
            </span>
          </div>
          <div className="space-y-2">
            {Object.entries(devices).map(([name, content]) => (
              <div key={name} className="rounded-xl overflow-hidden" style={{ border: '1px solid var(--pc-border)' }}>
                <button
                  onClick={() => setExpandedDevice(expandedDevice === name ? null : name)}
                  className="w-full flex items-center justify-between px-4 py-3 text-left transition-all"
                  style={{ background: 'var(--pc-bg-elevated)' }}
                  onMouseEnter={(e) => { e.currentTarget.style.background = 'var(--pc-hover)'; }}
                  onMouseLeave={(e) => { e.currentTarget.style.background = 'var(--pc-bg-elevated)'; }}
                >
                  <div className="flex items-center gap-2">
                    <span className="text-sm font-medium capitalize" style={{ color: 'var(--pc-text-primary)' }}>
                      {name.replace(/-/g, ' ')}
                    </span>
                    <span className="text-[10px] font-mono" style={{ color: 'var(--pc-text-faint)' }}>
                      {content.split('\n').filter(Boolean).length} lines
                    </span>
                  </div>
                  {expandedDevice === name
                    ? <ChevronDown className="h-4 w-4" style={{ color: 'var(--pc-text-muted)' }} />
                    : <ChevronRight className="h-4 w-4" style={{ color: 'var(--pc-text-muted)' }} />}
                </button>
                {expandedDevice === name && (
                  <pre
                    className="text-xs font-mono whitespace-pre-wrap leading-relaxed px-4 py-3 border-t"
                    style={{ background: 'var(--pc-bg-base)', color: 'var(--pc-text-secondary)', borderColor: 'var(--pc-border)' }}
                  >
                    {content}
                  </pre>
                )}
              </div>
            ))}
          </div>
        </div>
      )}

      {/* Reload Button */}
      <div className="flex items-center gap-3">
        <button
          onClick={handleReload}
          disabled={reloading}
          className="btn-electric flex items-center gap-2 px-4 py-2 text-sm"
        >
          <RefreshCw className={`h-4 w-4 ${reloading ? 'animate-spin' : ''}`} />
          {t('hardware.reload_context')}
        </button>
        {reloadResult && (
          <span
            className="text-xs flex items-center gap-1"
            style={{ color: reloadResult.ok ? 'var(--color-status-success)' : 'var(--color-status-error)' }}
          >
            {reloadResult.ok
              ? <CheckCircle2 className="h-4 w-4" />
              : <AlertCircle className="h-4 w-4" />}
            {reloadResult.text}
          </span>
        )}
      </div>
    </div>
  );
}

// ---------------------------------------------------------------------------
// Main Hardware Page
// ---------------------------------------------------------------------------

type HardwareTabId = 'boards' | 'gpio' | 'context';

const TABS: { id: HardwareTabId; labelKey: string; icon: typeof Cpu }[] = [
  { id: 'boards', labelKey: 'hardware.tab_boards',  icon: Cpu },
  { id: 'gpio',   labelKey: 'hardware.tab_gpio',   icon: Zap },
  { id: 'context',labelKey: 'hardware.tab_context', icon: Database },
];

export default function Hardware() {
  const [activeTab, setActiveTab] = useState<HardwareTabId>('boards');

  return (
    <div className="p-6 space-y-6 animate-fade-in">
      {/* Page Header */}
      <div className="flex items-center gap-3">
        <div
          className="p-2.5 rounded-2xl"
          style={{ background: 'rgba(var(--pc-accent-rgb), 0.1)', color: 'var(--pc-accent)' }}
        >
          <Cpu className="h-6 w-6" />
        </div>
        <div>
          <h1 className="text-xl font-bold" style={{ color: 'var(--pc-text-primary)' }}>
            {t('hardware.title')}
          </h1>
          <p className="text-sm" style={{ color: 'var(--pc-text-muted)' }}>
            {t('hardware.subtitle')}
          </p>
        </div>
      </div>

      {/* Tab Navigation */}
      <div
        className="flex items-center gap-1 p-1 rounded-2xl w-fit"
        style={{ background: 'var(--pc-bg-elevated)' }}
      >
        {TABS.map(({ id, labelKey, icon: Icon }) => (
          <button
            key={id}
            onClick={() => setActiveTab(id)}
            className="flex items-center gap-2 px-4 py-2.5 rounded-xl text-sm font-medium transition-all"
            style={
              activeTab === id
                ? { background: 'var(--pc-bg-primary)', color: 'var(--pc-accent)', boxShadow: '0 1px 3px rgba(0,0,0,0.1)' }
                : { background: 'transparent', color: 'var(--pc-text-muted)' }
            }
            onMouseEnter={(e) => {
              if (activeTab !== id) {
                e.currentTarget.style.color = 'var(--pc-text-primary)';
              }
            }}
            onMouseLeave={(e) => {
              if (activeTab !== id) {
                e.currentTarget.style.color = 'var(--pc-text-muted)';
              }
            }}
          >
            <Icon className="h-4 w-4" />
            {t(labelKey)}
          </button>
        ))}
      </div>

      {/* Tab Content */}
      {activeTab === 'boards'  && <BoardsTab />}
      {activeTab === 'gpio'    && <GpioTab />}
      {activeTab === 'context' && <ContextTab />}
    </div>
  );
}
