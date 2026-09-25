#include "nvdec_ffmpeg.h"

#include <errno.h>
#include <dlfcn.h>
#include <limits.h>
#include <stdint.h>
#include <stdlib.h>
#include <string.h>
#include <unistd.h>

#ifdef FLUXA_NVDEC_DIAGNOSTICS
#include <stdio.h>
#endif

#include <libavcodec/avcodec.h>
#include <libavutil/error.h>
#include <libavutil/hwcontext.h>
#include <libavutil/hwcontext_cuda.h>
#include <libavutil/pixfmt.h>

/* Resolve the CUDA driver at runtime so machines without an NVIDIA driver can
 * still start the application and use the normal CPU artwork decoder. */
typedef struct FluxaCudaApi {
    void *library;
    tcuCtxPushCurrent_v2 *push_current;
    tcuCtxPopCurrent_v2 *pop_current;
    tcuCtxGetDevice *context_get_device;
    tcuDeviceGetUuid_v2 *device_get_uuid;
    tcuMemcpy2D_v2 *memcpy_2d;
    tcuMemAlloc_v2 *mem_alloc;
    tcuMemFree_v2 *mem_free;
    tcuImportExternalMemory *import_external_memory;
    tcuDestroyExternalMemory *destroy_external_memory;
    tcuExternalMemoryGetMappedBuffer *map_external_buffer;
    tcuImportExternalSemaphore *import_external_semaphore;
    tcuDestroyExternalSemaphore *destroy_external_semaphore;
    tcuWaitExternalSemaphoresAsync *wait_external_semaphores_async;
    tcuSignalExternalSemaphoresAsync *signal_external_semaphores_async;
} FluxaCudaApi;

struct FluxaNvdecDecoder {
    AVCodecContext *codec_context;
    AVFrame *frame;
    CUcontext cuda_context;
    FluxaCudaApi cuda;
    CUexternalMemory imported_memory;
    CUdeviceptr imported_buffer;
    size_t imported_buffer_size;
    CUexternalSemaphore imported_semaphore;
    CUexternalSemaphore release_semaphore;
    int release_semaphore_armed;
    int width;
    int height;
};

static int fluxa_nvdec_load_cuda(FluxaCudaApi *api)
{
    memset(api, 0, sizeof(*api));
    api->library = dlopen("libcuda.so.1", RTLD_NOW | RTLD_LOCAL);
    if (api->library == NULL) {
        api->library = dlopen("libcuda.so", RTLD_NOW | RTLD_LOCAL);
    }
    if (api->library == NULL) {
        return FLUXA_NVDEC_CUDA_ERROR;
    }

    *(void **)(&api->push_current) = dlsym(api->library, "cuCtxPushCurrent_v2");
    *(void **)(&api->pop_current) = dlsym(api->library, "cuCtxPopCurrent_v2");
    *(void **)(&api->context_get_device) = dlsym(api->library, "cuCtxGetDevice");
    *(void **)(&api->device_get_uuid) = dlsym(api->library, "cuDeviceGetUuid_v2");
    if (api->device_get_uuid == NULL) {
        *(void **)(&api->device_get_uuid) = dlsym(api->library, "cuDeviceGetUuid");
    }
    *(void **)(&api->memcpy_2d) = dlsym(api->library, "cuMemcpy2D_v2");
    *(void **)(&api->mem_alloc) = dlsym(api->library, "cuMemAlloc_v2");
    *(void **)(&api->mem_free) = dlsym(api->library, "cuMemFree_v2");
    *(void **)(&api->import_external_memory) = dlsym(api->library, "cuImportExternalMemory");
    *(void **)(&api->destroy_external_memory) = dlsym(api->library, "cuDestroyExternalMemory");
    *(void **)(&api->map_external_buffer) = dlsym(api->library, "cuExternalMemoryGetMappedBuffer");
    /* Semaphores are optional so systems with an older CUDA driver can still
     * use the decoder and the CPU fallback. The import API reports unsupported. */
    *(void **)(&api->import_external_semaphore) = dlsym(api->library, "cuImportExternalSemaphore");
    *(void **)(&api->destroy_external_semaphore) = dlsym(api->library, "cuDestroyExternalSemaphore");
    *(void **)(&api->wait_external_semaphores_async) = dlsym(api->library, "cuWaitExternalSemaphoresAsync");
    *(void **)(&api->signal_external_semaphores_async) = dlsym(api->library, "cuSignalExternalSemaphoresAsync");
    if (api->push_current == NULL || api->pop_current == NULL ||
        api->memcpy_2d == NULL || api->mem_alloc == NULL ||
        api->mem_free == NULL || api->import_external_memory == NULL ||
        api->destroy_external_memory == NULL || api->map_external_buffer == NULL) {
        dlclose(api->library);
        memset(api, 0, sizeof(*api));
        return FLUXA_NVDEC_CUDA_ERROR;
    }
    return FLUXA_NVDEC_OK;
}

