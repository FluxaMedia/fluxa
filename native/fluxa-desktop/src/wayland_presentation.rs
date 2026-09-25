use std::{
    collections::HashMap,
    os::raw::c_int,
    sync::mpsc::{self, Receiver, Sender},
};

use raw_window_handle::{HasDisplayHandle, HasWindowHandle, RawDisplayHandle, RawWindowHandle};
use wayland_client::{
    Connection, Dispatch, EventQueue, Proxy, QueueHandle, WEnum,
    backend::{Backend, ObjectId},
    globals::{GlobalListContents, registry_queue_init},
    protocol::{wl_registry, wl_surface},
};
use wayland_protocols::wp::presentation_time::client::{wp_presentation, wp_presentation_feedback};
use winit::window::Window;

#[derive(Debug, Clone)]
pub enum FeedbackEvent {
    Presented {
        frame_id: u64,
        requested_ns: u64,
        presented_ns: u64,
        refresh_ns: u32,
        sequence: u64,
        flags: u32,
    },
    Discarded {
        frame_id: u64,
    },
}

#[derive(Debug)]
struct FeedbackRequest {
    frame_id: u64,
    requested_ns: u64,
}

struct DispatchState {
    presentation: wp_presentation::WpPresentation,
    surface: wl_surface::WlSurface,
    clock_id: Option<u32>,
    pending: HashMap<u64, wp_presentation_feedback::WpPresentationFeedback>,
    sender: Sender<FeedbackEvent>,
}

pub struct WaylandPresentation {
    connection: Connection,
    queue: EventQueue<DispatchState>,
    queue_handle: QueueHandle<DispatchState>,
    state: DispatchState,
    receiver: Receiver<FeedbackEvent>,
}

impl WaylandPresentation {
    pub fn from_window(window: &Window) -> Result<Option<Self>, String> {
        let display_handle = window
            .display_handle()
            .map_err(|error| format!("get Wayland display handle: {error}"))?;
        let RawDisplayHandle::Wayland(display) = display_handle.as_raw() else {
            return Ok(None);
        };
        let window_handle = window
            .window_handle()
            .map_err(|error| format!("get Wayland surface handle: {error}"))?;
        let RawWindowHandle::Wayland(surface) = window_handle.as_raw() else {
            return Ok(None);
        };

        let backend = unsafe { Backend::from_foreign_display(display.display.as_ptr().cast()) };
        let connection = Connection::from_backend(backend);
        let (globals, mut queue) = registry_queue_init::<DispatchState>(&connection)
            .map_err(|error| format!("read Wayland globals: {error}"))?;
        if !globals
            .contents()
            .clone_list()
            .iter()
            .any(|global| global.interface == "wp_presentation")
        {
            return Ok(None);
        }

        let queue_handle = queue.handle();
        let presentation = globals
            .bind::<wp_presentation::WpPresentation, _, _>(&queue_handle, 1..=2, ())
            .map_err(|error| format!("bind wp_presentation: {error}"))?;
        let surface_id = unsafe {
            ObjectId::from_ptr(
                wl_surface::WlSurface::interface(),
                surface.surface.as_ptr().cast(),
            )
        }
        .map_err(|error| format!("wrap winit wl_surface: {error}"))?;
        let surface_proxy = wl_surface::WlSurface::from_id(&connection, surface_id)
            .map_err(|error| format!("create wl_surface proxy: {error}"))?;
        let (sender, receiver) = mpsc::channel();
        let mut state = DispatchState {
            presentation,
            surface: surface_proxy,
            clock_id: None,
            pending: HashMap::new(),
            sender,
        };

        // This one-time setup roundtrip receives wp_presentation.clock_id.
        // Subsequent feedback is dispatched from winit's normal Wayland wakeups.
        queue
            .roundtrip(&mut state)
            .map_err(|error| format!("initialize wp_presentation clock: {error}"))?;
        if state.clock_id.is_none() {
            return Err("compositor did not provide wp_presentation.clock_id".to_owned());
        }

        Ok(Some(Self {
            connection,
            queue,
            queue_handle,
            state,
            receiver,
        }))
    }

