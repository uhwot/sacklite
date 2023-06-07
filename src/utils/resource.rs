use std::path::PathBuf;

pub fn check_sha1(hash: &str) -> bool {
    match hex::decode(hash) {
        Ok(hash) => hash.len() == 20,
        Err(_) => false
    }
}

pub fn get_res_path(resource_dir: &str, hash: &str) -> PathBuf {
    let mut path = PathBuf::from(resource_dir);
    path.push(hash);
    path
}