static enum AVPixelFormat fluxa_nvdec_get_format(AVCodecContext *context,
                                                 const enum AVPixelFormat *formats)
{
    (void)context;
    for (const enum AVPixelFormat *format = formats;
         *format != AV_PIX_FMT_NONE;
         ++format) {
        if (*format == AV_PIX_FMT_CUDA) {
            return *format;
        }
    }
    return AV_PIX_FMT_NONE;
}

void fluxa_nvdec_destroy(FluxaNvdecDecoder *decoder)
{
    if (decoder == NULL) {
        return;
    }

    if (decoder->cuda_context != NULL &&
        (decoder->imported_memory != NULL || decoder->imported_semaphore != NULL ||
         decoder->release_semaphore != NULL)) {
        CUcontext popped_context = NULL;
        if (decoder->cuda.push_current(decoder->cuda_context) == CUDA_SUCCESS) {
            if (decoder->imported_semaphore != NULL &&
                decoder->cuda.destroy_external_semaphore != NULL) {
                decoder->cuda.destroy_external_semaphore(decoder->imported_semaphore);
                decoder->imported_semaphore = NULL;
            }
            if (decoder->release_semaphore != NULL &&
                decoder->cuda.destroy_external_semaphore != NULL) {
                decoder->cuda.destroy_external_semaphore(decoder->release_semaphore);
                decoder->release_semaphore = NULL;
            }
            if (decoder->imported_buffer != 0) {
                decoder->cuda.mem_free(decoder->imported_buffer);
                decoder->imported_buffer = 0;
            }
            if (decoder->imported_memory != NULL) {
                decoder->cuda.destroy_external_memory(decoder->imported_memory);
                decoder->imported_memory = NULL;
            }
            decoder->cuda.pop_current(&popped_context);
        }
    }
    av_frame_free(&decoder->frame);
    avcodec_free_context(&decoder->codec_context);
    if (decoder->cuda.library != NULL) {
        dlclose(decoder->cuda.library);
    }
    free(decoder);
}

