/*
use std::cmp::Eq;
use std::collections::HashMap;
use std::hash::Hash;

pub fn hashmap_visit<K: Hash + Eq, V, F>(hashmap: HashMap<K, V>, key: K, mut f: F)
where
    F: FnMut(&V),
{
    if let Some(v) = &hashmap.get(&key) {
        f(v);
    }
}

pub fn option_visit<V, F>(option: Option<V>, mut f: F)
where
    F: FnMut(&V),
{
    if let Some(v) = &option {
        f(v);
    }
}
*/

pub fn filename_with_extention(filename: &str, extention: &str) -> String {
    let ext_pos = filename.rfind(".").unwrap_or(filename.len());
    format!("{}.{}", filename.get(..ext_pos).unwrap(), extention)
}

pub fn print_cairo_result(result: Result<(), cairo::Error>) {
    match result {
        Ok(()) => (),
        Err(e) => println!("TalkerControl Cairo error {}", e),
    }
}
