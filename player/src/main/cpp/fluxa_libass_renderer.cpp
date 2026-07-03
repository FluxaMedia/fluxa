#include <android/bitmap.h>
#include <android/log.h>
#include <dlfcn.h>
#include <jni.h>

#include <algorithm>
#include <cstdint>
#include <cstdlib>
#include <cstring>

#define LOG_TAG "FluxaLibassRenderer"
#define LOGE(...) __android_log_print(ANDROID_LOG_ERROR, LOG_TAG, __VA_ARGS__)

struct ASS_Library;
struct ASS_Renderer;
struct ASS_Track;

struct ASS_Image {
    int w;
    int h;
    int stride;
    unsigned char* bitmap;
    uint32_t color;
    int dst_x;
    int dst_y;
    ASS_Image* next;
    int type;
};

using ass_library_init_fn = ASS_Library* (*)();
using ass_library_done_fn = void (*)(ASS_Library*);
using ass_renderer_init_fn = ASS_Renderer* (*)(ASS_Library*);
using ass_renderer_done_fn = void (*)(ASS_Renderer*);
using ass_read_memory_fn = ASS_Track* (*)(ASS_Library*, char*, size_t, char*);
using ass_free_track_fn = void (*)(ASS_Track*);
using ass_add_font_fn = void (*)(ASS_Library*, char*, char*, int);
using ass_set_fonts_fn = void (*)(ASS_Renderer*, const char*, const char*, int, const char*, int);
using ass_set_fonts_dir_fn = void (*)(ASS_Library*, const char*);
using ass_set_frame_size_fn = void (*)(ASS_Renderer*, int, int);
using ass_set_storage_size_fn = void (*)(ASS_Renderer*, int, int);
using ass_render_frame_fn = ASS_Image* (*)(ASS_Renderer*, ASS_Track*, long long, int*);
using ass_set_use_margins_fn = void (*)(ASS_Renderer*, int);
using ass_set_aspect_ratio_fn = void (*)(ASS_Renderer*, double, double);
using ass_set_font_scale_fn = void (*)(ASS_Renderer*, double);
using ass_set_line_spacing_fn = void (*)(ASS_Renderer*, double);
using ass_set_check_readorder_fn = void (*)(ASS_Track*, int);
using ass_process_data_fn = void (*)(ASS_Track*, char*, int);
using ass_flush_events_fn = void (*)(ASS_Track*);

struct LibassApi {
    void* handle = nullptr;
    ass_library_init_fn library_init = nullptr;
    ass_library_done_fn library_done = nullptr;
    ass_renderer_init_fn renderer_init = nullptr;
    ass_renderer_done_fn renderer_done = nullptr;
    ass_read_memory_fn read_memory = nullptr;
    ass_free_track_fn free_track = nullptr;
    ass_add_font_fn add_font = nullptr;
    ass_set_fonts_fn set_fonts = nullptr;
    ass_set_fonts_dir_fn set_fonts_dir = nullptr;
    ass_set_frame_size_fn set_frame_size = nullptr;
    ass_set_storage_size_fn set_storage_size = nullptr;
    ass_render_frame_fn render_frame = nullptr;
    ass_set_use_margins_fn set_use_margins = nullptr;
    ass_set_aspect_ratio_fn set_aspect_ratio = nullptr;
    ass_set_font_scale_fn set_font_scale = nullptr;
    ass_set_line_spacing_fn set_line_spacing = nullptr;
    ass_set_check_readorder_fn set_check_readorder = nullptr;
    ass_process_data_fn process_data = nullptr;
    ass_flush_events_fn flush_events = nullptr;
};

struct Session {
    ASS_Library* library = nullptr;
    ASS_Renderer* renderer = nullptr;
    ASS_Track* track = nullptr;
    int width = 0;
    int height = 0;
};

static LibassApi g_api;
static bool g_api_loaded = false;

static void* symbol(void* handle, const char* name) {
    void* value = dlsym(handle, name);
    if (!value) LOGE("Missing libass symbol: %s", name);
    return value;
}