int fluxa_nvdec_create(FluxaNvdecDecoder **out_decoder, int width, int height)
{
    if (out_decoder == NULL) {
        return FLUXA_NVDEC_INVALID_ARGUMENT;
    }
    *out_decoder = NULL;

    if (width <= 0 || height <= 0) {
        return FLUXA_NVDEC_INVALID_ARGUMENT;
    }

    FluxaNvdecDecoder *decoder = calloc(1, sizeof(*decoder));
    if (decoder == NULL) {
        return FLUXA_NVDEC_OUT_OF_MEMORY;
    }
    decoder->width = width;
    decoder->height = height;

    if (fluxa_nvdec_load_cuda(&decoder->cuda) != FLUXA_NVDEC_OK) {
        fluxa_nvdec_destroy(decoder);
        return FLUXA_NVDEC_CUDA_ERROR;
    }

    const AVCodec *codec = avcodec_find_decoder_by_name("vp8_cuvid");
    if (codec == NULL) {
        fluxa_nvdec_destroy(decoder);
        return FLUXA_NVDEC_FFMPEG_ERROR;
    }

    AVBufferRef *device_ref = NULL;
    if (av_hwdevice_ctx_create(&device_ref, AV_HWDEVICE_TYPE_CUDA,
                               NULL, NULL, 0) < 0) {
        fluxa_nvdec_destroy(decoder);
        return FLUXA_NVDEC_CUDA_ERROR;
    }

    AVHWDeviceContext *device_context =
        (AVHWDeviceContext *)device_ref->data;
    if (device_context == NULL || device_context->hwctx == NULL) {
        av_buffer_unref(&device_ref);
        fluxa_nvdec_destroy(decoder);
        return FLUXA_NVDEC_CUDA_ERROR;
    }
    AVCUDADeviceContext *cuda_device_context =
        (AVCUDADeviceContext *)device_context->hwctx;
    decoder->cuda_context = cuda_device_context->cuda_ctx;
    if (decoder->cuda_context == NULL) {
        av_buffer_unref(&device_ref);
        fluxa_nvdec_destroy(decoder);
        return FLUXA_NVDEC_CUDA_ERROR;
    }

    decoder->codec_context = avcodec_alloc_context3(codec);
    if (decoder->codec_context == NULL) {
        av_buffer_unref(&device_ref);
        fluxa_nvdec_destroy(decoder);
        return FLUXA_NVDEC_OUT_OF_MEMORY;
    }

    decoder->codec_context->width = width;
    decoder->codec_context->height = height;
    decoder->codec_context->pkt_timebase = (AVRational){ .num = 1, .den = 1000 };
    decoder->codec_context->time_base = decoder->codec_context->pkt_timebase;
    decoder->codec_context->sample_aspect_ratio = (AVRational){ .num = 1, .den = 1 };
    decoder->codec_context->get_format = fluxa_nvdec_get_format;
    decoder->codec_context->hw_device_ctx = device_ref;
    device_ref = NULL; /* codec_context now owns the reference */

    if (avcodec_open2(decoder->codec_context, codec, NULL) < 0) {
        fluxa_nvdec_destroy(decoder);
        return FLUXA_NVDEC_FFMPEG_ERROR;
    }

    decoder->frame = av_frame_alloc();
    if (decoder->frame == NULL) {
        fluxa_nvdec_destroy(decoder);
        return FLUXA_NVDEC_OUT_OF_MEMORY;
    }

    *out_decoder = decoder;
    return FLUXA_NVDEC_OK;
}

static int fluxa_nvdec_copy_rect(const FluxaCudaApi *cuda,
                                 CUdeviceptr source,
                                 size_t source_pitch,
                                 size_t source_x,
                                 size_t source_y,
                                 CUdeviceptr destination,
                                 size_t destination_pitch,
                                 size_t destination_x,
                                 size_t destination_y,
                                 size_t row_bytes,
                                 size_t rows)
{
    CUDA_MEMCPY2D copy;
    memset(&copy, 0, sizeof(copy));
    copy.srcMemoryType = CU_MEMORYTYPE_DEVICE;
    copy.srcDevice = source;
    copy.srcPitch = source_pitch;
    copy.srcXInBytes = source_x;
    copy.srcY = source_y;
    copy.dstMemoryType = CU_MEMORYTYPE_DEVICE;
    copy.dstDevice = destination;
    copy.dstPitch = destination_pitch;
    copy.dstXInBytes = destination_x;
    copy.dstY = destination_y;
    copy.WidthInBytes = row_bytes;
    copy.Height = rows;
    return cuda->memcpy_2d(&copy) == CUDA_SUCCESS
        ? FLUXA_NVDEC_OK
        : FLUXA_NVDEC_CUDA_COPY_ERROR;
}

static int fluxa_nvdec_signal_output(FluxaNvdecDecoder *decoder)
{
    if (decoder->imported_semaphore == NULL) {
        return FLUXA_NVDEC_OK;
    }
    if (decoder->cuda.signal_external_semaphores_async == NULL) {
        return FLUXA_NVDEC_CUDA_SEMAPHORE_UNSUPPORTED;
    }

    /* Vulkan OPAQUE_FD binary semaphores do not take a payload value. Keep
     * every field, including reserved words and flags, zero per the CUDA ABI. */
    CUDA_EXTERNAL_SEMAPHORE_SIGNAL_PARAMS signal_params;
    memset(&signal_params, 0, sizeof(signal_params));
    const CUexternalSemaphore semaphore = decoder->imported_semaphore;
    return decoder->cuda.signal_external_semaphores_async(
               &semaphore, &signal_params, 1, (CUstream)0) == CUDA_SUCCESS
        ? FLUXA_NVDEC_OK
        : FLUXA_NVDEC_CUDA_SEMAPHORE_ERROR;
}

