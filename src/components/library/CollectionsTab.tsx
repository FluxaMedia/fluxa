import React, { useState } from 'react';
import { Plus } from '@phosphor-icons/react';
import { effectiveFolderImageUrl, effectiveFolderShape } from '../../core/collections';
import type { UserCollection, UserCollectionFolder } from '../../core/types';
import { t } from '../../i18n';

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
  onFolderClick: (folder: UserCollectionFolder, title: string) => void;
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
        <div style={{ height: 16 }} />
        <button
          onClick={onNewCollection}
          style={{
            display: 'flex', alignItems: 'center', gap: 8,
            padding: '10px 20px', background: accent, border: 'none', borderRadius: 24,
            color: '#fff', fontWeight: 700, fontSize: 14, fontFamily: 'sans-serif', cursor: 'pointer',
          }}
        >
          <Plus size={18} />
          {t('auto.new_collection')}
        </button>
      </div>
    );
  }

  return (
    <div style={{ paddingLeft: 58, paddingRight: 58, paddingBottom: 80 }}>
      <div style={{ display: 'flex', alignItems: 'center', gap: 10, marginBottom: 20 }}>
        {collections.some((c) => !c.showOnHome) && (
          <button
            onClick={onShowAllOnHome}
            style={{
              padding: '8px 16px', background: 'rgba(255,255,255,0.07)', border: '1px solid rgba(255,255,255,0.18)',
              borderRadius: 20, color: '#fff', fontWeight: 700, fontSize: 13, fontFamily: 'sans-serif', cursor: 'pointer',
            }}
          >
            Show all on Home
          </button>
        )}
        <div style={{ flex: 1 }} />
        <button
          onClick={onNewCollection}
          style={{
            display: 'flex', alignItems: 'center', gap: 6,
            padding: '8px 16px', background: accent, border: 'none', borderRadius: 20,
            color: '#fff', fontWeight: 700, fontSize: 13, fontFamily: 'sans-serif', cursor: 'pointer',
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
  onFolderClick: (folder: UserCollectionFolder, title: string) => void;
  onEdit: () => void;
  onDelete: () => void;
}) {
  const [confirmDelete, setConfirmDelete] = useState(false);
  const folders = collection.folders ?? [];

  return (
    <div style={{ marginBottom: 32 }}>
      <div style={{ display: 'flex', alignItems: 'center', gap: 12, marginBottom: 14 }}>
        <span style={{ flex: 1, color: '#fff', fontSize: 18, fontWeight: 900, fontFamily: 'sans-serif' }}>
          {collection.title}
        </span>
        <button onClick={onEdit} style={S.actionBtn}>Edit</button>
        {confirmDelete ? (
          <>
            <button onClick={() => { onDelete(); setConfirmDelete(false); }} style={{ ...S.actionBtn, background: '#c0392b', color: '#fff' }}>
              Confirm
            </button>
            <button onClick={() => setConfirmDelete(false)} style={S.actionBtnDim}>Cancel</button>
          </>
        ) : (
          <button onClick={() => setConfirmDelete(true)} style={S.actionBtnDim}>Delete</button>
        )}
      </div>

      {folders.length === 0 ? (
        <div style={{ color: 'rgba(255,255,255,0.3)', fontSize: 13, fontFamily: 'sans-serif', fontStyle: 'italic' }}>
          No folders. Edit to add some.
        </div>
      ) : (
        <div style={{ display: 'flex', gap: 14, overflowX: 'auto', paddingBottom: 8 }}>
          {folders.map((folder) => (
            <FolderTile
              key={folder.id}
              folder={folder}
              accent={accent}
              onClick={() => onFolderClick(folder, `${collection.title} · ${folder.title}`)}
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

  return (
    <div
      onClick={onClick}
      onMouseEnter={() => setHovered(true)}
      onMouseLeave={() => setHovered(false)}
      style={{
        width: tileW, flexShrink: 0, cursor: 'pointer',
        opacity: hovered ? 0.85 : 1,
        transition: 'opacity 0.15s, transform 0.15s',
        transform: hovered ? 'scale(1.02)' : 'none',
      }}
    >
      <div style={{ width: '100%', height: tileH, borderRadius: 8, background: 'rgba(255,255,255,0.06)', overflow: 'hidden', position: 'relative' }}>
        {imgUrl ? (
          <img src={imgUrl} alt={folder.title} style={{ width: '100%', height: '100%', objectFit: 'cover' }} />
        ) : folder.coverEmoji ? (
          <div style={{ width: '100%', height: '100%', display: 'flex', alignItems: 'center', justifyContent: 'center', fontSize: 40 }}>
            {folder.coverEmoji}
          </div>
        ) : (
          <div style={{ width: '100%', height: '100%', background: `${accent}22`, display: 'flex', alignItems: 'center', justifyContent: 'center' }}>
            <span style={{ color: accent, fontSize: 11, fontWeight: 700, fontFamily: 'sans-serif' }}>
              {folder.title.charAt(0).toUpperCase()}
            </span>
          </div>
        )}
      </div>
      {!folder.hideTitle && (
        <div style={{ marginTop: 6, color: '#fff', fontSize: 12, fontWeight: 700, fontFamily: 'sans-serif', overflow: 'hidden', textOverflow: 'ellipsis', whiteSpace: 'nowrap' }}>
          {folder.title}
        </div>
      )}
      {folder.catalogTitle && (
        <div style={{ color: 'rgba(255,255,255,0.45)', fontSize: 11, fontFamily: 'sans-serif', overflow: 'hidden', textOverflow: 'ellipsis', whiteSpace: 'nowrap' }}>
          {folder.catalogTitle}
        </div>
      )}
    </div>
  );
}

const S: Record<string, React.CSSProperties> = {
  empty: {
    display: 'flex', flexDirection: 'column', alignItems: 'center',
    justifyContent: 'center', paddingTop: 80, gap: 10,
  },
  emptyTitle: { color: '#FFFFFF', fontSize: 20, fontWeight: 700, margin: 0, fontFamily: 'sans-serif' },
  emptyHint: { color: 'rgba(255,255,255,0.4)', fontSize: 14, margin: 0, fontFamily: 'sans-serif', textAlign: 'center', maxWidth: 320, lineHeight: 1.5 },
  actionBtn: {
    background: 'rgba(255,255,255,0.08)', border: 'none', borderRadius: 16,
    color: 'rgba(255,255,255,0.7)', fontSize: 12, fontWeight: 700,
    fontFamily: 'sans-serif', padding: '5px 12px', cursor: 'pointer',
  },
  actionBtnDim: {
    background: 'rgba(255,255,255,0.05)', border: 'none', borderRadius: 16,
    color: 'rgba(255,255,255,0.4)', fontSize: 12, fontWeight: 700,
    fontFamily: 'sans-serif', padding: '5px 12px', cursor: 'pointer',
  },
};
