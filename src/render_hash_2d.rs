extern crate ocl;

use ocl::{Buffer};

const IMAGE_SIZE_X: usize = 1920 * 2;
const IMAGE_SIZE_Y: usize = 1080 * 2;
const IMAGE_SIZE_BYTE: usize = IMAGE_SIZE_X * IMAGE_SIZE_Y * 4;
const RECT_LIST_BUF_SIZE: usize = 1000 * 8 * 4;

struct OpenCL {
	platform: ocl::Platform,
	device: ocl::Device,
	context: ocl::Context,
	queue: ocl::Queue
}

pub struct RenderCL {
	queue: ocl::Queue,
	render_kern: ocl::Kernel,
	imgsize: usize,

	imgwidth: usize,
	imgheight: usize,
	imgbuf: ocl::Buffer<u8>,
}

impl RenderCL {
	pub fn new() -> RenderCL {
		let platform = ocl::Platform::default();
        let device = ocl::Device::first(platform).unwrap();   /* TODO: Should be smarter with selecting GPU */
        let context = ocl::Context::builder().platform(platform).devices(device.clone()).build().unwrap();
        let queue = ocl::Queue::new(&context, device, None).unwrap();

		// let rect_list_buf_gpu = ocl::Buffer::<u8>::builder().queue(queue.clone()).flags(ocl::)
		let rect_list_buf_gpu = ocl::Buffer::<u8>::builder()
			.queue(queue.clone())
			.flags(ocl::flags::MEM_READ_ONLY)
			.dims(RECT_LIST_BUF_SIZE)
			.unwrap();
	}
}

fn init_opencl() -> OpenCL {
	let platform = ocl::Platform::default();
	let device =  match ocl::Device::first(platform) {
		Err(why) => panic!("{:?}", why),
		Ok(d) => d
	};

	// let device = ocl::Device::list(platform)[1];
	// let devices = ocl::Device::list(platform, );

	let context = ocl::Context::builder().platform(platform).devices(device.clone()).build().unwrap();
	let queue = ocl::Queue::new(&context, device, None).unwrap();

	OpenCL {
		platform,
		device,
		context,
		queue
	}
}

pub fn render_hash_2d() {
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
