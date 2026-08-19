import React, { useMemo, useState } from 'react';
import { ChevronLeft as ArrowBack } from 'lucide-react';
import { exportCollectionsJson } from '../core/collections';
import type { HomeCategory, UserCollection, UserCollectionFolder } from '../core/types';
import { t } from '../i18n';
import {
  uid,
  cleanUrl,
  UtilButton,
  SaveButton,
  FieldInput,
  Toggle,
  FolderRow,
  ImagePreviewField,
  SectionLabel,
  Card,
} from './CollectionEditorPrimitives';
import { FolderEditorPage } from './FolderEditorPage';
import { ImportDialog } from './ImportDialog';

interface CollectionEditorProps {
  accent: string;
  initial: UserCollection;
  allCollections: UserCollection[];
  catalogOptions: HomeCategory[];
  onDismiss: () => void;
  onSave: (c: UserCollection) => void;
  onImportClick: () => void;
  onExportClick: () => void;
}

function CollectionEditorPage({
  accent,
  initial,
  allCollections,
  catalogOptions,
  onDismiss,
  onSave,
  onImportClick,
  onExportClick,
}: CollectionEditorProps) {
  const [title, setTitle] = useState(initial.title ?? '');
  const [imageUrl, setImageUrl] = useState(initial.imageUrl ?? '');
  const [showOnHome, setShowOnHome] = useState(initial.showOnHome ?? false);
  const [folders, setFolders] = useState<UserCollectionFolder[]>(initial.folders ?? []);
  const [editingFolder, setEditingFolder] = useState<UserCollectionFolder | null>(null);

  function buildDraft(nextFolders = folders, nextShowOnHome = showOnHome, nextImageUrl = imageUrl): UserCollection {
    return {
      ...initial,
      title: title.trim(),
      imageUrl: cleanUrl(nextImageUrl),
      showOnHome: nextShowOnHome,
      folders: nextFolders,
    };
  }

  async function handleCopyJson() {
    const draft = buildDraft();
    const json = await exportCollectionsJson([draft]);
    await navigator.clipboard.writeText(json);
  }

  const canSave = title.trim().length > 0;

  return (
    <div
      style={{
        position: 'absolute',
        inset: 0,
        zIndex: 20,
        background: '#060810',
        overflowY: 'auto',
        padding: '1.5rem 2rem 7.5rem',
        boxSizing: 'border-box',
        display: 'flex',
        justifyContent: 'center',
      }}
    >
      <div style={{ width: '100%', maxWidth: '40rem', display: 'flex', flexDirection: 'column', gap: '1.375rem' }}>
        <div
          style={{
            display: 'flex',
            alignItems: 'center',
            gap: '0.75rem',
            paddingBottom: '1.125rem',
            borderBottom: '1px solid rgba(255,255,255,0.08)',
          }}
        >
          <button
            onClick={onDismiss}
            style={{
              width: '2.25rem',
              height: '2.25rem',
              borderRadius: '50%',
              border: 'none',
              background: 'rgba(255,255,255,0.06)',
              color: '#fff',
              cursor: 'pointer',
              display: 'flex',
              alignItems: 'center',
              justifyContent: 'center',
              flexShrink: 0,
            }}
          >
            <ArrowBack size={20} />
          </button>
          <span style={{ flex: 1, color: '#fff', fontSize: '1.375rem', fontWeight: 700, letterSpacing: '-0.025em' }}>
            {initial.title ? t('auto.edit_collection') : t('auto.new_collection')}
          </span>
        </div>

        <div>
          <SectionLabel>{t('auto.collection_name')}</SectionLabel>
          <Card>
            <FieldInput value={title} placeholder={t('auto.collection_name')} onChange={setTitle} accent={accent} />
            <ImagePreviewField label={t('library.collection_image')} value={imageUrl} onChange={setImageUrl} accent={accent} />
            <div style={{ display: 'flex', alignItems: 'center', gap: '0.75rem' }}>
              <span style={{ flex: 1, color: '#fff', fontWeight: 600, fontSize: '0.8125rem' }}>
                {t('library.show_above_continue_watching')}
              </span>
              <Toggle checked={showOnHome} onChange={setShowOnHome} accent={accent} />
            </div>
          </Card>
        </div>

        <div>
          <SectionLabel>{`${t('library.folder')} · ${folders.length}/10`}</SectionLabel>
          <Card>
            {folders.length === 0 && (
              <div style={{ color: 'rgba(255,255,255,0.35)', fontSize: '0.8125rem', textAlign: 'center', padding: '0.5rem 0' }}>
                {t('library.no_folders_yet')}
              </div>
            )}
            {folders.map((folder) => (
              <FolderRow key={folder.id} folder={folder} accent={accent} onClick={() => setEditingFolder(folder)} />
            ))}
            <button
              disabled={folders.length >= 10}
              onClick={() =>
                setEditingFolder({
                  id: `folder_${uid()}`,
                  title: '',
                  shape: 'poster',
                })
              }
              style={{
                width: '100%',
                height: '2.75rem',
                border: `1px dashed ${folders.length >= 10 ? 'rgba(255,255,255,0.15)' : 'rgba(255,255,255,0.25)'}`,
                borderRadius: '0.5rem',
                background: 'transparent',
                color: folders.length >= 10 ? 'rgba(255,255,255,0.3)' : 'rgba(255,255,255,0.7)',
                fontWeight: 700,
                fontSize: '0.875rem',
                cursor: folders.length >= 10 ? 'default' : 'pointer',
              }}
            >
              {t('library.add_folder')}
            </button>
          </Card>
        </div>

        <SaveButton
          label={t('library.save_collection')}
          accent={accent}
          disabled={!canSave}
          onClick={() => {
            if (canSave) onSave(buildDraft());
          }}
        />

        <div>
          <SectionLabel>{t('settings.advanced')}</SectionLabel>
          <Card>
            <div style={{ display: 'flex', gap: '0.5rem' }}>
              <UtilButton label={t('library.import_collections')} accent={accent} onClick={onImportClick} fullWidth />
              <UtilButton
                label={t('library.export_collections')}
                accent={accent}
                disabled={allCollections.length === 0}
                onClick={onExportClick}
                fullWidth
              />
            </div>
            <UtilButton
              label={t('library.copy_collection_json')}
              accent={accent}
              disabled={!canSave}
              onClick={() => void handleCopyJson()}
              fullWidth
            />
          </Card>
        </div>
      </div>

      {editingFolder && (
        <FolderEditorPage
          initial={editingFolder}
          accent={accent}
          catalogOptions={catalogOptions}
          onDismiss={() => setEditingFolder(null)}
          onSave={(saved) => {
            setFolders((prev) => {
              const without = prev.filter((f) => f.id !== saved.id);
              return [...without, saved].slice(0, 10);
            });
            setEditingFolder(null);
          }}
        />
      )}
    </div>
  );
}

