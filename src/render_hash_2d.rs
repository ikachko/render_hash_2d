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
