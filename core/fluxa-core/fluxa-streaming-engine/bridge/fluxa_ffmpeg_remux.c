#include "fluxa_ffmpeg_remux.h"

#include <limits.h>

#include <libavcodec/avcodec.h>
#include <libavformat/avformat.h>
#include <libavutil/audio_fifo.h>
#include <libavutil/avutil.h>
#include <libavutil/dict.h>
#include <libavutil/error.h>
#include <libswresample/swresample.h>

typedef struct {
    fluxa_ffmpeg_write_callback callback;
    void *opaque;
    int failed;
} FluxaOutput;

typedef struct {
    int active;
    int source_index;
    AVCodecContext *decoder;
    AVCodecContext *encoder;
    SwrContext *resampler;
    AVAudioFifo *fifo;
    AVStream *output_stream;
    AVRational source_time_base;
    int64_t start_microseconds;
    int64_t next_pts;
} FluxaAudioTranscoder;

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

static int audio_copy_safe(enum AVCodecID codec_id) {
    return codec_id == AV_CODEC_ID_AAC || codec_id == AV_CODEC_ID_AC3 ||
           codec_id == AV_CODEC_ID_EAC3 || codec_id == AV_CODEC_ID_MP3 ||
           codec_id == AV_CODEC_ID_ALAC || codec_id == AV_CODEC_ID_FLAC;
}

static void close_audio_transcoder(FluxaAudioTranscoder *audio) {
    if (!audio) {
        return;
    }
    swr_free(&audio->resampler);
    av_audio_fifo_free(audio->fifo);
    avcodec_free_context(&audio->decoder);
    avcodec_free_context(&audio->encoder);
}

static int setup_audio_transcoder(
    FluxaAudioTranscoder *audio,
    AVStream *source,
    AVStream *destination
) {
    const AVCodec *decoder_codec = avcodec_find_decoder(source->codecpar->codec_id);
    const AVCodec *encoder_codec = avcodec_find_encoder(AV_CODEC_ID_ALAC);
    int result;
    if (!decoder_codec || !encoder_codec) {
        return AVERROR_ENCODER_NOT_FOUND;
    }

    audio->decoder = avcodec_alloc_context3(decoder_codec);
    audio->encoder = avcodec_alloc_context3(encoder_codec);
    if (!audio->decoder || !audio->encoder) {
        return AVERROR(ENOMEM);
    }
    result = avcodec_parameters_to_context(audio->decoder, source->codecpar);
    if (result < 0) {
        return result;
    }
    result = avcodec_open2(audio->decoder, decoder_codec, NULL);
    if (result < 0) {
        return result;
    }

    if (audio->decoder->sample_rate <= 0) {
        audio->decoder->sample_rate = 48000;
    }
    if (audio->decoder->ch_layout.nb_channels == 0) {
        av_channel_layout_default(&audio->decoder->ch_layout, 2);
    }

    audio->encoder->sample_rate = audio->decoder->sample_rate > 0
        ? audio->decoder->sample_rate
        : 48000;
    result = av_channel_layout_copy(&audio->encoder->ch_layout, &audio->decoder->ch_layout);
    if (result < 0 || audio->encoder->ch_layout.nb_channels == 0) {
        av_channel_layout_uninit(&audio->encoder->ch_layout);
        av_channel_layout_default(&audio->encoder->ch_layout, 2);
    }
    // ALAC accepts signed 16-bit PCM across the Apple-supported FFmpeg
    // versions. swresample converts planar/float decoder output to it.
    audio->encoder->sample_fmt = AV_SAMPLE_FMT_S16;
    audio->encoder->time_base = (AVRational){ 1, audio->encoder->sample_rate };
    result = avcodec_open2(audio->encoder, encoder_codec, NULL);
    if (result < 0) {
        return result;
    }
    result = swr_alloc_set_opts2(
        &audio->resampler,
        &audio->encoder->ch_layout,
        audio->encoder->sample_fmt,
        audio->encoder->sample_rate,
        &audio->decoder->ch_layout,
        audio->decoder->sample_fmt,
        audio->decoder->sample_rate,
        0,
        NULL
    );
    if (result < 0 || !audio->resampler) {
        return result < 0 ? result : AVERROR(ENOMEM);
    }
    result = swr_init(audio->resampler);
    if (result < 0) {
        return result;
    }
    result = avcodec_parameters_from_context(destination->codecpar, audio->encoder);
    if (result < 0) {
        return result;
    }
    destination->codecpar->codec_tag = 0;
    destination->time_base = audio->encoder->time_base;
    audio->active = 1;
    audio->source_index = source->index;
    audio->output_stream = destination;
    audio->source_time_base = source->time_base;
    audio->start_microseconds = 0;
    audio->next_pts = AV_NOPTS_VALUE;
    audio->fifo = av_audio_fifo_alloc(
        audio->encoder->sample_fmt,
        audio->encoder->ch_layout.nb_channels,
        audio->encoder->frame_size > 0 ? audio->encoder->frame_size : 1024
    );
    if (!audio->fifo) {
        return AVERROR(ENOMEM);
    }
    return 0;
}

