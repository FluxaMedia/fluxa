use crate::chapters::{parse_mkv_chapters_at_offset_json, parse_mkv_chapters_json};
use crate::dv_rewrite::{
    create_dv_segment_rewriter, dv_auto_detect_was_iptpqc2, dv_get_current_l1_json,
    dv_get_stream_stats_json, dv_rewrite_segment_bytes, dv_rpu_self_test,
    finish_dv_segment_rewriter, process_dv_segment_rewriter, start_dv_rewrite_local_stream_server,
};
use crate::local_stream::{start_local_stream_server, stop_local_stream_server};
use crate::torrent_engine;
use jni::JNIEnv;
use jni::objects::{JByteArray, JClass, JString};
use jni::sys::{jboolean, jbyteArray, jint, jlong, jstring};
use std::ptr;

type JBoolean = jboolean;
type JInt = jint;
type JLong = jlong;
type JObject<'local> = JClass<'local>;
type JStringReturn = jstring;

fn read_jstring(env: &mut JNIEnv<'_>, value: &JString<'_>) -> Option<String> {
    env.get_string(value).ok().map(|s| s.into())
}

fn write_jstring(env: &mut JNIEnv<'_>, value: Option<String>) -> JStringReturn {
    value
        .and_then(|s| env.new_string(s).ok())
        .map(|s| s.into_raw())
        .unwrap_or(ptr::null_mut())
}

fn read_signed_byte_array(env: &mut JNIEnv<'_>, data: &JByteArray<'_>) -> Option<Vec<i8>> {
    let len = env.get_array_length(data).ok()? as usize;
    let mut values = vec![0i8; len];
    if len > 0 && env.get_byte_array_region(data, 0, &mut values).is_err() {
        return None;
    }
    Some(values)
}

fn write_byte_array(env: &mut JNIEnv<'_>, data: &[u8]) -> jbyteArray {
    let Ok(result) = env.new_byte_array(data.len() as i32) else {
        return ptr::null_mut();
    };
    let values = unsafe { std::slice::from_raw_parts(data.as_ptr() as *const i8, data.len()) };
    if env.set_byte_array_region(&result, 0, &values).is_err() {
        return ptr::null_mut();
    }
    result.into_raw()
}

#[unsafe(no_mangle)]
/// # Safety
/// Called by the JVM with JNI-managed arguments. The function validates JNI
/// conversions, catches panics, and returns null on failure.
pub unsafe extern "system" fn Java_com_fluxa_app_core_rust_FluxaStreamingNative_startLocalStreamServerNative(
    mut env: JNIEnv<'_>,
    _class: JObject<'_>,
    target_url: JString<'_>,
    headers_json: JString<'_>,
    preferred_port: JInt,
) -> JStringReturn {
    // A panic anywhere below must not abort the host process — catch it and
    // hand back null, same as any other "couldn't compute a result" outcome.
    std::panic::catch_unwind(std::panic::AssertUnwindSafe(|| {
        let output = read_jstring(&mut env, &target_url).and_then(|target_url| {
            start_local_stream_server(
                &target_url,
                &read_jstring(&mut env, &headers_json)?,
                preferred_port,
            )
        });
        write_jstring(&mut env, output)
    }))
    .unwrap_or(ptr::null_mut())
}

#[unsafe(no_mangle)]
/// # Safety
/// Called by the JVM with JNI-managed arguments. The function validates JNI
/// conversions, catches panics, and returns null on failure.
pub unsafe extern "system" fn Java_com_fluxa_app_core_rust_FluxaStreamingNative_startDvRewriteLocalStreamServerNative(
    mut env: JNIEnv<'_>,
    _class: JObject<'_>,
    target_url: JString<'_>,
    headers_json: JString<'_>,
    dv_config_json: JString<'_>,
    preferred_port: JInt,
    spool_directory: JString<'_>,
) -> JStringReturn {
    std::panic::catch_unwind(std::panic::AssertUnwindSafe(|| {
        let output = read_jstring(&mut env, &target_url).and_then(|target_url| {
            start_dv_rewrite_local_stream_server(
                &target_url,
                &read_jstring(&mut env, &headers_json).unwrap_or_default(),
                &read_jstring(&mut env, &dv_config_json).unwrap_or_default(),
                preferred_port,
                &read_jstring(&mut env, &spool_directory).unwrap_or_default(),
            )
        });
        write_jstring(&mut env, output)
    }))
    .unwrap_or(ptr::null_mut())
}

