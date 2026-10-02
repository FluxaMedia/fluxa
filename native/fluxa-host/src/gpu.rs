use super::*;
use fluxa_renderer::glass::GlassStyle;

pub(super) fn surface_alpha_mode(modes: &[wgpu::CompositeAlphaMode]) -> wgpu::CompositeAlphaMode {
    if cfg!(any(
        target_os = "android",
        target_os = "ios",
        target_os = "tvos",
        target_os = "macos"
    )) && let Some(mode) = [
        wgpu::CompositeAlphaMode::PreMultiplied,
        wgpu::CompositeAlphaMode::PostMultiplied,
        wgpu::CompositeAlphaMode::Inherit,
    ]
    .into_iter()
    .find(|mode| modes.contains(mode))
    {
        return mode;
    }
    modes
        .first()
        .copied()
        .unwrap_or(wgpu::CompositeAlphaMode::Auto)
}

pub(super) struct Gpu {
    pub(super) instance: wgpu::Instance,
    _surface: NativeSurface,
    adapter: wgpu::Adapter,
    surface: Option<wgpu::Surface<'static>>,
    pub(super) device: wgpu::Device,
    queue: wgpu::Queue,
    pub(super) config: wgpu::SurfaceConfiguration,
    clear_color: wgpu::Color,
    pub(super) egui_context: egui::Context,
    pub(super) egui_renderer: EguiWgpuBackend,
    icons: SvgIconRegistry,
    background_texture: egui::TextureHandle,
    brand_mark: Option<egui::TextureHandle>,
    brand_icon: &'static fluxa_core::app_icon::AppIcon,
    app_icons: Vec<(String, egui::TextureHandle)>,
    ambient_glow: egui::TextureHandle,
    pub(super) artwork: ArtworkLoader,
    started_at: Instant,
    pub(super) density: f32,
    adapter_name: String,
}

pub(super) struct FrameOutput {
    pub(super) layout: HomeLayout,
    pub(super) cursor: egui::CursorIcon,
    pub(super) wants_keyboard: bool,
    pub(super) repaint_delay: Duration,
    pub(super) menu_outcome: Option<fluxa_ui::ActionMenuOutcome>,
}

pub(super) struct HostAssets<'a> {
    background: egui::TextureId,
    brand_mark: Option<egui::TextureId>,
    brand_colors: Option<[egui::Color32; 2]>,
    app_icons: &'a [(String, egui::TextureHandle)],
    ambient_glow: egui::TextureId,
    accent: Option<egui::Color32>,
    pub(super) artwork: &'a mut ArtworkLoader,
    icons: &'a SvgIconRegistry,
    profile_name: Option<&'a str>,
    profile_avatar_url: Option<&'a str>,
    custom_background: Option<&'a str>,
}

