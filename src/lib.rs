pub mod assets {
    use std::{
        env, fs, io,
        path::{Path, PathBuf},
    };

    fn get_base_path() -> PathBuf {
        if let Ok(cargo_manifest_dir) = env::var("CARGO_MANIFEST_DIR") {
            PathBuf::from(cargo_manifest_dir)
        } else {
            let exe_path = env::current_exe().expect("unable to get path to executable");
            exe_path
                .parent()
                .expect("unable to get directory of executable")
                .to_path_buf()
        }
    }

    /// Load the entire contents of the asset at the given `path` relative to the assets folder into a bytes vector.
    pub fn load<P>(path: P) -> io::Result<Vec<u8>>
    where
        P: AsRef<Path>,
    {
        let abs_path = get_base_path().join(path);
        fs::read(abs_path)
    }
}
