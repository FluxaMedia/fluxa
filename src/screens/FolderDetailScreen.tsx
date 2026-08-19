import { useMemo, useState } from 'react';
import { ChevronLeft } from 'lucide-react';
import type { CSSProperties } from 'react';
import type { Meta } from '../core/types';
import type { PosterPrefs } from '../core/posterPrefs';
import { ModernTabBar } from '../components/detail/DetailButtons';
import { VirtualizedPosterGrid } from '../components/VirtualizedPosterGrid';
import { ShelfRow } from '../components/ShelfRow';
import { t } from '../i18n';

export interface FolderTab {
  id: string;
  title: string;
  type: string;
  items: Meta[];
}

interface Props {
  title: string;
  viewMode?: string;
  tabs: FolderTab[];
  posterPrefs: PosterPrefs;
  onNavigateDetail: (meta: Meta) => void;
  onBack: () => void;
}

export function FolderDetailScreen({ title, viewMode, tabs, posterPrefs, onNavigateDetail, onBack }: Props) {
  const isTabbed = viewMode === 'TABBED_GRID';
  const [activeTab, setActiveTab] = useState(() => tabs[0]?.id ?? '');
  const activeItems = useMemo(() => tabs.find((tab) => tab.id === activeTab)?.items ?? tabs[0]?.items ?? [], [tabs, activeTab]);

  return (
    <div style={S.screen}>
      <div style={S.header}>
        <button style={S.backBtn} onClick={onBack}>
          <ChevronLeft size={20} />
        </button>
        <h2 style={S.title}>{title}</h2>
      </div>

      {isTabbed ? (
        <>
          {tabs.length > 1 && (
            <div style={S.tabBarWrap}>
              <ModernTabBar
                tabs={tabs.map((tab) => ({ id: tab.id, label: tab.title }))}
                active={activeTab || tabs[0]?.id}
                onChange={setActiveTab}
              />
            </div>
          )}
          <div style={S.gridWrap}>
            {activeItems.length === 0 ? (
              <p style={S.empty}>{t('home.folder_load_failed_body')}</p>
            ) : (
              <VirtualizedPosterGrid
                items={activeItems}
                selectedId={null}
                posterPrefs={posterPrefs}
                onHover={() => false}
                onClick={onNavigateDetail}
                onScrollActivity={() => {}}
              />
            )}
          </div>
        </>
      ) : (
        <div style={S.rowsWrap}>
          {tabs
            .filter((tab) => tab.id !== 'all')
            .map((tab) => (
              <ShelfRow key={tab.id} title={tab.title} items={tab.items} onItemClick={onNavigateDetail} posterPrefs={posterPrefs} />
            ))}
        </div>
      )}
    </div>
  );
}

const S: Record<string, CSSProperties> = {
  screen: {
    display: 'flex',
    flexDirection: 'column',
    width: 'calc(100% - 6.5rem)',
    height: 'calc(100% - 3.25rem)',
    marginLeft: '6.5rem',
    marginTop: '3.25rem',
    background: '#09091280',
    overflow: 'hidden',
  },
  header: {
    display: 'flex',
    alignItems: 'center',
    gap: '0.75rem',
    padding: '1rem 1.5rem',
    flexShrink: 0,
    borderBottom: '1px solid rgba(255,255,255,0.05)',
  },
  backBtn: {
    width: '2.25rem',
    height: '2.25rem',
    borderRadius: '50%',
    border: '1px solid rgba(255,255,255,0.12)',
    background: 'rgba(255,255,255,0.06)',
    color: '#FFFFFF',
    cursor: 'pointer',
    display: 'flex',
    alignItems: 'center',
    justifyContent: 'center',
    flexShrink: 0,
    padding: 0,
  },
  title: { color: '#FFFFFF', fontSize: '1.125rem', fontWeight: 700, margin: 0, letterSpacing: '-0.01em' },
  tabBarWrap: { padding: '0.875rem 1.5rem 0', flexShrink: 0 },
  gridWrap: { flex: 1, overflowY: 'auto', padding: '1.25rem 1.5rem 3.75rem' },
  rowsWrap: { flex: 1, overflowY: 'auto', padding: '1.25rem 0 3.75rem', display: 'flex', flexDirection: 'column', gap: '0.5rem' },
  empty: { color: 'rgba(255,255,255,0.35)', fontSize: '0.875rem', padding: '2rem', textAlign: 'center' },
};