static int fluxa_nvdec_wait_until_released(FluxaNvdecDecoder *decoder)
{
    if (!decoder->release_semaphore_armed) {
        return FLUXA_NVDEC_OK;
    }
    if (decoder->release_semaphore == NULL ||
        decoder->cuda.wait_external_semaphores_async == NULL) {
        return FLUXA_NVDEC_CUDA_SEMAPHORE_UNSUPPORTED;
    }

    CUDA_EXTERNAL_SEMAPHORE_WAIT_PARAMS wait_params;
    memset(&wait_params, 0, sizeof(wait_params));
    const CUexternalSemaphore semaphore = decoder->release_semaphore;
    if (decoder->cuda.wait_external_semaphores_async(
            &semaphore, &wait_params, 1, (CUstream)0) != CUDA_SUCCESS) {
        return FLUXA_NVDEC_CUDA_SEMAPHORE_ERROR;
    }
    decoder->release_semaphore_armed = 0;
    return FLUXA_NVDEC_OK;
}

int fluxa_nvdec_receive_frame(FluxaNvdecDecoder *decoder,
                              CUdeviceptr destination,
                              size_t destination_pitch)
{
    if (decoder == NULL) {
        return FLUXA_NVDEC_INVALID_ARGUMENT;
    }
    const size_t target_width = (size_t)decoder->width;
    const size_t target_height = (size_t)decoder->height;
    const size_t target_uv_width = (target_width + 1u) & ~(size_t)1u;
    const size_t target_uv_rows = (target_height + 1u) / 2u;
    if (destination == 0 ||
        destination_pitch < target_uv_width ||
        destination_pitch >
            SIZE_MAX / (target_height + target_uv_rows)) {
        return FLUXA_NVDEC_INVALID_ARGUMENT;
    }

    const size_t uv_offset = destination_pitch * target_height;
    if ((uint64_t)destination > UINT64_MAX - (uint64_t)uv_offset) {
        return FLUXA_NVDEC_INVALID_ARGUMENT;
    }

    av_frame_unref(decoder->frame);
    int result = FLUXA_NVDEC_FFMPEG_ERROR;
    int receive_result = avcodec_receive_frame(decoder->codec_context,
                                               decoder->frame);
    if (receive_result == AVERROR(EAGAIN) || receive_result == AVERROR_EOF) {
        return FLUXA_NVDEC_NO_FRAME;
    }
    if (receive_result < 0) {
        return FLUXA_NVDEC_FFMPEG_ERROR;
    }

    AVFrame *frame = decoder->frame;
    const size_t source_width = frame->width > 0 ? (size_t)frame->width : 0;
    const size_t source_height = frame->height > 0 ? (size_t)frame->height : 0;
    const size_t source_uv_width = (source_width + 1u) & ~(size_t)1u;
    const size_t source_uv_rows = (source_height + 1u) / 2u;
    if (frame->format != AV_PIX_FMT_CUDA ||
        source_width == 0 || source_height == 0 ||
        source_width > target_width || source_height > target_height ||
        target_width - source_width > 1u || target_height - source_height > 1u ||
        (target_width != source_width && (target_width & 1u) == 0) ||
        (target_height != source_height && (target_height & 1u) == 0) ||
        frame->data[0] == NULL || frame->data[1] == NULL ||
        frame->linesize[0] < frame->width ||
        frame->linesize[1] < (int)source_uv_width || frame->hw_frames_ctx == NULL) {
#ifdef FLUXA_NVDEC_DIAGNOSTICS
        fprintf(stderr, "NVDEC frame metadata mismatch: format=%d size=%dx%d requested=%dx%d lines=%d,%d hwctx=%p\n",
                frame->format, frame->width, frame->height, decoder->width,
                decoder->height, frame->linesize[0], frame->linesize[1],
                (void *)frame->hw_frames_ctx);
#endif
        return FLUXA_NVDEC_UNSUPPORTED_FRAME;
    }

    AVHWFramesContext *frames_context =
        (AVHWFramesContext *)frame->hw_frames_ctx->data;
    if (frames_context == NULL || frames_context->sw_format != AV_PIX_FMT_NV12) {
#ifdef FLUXA_NVDEC_DIAGNOSTICS
        fprintf(stderr, "NVDEC frame software format mismatch: %d (expected %d)\n",
                frames_context != NULL ? frames_context->sw_format : -1,
                AV_PIX_FMT_NV12);
#endif
        return FLUXA_NVDEC_UNSUPPORTED_FRAME;
    }

    CUcontext popped_context = NULL;
    if (decoder->cuda.push_current(decoder->cuda_context) != CUDA_SUCCESS) {
        return FLUXA_NVDEC_CUDA_CONTEXT_ERROR;
    }

    result = fluxa_nvdec_wait_until_released(decoder);
    if (result != FLUXA_NVDEC_OK) {
        CUcontext popped_context = NULL;
        decoder->cuda.pop_current(&popped_context);
        return result;
    }

    const CUdeviceptr source_y = (CUdeviceptr)(uintptr_t)frame->data[0];
    const CUdeviceptr source_uv = (CUdeviceptr)(uintptr_t)frame->data[1];
    const size_t y_width = source_width;
    const size_t y_rows = source_height;
    const size_t uv_width = source_uv_width;
    result = fluxa_nvdec_copy_rect(&decoder->cuda, source_y,
                                   (size_t)frame->linesize[0], 0, 0,
                                   destination, destination_pitch, 0, 0,
                                   y_width, y_rows);
    if (result == FLUXA_NVDEC_OK && target_height > source_height) {
        result = fluxa_nvdec_copy_rect(&decoder->cuda, source_y,
                                      (size_t)frame->linesize[0], 0,
                                      source_height - 1u,
                                      destination, destination_pitch, 0,
                                      source_height,
                                      y_width, 1);
    }
    if (result == FLUXA_NVDEC_OK && target_width > source_width) {
        result = fluxa_nvdec_copy_rect(&decoder->cuda, source_y,
                                      (size_t)frame->linesize[0],
                                      source_width - 1u, 0,
                                      destination, destination_pitch,
                                      source_width, 0, 1,
                                      source_height);
    }
    if (result == FLUXA_NVDEC_OK && target_width > source_width &&
        target_height > source_height) {
        result = fluxa_nvdec_copy_rect(&decoder->cuda, source_y,
                                      (size_t)frame->linesize[0],
                                      source_width - 1u, source_height - 1u,
                                      destination, destination_pitch,
                                      source_width, source_height, 1, 1);
    }
    if (result == FLUXA_NVDEC_OK) {
        result = fluxa_nvdec_copy_rect(&decoder->cuda, source_uv,
                                      (size_t)frame->linesize[1], 0, 0,
                                      destination + (CUdeviceptr)uv_offset,
                                      destination_pitch, 0, 0,
                                      uv_width, source_uv_rows);
    }
    if (result == FLUXA_NVDEC_OK && target_uv_rows > source_uv_rows) {
        result = fluxa_nvdec_copy_rect(&decoder->cuda, source_uv,
                                      (size_t)frame->linesize[1], 0,
                                      source_uv_rows - 1u,
                                      destination + (CUdeviceptr)uv_offset,
                                      destination_pitch, 0, source_uv_rows,
                                      uv_width, 1);
    }
    if (result == FLUXA_NVDEC_OK && target_uv_width > source_uv_width) {
        result = fluxa_nvdec_copy_rect(&decoder->cuda, source_uv,
                                      (size_t)frame->linesize[1],
                                      source_uv_width - 2u, 0,
                                      destination + (CUdeviceptr)uv_offset,
                                      destination_pitch, source_uv_width, 0,
                                      2, source_uv_rows);
    }
    if (result == FLUXA_NVDEC_OK && target_uv_width > source_uv_width &&
        target_uv_rows > source_uv_rows) {
        result = fluxa_nvdec_copy_rect(&decoder->cuda, source_uv,
                                      (size_t)frame->linesize[1],
                                      source_uv_width - 2u, source_uv_rows - 1u,
                                      destination + (CUdeviceptr)uv_offset,
                                      destination_pitch, source_uv_width,
                                      source_uv_rows, 2, 1);
    }

    /* Signal only after every Y/UV copy (including edge completion) succeeded.
     * The semaphore signal is queued on CUDA's default stream. */
    if (result == FLUXA_NVDEC_OK) {
        result = fluxa_nvdec_signal_output(decoder);
        if (result == FLUXA_NVDEC_OK && decoder->release_semaphore != NULL) {
            decoder->release_semaphore_armed = 1;
        }
    }

    if (decoder->cuda.pop_current(&popped_context) != CUDA_SUCCESS) {
        result = FLUXA_NVDEC_CUDA_CONTEXT_ERROR;
    }
    return result;
}

