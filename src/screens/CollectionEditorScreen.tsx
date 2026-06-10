import React, { useMemo, useState } from 'react';
import { CaretLeft as ArrowBack } from '@phosphor-icons/react';
import { exportCollectionsJson } from '../core/collections';
import type { HomeCategory, UserCollection, UserCollectionFolder } from '../core/types';
import { t } from '../i18n';
import {
  contrastOn,
  uid,
  cleanUrl,
  UtilButton,
  SaveButton,
  FieldInput,
  Toggle,
  FolderRow,
  ImagePreviewField,
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

  function buildDraft(
    nextFolders = folders,
    nextShowOnHome = showOnHome,
    nextImageUrl = imageUrl,
  ): UserCollection {
    return {
      ...initial,
      title: title.trim(),
      imageUrl: cleanUrl(nextImageUrl),
      showOnHome: nextShowOnHome,
      folders: nextFolders,
    };
  }

  function handleCopyJson() {
    const draft = buildDraft();
    const json = exportCollectionsJson([draft]);
    void navigator.clipboard.writeText(json);
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
        padding: '18px 16px 120px',
        display: 'flex',
        flexDirection: 'column',
        gap: 14,
        boxSizing: 'border-box',
      }}
    >
      <div style={{ display: 'flex', alignItems: 'center', gap: 8 }}>
        <button
          onClick={onDismiss}
          style={{
            width: 36,
            height: 36,
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
        <span style={{ flex: 1, color: '#fff', fontSize: 20, fontWeight: 900, fontFamily: 'sans-serif' }}>
          {initial.title ? t('auto.edit_collection') : t('auto.new_collection')}
        </span>
      </div>

      <div style={{ display: 'flex', gap: 8 }}>
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
        onClick={handleCopyJson}
        fullWidth
      />

      <FieldInput value={title} placeholder={t('auto.collection_name')} onChange={setTitle} accent={accent} />

      <div style={{ display: 'flex', alignItems: 'center', gap: 12 }}>
        <span style={{ flex: 1, color: '#fff', fontWeight: 700, fontSize: 14, fontFamily: 'sans-serif' }}>
          {t('library.show_above_continue_watching')}
        </span>
        <Toggle checked={showOnHome} onChange={setShowOnHome} accent={accent} />
      </div>

      <ImagePreviewField label={t('library.collection_image')} value={imageUrl} onChange={setImageUrl} accent={accent} />

      <div style={{ color: 'rgba(255,255,255,0.85)', fontWeight: 900, fontSize: 14, fontFamily: 'sans-serif' }}>
        {`${folders.length}/10 folders`}
      </div>

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
          height: 44,
          border: `1px solid ${folders.length >= 10 ? 'rgba(255,255,255,0.15)' : accent}`,
          borderRadius: 8,
          background: 'transparent',
          color: folders.length >= 10 ? 'rgba(255,255,255,0.3)' : accent,
          fontWeight: 700,
          fontSize: 14,
          fontFamily: 'sans-serif',
          cursor: folders.length >= 10 ? 'default' : 'pointer',
        }}
      >
        {t('library.add_folder')}
      </button>

      <SaveButton
        label={t('library.save_collection')}
        accent={accent}
        disabled={!canSave}
        onClick={() => { if (canSave) onSave(buildDraft()); }}
      />

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
