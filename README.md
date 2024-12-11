# find_ext_quick
Quickly determine the most common file extension in a folder, used for a ZSH plugin.

## Install:
`cargo install --path .`

## Usage:
`find_ext $(pwd)` will output the post common file.

## Config:
The following environment variables can be used to configure `find_ext`:
```bash
# Which extensions to look for
export FIND_EXT_SEARCH_EXTENSIONS="rs,py,ml,php,kt,ts,js,lua,res,c,cpp,hs,hc"
# Ignore these large folders
export FIND_EXT_DISALLOWED_FOLDERS="node_modules,target/debug/build,target/release/build"
# Should folder types be cached?
export FIND_EXT_USE_CACHE=true
# Where to store the folder cache
export FIND_EXT_CACHE_FILE="$HOME/.find_ext_cache.csv"
```
