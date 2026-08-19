import React, { useEffect, useState } from 'react';
import { Heart, X } from 'lucide-react';
import { t } from '../../i18n';
import { coreInvoke, httpExecuteText } from '../../core/engine';
import { _appVersion, platformFetch } from '../../core/httpClient';
import { getOAuthClientId } from '../../core/traktSync';
import type { Meta } from '../../core/types';

type TraktComment = {
  id?: number;
  parent_id?: number;
  comment?: string;
  review?: boolean;
  spoiler?: boolean;
  likes?: number;
  replies?: number;
  created_at?: string;
  updated_at?: string;
  language?: string | null;
  user_rating?: number | null;
  user_stats?: { rating?: number | null; play_count?: number; completed_count?: number };
  user?: {
    username?: string;
    private?: boolean;
    deleted?: boolean;
    name?: string | null;
    vip?: boolean | null;
    vip_ep?: boolean | null;
    director?: boolean | null;
    ids?: { slug?: string | null; trakt?: number };
    joined_at?: string | null;
    location?: string | null;
    about?: string | null;
    gender?: string | null;
    age?: number | null;
    images?: { avatar?: { full?: string } } | null;
    vip_og?: boolean | null;
    vip_years?: number | null;
    vip_cover_image?: string | null;
  };
};

type TraktSearchItem = {
  movie?: { ids?: { slug?: string } };
  show?: { ids?: { slug?: string } };
};

type Props = {
  meta: Meta;
  onClose: () => void;
};

function CommentAvatar({ name, url }: { name: string; url?: string }) {
  const [failed, setFailed] = useState(false);
  const [imageUrl, setImageUrl] = useState<string | null>(null);
  const initials =
    name
      .trim()
      .split(/\s+/)
      .map((part) => part[0])
      .join('')
      .slice(0, 2)
      .toUpperCase() || '?';

  useEffect(() => {
    let cancelled = false;
    let objectUrl: string | null = null;
    setImageUrl(null);
    setFailed(false);
    if (!url) return;
    void platformFetch(url)
      .then(async (response) => {
        if (!response.ok) throw new Error(`HTTP ${response.status}`);
        return response.blob();
      })
      .then((blob) => {
        objectUrl = URL.createObjectURL(blob);
        if (cancelled) URL.revokeObjectURL(objectUrl);
        else setImageUrl(objectUrl);
      })
      .catch(() => {
        if (!cancelled) setFailed(true);
      });
    return () => {
      cancelled = true;
      if (objectUrl) URL.revokeObjectURL(objectUrl);
    };
  }, [url]);

  if (!imageUrl || failed)
    return (
      <span aria-hidden="true" style={styles.avatarFallback}>
        {initials}
      </span>
    );
  return <img src={imageUrl} alt="" onError={() => setFailed(true)} style={styles.avatar} />;
}