static bool ensure_api() {
    if (g_api_loaded) return g_api.handle != nullptr;
    g_api_loaded = true;
    g_api.handle = dlopen("libmpv.so", RTLD_LAZY | RTLD_LOCAL);
    if (!g_api.handle) {
        LOGE("Could not open libmpv.so: %s", dlerror());
        return false;
    }
    g_api.library_init = reinterpret_cast<ass_library_init_fn>(symbol(g_api.handle, "ass_library_init"));
    g_api.library_done = reinterpret_cast<ass_library_done_fn>(symbol(g_api.handle, "ass_library_done"));
    g_api.renderer_init = reinterpret_cast<ass_renderer_init_fn>(symbol(g_api.handle, "ass_renderer_init"));
    g_api.renderer_done = reinterpret_cast<ass_renderer_done_fn>(symbol(g_api.handle, "ass_renderer_done"));
    g_api.read_memory = reinterpret_cast<ass_read_memory_fn>(symbol(g_api.handle, "ass_read_memory"));
    g_api.free_track = reinterpret_cast<ass_free_track_fn>(symbol(g_api.handle, "ass_free_track"));
    g_api.add_font = reinterpret_cast<ass_add_font_fn>(symbol(g_api.handle, "ass_add_font"));
    g_api.set_fonts = reinterpret_cast<ass_set_fonts_fn>(symbol(g_api.handle, "ass_set_fonts"));
    g_api.set_fonts_dir = reinterpret_cast<ass_set_fonts_dir_fn>(symbol(g_api.handle, "ass_set_fonts_dir"));
    g_api.set_frame_size = reinterpret_cast<ass_set_frame_size_fn>(symbol(g_api.handle, "ass_set_frame_size"));
    g_api.set_storage_size = reinterpret_cast<ass_set_storage_size_fn>(symbol(g_api.handle, "ass_set_storage_size"));
    g_api.render_frame = reinterpret_cast<ass_render_frame_fn>(symbol(g_api.handle, "ass_render_frame"));
    g_api.set_use_margins = reinterpret_cast<ass_set_use_margins_fn>(symbol(g_api.handle, "ass_set_use_margins"));
    g_api.set_aspect_ratio = reinterpret_cast<ass_set_aspect_ratio_fn>(symbol(g_api.handle, "ass_set_aspect_ratio"));
    g_api.set_font_scale = reinterpret_cast<ass_set_font_scale_fn>(symbol(g_api.handle, "ass_set_font_scale"));
    g_api.set_line_spacing = reinterpret_cast<ass_set_line_spacing_fn>(symbol(g_api.handle, "ass_set_line_spacing"));
    g_api.set_check_readorder = reinterpret_cast<ass_set_check_readorder_fn>(symbol(g_api.handle, "ass_set_check_readorder"));
    g_api.process_data = reinterpret_cast<ass_process_data_fn>(symbol(g_api.handle, "ass_process_data"));
    g_api.flush_events = reinterpret_cast<ass_flush_events_fn>(symbol(g_api.handle, "ass_flush_events"));
    return g_api.library_init && g_api.library_done && g_api.renderer_init && g_api.renderer_done &&
        g_api.read_memory && g_api.free_track && g_api.add_font && g_api.set_fonts && g_api.set_frame_size &&
        g_api.set_storage_size && g_api.render_frame;
}

static void release_session(Session* session) {
    if (!session) return;
    if (session->track && g_api.free_track) g_api.free_track(session->track);
    if (session->renderer && g_api.renderer_done) g_api.renderer_done(session->renderer);
    if (session->library && g_api.library_done) g_api.library_done(session->library);
    delete session;
}

