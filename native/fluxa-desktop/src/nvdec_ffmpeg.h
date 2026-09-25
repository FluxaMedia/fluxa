#ifndef FLUXA_NVDEC_FFMPEG_H
#define FLUXA_NVDEC_FFMPEG_H

#include <stddef.h>
#include <stdint.h>

#include <ffnvcodec/dynlink_cuda.h>

#ifdef __cplusplus
extern "C" {
#endif

typedef struct FluxaNvdecDecoder FluxaNvdecDecoder;

/* API results: zero means success; failures are negative. */
enum FluxaNvdecResult {
    FLUXA_NVDEC_OK = 0,
    FLUXA_NVDEC_INVALID_ARGUMENT = -1,
    FLUXA_NVDEC_OUT_OF_MEMORY = -2,
    FLUXA_NVDEC_FFMPEG_ERROR = -3,
    FLUXA_NVDEC_CUDA_ERROR = -4,
    FLUXA_NVDEC_NO_FRAME = -5,
    FLUXA_NVDEC_UNSUPPORTED_FRAME = -6,
    FLUXA_NVDEC_CUDA_CONTEXT_ERROR = -7,
    FLUXA_NVDEC_CUDA_COPY_ERROR = -8,
    FLUXA_NVDEC_CUDA_SEMAPHORE_ERROR = -9,
    FLUXA_NVDEC_CUDA_SEMAPHORE_UNSUPPORTED = -10
};

/* Create a VP8 CUVID decoder for fixed positive dimensions. */
int fluxa_nvdec_create(FluxaNvdecDecoder **out_decoder, int width, int height);

/* Submit complete raw VP8 frames, then drain receive_frame until NO_FRAME. */
int fluxa_nvdec_send_packet(FluxaNvdecDecoder *decoder,
                            const uint8_t *packet_data,
                            size_t packet_size);
/* Send end-of-stream; drain receive_frame until NO_FRAME afterward. */
int fluxa_nvdec_send_end(FluxaNvdecDecoder *decoder);
/* Copy one decoded NV12 image to CUDA memory; the destination needs
 * pitch * (height + ceil(height / 2)) bytes. NV12 UV rows have even byte
 * width, so pitch must be at least width rounded up to an even number. */
int fluxa_nvdec_receive_frame(FluxaNvdecDecoder *decoder,
                              CUdeviceptr destination,
                              size_t destination_pitch);

/* Allocate/free CUDA memory in the decoder's CUDA context. */
int fluxa_nvdec_alloc_output(FluxaNvdecDecoder *decoder,
                             size_t size,
                             CUdeviceptr *out_memory);
int fluxa_nvdec_free_output(FluxaNvdecDecoder *decoder,
                            CUdeviceptr memory);
/* Import a Vulkan opaque-FD buffer into the decoder's CUDA context. The
 * caller retains ownership of fd and must keep the Vulkan allocation alive. */
int fluxa_nvdec_import_vulkan_buffer(FluxaNvdecDecoder *decoder,
                                     int fd,
                                     size_t allocation_size,
                                     size_t buffer_size,
                                     CUdeviceptr *out_memory);

/* Import a Vulkan OPAQUE_FD binary semaphore. The caller retains ownership of
 * fd: the shim duplicates it before CUDA import. Once imported, every
 * successful receive_frame signals this semaphore after copying both NV12
 * planes. Before each receive_frame, Vulkan must have waited on/reset the
 * binary semaphore so it is unsignaled and ready for the next signal. The
 * Vulkan semaphore must outlive the decoder/import. */
int fluxa_nvdec_import_vulkan_semaphore(FluxaNvdecDecoder *decoder, int fd);
/* Import the reverse-direction binary semaphore. CUDA waits on this semaphore
 * before overwriting the shared NV12 buffer after its first frame. Vulkan
 * signals it after the GPU consumer has finished reading that frame. */
int fluxa_nvdec_import_vulkan_release_semaphore(FluxaNvdecDecoder *decoder, int fd);
/* Return the 16-byte UUID of the CUDA device backing this decoder context. */
int fluxa_nvdec_cuda_device_uuid(FluxaNvdecDecoder *decoder, uint8_t out_uuid[16]);

void fluxa_nvdec_destroy(FluxaNvdecDecoder *decoder);

#ifdef __cplusplus
}
#endif

#endif /* FLUXA_NVDEC_FFMPEG_H */
