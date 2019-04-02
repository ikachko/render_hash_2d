assert = require 'assert'
module = @
@json_eq = (a,b)->
  assert.strictEqual JSON.stringify(a,null,2), JSON.stringify(b,null,2)

@wrap = (fn, fin)->
  old_perr = global.perr
  old_pp   = global.pp
  old_p    = global.p
  old_puts = global.puts
  old_exit = process.exit
  global.perr = ()->
  global.pp   = ()->
  global.p    = ()->
  global.puts = ()->
  process.exit = ()->throw new Error "process.exit stub"
  
  e = null
  try
    fn()
  catch _e
    e = _e
  
  global.perr = old_perr
  global.pp   = old_pp
  global.p    = old_p
  global.puts = old_puts
  process.exit = old_exit
  fin?()
  
  throw e if e
  return
  
@not_throws = (t, fin)->
  module.wrap t, fin
  return

@throws = (t, fin)->
  module.wrap ()->
    assert.throws t
  , fin
  return
