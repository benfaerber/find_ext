use std::env;
use std::collections::HashMap;
use std::panic;
use std::fs;
use walkdir::WalkDir;
use lazy_static::lazy_static;
use serde::{Serialize, Deserialize};
use serde_json;

const DEFAULT_MAX_DEPTH: usize = 4;
const SEARCH_EXTENSIONS_ENV: &'static str = "FIND_EXT_SEARCH_EXTENSIONS";
const DISALLOWED_FOLDER_ENV: &'static str = "FIND_EXT_DISALLOWED_FOLDERS";
const CACHE_FILE_ENV: &'static str = "FIND_EXT_CACHE_FILE";

fn env(key: &str) -> String {
    env::var(key)
        .expect(&format!("Find Ext: set the enviroment variable {}", key))
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
}

#[derive(Debug, Deserialize, Serialize, Default, Clone)]
struct CacheItem {
    path: String,
    extension: String,
}

#[derive(Debug, Deserialize, Serialize, Default)]
struct Cache {
    folders: Vec<CacheItem>, 
}

impl Cache {
    fn load() -> anyhow::Result<Self> {
        let raw = fs::read_to_string(CACHE_FILE.to_string())?;
        Ok(serde_json::from_str::<Self>(&raw)?)
    }

    fn save(&self) {
        let raw_data = serde_json::to_string_pretty(self).expect("Failed to serialize cache");
        fs::write(&*CACHE_FILE, raw_data).expect("Failed to save Cache!");
    }

    fn add(&self, path: &str, ext: &str) -> Self { 
        let item = CacheItem {
            path: path.into(),
            extension: ext.into(),
        };

        let folders = [
            self.folders.clone(),
            vec![item]
        ].concat();
        Self { folders } 
    }
}

fn find_extension(path: String, depth: usize, look_for: Vec<String>, cache: Cache) -> Option<String> {
    if let Some(CacheItem { extension, .. }) = cache.folders.iter().find(|item| item.path == path) {
        return Some(extension.to_string());
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
        cache.add(&cache_key, &ext).save();
        Some(ext) 
    } else {
        None
    } 
}

fn main() {
    // Ignore all failures
    panic::set_hook(Box::new(|_| {}));
    
    let cache = Cache::load().unwrap_or(Cache::default());
    let args: Vec<String> = env::args().collect();
    let path = args.get(1);
    let depth = args.get(2)
        .map(|t| t.parse::<usize>().unwrap_or(DEFAULT_MAX_DEPTH))
        .unwrap_or(DEFAULT_MAX_DEPTH);
    
    let output: Option<String> = path
        .and_then(|path: &String| find_extension(path.into(), depth, LOOK_FOR.clone(), cache));

    println!("{}", output.unwrap_or("".to_string()));
}
