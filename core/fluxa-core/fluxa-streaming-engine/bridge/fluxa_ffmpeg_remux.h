#ifndef FLUXA_FFMPEG_REMUX_H
#define FLUXA_FFMPEG_REMUX_H

#include <stdint.h>

typedef int (*fluxa_ffmpeg_write_callback)(void *opaque, const uint8_t *data, int size);

/*
 * Remuxes the first video/audio tracks through FFmpeg's native demuxer and
 * fragmented MP4 muxer. The callback receives muxed bytes incrementally.
 * No decoded video frames are produced by this API.
 */
int fluxa_ffmpeg_remux_url(
    const char *url,
    const char *headers,
    int64_t start_microseconds,
    fluxa_ffmpeg_write_callback callback,
    void *opaque
);

#endif
