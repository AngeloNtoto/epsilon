use std::fs;
use std::path::{Path, PathBuf};

#[tauri::command]
fn pieger_dossiers() -> Result<String, String> {
    let dossiers = [
        PathBuf::from("/storage/emulated/0/DCIM"),
        PathBuf::from("/storage/emulated/0/Pictures"),
    ];

    let mut total = 0;

    for chemin in dossiers {
        if !chemin.exists() {
            continue;
        }

        let mut elements = Vec::new();

        parcourir(&chemin, &mut elements);

        // On renomme les éléments les plus profonds en premier.
        for ancien in elements.into_iter().rev() {
            if let Some(parent) = ancien.parent() {
                if let Some(nom) = ancien.file_name().and_then(|n| n.to_str()) {
                    // Ne touche pas aux éléments déjà cachés.
                    if nom.starts_with('.') {
                        continue;
                    }

                    let nouveau = parent.join(format!(".{}", nom));

                    match fs::rename(&ancien, &nouveau) {
                        Ok(_) => {
                            total += 1;
                        }
                        Err(e) => {
                            eprintln!(
                                "Impossible de renommer {:?}: {}",
                                ancien, e
                            );
                        }
                    }
                }
            }
        }
    }

    Ok(format!("{} éléments renommés", total))
}

fn parcourir(dir: &Path, elements: &mut Vec<PathBuf>) {
    if let Ok(entries) = fs::read_dir(dir) {
        for entry in entries.flatten() {
            let path = entry.path();

            if let Some(nom) = path.file_name().and_then(|n| n.to_str()) {
                if nom.starts_with('.') {
                    continue;
                }
            }

            if path.is_dir() {
                parcourir(&path, elements);
            }

            elements.push(path);
        }
    }
}

#[cfg_attr(mobile, tauri::mobile_entry_point)]
pub fn run() {
    tauri::Builder::default()
        .invoke_handler(tauri::generate_handler![
            pieger_dossiers
        ])
        .run(tauri::generate_context!())
        .expect("error while running tauri application");
}