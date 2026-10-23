use super::*;

impl FluxaHost {
    pub fn surface_created(&self, surface: NativeSurface, width: u32, height: u32) {
        let size = [width.max(1), height.max(1)];
        host_log(format!("Surface created: {}x{}", size[0], size[1]));
        let reattached = self
            .with_state(|state| {
                state.generation = state.generation.saturating_add(1);
                state.size = size;
                state.pending_resize = None;
                let Some(gpu) = state.gpu.as_mut() else {
                    return false;
                };
                match gpu.attach_surface(surface.clone(), size) {
                    Ok(()) => true,
                    Err(error) => {
                        host_log(format!("Surface reattach failed: {error}"));
                        state.gpu = None;
                        false
                    }
                }
            })
            .unwrap_or(false);
        if reattached {
            host_log("Surface reattached to existing GPU");
            return;
        }
        let Some((generation, density, artwork_cache_dir, opener)) = self.with_state(|state| {
            (
                state.generation,
                state.density,
                state.artwork_cache_dir.clone(),
                state.video.as_ref().and_then(|video| video.device_opener()),
            )
        }) else {
            return;
        };
        let shared = self.0.clone();
        let task = async move {
            let result = Gpu::create(surface, size, density, artwork_cache_dir, opener).await;
            let Ok(mut state) = shared.lock() else { return };
            if state.generation != generation {
                return;
            }
            match result {
                Ok(gpu) => state.gpu = Some(gpu),
                Err(error) => {
                    host_log(format!("GPU surface init failed: {error}"));
                    #[cfg(not(target_arch = "wasm32"))]
                    fluxa_crash::capture(
                        "GPU init failed on every backend",
                        &[("gpu.error", &error)],
                    );
                }
            }
        };
        #[cfg(target_arch = "wasm32")]
        wasm_bindgen_futures::spawn_local(task);
        #[cfg(not(target_arch = "wasm32"))]
        std::thread::spawn(move || pollster::block_on(task));
    }

    pub fn surface_changed(&self, width: u32, height: u32) {
        let size = [width.max(1), height.max(1)];
        self.with_state(|state| {
            state.size = size;
            state.pending_resize = Some(size);
        });
    }

    pub fn surface_destroyed(&self) {
        self.with_state(|state| {
            state.generation = state.generation.saturating_add(1);
            state.pending_resize = None;
            if let Some(gpu) = state.gpu.as_mut() {
                gpu.detach_surface();
            }
        });
    }

    pub fn gpu_ready(&self) -> bool {
        self.peek(|state| state.gpu.is_some()).unwrap_or(false)
    }

    pub fn render(&self) {
        self.peek(render_frame);
    }

    pub fn render_due(&self) -> bool {
        self.peek(|state| {
            let due = next_redraw(state)
                .is_some_and(|at| at <= Instant::now() + Duration::from_millis(4));
            if due {
                render_frame(state);
            }
            due
        })
        .unwrap_or(false)
    }

    /// Returns when the next frame is due; `None` means the host is idle until input arrives.
    pub fn next_redraw(&self) -> Option<Instant> {
        self.peek(next_redraw).flatten()
    }
}
