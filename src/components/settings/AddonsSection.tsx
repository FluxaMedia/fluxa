import React, { useState } from 'react';
import { open as shellOpen } from '@tauri-apps/plugin-shell';
import { addonKey, addonLogo, addonName, addonTypes, addonVersion } from '../../core/addons';
import type { AddonDescriptor } from '../../core/types';
import { t } from '../../i18n';
import { SettingsPanel, SettingsSection } from './SettingsUI';
import type { Prefs } from './settingsTypes';

const ADDON_AVATAR_PALETTES: [string, string][] = [
  ['#7C3AED', '#4C1D95'],
  ['#0369A1', '#082F49'],
  ['#047857', '#022C22'],
  ['#B45309', '#451A03'],
  ['#BE185D', '#500724'],
  ['#0F766E', '#042F2E'],
  ['#6D28D9', '#2E1065'],
  ['#B91C1C', '#450A0A'],
];

function addonAvatarPalette(name: string): [string, string] {
  let h = 0;
  for (let i = 0; i < name.length; i++) h = (h * 31 + name.charCodeAt(i)) >>> 0;
  return ADDON_AVATAR_PALETTES[h % ADDON_AVATAR_PALETTES.length];
}

function AddonToggle({ enabled, onChange }: { enabled: boolean; onChange: () => void }) {
  const [hovered, setHovered] = useState(false);
  return (
    <button
      onClick={onChange}
      onMouseEnter={() => setHovered(true)}
      onMouseLeave={() => setHovered(false)}
      style={{
        width: 44, height: 26, borderRadius: 13, border: 'none', cursor: 'pointer', padding: 0, flexShrink: 0,
        background: enabled ? (hovered ? '#e0e0e0' : '#ffffff') : (hovered ? 'rgba(255,255,255,0.18)' : 'rgba(255,255,255,0.12)'),
        transition: 'background 0.18s', position: 'relative',
      }}
    >
      <div style={{ position: 'absolute', top: 3, left: enabled ? 21 : 3, width: 20, height: 20, borderRadius: '50%', background: enabled ? '#111' : 'rgba(255,255,255,0.55)', transition: 'left 0.18s' }} />
    </button>
  );
}

function AddonIconBtn({ title, disabled, onClick, children }: { title: string; disabled?: boolean; onClick: () => void; children: React.ReactNode }) {
  const [hovered, setHovered] = useState(false);
  return (
    <button
      title={title}
      onClick={onClick}
      disabled={disabled}
      onMouseEnter={() => setHovered(true)}
      onMouseLeave={() => setHovered(false)}
      style={{
        width: 32, height: 32, borderRadius: 8, border: 'none',
        background: hovered && !disabled ? 'rgba(255,255,255,0.09)' : 'transparent',
        color: disabled ? 'rgba(255,255,255,0.18)' : (hovered ? 'rgba(255,255,255,0.9)' : 'rgba(255,255,255,0.45)'),
        cursor: disabled ? 'default' : 'pointer',
        display: 'flex', alignItems: 'center', justifyContent: 'center',
        transition: 'background 0.12s, color 0.12s', padding: 0, flexShrink: 0,
      }}
    >{children}</button>
  );
}

