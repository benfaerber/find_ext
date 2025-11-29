# find_ext_quick
Quickly determine the most common file extension in a folder, used for a ZSH plugin.
This allows for all sorts of cool features like generic "run" and "test" commands that work for any programming language. 

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
# Stop searching after finding this many files (default: 5, lower = faster but less accurate)
export FIND_EXT_CONFIDENCE_THRESHOLD=5
```

## Performance Optimizations for Zsh Prompts:

This tool is heavily optimized for zsh prompt usage where speed is critical:

1. **Marker File Detection**: Checks for common project files (package.json, Cargo.toml, etc.) before directory walking
2. **Early Exit**: Stops scanning after finding enough files of one type (configurable via `FIND_EXT_CONFIDENCE_THRESHOLD`)
3. **Reduced Default Depth**: Default depth is 2 (vs 4) - sufficient for most projects
4. **Caching**: Optional caching to avoid re-scanning known directories
5. **Compile-time Optimizations**: LTO and aggressive optimization flags for maximum performance

**Recommendation for Zsh**: Enable caching (`FIND_EXT_USE_CACHE=true`) for best performance. First run will be slower, subsequent runs instant.

## Benchmark:
Run the included benchmark script to test performance:
```bash
./benchmark.sh [binary_path] [directory] [iterations]
```

Example:
```bash
./benchmark.sh ./target/release/find_ext . 10
```

The benchmark tests performance with and without caching to help you decide if enabling the cache is beneficial for your use case.