int fluxa_nvdec_send_packet(FluxaNvdecDecoder *decoder,
                            const uint8_t *packet_data,
                            size_t packet_size)
{
    if (decoder == NULL || packet_data == NULL || packet_size == 0 ||
        packet_size > INT_MAX) {
        return FLUXA_NVDEC_INVALID_ARGUMENT;
    }
    AVPacket *packet = av_packet_alloc();
    if (packet == NULL) {
        return FLUXA_NVDEC_OUT_OF_MEMORY;
    }
    if (av_new_packet(packet, (int)packet_size) < 0) {
        av_packet_free(&packet);
        return FLUXA_NVDEC_OUT_OF_MEMORY;
    }
    memcpy(packet->data, packet_data, packet_size);

    int send_result = avcodec_send_packet(decoder->codec_context, packet);
    av_packet_free(&packet);
    if (send_result < 0) {
        return send_result == AVERROR(EAGAIN)
            ? FLUXA_NVDEC_NO_FRAME
            : FLUXA_NVDEC_FFMPEG_ERROR;
    }
    return FLUXA_NVDEC_OK;
}

int fluxa_nvdec_send_end(FluxaNvdecDecoder *decoder)
{
    if (decoder == NULL) {
        return FLUXA_NVDEC_INVALID_ARGUMENT;
    }
    int send_result = avcodec_send_packet(decoder->codec_context, NULL);
    if (send_result < 0 && send_result != AVERROR_EOF) {
        return send_result == AVERROR(EAGAIN)
            ? FLUXA_NVDEC_NO_FRAME
            : FLUXA_NVDEC_FFMPEG_ERROR;
    }
    return FLUXA_NVDEC_OK;
}

