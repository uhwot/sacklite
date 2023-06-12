use std::path::PathBuf;

pub fn check_sha1(hash: &str) -> bool {
    hash.len() == 40 && hash.chars().all(|c| matches!(c, '0'..='9' | 'a'..='f'))
}

pub fn get_res_path(resource_dir: &str, hash: &str) -> PathBuf {
    let mut path = PathBuf::from(resource_dir);
    path.push(hash);
    path
}

pub fn res_exists(resource_dir: &str, res_ref: &str, required: bool, is_hash: bool) -> bool {
    if !required && (res_ref.is_empty() || res_ref == "0") { return true; }
    if is_hash && res_ref.starts_with("g") {
        match res_ref[1..].parse::<u32>() {
            Ok(_) => return true,
            Err(_) => return false,
        }
    }
    check_sha1(res_ref) && get_res_path(resource_dir, res_ref).exists()
}