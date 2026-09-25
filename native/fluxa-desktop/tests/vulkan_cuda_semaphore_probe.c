/*
 * Hardware smoke test for opaque-FD binary semaphore interoperability.
 *
 * Creates Vulkan ready/release semaphores, imports them into CUDA, signals
 * ready from CUDA, waits/signals on Vulkan, then waits for release in CUDA.
 * This tests driver interop only; it does not test wgpu queue integration.
 */
#define _GNU_SOURCE
#include <vulkan/vulkan.h>
#include <ffnvcodec/dynlink_cuda.h>

#include <dlfcn.h>
#include <stdio.h>
#include <stdlib.h>
#include <string.h>
#include <unistd.h>

typedef CUresult (*CuInitFn)(unsigned int);
typedef CUresult (*CuDeviceGetFn)(CUdevice *, int);
typedef CUresult (*CuCtxCreateFn)(CUcontext *, unsigned int, CUdevice);
typedef CUresult (*CuCtxDestroyFn)(CUcontext);
typedef CUresult (*CuCtxSynchronizeFn)(void);
typedef CUresult (*CuImportSemaphoreFn)(CUexternalSemaphore *,
                                        const CUDA_EXTERNAL_SEMAPHORE_HANDLE_DESC *);
typedef CUresult (*CuDestroySemaphoreFn)(CUexternalSemaphore);
typedef CUresult (*CuSignalSemaphoresFn)(const CUexternalSemaphore *,
                                        const CUDA_EXTERNAL_SEMAPHORE_SIGNAL_PARAMS *,
                                        unsigned int, CUstream);
typedef CUresult (*CuWaitSemaphoresFn)(const CUexternalSemaphore *,
                                     const CUDA_EXTERNAL_SEMAPHORE_WAIT_PARAMS *,
                                     unsigned int, CUstream);

#define CUDA_CHECK(call) do { \
    CUresult result_ = (call); \
    if (result_ != CUDA_SUCCESS) { \
        fprintf(stderr, "CUDA call failed at %s:%d: %s = %d\n", \
                __FILE__, __LINE__, #call, (int)result_); \
        goto cleanup; \
    } \
} while (0)

#define VK_CHECK(call) do { \
    VkResult result_ = (call); \
    if (result_ != VK_SUCCESS) { \
        fprintf(stderr, "Vulkan call failed at %s:%d: %s = %d\n", \
                __FILE__, __LINE__, #call, (int)result_); \
        goto cleanup; \
    } \
} while (0)

static int load_cuda_symbol(void *library, const char *name, void *target, size_t size)
{
    void *symbol = dlsym(library, name);
    if (symbol == NULL || size != sizeof(symbol)) {
        fprintf(stderr, "missing CUDA symbol %s\n", name);
        return 0;
    }
    memcpy(target, &symbol, size);
    return 1;
}