int fluxa_nvdec_alloc_output(FluxaNvdecDecoder *decoder,
                             size_t size,
                             CUdeviceptr *out_memory)
{
    if (decoder == NULL || size == 0 || out_memory == NULL) {
        return FLUXA_NVDEC_INVALID_ARGUMENT;
    }
    *out_memory = 0;
    if (decoder->cuda.push_current(decoder->cuda_context) != CUDA_SUCCESS) {
        return FLUXA_NVDEC_CUDA_CONTEXT_ERROR;
    }
    int result = decoder->cuda.mem_alloc(out_memory, size) == CUDA_SUCCESS
        ? FLUXA_NVDEC_OK
        : FLUXA_NVDEC_OUT_OF_MEMORY;
    CUcontext popped_context = NULL;
    if (decoder->cuda.pop_current(&popped_context) != CUDA_SUCCESS) {
        if (*out_memory != 0) {
            decoder->cuda.push_current(decoder->cuda_context);
            decoder->cuda.mem_free(*out_memory);
            decoder->cuda.pop_current(&popped_context);
            *out_memory = 0;
        }
        return FLUXA_NVDEC_CUDA_CONTEXT_ERROR;
    }
    return result;
}

int fluxa_nvdec_free_output(FluxaNvdecDecoder *decoder, CUdeviceptr memory)
{
    if (decoder == NULL || memory == 0) {
        return FLUXA_NVDEC_INVALID_ARGUMENT;
    }
    if (decoder->cuda.push_current(decoder->cuda_context) != CUDA_SUCCESS) {
        return FLUXA_NVDEC_CUDA_CONTEXT_ERROR;
    }
    int result = decoder->cuda.mem_free(memory) == CUDA_SUCCESS
        ? FLUXA_NVDEC_OK
        : FLUXA_NVDEC_CUDA_ERROR;
    CUcontext popped_context = NULL;
    if (decoder->cuda.pop_current(&popped_context) != CUDA_SUCCESS) {
        return FLUXA_NVDEC_CUDA_CONTEXT_ERROR;
    }
    return result;
}

