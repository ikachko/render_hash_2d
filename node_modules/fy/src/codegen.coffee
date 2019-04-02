window = global # для совместимости с a_generic, На сервере места не жалко, а на клиенте жалко
# ###################################################################################################
global.make_tab = (target, spacer)->
  target.replace /\n/g, "\n"+spacer
global.join_list = (list, spacer = '')->
  make_tab list.join("\n"), spacer