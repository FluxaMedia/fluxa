#include <jni.h>
#include <stdbool.h>
#include <stdint.h>
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
    jstring s = (*env)->NewStringUTF(env, v);
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
