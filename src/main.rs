use lazy_static::lazy_static;
use std::collections::{HashMap as Map, HashSet};
use std::env;
use std::fs;
use std::io;
use std::process::exit;
use walkdir::WalkDir;

const DEFAULT_MAX_DEPTH: usize = 2;
const DEFAULT_CONFIDENCE_THRESHOLD: u32 = 5; // Stop after finding this many files of same type
const SEARCH_EXTENSIONS_ENV: &'static str = "FIND_EXT_SEARCH_EXTENSIONS";
const DISALLOWED_FOLDER_ENV: &'static str = "FIND_EXT_DISALLOWED_FOLDERS";
const CACHE_FILE_ENV: &'static str = "FIND_EXT_CACHE_FILE";
const USE_CACHE_ENV: &'static str = "FIND_EXT_USE_CACHE";
const CONFIDENCE_THRESHOLD_ENV: &'static str = "FIND_EXT_CONFIDENCE_THRESHOLD";

fn env(key: &str) -> String {
    env::var(key).expect(&format!("Find Ext: set the enviroment variable '{key}'"))
}

fn env_as_set(key: &str) -> HashSet<String> {
    env(key).split(',').map(str::to_string).collect()
}

lazy_static! {
    static ref LOOK_FOR: HashSet<String> = env_as_set(SEARCH_EXTENSIONS_ENV);
    static ref DISALLOWED_FOLDERS: HashSet<String> = env_as_set(DISALLOWED_FOLDER_ENV);
    static ref CACHE_FILE: String = env(CACHE_FILE_ENV);
    static ref USE_CACHE: bool = env(USE_CACHE_ENV).parse().unwrap_or(false);
}

// Fast marker file detection - check these before walking
fn check_marker_files(path: &str) -> Option<String> {
    let markers = [
        ("package.json", "js"),
        ("Cargo.toml", "rs"),
        ("requirements.txt", "py"),
        ("setup.py", "py"),
        ("go.mod", "go"),
        ("composer.json", "php"),
        ("build.gradle", "kt"),
        ("pom.xml", "java"),
        ("Gemfile", "rb"),
    ];

    for (file, ext) in &markers {
        let marker_path = format!("{}/{}", path, file);
        if fs::metadata(&marker_path).is_ok() {
            return Some(ext.to_string());
        }
    }
    None
}

#[derive(Debug, Default)]
struct Cache {
    folders: Map<String, String>,
}

impl Cache {
    fn load() -> io::Result<Self> {
        let raw = fs::read_to_string(CACHE_FILE.to_string())?;

        let mut folders: Map<String, String> = Map::new();
        let lines = raw.trim().split("\n");
        for line in lines {
            let mut pair = line.split(";");
            let mut next = || pair.next().expect("Invalid Cache").to_string();
            folders.insert(next(), next());
        }

        Ok(Cache { folders })
    }

    fn load_or_new() -> Self {
        Self::load().unwrap_or(Self::default())
    }

    fn save(&self) {
        let raw_data = &self
            .folders
            .iter()
            .map(|(path, extension)| format!("{path};{extension}"))
            .collect::<Vec<String>>()
            .join("\n");

        fs::write(&*CACHE_FILE, raw_data).expect("Failed to save Cache!");
    }

    fn clear() {
        fs::remove_file(&*CACHE_FILE).unwrap();
    }

    fn add(&mut self, path: &str, extension: &str) -> &mut Self {
        self.folders.insert(path.into(), extension.into());
        self
    }
}

fn find_extension(
    path: &str,
    depth: usize,
    look_for: &HashSet<String>,
    cache_opt: &mut Option<Cache>,
    confidence_threshold: u32,
) -> Option<String> {
    if let Some(cache) = cache_opt {
        if let Some(ext) = cache.folders.get(path) {
            return Some(ext.to_string());
        }
    }

    if let Some(marker_ext) = check_marker_files(path) {
        if look_for.contains(&marker_ext) {
            if let Some(cache) = cache_opt {
                cache.add(path, &marker_ext);
            }
            return Some(marker_ext);
        }
    }

    let mut counts: Map<String, u32> = Map::new();
    let mut max_count = 0u32;
    let mut leading_ext: Option<String> = None;

    for entry in WalkDir::new(&path)
        .max_depth(depth)
        .into_iter()
        .filter_entry(|e| {
            !DISALLOWED_FOLDERS
                .iter()
                .any(|disallowed| e.path().to_string_lossy().contains(disallowed))
        })
        .filter_map(|e| e.ok())
    {
        if let Some(ext) = entry.path().extension().and_then(|ext| ext.to_str()) {
            if look_for.contains(ext) {
                let count = counts.entry(ext.to_string()).or_insert(0);
                *count += 1;

                if *count > max_count {
                    max_count = *count;
                    leading_ext = Some(ext.to_string());
                }

                // Early exit: if we found enough files of one type, stop searching
                if max_count >= confidence_threshold {
                    break;
                }
            }
        }
    }

    let max_ext = if max_count >= confidence_threshold {
        leading_ext
    } else {
        counts
            .into_iter()
            .max_by_key(|(_, count)| *count)
            .map(|(ext, _)| ext)
    };

    if let Some(ext) = &max_ext {
        if let Some(cache) = cache_opt {
            cache.add(path, ext);
        }
    }

    max_ext
}

fn display_help_message() {
    let msg = r#"
Usage: find_ext PATH
    --clear (-c) = Clear cache 
    "#
    .trim();
    println!("{msg}");
}

fn main() {
    let mut cache = (*USE_CACHE).then_some(Cache::load_or_new());

    let args: Vec<String> = env::args().collect();
    let path = match args.get(1) {
        Some(p) if p == "-c" || p == "--clear" => {
            Cache::clear();
            println!("Cleared cache!");
            return;
        }
        Some(p) => p,
        None => {
            display_help_message();
            exit(1);
        }
    };

    let depth = args
        .get(2)
        .and_then(|t| t.parse::<usize>().ok())
        .unwrap_or(DEFAULT_MAX_DEPTH);

    let confidence_threshold = env::var(CONFIDENCE_THRESHOLD_ENV)
        .ok()
        .and_then(|v| v.parse::<u32>().ok())
        .unwrap_or(DEFAULT_CONFIDENCE_THRESHOLD);

    let output = find_extension(path, depth, &*LOOK_FOR, &mut cache, confidence_threshold);

    // Save cache once at the end if we have one
    if let Some(c) = cache {
        c.save();
    }

    println!("{}", output.unwrap_or_default());
}