impl HomeAssets for HostAssets<'_> {
    fn background(&self) -> egui::TextureId {
        self.background
    }
    fn active_profile_name(&self) -> Option<&str> {
        self.profile_name
    }
    fn brand_mark(&self) -> Option<egui::TextureId> {
        self.brand_mark
    }
    fn brand_colors(&self) -> Option<[egui::Color32; 2]> {
        self.brand_colors
    }
    fn app_icon(&self, id: &str) -> Option<egui::TextureId> {
        self.app_icons
            .iter()
            .find(|(icon, _)| icon == id)
            .map(|(_, texture)| texture.id())
    }
    fn ambient_glow(&self) -> Option<egui::TextureId> {
        Some(self.ambient_glow)
    }
    fn accent_color(&self) -> Option<egui::Color32> {
        self.accent
    }
    fn custom_background_url(&self) -> Option<&str> {
        self.custom_background
    }
    fn active_profile_avatar_url(&self) -> Option<&str> {
        self.profile_avatar_url
    }
    fn texture(&mut self, url: Option<&str>) -> Option<egui::TextureId> {
        self.artwork.texture(url)
    }
    fn has_native_emoji(&self) -> bool {
        crate::EMOJI_RASTERIZER.get().is_some()
    }
    fn emoji(&mut self, cluster: &str) -> Option<egui::TextureId> {
        self.artwork.emoji(cluster)
    }
    fn texture_for(
        &mut self,
        url: Option<&str>,
        target_size: [u32; 2],
        priority: ArtworkPriority,
    ) -> Option<egui::TextureId> {
        self.artwork.texture_for_priority(
            url,
            target_size,
            match priority {
                ArtworkPriority::Hero => ArtworkFetchPriority::Hero,
                ArtworkPriority::Visible => ArtworkFetchPriority::Visible,
                ArtworkPriority::Prefetch => ArtworkFetchPriority::Prefetch,
            },
        )
    }
    fn animated_texture_for(
        &mut self,
        url: Option<&str>,
        target_size: [u32; 2],
        priority: ArtworkPriority,
    ) -> Option<AnimatedTexture> {
        self.artwork.animated_texture_for_priority(
            url,
            target_size,
            match priority {
                ArtworkPriority::Hero => ArtworkFetchPriority::Hero,
                ArtworkPriority::Visible => ArtworkFetchPriority::Visible,
                ArtworkPriority::Prefetch => ArtworkFetchPriority::Prefetch,
            },
        )
    }
    fn prefetch_for(
        &mut self,
        url: Option<&str>,
        target_size: [u32; 2],
        priority: ArtworkPriority,
    ) {
        let _ = self.artwork.texture_for_priority(
            url,
            target_size,
            match priority {
                ArtworkPriority::Hero => ArtworkFetchPriority::Hero,
                ArtworkPriority::Visible => ArtworkFetchPriority::Visible,
                ArtworkPriority::Prefetch => ArtworkFetchPriority::Prefetch,
            },
        );
    }
    fn prefetch_animated_for(
        &mut self,
        url: Option<&str>,
        target_size: [u32; 2],
        priority: ArtworkPriority,
    ) {
        self.artwork.prefetch_animated_for_priority(
            url,
            target_size,
            match priority {
                ArtworkPriority::Hero => ArtworkFetchPriority::Hero,
                ArtworkPriority::Visible => ArtworkFetchPriority::Visible,
                ArtworkPriority::Prefetch => ArtworkFetchPriority::Prefetch,
            },
        );
    }
    fn artwork_tones(&self, url: Option<&str>) -> Option<[[u8; 3]; 2]> {
        self.artwork.tones(url)
    }
    fn texture_size(&self, url: Option<&str>) -> Option<[u32; 2]> {
        self.artwork.size(url)
    }
    fn cached_texture(&self, url: Option<&str>) -> Option<egui::TextureId> {
        self.artwork.cached_texture(url)
    }
    fn icon(&self, name: &str) -> Option<egui::TextureId> {
        self.icons.texture(name)
    }
    fn logo(&self, name: &str) -> Option<(egui::TextureId, egui::Vec2)> {
        let texture = self.icons.textures.get(name)?;
        let [width, height] = texture.size();
        Some((texture.id(), egui::vec2(width as f32 / height as f32, 1.0)))
    }
}

pub(super) struct SvgIconRegistry {
    icons: HashMap<&'static str, egui::TextureId>,
    textures: HashMap<&'static str, egui::TextureHandle>,
}

impl SvgIconRegistry {
    pub(super) fn new(
        context: &egui::Context,
        device: &wgpu::Device,
        queue: &wgpu::Queue,
        renderer: &mut EguiWgpuBackend,
    ) -> Self {
        let mut icons = HashMap::new();
        for (name, svg) in ICONS {
            let Ok(image) = rasterize_svg(svg.as_bytes(), ICON_SIZE) else {
                host_log(format!("failed to rasterize shared SVG icon {name}"));
                continue;
            };
            icons.insert(*name, upload_mipmapped(device, queue, renderer, image));
        }
        let mut textures = HashMap::new();
        for (name, svg) in LOGOS {
            let Ok(image) = rasterize_svg(svg.as_bytes(), LOGO_SIZE) else {
                host_log(format!("failed to rasterize shared SVG logo {name}"));
                continue;
            };
            let image = egui::ColorImage::from_rgba_premultiplied(
                [image.width() as usize, image.height() as usize],
                image.as_raw(),
            );
            let texture = context.load_texture(
                format!("fluxa-shared-svg-logo-{name}"),
                image,
                egui::TextureOptions::LINEAR,
            );
            textures.insert(*name, texture);
        }
        Self { icons, textures }
    }

