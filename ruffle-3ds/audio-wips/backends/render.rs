fn get_wgpu_descriptors() -> Arc<Descriptors> {
    let instance = wgpu::Instance::new(Default::default());
    let (adapter, device, queue) = futures::executor::block_on(request_adapter_and_device(
        wgpu::Backends::all(),
        &instance,
        None,
        Default::default(),
        None,
    ))
        .unwrap();

    Arc::new(Descriptors::new(instance, adapter, device, queue))
}

fn get_renderer() -> WgpuRenderBackend {
    let descriptors = get_wgpu_descriptors();
    let movie = SwfMovie::from_data(include_bytes!("../swf/sample.swf"), "".into(), None).unwrap();
    let target = TextureTarget::new(&descriptors.device, (
        movie.width().to_pixels() as u32,
        movie.height().to_pixels() as u32,
    )).unwrap();
    WgpuRenderBackend::new(descriptors, target).unwrap()
}