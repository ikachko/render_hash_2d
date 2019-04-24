__kernel void draw_call_rect_list(
	__global int *debug_arr,
	__global int *rect_list,
	__global uchar *image_atlas,
	__global uchar *image_result,
	const unsigned int rect_list_length,
	const unsigned int size_x,
	const unsigned int tex_size_x,
	const unsigned int tex_size_y
	)
{
	// per pixel shader
	int global_id = get_global_id(0);
	int id = global_id * 4;
	int x = global_id % size_x;
	int y = global_id / size_x;
	
	debug_arr[global_id] = global_id;

	int i;
	int rect_id = -1;
	for(i = 0;i < rect_list_length;i++){
		int offset = i * 5;
		int rect_x = rect_list[offset  ];
		int rect_y = rect_list[offset+1];
		int rect_w = rect_list[offset+2];
		int rect_h = rect_list[offset+3];
		
		bool fit_x = x >= rect_x && x < rect_x + rect_w;
		bool fit_y = y >= rect_y && y < rect_y + rect_h;
		rect_id = (fit_x && fit_y) ? i : rect_id;
	}
	
	if (rect_id == -1) {
		image_result[id] = 128;
		image_result[id + 1] = 128;
		image_result[id + 2] = 128;
		image_result[id + 3] = 255;

		debug_arr[global_id] = -1;
		return;
	}
	int rect_offset = 5 * rect_id;
	int rect_x = rect_list[rect_offset  ];
	int rect_y = rect_list[rect_offset+1];
	int rect_tex_idx = rect_list[rect_offset+4];
	
	int tex_offset_x = (x - rect_x) % tex_size_x;
	int tex_offset_y = (y - rect_y) % tex_size_y;
	int tex_offset = tex_offset_x + tex_offset_y*tex_size_x;
	tex_offset += rect_tex_idx * tex_size_x * tex_size_y;
	tex_offset *= 4;

	image_result[id] = image_atlas[tex_offset];
	image_result[id + 1] = image_atlas[tex_offset + 1];
	image_result[id + 2] = image_atlas[tex_offset + 2];
	image_result[id + 3] = image_atlas[tex_offset + 3];

	debug_arr[global_id] = 1;
}
