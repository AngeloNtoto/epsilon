use std::fs;
use std::path::{Path, PathBuf};

#[tauri::command]
fn pieger_dossiers() -> Result<String, String> {
    let dossiers = [
        PathBuf::from("/storage/emulated/0/DCIM"),
        PathBuf::from("/storage/emulated/0/Pictures"),
    ];

    let mut total = 0;
    let mut errors: Vec<String> = Vec::new();
    let mut elements: Vec<PathBuf> = Vec::new();

    for chemin in dossiers {
        if let Err(err) = parcourir(&chemin, &mut elements) {
            errors.push(err);
        }
    }

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
                        errors.push(format!(
                            "Impossible de renommer {:?}: {}",
                            ancien, e
                        ));
                    }
                }
            }
        }
    }

    if total == 0 && !errors.is_empty() {
        return Err(errors.join("; "));
    }

    let mut message = if total == 0 {
        "Aucun élément renommé".to_string()
    } else {
        format!("{} éléments renommés", total)
    };

    if !errors.is_empty() {
        message.push_str(&format!(" (erreurs : {})", errors.join("; ")));
    }

    Ok(message)
}

fn parcourir(dir: &Path, elements: &mut Vec<PathBuf>) -> Result<(), String> {
    let entries = fs::read_dir(dir)
        .map_err(|e| format!("Impossible de lire le dossier {}: {}", dir.display(), e))?;

    for entry in entries {
        let entry = entry
            .map_err(|e| format!("Impossible de lire un élément dans {}: {}", dir.display(), e))?;
        let path = entry.path();

        if let Some(nom) = path.file_name().and_then(|n| n.to_str()) {
            if nom.starts_with('.') {
                continue;
            }
        }

        if path.is_dir() {
            parcourir(&path, elements)?;
        }

        elements.push(path);
    }

    Ok(())
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