int fluxa_nvdec_import_vulkan_buffer(FluxaNvdecDecoder *decoder,
                                     int fd,
                                     size_t allocation_size,
                                     size_t buffer_size,
                                     CUdeviceptr *out_memory)
{
    if (decoder == NULL || fd < 0 || allocation_size == 0 || buffer_size == 0 ||
        buffer_size > allocation_size || out_memory == NULL ||
        decoder->imported_memory != NULL) {
        return FLUXA_NVDEC_INVALID_ARGUMENT;
    }
    *out_memory = 0;

    /* Keep ownership of the export FD in Rust; CUDA takes ownership of the
     * descriptor passed to a successful OPAQUE_FD import. */
    int cuda_fd = dup(fd);
    if (cuda_fd < 0) {
        return FLUXA_NVDEC_CUDA_ERROR;
    }

    CUDA_EXTERNAL_MEMORY_HANDLE_DESC handle_desc;
    memset(&handle_desc, 0, sizeof(handle_desc));
    handle_desc.type = CU_EXTERNAL_MEMORY_HANDLE_TYPE_OPAQUE_FD;
    handle_desc.handle.fd = cuda_fd;
    handle_desc.size = (unsigned long long)allocation_size;

    CUDA_EXTERNAL_MEMORY_BUFFER_DESC buffer_desc;
    memset(&buffer_desc, 0, sizeof(buffer_desc));
    buffer_desc.size = (unsigned long long)buffer_size;

    if (decoder->cuda.push_current(decoder->cuda_context) != CUDA_SUCCESS) {
        close(cuda_fd);
        return FLUXA_NVDEC_CUDA_CONTEXT_ERROR;
    }
    int result = FLUXA_NVDEC_CUDA_ERROR;
    CUresult import_result = decoder->cuda.import_external_memory(
        &decoder->imported_memory, &handle_desc);
    if (import_result == CUDA_SUCCESS) {
        /* Successful import transfers cuda_fd ownership to CUDA. */
        if (decoder->cuda.map_external_buffer(&decoder->imported_buffer,
                                              decoder->imported_memory,
                                              &buffer_desc) == CUDA_SUCCESS) {
            decoder->imported_buffer_size = buffer_size;
            *out_memory = decoder->imported_buffer;
            result = FLUXA_NVDEC_OK;
        } else {
            decoder->cuda.destroy_external_memory(decoder->imported_memory);
            decoder->imported_memory = NULL;
            decoder->imported_buffer = 0;
        }
    } else {
        close(cuda_fd);
    }
    CUcontext popped_context = NULL;
    if (decoder->cuda.pop_current(&popped_context) != CUDA_SUCCESS) {
        *out_memory = 0;
        return FLUXA_NVDEC_CUDA_CONTEXT_ERROR;
    }
    return result;
}

int fluxa_nvdec_import_vulkan_semaphore(FluxaNvdecDecoder *decoder, int fd)
{
    if (decoder == NULL || fd < 0 || decoder->imported_semaphore != NULL) {
        return FLUXA_NVDEC_INVALID_ARGUMENT;
    }
    if (decoder->cuda.import_external_semaphore == NULL ||
        decoder->cuda.destroy_external_semaphore == NULL ||
        decoder->cuda.signal_external_semaphores_async == NULL) {
        return FLUXA_NVDEC_CUDA_SEMAPHORE_UNSUPPORTED;
    }

    /* CUDA takes ownership of an OPAQUE_FD after successful import. Duplicate
     * it so ownership of the caller's descriptor remains unchanged. */
    int cuda_fd = dup(fd);
    if (cuda_fd < 0) {
        return FLUXA_NVDEC_CUDA_SEMAPHORE_ERROR;
    }

    CUDA_EXTERNAL_SEMAPHORE_HANDLE_DESC handle_desc;
    memset(&handle_desc, 0, sizeof(handle_desc));
    handle_desc.type = CU_EXTERNAL_SEMAPHORE_HANDLE_TYPE_OPAQUE_FD;
    handle_desc.handle.fd = cuda_fd;

    if (decoder->cuda.push_current(decoder->cuda_context) != CUDA_SUCCESS) {
        close(cuda_fd);
        return FLUXA_NVDEC_CUDA_CONTEXT_ERROR;
    }

    CUexternalSemaphore imported_semaphore = NULL;
    CUresult import_result = decoder->cuda.import_external_semaphore(
        &imported_semaphore, &handle_desc);
    if (import_result == CUDA_SUCCESS && imported_semaphore != NULL) {
        /* Successful import transfers cuda_fd ownership to CUDA. */
        decoder->imported_semaphore = imported_semaphore;
    } else if (import_result != CUDA_SUCCESS) {
        close(cuda_fd);
    }

    CUcontext popped_context = NULL;
    if (decoder->cuda.pop_current(&popped_context) != CUDA_SUCCESS) {
        return FLUXA_NVDEC_CUDA_CONTEXT_ERROR;
    }
    return import_result == CUDA_SUCCESS && imported_semaphore != NULL
        ? FLUXA_NVDEC_OK
        : FLUXA_NVDEC_CUDA_SEMAPHORE_ERROR;
}