    pub(super) fn texture(&self, name: &str) -> Option<egui::TextureId> {
        self.icons
            .get(name)
            .copied()
            .or_else(|| self.textures.get(name).map(egui::TextureHandle::id))
    }
}

fn upload_mipmapped(
    device: &wgpu::Device,
    queue: &wgpu::Queue,
    renderer: &mut EguiWgpuBackend,
    image: image::RgbaImage,
) -> egui::TextureId {
    let mut levels = vec![image];
    while levels
        .last()
        .is_some_and(|level| level.width() > 1 && level.height() > 1)
    {
        let prev = levels.last().unwrap();
        let (width, height) = (prev.width() / 2, prev.height() / 2);
        let next = image::RgbaImage::from_fn(width, height, |x, y| {
            let mut sum = [0u32; 4];
            for (dx, dy) in [(0, 0), (1, 0), (0, 1), (1, 1)] {
                let pixel = prev.get_pixel(x * 2 + dx, y * 2 + dy);
                for channel in 0..4 {
                    sum[channel] += pixel[channel] as u32;
                }
            }
            image::Rgba(sum.map(|value| ((value + 2) / 4) as u8))
        });
        levels.push(next);
    }
    let texture = device.create_texture(&wgpu::TextureDescriptor {
        label: Some("fluxa-svg-icon"),
        size: wgpu::Extent3d {
            width: levels[0].width(),
            height: levels[0].height(),
            depth_or_array_layers: 1,
        },
        mip_level_count: levels.len() as u32,
        sample_count: 1,
        dimension: wgpu::TextureDimension::D2,
        format: wgpu::TextureFormat::Rgba8Unorm,
        usage: wgpu::TextureUsages::TEXTURE_BINDING | wgpu::TextureUsages::COPY_DST,
        view_formats: &[],
    });
    for (mip, level) in levels.iter().enumerate() {
        queue.write_texture(
            wgpu::TexelCopyTextureInfo {
                texture: &texture,
                mip_level: mip as u32,
                origin: wgpu::Origin3d::ZERO,
                aspect: wgpu::TextureAspect::All,
            },
            level.as_raw(),
            wgpu::TexelCopyBufferLayout {
                offset: 0,
                bytes_per_row: Some(level.width() * 4),
                rows_per_image: Some(level.height()),
            },
            wgpu::Extent3d {
                width: level.width(),
                height: level.height(),
                depth_or_array_layers: 1,
            },
        );
    }
    let view = texture.create_view(&wgpu::TextureViewDescriptor::default());
    renderer.register_mipmapped_texture(device, &view)
}

impl Gpu {
    pub(super) async fn create(
        surface: NativeSurface,
        size: [u32; 2],
        density: f32,
        artwork_cache_dir: Option<PathBuf>,
        opener: Option<DeviceOpener>,
    ) -> Result<Self, String> {
        host_log(format!(
            "GPU init start: surface={}x{}, density={density:.2}",
            size[0], size[1]
        ));
        let mut errors = Vec::new();
        for &backend in surface.backends {
            host_log(format!("Trying {} backend", backend.label()));
            match Self::create_for_backend(
                &surface,
                size,
                density,
                artwork_cache_dir.clone(),
                backends_for(backend),
                opener.clone(),
            )
            .await
            {
                Ok(gpu) => {
                    host_log(format!(
                        "GPU ready: {} ({})",
                        gpu.adapter_name,
                        backend.label()
                    ));
                    return Ok(gpu);
                }
                Err(error) => {
                    host_log(format!("{} backend failed: {error}", backend.label()));
                    errors.push(format!("{}: {error}", backend.label()));
                }
            }
        }
        Err(format!("no GPU backend succeeded ({})", errors.join("; ")))
    }

