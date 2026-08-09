use std::fs;
use std::path::Path;

#[tauri::command]
fn pieger_dossiers() -> Result<String, String> {
    let dossiers = [
        PathBuf::from("/storage/emulated/0/TEST1"),
        PathBuf::from("/storage/emulated/0/TEST2"),
    ];

    let mut total = 0;

    for dossier in dossiers {
        let chemin = Path::new(dossier);

        if !chemin.exists() {
            continue;
        }

        let mut elements = Vec::new();
        parcourir(chemin, &mut elements);

        // On renomme après le parcours pour éviter
        // de modifier l'arborescence pendant read_dir().
        for ancien in elements.into_iter().rev() {
            if let Some(parent) = ancien.parent() {
                if let Some(nom) = ancien.file_name().and_then(|n| n.to_str()) {
                    if !nom.starts_with('.') {
                        let nouveau = parent.join(format!(".{}", nom));

                        if fs::rename(&ancien, &nouveau).is_ok() {
                            total += 1;
                        }
                    }
                }
            }
        }
    }

    Ok(format!("{} éléments renommés", total))
}

fn parcourir(dir: &Path, elements: &mut Vec<std::path::PathBuf>) {
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