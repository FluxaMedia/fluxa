use super::*;

pub(super) struct MpvApi {
    pub(super) _library: Library,
    pub(super) mpv_create: MpvCreate,
    pub(super) mpv_initialize: MpvInitialize,
    pub(super) mpv_terminate_destroy: MpvTerminateDestroy,
    pub(super) mpv_set_option_string: MpvSetOptionString,
    pub(super) mpv_command: MpvCommand,
    pub(super) mpv_command_string: MpvCommandString,
    pub(super) mpv_command_async: MpvCommandAsync,
    pub(super) mpv_get_property: MpvGetProperty,
    pub(super) mpv_free: MpvFree,
    pub(super) mpv_error_string: MpvErrorString,
    pub(super) mpv_render_context_create: MpvRenderContextCreate,
    pub(super) mpv_render_context_render: MpvRenderContextRender,
    pub(super) mpv_render_context_update: MpvRenderContextUpdate,
    pub(super) mpv_render_context_report_swap: MpvRenderContextReportSwap,
    pub(super) mpv_render_context_set_parameter: MpvRenderContextSetParameter,
    pub(super) mpv_render_context_free: MpvRenderContextFree,
    pub(super) mpv_wait_event: MpvWaitEvent,
    pub(super) mpv_request_log_messages: MpvRequestLogMessages,
}

unsafe impl Send for MpvApi {}
unsafe impl Sync for MpvApi {}

impl MpvApi {
    pub(super) fn load() -> Result<Self, String> {
        let lib_path = find_libmpv_path();
        let library = load_library(&lib_path)
            .map_err(|error| format!("failed to load libmpv from '{lib_path}': {error}"))?;
        unsafe {
            let mpv_create = *library
                .get::<MpvCreate>(b"mpv_create\0")
                .map_err(load_error)?;
            let mpv_initialize = *library
                .get::<MpvInitialize>(b"mpv_initialize\0")
                .map_err(load_error)?;
            let mpv_terminate_destroy = *library
                .get::<MpvTerminateDestroy>(b"mpv_terminate_destroy\0")
                .map_err(load_error)?;
            let mpv_set_option_string = *library
                .get::<MpvSetOptionString>(b"mpv_set_option_string\0")
                .map_err(load_error)?;
            let mpv_command = *library
                .get::<MpvCommand>(b"mpv_command\0")
                .map_err(load_error)?;
            let mpv_command_string = *library
                .get::<MpvCommandString>(b"mpv_command_string\0")
                .map_err(load_error)?;
            let mpv_command_async = *library
                .get::<MpvCommandAsync>(b"mpv_command_async\0")
                .map_err(load_error)?;
            let mpv_get_property = *library
                .get::<MpvGetProperty>(b"mpv_get_property\0")
                .map_err(load_error)?;
            let mpv_free = *library.get::<MpvFree>(b"mpv_free\0").map_err(load_error)?;
            let mpv_error_string = *library
                .get::<MpvErrorString>(b"mpv_error_string\0")
                .map_err(load_error)?;
            let mpv_render_context_create = *library
                .get::<MpvRenderContextCreate>(b"mpv_render_context_create\0")
                .map_err(load_error)?;
            let mpv_render_context_render = *library
                .get::<MpvRenderContextRender>(b"mpv_render_context_render\0")
                .map_err(load_error)?;
            let mpv_render_context_update = *library
                .get::<MpvRenderContextUpdate>(b"mpv_render_context_update\0")
                .map_err(load_error)?;
            let mpv_render_context_report_swap = *library
                .get::<MpvRenderContextReportSwap>(b"mpv_render_context_report_swap\0")
                .map_err(load_error)?;
            let mpv_render_context_set_parameter = *library
                .get::<MpvRenderContextSetParameter>(b"mpv_render_context_set_parameter\0")
                .map_err(load_error)?;
            let mpv_render_context_free = *library
                .get::<MpvRenderContextFree>(b"mpv_render_context_free\0")
                .map_err(load_error)?;
            let mpv_wait_event = *library
                .get::<MpvWaitEvent>(b"mpv_wait_event\0")
                .map_err(load_error)?;
            let mpv_request_log_messages = *library
                .get::<MpvRequestLogMessages>(b"mpv_request_log_messages\0")
                .map_err(load_error)?;
            Ok(Self {
                _library: library,
                mpv_create,
                mpv_initialize,
                mpv_terminate_destroy,
                mpv_set_option_string,
                mpv_command,
                mpv_command_string,
                mpv_command_async,
                mpv_get_property,
                mpv_free,
                mpv_error_string,
                mpv_render_context_create,
                mpv_render_context_render,
                mpv_render_context_update,
                mpv_render_context_report_swap,
                mpv_render_context_set_parameter,
                mpv_render_context_free,
                mpv_wait_event,
                mpv_request_log_messages,
            })
        }
    }

    pub(super) fn error_string(&self, code: c_int) -> String {
        let ptr = unsafe { (self.mpv_error_string)(code) };
        if ptr.is_null() {
            return format!("mpv error {code}");
        }
        unsafe { CStr::from_ptr(ptr).to_string_lossy().into_owned() }
    }
}
