extern crate ocl;
extern crate image;
extern crate png;


use ocl::{flags, Platform, Device, Context, Queue, Program, Buffer, Kernel};
use std::io;

use std::fs::{self, DirEntry, File};
use std::path::Path;
use std::fmt;
use std::io::BufWriter;

use png::HasParameters;
// use image;
// use byteorder::{ByteOrder, LittleEndian};
use sha2::{Sha256, Sha512, Digest};

const IMAGE_SIZE_X: usize = 1920 * 2;
const IMAGE_SIZE_Y: usize = 1080 * 2;
const IMAGE_SIZE_BYTE: usize = IMAGE_SIZE_X * IMAGE_SIZE_Y * 4;

const TEX_SIZE_X: usize = 1920;
const TEX_SIZE_Y: usize = 1080;

const RECT_COUNT: usize = 6;

const RECT_LIST_BUF_SIZE: usize = 1000 * 8 * 4;

const TEX_COUNT: usize = 4;

static KERNEL_SRC: &'static str = r#"
__kernel void draw_call_rect_list(
	__global int *rect_list,
	__global char4 *image_atlas,
	__global char4 *image_result,
	const unsigned int rect_list_length,
	const unsigned int size_x,
	const unsigned int tex_size_x,
	const unsigned int tex_size_y
	)
{
	// per pixel shader
	int id = get_global_id(0);
	int x = id % size_x;
	int y = id / size_x;
	
	int i;
	int rect_id = -1;
	for(i = 0;i < rect_list_length;i++){
		int offset = i*8;
		int rect_x = rect_list[offset  ];
		int rect_y = rect_list[offset+1];
		int rect_w = rect_list[offset+2];
		int rect_h = rect_list[offset+3];
		
		bool fit_x = x >= rect_x && x < rect_x + rect_w;
		bool fit_y = y >= rect_y && y < rect_y + rect_h;
		rect_id = (fit_x && fit_y) ? i : rect_id;
	}
	
	if (rect_id == -1) {
		image_result[id].x = (unsigned char)128;
		image_result[id].y = (unsigned char)128;
		image_result[id].z = (unsigned char)128;
		image_result[id].w = (unsigned char)255;
		return;
	}
	int rect_offset = 8*rect_id;
	int rect_x = rect_list[rect_offset  ];
	int rect_y = rect_list[rect_offset+1];
	int rect_tex_idx = rect_list[rect_offset+4];
	
	int tex_offset_x = (x - rect_x) % tex_size_x;
	int tex_offset_y = (y - rect_y) % tex_size_y;
	int tex_offset = tex_offset_x + tex_offset_y*tex_size_x;
	tex_offset += rect_tex_idx * tex_size_x * tex_size_y;

	image_result[id] = image_atlas[tex_offset];
}
"#;

enum FileReadError {
	NotFound(),
	IOError(io::Error),
	DecodeError(png::DecodingError),
	// RequestError(req::Error),
}

struct Rect {
	x: usize,
	y: usize,
	w: usize,
	h: usize,
	t: usize
}

impl fmt::Display for Rect {
	fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
		write!(f, "x: {}, y: {}, w: {}, h: {}, t: {}",
					self.x, self.y, self.w, self.h, self.t)
	}
}

struct OpenCL {
	platform: ocl::Platform,
	device: ocl::Device,
	context: ocl::Context,
	queue: ocl::Queue
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

fn read_files(dir: &str, s_open_cl: &OpenCL) -> Result<Vec<u8>, FileReadError> {
	let paths = fs::read_dir(dir)?;
	let tex_size_bytes = TEX_SIZE_X * TEX_SIZE_Y * 4 * TEX_COUNT;
	let mut image_atlas = vec![0; tex_size_bytes];
	
	let mut tex_offset = 0;
	
	for path in paths {
		let decoder = png::Decoder::new(File::open(path?.path())?);
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
				let dst_offset = 4 * (x + y * TEX_SIZE_X) + tex_offset;

				image_atlas[dst_offset] = buff[src_offset];
				image_atlas[dst_offset + 1] = buff[src_offset + 1];
				image_atlas[dst_offset + 2] = buff[src_offset + 2];
				image_atlas[dst_offset + 3] = 255;
			}
		}
		tex_offset += width * height * num_bytes;
	}
	println!("Reading files from {} is finished.", &dir);
	Ok(image_atlas)
} 

fn sha256_hash(data: &[u8]) -> [u8; 32] {
	let mut ret = [0; 32];
	let mut sha2 = Sha256::new();
	sha2.input(data);
	ret.copy_from_slice(sha2.result().as_slice());
	ret
}

