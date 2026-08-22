import React from 'react';
import type { PluginRepository, PluginScraper } from '../../core/types';
import { t } from '../../i18n';
import { ExtensionIcon, SettingsSection } from './SettingsUI';
import { FONT } from './settingsStyles';

function Toggle({ enabled, onClick }: { enabled: boolean; onClick: () => void }) {
  return (
    <button
      onClick={onClick}
      aria-label={enabled ? t('plugins.disable') : t('plugins.enable')}
      style={{
        width: '2.75rem',
        height: '1.625rem',
        borderRadius: '0.8125rem',
        border: 'none',
        padding: 0,
        cursor: 'pointer',
        background: enabled ? '#fff' : 'rgba(255,255,255,0.14)',
        position: 'relative',
        flexShrink: 0,
      }}
    >
      <span
        style={{
          position: 'absolute',
          top: '0.1875rem',
          left: enabled ? 21 : 3,
          width: '1.25rem',
          height: '1.25rem',
          borderRadius: '50%',
          background: enabled ? '#111' : 'rgba(255,255,255,0.6)',
          transition: 'left 0.18s',
        }}
      />
    </button>
  );
}

function IconButton({
  title,
  onClick,
  destructive = false,
  children,
}: {
  title: string;
  onClick: () => void;
  destructive?: boolean;
  children: React.ReactNode;
}) {
  return (
    <button
      title={title}
      onClick={onClick}
      style={{
        width: '1.875rem',
        height: '1.875rem',
        border: 'none',
        borderRadius: '0.4375rem',
        padding: 0,
        background: 'transparent',
        color: destructive ? 'rgba(255,120,120,0.75)' : 'rgba(255,255,255,0.5)',
        cursor: 'pointer',
        display: 'grid',
        placeItems: 'center',
      }}
    >
      {children}
    </button>
  );
}

type PluginRepositoryGroup = {
  sourceUrl: string;
  sourceName: string;
  plugins: PluginRepository[];
};

function repositorySource(manifestUrl: string): { sourceUrl: string; sourceName: string } {
  try {
    const url = new URL(manifestUrl);
    const parts = url.pathname.split('/').filter(Boolean);
    if (url.hostname === 'raw.githubusercontent.com' && parts.length >= 2) {
      const sourceUrl = `https://github.com/${parts[0]}/${parts[1]}`;
      return { sourceUrl, sourceName: `${parts[0]}/${parts[1]}` };
    }
    if (url.hostname === 'github.com' && parts.length >= 2) {
      const sourceUrl = `${url.origin}/${parts[0]}/${parts[1]}`;
      return { sourceUrl, sourceName: `${parts[0]}/${parts[1]}` };
    }
    return { sourceUrl: url.origin, sourceName: url.hostname };
  } catch {
    return { sourceUrl: manifestUrl, sourceName: manifestUrl };
  }
}

function groupPluginRepositories(repositories: PluginRepository[]): PluginRepositoryGroup[] {
  const groups = new Map<string, PluginRepositoryGroup>();
  for (const plugin of repositories) {
    const source = repositorySource(plugin.manifestUrl);
    const group = groups.get(source.sourceUrl) ?? { ...source, plugins: [] };
    group.plugins.push(plugin);
    groups.set(source.sourceUrl, group);
  }
  return [...groups.values()];
}

