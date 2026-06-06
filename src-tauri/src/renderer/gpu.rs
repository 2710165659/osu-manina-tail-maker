use skia_safe::Surface;

pub enum RenderBackend {
    Cpu,
}

pub fn detect_gpu_backend() -> RenderBackend {
    // GPU support can be enabled in future with gl feature
    RenderBackend::Cpu
}

pub fn create_surface(width: i32, height: i32, _backend: &RenderBackend) -> Option<Surface> {
    skia_safe::surfaces::raster_n32_premul((width, height))
}