#[unsafe(no_mangle)]
/// # Safety
/// Called by the JVM with JNI-managed arguments. The function validates JNI
/// conversions, catches panics, and returns false on failure.
pub unsafe extern "system" fn Java_com_fluxa_app_core_rust_FluxaStreamingNative_stopLocalStreamServerNative(
    mut env: JNIEnv<'_>,
    _class: JObject<'_>,
    server_id: JString<'_>,
) -> JBoolean {
    std::panic::catch_unwind(std::panic::AssertUnwindSafe(|| {
        let result = read_jstring(&mut env, &server_id)
            .map(|server_id| stop_local_stream_server(&server_id))
            .unwrap_or(false);
        if result { 1 } else { 0 }
    }))
    .unwrap_or(0)
}

#[unsafe(no_mangle)]
/// # Safety
/// Called by the JVM with JNI-managed arguments. The function validates JNI
/// conversions, catches panics, and returns null on failure.
pub unsafe extern "system" fn Java_com_fluxa_app_core_rust_FluxaStreamingNative_startTorrentServerNative(
    mut env: JNIEnv<'_>,
    _class: JObject<'_>,
    cache_dir: JString<'_>,
    preferred_port: JInt,
    access_token: JString<'_>,
) -> JStringReturn {
    std::panic::catch_unwind(std::panic::AssertUnwindSafe(|| {
        let output = read_jstring(&mut env, &cache_dir).and_then(|cache_dir| {
            let access_token = read_jstring(&mut env, &access_token).unwrap_or_default();
            torrent_engine::start_torrent_server(&cache_dir, preferred_port, &access_token)
        });
        write_jstring(&mut env, output)
    }))
    .unwrap_or(ptr::null_mut())
}

#[unsafe(no_mangle)]
/// # Safety
/// Called by the JVM. The function catches panics and returns false on failure.
pub unsafe extern "system" fn Java_com_fluxa_app_core_rust_FluxaStreamingNative_stopTorrentServerNative(
    _env: JNIEnv<'_>,
    _class: JObject<'_>,
    expected_generation: JLong,
) -> JBoolean {
    std::panic::catch_unwind(std::panic::AssertUnwindSafe(|| {
        let expected_generation = u64::try_from(expected_generation).ok();
        if torrent_engine::stop_torrent_server(expected_generation) {
            1
        } else {
            0
        }
    }))
    .unwrap_or(0)
}

#[unsafe(no_mangle)]
/// # Safety
/// Called by the JVM. The function catches panics and returns false on failure.
pub unsafe extern "system" fn Java_com_fluxa_app_core_rust_FluxaStreamingNative_dvRpuSelfTestNative(
    _env: JNIEnv<'_>,
    _class: JObject<'_>,
) -> JBoolean {
    std::panic::catch_unwind(std::panic::AssertUnwindSafe(|| {
        if dv_rpu_self_test() { 1 } else { 0 }
    }))
    .unwrap_or(0)
}

#[unsafe(no_mangle)]
/// # Safety
/// Called by the JVM. The function catches panics and returns false on failure.
pub unsafe extern "system" fn Java_com_fluxa_app_core_rust_FluxaStreamingNative_dvAutoDetectWasIptPqc2Native(
    _env: JNIEnv<'_>,
    _class: JObject<'_>,
) -> JBoolean {
    std::panic::catch_unwind(std::panic::AssertUnwindSafe(|| {
        if dv_auto_detect_was_iptpqc2() { 1 } else { 0 }
    }))
    .unwrap_or(0)
}