static int write_encoded_audio(FluxaAudioTranscoder *audio, AVFormatContext *output) {
    AVPacket *packet = av_packet_alloc();
    int result;
    if (!packet) {
        return AVERROR(ENOMEM);
    }
    while ((result = avcodec_receive_packet(audio->encoder, packet)) >= 0) {
        packet->stream_index = audio->output_stream->index;
        av_packet_rescale_ts(packet, audio->encoder->time_base,
                             audio->output_stream->time_base);
        result = av_interleaved_write_frame(output, packet);
        av_packet_unref(packet);
        if (result < 0) {
            break;
        }
    }
    av_packet_free(&packet);
    return result == AVERROR(EAGAIN) || result == AVERROR_EOF ? 0 : result;
}

static int encode_audio_fifo(
    FluxaAudioTranscoder *audio,
    AVFormatContext *output,
    int flush
) {
    const int frame_size = audio->encoder->frame_size > 0
        ? audio->encoder->frame_size
        : 1024;
    while (av_audio_fifo_size(audio->fifo) >= (flush ? 1 : frame_size)) {
        int samples = flush ? av_audio_fifo_size(audio->fifo) : frame_size;
        AVFrame *frame = av_frame_alloc();
        int result;
        if (!frame) {
            return AVERROR(ENOMEM);
        }
        frame->format = audio->encoder->sample_fmt;
        frame->sample_rate = audio->encoder->sample_rate;
        frame->nb_samples = samples;
        result = av_channel_layout_copy(&frame->ch_layout, &audio->encoder->ch_layout);
        if (result >= 0) {
            result = av_frame_get_buffer(frame, 0);
        }
        if (result >= 0) {
            result = av_audio_fifo_read(audio->fifo, (void **)frame->data, samples);
        }
        if (result >= 0) {
            frame->pts = audio->next_pts == AV_NOPTS_VALUE ? 0 : audio->next_pts;
            audio->next_pts = frame->pts + samples;
            result = avcodec_send_frame(audio->encoder, frame);
        }
        if (result >= 0) {
            result = write_encoded_audio(audio, output);
        }
        av_frame_free(&frame);
        if (result < 0) {
            return result;
        }
        if (!flush && av_audio_fifo_size(audio->fifo) < frame_size) {
            break;
        }
        if (flush) {
            break;
        }
    }
    return 0;
}

static int encode_audio_frame(
    FluxaAudioTranscoder *audio,
    AVFrame *decoded,
    AVFormatContext *output
) {
    AVFrame *converted = av_frame_alloc();
    int result;
    if (!converted) {
        return AVERROR(ENOMEM);
    }
    converted->format = audio->encoder->sample_fmt;
    converted->sample_rate = audio->encoder->sample_rate;
    result = av_channel_layout_copy(&converted->ch_layout, &audio->encoder->ch_layout);
    if (result >= 0) {
        converted->nb_samples = (int)av_rescale_rnd(
            swr_get_delay(audio->resampler, audio->decoder->sample_rate) + decoded->nb_samples,
            audio->encoder->sample_rate,
            audio->decoder->sample_rate,
            AV_ROUND_UP
        );
        result = av_frame_get_buffer(converted, 0);
    }
    if (result >= 0) {
        result = swr_convert_frame(audio->resampler, converted, decoded);
    }
    if (result >= 0 && audio->next_pts == AV_NOPTS_VALUE) {
        if (decoded->pts != AV_NOPTS_VALUE) {
            audio->next_pts = av_rescale_q(decoded->pts, audio->source_time_base,
                                           audio->encoder->time_base);
            audio->next_pts -= av_rescale_q(audio->start_microseconds,
                                            AV_TIME_BASE_Q,
                                            audio->encoder->time_base);
        } else {
            audio->next_pts = 0;
        }
    }
    if (result >= 0) {
        result = av_audio_fifo_write(audio->fifo, (void **)converted->data,
                                     converted->nb_samples);
    }
    if (result >= 0) {
        result = encode_audio_fifo(audio, output, 0);
    }
    av_frame_free(&converted);
    return result < 0 ? result : 0;
}

