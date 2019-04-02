# renderhash2d
mining hash function when gpu is the only asic

done as part of 6th hackathon http://blockchainua-hackathon.com/

# How to launch

Assembled for ubuntu but will also work with any other distribution after `npm i` and probably even on windows (visual studio needed for compile)

 * `git clone https://github.com/hu2prod/renderhash2d`
 * `cd renderhash2d`
 * install [nvm](https://github.com/creationix/nvm)
   * `curl -o- https://raw.githubusercontent.com/creationix/nvm/v0.34.0/install.sh | bash`
   * relogin or `source ~/.bash_profile`
 * install node 11 
   * `nvm i 11`
 * install iced-coffee-script globally
   * `npm i -g iced-coffee-script`
 * launch base.coffee
   * `./base.coffee`

## How to launch gpu code
 * `./base.coffee --fn=gpu`

## How to launch renderhash_2d_hard
 * create and fill directory tex_hard with textures from https://rutracker.org/forum/viewtopic.php?t=3597202
   * you must keep only 1001.png - 2820.png if you want recieve same result as me
   * Full torrent recommended
 * `./base.coffee --fn=gpu_hard`

## How to launch renderhash_2d_hard_unpack
unpack is only optimized version when you don't need decode png to texture atlas
 * Make sure that you have enough free space, because unpacked images are 1.5 times larger than original one's
 * `mkdir tex_hard_unpack`
 * `./tex_hard_unpack.coffee`
 * wait
 * `./base.coffee --fn=gpu_hard_unpack`

## How to launch renderhash_2d_hard_plus or renderhash_2d_hard_plus_unpack
 * `./base.coffee --plus --fn=gpu_hard`
 * `./base.coffee --plus --fn=gpu_hard_unpack`
   
# Throubleshooting
## gpu code doesn't work

e.g.

    Error! Fail to load fglrx kernel module! Maybe you can switch to root user to load kernel module directly
    Segmentation fault
 
  * make sure that at least any other open source gpu miner works. (Claymore is not open source)
  * probably binaries are bad for your system. Make `npm i` to make sure that binaries using exactly your env
  * `clinfo` should list your openCL compatible GPU
  * write me (fb virdvird), I'll try to investigate. (еще не зажрался)
