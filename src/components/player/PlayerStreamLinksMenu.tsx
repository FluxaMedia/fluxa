import { Download, Link2, Magnet } from 'lucide-react';
import type { RefObject } from 'react';
import { t } from '../../i18n';
import { enqueueOfflineDownload, streamMagnetLink } from '../../core/engine';
import { buildOfflineDownloadRequest } from '../../core/streamLinks';
import type { Meta, Stream, Video } from '../../core/types';
import { ContextMenu } from '../ui/ContextMenu';

export function PlayerStreamLinksMenu({
  point,
  onClose,
  plan,
  streamRef,
  metaRef,
  currentEpisode,
}: {
  point: { x: number; y: number } | null;
  onClose: () => void;
  plan: { isTorrent: boolean; sourceLink?: string; downloadLink?: string } | null;
  streamRef?: RefObject<Stream | null>;
  metaRef?: RefObject<Meta | null>;
  currentEpisode?: Video | null;
}) {
  const stream = streamRef?.current;
  const meta = metaRef?.current;
  const sourceLink = plan?.sourceLink;
  const downloadLink = plan?.downloadLink;
  return (
    <ContextMenu
      point={point}
      onClose={onClose}
      items={
        stream
          ? [
              ...(sourceLink
                ? [
                    {
                      icon: <Link2 size={15} />,
                      label: t('player.copy_stream_link'),
                      onSelect: () => {
                        void navigator.clipboard.writeText(sourceLink);
                      },
                    },
                  ]
                : []),
              ...(plan?.isTorrent
                ? [
                    {
                      icon: <Magnet size={15} />,
                      label: t('player.copy_magnet_link'),
                      onSelect: () => {
                        void streamMagnetLink(stream).then((link) => {
                          if (link) void navigator.clipboard.writeText(link);
                        });
                      },
                    },
                  ]
                : []),
              ...(meta && downloadLink
                ? [
                    {
                      icon: <Download size={15} />,
                      label: t('player.download_this_video'),
                      onSelect: () => {
                        void enqueueOfflineDownload(buildOfflineDownloadRequest(meta, stream, currentEpisode));
                      },
                    },
                  ]
                : []),
            ]
          : []
      }
    />
  );
}
