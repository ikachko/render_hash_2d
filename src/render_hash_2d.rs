extern crate ocl;
extern crate ocl_core;

use ocl::{Buffer};
// use ocl_core;

use std::fs::{self, File};
use std::io;
use sha2::{Sha256, Digest};
use std::path::Path;
use std::io::BufWriter;
use png::HasParameters;

const TEX_SIZE_X: usize = 1920;
const TEX_SIZE_Y: usize = 1080;

const RECT_COUNT: usize = 6;
const TEX_COUNT: usize = 4;

const IMAGE_SIZE_X: usize = 1920 * 2;
const IMAGE_SIZE_Y: usize = 1080 * 2;
const IMAGE_SIZE_BYTE: usize = IMAGE_SIZE_X * IMAGE_SIZE_Y * 4;
const RECT_LIST_BUF_SIZE: usize = 1000 * 8 * 4;
const RECT_LIST_LENGTH: u8 = 6;
const TEX_SIZE_CHAR4: usize = TEX_SIZE_X * TEX_SIZE_Y * TEX_COUNT;

enum FileReadError {
	IOError(io::Error),
	DecodeError(png::DecodingError),
}

impl From<io::Error> for FileReadError {
	fn from(e: io::Error) -> FileReadError {
		FileReadError::IOError(e)
	}
}

impl From<png::DecodingError> for FileReadError {
	fn from(e: png::DecodingError) -> FileReadError {
		FileReadError::DecodeError(e)
	}
}

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

struct Rect {
	x: usize,
	y: usize,
	w: usize,
	h: usize,
	t: usize
}
/// Function: sha256_hash 
/// --------------------
/// @desc function for sha256 hash of u8 array
/// @param &[u8] data - message, that need to be hashed
/// @return [u8; 32] - returns 32 byte array
fn sha256_hash(data: &[u8]) -> [u8; 32] {
	let mut ret = [0; 32];
	let mut sha2 = Sha256::new();
	sha2.input(data);
	ret.copy_from_slice(sha2.result().as_slice());
	ret
}

impl RenderCL {
	pub fn new() {
		let platform = ocl::Platform::default();
        let device = ocl::Device::first(platform);   /* TODO: Should be smarter with selecting GPU */
        let context = ocl::Context::builder().platform(platform).devices(device.clone()).build().unwrap();
        let queue = ocl::Queue::new(&context, device, None).unwrap();

		// let rect_list_buf_gpu = ocl::Buffer::<u8>::builder().queue(queue.clone()).flags(ocl::)
		let rect_list_buf_gpu = ocl::builders::BufferBuilder::<u8>::new()
			.flags(ocl::flags::MEM_READ_WRITE)
			.queue(queue.clone())
			.dims(RECT_LIST_BUF_SIZE)
			.build()
			.unwrap();
	
		println!("{:#?}", rect_list_buf_gpu);
		// let rect_list_buf_gpu = ocl::Buffer::<u8>::builder()
			// .queue(queue.clone())
			// .flags(ocl::flags::MEM_READ_ONLY)
			// .dims(RECT_LIST_BUF_SIZE)
			// .unwrap();
	}
}

fn dump_image(file_name: &str, image: &Vec<u8>, width: u32, height: u32) {
	let path = Path::new(file_name);
	let file = File::create(path).unwrap();

	let ref mut w = BufWriter::new(file);

	let mut encoder = png::Encoder::new(w, width, height);
	encoder.set(png::ColorType::RGBA).set(png::BitDepth::Eight);

	let mut writer = encoder.write_header().unwrap();

	writer.write_image_data(&image).unwrap();
	println!("Image {} is dumped.", &file_name);
}

