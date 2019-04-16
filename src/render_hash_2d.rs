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

const IMAGE_SIZE_X: usize = 1920;
const IMAGE_SIZE_Y: usize = 1080;
const IMAGE_SIZE_BYTE: usize = IMAGE_SIZE_X * IMAGE_SIZE_Y * 4;

const RECT_LIST_BUF_SIZE: usize = 5 * 6;
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

pub struct RenderCL {
	platform: ocl::Platform,
	device: ocl::Device,
	context: ocl::Context,
	queue: ocl::Queue,

	rect_list_buf_gpu: ocl::Buffer<i32>,
	debug_buf_gpu: ocl::Buffer<i32>,
	image_buf_gpu: ocl::Buffer<ocl_core::Char4>,

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
	pub fn new() -> RenderCL {
		let platform = ocl::Platform::default();
        let device = ocl::Device::first(platform);
        let context = ocl::Context::builder().platform(platform).devices(device.clone()).build().unwrap();
        let queue = ocl::Queue::new(&context, device, None).unwrap();

		let rect_list_buf_gpu = ocl::builders::BufferBuilder::<i32>::new()
			.flags(ocl::flags::MEM_READ_WRITE)
			.queue(queue.clone())
			.dims(RECT_LIST_BUF_SIZE)
			.build()
			.unwrap();
	
		let debug_buf_gpu = Buffer::<i32>::builder()
			.queue(queue.clone())
			.flags(ocl::flags::MEM_READ_WRITE)
			.dims(IMAGE_SIZE_X * IMAGE_SIZE_Y)
			.build()
			.unwrap();
		
		let image_buf_gpu = Buffer::<ocl_core::Char4>::builder()
			.queue(queue.clone())
			.flags(ocl::flags::MEM_READ_WRITE)
			.dims(TEX_SIZE_X * TEX_SIZE_Y)
			.build()
			.unwrap();
		
		RenderCL {
			platform,
			device,
			context,
			queue,
			rect_list_buf_gpu,
			debug_buf_gpu,
			image_buf_gpu
		}
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

fn read_files(dir: &str, s_ocl: &RenderCL, printable: bool) -> Result<ocl::Buffer<ocl_core::Char4>, FileReadError> {
	let paths = fs::read_dir(dir)?;	
	let tex_size_char4 = TEX_SIZE_X * TEX_SIZE_Y * TEX_COUNT;	
	let mut tex_offset = 0;
	let mut tex_offset_png = 0;

	let tex_size_bytes = TEX_SIZE_X * TEX_SIZE_Y * 4 * TEX_COUNT;
	let mut image_atlas_buf_gpu = ocl::Buffer::<ocl_core::Char4>::builder()
		.queue(s_ocl.queue.clone())
		.flags(ocl::flags::MEM_WRITE_ONLY)
		.dims(tex_size_bytes)
		.build()
		.unwrap();
	let image_atlas: Vec<ocl_core::Char4> = vec![ocl_core::Char4::new(0,0,0,0); tex_size_char4];
	let mut image_atlas_png = vec![0; tex_size_bytes];


	for path in paths {
		let decoder = png::Decoder::new(File::open(&path?.path())?);
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
		tex_offset_png += width * height * num_bytes;
	}
	image_atlas_buf_gpu.write(&image_atlas)
		.len(image_atlas.len())
		.enq()
		.unwrap();
	Ok(image_atlas_buf_gpu)
}

pub fn render_hash_2d(msg: &[u8]) -> [u8; 32]
{
	println!("================\nRENDER_HASH_2D");
	let mut rect_list_buf_host: Vec<i32> = vec![0; RECT_LIST_BUF_SIZE];
	let mut image_vec_host: Vec<u8> = vec![0; IMAGE_SIZE_BYTE];
	let mut debug_arr = vec![0; IMAGE_SIZE_X * IMAGE_SIZE_Y];
	let mut image_buf_host: Vec<ocl_core::Char4> = vec![ocl_core::Char4::new(0,0,0,0); TEX_SIZE_X * TEX_SIZE_Y];
	let scene_seed = sha256_hash(&msg);
	let mut rect_list: Vec<Rect> = Vec::new();

	let scale_x = (IMAGE_SIZE_X as f64 / 255.0).floor() as usize;
	let scale_y = (IMAGE_SIZE_Y as f64 / 255.0).floor() as usize;

	let render_cl = RenderCL::new();

	let image_atlas_buf_gpu = match read_files("./tex/", &render_cl, true) {
		Ok(atlas) => atlas,
		Err(e) => panic!(e)
	};	
	println!("Atlas is finished.");
	
	let program = ocl::Program::builder()
		.devices(&render_cl.device)
		.src_file("kernel.cl")
		.build(&render_cl.context)
		.unwrap();
	let mut kern = ocl::Kernel::new("draw_call_rect_list", &program).unwrap()
		.arg_buf_named("debug_arr", None::<ocl::Buffer<i32>>)
		.arg_buf_named("rect_list", None::<ocl::Buffer<i32>>)
		.arg_buf_named("image_atlas", None::<ocl::Buffer<ocl_core::Char4>>)
		.arg_buf_named("image_result", None::<ocl::Buffer<ocl_core::Char4>>)
		.arg_scl_named("rect_list_length", Some(RECT_COUNT as u32))
		.arg_scl_named("size_x", Some(IMAGE_SIZE_X as u32))
		.arg_scl_named("tex_size_x", Some(TEX_SIZE_X as u32))
		.arg_scl_named("tex_size_y", Some(TEX_SIZE_Y as u32))
		.queue(render_cl.queue.clone());
	
	let mut seed_iter = scene_seed.into_iter().cycle();
	println!("Scene seed: {:?}", scene_seed);
	println!("Scale x: {}, Scale y: {}", scale_x, scale_y);
	for i in (0..RECT_COUNT) {
		let x = *seed_iter.next().unwrap() as usize * scale_x;
		let y = *seed_iter.next().unwrap() as usize * scale_y;
		let w = *seed_iter.next().unwrap() as usize * scale_x;
		let h = *seed_iter.next().unwrap() as usize * scale_y;
		let t = *seed_iter.next().unwrap() as usize % TEX_COUNT;
		println!("[{}] - x: {}, y: {}, w: {}, h: {}, t: {}",
			i,
			x,
			y,
			w,
			h,
			t);
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
		let rect_offset = idx * 5;
		
		rect_list_buf_host[rect_offset] = rect.x as i32;
		rect_list_buf_host[rect_offset + 1] = rect.y as i32;
		rect_list_buf_host[rect_offset + 2] = rect.w as i32;
		rect_list_buf_host[rect_offset + 3] = rect.h as i32;
		rect_list_buf_host[rect_offset + 4] = rect.t as i32;
	}
	println!("Rect list buf calculated.");
	render_cl.rect_list_buf_gpu.write(&rect_list_buf_host)
			.src_offset(0)
			.len(RECT_LIST_BUF_SIZE)
			.enq()
			.unwrap();	
	println!("Rect list buf wrote.");

	// Setting kernel arguments
	kern.set_arg_buf_named("debug_arr", Some(&render_cl.debug_buf_gpu)).unwrap();
	kern.set_arg_buf_named("rect_list", Some(&render_cl.rect_list_buf_gpu)).unwrap();
	kern.set_arg_buf_named("image_atlas", Some(&image_atlas_buf_gpu)).unwrap();
	kern.set_arg_buf_named("image_result", Some(&render_cl.image_buf_gpu)).unwrap();

	// Setting kernel parameters
	let kernel_global_size = IMAGE_SIZE_X * IMAGE_SIZE_Y;
	let kernel_local_size = 32;

	// Launching kernel
	unsafe {
		kern.cmd()
			.lws(kernel_local_size)
			.gws(kernel_global_size)
			.enq()
			.unwrap();
	}
	println!("Kern finished.");
	println!("image_buf_gpu len: {}, image_buf_host len: {}", render_cl.image_buf_gpu.len(), image_buf_host.len());
	
	// Read buffers after GPU computing
	render_cl.image_buf_gpu.read(&mut image_buf_host)
		.enq()
		.unwrap();
	println!("Image buf gpu finished.");
	render_cl.debug_buf_gpu.read(&mut debug_arr)
		.enq()
		.unwrap();

	println!("Printing char4");
	dump_image("image_res.png", &image_vec_host, 1920, 1080);
	println!("image_buf_host len: {}", image_buf_host.len());
	println!("image_vec_host len: {}", image_vec_host.len());

	// Copying from char4 to u8 array
	for (idx, c4) in image_buf_host.iter().enumerate() {
		let offset = idx * 4;
		image_vec_host[offset] = *c4.get(0).unwrap() as u8;
		image_vec_host[offset + 1] = *c4.get(1).unwrap() as u8;
		image_vec_host[offset + 2] = *c4.get(2).unwrap() as u8;
		image_vec_host[offset + 3] = *c4.get(3).unwrap() as u8;
	}
	
	let hash = sha256_hash(&image_vec_host);
	hash
}