export interface CollectionEditorScreenProps {
  accent: string;
  initial: UserCollection | null;
  allCollections: UserCollection[];
  catalogOptions: HomeCategory[];
  onDismiss: () => void;
  onSave: (c: UserCollection) => void;
  onImportJson: (json: string) => void;
  onExportAll: () => void;
}

export function CollectionEditorScreen({
  accent,
  initial,
  allCollections,
  catalogOptions,
  onDismiss,
  onSave,
  onImportJson,
  onExportAll,
}: CollectionEditorScreenProps) {
  const [showImport, setShowImport] = useState(false);

  const draft: UserCollection = initial ?? {
    id: `col_${uid()}`,
    title: '',
    folders: [],
    showOnHome: false,
    showAllTab: true,
    viewMode: 'FOLLOW_LAYOUT',
    focusGlowEnabled: true,
  };

  return (
    <>
      <CollectionEditorPage
        accent={accent}
        initial={draft}
        allCollections={allCollections}
        catalogOptions={catalogOptions}
        onDismiss={onDismiss}
        onSave={onSave}
        onImportClick={() => setShowImport(true)}
        onExportClick={onExportAll}
      />
      {showImport && (
        <ImportDialog
          accent={accent}
          onDismiss={() => setShowImport(false)}
          onImport={(json) => {
            setShowImport(false);
            onImportJson(json);
          }}
        />
      )}
    </>
  );
}
