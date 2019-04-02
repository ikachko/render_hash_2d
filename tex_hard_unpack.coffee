#!/usr/bin/env iced
require 'fy'
fs = require 'fs'
{PNG} = require 'pngjs'
# ###################################################################################################
#    config
# ###################################################################################################

tex_size_x = 1920
tex_size_y = 1080

# ###################################################################################################

tex_buf_host = Buffer.alloc tex_size_x*tex_size_y*4
file_list = fs.readdirSync('./tex_hard')

for file, idx in file_list
  p "[#{idx+1}/#{file_list.length}] #{file}"
  src_file = "./tex_hard/#{file}"
  dst_file = "./tex_hard_unpack/#{file}.unpack"
  png_data = PNG.sync.read fs.readFileSync src_file
  {
    data
    width
    height
  } = png_data
  tex_buf_host.fill 0
  for x in [0 ... tex_size_x]
    for y in [0 ... tex_size_y]
      src_offset = 4*(x + y*width)
      dst_offset = 4*(x + y*tex_size_x)
      tex_buf_host[dst_offset+0] = data[src_offset+0]
      tex_buf_host[dst_offset+1] = data[src_offset+1]
      tex_buf_host[dst_offset+2] = data[src_offset+2]
      tex_buf_host[dst_offset+3] = 255
  
  fs.writeFileSync dst_file, tex_buf_host