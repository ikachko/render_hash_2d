__kernel void draw_call_rect_list(
	__global int *rect_list,
	__global uchar4 *image_atlas,
	__global uchar4 *image_result,
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
  unsigned int r = 128 << 8;
  unsigned int g = 128 << 8;
  unsigned int b = 128 << 8;
  
	for(i = 0;i < rect_list_length;i++){
		int offset = i*8;
		int rect_x = rect_list[offset  ];
		int rect_y = rect_list[offset+1];
		int rect_w = rect_list[offset+2];
		int rect_h = rect_list[offset+3];
    
		bool fit_x = x >= rect_x && x < rect_x + rect_w;
		bool fit_y = y >= rect_y && y < rect_y + rect_h;
    if (fit_x && fit_y) {
      int rect_tex_idx = rect_list[offset+4];
      int tex_offset_x = (x - rect_x) % tex_size_x;
      int tex_offset_y = (y - rect_y) % tex_size_y;
      int tex_offset = tex_offset_x + tex_offset_y*tex_size_x;
      tex_offset += rect_tex_idx * tex_size_x * tex_size_y;
      uchar4 rgba = image_atlas[tex_offset];
      r = (r + ((int)(rgba.x) << 8)) >> 1;
      g = (g + ((int)(rgba.y) << 8)) >> 1;
      b = (b + ((int)(rgba.z) << 8)) >> 1;
    }
	}
  
  image_result[id].x = r >> 8;
  image_result[id].y = g >> 8;
  image_result[id].z = b >> 8;
  image_result[id].w = 255;
}