function AddonTile({
  addon, enabled, isFirst, isLast, onRemove, onToggle, onMoveUp, onMoveDown, onRefresh,
}: {
  addon: AddonDescriptor; enabled: boolean; isFirst: boolean; isLast: boolean;
  onRemove: () => void; onToggle: () => void; onMoveUp: () => void; onMoveDown: () => void; onRefresh: () => void;
}) {
  const [removeHovered, setRemoveHovered] = useState(false);
  const name = addonName(addon);
  const logo = addonLogo(addon);
  const version = addonVersion(addon);
  const types = addonTypes(addon);
  const description = addon.manifest?.description ?? addon.description ?? null;
  const resourceCount = (addon.manifest?.resources ?? addon.resources ?? []).length;
  const isConfigurable = addon.manifest?.configurable ?? addon.behaviorHints?.configurable ?? false;
  const [fg, bg] = addonAvatarPalette(name);

  const handleConfigure = () => {
    const base = (addon.transportUrl ?? '').replace(/\/manifest\.json$/, '');
    if (base) shellOpen(`${base}/configure`).catch(() => {});
  };

  return (
    <div style={{ background: '#15191f', border: '1px solid rgba(255,255,255,0.07)', borderRadius: 14, padding: '14px 16px', boxSizing: 'border-box', opacity: enabled ? 1 : 0.6, transition: 'opacity 0.2s' }}>
      <div style={{ display: 'flex', alignItems: 'center', gap: 13, marginBottom: 12 }}>
        {logo ? (
          <img src={logo} alt="" style={{ width: 52, height: 52, borderRadius: 12, objectFit: 'contain', background: '#1c2130', flexShrink: 0, border: '1px solid rgba(255,255,255,0.07)' }} />
        ) : (
          <div style={{ width: 52, height: 52, borderRadius: 12, flexShrink: 0, background: `linear-gradient(140deg, ${fg}dd 0%, ${bg} 100%)`, display: 'flex', alignItems: 'center', justifyContent: 'center', border: `1px solid ${fg}33` }}>
            <span style={{ color: '#fff', fontSize: 18, fontWeight: 800, fontFamily: 'sans-serif', letterSpacing: '-0.5px' }}>{name.slice(0, 2).toUpperCase()}</span>
          </div>
        )}
        <div style={{ flex: 1, minWidth: 0 }}>
          <p style={{ color: 'rgba(255,255,255,0.92)', fontSize: 15, fontWeight: 700, margin: 0, fontFamily: 'sans-serif', overflow: 'hidden', textOverflow: 'ellipsis', whiteSpace: 'nowrap' }}>{name}</p>
          {version && <p style={{ color: 'rgba(255,255,255,0.35)', fontSize: 12, margin: '2px 0 0', fontFamily: 'sans-serif' }}>Version {version}</p>}
        </div>
        <AddonToggle enabled={enabled} onChange={onToggle} />
      </div>

      <div style={{ display: 'flex', alignItems: 'center', gap: 2, marginBottom: 10 }}>
        {isConfigurable && (
          <AddonIconBtn title="Configure" onClick={handleConfigure}>
            <svg width="15" height="15" viewBox="0 0 24 24" fill="none" stroke="currentColor" strokeWidth="2" strokeLinecap="round" strokeLinejoin="round">
              <circle cx="12" cy="12" r="3"/><path d="M19.4 15a1.65 1.65 0 0 0 .33 1.82l.06.06a2 2 0 0 1-2.83 2.83l-.06-.06a1.65 1.65 0 0 0-1.82-.33 1.65 1.65 0 0 0-1 1.51V21a2 2 0 0 1-4 0v-.09A1.65 1.65 0 0 0 9 19.4a1.65 1.65 0 0 0-1.82.33l-.06.06a2 2 0 0 1-2.83-2.83l.06-.06A1.65 1.65 0 0 0 4.68 15a1.65 1.65 0 0 0-1.51-1H3a2 2 0 0 1 0-4h.09A1.65 1.65 0 0 0 4.6 9a1.65 1.65 0 0 0-.33-1.82l-.06-.06a2 2 0 0 1 2.83-2.83l.06.06A1.65 1.65 0 0 0 9 4.68a1.65 1.65 0 0 0 1-1.51V3a2 2 0 0 1 4 0v.09a1.65 1.65 0 0 0 1 1.51 1.65 1.65 0 0 0 1.82-.33l.06-.06a2 2 0 0 1 2.83 2.83l-.06.06A1.65 1.65 0 0 0 19.4 9a1.65 1.65 0 0 0 1.51 1H21a2 2 0 0 1 0 4h-.09a1.65 1.65 0 0 0-1.51 1z"/>
            </svg>
          </AddonIconBtn>
        )}
        <AddonIconBtn title="Refresh" onClick={onRefresh}>
          <svg width="15" height="15" viewBox="0 0 24 24" fill="none" stroke="currentColor" strokeWidth="2" strokeLinecap="round" strokeLinejoin="round">
            <polyline points="23 4 23 10 17 10"/><polyline points="1 20 1 14 7 14"/><path d="M3.51 9a9 9 0 0 1 14.85-3.36L23 10M1 14l4.64 4.36A9 9 0 0 0 20.49 15"/>
          </svg>
        </AddonIconBtn>
        <AddonIconBtn title="Move up" disabled={isFirst} onClick={onMoveUp}>
          <svg width="15" height="15" viewBox="0 0 24 24" fill="none" stroke="currentColor" strokeWidth="2.2" strokeLinecap="round" strokeLinejoin="round"><polyline points="18 15 12 9 6 15"/></svg>
        </AddonIconBtn>
        <AddonIconBtn title="Move down" disabled={isLast} onClick={onMoveDown}>
          <svg width="15" height="15" viewBox="0 0 24 24" fill="none" stroke="currentColor" strokeWidth="2.2" strokeLinecap="round" strokeLinejoin="round"><polyline points="6 9 12 15 18 9"/></svg>
        </AddonIconBtn>
        <div style={{ flex: 1 }} />
        <button
          title={t('common.forget')}
          onClick={onRemove}
          onMouseEnter={() => setRemoveHovered(true)}
          onMouseLeave={() => setRemoveHovered(false)}
          style={{ background: removeHovered ? 'rgba(255,60,60,0.14)' : 'transparent', border: 'none', color: removeHovered ? 'rgba(255,100,100,0.9)' : 'rgba(255,255,255,0.22)', width: 32, height: 32, borderRadius: 8, display: 'flex', alignItems: 'center', justifyContent: 'center', cursor: 'pointer', transition: 'background 0.12s, color 0.12s', padding: 0, flexShrink: 0 }}
        >
          <svg width="14" height="14" viewBox="0 0 24 24" fill="none" stroke="currentColor" strokeWidth="2" strokeLinecap="round" strokeLinejoin="round">
            <polyline points="3 6 5 6 21 6"/><path d="M19 6l-1 14a2 2 0 01-2 2H8a2 2 0 01-2-2L5 6"/><path d="M10 11v6"/><path d="M14 11v6"/><path d="M9 6V4h6v2"/>
          </svg>
        </button>
      </div>

      {description && (
        <p style={{ color: 'rgba(255,255,255,0.42)', fontSize: 12.5, margin: '0 0 10px', fontFamily: 'sans-serif', lineHeight: 1.5, display: '-webkit-box', WebkitLineClamp: 2, WebkitBoxOrient: 'vertical', overflow: 'hidden' }}>{description}</p>
      )}

      <div style={{ display: 'flex', flexWrap: 'wrap', gap: 6 }}>
        <span style={{ fontSize: 11, fontWeight: 600, fontFamily: 'sans-serif', color: enabled ? 'rgba(255,255,255,0.75)' : 'rgba(255,255,255,0.35)', background: enabled ? 'rgba(255,255,255,0.1)' : 'rgba(255,255,255,0.05)', border: `1px solid ${enabled ? 'rgba(255,255,255,0.14)' : 'rgba(255,255,255,0.07)'}`, borderRadius: 6, padding: '3px 8px' }}>{enabled ? 'Active' : 'Inactive'}</span>
        {resourceCount > 0 && <span style={{ fontSize: 11, fontWeight: 600, fontFamily: 'sans-serif', color: 'rgba(255,255,255,0.55)', background: 'rgba(255,255,255,0.06)', border: '1px solid rgba(255,255,255,0.08)', borderRadius: 6, padding: '3px 8px' }}>{resourceCount} resource{resourceCount !== 1 ? 's' : ''}</span>}
        {isConfigurable && <span style={{ fontSize: 11, fontWeight: 600, fontFamily: 'sans-serif', color: 'rgba(255,255,255,0.55)', background: 'rgba(255,255,255,0.06)', border: '1px solid rgba(255,255,255,0.08)', borderRadius: 6, padding: '3px 8px' }}>Configurable</span>}
        {types.slice(0, 4).map((type) => (
          <span key={type} style={{ fontSize: 11, fontWeight: 600, fontFamily: 'sans-serif', color: 'rgba(255,255,255,0.35)', background: 'rgba(255,255,255,0.04)', border: '1px solid rgba(255,255,255,0.06)', borderRadius: 6, padding: '3px 8px', textTransform: 'capitalize' }}>{type}</span>
        ))}
      </div>
    </div>
  );
}

