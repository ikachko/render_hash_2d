#!/usr/bin/env iced
require 'fy'
crypto = require 'crypto'
fs = require 'fs'
{PNG} = require 'pngjs'

####################################################################################################
# config
####################################################################################################
image_size_x = 1920*2
image_size_y = 1080*2
image_size_byte = image_size_x*image_size_y*4

tex_size_x = 1920
tex_size_y = 1080

rect_count = 6

scale_x = Math.floor image_size_x/255
scale_y = Math.floor image_size_y/255

rect_list_buf_size = 1000*8*4

image_atlas = null
image_result = null

tex_count = null

####################################################################################################
# gpu
####################################################################################################
@init = (opt, on_end) ->
  file_list = fs.readdirSync('./tex')
  tex_count = file_list.length
  tex_size_bytes = tex_size_x*tex_size_y*4*tex_count

  image_atlas = Buffer.alloc tex_size_bytes
  atlases = []
  tex_offset = 0
  for file in file_list
    p file
    full_file = "./tex/#{file}"
    png_data = PNG.sync.read fs.readFileSync full_file
    {
      data
      width
      height
    } = png_data
    for x in [0 ... tex_size_x]
      for y in [0 ... tex_size_y]
        src_offset = 4*(x + y*width)
        dst_offset = 4*(x + y*tex_size_x) + tex_offset
        image_atlas[dst_offset+0] = data[src_offset+0]
        image_atlas[dst_offset+1] = data[src_offset+1]
        image_atlas[dst_offset+2] = data[src_offset+2]
        image_atlas[dst_offset+3] = 255
    
    tex_offset += tex_size_x*tex_size_y*4
    
    # atlases.push(data)


  # options = { colorType: 6 } # RGBA
  # buffer = PNG.sync.write {data:image_atlas, width:tex_size_x, height:tex_size_y * tex_count}, options
  # fs.writeFileSync 'atlas.png', buffer

  ####################################################################################################
  # kernel
  ####################################################################################################

  on_end null


####################################################################################################
# hash
####################################################################################################
@hash = (msg_buf, cb)->
  # TODO lock
  scene_seed = crypto.createHash('sha256').update(msg_buf).digest()
  
  offset = 0
  rect_list = []
  for i in [0 ... rect_count]
    rect_list.push {
      x : scene_seed[offset++ % scene_seed.length] * scale_x
      y : scene_seed[offset++ % scene_seed.length] * scale_y
      w : scene_seed[offset++ % scene_seed.length] * scale_x
      h : scene_seed[offset++ % scene_seed.length] * scale_y
      t : scene_seed[offset++ % scene_seed.length] % tex_count
    }
  image_result = Buffer.alloc image_size_byte
  this.render_image(rect_list, image_atlas, image_result, image_size_x, image_size_y)

  cb null, crypto.createHash('sha256').update(image_result).digest()

@dump_img = ()->
  options = { colorType: 6 } # RGBA
  buffer = PNG.sync.write {data:image_result, width:image_size_x, height:image_size_y}, options
  fs.writeFileSync 'result.png', buffer
  p "saved"

@render_image = (rect_list, image_atlas, image_result, size_x, size_y)->
  for id in [0 ... size_x * size_y]
      x = id % size_x
      y = id // size_x
      rect_id = -1
      for i in [0 ... rect_list.length]
        rect = rect_list[i]
        rect_x = rect.x
        rect_y = rect.y
        rect_w = rect.w
        rect_h = rect.h

        fit_x = x >= rect_x && x < rect_x + rect_w
        fit_y = y >= rect_y && y < rect_y + rect_h
        rect_id = if (fit_x && fit_y) then i else rect_id
      
      if rect_id == -1
        image_result.writeUInt8(128, 4*id)
        image_result.writeUInt8(128, 4*id + 1)
        image_result.writeUInt8(128, 4*id + 2)
        image_result.writeUInt8(255, 4*id + 3)
        continue
      
      rect_x = rect_list[rect_id].x;
      rect_y = rect_list[rect_id].y;
      rect_tex_idx = rect_list[rect_id].t;
      
      tex_offset_x = (x - rect_x) % tex_size_x;
      tex_offset_y = (y - rect_y) % tex_size_y;
      tex_offset = tex_offset_x + tex_offset_y * tex_size_x;
      tex_offset += rect_tex_idx * tex_size_x * tex_size_y;
      
      image_result[4*id+0] = image_atlas[4*tex_offset+0]
      image_result[4*id+1] = image_atlas[4*tex_offset+1]
      image_result[4*id+2] = image_atlas[4*tex_offset+2]
      image_result[4*id+3] = image_atlas[4*tex_offset+3]
      #pixel = image_atlas.readUInt32BE(Math.floor tex_offset)
      #image_result.writeUInt32BE(pixel, id)