export function TraktCommentsDialog({ meta, onClose }: Props) {
  const [comments, setComments] = useState<TraktComment[]>([]);
  const [loading, setLoading] = useState(true);
  const [error, setError] = useState<string | null>(null);

  useEffect(() => {
    let cancelled = false;
    const load = async () => {
      try {
        const clientId = await getOAuthClientId('trakt');
        const request = await coreInvoke<{ resource: string; id: string; lookupType: string; wantType: string } | null>(
          'traktCommentsRequest',
          JSON.stringify({ contentId: meta.id, contentType: meta.type }),
        );
        if (!clientId || !request?.resource || !request.id || !request.lookupType || !request.wantType)
          throw new Error('comments unavailable');
        const headers = {
          'Content-Type': 'application/json',
          'User-Agent': `Fluxa Desktop/${_appVersion}`,
          'trakt-api-version': '2',
          'trakt-api-key': clientId,
        };
        const slugFromSearch = async (url: string) => {
          const response = await httpExecuteText(url, 'GET', headers);
          if (response.statusCode !== 200 || !response.body.trim()) return null;
          const data = JSON.parse(response.body) as unknown;
          if (!Array.isArray(data)) return null;
          const item = (data as TraktSearchItem[]).find((candidate) => {
            const match = request.wantType === 'show' ? candidate.show : candidate.movie;
            return typeof match?.ids?.slug === 'string' && match.ids.slug.length > 0;
          });
          return request.wantType === 'show' ? (item?.show?.ids?.slug ?? null) : (item?.movie?.ids?.slug ?? null);
        };
        const slug =
          (await slugFromSearch(
            `https://api.trakt.tv/search/${encodeURIComponent(request.lookupType)}/${encodeURIComponent(request.id)}?type=${encodeURIComponent(request.wantType)}`,
          )) ??
          (await slugFromSearch(
            `https://api.trakt.tv/search/${encodeURIComponent(request.wantType)}?query=${encodeURIComponent(meta.name)}&limit=1`,
          ));
        if (!slug) throw new Error('comments unavailable');
        const response = await httpExecuteText(
          `https://api.trakt.tv/${request.resource}/${encodeURIComponent(slug)}/comments/likes?extended=full&limit=100`,
          'GET',
          headers,
        );
        if (response.statusCode < 200 || response.statusCode >= 300) throw new Error(`HTTP ${response.statusCode}`);
        const data = JSON.parse(response.body) as unknown;
        if (!cancelled) setComments(Array.isArray(data) ? (data as TraktComment[]) : []);
      } catch (reason) {
        if (!cancelled) setError(reason instanceof Error ? reason.message : String(reason));
      } finally {
        if (!cancelled) setLoading(false);
      }
    };
    void load();
    return () => {
      cancelled = true;
    };
  }, [meta.id, meta.type]);

  return (
    <div role="presentation" onClick={onClose} style={styles.backdrop}>
      <section
        role="dialog"
        aria-modal="true"
        aria-label={t('detail.trakt_comments')}
        onClick={(event) => event.stopPropagation()}
        style={styles.dialog}
      >
        <div style={styles.header}>
          <div>
            <h2 style={styles.title}>{t('detail.trakt_comments')}</h2>
            <p style={styles.subtitle}>{meta.name}</p>
          </div>
          <button type="button" aria-label={t('common.close')} title={t('common.close')} onClick={onClose} style={styles.closeButton}>
            <X size={20} />
          </button>
        </div>
        <div style={styles.content}>
          {loading && <p style={styles.message}>{t('detail.comments_loading')}</p>}
          {!loading && error && <p style={styles.message}>{t('detail.comments_unavailable', error)}</p>}
          {!loading && !error && comments.length === 0 && <p style={styles.message}>{t('detail.comments_empty')}</p>}
          {!loading &&
            !error &&
            comments.map((comment, index) => (
              <article key={comment.id ?? index} style={styles.comment}>
                <div style={styles.commentHeader}>
                  <CommentAvatar
                    name={comment.user?.name || comment.user?.username || t('detail.comments_anonymous')}
                    url={comment.user?.images?.avatar?.full}
                  />
                  <div style={styles.authorBlock}>
                    <strong>{comment.user?.name || comment.user?.username || t('detail.comments_anonymous')}</strong>
                    {comment.user?.username && <span style={styles.username}>@{comment.user.username}</span>}
                  </div>
                  {comment.review && <span style={styles.review}>{t('detail.comments_review')}</span>}
                  {comment.user?.private && <span style={styles.badge}>{t('detail.comments_private')}</span>}
                  {comment.user?.deleted && <span style={styles.badge}>{t('detail.comments_deleted')}</span>}
                  {comment.user?.vip && <span style={styles.badge}>{t('detail.comments_vip')}</span>}
                  {comment.user?.vip_ep && <span style={styles.badge}>{t('detail.comments_vip_ep')}</span>}
                  {comment.user?.director && <span style={styles.badge}>{t('detail.comments_director')}</span>}
                  {typeof comment.likes === 'number' && (
                    <span style={styles.stats}>
                      <Heart size={14} fill="currentColor" />
                      {t('detail.comments_likes', comment.likes)}
                    </span>
                  )}
                </div>
                <p style={styles.body}>{comment.spoiler ? t('detail.comments_spoiler') : comment.comment}</p>
                <div style={styles.commentMeta}>
                  {typeof comment.id === 'number' && <span>{t('detail.comments_id', comment.id)}</span>}
                  {typeof comment.parent_id === 'number' && <span>{t('detail.comments_parent', comment.parent_id)}</span>}
                  {comment.created_at && <span>{t('detail.comments_posted', formatDate(comment.created_at))}</span>}
                  {comment.updated_at && comment.updated_at !== comment.created_at && (
                    <span>{t('detail.comments_edited', formatDate(comment.updated_at))}</span>
                  )}
                  {typeof comment.replies === 'number' && <span>{t('detail.comments_replies', comment.replies)}</span>}
                  {comment.language && <span>{t('detail.comments_language', comment.language)}</span>}
                  {typeof comment.user_rating === 'number' && <span>{t('detail.comments_rating', comment.user_rating)}</span>}
                  {typeof comment.user_stats?.rating === 'number' && (
                    <span>{t('detail.comments_profile_rating', comment.user_stats.rating)}</span>
                  )}
                  {typeof comment.user_stats?.play_count === 'number' && (
                    <span>{t('detail.comments_plays', comment.user_stats.play_count)}</span>
                  )}
                  {typeof comment.user_stats?.completed_count === 'number' && (
                    <span>{t('detail.comments_completed', comment.user_stats.completed_count)}</span>
                  )}
                </div>
                {comment.user && (
                  <details style={styles.profile}>
                    <summary style={styles.profileSummary}>{t('detail.comments_profile')}</summary>
                    {comment.user.vip_cover_image && <img src={comment.user.vip_cover_image} alt="" style={styles.coverImage} />}
                    <div style={styles.profileGrid}>
                      {comment.user.ids?.slug && <span>{t('detail.comments_slug', comment.user.ids.slug)}</span>}
                      {typeof comment.user.ids?.trakt === 'number' && <span>{t('detail.comments_trakt_id', comment.user.ids.trakt)}</span>}
                      {comment.user.joined_at && <span>{t('detail.comments_joined', formatDate(comment.user.joined_at))}</span>}
                      {comment.user.location && <span>{t('detail.comments_location', comment.user.location)}</span>}
                      {comment.user.gender && <span>{t('detail.comments_gender', comment.user.gender)}</span>}
                      {typeof comment.user.age === 'number' && <span>{t('detail.comments_age', comment.user.age)}</span>}
                      {typeof comment.user.vip_years === 'number' && <span>{t('detail.comments_vip_years', comment.user.vip_years)}</span>}
                      {comment.user.vip_og && <span>{t('detail.comments_vip_og')}</span>}
                    </div>
                    {comment.user.about && <p style={styles.about}>{comment.user.about}</p>}
                  </details>
                )}
              </article>
            ))}
        </div>
      </section>
    </div>
  );
}

