use std::sync::Arc;
use winit::window::Window;

#[repr(transparent)]
#[derive(Debug, Copy, Clone, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub struct DeviceID(usize);

impl From<usize> for DeviceID {
    fn from(value: usize) -> Self {
        DeviceID(value)
    }
}

impl Into<usize> for DeviceID {
    fn into(self) -> usize { self.0 }
}


pub struct DeviceHandle {
    pub device: wgpu::Device,
    pub queue: wgpu::Queue,
    pub adapter: wgpu::Adapter,
}

impl DeviceHandle {
    async fn new<'s>(ctx: &RenderContext, surface: &wgpu::Surface<'_>) -> Self {
        let adapter = ctx.instance.request_adapter(&wgpu::RequestAdapterOptions{
            power_preference: Default::default(),
            force_fallback_adapter: false, //don't know why, window do not like the fallback adapter
            compatible_surface: Some(&surface),
        }).await.unwrap();

        let (device, queue) = adapter.request_device(&wgpu::DeviceDescriptor{
            label: None, // may add some kind of naming ?
            required_features: Default::default(),
            required_limits: Default::default(),
            memory_hints: Default::default(),
        }, None).await.unwrap();

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
    pub associated_device: DeviceID,
}

impl<'s> RenderSurface<'s> {
    pub fn resize(&mut self, ctx: &RenderContext, (width, height): (u32, u32)) {
        self.config.width = width;
        self.config.height = height;
        ctx.configure_surface(self)
    }
}

pub struct RenderContext {
    instance: wgpu::Instance,
    /* cache devices to reduces resume costs */
    devices: Vec<DeviceHandle>,
}

impl RenderContext {
    pub fn new() -> Self {
        Self {
            instance: wgpu::Instance::new(wgpu::InstanceDescriptor::default()),
            devices: Vec::new(),
        }
    }

    pub fn get_device_handle<'s>(&'s self, surface: &'_ RenderSurface) -> &'s DeviceHandle {
        self.handle_from_id(surface.associated_device)
    }

    pub fn handle_from_id(&self, id: DeviceID) -> &DeviceHandle {
        &self.devices[id.0]
    }

    /** get a suitable device for a given Surface, or create one, and return the associated ID**/
    async fn device_for_surface(&mut self, surface: &wgpu::Surface<'_>) -> DeviceID {
        let device_id = self.devices.iter()
            .position(|device| device.adapter.is_surface_supported(&surface));

        if let Some(device_id) = device_id {
            return device_id.into();
        };

        let device = DeviceHandle::new(&self, surface).await;
        self.devices.push(device);
        DeviceID(self.devices.len() - 1)
    }

    /** create a surface with a given present mode, if not supported, another mod will be used in fallback...
    *   this also ensure a ``DeviceHandle``, containing respective ``wgpu::Device``, ``wpgu::Adpter`` and ``wgpu::Queue``, will be initiated
    *   all of this can be retried using ``RenderContext::device_handle()``
    *   ``RenderSurface::resize()`` must be called once to make the surface valid
    **/
    pub async fn create_surface<'w>(&mut self,
                                    target: impl Into<wgpu::SurfaceTarget<'w>>,
                                    (width, height): (u32, u32),
                                    present_mode: wgpu::PresentMode) -> Result<RenderSurface<'w>, wgpu::CreateSurfaceError> {
        let surface = self.instance.create_surface(target.into())?;

        // find or create a suitable device
        let device_id = self.device_for_surface(&surface).await;
        let handle = self.handle_from_id(device_id);

        let wgpu::SurfaceCapabilities {
            formats,
            ..
        } = surface.get_capabilities(&handle.adapter);

        // chose a texture_format for the swap chain
        let format = formats.into_iter()
            .find(|format| format.is_srgb()).unwrap();

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

        Ok(RenderSurface {
            surface,
            config: surface_config,
            associated_device: device_id,
        })
    }

    pub fn configure_surface(&self, surface: &RenderSurface) {
        let device = self.get_device_handle(surface);
        surface.surface.configure(&device.device, &surface.config)
    }
}