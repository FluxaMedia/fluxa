#include "fluxa_ffmpeg_remux.h"

#include <limits.h>

#include <libavcodec/avcodec.h>
#include <libavformat/avformat.h>
#include <libavutil/avutil.h>
#include <libavutil/error.h>

typedef struct {
    fluxa_ffmpeg_write_callback callback;
    void *opaque;
    int failed;
} FluxaOutput;

static int write_output(void *opaque, const uint8_t *buffer, int size) {
    FluxaOutput *output = (FluxaOutput *)opaque;
    if (output->failed || !output->callback) {
        return AVERROR(EIO);
    }
    if (output->callback(output->opaque, buffer, size) < 0) {
        output->failed = 1;
        return AVERROR(EPIPE);
    }
    return size;
}

static void close_input(AVFormatContext **input) {
    if (input && *input) {
        avformat_close_input(input);
    }
}

int fluxa_ffmpeg_remux_url(
    const char *url,
    const char *headers,
    int64_t start_microseconds,
    fluxa_ffmpeg_write_callback callback,
    void *opaque
) {
    AVFormatContext *input = NULL;
    AVFormatContext *output = NULL;
    AVIOContext *output_io = NULL;
    AVDictionary *options = NULL;
    AVPacket *packet = NULL;
    int *stream_map = NULL;
    AVDictionary *mux_options = NULL;
    FluxaOutput output_state = { callback, opaque, 0 };
    int result = 0;

    if (!url || !callback) {
        return AVERROR(EINVAL);
    }

    if (headers && headers[0] != '\0') {
        av_dict_set(&options, "headers", headers, 0);
    }
    av_dict_set(&options, "reconnect", "1", 0);
    av_dict_set(&options, "reconnect_streamed", "1", 0);

    result = avformat_open_input(&input, url, NULL, &options);
    av_dict_free(&options);
    if (result < 0) {
        goto cleanup;
    }
    result = avformat_find_stream_info(input, NULL);
    if (result < 0) {
        goto cleanup;
    }

    if (start_microseconds > 0) {
        result = avformat_seek_file(
            input,
            -1,
            INT64_MIN,
            start_microseconds,
            INT64_MAX,
            AVSEEK_FLAG_BACKWARD
        );
        if (result < 0) {
            goto cleanup;
        }
    }

    result = avformat_alloc_output_context2(&output, NULL, "mp4", NULL);
    if (result < 0 || !output) {
        result = result < 0 ? result : AVERROR(EINVAL);
        goto cleanup;
    }

    output_io = avio_alloc_context(
        av_malloc(64 * 1024),
        64 * 1024,
        1,
        &output_state,
        NULL,
        write_output,
        NULL
    );
    if (!output_io) {
        result = AVERROR(ENOMEM);
        goto cleanup;
    }
    output->pb = output_io;
    output->flags |= AVFMT_FLAG_CUSTOM_IO;

    stream_map = av_malloc_array(input->nb_streams, sizeof(*stream_map));
    if (!stream_map) {
        result = AVERROR(ENOMEM);
        goto cleanup;
    }
    for (unsigned int i = 0; i < input->nb_streams; ++i) {
        stream_map[i] = -1;
        AVStream *source = input->streams[i];
        enum AVMediaType type = source->codecpar->codec_type;
        if (type != AVMEDIA_TYPE_VIDEO && type != AVMEDIA_TYPE_AUDIO) {
            continue;
        }
        int output_index = (int)output->nb_streams;
        AVStream *destination = avformat_new_stream(output, NULL);
        if (!destination) {
            result = AVERROR(ENOMEM);
            goto cleanup;
        }
        result = avcodec_parameters_copy(destination->codecpar, source->codecpar);
        if (result < 0) {
            goto cleanup;
        }
        destination->codecpar->codec_tag = 0;
        destination->time_base = source->time_base;
        stream_map[i] = output_index;
    }
    if (output->nb_streams == 0) {
        result = AVERROR_STREAM_NOT_FOUND;
        goto cleanup;
    }

    av_dict_set(&mux_options, "movflags", "frag_keyframe+empty_moov+default_base_moof", 0);
    av_dict_set(&mux_options, "avoid_negative_ts", "make_zero", 0);
    result = avformat_write_header(output, &mux_options);
    av_dict_free(&mux_options);
    if (result < 0) {
        goto cleanup;
    }

    packet = av_packet_alloc();
    if (!packet) {
        result = AVERROR(ENOMEM);
        goto trailer;
    }
    while ((result = av_read_frame(input, packet)) >= 0) {
        int source_index = packet->stream_index;
        if (source_index < 0 || source_index >= (int)input->nb_streams ||
            stream_map[source_index] < 0) {
            av_packet_unref(packet);
            continue;
        }
        packet->stream_index = stream_map[source_index];
        av_packet_rescale_ts(packet, input->streams[source_index]->time_base,
                             output->streams[packet->stream_index]->time_base);
        result = av_interleaved_write_frame(output, packet);
        av_packet_unref(packet);
        if (result < 0 || output_state.failed) {
            goto trailer;
        }
    }
    if (result == AVERROR_EOF) {
        result = 0;
    }

trailer:
    {
        int trailer_result = av_write_trailer(output);
        if (result >= 0 && trailer_result < 0) {
            result = trailer_result;
        }
    }

cleanup:
    av_packet_free(&packet);
    av_free(stream_map);
    if (output_io) {
        av_freep(&output_io->buffer);
        avio_context_free(&output_io);
    }
    if (output) {
        avformat_free_context(output);
    }
    close_input(&input);
    av_dict_free(&options);
    av_dict_free(&mux_options);
    return result < 0 ? result : (output_state.failed ? AVERROR(EPIPE) : 0);
}
