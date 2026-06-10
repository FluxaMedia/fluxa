import React, { useEffect } from 'react';
import { Check, Trash as Trash2, UploadSimple as Upload, X } from '@phosphor-icons/react';
import { colors } from '../theme';
import { t } from '../i18n';

export const AVATAR_CATEGORIES = [
  {
    name: 'The Boys',
    key: 'TheBoys',
    avatars: ['a-train', 'billybutcher', 'blacknoir', 'homelander', 'queenmaeve', 'starlight', 'stormfront', 'thedeep'],
  },
  {
    name: 'Breaking Bad',
    key: 'BreakingBad',
    avatars: ['jessepinkman', 'saulgoodman', 'walterwhite'],
  },
  {
    name: 'Peaky Blinders',
    key: 'PeakyBlinders',
    avatars: ['arthurshelby', 'finnshelby', 'johnshelby', 'thomasshelby'],
  },
  {
    name: 'Invincible',
    key: 'Invincible',
    avatars: ['allenthealien', 'atomeve', 'cecilsteadman', 'debbiegrayson', 'invincible', 'olivergrayson', 'omniman'],
  },
  {
    name: 'Avatar: The Last Airbender',
    key: 'ATLA',
    avatars: ['avataraang', 'azula', 'iroh', 'katara', 'sokka', 'tophbeifong', 'zuko'],
  },
];

export function avatarCategoryUrl(categoryKey: string, avatar: string) {
  return `/avatars/${categoryKey}/${avatar}.jpg`;
}

export function AvatarPickerModal({
  selected,
  onSelect,
  onUpload,
  onClear,
  onClose,
}: {
  selected: string | undefined;
  onSelect: (url: string) => void;
  onUpload: () => void;
  onClear: () => void;
  onClose: () => void;
}) {
  useEffect(() => {
    const handler = (e: KeyboardEvent) => { if (e.key === 'Escape') onClose(); };
    window.addEventListener('keydown', handler);
    return () => window.removeEventListener('keydown', handler);
  }, [onClose]);

  return (
    <div style={S.modalBackdrop} onClick={onClose}>
      <div style={S.modal} onClick={(e) => e.stopPropagation()}>
        <div style={S.modalHeader}>
          <div>
            <p style={S.modalTitle}>{t('profiles.choose_image')}</p>
            <p style={S.modalSubtitle}>{t('profiles.choose_image_desc')}</p>
          </div>
          <div style={S.modalActions}>
            <button onClick={onUpload} style={S.compactButton}>
              <Upload size={15} />
              {t('profiles.upload_image')}
            </button>
            {selected && (
              <button onClick={onClear} style={{ ...S.compactButton, color: '#FF9C9C', borderColor: 'rgba(255,90,90,0.24)' }}>
                <Trash2 size={15} />
                {t('common.remove')}
              </button>
            )}
            <button onClick={onClose} style={S.closeButton} aria-label={t('common.close')}>
              <X size={19} />
            </button>
          </div>
        </div>

        <div style={S.avatarCategoryList}>
          {AVATAR_CATEGORIES.map((cat) => (
            <section key={cat.key} style={S.avatarCategory}>
              <p style={S.avatarCategoryTitle}>{cat.name}</p>
              <div style={S.avatarGrid}>
                {cat.avatars.map((av) => {
                  const url = avatarCategoryUrl(cat.key, av);
                  const isSelected = selected === url;
                  return (
                    <button
                      key={av}
                      onClick={() => onSelect(url)}
                      style={{
                        ...S.avatarOption,
                        borderColor: isSelected ? '#FFFFFF' : 'rgba(255,255,255,0.08)',
                        boxShadow: isSelected ? '0 0 0 3px rgba(232,93,63,0.45)' : 'none',
                      }}
                      aria-label={av}
                    >
                      <img src={url} alt={av} style={S.avatarOptionImage} loading="lazy" />
                      {isSelected && (
                        <span style={S.selectedBadge}>
                          <Check size={13} />
                        </span>
                      )}
                    </button>
                  );
                })}
              </div>
            </section>
          ))}
        </div>
      </div>
    </div>
  );
}

const S: Record<string, React.CSSProperties> = {
  modalBackdrop: { position: 'fixed', inset: 0, zIndex: 10000, background: 'rgba(0,0,0,0.76)', display: 'flex', alignItems: 'center', justifyContent: 'center', padding: 24 },
  modal: { width: 'min(760px, calc(100vw - 48px))', maxHeight: 'min(760px, calc(100vh - 48px))', borderRadius: 8, border: '1px solid rgba(255,255,255,0.1)', background: '#10141C', boxShadow: '0 32px 90px rgba(0,0,0,0.72)', overflow: 'hidden', display: 'flex', flexDirection: 'column' },
  modalHeader: { padding: '20px 22px', borderBottom: '1px solid rgba(255,255,255,0.08)', display: 'flex', alignItems: 'center', justifyContent: 'space-between', gap: 16 },
  modalTitle: { margin: 0, color: colors.white, fontSize: 18, fontWeight: 900 },
  modalSubtitle: { margin: '5px 0 0', color: colors.textDim, fontSize: 12, fontWeight: 650 },
  modalActions: { display: 'flex', alignItems: 'center', gap: 8 },
  compactButton: { height: 36, borderRadius: 8, border: '1px solid rgba(255,255,255,0.12)', background: 'rgba(255,255,255,0.06)', color: colors.white, padding: '0 12px', display: 'flex', alignItems: 'center', gap: 7, fontSize: 12, fontWeight: 800 },
  closeButton: { width: 38, height: 38, borderRadius: 8, border: '1px solid rgba(255,255,255,0.12)', background: 'rgba(255,255,255,0.06)', color: colors.white, display: 'flex', alignItems: 'center', justifyContent: 'center' },
  avatarCategoryList: { overflowY: 'auto', padding: '18px 22px 24px', display: 'flex', flexDirection: 'column', gap: 24 },
  avatarCategory: { display: 'flex', flexDirection: 'column', gap: 12 },
  avatarCategoryTitle: { margin: 0, color: colors.textSecondary, fontSize: 12, fontWeight: 850, textTransform: 'uppercase' },
  avatarGrid: { display: 'grid', gridTemplateColumns: 'repeat(auto-fill, minmax(72px, 1fr))', gap: 10 },
  avatarOption: { position: 'relative', aspectRatio: '1 / 1', borderRadius: 8, border: '2px solid rgba(255,255,255,0.08)', background: 'rgba(255,255,255,0.04)', padding: 0, overflow: 'hidden' },
  avatarOptionImage: { width: '100%', height: '100%', objectFit: 'cover', display: 'block' },
  selectedBadge: { position: 'absolute', right: 6, bottom: 6, width: 22, height: 22, borderRadius: 8, background: colors.primary, color: colors.white, display: 'flex', alignItems: 'center', justifyContent: 'center' },
};
