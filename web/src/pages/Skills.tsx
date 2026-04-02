import { useState, useEffect, useMemo, useCallback } from 'react';
import {
  Sparkles,
  Search,
  Save,
  FolderOpen,
  RefreshCw,
  X,
  Info,
} from 'lucide-react';
import { getSkills, putSkills, type SkillsResponse, type SkillRow } from '@/lib/api';
import { t } from '@/lib/i18n';

function setsEqual(a: Set<string>, b: Set<string>): boolean {
  if (a.size !== b.size) return false;
  for (const x of a) {
    if (!b.has(x)) return false;
  }
  return true;
}

export default function Skills() {
  const [data, setData] = useState<SkillsResponse | null>(null);
  const [disabledNames, setDisabledNames] = useState<Set<string>>(new Set());
  const [baselineDisabled, setBaselineDisabled] = useState<Set<string>>(new Set());
  const [search, setSearch] = useState('');
  const [loading, setLoading] = useState(true);
  const [saving, setSaving] = useState(false);
  const [error, setError] = useState<string | null>(null);
  const [savedOk, setSavedOk] = useState(false);
  const [selectedSkill, setSelectedSkill] = useState<SkillRow | null>(null);

  const load = useCallback(() => {
    setLoading(true);
    setError(null);
    getSkills()
      .then((res) => {
        setData(res);
        const d = new Set(res.disabled_skills);
        setDisabledNames(d);
        setBaselineDisabled(new Set(d));
      })
      .catch((err: unknown) => setError(err instanceof Error ? err.message : String(err)))
      .finally(() => setLoading(false));
  }, []);

  useEffect(() => {
    load();
  }, [load]);

  const dirty = useMemo(
    () => !setsEqual(disabledNames, baselineDisabled),
    [disabledNames, baselineDisabled],
  );

  const toggleSkill = (name: string) => {
    setDisabledNames((prev) => {
      const next = new Set(prev);
      if (next.has(name)) next.delete(name);
      else next.add(name);
      return next;
    });
    setSavedOk(false);
  };

  const handleSave = async () => {
    setSaving(true);
    setError(null);
    setSavedOk(false);
    try {
      await putSkills({ disabled_skills: [...disabledNames] });
      setBaselineDisabled(new Set(disabledNames));
      setSavedOk(true);
      const refreshed = await getSkills();
      setData(refreshed);
    } catch (e: unknown) {
      setError(e instanceof Error ? e.message : String(e));
    } finally {
      setSaving(false);
    }
  };

  const filtered = useMemo(() => {
    if (!data?.skills) return [];
    const q = search.trim().toLowerCase();
    if (!q) return data.skills;
    return data.skills.filter(
      (s) =>
        s.name.toLowerCase().includes(q) ||
        s.description.toLowerCase().includes(q) ||
        (s.tags ?? []).some((tag) => tag.toLowerCase().includes(q)),
    );
  }, [data, search]);

  if (error && !data) {
    return (
      <div className="p-6 animate-fade-in">
        <div
          className="rounded-2xl border p-4"
          style={{
            background: 'rgba(239, 68, 68, 0.08)',
            borderColor: 'rgba(239, 68, 68, 0.2)',
            color: '#f87171',
          }}
        >
          {t('skills.load_error')}: {error}
        </div>
      </div>
    );
  }

  if (loading && !data) {
    return (
      <div className="flex items-center justify-center h-64">
        <div
          className="h-8 w-8 border-2 rounded-full animate-spin"
          style={{ borderColor: 'var(--pc-border)', borderTopColor: 'var(--pc-accent)' }}
        />
      </div>
    );
  }

  return (
    <div className="p-6 space-y-6 animate-fade-in">
      <div className="flex flex-wrap items-center justify-between gap-4">
        <div className="flex items-center gap-2">
          <Sparkles className="h-5 w-5" style={{ color: 'var(--pc-accent)' }} />
          <h1 className="text-sm font-semibold uppercase tracking-wider" style={{ color: 'var(--pc-text-primary)' }}>
            {t('skills.title')}
          </h1>
        </div>
        <div className="flex items-center gap-2">
          <button
            type="button"
            onClick={() => load()}
            disabled={loading}
            className="btn-electric flex items-center gap-2 text-sm px-3 py-2"
            style={{ opacity: loading ? 0.6 : 1 }}
          >
            <RefreshCw className={`h-4 w-4 ${loading ? 'animate-spin' : ''}`} />
            {t('skills.reload')}
          </button>
          <button
            type="button"
            onClick={handleSave}
            disabled={saving || !dirty}
            className="btn-electric flex items-center gap-2 text-sm px-4 py-2"
          >
            <Save className="h-4 w-4" />
            {saving ? t('skills.saving') : t('skills.save')}
          </button>
        </div>
      </div>

      {savedOk && (
        <div
          className="rounded-xl p-3 border text-sm animate-fade-in"
          style={{ borderColor: 'rgba(0,230,138,0.2)', background: 'rgba(0,230,138,0.06)', color: 'var(--color-status-success)' }}
        >
          {t('skills.save_success')}
        </div>
      )}

      {error && data && (
        <div
          className="rounded-xl p-3 border text-sm"
          style={{ borderColor: 'rgba(239,68,68,0.2)', background: 'rgba(239,68,68,0.06)', color: 'var(--color-status-error)' }}
        >
          {error}
        </div>
      )}

      {data && (
        <div
          className="rounded-2xl border p-4 flex flex-col gap-2 text-sm"
          style={{ borderColor: 'var(--pc-border)', background: 'var(--pc-bg-elevated)' }}
        >
          <div className="flex items-start gap-2" style={{ color: 'var(--pc-text-muted)' }}>
            <FolderOpen className="h-4 w-4 mt-0.5 shrink-0" />
            <div>
              <p className="font-medium" style={{ color: 'var(--pc-text-primary)' }}>{t('skills.workspace_dir')}</p>
              <p className="font-mono text-xs break-all mt-1">{data.workspace_skills_dir}</p>
              <p className="text-xs mt-2">
                {t('skills.open_skills')}: {data.open_skills_enabled ? t('skills.on') : t('skills.off')} · {t('skills.allow_scripts')}:{' '}
                {data.allow_scripts ? t('skills.on') : t('skills.off')}
              </p>
            </div>
          </div>
        </div>
      )}

      <div className="relative max-w-md">
        <Search className="absolute left-3 top-1/2 -translate-y-1/2 h-4 w-4" style={{ color: 'var(--pc-text-faint)' }} />
        <input
          type="text"
          value={search}
          onChange={(e) => setSearch(e.target.value)}
          placeholder={t('skills.search')}
          className="input-electric w-full pl-10 pr-4 py-2.5 text-sm"
        />
      </div>

      {!data?.skills.length ? (
        <p className="text-sm" style={{ color: 'var(--pc-text-muted)' }}>{t('skills.empty')}</p>
      ) : (
        <div className="grid grid-cols-1 md:grid-cols-2 xl:grid-cols-3 gap-4 stagger-children">
          {filtered.map((skill) => {
            const enabled = !disabledNames.has(skill.name);
            const key = skill.path ?? skill.name;
            return (
              <div
                key={key}
                className="card rounded-2xl p-4 border flex flex-col gap-3 transition-all duration-150 hover:border-opacity-100"
                style={{ borderColor: 'var(--pc-border)', cursor: 'pointer' }}
                onClick={() => setSelectedSkill(skill)}
                onMouseEnter={(e) => {
                  (e.currentTarget as HTMLElement).style.borderColor = 'var(--pc-accent)';
                  (e.currentTarget as HTMLElement).style.boxShadow = '0 0 0 1px var(--pc-accent)';
                }}
                onMouseLeave={(e) => {
                  (e.currentTarget as HTMLElement).style.borderColor = 'var(--pc-border)';
                  (e.currentTarget as HTMLElement).style.boxShadow = 'none';
                }}
              >
                <div className="flex items-start justify-between gap-2">
                  <div className="min-w-0">
                    <h3 className="text-sm font-semibold truncate" style={{ color: 'var(--pc-text-primary)' }}>
                      {skill.name}
                    </h3>
                    <p className="text-xs mt-1 line-clamp-3" style={{ color: 'var(--pc-text-muted)' }}>
                      {skill.description || '—'}
                    </p>
                  </div>
                  <label className="relative inline-flex items-center cursor-pointer shrink-0">
                    <input
                      type="checkbox"
                      checked={enabled}
                      onChange={() => toggleSkill(skill.name)}
                      onClick={(e) => e.stopPropagation()}
                      className="sr-only peer"
                    />
                    <div
                      className="w-9 h-5 rounded-full peer transition-colors"
                      style={{ backgroundColor: enabled ? 'var(--pc-accent)' : 'var(--pc-border)' }}
                    />
                    <div
                      className="absolute left-0.5 top-0.5 w-4 h-4 bg-white rounded-full transition-transform"
                      style={{ transform: enabled ? 'translateX(16px)' : 'translateX(0)' }}
                    />
                  </label>
                </div>
                <div className="flex flex-wrap gap-2 text-[10px] uppercase tracking-wider" style={{ color: 'var(--pc-text-faint)' }}>
                  <span>v{skill.version}</span>
                  <span>·</span>
                  <span>{t('skills.tools')}: {skill.tools_count}</span>
                  <span>·</span>
                  <span>{t('skills.prompts')}: {skill.prompts_count}</span>
                </div>
                {skill.path && (
                  <p className="text-[10px] font-mono truncate" style={{ color: 'var(--pc-text-faint)' }} title={skill.path}>
                    {skill.path}
                  </p>
                )}
              </div>
            );
          })}
        </div>
      )}

      {data && filtered.length === 0 && data.skills.length > 0 && (
        <p className="text-sm" style={{ color: 'var(--pc-text-muted)' }}>{t('skills.no_match')}</p>
      )}

      {selectedSkill && (
        <SkillDetailPanel skill={selectedSkill} onClose={() => setSelectedSkill(null)} />
      )}
    </div>
  );
}