static void blend_ass_image(uint8_t* pixels, int bitmap_width, int bitmap_height, int stride, const ASS_Image* image) {
    const int src_alpha = 255 - static_cast<int>(image->color & 0xff);
    if (src_alpha <= 0) return;
    const int src_r = static_cast<int>((image->color >> 24) & 0xff);
    const int src_g = static_cast<int>((image->color >> 16) & 0xff);
    const int src_b = static_cast<int>((image->color >> 8) & 0xff);

    const int x0 = std::max(0, image->dst_x);
    const int y0 = std::max(0, image->dst_y);
    const int x1 = std::min(bitmap_width, image->dst_x + image->w);
    const int y1 = std::min(bitmap_height, image->dst_y + image->h);
    if (x0 >= x1 || y0 >= y1) return;

    for (int y = y0; y < y1; ++y) {
        const int src_y = y - image->dst_y;
        const uint8_t* src_row = image->bitmap + src_y * image->stride;
        uint8_t* dst_row = pixels + y * stride;
        for (int x = x0; x < x1; ++x) {
            const int src_x = x - image->dst_x;
            const int coverage = src_row[src_x];
            if (coverage == 0) continue;
            const int alpha = (src_alpha * coverage + 127) / 255;
            uint8_t* dst = dst_row + x * 4;
            dst[0] = static_cast<uint8_t>((src_r * alpha + dst[0] * (255 - alpha) + 127) / 255);
            dst[1] = static_cast<uint8_t>((src_g * alpha + dst[1] * (255 - alpha) + 127) / 255);
            dst[2] = static_cast<uint8_t>((src_b * alpha + dst[2] * (255 - alpha) + 127) / 255);
            dst[3] = static_cast<uint8_t>(std::min(255, alpha + (dst[3] * (255 - alpha) + 127) / 255));
        }
    }
}

extern "C" JNIEXPORT jlong JNICALL
Java_com_fluxa_app_player_NativeLibassRenderer_00024Companion_nativeCreate(
    JNIEnv* env,
    jobject,
    jbyteArray ass_data,
    jobjectArray font_names,
    jobjectArray font_data,
    jstring fonts_dir
) {
    if (!ensure_api() || !ass_data) return 0;
    auto* session = new Session();
    session->library = g_api.library_init();
    if (!session->library) {
        release_session(session);
        return 0;
    }

    const char* fonts_dir_chars = fonts_dir ? env->GetStringUTFChars(fonts_dir, nullptr) : nullptr;
    if (fonts_dir_chars && g_api.set_fonts_dir) {
        g_api.set_fonts_dir(session->library, fonts_dir_chars);
    }
    const jsize font_count = font_names && font_data
        ? std::min(env->GetArrayLength(font_names), env->GetArrayLength(font_data))
        : 0;
    for (jsize i = 0; i < font_count; ++i) {
        auto name = static_cast<jstring>(env->GetObjectArrayElement(font_names, i));
        auto bytes = static_cast<jbyteArray>(env->GetObjectArrayElement(font_data, i));
        if (!name || !bytes) continue;
        const char* name_chars = env->GetStringUTFChars(name, nullptr);
        const jsize byte_count = env->GetArrayLength(bytes);
        char* font_buffer = static_cast<char*>(std::malloc(static_cast<size_t>(byte_count)));
        if (name_chars && font_buffer) {
            env->GetByteArrayRegion(bytes, 0, byte_count, reinterpret_cast<jbyte*>(font_buffer));
            g_api.add_font(session->library, const_cast<char*>(name_chars), font_buffer, byte_count);
        }
        if (name_chars) env->ReleaseStringUTFChars(name, name_chars);
        std::free(font_buffer);
        env->DeleteLocalRef(name);
        env->DeleteLocalRef(bytes);
    }

    session->renderer = g_api.renderer_init(session->library);
    if (!session->renderer) {
        if (fonts_dir_chars) env->ReleaseStringUTFChars(fonts_dir, fonts_dir_chars);
        release_session(session);
        return 0;
    }
    g_api.set_fonts(session->renderer, nullptr, "sans-serif", 1, nullptr, 1);
    if (g_api.set_use_margins) g_api.set_use_margins(session->renderer, 0);
    if (g_api.set_font_scale) g_api.set_font_scale(session->renderer, 1.0);
    if (g_api.set_line_spacing) g_api.set_line_spacing(session->renderer, 0.0);

    const jsize size = env->GetArrayLength(ass_data);
    char* buffer = static_cast<char*>(std::malloc(static_cast<size_t>(size) + 1));
    if (!buffer) {
        if (fonts_dir_chars) env->ReleaseStringUTFChars(fonts_dir, fonts_dir_chars);
        release_session(session);
        return 0;
    }
    env->GetByteArrayRegion(ass_data, 0, size, reinterpret_cast<jbyte*>(buffer));
    buffer[size] = '\0';
    session->track = g_api.read_memory(session->library, buffer, static_cast<size_t>(size), nullptr);
    std::free(buffer);

    if (fonts_dir_chars) env->ReleaseStringUTFChars(fonts_dir, fonts_dir_chars);
    if (!session->track) {
        release_session(session);
        return 0;
    }
    if (g_api.set_check_readorder) g_api.set_check_readorder(session->track, 0);
    return reinterpret_cast<jlong>(session);
}

