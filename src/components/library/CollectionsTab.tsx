import React, { useState } from 'react';
import { Plus } from 'lucide-react';
import { effectiveFolderImageUrl, effectiveFolderShape } from '../../core/collections';
import { contrastOn } from '../../screens/CollectionEditorPrimitives';
import type { UserCollection, UserCollectionFolder } from '../../core/types';
import { t } from '../../i18n';
import { usePosterSrc } from '../../hooks/usePosterSrc';

const TILE_W = 140;
const TILE_H_POSTER = 210;
const TILE_H_WIDE = 90;
const TILE_H_SQUARE = 140;

export function CollectionsTab({
  collections,
  accent,
  onFolderClick,
  onEditCollection,
  onDeleteCollection,
  onNewCollection,
  onShowAllOnHome,
}: {
  collections: UserCollection[];
  accent: string;
  onFolderClick: (folder: UserCollectionFolder, title: string, collection: UserCollection) => void;
  onEditCollection: (col: UserCollection) => void;
  onDeleteCollection: (id: string) => void;
  onNewCollection: () => void;
  onShowAllOnHome: () => void;
}) {
  if (collections.length === 0) {
    return (
      <div style={S.empty}>
        <p style={S.emptyTitle}>No collections yet</p>
        <p style={S.emptyHint}>Create a collection to organize your catalogs into custom shelves.</p>
        <div style={{ height: '1rem' }} />
        <button
          onClick={onNewCollection}
          style={{
            display: 'flex',
            alignItems: 'center',
            gap: '0.5rem',
            padding: '0.625rem 1.25rem',
            background: accent,
            border: 'none',
            borderRadius: '1.5rem',
            color: contrastOn(accent),
            fontWeight: 700,
            fontSize: '0.875rem',
            cursor: 'pointer',
          }}
        >
          <Plus size={18} />
          {t('auto.new_collection')}
        </button>
      </div>
    );
  }

  return (
    <div style={{ paddingLeft: '3.625rem', paddingRight: '3.625rem', paddingBottom: '5rem' }}>
      <div style={{ display: 'flex', alignItems: 'center', gap: '0.625rem', marginBottom: '1.25rem' }}>
        {collections.some((c) => !c.showOnHome) && (
          <button
            onClick={onShowAllOnHome}
            style={{
              padding: '0.5rem 1rem',
              background: 'rgba(255,255,255,0.07)',
              border: '1px solid rgba(255,255,255,0.18)',
              borderRadius: '1.25rem',
              color: '#fff',
              fontWeight: 700,
              fontSize: '0.8125rem',
              cursor: 'pointer',
            }}
          >
            Show all on Home
          </button>
        )}
        <div style={{ flex: 1 }} />
        <button
          onClick={onNewCollection}
          style={{
            display: 'flex',
            alignItems: 'center',
            gap: '0.375rem',
            padding: '0.5rem 1rem',
            background: accent,
            border: 'none',
            borderRadius: '1.25rem',
            color: contrastOn(accent),
            fontWeight: 700,
            fontSize: '0.8125rem',
            cursor: 'pointer',
          }}
        >
          <Plus size={16} />
          {t('auto.new_collection')}
        </button>
      </div>

      {collections.map((col) => (
        <CollectionSection
          key={col.id}
          collection={col}
          accent={accent}
          onFolderClick={onFolderClick}
          onEdit={() => onEditCollection(col)}
          onDelete={() => onDeleteCollection(col.id)}
        />
      ))}
    </div>
  );
}

function CollectionSection({
  collection,
  accent,
  onFolderClick,
  onEdit,
  onDelete,
}: {
  collection: UserCollection;
  accent: string;
  onFolderClick: (folder: UserCollectionFolder, title: string, collection: UserCollection) => void;
  onEdit: () => void;
  onDelete: () => void;
}) {
  const [confirmDelete, setConfirmDelete] = useState(false);
  const folders = collection.folders ?? [];

  return (
    <div style={{ marginBottom: '2rem' }}>
      <div style={{ display: 'flex', alignItems: 'center', gap: '0.75rem', marginBottom: '0.875rem' }}>
        <span style={{ flex: 1, color: '#fff', fontSize: '1.125rem', fontWeight: 900 }}>{collection.title}</span>
        <button onClick={onEdit} style={S.actionBtn}>
          Edit
        </button>
        {confirmDelete ? (
          <>
            <button
              onClick={() => {
                onDelete();
                setConfirmDelete(false);
              }}
              style={{ ...S.actionBtn, background: '#c0392b', color: '#fff' }}
            >
              Confirm
            </button>
            <button onClick={() => setConfirmDelete(false)} style={S.actionBtnDim}>
              Cancel
            </button>
          </>
        ) : (
          <button onClick={() => setConfirmDelete(true)} style={S.actionBtnDim}>
            Delete
          </button>
        )}
      </div>

      {folders.length === 0 ? (
        <div style={{ color: 'rgba(255,255,255,0.3)', fontSize: '0.8125rem', fontStyle: 'italic' }}>No folders. Edit to add some.</div>
      ) : (
        <div style={{ display: 'flex', gap: '0.875rem', overflowX: 'auto', paddingBottom: '0.5rem' }}>
          {folders.map((folder) => (
            <FolderTile
              key={folder.id}
              folder={folder}
              accent={accent}
              onClick={() => onFolderClick(folder, `${collection.title} · ${folder.title}`, collection)}
            />
          ))}
        </div>
      )}
    </div>
  );
}