function SkillDetailPanel({ skill, onClose }: { skill: SkillRow; onClose: () => void }) {
  return (
    <>
      <div
        className="fixed inset-0 z-40 bg-black/40 animate-fade-in"
        onClick={onClose}
      />
      <div
        className="fixed right-0 top-0 bottom-0 z-50 w-full max-w-lg shadow-2xl animate-slide-in-right overflow-y-auto"
        style={{
          background: 'var(--pc-bg-elevated)',
          borderLeft: '1px solid var(--pc-border)',
        }}
      >
        <div className="sticky top-0 z-10 flex items-center justify-between p-4 border-b backdrop-blur-sm"
          style={{ borderColor: 'var(--pc-border)', background: 'var(--pc-bg-elevated)' }}>
          <div className="flex items-center gap-2 min-w-0">
            <Sparkles className="h-4 w-4 shrink-0" style={{ color: 'var(--pc-accent)' }} />
            <h2 className="text-sm font-semibold truncate" style={{ color: 'var(--pc-text-primary)' }}>
              {skill.name}
            </h2>
          </div>
          <button
            type="button"
            onClick={onClose}
            className="p-1.5 rounded-lg transition-colors shrink-0"
            style={{ color: 'var(--pc-text-muted)' }}
            onMouseEnter={(e) => (e.currentTarget.style.background = 'rgba(255,255,255,0.06)')}
            onMouseLeave={(e) => (e.currentTarget.style.background = 'transparent')}
          >
            <X className="h-4 w-4" />
          </button>
        </div>

        <div className="p-5 space-y-5">
          {skill.description && (
            <div>
              <p className="text-xs uppercase tracking-wider font-semibold mb-2" style={{ color: 'var(--pc-text-faint)' }}>
                {t('skills.detail.description')}
              </p>
              <p className="text-sm leading-relaxed" style={{ color: 'var(--pc-text-primary)' }}>
                {skill.description}
              </p>
            </div>
          )}

          <div className="grid grid-cols-2 gap-3">
            <div className="rounded-xl p-3 border" style={{ borderColor: 'var(--pc-border)', background: 'var(--pc-bg)' }}>
              <p className="text-[10px] uppercase tracking-wider mb-1" style={{ color: 'var(--pc-text-faint)' }}>
                {t('skills.detail.version')}
              </p>
              <p className="text-sm font-semibold" style={{ color: 'var(--pc-accent)' }}>v{skill.version}</p>
            </div>
            <div className="rounded-xl p-3 border" style={{ borderColor: 'var(--pc-border)', background: 'var(--pc-bg)' }}>
              <p className="text-[10px] uppercase tracking-wider mb-1" style={{ color: 'var(--pc-text-faint)' }}>
                {t('skills.detail.tools')}
              </p>
              <p className="text-sm font-semibold" style={{ color: 'var(--pc-accent)' }}>{skill.tools_count}</p>
            </div>
            <div className="rounded-xl p-3 border" style={{ borderColor: 'var(--pc-border)', background: 'var(--pc-bg)' }}>
              <p className="text-[10px] uppercase tracking-wider mb-1" style={{ color: 'var(--pc-text-faint)' }}>
                {t('skills.detail.prompts')}
              </p>
              <p className="text-sm font-semibold" style={{ color: 'var(--pc-accent)' }}>{skill.prompts_count}</p>
            </div>
            <div className="rounded-xl p-3 border" style={{ borderColor: 'var(--pc-border)', background: 'var(--pc-bg)' }}>
              <p className="text-[10px] uppercase tracking-wider mb-1" style={{ color: 'var(--pc-text-faint)' }}>
                {t('skills.detail.author')}
              </p>
              <p className="text-sm font-semibold truncate" style={{ color: 'var(--pc-accent)' }}>
                {skill.author || '—'}
              </p>
            </div>
          </div>

          {skill.tags.length > 0 && (
            <div>
              <p className="text-xs uppercase tracking-wider font-semibold mb-2" style={{ color: 'var(--pc-text-faint)' }}>
                {t('skills.detail.tags')}
              </p>
              <div className="flex flex-wrap gap-2">
                {skill.tags.map((tag) => (
                  <span
                    key={tag}
                    className="inline-flex items-center px-2.5 py-1 rounded-full text-xs font-medium"
                    style={{
                      background: 'rgba(124,58,237,0.12)',
                      color: '#a78bfa',
                      border: '1px solid rgba(124,58,237,0.2)',
                    }}
                  >
                    {tag}
                  </span>
                ))}
              </div>
            </div>
          )}

          {skill.path && (
            <div>
              <p className="text-xs uppercase tracking-wider font-semibold mb-2" style={{ color: 'var(--pc-text-faint)' }}>
                {t('skills.detail.path')}
              </p>
              <div className="rounded-xl p-3 border font-mono text-xs break-all"
                style={{ borderColor: 'var(--pc-border)', background: 'var(--pc-bg)', color: 'var(--pc-text-muted)' }}>
                {skill.path}
              </div>
            </div>
          )}

          <div className="rounded-xl p-4 border flex items-start gap-3"
            style={{ borderColor: 'var(--pc-border)', background: 'rgba(124,58,237,0.04)' }}>
            <Info className="h-4 w-4 mt-0.5 shrink-0" style={{ color: 'var(--pc-accent)' }} />
            <p className="text-xs leading-relaxed" style={{ color: 'var(--pc-text-muted)' }}>
              {skill.enabled ? t('skills.detail.enabled_tip') : t('skills.detail.disabled_tip')}
            </p>
          </div>
        </div>
      </div>
    </>
  );
}