export function PluginsSection({
  pluginUrl,
  setPluginUrl,
  repositories,
  scrapers,
  loading,
  error,
  onInstall,
  onRemove,
  onRefresh,
  onToggleScraper,
}: {
  pluginUrl: string;
  setPluginUrl: (value: string) => void;
  repositories: PluginRepository[];
  scrapers: PluginScraper[];
  loading: boolean;
  error: string | null;
  onInstall: () => void;
  onRemove: (repository: PluginRepository) => void;
  onRefresh: (repository: PluginRepository) => void;
  onToggleScraper: (scraper: PluginScraper) => void;
}) {
  const repositoryGroups = groupPluginRepositories(repositories);
  const [expandedRepositoryUrls, setExpandedRepositoryUrls] = React.useState<Set<string>>(() => new Set());
  const toggleRepository = (sourceUrl: string) => {
    setExpandedRepositoryUrls((current) => {
      const next = new Set(current);
      if (next.has(sourceUrl)) next.delete(sourceUrl);
      else next.add(sourceUrl);
      return next;
    });
  };
  return (
    <>
      <SettingsSection title={t('plugins.install')} subtitle={t('plugins.install_description')}>
        <div
          style={{
            padding: '0.875rem 1rem',
            borderBottom: '1px solid rgba(255,255,255,0.055)',
          }}
        >
          <input
            value={pluginUrl}
            onChange={(event) => setPluginUrl(event.target.value)}
            onKeyDown={(event) => event.key === 'Enter' && onInstall()}
            disabled={loading}
            placeholder={t('plugins.install_placeholder')}
            style={{
              width: '100%',
              boxSizing: 'border-box',
              background: 'rgba(255,255,255,0.05)',
              border: '1px solid rgba(255,255,255,0.10)',
              borderRadius: '0.5rem',
              padding: '0.6875rem 0.8125rem',
              color: '#fff',
              fontSize: '0.8125rem',
              fontFamily: FONT,
              outline: 'none',
              marginBottom: error ? 8 : 10,
            }}
          />
          {error && (
            <p
              style={{
                color: '#FF6B6B',
                fontSize: '0.75rem',
                margin: '0 0 0.625rem',
                fontFamily: FONT,
              }}
            >
              {error}
            </p>
          )}
          <button
            onClick={onInstall}
            disabled={!pluginUrl.trim() || loading}
            style={{
              background: pluginUrl.trim() && !loading ? '#fff' : 'rgba(255,255,255,0.10)',
              color: pluginUrl.trim() && !loading ? '#000' : 'rgba(255,255,255,0.35)',
              border: 'none',
              borderRadius: '0.5rem',
              padding: '0.5rem 1.125rem',
              fontSize: '0.8125rem',
              fontWeight: 500,
              fontFamily: FONT,
              cursor: pluginUrl.trim() && !loading ? 'pointer' : 'default',
            }}
          >
            {loading ? t('plugins.installing') : t('plugins.install')}
          </button>
        </div>
      </SettingsSection>

      {repositoryGroups.length > 0 ? (
        <SettingsSection title={`${t('plugins.repositories')} (${repositoryGroups.length})`} subtitle={t('plugins.installed_description')}>
          <div
            style={{
              display: 'flex',
              flexDirection: 'column',
              gap: '0.5rem',
              padding: '0.5rem',
            }}
          >
            {repositoryGroups.map((group) => {
              const expanded = expandedRepositoryUrls.has(group.sourceUrl);
              return (
                <div
                  key={group.sourceUrl}
                  style={{
                    background: '#1A1A1A',
                    border: '1px solid rgba(255,255,255,0.09)',
                    borderRadius: '0.75rem',
                    overflow: 'hidden',
                  }}
                >
                  <div
                    role="button"
                    tabIndex={0}
                    onClick={() => toggleRepository(group.sourceUrl)}
                    onKeyDown={(event) => {
                      if (event.key === 'Enter' || event.key === ' ') {
                        event.preventDefault();
                        toggleRepository(group.sourceUrl);
                      }
                    }}
                    style={{
                      display: 'flex',
                      gap: '0.6875rem',
                      alignItems: 'center',
                      padding: '0.8125rem',
                      borderBottom: expanded ? '1px solid rgba(255,255,255,0.07)' : 'none',
                      cursor: 'pointer',
                    }}
                  >
                    <div
                      style={{
                        width: '2.5rem',
                        height: '2.5rem',
                        borderRadius: '0.625rem',
                        display: 'grid',
                        placeItems: 'center',
                        background: 'rgba(255,255,255,0.06)',
                        color: 'rgba(255,255,255,0.45)',
                        flexShrink: 0,
                      }}
                    >
                      <ExtensionIcon />
                    </div>
                    <div style={{ flex: 1, minWidth: 0 }}>
                      <p
                        style={{
                          color: 'rgba(255,255,255,0.38)',
                          fontSize: '0.625rem',
                          fontWeight: 700,
                          letterSpacing: '0.07em',
                          margin: 0,
                          fontFamily: FONT,
                        }}
                      >
                        {t('plugins.repository')}
                      </p>
                      <p
                        style={{
                          color: 'rgba(255,255,255,0.9)',
                          fontSize: '0.8125rem',
                          fontWeight: 650,
                          margin: '0.15rem 0 0',
                          fontFamily: FONT,
                        }}
                      >
                        {group.sourceName}
                      </p>
                    </div>
                    <a
                      href={group.sourceUrl}
                      target="_blank"
                      rel="noreferrer"
                      onClick={(event) => event.stopPropagation()}
                      style={{
                        color: 'rgba(255,255,255,0.38)',
                        fontSize: '0.6875rem',
                        fontFamily: FONT,
                        textDecoration: 'none',
                      }}
                    >
                      {group.plugins.length} {t('plugins.plugins')}
                    </a>
                    <svg
                      width="16"
                      height="16"
                      viewBox="0 0 24 24"
                      fill="none"
                      stroke="currentColor"
                      strokeWidth="2"
                      style={{
                        color: 'rgba(255,255,255,0.42)',
                        transition: 'transform 0.18s ease',
                        transform: expanded ? 'rotate(180deg)' : 'rotate(0deg)',
                        flexShrink: 0,
                      }}
                    >
                      <polyline points="6 9 12 15 18 9" />
                    </svg>
                  </div>
                  {expanded && (
                    <div
                      style={{
                        display: 'flex',
                        flexDirection: 'column',
                        gap: '0.5rem',
                        padding: '0.5rem',
                      }}
                    >
                      {group.plugins.map((plugin) => {
                        const providers = scrapers.filter((scraper) => scraper.repositoryUrl === plugin.manifestUrl);
                        return (
                          <div
                            key={plugin.manifestUrl}
                            style={{
                              background: 'rgba(0,0,0,0.2)',
                              border: '1px solid rgba(255,255,255,0.06)',
                              borderRadius: '0.625rem',
                              overflow: 'hidden',
                            }}
                          >
                            <div
                              style={{
                                display: 'flex',
                                gap: '0.625rem',
                                alignItems: 'center',
                                padding: '0.6875rem 0.75rem',
                              }}
                            >
                              <div style={{ flex: 1, minWidth: 0 }}>
                                <p
                                  style={{
                                    color: 'rgba(255,255,255,0.38)',
                                    fontSize: '0.625rem',
                                    fontWeight: 700,
                                    letterSpacing: '0.07em',
                                    margin: 0,
                                    fontFamily: FONT,
                                  }}
                                >
                                  {t('plugins.plugins')}
                                </p>
                                <p
                                  style={{
                                    color: 'rgba(255,255,255,0.92)',
                                    fontSize: '0.8125rem',
                                    fontWeight: 650,
                                    margin: '0.15rem 0 0',
                                    fontFamily: FONT,
                                  }}
                                >
                                  {plugin.name || t('plugins.unnamed')}
                                </p>
                                <p
                                  style={{
                                    color: 'rgba(255,255,255,0.32)',
                                    fontSize: '0.6875rem',
                                    margin: '0.125rem 0 0',
                                    fontFamily: FONT,
                                  }}
                                >
                                  {plugin.version || plugin.manifestUrl}
                                </p>
                              </div>
                              <IconButton title={t('common.refresh')} onClick={() => onRefresh(plugin)}>
                                <svg width="14" height="14" viewBox="0 0 24 24" fill="none" stroke="currentColor" strokeWidth="2">
                                  <polyline points="23 4 23 10 17 10" />
                                  <polyline points="1 20 1 14 7 14" />
                                  <path d="M3.51 9a9 9 0 0 1 14.85-3.36L23 10M1 14l4.64 4.36A9 9 0 0 0 20.49 15" />
                                </svg>
                              </IconButton>
                              <IconButton title={t('common.forget')} destructive onClick={() => onRemove(plugin)}>
                                <svg width="14" height="14" viewBox="0 0 24 24" fill="none" stroke="currentColor" strokeWidth="2">
                                  <polyline points="3 6 5 6 21 6" />
                                  <path d="M19 6l-1 14a2 2 0 0 1-2 2H8a2 2 0 0 1-2-2L5 6" />
                                  <path d="M10 11v6M14 11v6" />
                                </svg>
                              </IconButton>
                            </div>
                            {plugin.description && (
                              <p
                                style={{
                                  color: 'rgba(255,255,255,0.4)',
                                  fontSize: '0.75rem',
                                  lineHeight: 1.5,
                                  margin: '0 0.75rem 0.625rem',
                                  fontFamily: FONT,
                                }}
                              >
                                {plugin.description}
                              </p>
                            )}
                            <div
                              style={{
                                borderTop: '1px solid rgba(255,255,255,0.055)',
                              }}
                            >
                              <p
                                style={{
                                  color: 'rgba(255,255,255,0.34)',
                                  fontSize: '0.625rem',
                                  fontWeight: 700,
                                  letterSpacing: '0.07em',
                                  margin: '0',
                                  padding: '0.5625rem 0.75rem 0.375rem',
                                  fontFamily: FONT,
                                }}
                              >
                                {t('plugins.providers')} ({providers.length})
                              </p>
                              {providers.map((provider) => (
                                <div
                                  key={provider.id}
                                  style={{
                                    display: 'flex',
                                    gap: '0.625rem',
                                    alignItems: 'center',
                                    padding: '0.5rem 0.75rem 0.6875rem',
                                    opacity: provider.enabled ? 1 : 0.5,
                                  }}
                                >
                                  <div style={{ flex: 1, minWidth: 0 }}>
                                    <p
                                      style={{
                                        color: 'rgba(255,255,255,0.8)',
                                        fontSize: '0.75rem',
                                        margin: 0,
                                        fontFamily: FONT,
                                      }}
                                    >
                                      {provider.name}
                                    </p>
                                    <p
                                      style={{
                                        color: 'rgba(255,255,255,0.32)',
                                        fontSize: '0.6875rem',
                                        margin: '0.125rem 0 0',
                                        fontFamily: FONT,
                                      }}
                                    >
                                      {provider.supportedTypes?.join(', ') || t('plugins.all_media')}
                                    </p>
                                  </div>
                                  <Toggle enabled={provider.enabled} onClick={() => onToggleScraper(provider)} />
                                </div>
                              ))}
                              {providers.length === 0 && (
                                <p
                                  style={{
                                    color: 'rgba(255,255,255,0.3)',
                                    fontSize: '0.75rem',
                                    margin: 0,
                                    padding: '0 0.75rem 0.6875rem',
                                    fontFamily: FONT,
                                  }}
                                >
                                  {t('plugins.no_providers')}
                                </p>
                              )}
                            </div>
                          </div>
                        );
                      })}
                    </div>
                  )}
                </div>
              );
            })}
          </div>
        </SettingsSection>
      ) : (
        <div style={{ padding: '0.875rem 1rem' }}>
          <p
            style={{
              color: 'rgba(255,255,255,0.32)',
              fontSize: '0.8125rem',
              margin: 0,
              fontFamily: FONT,
            }}
          >
            {t('plugins.none')}
          </p>
        </div>
      )}
    </>
  );
}
