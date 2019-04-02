extern crate ocl;
extern crate image;
extern crate png;


use ocl::{flags, Platform, Device, Context, Queue, Program, Buffer, Kernel};
use std::io;

use std::fs::{self, DirEntry, File};
use std::path::Path;

use byteorder::{ByteOrder, LittleEndian};
use sha2::{Sha256, Sha512, Digest};

const IMAGE_SIZE_X: usize = 1920 * 2;
const IMAGE_SIZE_Y: usize = 1080 * 2;
const IMAGE_SIZE_BYTE: usize = IMAGE_SIZE_X * IMAGE_SIZE_Y * 4;

const TEX_SIZE_X: usize = 1920;
const TEX_SIZE_Y: usize = 1080;

const RECT_COUNT: usize = 6;

// const SCALE_X: usize = (IMAGE_SIZE_X as f64 / 255.0).floor() as usize;
// const SCALE_Y: usize = (IMAGE_SIZE_Y as f64 / 255.0).floor() as usize;

const RECT_LIST_BUF_SIZE: usize = 1000 * 8 * 4;

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

struct RectList {
	x: usize,
	y: usize,
	w: usize,
	h: usize,
	t: usize
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
	let tex_size_bytes = TEX_SIZE_X * TEX_SIZE_Y * 4 * 4;
	let mut image_atlas = vec![0; tex_size_bytes];
	
	let mut tex_offset = 0;
	
	for path in paths {
		let decoder = png::Decoder::new(File::open(path?.path())?);
		let (info, mut reader) = decoder.read_info()?;
		let width = info.width as usize;
		let height = info.height as usize;
		let mut buff = vec![0; info.buffer_size()];

		reader.next_frame(&mut buff)?;
		
		for x in 0..TEX_SIZE_X {
			for y in 0..TEX_SIZE_Y {
				let src_offset = 3 * (x + y * width);
				let dst_offset = 4 * (x + y * TEX_SIZE_X) + tex_offset;
				if (x >= width || y >= height) {
					image_atlas[dst_offset] = 0;
					image_atlas[dst_offset + 1] = 0;
					image_atlas[dst_offset + 2] = 0;
					image_atlas[dst_offset + 3] = 255;
				} else {
					image_atlas[dst_offset] = buff[src_offset];
					image_atlas[dst_offset + 1] = buff[src_offset + 1];
					image_atlas[dst_offset + 2] = buff[src_offset + 2];
					image_atlas[dst_offset + 3] = 255;
				}
			}
		}
		tex_offset += TEX_SIZE_X * TEX_SIZE_Y * 4;
	}
	Ok(image_atlas)
} 

fn sha256_hash(data: &[u8]) -> [u8; 32] {
	let mut ret = [0; 32];
	let mut sha2 = Sha256::new();
	sha2.input(data);
	ret.copy_from_slice(sha2.result().as_slice());
	ret
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

	println!("{}", image_atlas.len()); 

}

fn main() {
	render();
}
