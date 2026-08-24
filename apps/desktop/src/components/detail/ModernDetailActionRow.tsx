import React from 'react';
import { Bookmark, BookmarkCheck, CheckCircle2, Film, Heart, MessageCircle, XCircle } from 'lucide-react';
import { t } from '../../i18n';
import { MS } from './detailStyles';
import { ModernIconBtn, ModernPlayButton } from './DetailButtons';

export function ModernDetailActionRow({
  continueLabel,
  hasProgress,
  onPlayClick,
  onPlayTrailer,
  isInWatchlist,
  onToggleWatchlist,
  isCompleted,
  onToggleCompleted,
  isDropped,
  onToggleDropped,
  isFavorite,
  onToggleFavorite,
  onOpenComments,
}: {
  continueLabel: string | null;
  hasProgress: boolean;
  onPlayClick: () => void;
  onPlayTrailer?: () => void;
  isInWatchlist: boolean;
  onToggleWatchlist: () => void;
  isCompleted: boolean;
  onToggleCompleted: () => void;
  isDropped: boolean;
  onToggleDropped: () => void;
  isFavorite: boolean;
  onToggleFavorite: () => void;
  onOpenComments?: () => void;
}) {
  return (
    <div className="detail-action-row" style={MS.actionRow}>
      <ModernPlayButton continueLabel={continueLabel} hasProgress={hasProgress} onClick={onPlayClick} />
      <div style={MS.actionIcons}>
        {onPlayTrailer && (
          <ModernIconBtn title={t('detail.watch_trailer')} onClick={onPlayTrailer}>
            <Film size={19} />
          </ModernIconBtn>
        )}
        <ModernIconBtn
          title={isInWatchlist ? t('detail.in_library') : t('detail.add_to_library')}
          active={isInWatchlist}
          onClick={onToggleWatchlist}
        >
          {isInWatchlist ? <BookmarkCheck size={18} /> : <Bookmark size={18} />}
        </ModernIconBtn>
        <ModernIconBtn
          title={isCompleted ? t('library.unmark_completed') : t('library.mark_completed')}
          active={isCompleted}
          onClick={onToggleCompleted}
        >
          <CheckCircle2 size={19} />
        </ModernIconBtn>
        <ModernIconBtn
          title={isDropped ? t('library.unmark_dropped') : t('library.mark_dropped')}
          active={isDropped}
          onClick={onToggleDropped}
        >
          <XCircle size={19} />
        </ModernIconBtn>
        <ModernIconBtn title={t('detail.favourite')} active={isFavorite} onClick={onToggleFavorite}>
          <Heart size={18} fill={isFavorite ? 'currentColor' : 'none'} />
        </ModernIconBtn>
        {onOpenComments && (
          <ModernIconBtn title={t('detail.trakt_comments')} onClick={onOpenComments}>
            <MessageCircle size={19} />
          </ModernIconBtn>
        )}
      </div>
    </div>
  );
}