    /// Request feedback for the next commit on the winit surface. Call before
    /// `SurfaceTexture::present()` so feedback is associated with that frame.
    pub fn request_feedback(&mut self, frame_id: u64) -> Result<(), String> {
        let clock_id = self
            .state
            .clock_id
            .ok_or_else(|| "wp_presentation clock is not initialized".to_owned())?;
        let requested_ns = clock_now_ns(clock_id)?;
        let request_data = FeedbackRequest {
            frame_id,
            requested_ns,
        };
        let feedback =
            self.state
                .presentation
                .feedback(&self.state.surface, &self.queue_handle, request_data);
        self.state.pending.insert(frame_id, feedback);
        self.connection
            .flush()
            .map_err(|error| format!("flush presentation feedback request: {error}"))
    }

    pub fn dispatch_pending(&mut self) -> Result<(), String> {
        self.queue
            .dispatch_pending(&mut self.state)
            .map(|_| ())
            .map_err(|error| format!("dispatch presentation feedback: {error}"))
    }

    pub fn drain_events(&self) -> impl Iterator<Item = FeedbackEvent> + '_ {
        self.receiver.try_iter()
    }
}

impl Dispatch<wl_registry::WlRegistry, GlobalListContents> for DispatchState {
    fn event(
        _state: &mut Self,
        _proxy: &wl_registry::WlRegistry,
        _event: wl_registry::Event,
        _data: &GlobalListContents,
        _conn: &Connection,
        _qhandle: &QueueHandle<Self>,
    ) {
    }
}

impl Dispatch<wp_presentation::WpPresentation, ()> for DispatchState {
    fn event(
        state: &mut Self,
        _proxy: &wp_presentation::WpPresentation,
        event: wp_presentation::Event,
        _data: &(),
        _conn: &Connection,
        _qhandle: &QueueHandle<Self>,
    ) {
        if let wp_presentation::Event::ClockId { clk_id } = event {
            state.clock_id = Some(clk_id);
        }
    }
}

impl Dispatch<wp_presentation_feedback::WpPresentationFeedback, FeedbackRequest> for DispatchState {
    fn event(
        state: &mut Self,
        _proxy: &wp_presentation_feedback::WpPresentationFeedback,
        event: wp_presentation_feedback::Event,
        data: &FeedbackRequest,
        _conn: &Connection,
        _qhandle: &QueueHandle<Self>,
    ) {
        let result = match event {
            wp_presentation_feedback::Event::Presented {
                tv_sec_hi,
                tv_sec_lo,
                tv_nsec,
                refresh,
                seq_hi,
                seq_lo,
                flags,
            } => {
                let presented_ns = (((tv_sec_hi as u64) << 32) | tv_sec_lo as u64) * 1_000_000_000
                    + tv_nsec as u64;
                let sequence = ((seq_hi as u64) << 32) | seq_lo as u64;
                let flags = match flags {
                    WEnum::Value(value) => value.bits(),
                    WEnum::Unknown(value) => value,
                };
                Some(FeedbackEvent::Presented {
                    frame_id: data.frame_id,
                    requested_ns: data.requested_ns,
                    presented_ns,
                    refresh_ns: refresh,
                    sequence,
                    flags,
                })
            }
            wp_presentation_feedback::Event::Discarded => Some(FeedbackEvent::Discarded {
                frame_id: data.frame_id,
            }),
            wp_presentation_feedback::Event::SyncOutput { .. } => None,
            _ => None,
        };
        if let Some(result) = result {
            state.pending.remove(&data.frame_id);
            let _ = state.sender.send(result);
        }
    }
}

fn clock_now_ns(clock_id: u32) -> Result<u64, String> {
    let clock_id = clock_id as c_int;
    let mut time = libc::timespec {
        tv_sec: 0,
        tv_nsec: 0,
    };
    let result = unsafe { libc::clock_gettime(clock_id, &mut time) };
    if result != 0 {
        return Err(format!(
            "read compositor presentation clock {clock_id}: {}",
            std::io::Error::last_os_error()
        ));
    }
    Ok((time.tv_sec as u64) * 1_000_000_000 + (time.tv_nsec as u64))
}