    async fn create_for_backend(
        native_surface: &NativeSurface,
        size: [u32; 2],
        density: f32,
        artwork_cache_dir: Option<PathBuf>,
        backends: wgpu::Backends,
        opener: Option<DeviceOpener>,
    ) -> Result<Self, String> {
        let mut descriptor = wgpu::InstanceDescriptor::new_without_display_handle();
        descriptor.backends = backends;
        let instance = wgpu::Instance::new(descriptor);
        let surface = unsafe {
            instance
                .create_surface_unsafe((native_surface.target)()?)
                .map_err(|error| error.to_string())?
        };
        let adapter = instance
            .request_adapter(&wgpu::RequestAdapterOptions {
                power_preference: wgpu::PowerPreference::HighPerformance,
                compatible_surface: Some(&surface),
                force_fallback_adapter: false,
            })
            .await
            .map_err(|error| error.to_string())?;
        let info = adapter.get_info();
        let adapter_name = info.name.clone();
        host_log(format!(
            "Adapter selected: {:?} / {}",
            info.backend, info.name
        ));
        let required_limits = if cfg!(target_arch = "wasm32") {
            wgpu::Limits::downlevel_webgl2_defaults().using_resolution(adapter.limits())
        } else {
            wgpu::Limits::default()
        };
        let device_descriptor = wgpu::DeviceDescriptor {
            label: Some("fluxa-host-device"),
            required_features: wgpu::Features::empty(),
            required_limits,
            memory_hints: wgpu::MemoryHints::Performance,
            trace: wgpu::Trace::Off,
            experimental_features: wgpu::ExperimentalFeatures::disabled(),
        };
        let (device, queue) = match opener {
            Some(opener) => opener(&adapter, &device_descriptor)?,
            None => adapter
                .request_device(&device_descriptor)
                .await
                .map_err(|error| error.to_string())?,
        };
        host_log("Device requested successfully");
        let capabilities = surface.get_capabilities(&adapter);
        let format = capabilities
            .formats
            .iter()
            .copied()
            .find(wgpu::TextureFormat::is_srgb)
            .or_else(|| capabilities.formats.first().copied())
            .ok_or_else(|| "surface has no compatible format".to_owned())?;
        let config = wgpu::SurfaceConfiguration {
            usage: wgpu::TextureUsages::RENDER_ATTACHMENT,
            format,
            width: size[0].max(1),
            height: size[1].max(1),
            present_mode: if capabilities
                .present_modes
                .contains(&wgpu::PresentMode::Mailbox)
            {
                wgpu::PresentMode::Mailbox
            } else {
                wgpu::PresentMode::Fifo
            },
            alpha_mode: surface_alpha_mode(&capabilities.alpha_modes),
            view_formats: vec![],
            desired_maximum_frame_latency: 2,
        };
        surface.configure(&device, &config);
        host_log(format!(
            "Surface configured: {:?}, {}x{}, {:?} of {:?}",
            format, config.width, config.height, config.present_mode, capabilities.present_modes
        ));
        let clear = fluxa_renderer::theme::theme("fluxa-dark")
            .and_then(|theme| theme.color("background"))
            .map(|color| wgpu::Color {
                r: f64::from(color.r),
                g: f64::from(color.g),
                b: f64::from(color.b),
                a: f64::from(color.a),
            })
            .unwrap_or(wgpu::Color {
                r: 0.0235,
                g: 0.0235,
                b: 0.0235,
                a: 1.0,
            });
        let egui_context = egui::Context::default();
        egui_context.set_fonts(fluxa_ui::fonts::definitions());
        let background_image = {
            let image = fluxa_renderer::ambient_background();
            egui::ColorImage::from_rgba_unmultiplied(
                [image.width() as usize, image.height() as usize],
                image.as_raw(),
            )
        };
        let background_texture = egui_context.load_texture(
            "fluxa-native-background",
            background_image,
            egui::TextureOptions::LINEAR,
        );
        let brand_mark = image::load_from_memory(BRAND_MARK_BYTES).ok().map(|image| {
            let image = image.to_rgba8();
            egui_context.load_texture(
                "fluxa-brand-mark",
                egui::ColorImage::from_rgba_unmultiplied(
                    [image.width() as usize, image.height() as usize],
                    image.as_raw(),
                ),
                egui::TextureOptions::LINEAR,
            )
        });
        let ambient_glow = {
            let image = fluxa_renderer::ambient_glow();
            egui_context.load_texture(
                "fluxa-ambient-glow",
                egui::ColorImage::from_rgba_unmultiplied(
                    [image.width() as usize, image.height() as usize],
                    image.as_raw(),
                ),
                egui::TextureOptions::LINEAR,
            )
        };
        let mut egui_renderer = EguiWgpuBackend::new(&device, format);
        host_log("egui renderer created");
        let icons = SvgIconRegistry::new(&egui_context, &device, &queue, &mut egui_renderer);
        fluxa_ui::set_rating_logos(
            &egui_context,
            ["imdb", "mdblist"]
                .into_iter()
                .filter_map(|name| {
                    let texture = icons.textures.get(name)?;
                    let [width, height] = texture.size();
                    Some((
                        name,
                        texture.id(),
                        egui::vec2(width as f32 / height as f32, 1.0),
                    ))
                })
                .collect(),
        );
        host_log(format!(
            "Shared SVG icons uploaded: {}",
            icons.textures.len()
        ));
        Ok(Self {
            instance: instance,
            _surface: native_surface.clone(),
            adapter,
            surface: Some(surface),
            device,
            queue,
            config,
            clear_color: clear,
            egui_context,
            egui_renderer,
            icons,
            background_texture,
            brand_mark,
            brand_icon: fluxa_core::app_icon::default_app_icon(),
            app_icons: Vec::new(),
            ambient_glow,
            artwork: ArtworkLoader::new(artwork_cache_dir),
            started_at: Instant::now(),
            density,
            adapter_name,
        })
    }