extern "C" JNIEXPORT jboolean JNICALL
Java_com_fluxa_app_player_NativeLibassRenderer_nativeRender(JNIEnv* env, jobject, jlong handle, jlong time_ms, jobject bitmap) {
    auto* session = reinterpret_cast<Session*>(handle);
    if (!session || !session->renderer || !session->track || !bitmap || !ensure_api()) return JNI_FALSE;

    AndroidBitmapInfo info;
    if (AndroidBitmap_getInfo(env, bitmap, &info) != ANDROID_BITMAP_RESULT_SUCCESS) return JNI_FALSE;
    if (info.format != ANDROID_BITMAP_FORMAT_RGBA_8888 || info.width <= 0 || info.height <= 0) return JNI_FALSE;

    void* raw_pixels = nullptr;
    if (AndroidBitmap_lockPixels(env, bitmap, &raw_pixels) != ANDROID_BITMAP_RESULT_SUCCESS) return JNI_FALSE;
    auto* pixels = static_cast<uint8_t*>(raw_pixels);
    std::memset(pixels, 0, static_cast<size_t>(info.stride) * info.height);

    if (session->width != static_cast<int>(info.width) || session->height != static_cast<int>(info.height)) {
        session->width = static_cast<int>(info.width);
        session->height = static_cast<int>(info.height);
        g_api.set_frame_size(session->renderer, session->width, session->height);
        g_api.set_storage_size(session->renderer, session->width, session->height);
        if (g_api.set_aspect_ratio && session->height > 0) {
            g_api.set_aspect_ratio(session->renderer, static_cast<double>(session->width) / session->height, 1.0);
        }
    }

    int detect_change = 0;
    ASS_Image* images = g_api.render_frame(session->renderer, session->track, static_cast<long long>(time_ms), &detect_change);
    for (ASS_Image* image = images; image != nullptr; image = image->next) {
        blend_ass_image(pixels, static_cast<int>(info.width), static_cast<int>(info.height), static_cast<int>(info.stride), image);
    }
    AndroidBitmap_unlockPixels(env, bitmap);
    return images != nullptr ? JNI_TRUE : JNI_FALSE;
}

extern "C" JNIEXPORT void JNICALL
Java_com_fluxa_app_player_NativeLibassRenderer_nativeRelease(JNIEnv*, jobject, jlong handle) {
    release_session(reinterpret_cast<Session*>(handle));
}

extern "C" JNIEXPORT void JNICALL
Java_com_fluxa_app_player_NativeLibassRenderer_nativeAddEvent(JNIEnv* env, jobject, jlong handle, jstring dialogue_line) {
    auto* session = reinterpret_cast<Session*>(handle);
    if (!session || !session->track || !dialogue_line || !ensure_api() || !g_api.process_data) return;
    const char* line = env->GetStringUTFChars(dialogue_line, nullptr);
    if (!line) return;
    const int len = static_cast<int>(std::strlen(line));
    char* buf = static_cast<char*>(std::malloc(static_cast<size_t>(len) + 2));
    if (buf) {
        std::memcpy(buf, line, static_cast<size_t>(len));
        buf[len] = '\n';
        buf[len + 1] = '\0';
        g_api.process_data(session->track, buf, len + 1);
        std::free(buf);
    }
    env->ReleaseStringUTFChars(dialogue_line, line);
}

extern "C" JNIEXPORT void JNICALL
Java_com_fluxa_app_player_NativeLibassRenderer_nativeClearEvents(JNIEnv*, jobject, jlong handle) {
    auto* session = reinterpret_cast<Session*>(handle);
    if (!session || !session->track || !ensure_api() || !g_api.flush_events) return;
    g_api.flush_events(session->track);
}