int main(void)
{
    int exit_code = 1;
    void *cuda_library = NULL;
    CUcontext cuda_context = NULL;
    CUexternalSemaphore cuda_ready = NULL;
    CUexternalSemaphore cuda_release = NULL;
    VkInstance instance = VK_NULL_HANDLE;
    VkDevice device = VK_NULL_HANDLE;
    VkSemaphore ready = VK_NULL_HANDLE;
    VkSemaphore release = VK_NULL_HANDLE;
    VkQueue queue = VK_NULL_HANDLE;
    int ready_fd = -1;
    int release_fd = -1;

    CuInitFn cuInit_fn = NULL;
    CuDeviceGetFn cuDeviceGet_fn = NULL;
    CuCtxCreateFn cuCtxCreate_fn = NULL;
    CuCtxDestroyFn cuCtxDestroy_fn = NULL;
    CuCtxSynchronizeFn cuCtxSynchronize_fn = NULL;
    CuImportSemaphoreFn cuImportExternalSemaphore_fn = NULL;
    CuDestroySemaphoreFn cuDestroyExternalSemaphore_fn = NULL;
    CuSignalSemaphoresFn cuSignalExternalSemaphoresAsync_fn = NULL;
    CuWaitSemaphoresFn cuWaitExternalSemaphoresAsync_fn = NULL;

    cuda_library = dlopen("libcuda.so.1", RTLD_NOW | RTLD_LOCAL);
    if (cuda_library == NULL ||
        !load_cuda_symbol(cuda_library, "cuInit", &cuInit_fn, sizeof(cuInit_fn)) ||
        !load_cuda_symbol(cuda_library, "cuDeviceGet", &cuDeviceGet_fn, sizeof(cuDeviceGet_fn)) ||
        !load_cuda_symbol(cuda_library, "cuCtxCreate_v2", &cuCtxCreate_fn, sizeof(cuCtxCreate_fn)) ||
        !load_cuda_symbol(cuda_library, "cuCtxDestroy_v2", &cuCtxDestroy_fn, sizeof(cuCtxDestroy_fn)) ||
        !load_cuda_symbol(cuda_library, "cuCtxSynchronize", &cuCtxSynchronize_fn, sizeof(cuCtxSynchronize_fn)) ||
        !load_cuda_symbol(cuda_library, "cuImportExternalSemaphore", &cuImportExternalSemaphore_fn, sizeof(cuImportExternalSemaphore_fn)) ||
        !load_cuda_symbol(cuda_library, "cuDestroyExternalSemaphore", &cuDestroyExternalSemaphore_fn, sizeof(cuDestroyExternalSemaphore_fn)) ||
        !load_cuda_symbol(cuda_library, "cuSignalExternalSemaphoresAsync", &cuSignalExternalSemaphoresAsync_fn, sizeof(cuSignalExternalSemaphoresAsync_fn)) ||
        !load_cuda_symbol(cuda_library, "cuWaitExternalSemaphoresAsync", &cuWaitExternalSemaphoresAsync_fn, sizeof(cuWaitExternalSemaphoresAsync_fn))) {
        goto cleanup;
    }
    CUDA_CHECK(cuInit_fn(0));

    VkApplicationInfo app_info = {
        .sType = VK_STRUCTURE_TYPE_APPLICATION_INFO,
        .pApplicationName = "Fluxa semaphore probe",
        .apiVersion = VK_API_VERSION_1_1,
    };
    VkInstanceCreateInfo instance_info = {
        .sType = VK_STRUCTURE_TYPE_INSTANCE_CREATE_INFO,
        .pApplicationInfo = &app_info,
    };
    VK_CHECK(vkCreateInstance(&instance_info, NULL, &instance));

    uint32_t physical_count = 0;
    VK_CHECK(vkEnumeratePhysicalDevices(instance, &physical_count, NULL));
    VkPhysicalDevice *physical_devices = calloc(physical_count, sizeof(*physical_devices));
    if (physical_devices == NULL || physical_count == 0) {
        fprintf(stderr, "no Vulkan physical devices found\n");
        free(physical_devices);
        goto cleanup;
    }
    VK_CHECK(vkEnumeratePhysicalDevices(instance, &physical_count, physical_devices));
    VkPhysicalDevice physical = VK_NULL_HANDLE;
    VkPhysicalDeviceProperties physical_properties = {0};
    for (uint32_t index = 0; index < physical_count; ++index) {
        VkPhysicalDeviceProperties candidate;
        vkGetPhysicalDeviceProperties(physical_devices[index], &candidate);
        if (candidate.vendorID == 0x10de) {
            physical = physical_devices[index];
            physical_properties = candidate;
            break;
        }
    }
    free(physical_devices);
    if (physical == VK_NULL_HANDLE) {
        fprintf(stderr, "no NVIDIA Vulkan device found\n");
        goto cleanup;
    }
    fprintf(stderr, "Vulkan device: %s\n", physical_properties.deviceName);

    uint32_t queue_family_count = 0;
    vkGetPhysicalDeviceQueueFamilyProperties(physical, &queue_family_count, NULL);
    VkQueueFamilyProperties *queue_families = calloc(queue_family_count, sizeof(*queue_families));
    if (queue_families == NULL || queue_family_count == 0) {
        fprintf(stderr, "no Vulkan queue families found\n");
        free(queue_families);
        goto cleanup;
    }
    vkGetPhysicalDeviceQueueFamilyProperties(physical, &queue_family_count, queue_families);
    uint32_t queue_family = UINT32_MAX;
    for (uint32_t index = 0; index < queue_family_count; ++index) {
        if ((queue_families[index].queueFlags & VK_QUEUE_COMPUTE_BIT) != 0) {
            queue_family = index;
            break;
        }
    }
    free(queue_families);
    if (queue_family == UINT32_MAX) {
        fprintf(stderr, "no Vulkan compute queue family found\n");
        goto cleanup;
    }

    const float priority = 1.0f;
    VkDeviceQueueCreateInfo queue_info = {
        .sType = VK_STRUCTURE_TYPE_DEVICE_QUEUE_CREATE_INFO,
        .queueFamilyIndex = queue_family,
        .queueCount = 1,
        .pQueuePriorities = &priority,
    };
    const char *device_extensions[] = { VK_KHR_EXTERNAL_SEMAPHORE_FD_EXTENSION_NAME };
    VkDeviceCreateInfo device_info = {
        .sType = VK_STRUCTURE_TYPE_DEVICE_CREATE_INFO,
        .queueCreateInfoCount = 1,
        .pQueueCreateInfos = &queue_info,
        .enabledExtensionCount = 1,
        .ppEnabledExtensionNames = device_extensions,
    };
    VK_CHECK(vkCreateDevice(physical, &device_info, NULL, &device));
    vkGetDeviceQueue(device, queue_family, 0, &queue);

    VkExportSemaphoreCreateInfo export_info = {
        .sType = VK_STRUCTURE_TYPE_EXPORT_SEMAPHORE_CREATE_INFO,
        .handleTypes = VK_EXTERNAL_SEMAPHORE_HANDLE_TYPE_OPAQUE_FD_BIT,
    };
    VkSemaphoreCreateInfo semaphore_info = {
        .sType = VK_STRUCTURE_TYPE_SEMAPHORE_CREATE_INFO,
        .pNext = &export_info,
    };
    VK_CHECK(vkCreateSemaphore(device, &semaphore_info, NULL, &ready));
    VK_CHECK(vkCreateSemaphore(device, &semaphore_info, NULL, &release));

    PFN_vkGetSemaphoreFdKHR get_semaphore_fd =
        (PFN_vkGetSemaphoreFdKHR)vkGetDeviceProcAddr(device, "vkGetSemaphoreFdKHR");
    if (get_semaphore_fd == NULL) {
        fprintf(stderr, "vkGetSemaphoreFdKHR unavailable\n");
        goto cleanup;
    }
    VkSemaphoreGetFdInfoKHR fd_info = {
        .sType = VK_STRUCTURE_TYPE_SEMAPHORE_GET_FD_INFO_KHR,
        .handleType = VK_EXTERNAL_SEMAPHORE_HANDLE_TYPE_OPAQUE_FD_BIT,
        .semaphore = ready,
    };
    VK_CHECK(get_semaphore_fd(device, &fd_info, &ready_fd));
    fd_info.semaphore = release;
    VK_CHECK(get_semaphore_fd(device, &fd_info, &release_fd));

    CUdevice cuda_device;
    CUDA_CHECK(cuDeviceGet_fn(&cuda_device, 0));
    CUDA_CHECK(cuCtxCreate_fn(&cuda_context, 0, cuda_device));
    CUDA_EXTERNAL_SEMAPHORE_HANDLE_DESC cuda_handle = {0};
    cuda_handle.type = CU_EXTERNAL_SEMAPHORE_HANDLE_TYPE_OPAQUE_FD;
    cuda_handle.handle.fd = ready_fd;
    CUDA_CHECK(cuImportExternalSemaphore_fn(&cuda_ready, &cuda_handle));
    ready_fd = -1; /* Successful opaque-FD import transfers ownership. */
    cuda_handle.handle.fd = release_fd;
    CUDA_CHECK(cuImportExternalSemaphore_fn(&cuda_release, &cuda_handle));
    release_fd = -1;

    CUDA_EXTERNAL_SEMAPHORE_SIGNAL_PARAMS signal_params = {0};
    CUDA_CHECK(cuSignalExternalSemaphoresAsync_fn(&cuda_ready, &signal_params, 1, (CUstream)0));
    CUDA_CHECK(cuCtxSynchronize_fn());

    VkPipelineStageFlags wait_stage = VK_PIPELINE_STAGE_ALL_COMMANDS_BIT;
    VkSubmitInfo submit_info = {
        .sType = VK_STRUCTURE_TYPE_SUBMIT_INFO,
        .waitSemaphoreCount = 1,
        .pWaitSemaphores = &ready,
        .pWaitDstStageMask = &wait_stage,
        .signalSemaphoreCount = 1,
        .pSignalSemaphores = &release,
    };
    VK_CHECK(vkQueueSubmit(queue, 1, &submit_info, VK_NULL_HANDLE));
    VK_CHECK(vkQueueWaitIdle(queue));

    CUDA_EXTERNAL_SEMAPHORE_WAIT_PARAMS wait_params = {0};
    CUDA_CHECK(cuWaitExternalSemaphoresAsync_fn(&cuda_release, &wait_params, 1, (CUstream)0));
    CUDA_CHECK(cuCtxSynchronize_fn());
    fprintf(stderr, "CUDA signal -> Vulkan wait/signal -> CUDA wait passed\n");
    exit_code = 0;

cleanup:
    if (cuda_context != NULL) {
        if (cuda_ready != NULL && cuDestroyExternalSemaphore_fn != NULL) {
            cuDestroyExternalSemaphore_fn(cuda_ready);
        }
        if (cuda_release != NULL && cuDestroyExternalSemaphore_fn != NULL) {
            cuDestroyExternalSemaphore_fn(cuda_release);
        }
        if (cuCtxDestroy_fn != NULL) {
            cuCtxDestroy_fn(cuda_context);
        }
    }
    if (ready_fd >= 0) close(ready_fd);
    if (release_fd >= 0) close(release_fd);
    if (device != VK_NULL_HANDLE) {
        if (ready != VK_NULL_HANDLE) vkDestroySemaphore(device, ready, NULL);
        if (release != VK_NULL_HANDLE) vkDestroySemaphore(device, release, NULL);
        vkDestroyDevice(device, NULL);
    }
    if (instance != VK_NULL_HANDLE) vkDestroyInstance(instance, NULL);
    if (cuda_library != NULL) dlclose(cuda_library);
    return exit_code;
}
