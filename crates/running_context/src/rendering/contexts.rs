use wgpu::Features;

pub struct DeviceHandle {
    pub device: wgpu::Device,
    pub queue: wgpu::Queue,
    pub adapter: wgpu::Adapter,
}

impl DeviceHandle {
    async fn new<'s>(ctx: &RenderContext, surface: &wgpu::Surface<'_>) -> Self {
        let adapter = ctx
            .instance
            .request_adapter(&wgpu::RequestAdapterOptions {
                power_preference: Default::default(),
                force_fallback_adapter: false, //don't know why, window do not like the fallback adapter
                compatible_surface: Some(&surface),
            })
            .await
            .unwrap();

        //assert!(adapter.features().contains(Features::POLYGON_MODE_LINE)); //TODO: support that properly

        let (device, queue) = adapter
            .request_device(
                &wgpu::DeviceDescriptor {
                    label: None, // may add some kind of naming ?
                    required_features: Features::POLYGON_MODE_LINE,
                    required_limits: Default::default(),
                    memory_hints: Default::default(),
                },
                None,
            )
            .await
            .unwrap();

        Self {
            device,
            queue,
            adapter,
        }
    }
}

pub struct RenderSurface<'s> {
    pub surface: wgpu::Surface<'s>,
    pub config: wgpu::SurfaceConfiguration,
}

impl<'s> RenderSurface<'s> {
    pub fn resize(&mut self, ctx: &RenderContext, (width, height): (u32, u32)) {
        self.config.width = width;
        self.config.height = height;
        if self.is_valid() {
            ctx.configure_surface(self);
        }
    }

    pub fn is_valid(&self) -> bool {
        self.config.width > 0 && self.config.height > 0
    }
}

pub struct RenderContext {
    instance: wgpu::Instance,
    /* cache devices to reduces resume costs */
    device: Option<DeviceHandle>,
}

impl RenderContext {
    pub fn new() -> Self {
        Self {
            #[cfg(all(target_arch = "aarch64", target_os = "windows"))]
            instance: wgpu::Instance::new(&wgpu::InstanceDescriptor {
                backends: wgpu::Backends::DX12,
                flags: Default::default(),
                backend_options: Default::default(),
            }),
            #[cfg(not(all(target_arch = "aarch64", target_os = "windows")))]
            instance: wgpu::Instance::default(),
            device: None,
        }
    }

    pub fn device(&self) -> &DeviceHandle {
        self.device.as_ref().expect("called to early")
    }

    /** get a suitable device for a given Surface, or create one, and return the associated ID**/
    async fn device_for_surface(&mut self, surface: &wgpu::Surface<'_>) {
        if let Some(device) = &self.device {
            device.adapter.is_surface_supported(&surface);
            return;
        }

        self.device = Some(DeviceHandle::new(&self, surface).await);
    }

    /** create a surface with a given present mode, if not supported, another mod will be used in fallback...
     *   this also ensure a ``DeviceHandle``, containing respective ``wgpu::Device``, ``wpgu::Adpter`` and ``wgpu::Queue``, will be initiated
     *   all of this can be retried using ``RenderContext::device_handle()``
     **/
    pub async fn create_surface<'w>(
        &mut self,
        target: impl Into<wgpu::SurfaceTarget<'w>>,
        (width, height): (u32, u32),
        present_mode: wgpu::PresentMode,
    ) -> Result<RenderSurface<'w>, wgpu::CreateSurfaceError> {
        let surface = self.instance.create_surface(target.into())?;

        // find or create a suitable device
        self.device_for_surface(&surface).await;

        let wgpu::SurfaceCapabilities { formats, .. } =
            surface.get_capabilities(&self.device().adapter);

        // chose a texture_format for the swap chain
        let format = formats
            .into_iter()
            .find(|format| !format.is_srgb() && format.components() == 4)
            .unwrap();

        let surface_config = wgpu::SurfaceConfiguration {
            usage: wgpu::TextureUsages::RENDER_ATTACHMENT,
            format,
            width,
            height,
            present_mode,
            desired_maximum_frame_latency: 2,
            alpha_mode: wgpu::CompositeAlphaMode::Auto,
            view_formats: vec![],
        };

        let surface = RenderSurface {
            surface,
            config: surface_config,
        };

        self.configure_surface(&surface);

        Ok(surface)
    }

    pub fn configure_surface(&self, surface: &RenderSurface) {
        let device = self.device();
        surface.surface.configure(&device.device, &surface.config)
    }
}
