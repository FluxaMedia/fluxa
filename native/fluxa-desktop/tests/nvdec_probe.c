/*
 * Optional hardware smoke test for nvdec_ffmpeg.c.
 *
 * Usage: nvdec_probe one-frame.ivf
 * The IVF must contain one even-sized VP8 frame. This executable intentionally
 * validates decode/copy only; it does not read the destination pixels back.
 */
#include "../src/nvdec_ffmpeg.h"

#include <stdio.h>
#include <stdlib.h>

static uint32_t read_u32_le(const uint8_t *bytes)
{
    return (uint32_t)bytes[0] | ((uint32_t)bytes[1] << 8) |
           ((uint32_t)bytes[2] << 16) | ((uint32_t)bytes[3] << 24);
}

static uint16_t read_u16_le(const uint8_t *bytes)
{
    return (uint16_t)bytes[0] | ((uint16_t)bytes[1] << 8);
}

static int drain_frames(FluxaNvdecDecoder *decoder,
                        CUdeviceptr output,
                        size_t pitch,
                        unsigned int *decoded_frames)
{
    for (;;) {
        const int result = fluxa_nvdec_receive_frame(decoder, output, pitch);
        if (result == FLUXA_NVDEC_NO_FRAME) {
            return FLUXA_NVDEC_OK;
        }
        if (result != FLUXA_NVDEC_OK) {
            return result;
        }
        ++*decoded_frames;
    }
}

int main(int argc, char **argv)
{
    if (argc != 2) {
        fprintf(stderr, "usage: %s one-frame.ivf\n", argv[0]);
        return 2;
    }

    FILE *file = fopen(argv[1], "rb");
    if (file == NULL || fseek(file, 0, SEEK_END) != 0) {
        perror("open IVF");
        return 2;
    }
    long file_size = ftell(file);
    if (file_size < 44 || fseek(file, 0, SEEK_SET) != 0) {
        fprintf(stderr, "IVF file is truncated\n");
        fclose(file);
        return 2;
    }
    uint8_t *ivf = malloc((size_t)file_size);
    if (ivf == NULL || fread(ivf, 1, (size_t)file_size, file) != (size_t)file_size) {
        fprintf(stderr, "failed to read IVF file\n");
        free(ivf);
        fclose(file);
        return 2;
    }
    fclose(file);

    if (ivf[0] != 'D' || ivf[1] != 'K' || ivf[2] != 'I' || ivf[3] != 'F') {
        fprintf(stderr, "input is not IVF\n");
        free(ivf);
        return 2;
    }
    const int width = (int)read_u16_le(&ivf[12]);
    const int height = (int)read_u16_le(&ivf[14]);
    const uint32_t frame_count = read_u32_le(&ivf[24]);
    if (frame_count == 0) {
        fprintf(stderr, "IVF has no frames\n");
        free(ivf);
        return 2;
    }

    const size_t pitch = ((size_t)width + 255u) & ~255u;
    const size_t output_size = pitch * ((size_t)height + ((size_t)height + 1u) / 2u);
    CUdeviceptr output = 0;
    FluxaNvdecDecoder *decoder = NULL;
    int result = fluxa_nvdec_create(&decoder, width, height);
    if (result == FLUXA_NVDEC_OK) {
        result = fluxa_nvdec_alloc_output(decoder, output_size, &output);
    }

    size_t offset = 32;
    unsigned int decoded_frames = 0;
    for (uint32_t frame_index = 0;
         result == FLUXA_NVDEC_OK && frame_index < frame_count;
         ++frame_index) {
        if (offset + 12 > (size_t)file_size) {
            result = FLUXA_NVDEC_INVALID_ARGUMENT;
            break;
        }
        const uint32_t packet_size = read_u32_le(&ivf[offset]);
        offset += 12;
        if (packet_size == 0 || packet_size > (size_t)file_size - offset) {
            result = FLUXA_NVDEC_INVALID_ARGUMENT;
            break;
        }
        for (;;) {
            result = fluxa_nvdec_send_packet(decoder, &ivf[offset], packet_size);
            if (result != FLUXA_NVDEC_NO_FRAME) {
                break;
            }
            result = drain_frames(decoder, output, pitch, &decoded_frames);
            if (result != FLUXA_NVDEC_OK) {
                break;
            }
        }
        offset += packet_size;
        if (result == FLUXA_NVDEC_OK) {
            result = drain_frames(decoder, output, pitch, &decoded_frames);
        }
    }
    if (result == FLUXA_NVDEC_OK) {
        for (;;) {
            const int send_result = fluxa_nvdec_send_end(decoder);
            if (send_result != FLUXA_NVDEC_NO_FRAME) {
                result = send_result;
                break;
            }
            result = drain_frames(decoder, output, pitch, &decoded_frames);
            if (result != FLUXA_NVDEC_OK) {
                break;
            }
        }
    }
    if (result == FLUXA_NVDEC_OK) {
        result = drain_frames(decoder, output, pitch, &decoded_frames);
    }

    if (output != 0) {
        const int free_result = fluxa_nvdec_free_output(decoder, output);
        if (result == FLUXA_NVDEC_OK && free_result != FLUXA_NVDEC_OK) {
            result = free_result;
        }
    }
    fluxa_nvdec_destroy(decoder);
    free(ivf);
    if (result != FLUXA_NVDEC_OK) {
        fprintf(stderr, "NVDEC decode failed: %d\n", result);
        return 1;
    }
    printf("NVDEC decoded %u/%u VP8 frames at %dx%d into CUDA NV12 memory (%zu bytes)\n",
           decoded_frames, frame_count, width, height, output_size);
    if (decoded_frames != frame_count) {
        return 1;
    }
    return 0;
}