    pub(super) fn resize(&mut self, size: [u32; 2]) {
        self.config.width = size[0].max(1);
        self.config.height = size[1].max(1);
        if let Some(surface) = self.surface.as_ref() {
            surface.configure(&self.device, &self.config);
        }
    }

    pub(super) fn detach_surface(&mut self) {
        self.surface = None;
    }

    pub(super) fn attach_surface(
        &mut self,
        native_surface: NativeSurface,
        size: [u32; 2],
    ) -> Result<(), String> {
        let surface = unsafe {
            self.instance
                .create_surface_unsafe((native_surface.target)()?)
                .map_err(|error| error.to_string())?
        };
        let capabilities = surface.get_capabilities(&self.adapter);
        if !capabilities.formats.contains(&self.config.format) {
            return Err("surface format changed".to_owned());
        }
        self.config.width = size[0].max(1);
        self.config.height = size[1].max(1);
        surface.configure(&self.device, &self.config);
        self.surface = Some(surface);
        self._surface = native_surface;
        Ok(())
    }

    pub(super) fn render(
        &mut self,
        route: Route,
        home: &HomeModel,
        library: &LibraryModel,
        library_tab: LibraryTab,
        discover: &DiscoverModel,
        calendar: &CalendarModel,
        shorts: &fluxa_ui::ShortsModel,
        detail: &DetailModel,
        settings: &SettingsModel,
        mut profiles: Option<&mut fluxa_ui::ProfilesModel>,
        custom_background: Option<&str>,
        player: Option<&PlayerModel>,
        focused: Option<u64>,
        safe_bottom: f32,
        scroll_y: f32,
        events: Vec<egui::Event>,
        modifiers: egui::Modifiers,
        pre_present: Option<&PrePresent>,
        menu: Option<&card_menu::MenuView>,
        timer: &mut FrameTimer,
    ) -> Result<FrameOutput, String> {
        let passthrough =
            player.is_some_and(|player| player.passthrough && player.recommendations.is_empty());
        self.artwork.poll(&self.egui_context);
        self.artwork.begin_frame();
        timer.mark("artwork");
        let screen_size = [self.config.width, self.config.height];
        let logical_size = [
            (screen_size[0] as f32 / self.density).round().max(1.0) as u32,
            (screen_size[1] as f32 / self.density).round().max(1.0) as u32,
        ];
        let raw_input = egui::RawInput {
            max_texture_side: Some(self.device.limits().max_texture_dimension_2d as usize),
            viewports: std::iter::once((
                egui::ViewportId::ROOT,
                egui::ViewportInfo {
                    native_pixels_per_point: Some(self.density),
                    ..Default::default()
                },
            ))
            .collect(),
            screen_rect: Some(EguiRect::from_min_size(
                Pos2::ZERO,
                Vec2::new(logical_size[0] as f32, logical_size[1] as f32),
            )),
            time: Some(self.started_at.elapsed().as_secs_f64()),
            events,
            modifiers,
            focused: true,
            ..Default::default()
        };
        let brand_icon = fluxa_core::app_icon::resolve_app_icon(settings.app_icon());
        if brand_icon.id != self.brand_icon.id {
            self.brand_icon = brand_icon;
            if let Some(image) = crate::app_icon_rgba(&brand_icon.id, 512) {
                self.brand_mark = Some(self.egui_context.load_texture(
                    "fluxa-brand-mark",
                    egui::ColorImage::from_rgba_premultiplied(
                        [image.width() as usize, image.height() as usize],
                        image.as_raw(),
                    ),
                    egui::TextureOptions::LINEAR,
                ));
            }
        }
        if route == Route::Settings && self.app_icons.is_empty() {
            for icon in fluxa_core::app_icon::app_icons() {
                if let Some(image) = crate::app_icon_rgba(&icon.id, 128) {
                    let texture = self.egui_context.load_texture(
                        format!("fluxa-app-icon-{}", icon.id),
                        egui::ColorImage::from_rgba_premultiplied(
                            [image.width() as usize, image.height() as usize],
                            image.as_raw(),
                        ),
                        egui::TextureOptions::LINEAR,
                    );
                    self.app_icons.push((icon.id.to_string(), texture));
                }
            }
        }
        fluxa_ui::set_liquid_glass(settings.bool_value("liquidGlass"));
        fluxa_ui::set_poster_landscape(settings.poster_landscape());
        fluxa_ui::set_nav_language(&home.language);
        fluxa_ui::set_mobile_nav_style(
            settings.bool_value("navFloating"),
            settings.bool_value("navLabels"),
        );
        fluxa_ui::set_poster_overlays(&self.egui_context, settings.poster_overlays());
        fluxa_ui::set_top_numbers(settings.top_numbers());
        fluxa_ui::set_poster_personal(&self.egui_context, library.personal.clone());
        let mut rendered_layout = HomeLayout::default();
        let mut menu_outcome = None;
        let _ = self.artwork.texture_for_priority(
            home.profile_avatar_url.as_deref(),
            [96, 96],
            ArtworkFetchPriority::Visible,
        );
        let output = self.egui_context.run_ui(raw_input, |ui| {
            let mut assets = HostAssets {
                background: self.background_texture.id(),
                brand_mark: self.brand_mark.as_ref().map(egui::TextureHandle::id),
                brand_colors: Some([
                    hex_color(&self.brand_icon.from),
                    hex_color(&self.brand_icon.to),
                ]),
                app_icons: &self.app_icons,
                ambient_glow: self.ambient_glow.id(),
                accent: home.accent,
                artwork: &mut self.artwork,
                icons: &self.icons,
                profile_name: home.profile_name.as_deref(),
                profile_avatar_url: home.profile_avatar_url.as_deref(),
                custom_background,
            };
            let viewport = Viewport::new(logical_size[0], logical_size[1], home.form_factor.into())
                .with_platform(home.platform)
                .with_safe_bottom(safe_bottom)
                .with_scroll_y(scroll_y);
            if let Some(profiles) = profiles.as_deref_mut() {
                rendered_layout =
                    fluxa_ui::draw_profiles(ui.ctx(), viewport, profiles, &mut assets);
            } else if let Some(player) = player {
                rendered_layout = draw_player(ui.ctx(), viewport, player, &mut assets, focused);
            } else if route == Route::Library {
                rendered_layout = draw_library(
                    ui.ctx(),
                    viewport,
                    library,
                    library_tab,
                    &mut assets,
                    focused,
                );
            } else if route == Route::Discover {
                rendered_layout = draw_discover(ui.ctx(), viewport, discover, &mut assets, focused);
            } else if route == Route::Calendar {
                rendered_layout = draw_calendar(ui.ctx(), viewport, calendar, &mut assets, focused);
            } else if route == Route::Shorts {
                rendered_layout =
                    fluxa_ui::draw_shorts(ui.ctx(), viewport, shorts, &mut assets, focused);
            } else if route == Route::Detail {
                rendered_layout = draw_detail(ui.ctx(), viewport, detail, &mut assets, focused);
            } else if route == Route::Settings {
                rendered_layout = draw_settings(ui.ctx(), viewport, settings, &mut assets, focused);
            } else {
                rendered_layout = draw_home(ui.ctx(), viewport, home, &mut assets, focused);
            }
            let page = if profiles.is_some() {
                100
            } else if player.is_some() {
                101
            } else {
                route as u64
            };
            fluxa_ui::page_transition(ui.ctx(), page, &[egui::Id::new("fluxa-shared-bottom-bar")]);
            if let Some(menu) = menu {
                menu_outcome = fluxa_ui::draw_action_menu(
                    ui.ctx(),
                    viewport,
                    fluxa_ui::metrics_for_assets(viewport, &assets),
                    &assets,
                    &menu.title,
                    &menu.items,
                    menu.selected,
                    menu.anchor,
                    menu.serial,
                );
            }
        });
        timer.mark("draw");
        let paint_jobs = self
            .egui_context
            .tessellate(output.shapes, output.pixels_per_point);
        self.egui_renderer
            .apply_texture_deltas(&self.device, &self.queue, &output.textures_delta);
        timer.mark("upload");
        let screen_descriptor = ScreenDescriptor {
            size_in_pixels: screen_size,
            pixels_per_point: output.pixels_per_point,
        };
        let Some(surface) = self.surface.as_ref() else {
            return Err("surface detached".to_owned());
        };
        let frame = match surface.get_current_texture() {
            wgpu::CurrentSurfaceTexture::Success(frame)
            | wgpu::CurrentSurfaceTexture::Suboptimal(frame) => frame,
            wgpu::CurrentSurfaceTexture::Timeout => return Err("surface timeout".to_owned()),
            wgpu::CurrentSurfaceTexture::Occluded => return Err("surface occluded".to_owned()),
            wgpu::CurrentSurfaceTexture::Outdated => return Err("surface outdated".to_owned()),
            wgpu::CurrentSurfaceTexture::Lost => return Err("surface lost".to_owned()),
            wgpu::CurrentSurfaceTexture::Validation => {
                return Err("surface validation error".to_owned());
            }
        };
        timer.mark("acquire");
        let view = frame
            .texture
            .create_view(&wgpu::TextureViewDescriptor::default());
        let commands = self.egui_renderer.render_frame(
            &self.device,
            &self.queue,
            &view,
            if passthrough {
                wgpu::Color::TRANSPARENT
            } else {
                self.clear_color
            },
            &paint_jobs,
            &screen_descriptor,
            |callback| {
                let glass = callback.callback.downcast_ref::<fluxa_ui::Glass>()?;
                Some(GlassStyle {
                    radius: glass.radius,
                    tint: glass.tint,
                    refraction: glass.refraction,
                    bevel: glass.bevel,
                    rim: glass.rim,
                })
            },
        );
        self.queue.submit([commands]);
        if let Some(pre_present) = pre_present {
            pre_present();
        }
        frame.present();
        timer.mark("present");
        self.egui_renderer
            .free_texture_deltas(&output.textures_delta);
        Ok(FrameOutput {
            layout: rendered_layout,
            cursor: output.platform_output.cursor_icon,
            wants_keyboard: self.egui_context.egui_wants_keyboard_input(),
            repaint_delay: output
                .viewport_output
                .get(&egui::ViewportId::ROOT)
                .map_or(Duration::ZERO, |viewport| viewport.repaint_delay),
            menu_outcome,
        })
    }
}

pub(super) const BRAND_MARK_BYTES: &[u8] = include_bytes!("../assets/fluxa.png");

fn hex_color(hex: &str) -> egui::Color32 {
    let value = u32::from_str_radix(hex.trim_start_matches('#'), 16).unwrap_or(0xFFFFFF);
    egui::Color32::from_rgb((value >> 16) as u8, (value >> 8) as u8, value as u8)
}
