#!/usr/bin/env iced
require 'fy'
nooocl = require 'nooocl'
{
  CLBuffer
  CLHost
  CLContext
  CLCommandQueue
  NDRange
} = nooocl
crypto = require 'crypto'
fs = require 'fs'
{PNG} = require 'pngjs'

####################################################################################################
# config
####################################################################################################
image_size_x = 1920
image_size_y = 1080
image_size_byte = image_size_x*image_size_y*4

tex_size_x = 1920
tex_size_y = 1080

rect_count = 16

scale_x = Math.floor image_size_x/255
scale_y = Math.floor image_size_y/255

rect_list_buf_size = rect_count*8*4

rect_list_buf_gpu = null
rect_list_buf_host = null
image_atlas_buf_gpu = null
image_buf_gpu = null
image_buf_host = null
tex_count = null
queue = null
kernel_draw_call_rect_list = null
kernel_global_size = null
kernel_local_size = null
file_list = null

####################################################################################################
# gpu
####################################################################################################
@init = (opt, on_end) ->
  host = CLHost.createV11()
  {defs} = host.cl

  gpu_list = []
  platform_list = host.getPlatforms()
  if !platform_list.length
    return on_end new Error "missing compatible opencl plaftorm"

  for platform in platform_list
    gpu_list.append platform.gpuDevices()
  if !gpu_list.length
    return on_end new Error "missing compatible opencl gpu "

  p "gpu count: #{gpu_list.length}"
  gpu = gpu_list[0];
  p "device: #{gpu.name} #{gpu.platform.name}"
  ctx = new CLContext gpu

  queue = new CLCommandQueue ctx, gpu
  ####################################################################################################
  # buffers
  ####################################################################################################

  rect_list_buf_host = Buffer.alloc rect_list_buf_size
  rect_list_buf_gpu  = new CLBuffer ctx, defs.CL_MEM_READ_ONLY, rect_list_buf_size

  image_buf_host = Buffer.alloc image_size_byte
  image_buf_gpu  = new CLBuffer ctx, defs.CL_MEM_READ_WRITE, image_size_byte

  file_list = fs.readdirSync('./tex_hard_unpack')
  tex_count = file_list.length
  tex_size_bytes = tex_size_x*tex_size_y*4*tex_count

  tex_buf_host = Buffer.alloc tex_size_x*tex_size_y*4
  image_atlas_buf_gpu  = new CLBuffer ctx, defs.CL_MEM_WRITE_ONLY, tex_size_bytes
  
  ####################################################################################################
  # kernel
  ####################################################################################################
  kernel_file =  "./kernel_hard.cl"
  if opt.plus
    kernel_file = "./kernel_hard_plus.cl"
  program = ctx.createProgram fs.readFileSync kernel_file, 'utf-8'
  await program.build('').then defer()
  build_status = program.getBuildStatus gpu
  p program.getBuildLog gpu
  if build_status < 0
    return on_end new Error "can't build."
  kernel_draw_call_rect_list = program.createKernel "draw_call_rect_list"
  kernel_global_size = new NDRange image_size_x*image_size_y
  kernel_local_size  = new NDRange 32

  on_end null


####################################################################################################
# hash
####################################################################################################
@hash = (msg_buf, cb)->
  # TODO lock
  msg_buf1 = Buffer.alloc msg_buf.length + 4
  for i in [0 ... msg_buf.length]
    msg_buf1[i+4] = msg_buf[i]
  
  
  offset = 0
  rect_list = []
  for i in [0 ... rect_count]
    msg_buf1.writeInt32LE i, 0
    scene_seed = crypto.createHash('sha256').update(msg_buf1).digest()
    rect_list.push {
      x : scene_seed[offset++ % scene_seed.length]
      y : scene_seed[offset++ % scene_seed.length]
      w : scene_seed[offset++ % scene_seed.length] * scale_x
      h : scene_seed[offset++ % scene_seed.length] * scale_y
      t : scene_seed[offset++ % scene_seed.length] % tex_count
    }
  
  t_idx = 0
  t_hash = {}
  selected_file_list = []
  for rect in rect_list
    if !t_hash[rect.t]?
      t_hash[rect.t] = t_idx++
      selected_file_list.push file_list[rect.t]
  
  for rect in rect_list
    rect.t = t_hash[rect.t]
  
  tex_offset = 0
  for file in selected_file_list
    full_file = "./tex_hard_unpack/#{file}"
    tex_buf_host = fs.readFileSync full_file
    if tex_buf_host.length != tex_size_x*tex_size_y*4
      return cb new Error "bad file size #{full_file} #{tex_buf_host.length} != #{tex_size_x*tex_size_y*4}"
    
    await queue.waitable().enqueueWriteBuffer(image_atlas_buf_gpu, tex_offset, tex_buf_host.length, tex_buf_host).promise.then defer()
    tex_offset += tex_buf_host.length
  
  for rect,idx in rect_list
    rect_offset = idx*8*4
    rect_list_buf_host.writeInt32LE(rect.x, rect_offset); rect_offset += 4
    rect_list_buf_host.writeInt32LE(rect.y, rect_offset); rect_offset += 4
    rect_list_buf_host.writeInt32LE(rect.w, rect_offset); rect_offset += 4
    rect_list_buf_host.writeInt32LE(rect.h, rect_offset); rect_offset += 4
    rect_list_buf_host.writeInt32LE(rect.t, rect_offset); rect_offset += 4
  
  queue.enqueueWriteBuffer rect_list_buf_gpu, 0, rect_list_buf_size, rect_list_buf_host

  kernel_draw_call_rect_list.setArg 0, rect_list_buf_gpu
  kernel_draw_call_rect_list.setArg 1, image_atlas_buf_gpu
  kernel_draw_call_rect_list.setArg 2, image_buf_gpu
  kernel_draw_call_rect_list.setArg 3, rect_list.length, "uint"
  kernel_draw_call_rect_list.setArg 4, image_size_x, "uint"
  kernel_draw_call_rect_list.setArg 5, tex_size_x, "uint"
  kernel_draw_call_rect_list.setArg 6, tex_size_y, "uint"

  queue.enqueueNDRangeKernel kernel_draw_call_rect_list, kernel_global_size, kernel_local_size

  await queue.waitable().enqueueReadBuffer(image_buf_gpu, 0, image_size_byte, image_buf_host).promise.then defer()

  cb null, crypto.createHash('sha256').update(image_buf_host).digest()

@dump_img = ()->
  options = { colorType: 6 } # RGBA
  buffer = PNG.sync.write {data:image_buf_host, width:image_size_x, height:image_size_y}, options
  fs.writeFileSync 'result.png', buffer