extern crate ocl;
extern crate ocl_core;

use ocl::{Buffer};
// use ocl_core;

use std::fs::{self, File};
use std::io::{self, Write};
use sha2::{Sha256, Digest};
use std::io::prelude::*; 
use std::path::Path;
use std::io::BufWriter;
use png::HasParameters;

const TEX_SIZE_X: usize = 1920;
const TEX_SIZE_Y: usize = 1080;

const RECT_COUNT: usize = 6;
const TEX_COUNT: usize = 4;

const IMAGE_SIZE_X: usize = 1920;
const IMAGE_SIZE_Y: usize = 1080;
const IMAGE_SIZE: usize = IMAGE_SIZE_X * IMAGE_SIZE_Y;

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
	image_buf_gpu: ocl::Buffer<u8>,

}

struct Rect {
	x: usize,
	y: usize,
	w: usize,
	h: usize,
	t: usize
}

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

		let rect_list_buf_gpu = ocl::Buffer::<i32>::new(
			&queue,
			None,
			RECT_LIST_BUF_SIZE,
			None
		).unwrap();

		let debug_buf_gpu = Buffer::<i32>::new(
			&queue,
			None,
			IMAGE_SIZE,
			None
		).unwrap();

		let image_buf_gpu = Buffer::<u8>::new(
			&queue,
			None,
			IMAGE_SIZE_BYTE,
			None
		).unwrap();

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

fn read_files(dir: &str, s_ocl: &RenderCL, printable: bool) -> Result<ocl::Buffer<u8>, FileReadError> {
	let paths = fs::read_dir(dir)?;	
	let tex_size_char4 = TEX_SIZE_X * TEX_SIZE_Y * TEX_COUNT;	
	let mut tex_offset = 0;
	let mut tex_offset_png = 0;

	let tex_size_bytes = TEX_SIZE_X * TEX_SIZE_Y * 4 * TEX_COUNT;
	
	let mut image_atlas_buf_gpu = Buffer::<u8>::new(
		&s_ocl.queue,
		None,
		tex_size_bytes,
		None
	).unwrap();
	let mut image_atlas: Vec<u8> = vec![0; tex_size_bytes];
	let mut image_atlas_png = vec![0; tex_size_bytes];


	for path in paths {
		let decoder = png::Decoder::new(File::open(&path?.path())?);
		let (info, mut reader) = decoder.read_info()?;
		let width = info.width as usize;
		let height = info.height as usize;
		let mut buff = vec![0; info.buffer_size()];		
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

				image_atlas_png[dst_png_offset] = buff[src_offset];
				image_atlas_png[dst_png_offset + 1] = buff[src_offset + 1];
				image_atlas_png[dst_png_offset + 2] = buff[src_offset + 2];
				image_atlas_png[dst_png_offset + 3] = 255;
			}
		}
		tex_offset += width * height;
		tex_offset_png += width * height * num_bytes;
	}
	// println!("{:#?}", image_atlas_png);
	dump_image("GPU_atlas.png", &image_atlas_png, 1920, 1080 * 4);
	image_atlas_buf_gpu.write(&image_atlas_png)
		.len(image_atlas_png.len())
		.enq()
		.unwrap();
	Ok(image_atlas_buf_gpu)
}

fn generate_rectangles(msg: &[u8]) -> Vec<i32>{
	let scene_seed = sha256_hash(&msg);
	let mut rect_list: Vec<Rect> = Vec::new();
	let mut rect_list_buf_host: Vec<i32> = vec![0; RECT_LIST_BUF_SIZE];
	let scale_x = (IMAGE_SIZE_X as f64 / 255.0).floor() as usize;
	let scale_y = (IMAGE_SIZE_Y as f64 / 255.0).floor() as usize;
	let mut seed_iter = scene_seed.into_iter().cycle();

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

	for (idx, rect) in rect_list.iter().enumerate() {
		let rect_offset = idx * 5;
		
		rect_list_buf_host[rect_offset] = rect.x as i32;
		rect_list_buf_host[rect_offset + 1] = rect.y as i32;
		rect_list_buf_host[rect_offset + 2] = rect.w as i32;
		rect_list_buf_host[rect_offset + 3] = rect.h as i32;
		rect_list_buf_host[rect_offset + 4] = rect.t as i32;
	}
	println!("Rect list buf calculated.");

	rect_list_buf_host
}

pub fn render_hash_2d(msg: &[u8]) -> [u8; 32]
{
	println!("================\nRENDER_HASH_2D");
	let kernel_global_size = IMAGE_SIZE_X * IMAGE_SIZE_Y;
	let kernel_local_size = 32;
	let mut debug_arr = vec![0; IMAGE_SIZE_X * IMAGE_SIZE_Y];
	let mut image_buf_host: Vec<u8> = vec![0; IMAGE_SIZE_BYTE];

	let mut render_cl = RenderCL::new();

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

	let rect_list_buf_host = generate_rectangles(&msg);
	println!("Scene seed finished.");

	render_cl.rect_list_buf_gpu.write(&rect_list_buf_host)
			.enq()
			.unwrap();

	let kern = ocl::Kernel::new("draw_call_rect_list", &program).unwrap()
		.lws(kernel_local_size)
		.gws(kernel_global_size)
		.arg_buf_named("debug_arr",  Some(&render_cl.debug_buf_gpu))
		.arg_buf_named("rect_list", Some(&render_cl.rect_list_buf_gpu))
		.arg_buf_named("image_atlas", Some(&image_atlas_buf_gpu))
		.arg_buf_named("image_result", Some(&render_cl.image_buf_gpu))
		.arg_scl_named("rect_list_length", Some(RECT_COUNT as u32))
		.arg_scl_named("size_x", Some(IMAGE_SIZE_X as u32))
		.arg_scl_named("tex_size_x", Some(TEX_SIZE_X as u32))
		.arg_scl_named("tex_size_y", Some(TEX_SIZE_Y as u32))
		.queue(render_cl.queue);
	

	// Launching kernel
	unsafe {
		// await!(kern.enq().unwrap());
		kern.enq()
			.unwrap();
	}
	println!("Kern finished.");
	println!("image_buf_gpu len: {}, image_buf_host len: {}", render_cl.image_buf_gpu.len(), image_buf_host.len());
	
	// Read buffers after GPU computing
	render_cl.image_buf_gpu.read(&mut image_buf_host)
		.enq()
		.unwrap();
	// println!("{:#?}", image_buf_host);
	println!("Image buf gpu finished.");
	render_cl.debug_buf_gpu.read(&mut debug_arr)
		.enq()
		.unwrap();

	let strings: Vec<String> = debug_arr.iter().map(|n| n.to_string()).collect();
	let mut file = File::create("./debug.log").expect("Unable to create file");
	
	writeln!(file, "{}", strings.join("\n")).unwrap();
	println!("image_buf_host len : {}", image_buf_host.len());

	dump_image("image_res.png", &image_buf_host, IMAGE_SIZE_X as u32, IMAGE_SIZE_Y as u32);
	println!("image_buf_host len: {}", image_buf_host.len());

	let hash = sha256_hash(&image_buf_host);
	hash
}