int fluxa_nvdec_import_vulkan_release_semaphore(FluxaNvdecDecoder *decoder, int fd)
{
    if (decoder == NULL || fd < 0 || decoder->release_semaphore != NULL) {
        return FLUXA_NVDEC_INVALID_ARGUMENT;
    }
    if (decoder->cuda.import_external_semaphore == NULL ||
        decoder->cuda.destroy_external_semaphore == NULL ||
        decoder->cuda.wait_external_semaphores_async == NULL) {
        return FLUXA_NVDEC_CUDA_SEMAPHORE_UNSUPPORTED;
    }

    int cuda_fd = dup(fd);
    if (cuda_fd < 0) {
        return FLUXA_NVDEC_CUDA_SEMAPHORE_ERROR;
    }
    CUDA_EXTERNAL_SEMAPHORE_HANDLE_DESC handle_desc;
    memset(&handle_desc, 0, sizeof(handle_desc));
    handle_desc.type = CU_EXTERNAL_SEMAPHORE_HANDLE_TYPE_OPAQUE_FD;
    handle_desc.handle.fd = cuda_fd;
    if (decoder->cuda.push_current(decoder->cuda_context) != CUDA_SUCCESS) {
        close(cuda_fd);
        return FLUXA_NVDEC_CUDA_CONTEXT_ERROR;
    }

    CUexternalSemaphore imported = NULL;
    CUresult import_result = decoder->cuda.import_external_semaphore(&imported, &handle_desc);
    if (import_result == CUDA_SUCCESS && imported != NULL) {
        decoder->release_semaphore = imported;
    } else {
        close(cuda_fd);
    }
    CUcontext popped_context = NULL;
    if (decoder->cuda.pop_current(&popped_context) != CUDA_SUCCESS) {
        return FLUXA_NVDEC_CUDA_CONTEXT_ERROR;
    }
    return import_result == CUDA_SUCCESS && imported != NULL
        ? FLUXA_NVDEC_OK
        : FLUXA_NVDEC_CUDA_SEMAPHORE_ERROR;
}

int fluxa_nvdec_cuda_device_uuid(FluxaNvdecDecoder *decoder, uint8_t out_uuid[16])
{
    if (decoder == NULL || out_uuid == NULL ||
        decoder->cuda.context_get_device == NULL || decoder->cuda.device_get_uuid == NULL) {
        return FLUXA_NVDEC_INVALID_ARGUMENT;
    }
    if (decoder->cuda.push_current(decoder->cuda_context) != CUDA_SUCCESS) {
        return FLUXA_NVDEC_CUDA_CONTEXT_ERROR;
    }
    CUdevice device;
    CUuuid uuid;
    CUresult result = decoder->cuda.context_get_device(&device);
    if (result == CUDA_SUCCESS) {
        result = decoder->cuda.device_get_uuid(&uuid, device);
    }
    CUcontext popped_context = NULL;
    if (decoder->cuda.pop_current(&popped_context) != CUDA_SUCCESS) {
        return FLUXA_NVDEC_CUDA_CONTEXT_ERROR;
    }
    if (result != CUDA_SUCCESS) {
        return FLUXA_NVDEC_CUDA_ERROR;
    }
    memcpy(out_uuid, uuid.bytes, sizeof(uuid.bytes));
    return FLUXA_NVDEC_OK;
}
