// use anyhow::Result;
// use std::env;
// use std::path::Path;

// /// Detect a Python virtual environment and return its bin path
// pub fn detect_venv_bin() -> Result<Option<String>> {
//     let candidates = [".venv", "venv"];

//     for name in candidates {
//         let bin_path = Path::new(name).join("bin");
//         if bin_path.exists() {
//             return Ok(Some(bin_path.to_string_lossy().to_string()));
//         }
//     }

//     Ok(None)
// }

// /// Build a PATH with venv bin prepended
// pub fn build_env_with_venv() -> Result<Vec<(String, String)>> {
//     let mut envs: Vec<(String, String)> = env::vars().collect();

//     if let Some(venv_bin) = detect_venv_bin()? {
//         let current_path = env::var("PATH").unwrap_or_default();
//         let new_path = format!("{}:{}", venv_bin, current_path);

//         envs.push(("PATH".to_string(), new_path));
//     }

//     Ok(envs)
// }