export function AddonsSection({
  prefs: _prefs,
  setPref: _setPref,
  addonUrl,
  setAddonUrl,
  installedAddons,
  disabledAddonKeys,
  installLoading,
  installError,
  onInstall,
  onRemove,
  onToggle,
  onReorder,
  onDispatch,
}: {
  prefs: Prefs;
  setPref: <K extends keyof Prefs>(k: K, v: Prefs[K]) => void;
  addonUrl: string;
  setAddonUrl: (v: string) => void;
  installedAddons: AddonDescriptor[];
  disabledAddonKeys: string[];
  installLoading: boolean;
  installError: string | null;
  onInstall: () => void;
  onRemove: (a: AddonDescriptor) => void;
  onToggle: (a: AddonDescriptor) => void;
  onReorder: (a: AddonDescriptor, dir: 'up' | 'down') => void;
  onDispatch: (actionJson: string) => void;
}) {
  return (
    <>
      <SettingsSection title={t('addons.install')} subtitle={t('auto.installed_add_ons_and_settings')}>
        <SettingsPanel>
          <p style={{ color: 'rgba(255,255,255,0.4)', fontSize: 12, margin: '0 0 8px', fontFamily: 'sans-serif', letterSpacing: '0.8px', textTransform: 'uppercase', fontWeight: 700 }}>{t('addons.manifest_url')}</p>
          <input
            type="text"
            value={addonUrl}
            onChange={(e) => setAddonUrl(e.target.value)}
            placeholder={t('addons.install_placeholder')}
            onKeyDown={(e) => e.key === 'Enter' && onInstall()}
            disabled={installLoading}
            style={{ width: '100%', boxSizing: 'border-box', background: 'rgba(255,255,255,0.05)', border: '1px solid rgba(255,255,255,0.12)', borderRadius: 10, padding: '13px 14px', color: '#FFFFFF', fontSize: 14, fontFamily: 'sans-serif', outline: 'none', marginBottom: 10, opacity: installLoading ? 0.7 : 1 }}
          />
          {installError && (
            <p style={{ color: '#FF6B6B', fontSize: 12, lineHeight: 1.45, margin: '0 0 10px', fontFamily: 'sans-serif' }}>{installError}</p>
          )}
          <button
            style={{ alignSelf: 'flex-start', background: addonUrl.trim() && !installLoading ? '#FFFFFF' : 'rgba(255,255,255,0.12)', color: addonUrl.trim() && !installLoading ? '#000000' : 'rgba(255,255,255,0.4)', border: 'none', borderRadius: 999, padding: '9px 20px', fontSize: 13, fontWeight: 700, cursor: addonUrl.trim() && !installLoading ? 'pointer' : 'default', fontFamily: 'sans-serif', transition: 'background 0.15s, color 0.15s' }}
            onClick={onInstall}
            disabled={!addonUrl.trim() || installLoading}
          >
            {installLoading ? 'Installing...' : t('addons.install')}
          </button>
        </SettingsPanel>
      </SettingsSection>

      {installedAddons.length > 0 ? (
        <SettingsSection title={`${t('addons.installed')} (${installedAddons.length})`} subtitle={t('auto.installed_add_ons_and_settings')}>
          <div style={{ display: 'flex', flexDirection: 'column', gap: 10, padding: '4px 0' }}>
            {installedAddons.map((addon, idx) => (
              <AddonTile
                key={addonKey(addon)}
                addon={addon}
                enabled={!disabledAddonKeys.includes(addonKey(addon))}
                isFirst={idx === 0}
                isLast={idx === installedAddons.length - 1}
                onRemove={() => onRemove(addon)}
                onToggle={() => onToggle(addon)}
                onMoveUp={() => onReorder(addon, 'up')}
                onMoveDown={() => onReorder(addon, 'down')}
                onRefresh={() => onDispatch(JSON.stringify({ type: 'addonsRefreshRequested', forceRefresh: true }))}
              />
            ))}
          </div>
        </SettingsSection>
      ) : (
        <SettingsPanel>
          <p style={{ color: 'rgba(255,255,255,0.35)', fontSize: 14, margin: 0, fontFamily: 'sans-serif' }}>{t('addons.no_addons')}</p>
        </SettingsPanel>
      )}
    </>
  );
}