fn read_files(dir: &str, s_ocl: &OpenCL, printable: bool) -> Result<ocl::Buffer::<ocl_core::Char4>, FileReadError> {
	let paths = fs::read_dir(dir)?;
	let tex_size_bytes = TEX_SIZE_X * TEX_SIZE_Y * 4 * TEX_COUNT;
	
	let tex_size_char4 = TEX_SIZE_X * TEX_SIZE_Y * TEX_COUNT;

	let mut image_atlas = vec![0; tex_size_bytes];
	
	let mut tex_offset = 0;
	let mut tex_offset_png = 0;

	let tex_size_bytes = TEX_SIZE_X * TEX_SIZE_Y * 4 * TEX_COUNT;
	// let mut tex_buf_host: Vec<u8> = vec![0; TEX_SIZE_X * TEX_SIZE_Y * 4];
	let mut image_atlas_buf_gpu = ocl::Buffer::<ocl_core::Char4>::builder()
		.queue(s_ocl.queue.clone())
		.flags(ocl::flags::MEM_WRITE_ONLY)
		.dims(tex_size_bytes)
		.build()
		.unwrap();
	let mut image_atlas: Vec<ocl_core::Char4> = vec![ocl_core::Char4::new(0,0,0,0); tex_size_char4];
	let mut image_atlas_png = vec![0; tex_size_bytes];

	for path in paths {
		let decoder = png::Decoder::new(File::open(path?.path())?);
		let (info, mut reader) = decoder.read_info()?;
		let width = info.width as usize;
		let height = info.height as usize;
		let mut buff = vec![0; info.buffer_size()];
		let mut image_atlas: Vec<ocl_core::Char4> = vec![ocl_core::Char4::new(0,0,0,0); tex_size_char4];
		
		


		let color_type = info.color_type;

		let num_bytes = {
			if color_type == png::ColorType::RGBA {
				4
			} else {3} 
		};
		
		reader.next_frame(&mut buff)?;
		
		for x in 0..TEX_SIZE_X {
			for y in 0..TEX_SIZE_Y {
				let src_offset = num_bytes * (x + y * width);
				let dst_offset = (x + y * TEX_SIZE_X) + tex_offset;
				let dst_png_offset = 4 * (x + y * TEX_SIZE_X) + tex_offset_png;

				image_atlas[dst_offset] = ocl_core::Char4::new(
					buff[src_offset] as i8,
					buff[src_offset + 1] as i8,
					buff[src_offset + 2] as i8,
					127
				);
				image_atlas_png[dst_png_offset] = buff[src_offset];
				image_atlas_png[dst_png_offset + 1] = buff[src_offset + 1];
				image_atlas_png[dst_png_offset + 2] = buff[src_offset + 2];
				image_atlas_png[dst_png_offset + 3] = 255;
			}
		}
		tex_offset += width * height;
		tex_offset_png += width * height * 4;

		// tex_offset += tex_buf_host.len();
	}
	dump_image("gpu_img.png", &image_atlas_png, IMAGE_SIZE_X as u32, IMAGE_SIZE_Y as u32);
	// Ok(image_atlas_buf_gpu)
	// println!("AAAA");
	// println!("{:#?}", tex_buf_host);
	// println!("image_atlas_buf_gpu len: {}", image_atlas_buf_gpu.len());
	// println!("src_offset: {}, len: {}", tex_offset, tex_buf_host.len());
	image_atlas_buf_gpu.write(&image_atlas)
		.len(image_atlas.len())
		.enq()
		.unwrap();
	// // println!("Image atlas buf gpu done.");
	// if printable {
	// 	println!("Reading files from {} is finished.", &dir);	
	// }
	Ok(image_atlas_buf_gpu)
} 

// fn create_program(msg: &[u8], s_ocl: &OpenCL) {
	
// }

