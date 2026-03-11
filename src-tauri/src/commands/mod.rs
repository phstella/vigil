// Tauri command wrappers over core services.
// Each command module will expose thin #[tauri::command] functions.

pub mod fs;
pub mod git;
pub mod links;
pub mod search;
pub mod status;
pub mod workspace;
