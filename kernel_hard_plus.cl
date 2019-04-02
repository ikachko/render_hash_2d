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
  uchar4 rgba;
  rgba.x = 0;
  rgba.y = 0;
  rgba.z = 0;
  rgba.w = 0;
  
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
      rgba = image_atlas[tex_offset];
      r = (r + ((int)(rgba.x) << 8)) >> 1;
      g = (g + ((int)(rgba.y) << 8)) >> 1;
      b = (b + ((int)(rgba.z) << 8)) >> 1;
    }
  }
  
  r >>= 8;
  g >>= 8;
  b >>= 8;
  
  int supp = (rgba.x << 16) + (rgba.y << 8) + rgba.z;
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
  
  image_result[id].x = r;
  image_result[id].y = g;
  image_result[id].z = b;
  image_result[id].w = 255;
}