#[unsafe(no_mangle)]
/// # Safety
/// Called by the JVM with JNI-managed arguments. The function validates byte
/// array conversions, catches panics, and returns null on unrecoverable failure.
pub unsafe extern "system" fn Java_com_fluxa_app_core_rust_FluxaStreamingNative_dvRewriteSegmentBytesNative(
    mut env: JNIEnv<'_>,
    _class: JObject<'_>,
    data: JByteArray<'_>,
    rpu_mode: JInt,
    zero_level5: JBoolean,
    remove_hdr10plus: JBoolean,
    spool_directory: JString<'_>,
) -> jbyteArray {
    std::panic::catch_unwind(std::panic::AssertUnwindSafe(|| {
        let empty = env
            .new_byte_array(0)
            .ok()
            .map(|a| a.into_raw())
            .unwrap_or(ptr::null_mut());

        let len = match env.get_array_length(&data) {
            Ok(l) => l as usize,
            Err(_) => return empty,
        };

        let mut buf_i8: Vec<i8> = vec![0i8; len];
        if len > 0 && env.get_byte_array_region(&data, 0, &mut buf_i8).is_err() {
            return empty;
        }
        let input: Vec<u8> = buf_i8.into_iter().map(|b| b as u8).collect();

        let output = dv_rewrite_segment_bytes(
            &input,
            rpu_mode as u8,
            zero_level5 != 0,
            remove_hdr10plus != 0,
            read_jstring(&mut env, &spool_directory)
                .filter(|value| !value.is_empty())
                .map(std::path::PathBuf::from),
        );

        let result = match env.new_byte_array(output.len() as i32) {
            Ok(a) => a,
            Err(_) => return empty,
        };
        let output_i8: Vec<i8> = output.iter().map(|&b| b as i8).collect();
        if env.set_byte_array_region(&result, 0, &output_i8).is_err() {
            return empty;
        }
        result.into_raw()
    }))
    .unwrap_or(ptr::null_mut())
}

#[unsafe(no_mangle)]
pub unsafe extern "system" fn Java_com_fluxa_app_core_rust_FluxaStreamingNative_createDvRewriteSegmentNative(
    mut env: JNIEnv<'_>,
    _class: JObject<'_>,
    rpu_mode: JInt,
    zero_level5: JBoolean,
    remove_hdr10plus: JBoolean,
    spool_directory: JString<'_>,
) -> JLong {
    std::panic::catch_unwind(std::panic::AssertUnwindSafe(|| {
        create_dv_segment_rewriter(
            rpu_mode as u8,
            zero_level5 != 0,
            remove_hdr10plus != 0,
            read_jstring(&mut env, &spool_directory)
                .filter(|value| !value.is_empty())
                .map(std::path::PathBuf::from),
        ) as JLong
    }))
    .unwrap_or(0)
}

#[unsafe(no_mangle)]
pub unsafe extern "system" fn Java_com_fluxa_app_core_rust_FluxaStreamingNative_processDvRewriteSegmentNative(
    mut env: JNIEnv<'_>,
    _class: JObject<'_>,
    handle: JLong,
    data: JByteArray<'_>,
) -> jbyteArray {
    std::panic::catch_unwind(std::panic::AssertUnwindSafe(|| {
        let Some(input) = read_signed_byte_array(&mut env, &data) else {
            return ptr::null_mut();
        };
        let input = unsafe { std::slice::from_raw_parts(input.as_ptr() as *const u8, input.len()) };
        let Some(output) = process_dv_segment_rewriter(handle as u64, input) else {
            return ptr::null_mut();
        };
        write_byte_array(&mut env, &output)
    }))
    .unwrap_or(ptr::null_mut())
}

