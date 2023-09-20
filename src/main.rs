use std::env;
use std::collections::HashMap;
use std::fs;
use std::io;
use walkdir::WalkDir;
use lazy_static::lazy_static;

const DEFAULT_MAX_DEPTH: usize = 4;
const SEARCH_EXTENSIONS_ENV: &'static str = "FIND_EXT_SEARCH_EXTENSIONS";
const DISALLOWED_FOLDER_ENV: &'static str = "FIND_EXT_DISALLOWED_FOLDERS";
const CACHE_FILE_ENV: &'static str = "FIND_EXT_CACHE_FILE";
const USE_CACHE_ENV: &'static str = "FIND_EXT_USE_CACHE";

fn env(key: &str) -> String {
    env::var(key)
        .expect(&format!("Find Ext: set the enviroment variable '{key}'"))
}

fn env_as_vec(key: &str) -> Vec<String> {
    env(key) 
        .split(',')
        .map(|k| k.to_string())
        .collect()
}

lazy_static! {
    static ref LOOK_FOR: Vec<String> = env_as_vec(SEARCH_EXTENSIONS_ENV);
    static ref DISALLOWED_FOLDERS: Vec<String> = env_as_vec(DISALLOWED_FOLDER_ENV);
    static ref CACHE_FILE: String = env(CACHE_FILE_ENV);
    static ref USE_CACHE: bool = env(USE_CACHE_ENV) == "true";
}

#[derive(Debug, Default, Clone)]
struct CacheItem {
    path: String,
    extension: String,
}

#[derive(Debug, Default, Clone)]
struct Cache(Vec<CacheItem>);

impl Cache {
    fn load() -> io::Result<Self> {
        let raw = fs::read_to_string(CACHE_FILE.to_string())?;
    
        let lines = raw.trim().split("\n");
        let folders = lines.map(|entry| {
            let mut pair = entry.split(";");
            let mut next = || pair.next().expect("Invalid Cache").to_string();
            CacheItem {
                path: next(), 
                extension: next(), 
            }
        }).collect();

        Ok(Cache(folders))
    }

    fn load_or_new() -> Self {
        Self::load().unwrap_or(Self::default())
    }

    fn save(&self) {
        let Cache(folders) = self;
        let raw_data = folders 
            .iter()
            .map(|CacheItem { path, extension }| format!("{path};{extension}")) 
            .collect::<Vec<String>>()
            .join("\n");
        
        fs::write(&*CACHE_FILE, raw_data)
            .expect("Failed to save Cache!");
    }

    fn add(&mut self, path: &str, extension: &str) -> &mut Self {
        let item = CacheItem {
            path: path.into(),
            extension: extension.into(),
        };
        
        self.0.push(item);
        self
    }
}

fn find_extension(path: &str, depth: usize, look_for: &Vec<String>, cache_opt: &mut Option<Cache>) -> Option<String> {
    if let Some(cache) = cache_opt {
        if let Some(CacheItem { extension, .. }) = cache.0.iter().find(|item| item.path == path) {
            return Some(extension.to_string());
        }
    }

    let cache_key = path.clone();
    let exts: Vec<String> = WalkDir::new(path)
        .max_depth(depth)
        .into_iter()
        .filter_map(|p| {
            let path = p.unwrap().path().to_string_lossy().to_string();
                    
            if (*DISALLOWED_FOLDERS).iter().any(|disallowed| path.contains(disallowed)) {
                return None
            } 
            
            Some(
                path.split(".")
                    .last()
                    .unwrap_or("")
                    .to_string()
            )
        })
        .collect();

    let mut counts: HashMap<String, u16> = HashMap::new();
    for ext in exts {
        if !look_for.contains(&ext) {
            continue;
        }
        let last = counts.get(&ext).unwrap_or(&0);
        counts.insert(ext, *last + 1);
    }
    
    let (ext, _) = counts.into_iter()
        .max_by(|(_, v1), (_, v2)| v1.cmp(v2))
        .unwrap_or(("".into(), 0));
    
    if ext != "" {
        if let Some(cache) = cache_opt {
            cache.add(&cache_key, &ext).save();
        }
        Some(ext) 
    } else {
        None
    } 
}

fn main() {
    let mut cache = (*USE_CACHE)
        .then_some(Cache::load_or_new());

    let args: Vec<String> = env::args().collect();
    let path = args.get(1);
    let depth = args.get(2)
        .and_then(|t| t.parse::<usize>().ok())
        .unwrap_or(DEFAULT_MAX_DEPTH);
    
    let output: Option<String> = path
        .and_then(|path: &String| {
            let mut find_depth = |depth: usize| find_extension(path, depth, &*LOOK_FOR, &mut cache); 
            let attempt = find_depth(depth); 
            if let None = attempt {
                find_depth(depth+2) 
            } else {
                attempt
            }
        });

    println!("{}", output.unwrap_or("".to_string()));
}
