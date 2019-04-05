fn init_opencl() -> OpenCL {
	let platform = ocl::Platform::default();
	let device =  match ocl::Device::first(platform) {
		Err(why) => panic!("{:?}", why),
		Ok(d) => d
	};

	let context = ocl::Context::builder().platform(platform).devices(device.clone()).build().unwrap();
	let queue = ocl::Queue::new(&context, device, None).unwrap();

	OpenCL {
		platform,
		device,
		context,
		queue
	}
}

fn render() {
	let s_open_cl: OpenCL = init_opencl();

	let rect_list_buf_gpu = Buffer::<f32>::builder()
		.queue(s_open_cl.queue.clone())
		.flags(ocl::flags::MEM_READ_WRITE)
		.len(RECT_LIST_BUF_SIZE)
		.build();
	
	let image_buf_gpu = Buffer::<f32>::builder()
		.queue(s_open_cl.queue.clone())
		.flags(ocl::flags::MEM_READ_WRITE)
		.len(IMAGE_SIZE_BYTE)
		.build();
}