import React from 'react';
import { Bookmark, BookmarkCheck, CheckCircle2, Film, MessageCircle, XCircle } from 'lucide-react';
import { open as shellOpen } from '@tauri-apps/plugin-shell';
import { t } from '../../i18n';
import { MS } from './detailStyles';
import { ModernIconBtn, ModernPlayButton } from './DetailButtons';

export function ModernDetailActionRow({
  continueLabel,
  hasProgress,
  onPlayClick,
  trailerUrl,
  isInWatchlist,
  onToggleWatchlist,
  isCompleted,
  onToggleCompleted,
  isDropped,
  onToggleDropped,
  onOpenComments,
}: {
  continueLabel: string | null;
  hasProgress: boolean;
  onPlayClick: () => void;
  trailerUrl?: string;
  isInWatchlist: boolean;
  onToggleWatchlist: () => void;
  isCompleted: boolean;
  onToggleCompleted: () => void;
  isDropped: boolean;
  onToggleDropped: () => void;
  onOpenComments?: () => void;
}) {
  return (
    <div style={MS.actionRow}>
      <ModernPlayButton
        continueLabel={continueLabel}
        hasProgress={hasProgress}
        onClick={onPlayClick}
      />
      {trailerUrl && (
        <ModernIconBtn title={t('detail.watch_trailer')} onClick={() => shellOpen(trailerUrl).catch(() => {})}>
          <Film size={18} />
        </ModernIconBtn>
      )}
      <ModernIconBtn title={isInWatchlist ? t('detail.in_library') : t('detail.add_to_library')} active={isInWatchlist} onClick={onToggleWatchlist}>
        {isInWatchlist ? <BookmarkCheck size={18} /> : <Bookmark size={18} />}
      </ModernIconBtn>
      <ModernIconBtn title={isCompleted ? t('library.unmark_completed') : t('library.mark_completed')} active={isCompleted} onClick={onToggleCompleted}>
        <CheckCircle2 size={18} />
      </ModernIconBtn>
      <ModernIconBtn title={isDropped ? t('library.unmark_dropped') : t('library.mark_dropped')} active={isDropped} onClick={onToggleDropped}>
        <XCircle size={18} />
      </ModernIconBtn>
      {onOpenComments && <ModernIconBtn title={t('detail.trakt_comments')} onClick={onOpenComments}>
        <MessageCircle size={18} />
      </ModernIconBtn>}
    </div>
  );
}