fn init_opencl() -> OpenCL {
	let platform = ocl::Platform::default();
	let device = ocl::Device::first(platform);
	
	// let device = ocl::Device::list_all(platform).unwrap();
	// println!("{:#?}", device);
	// let device = ocl::Device::list_select(platform, Some(ocl::core::DEVICE_TYPE_GPU), &[0]).unwrap()[0];
	
	
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

pub fn render_hash_2d(msg: &[u8]) -> [u8; 32]
{
	let s_ocl: OpenCL = init_opencl();

	// let rect_list_buf_host: Vec<u8> = vec![0; RECT_LIST_BUF_SIZE];

	let mut rect_list_buf_host: Vec<i32> = vec![0; RECT_LIST_BUF_SIZE];

	let rect_list_buf_gpu = Buffer::<i32>::builder()
		.queue(s_ocl.queue.clone())
		.flags(ocl::flags::MEM_READ_ONLY)
		.dims(RECT_LIST_BUF_SIZE)
		.build()
		.unwrap();

	let image_atlas_buf_gpu = match read_files("./tex/", &s_ocl, true) {
		Ok(atlas) => atlas,
		Err(e) => panic!(e)
	};	
	println!("Atlas is finished.");
	// let image_atlas_buf_gpu = Buffer::<ocl_core::Char4>::builder()
	// 	.queue(s_ocl.queue.clone())
	// 	.flags(ocl::flags::MEM_READ_WRITE)
	// 	.dims(IMAGE_SIZE_BYTE);
	let image_buf_gpu = Buffer::<ocl_core::Char4>::builder()
		.queue(s_ocl.queue.clone())
		.flags(ocl::flags::MEM_READ_WRITE)
		.dims(IMAGE_SIZE_BYTE)
		.build()
		.unwrap();
	
	let program = ocl::Program::builder()
		.devices(&s_ocl.device)
		.src_file("kernel.cl")
		.build(&s_ocl.context)
		.unwrap();
	let mut kern = ocl::Kernel::new("draw_call_rect_list", &program).unwrap()
		.arg_buf_named("rect_list", None::<ocl::Buffer<i32>>)
		.arg_buf_named("image_atlas", None::<ocl::Buffer<ocl_core::Char4>>)
		.arg_buf_named("image_result", None::<ocl::Buffer<ocl_core::Char4>>)
		.arg_scl_named("rect_list_length", Some(RECT_COUNT as u32))
		.arg_scl_named("size_x", Some(IMAGE_SIZE_X as u32))
		.arg_scl_named("tex_size_x", Some(TEX_SIZE_X as u32))
		.arg_scl_named("tex_size_y", Some(TEX_SIZE_Y as u32))
		.queue(s_ocl.queue.clone());

	let scene_seed = sha256_hash(&msg);

	let mut offset = (0..RECT_COUNT).into_iter().cycle();
	let mut rect_list: Vec<Rect> = Vec::new();

	let scale_x = (IMAGE_SIZE_X as f64 / 255.0).floor() as usize;
	let scale_y = (IMAGE_SIZE_Y as f64 / 255.0).floor() as usize;

	for i in (0..RECT_COUNT) {
		let x = scene_seed[offset.next().unwrap() % scene_seed.len()] as usize * scale_x;
		let y = scene_seed[offset.next().unwrap() % scene_seed.len()] as usize * scale_y;
		let w = scene_seed[offset.next().unwrap() % scene_seed.len()] as usize * scale_x;
		let h = scene_seed[offset.next().unwrap() % scene_seed.len()] as usize * scale_y;
		let t = scene_seed[offset.next().unwrap() % scene_seed.len()] as usize % TEX_COUNT;

		rect_list.push({
			Rect {
				x,
				y,
				w,
				h,
				t
			}
		})
	}
	println!("Scene seed finished.");
	for (idx, rect) in rect_list.iter().enumerate() {
		let mut rect_offset = idx * 8 * 4;
		
		rect_list_buf_host[rect_offset] = rect.x as i32;

		rect_offset += 4;

		rect_list_buf_host[rect_offset] = rect.y as i32;

		rect_offset += 4;

		rect_list_buf_host[rect_offset] = rect.w as i32;

		rect_offset += 4;

		rect_list_buf_host[rect_offset] = rect.h as i32;
		
		rect_offset += 4;

		rect_list_buf_host[rect_offset] = rect.t as i32;

		rect_offset += 4;
	}
	println!("Rect list buf calculated.");
	rect_list_buf_gpu.write(&rect_list_buf_host)
			.src_offset(0)
			.len(RECT_LIST_BUF_SIZE)
			.enq()
			.unwrap();
	
	println!("Rect list buf wrote.");
	kern.set_arg_buf_named("rect_list", Some(&rect_list_buf_gpu));
	kern.set_arg_buf_named("image_atlas", Some(&image_atlas_buf_gpu));
	kern.set_arg_buf_named("image_result", Some(&image_buf_gpu));

	let kernel_global_size = IMAGE_SIZE_X * IMAGE_SIZE_Y;
	let kernel_local_size = 32;

	unsafe {
		kern.cmd()
			.gws(kernel_global_size)
			.enq()
			.unwrap();
	}

	println!("Kern finished.");
	let mut image_buf_host: Vec<ocl_core::Char4> = vec![ocl_core::Char4::new(0,0,0,0); TEX_SIZE_CHAR4];
	let img_buf_len = image_buf_host.len();
	
	let old_image_buf_host = image_buf_host.clone();

	println!("image_buf_gpu len: {}, image_buf_host len: {}", image_buf_gpu.len(), image_buf_host.len());
	// println!("{:#?}", image_buf_gpu);
	image_buf_gpu.read(&mut image_buf_host)
		.enq()
		.unwrap();
	println!("Image buf gpu finished.");
	let mut image_vec_host: Vec<u8> = vec![0; IMAGE_SIZE_BYTE];

	for (idx, c4) in image_buf_host.iter().enumerate() {
		image_vec_host[idx] = *c4.get(0).unwrap() as u8;
		image_vec_host[idx + 1] = *c4.get(1).unwrap() as u8;
		image_vec_host[idx + 2] = *c4.get(2).unwrap() as u8;
		image_vec_host[idx + 3] = *c4.get(3).unwrap() as u8;
	}

	// for (old_i, i) in image_buf_host.iter().zip(&old_image_buf_host) {
	// 	if (
	// 		(*old_i.get(0).unwrap() != *i.get(0).unwrap()) &&
	// 		(*old_i.get(1).unwrap() != *i.get(1).unwrap()) &&
	// 		(*old_i.get(2).unwrap() != *i.get(2).unwrap()) &&
	// 		(*old_i.get(3).unwrap() != *i.get(3).unwrap()) 
	// 	)
	// 	 {
	// 		println!("<>_<>");
	// 	}
	// }
	let hash = sha256_hash(&image_vec_host);
	// println!("{:#?}", &hash);
	hash
	// for i in image_vec_host {
		// if i != 0 {
			// println!("{}", i);
		// }
	// }
	// println!("{:#?}", image_vec_host);
	// kern.set_arg_buf_named("rect_list_length", Some(rect_list.len() as u64));
	// kern.set_arg_buf_named("rect_list", Some(&rect_list_buf_gpu));
	// et kern = ocl::Kernel::new("draw_call_rect_list", &program).unwrap()
	// 	.arg_buf_named("rect_list", None::<ocl::Buffer<u8>>)
	// 	.arg_buf_named("image_atlas", None::<ocl::Buffer<u8>>)
	// 	.arg_buf_named("image_result", None::<ocl::Buffer<u8>>)
	// 	.arg_scl_named("rect_list_length", Some(RECT_COUNT as u8))
	// 	.arg_scl_named("size_x", Some(IMAGE_SIZE_X as u8))
	// 	.arg_scl_named("tex_size_x", Some(TEX_SIZE_X as u8))
	// 	.arg_scl_named("tex_size_y", Some(TEX_SIZE_Y as u8))
	// 	.queue(s_ocl.queue.clone());
}
