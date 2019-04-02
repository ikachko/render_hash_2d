#!/usr/bin/env iced
argv = require('minimist')(process.argv.slice(2))

switch argv.fn
  when 'gpu'
    hash_fn = require('./render_hash_2d')
  when 'gpu_hard'
    hash_fn = require('./render_hash_2d_hard')
  when 'gpu_hard_unpack'
    hash_fn = require('./render_hash_2d_hard_unpack')
  else
    hash_fn = require('./render_hash_2d_cpu')

await hash_fn.init {plus: argv.plus}, defer(err); throw err if err

msg = Buffer.alloc 80
for i in [0 ... 80]
  msg[i] = i;


start_ts = Date.now()
last_stat_ts = Date.now()
hash_count = 0
for i in [0 ... 2 ** 16]
  now = Date.now()
  if now - last_stat_ts > 1000
    hashrate = hash_count/(now - last_stat_ts)*1000
    process.stdout.write "hashrate: #{hashrate.toFixed(2)} h/s   \r"
    last_stat_ts = now
    hash_count = 0
  hash_count++
  
  msg.writeInt32BE i, 10
  await hash_fn.hash msg, defer(err, hash); throw err if err
  break if hash[0] == 0

esp_ts = Date.now() - start_ts
p '\n'
p "share found in #{esp_ts} ms"
p hash
hash_fn.dump_img()
