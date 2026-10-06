#include <jni.h>
#include <stdbool.h>
#include <stdint.h>
#include <stdio.h>
#include <stdlib.h>
#include <string.h>

#include <libavcodec/jni.h>
#include <mpv/client.h>

#define FN(name) Java_com_fluxa_app_ui_rust_Mpv_##name
#define H ((mpv_handle *)(intptr_t)handle)

JNIEXPORT jint JNI_OnLoad(JavaVM *vm, void *reserved)
{
    av_jni_set_java_vm(vm, NULL);
    return JNI_VERSION_1_6;
}

static jstring utf8_string(JNIEnv *env, const char *text)
{
    jsize len = (jsize)strlen(text);
    jbyteArray bytes = (*env)->NewByteArray(env, len);
    if (!bytes)
        return NULL;
    (*env)->SetByteArrayRegion(env, bytes, 0, len, (const jbyte *)text);
    jclass string = (*env)->FindClass(env, "java/lang/String");
    jmethodID init = (*env)->GetMethodID(env, string, "<init>", "([BLjava/lang/String;)V");
    jstring charset = (*env)->NewStringUTF(env, "UTF-8");
    jstring out = (jstring)(*env)->NewObject(env, string, init, bytes, charset);
    (*env)->DeleteLocalRef(env, charset);
    (*env)->DeleteLocalRef(env, bytes);
    (*env)->DeleteLocalRef(env, string);
    return out;
}

JNIEXPORT jlong JNICALL FN(create)(JNIEnv *env, jclass cls)
{
    return (jlong)(intptr_t)mpv_create();
}

JNIEXPORT jint JNICALL FN(initialize)(JNIEnv *env, jclass cls, jlong handle)
{
    return mpv_initialize(H);
}

JNIEXPORT void JNICALL FN(destroy)(JNIEnv *env, jclass cls, jlong handle)
{
    mpv_terminate_destroy(H);
}

JNIEXPORT jint JNICALL FN(setOption)(JNIEnv *env, jclass cls, jlong handle, jstring key, jstring value)
{
    const char *k = (*env)->GetStringUTFChars(env, key, NULL);
    const char *v = (*env)->GetStringUTFChars(env, value, NULL);
    int r = mpv_set_option_string(H, k, v);
    (*env)->ReleaseStringUTFChars(env, key, k);
    (*env)->ReleaseStringUTFChars(env, value, v);
    return r;
}

JNIEXPORT jint JNICALL FN(setProperty)(JNIEnv *env, jclass cls, jlong handle, jstring key, jstring value)
{
    const char *k = (*env)->GetStringUTFChars(env, key, NULL);
    const char *v = (*env)->GetStringUTFChars(env, value, NULL);
    int r = mpv_set_property_string(H, k, v);
    (*env)->ReleaseStringUTFChars(env, key, k);
    (*env)->ReleaseStringUTFChars(env, value, v);
    return r;
}

JNIEXPORT jstring JNICALL FN(getProperty)(JNIEnv *env, jclass cls, jlong handle, jstring key)
{
    const char *k = (*env)->GetStringUTFChars(env, key, NULL);
    char *v = mpv_get_property_string(H, k);
    (*env)->ReleaseStringUTFChars(env, key, k);
    if (!v)
        return NULL;
    jstring s = utf8_string(env, v);
    mpv_free(v);
    return s;
}

JNIEXPORT jint JNICALL FN(command)(JNIEnv *env, jclass cls, jlong handle, jobjectArray args)
{
    int n = (*env)->GetArrayLength(env, args);
    const char **argv = calloc(n + 1, sizeof(*argv));
    jstring *strs = calloc(n, sizeof(*strs));
    for (int i = 0; i < n; i++) {
        strs[i] = (*env)->GetObjectArrayElement(env, args, i);
        argv[i] = (*env)->GetStringUTFChars(env, strs[i], NULL);
    }
    int r = mpv_command(H, argv);
    for (int i = 0; i < n; i++) {
        (*env)->ReleaseStringUTFChars(env, strs[i], argv[i]);
        (*env)->DeleteLocalRef(env, strs[i]);
    }
    free(strs);
    free(argv);
    return r;
}

JNIEXPORT jboolean JNICALL FN(drain)(JNIEnv *env, jclass cls, jlong handle)
{
    bool failed = false;
    mpv_event *ev;
    while ((ev = mpv_wait_event(H, 0))->event_id != MPV_EVENT_NONE) {
        if (ev->event_id == MPV_EVENT_END_FILE)
            failed |= ((mpv_event_end_file *)ev->data)->reason == MPV_END_FILE_REASON_ERROR;
    }
    return failed;
}

JNIEXPORT jbyteArray JNICALL FN(thumbnail)(JNIEnv *env, jclass cls, jlong handle, jdouble seconds)
{
    char time[32];
    snprintf(time, sizeof(time), "%.3f", seconds);
    const char *args[] = {"thumbnail", time, NULL};
    mpv_node res;
    if (mpv_command_ret(H, args, &res) < 0)
        return NULL;
    int64_t w = 0, h = 0, stride = 0;
    double at = 0;
    mpv_byte_array *ba = NULL;
    if (res.format == MPV_FORMAT_NODE_MAP) {
        for (int i = 0; i < res.u.list->num; i++) {
            const char *k = res.u.list->keys[i];
            mpv_node *v = &res.u.list->values[i];
            if (!strcmp(k, "w")) w = v->u.int64;
            else if (!strcmp(k, "h")) h = v->u.int64;
            else if (!strcmp(k, "stride")) stride = v->u.int64;
            else if (!strcmp(k, "time")) at = v->u.double_;
            else if (!strcmp(k, "data")) ba = v->u.ba;
        }
    }
    jbyteArray out = NULL;
    if (ba && w > 0 && h > 0 && stride >= w * 4 && ba->size >= (size_t)(stride * h)) {
        out = (*env)->NewByteArray(env, 12 + (jsize)(w * h * 4));
        if (out) {
            int32_t head[3] = {(int32_t)w, (int32_t)h, (int32_t)(at * 1000.0)};
            (*env)->SetByteArrayRegion(env, out, 0, 12, (const jbyte *)head);
            jbyte *row = malloc(w * 4);
            for (int64_t y = 0; row && y < h; y++) {
                const uint8_t *src = (const uint8_t *)ba->data + y * stride;
                for (int64_t x = 0; x < w; x++) {
                    row[x * 4] = src[x * 4 + 2];
                    row[x * 4 + 1] = src[x * 4 + 1];
                    row[x * 4 + 2] = src[x * 4];
                    row[x * 4 + 3] = (jbyte)0xff;
                }
                (*env)->SetByteArrayRegion(env, out, 12 + y * w * 4, w * 4, row);
            }
            free(row);
        }
    }
    mpv_free_node_contents(&res);
    return out;
}

JNIEXPORT jlong JNICALL FN(attach)(JNIEnv *env, jclass cls, jlong handle, jobject surface)
{
    jobject ref = (*env)->NewGlobalRef(env, surface);
    int64_t wid = (intptr_t)ref;
    mpv_set_option(H, "wid", MPV_FORMAT_INT64, &wid);
    return wid;
}

JNIEXPORT void JNICALL FN(detach)(JNIEnv *env, jclass cls, jlong handle, jlong ref)
{
    int64_t wid = 0;
    mpv_set_option(H, "wid", MPV_FORMAT_INT64, &wid);
    (*env)->DeleteGlobalRef(env, (jobject)(intptr_t)ref);
}