function formatDate(value: string): string {
  const date = new Date(value);
  return Number.isNaN(date.getTime()) ? value : date.toLocaleString();
}

const styles: Record<string, React.CSSProperties> = {
  backdrop: {
    position: 'fixed',
    inset: 0,
    zIndex: 1000,
    display: 'flex',
    alignItems: 'center',
    justifyContent: 'center',
    padding: '1.5rem',
    background: 'rgba(0,0,0,0.72)',
  },
  dialog: {
    width: 'min(42rem, 100%)',
    maxHeight: 'min(44rem, calc(100vh - 3rem))',
    display: 'flex',
    flexDirection: 'column',
    overflow: 'hidden',
    border: '0.0625rem solid rgba(255,255,255,0.16)',
    borderRadius: '1rem',
    background: '#171717',
    color: '#fff',
    boxShadow: '0 1.5rem 5rem rgba(0,0,0,0.55)',
  },
  header: {
    display: 'flex',
    alignItems: 'flex-start',
    justifyContent: 'space-between',
    gap: '1rem',
    padding: '1.25rem 1.25rem 1rem',
    borderBottom: '0.0625rem solid rgba(255,255,255,0.1)',
  },
  title: { margin: 0, fontSize: '1.125rem' },
  subtitle: { margin: '0.25rem 0 0', color: 'rgba(255,255,255,0.58)', fontSize: '0.875rem' },
  closeButton: {
    display: 'flex',
    alignItems: 'center',
    justifyContent: 'center',
    width: '2.25rem',
    height: '2.25rem',
    border: 0,
    borderRadius: '50%',
    background: 'rgba(255,255,255,0.1)',
    color: '#fff',
    cursor: 'pointer',
  },
  content: { overflowY: 'auto', padding: '0.75rem 1.25rem 1.25rem' },
  message: { margin: '1.5rem 0', color: 'rgba(255,255,255,0.65)', textAlign: 'center' },
  comment: { padding: '1rem 0', borderBottom: '0.0625rem solid rgba(255,255,255,0.1)' },
  commentHeader: { display: 'flex', alignItems: 'center', gap: '0.5rem', flexWrap: 'wrap', fontSize: '0.875rem' },
  avatar: { width: '2.25rem', height: '2.25rem', borderRadius: '50%', objectFit: 'cover' },
  avatarFallback: {
    width: '2.25rem',
    height: '2.25rem',
    flexShrink: 0,
    display: 'inline-flex',
    alignItems: 'center',
    justifyContent: 'center',
    borderRadius: '50%',
    background: 'rgba(255,255,255,0.14)',
    color: 'rgba(255,255,255,0.82)',
    fontSize: '0.75rem',
    fontWeight: 700,
  },
  authorBlock: { display: 'flex', flexDirection: 'column', gap: '0.0625rem' },
  username: { color: 'rgba(255,255,255,0.55)', fontSize: '0.75rem' },
  review: {
    padding: '0.125rem 0.375rem',
    borderRadius: '0.25rem',
    background: 'rgba(255,255,255,0.13)',
    color: 'rgba(255,255,255,0.76)',
    fontSize: '0.6875rem',
  },
  badge: {
    padding: '0.125rem 0.375rem',
    borderRadius: '0.25rem',
    background: 'rgba(255,255,255,0.09)',
    color: 'rgba(255,255,255,0.68)',
    fontSize: '0.6875rem',
  },
  stats: {
    marginLeft: 'auto',
    display: 'inline-flex',
    alignItems: 'center',
    gap: '0.25rem',
    color: '#FF6B81',
    fontSize: '0.75rem',
    fontWeight: 600,
  },
  body: { margin: '0.625rem 0 0', color: 'rgba(255,255,255,0.82)', lineHeight: 1.55, whiteSpace: 'pre-wrap' },
  commentMeta: {
    display: 'flex',
    flexWrap: 'wrap',
    gap: '0.375rem 0.75rem',
    marginTop: '0.625rem',
    color: 'rgba(255,255,255,0.52)',
    fontSize: '0.75rem',
  },
  profile: { marginTop: '0.75rem', borderRadius: '0.5rem', overflow: 'hidden', background: 'rgba(255,255,255,0.05)' },
  profileSummary: { padding: '0.625rem 0.75rem', cursor: 'pointer', color: 'rgba(255,255,255,0.78)', fontSize: '0.8125rem' },
  coverImage: { width: '100%', maxHeight: '8rem', objectFit: 'cover', display: 'block' },
  profileGrid: {
    display: 'flex',
    flexWrap: 'wrap',
    gap: '0.375rem 0.75rem',
    padding: '0 0.75rem 0.75rem',
    color: 'rgba(255,255,255,0.62)',
    fontSize: '0.75rem',
  },
  about: { margin: '0 0.75rem 0.75rem', color: 'rgba(255,255,255,0.76)', fontSize: '0.8125rem', lineHeight: 1.45, whiteSpace: 'pre-wrap' },
};