fn render_image(rect_list: &[Rect], image_atlas: &Vec<u8>) -> Vec<u8> {
	let mut image_result: Vec<u8> = vec![0; IMAGE_SIZE_BYTE];

	let mut min_one = 0;
	let mut not_min_one = 0;

	for id in 0..(IMAGE_SIZE_X * IMAGE_SIZE_Y) {
		let x = id % IMAGE_SIZE_X;
		let y = (id as f64 / IMAGE_SIZE_X as f64).floor() as usize;
	
		let mut rect_id: isize = -1;

		let mut rect_x = 0;
		let mut rect_y = 0;
		let mut rect_w = 0;
		let mut rect_h = 0;
		let mut rect_tex_idx = 0;

		for i in 0..rect_list.len() {
			rect_x = rect_list[i].x;
			rect_y = rect_list[i].y;
			rect_w = rect_list[i].w;
			rect_h = rect_list[i].h;

			let fit_x = x >= rect_x && (x < rect_x + rect_w);
			let fit_y = y >= rect_y && (y < rect_y + rect_h);

			rect_id = if (fit_x && fit_y) {i as isize} else {rect_id};
			if rect_id != -1 {break};
		}

		if (rect_id == -1) {
			image_result[4 * id] = 128;
			image_result[4 * id + 1] = 128;
			image_result[4 * id + 2] = 128;
			image_result[4 * id + 3] = 255;
			min_one += 1;
			continue;
		}

		let rect_id = rect_id as usize;

		let rect_x = rect_list[rect_id].x;
		let rect_y = rect_list[rect_id].y;
		let rect_tex_idx = rect_list[rect_id].t;

		let tex_offset_x = (x - rect_x) % TEX_SIZE_X;
		let tex_offset_y = (y - rect_y) % TEX_SIZE_Y;
		let tex_offset = tex_offset_x + tex_offset_y * TEX_SIZE_X;
		let tex_offset = tex_offset + rect_tex_idx * TEX_SIZE_X * TEX_SIZE_Y;
		
		image_result[4*id+0] = image_atlas[4*tex_offset+0];
		image_result[4*id+1] = image_atlas[4*tex_offset+1];
		image_result[4*id+2] = image_atlas[4*tex_offset+2];
		image_result[4*id+3] = image_atlas[4*tex_offset+3];
		not_min_one += 1;
	}
	println!("minus one: {}, not minus one: {}", min_one, not_min_one);
	dump_image("./render_result.png", &image_result, IMAGE_SIZE_X as u32, IMAGE_SIZE_Y as u32);
	image_result
}

fn hash(msg: &[u8]) -> Vec<Rect> {
	let scene_seed = sha256_hash(msg);

	let mut offset = 0;
	let mut rect_list: Vec<Rect> = Vec::new();

	let scale_x = (IMAGE_SIZE_X as f64 / 255.0).floor() as usize;
	let scale_y = (IMAGE_SIZE_Y as f64 / 255.0).floor() as usize;

	for i in 0..RECT_COUNT {
		let x = scene_seed[offset % scene_seed.len()] as usize * scale_x;
		offset += 1;
		let y = scene_seed[offset % scene_seed.len()] as usize * scale_y;
		offset += 1;
		let w = scene_seed[offset % scene_seed.len()] as usize * scale_x;
		offset += 1;
		let h = scene_seed[offset % scene_seed.len()] as usize * scale_y;
		offset += 1;
		let t = scene_seed[offset % scene_seed.len()] as usize % TEX_COUNT;

		rect_list.push(Rect {
			x,
			y,
			w,
			h,
			t
		})
	}
	// for (i, rect) in rect_list.iter().enumerate() {
		// println!("{}: {}", i, rect);
	// }
	println!("Rectangles from hash are finished.");
	rect_list
	// let image = render_image(&rect_list, &image_atlas);
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

fn dump_image(file_name: &str, image: &Vec<u8>, width: u32, height: u32) {
	let path = Path::new(file_name);
	let file = File::create(path).unwrap();

	let ref mut w = BufWriter::new(file);

	let mut encoder = png::Encoder::new(w, width, height);
	encoder.set(png::ColorType::RGBA).set(png::BitDepth::Eight);

	let mut writer = encoder.write_header().unwrap();

	// let data: [u8; image.len()] = 
	writer.write_image_data(&image).unwrap();
	println!("Image {} is dumped.", &file_name);
}

fn render() {
	let s_open_cl: OpenCL = init_opencl();


	let tex_size_bytes = TEX_SIZE_X * TEX_SIZE_Y * 4 * 4;

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

	let image_atlas = match read_files("./tex/", &s_open_cl) {
		Ok(texture) => texture,
		Err(e) => panic!(e)
	};

	dump_image("./atlas_image.png", &image_atlas, 1920, 1080 * TEX_COUNT as u32);
	// for pixel in image_atlas {
		// if pixel != 0 && pixel != 255 {
			// println!("{}", pixel);
		// }
	// }
	// println!("{:#?}", image_atlas);

	let msg: [u8; 32] = [5; 32];

	let rect_list = hash(&msg);

	let image = render_image(&rect_list, &image_atlas);

	// println!("{:#?}", image.len());

	// dump_image("./render_result.png", &image, 1920, 1080);
	// println!("{}", image_atlas.len()); 

}

fn main() {
	render();
	
	// hash(&msg);
}
