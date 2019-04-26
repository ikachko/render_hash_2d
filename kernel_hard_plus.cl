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
  
  int i;
  unsigned int r = 128 << 8;
  unsigned int g = 128 << 8;
  unsigned int b = 128 << 8;
  // uchar4 rgba;
  int rgba_x = 0;
  int rgba_y = 0;
  int rgba_z = 0;
  int rgba_w = 0;
  
  for(i = 0;i < rect_list_length;i++){
    int offset = i * 5;
    int rect_x = rect_list[offset  ];
    int rect_y = rect_list[offset+1];
    int rect_w = rect_list[offset+2];
    int rect_h = rect_list[offset+3];
    
    bool fit_x = x >= rect_x && x < rect_x + rect_w;
    bool fit_y = y >= rect_y && y < rect_y + rect_h;
    if (fit_x && fit_y) {
      int rect_tex_idx = rect_list[offset + 4];

      int tex_offset_x = (x - rect_x) % tex_size_x;
      int tex_offset_y = (y - rect_y) % tex_size_y;
      
      int tex_offset = tex_offset_x + tex_offset_y*tex_size_x;
      tex_offset += rect_tex_idx * tex_size_x * tex_size_y;
      tex_offset *= 4;
      
      rgba_x = image_atlas[tex_offset];
      rgba_y = image_atlas[tex_offset + 1];
      rgba_z = image_atlas[tex_offset + 2];
      rgba_w = image_atlas[tex_offset + 3];
      r = (r + ((int)(rgba_x) << 8)) >> 1;
      g = (g + ((int)(rgba_y) << 8)) >> 1;
      b = (b + ((int)(rgba_z) << 8)) >> 1;
    }
  }
  
  r >>= 8;
  g >>= 8;
  b >>= 8;
  
  int supp = (rgba_x << 16) + (rgba_y << 8) + rgba_z;
  int construct = (r << 16) + (g << 8) + b;
  
  if (y == 500) {
    construct += supp;
  }
  if (y == 501) {
    construct *= supp;
  }
  if (y == 502) {
    construct /= supp;
  }
  if (y == 503) {
    construct %= supp;
  }
  if (y == 504) {
    construct |= supp;
  }
  if (y == 505) {
    construct &= supp;
  }
  if (y == 506) {
    construct ^= supp;
  }
  if (y == 507) {
    construct = ~construct;
  }
  if (y == 508) {
    construct = (construct > supp) ? 0xFFFFFFFF : construct;
  }
  if (y == 509) {
    construct = (construct < supp) ? 0xFFFFFFFF : construct;
  }
  if (y == 510) {
    construct = (construct >= supp) ? 0xFFFFFFFF : construct;
  }
  if (y == 511) {
    construct = (construct <= supp) ? 0xFFFFFFFF : construct;
  }
  if (y == 512) {
    construct = (construct == supp) ? 0xFFFFFFFF : construct;
  }
  if (y == 513) {
    construct = abs_diff(construct, supp);
  }
  if (y == 514) {
    construct = popcount(construct);
  }
  if (y == 515) {
    construct = hadd(construct, supp);
  }
  // if (y == 516) {
  //   construct = hradd(construct, supp);
  // }
  if (y == 517) {
    construct = max(construct, supp);
  }
  if (y == 518) {
    construct = min(construct, supp);
  }
  if (y == 519) {
    construct = max(construct, supp);
  }
  if (y == 520) {
    construct <<= supp;
  }
  if (y == 521) {
    construct >>= supp;
  }
  if (y == 522) {
    construct = clz(construct);
  }
  
  
  r = construct >> 16;
  g = construct >> 8;
  b = construct;
  
  image_result[id] = r;
  image_result[id + 1] = g;
  image_result[id + 2] = b;
  image_result[id + 3] = 255;
}