static int transcode_audio_packet(
    FluxaAudioTranscoder *audio,
    AVPacket *packet,
    AVFormatContext *output
) {
    AVFrame *frame = av_frame_alloc();
    int result;
    if (!frame) {
        return AVERROR(ENOMEM);
    }
    result = avcodec_send_packet(audio->decoder, packet);
    while (result >= 0) {
        result = avcodec_receive_frame(audio->decoder, frame);
        if (result == AVERROR(EAGAIN) || result == AVERROR_EOF) {
            result = 0;
            break;
        }
        if (result < 0) {
            break;
        }
        result = encode_audio_frame(audio, frame, output);
        av_frame_unref(frame);
    }
    av_frame_free(&frame);
    return result;
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
    FluxaAudioTranscoder *audio_transcoders = NULL;
    unsigned int audio_transcoder_count = 0;
    int best_video = -1;
    int64_t start_offset = start_microseconds;
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
    best_video = av_find_best_stream(input, AVMEDIA_TYPE_VIDEO, -1, -1, NULL, 0);

    if (start_microseconds > 0) {
        int seek_stream = best_video >= 0 ? best_video : 0;
        int64_t seek_timestamp = av_rescale_q(
            start_microseconds,
            AV_TIME_BASE_Q,
            input->streams[seek_stream]->time_base
        );
        result = av_seek_frame(input, seek_stream, seek_timestamp, AVSEEK_FLAG_BACKWARD);
        avformat_flush(input);
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
    audio_transcoders = av_calloc(input->nb_streams, sizeof(*audio_transcoders));
    if (!audio_transcoders) {
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
        if (type == AVMEDIA_TYPE_VIDEO && best_video != source->index) {
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
        destination->disposition = source->disposition;
        destination->avg_frame_rate = source->avg_frame_rate;
        destination->sample_aspect_ratio = source->sample_aspect_ratio;
        if (source->metadata) {
            result = av_dict_copy(&destination->metadata, source->metadata, 0);
            if (result < 0) {
                goto cleanup;
            }
        }
        destination->codecpar->codec_tag = 0;
        destination->time_base = source->time_base;
        stream_map[i] = output_index;
        if (type == AVMEDIA_TYPE_AUDIO &&
            !audio_copy_safe(source->codecpar->codec_id)) {
            FluxaAudioTranscoder *transcoder =
                &audio_transcoders[audio_transcoder_count++];
            result = setup_audio_transcoder(
                transcoder,
                source,
                destination
            );
            if (result < 0) {
                goto cleanup;
            }
            transcoder->start_microseconds = start_microseconds > 0
                ? start_microseconds
                : 0;
        }
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
        FluxaAudioTranscoder *transcoder = NULL;
        for (unsigned int i = 0; i < audio_transcoder_count; ++i) {
            if (audio_transcoders[i].source_index == source_index) {
                transcoder = &audio_transcoders[i];
                break;
            }
        }
        if (transcoder) {
            result = transcode_audio_packet(transcoder, packet, output);
            av_packet_unref(packet);
            if (result < 0 || output_state.failed) {
                goto trailer;
            }
            continue;
        }
        packet->stream_index = stream_map[source_index];
        av_packet_rescale_ts(packet, input->streams[source_index]->time_base,
                             output->streams[packet->stream_index]->time_base);
        if (start_offset > 0) {
            int64_t offset = av_rescale_q(
                start_offset,
                AV_TIME_BASE_Q,
                output->streams[packet->stream_index]->time_base
            );
            if (packet->pts != AV_NOPTS_VALUE) {
                packet->pts -= offset;
            }
            if (packet->dts != AV_NOPTS_VALUE) {
                packet->dts -= offset;
            }
        }
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
    if (result >= 0) {
        for (unsigned int i = 0; i < audio_transcoder_count; ++i) {
            FluxaAudioTranscoder *transcoder = &audio_transcoders[i];
            int audio_result = transcode_audio_packet(transcoder, NULL, output);
            if (audio_result >= 0) {
                audio_result = encode_audio_fifo(transcoder, output, 1);
            }
            if (audio_result >= 0) {
                audio_result = avcodec_send_frame(transcoder->encoder, NULL);
            }
            if (audio_result >= 0) {
                audio_result = write_encoded_audio(transcoder, output);
            }
            if (audio_result < 0) {
                result = audio_result;
                break;
            }
        }
    }
    {
        int trailer_result = av_write_trailer(output);
        if (result >= 0 && trailer_result < 0) {
            result = trailer_result;
        }
    }

cleanup:
    if (audio_transcoders) {
        for (unsigned int i = 0; i < audio_transcoder_count; ++i) {
            close_audio_transcoder(&audio_transcoders[i]);
        }
    }
    av_free(audio_transcoders);
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
