window = global
# ###################################################################################################
#    timer
# ###################################################################################################
window.once_interval = (timer, cb, interval=100)->
  if !timer
    return setTimeout cb, interval
  return timer
# ###################################################################################################
#    to_s
# ###################################################################################################
String.prototype.to_s = String.prototype.toString
Array.prototype.to_s  = Array.prototype.toString
Number.prototype.to_s = Number.prototype.toString

# ###################################################################################################
#    pretty print
# ###################################################################################################
global.ppw= (t)-> console.log JSON.stringify t, null, 4

# ###################################################################################################
#    hash missing parts
# ###################################################################################################
window.count = (t)->
  return t.length if t instanceof Array
  ret = 0
  for k of t
    ret++
  ret

Array.prototype.hash_key = (key)->
  hash = {}
  for v in @
    hash[v[key]] = @
  @hash = hash
  return

# ###################################################################################################
#    использовался пока не нашел комбинацию cb = defer();do(cb)->
# ###################################################################################################
window.stream_parallel = (on_end)->
  yield_constructor = null
  ret = ()->
    yield_constructor.left--
    on_end() if yield_constructor.left == 0
    return
  process.nextTick ret
  yield_constructor = (incr = 1)->
    yield_constructor.left += incr
    ret
  yield_constructor.left = 1
  yield_constructor.end = ()->
    unless yield_constructor.left == 0
      yield_constructor.left = 0
      on_end()
    return
  yield_constructor