#[unsafe(no_mangle)]
pub unsafe extern "system" fn Java_com_fluxa_app_core_rust_FluxaStreamingNative_finishDvRewriteSegmentNative(
    mut env: JNIEnv<'_>,
    _class: JObject<'_>,
    handle: JLong,
) -> jbyteArray {
    std::panic::catch_unwind(std::panic::AssertUnwindSafe(|| {
        let Some(output) = finish_dv_segment_rewriter(handle as u64) else {
            return ptr::null_mut();
        };
        write_byte_array(&mut env, &output)
    }))
    .unwrap_or(ptr::null_mut())
}

#[unsafe(no_mangle)]
/// # Safety
/// Called by the JVM. The function catches panics and returns null on failure.
pub unsafe extern "system" fn Java_com_fluxa_app_core_rust_FluxaStreamingNative_dvGetStreamStatsJsonNative(
    mut env: JNIEnv<'_>,
    _class: JObject<'_>,
) -> JStringReturn {
    std::panic::catch_unwind(std::panic::AssertUnwindSafe(|| {
        write_jstring(&mut env, Some(dv_get_stream_stats_json()))
    }))
    .unwrap_or(ptr::null_mut())
}

#[unsafe(no_mangle)]
/// # Safety
/// Called by the JVM. The function catches panics and returns null on failure.
pub unsafe extern "system" fn Java_com_fluxa_app_core_rust_FluxaStreamingNative_dvGetCurrentL1JsonNative(
    mut env: JNIEnv<'_>,
    _class: JObject<'_>,
) -> JStringReturn {
    std::panic::catch_unwind(std::panic::AssertUnwindSafe(|| {
        write_jstring(&mut env, Some(dv_get_current_l1_json()))
    }))
    .unwrap_or(ptr::null_mut())
}

#[unsafe(no_mangle)]
/// # Safety
/// Called by the JVM with JNI-managed arguments. The function validates byte
/// array conversions, catches panics, and returns null on unrecoverable failure.
pub unsafe extern "system" fn Java_com_fluxa_app_core_rust_FluxaStreamingNative_parseMkvChaptersNative(
    mut env: JNIEnv<'_>,
    _class: JObject<'_>,
    data: JByteArray<'_>,
) -> JStringReturn {
    std::panic::catch_unwind(std::panic::AssertUnwindSafe(|| {
        let len = match env.get_array_length(&data) {
            Ok(l) => l as usize,
            Err(_) => return write_jstring(&mut env, Some("[]".to_string())),
        };
        let mut buf_i8: Vec<i8> = vec![0i8; len];
        if len > 0 && env.get_byte_array_region(&data, 0, &mut buf_i8).is_err() {
            return write_jstring(&mut env, Some("[]".to_string()));
        }
        let input: Vec<u8> = buf_i8.into_iter().map(|b| b as u8).collect();
        write_jstring(&mut env, Some(parse_mkv_chapters_json(&input)))
    }))
    .unwrap_or(ptr::null_mut())
}

#[unsafe(no_mangle)]
pub unsafe extern "system" fn Java_com_fluxa_app_core_rust_FluxaStreamingNative_parseMkvChaptersAtOffsetNative(
    mut env: JNIEnv<'_>,
    _class: JObject<'_>,
    data: JByteArray<'_>,
) -> JStringReturn {
    std::panic::catch_unwind(std::panic::AssertUnwindSafe(|| {
        let len = match env.get_array_length(&data) {
            Ok(l) => l as usize,
            Err(_) => return write_jstring(&mut env, Some("[]".to_string())),
        };
        let mut buf_i8: Vec<i8> = vec![0i8; len];
        if len > 0 && env.get_byte_array_region(&data, 0, &mut buf_i8).is_err() {
            return write_jstring(&mut env, Some("[]".to_string()));
        }
        let input: Vec<u8> = buf_i8.into_iter().map(|b| b as u8).collect();
        write_jstring(&mut env, Some(parse_mkv_chapters_at_offset_json(&input)))
    }))
    .unwrap_or(ptr::null_mut())
}