function FolderTile({ folder, accent, onClick }: { folder: UserCollectionFolder; accent: string; onClick: () => void }) {
  const [hovered, setHovered] = useState(false);
  const imgUrl = effectiveFolderImageUrl(folder);
  const shape = effectiveFolderShape(folder);
  const tileH = shape === 'wide' ? TILE_H_WIDE : shape === 'square' ? TILE_H_SQUARE : TILE_H_POSTER;
  const tileW = shape === 'wide' ? TILE_W * 1.78 : TILE_W;

  const { src: posterSrc } = usePosterSrc(imgUrl);
  const hasImg = !!posterSrc;

  return (
    <div
      onClick={onClick}
      onMouseEnter={() => setHovered(true)}
      onMouseLeave={() => setHovered(false)}
      style={{
        width: `${tileW / 16}rem`,
        flexShrink: 0,
        cursor: 'pointer',
        opacity: hovered ? 0.85 : 1,
        transition: 'opacity 0.15s, transform 0.15s',
        transform: hovered ? 'scale(1.02)' : 'none',
      }}
    >
      <div
        style={{
          width: '100%',
          height: `${tileH / 16}rem`,
          borderRadius: '0.5rem',
          background: 'rgba(255,255,255,0.06)',
          overflow: 'hidden',
          position: 'relative',
        }}
      >
        {hasImg ? (
          <img
            src={posterSrc}
            alt={folder.title}
            loading="lazy"
            decoding="async"
            style={{ width: '100%', height: '100%', objectFit: 'cover' }}
          />
        ) : folder.coverEmoji ? (
          <div
            style={{ width: '100%', height: '100%', display: 'flex', alignItems: 'center', justifyContent: 'center', fontSize: '2.5rem' }}
          >
            {folder.coverEmoji}
          </div>
        ) : (
          <div
            style={{
              width: '100%',
              height: '100%',
              background: `${accent}22`,
              display: 'flex',
              alignItems: 'center',
              justifyContent: 'center',
            }}
          >
            <span style={{ color: accent, fontSize: '0.6875rem', fontWeight: 700 }}>{folder.title.charAt(0).toUpperCase()}</span>
          </div>
        )}
      </div>
      {!folder.hideTitle && (
        <div
          style={{
            marginTop: '0.375rem',
            color: '#fff',
            fontSize: '0.75rem',
            fontWeight: 700,
            overflow: 'hidden',
            textOverflow: 'ellipsis',
            whiteSpace: 'nowrap',
          }}
        >
          {folder.title}
        </div>
      )}
      {folder.catalogTitle && (
        <div
          style={{
            color: 'rgba(255,255,255,0.45)',
            fontSize: '0.6875rem',
            overflow: 'hidden',
            textOverflow: 'ellipsis',
            whiteSpace: 'nowrap',
          }}
        >
          {folder.catalogTitle}
        </div>
      )}
    </div>
  );
}

const S: Record<string, React.CSSProperties> = {
  empty: {
    display: 'flex',
    flexDirection: 'column',
    alignItems: 'center',
    justifyContent: 'center',
    paddingTop: '5rem',
    gap: '0.625rem',
  },
  emptyTitle: { color: '#FFFFFF', fontSize: '1.25rem', fontWeight: 700, margin: 0 },
  emptyHint: { color: 'rgba(255,255,255,0.4)', fontSize: '0.875rem', margin: 0, textAlign: 'center', maxWidth: '20rem', lineHeight: 1.5 },
  actionBtn: {
    background: 'rgba(255,255,255,0.08)',
    border: 'none',
    borderRadius: '1rem',
    color: 'rgba(255,255,255,0.7)',
    fontSize: '0.75rem',
    fontWeight: 700,
    padding: '0.3125rem 0.75rem',
    cursor: 'pointer',
  },
  actionBtnDim: {
    background: 'rgba(255,255,255,0.05)',
    border: 'none',
    borderRadius: '1rem',
    color: 'rgba(255,255,255,0.4)',
    fontSize: '0.75rem',
    fontWeight: 700,
    padding: '0.3125rem 0.75rem',
    cursor: 'pointer',
  },
};
