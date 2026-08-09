use serde::{Deserialize, Serialize};
use std::fs;
use std::path::{Path, PathBuf};

const MANIFEST_NAME: &str = ".epsilon-hidden.json";

#[derive(Serialize, Deserialize)]
struct HiddenEntry {
    original: String,
    hidden: String,
}

#[tauri::command]
fn pieger_dossiers() -> Result<String, String> {
    let dossiers = [
        PathBuf::from("/storage/emulated/0/DCIM"),
        PathBuf::from("/storage/emulated/0/Pictures"),
        PathBuf::from("/storage/emulated/0/Movies"),
        PathBuf::from("/storage/emulated/0/Download"),
        PathBuf::from("/storage/emulated/0/WhatsAppResources"),
        PathBuf::from("/storage/emulated/0/MyAlbums"),
    ];

    let mut total = 0;
    let mut errors: Vec<String> = Vec::new();
    let mut entries_by_root: Vec<(PathBuf, Vec<HiddenEntry>)> = Vec::new();

    for root in dossiers {
        let mut elements: Vec<PathBuf> = Vec::new();
        if let Err(err) = parcourir(&root, &mut elements) {
            errors.push(err);
            continue;
        }

        let mut hidden_entries = Vec::new();

        for ancien in elements.into_iter().rev() {
            if let Some(parent) = ancien.parent() {
                if let Some(nom) = ancien.file_name().and_then(|n| n.to_str()) {
                    if nom.starts_with('.') {
                        continue;
                    }

                    let nouveau = parent.join(format!(".{}", nom));

                    match fs::rename(&ancien, &nouveau) {
                        Ok(_) => {
                            total += 1;
                            if let (Ok(original), Ok(hidden)) = (
                                ancien.strip_prefix(&root).map(|p| p.to_string_lossy().to_string()),
                                nouveau.strip_prefix(&root).map(|p| p.to_string_lossy().to_string()),
                            ) {
                                hidden_entries.push(HiddenEntry { original, hidden });
                            }
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

        if !hidden_entries.is_empty() {
            entries_by_root.push((root.clone(), hidden_entries));
        }
    }

    for (root, hidden_entries) in entries_by_root {
        if let Err(err) = save_manifest(&root, &hidden_entries) {
            errors.push(err);
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

#[tauri::command]
fn demasquer_dossiers() -> Result<String, String> {
    let dossiers = [
        PathBuf::from("/storage/emulated/0/DCIM"),
        PathBuf::from("/storage/emulated/0/Pictures"),
        PathBuf::from("/storage/emulated/0/Movies"),
        PathBuf::from("/storage/emulated/0/Download"),
        PathBuf::from("/storage/emulated/0/WhatsAppResources"),
        PathBuf::from("/storage/emulated/0/MyAlbums"),
    ];

    let mut restored = 0;
    let mut errors: Vec<String> = Vec::new();

    for root in &dossiers {
        let manifest = load_manifest(root);
        let manifest = match manifest {
            Ok(Some(entries)) => entries,
            Ok(None) => continue,
            Err(err) => {
                errors.push(err);
                continue;
            }
        };

        let mut entries = manifest;
        entries.sort_by(|a, b| {
            b.hidden
                .matches('/')
                .count()
                .cmp(&a.hidden.matches('/').count())
        });

        for entry in entries {
            let hidden_path = resolve_hidden_path(root, &entry.hidden);
            let original_path = root.join(&entry.original);

            if !hidden_path.exists() {
                errors.push(format!("Fichier masqué introuvable : {}", hidden_path.display()));
                continue;
            }
            if original_path.exists() {
                errors.push(format!("Impossible de restaurer car le fichier existe déjà : {}", original_path.display()));
                continue;
            }

            if let Some(parent) = original_path.parent() {
                if !parent.exists() {
                    if let Err(err) = fs::create_dir_all(parent) {
                        errors.push(format!("Impossible de créer le dossier {}: {}", parent.display(), err));
                        continue;
                    }
                }
            }

            match fs::rename(&hidden_path, &original_path) {
                Ok(_) => restored += 1,
                Err(e) => errors.push(format!("Impossible de restaurer {:?}: {}", hidden_path, e)),
            }
        }

        if let Err(err) = remove_manifest(root) {
            errors.push(err);
        }
    }

    if restored == 0 && errors.is_empty() {
        return Ok("Aucun fichier masqué par le programme à restaurer.".to_string());
    }

    let mut message = format!("{} éléments restaurés", restored);
    if !errors.is_empty() {
        message.push_str(&format!(" (erreurs : {})", errors.join("; ")));
    }

    Ok(message)
}

fn manifest_path(root: &Path) -> PathBuf {
    root.join(MANIFEST_NAME)
}

fn save_manifest(root: &Path, entries: &[HiddenEntry]) -> Result<(), String> {
    let manifest_path = manifest_path(root);
    let file = fs::File::create(&manifest_path)
        .map_err(|e| format!("Impossible d’écrire le manifeste {}: {}", manifest_path.display(), e))?;
    serde_json::to_writer_pretty(file, entries)
        .map_err(|e| format!("Impossible de sérialiser le manifeste {}: {}", manifest_path.display(), e))
}

fn load_manifest(root: &Path) -> Result<Option<Vec<HiddenEntry>>, String> {
    let manifest_path = manifest_path(root);
    if !manifest_path.exists() {
        return Ok(None);
    }
    let file = fs::File::open(&manifest_path)
        .map_err(|e| format!("Impossible d’ouvrir le manifeste {}: {}", manifest_path.display(), e))?;
    let entries: Vec<HiddenEntry> = serde_json::from_reader(file)
        .map_err(|e| format!("Impossible de lire le manifeste {}: {}", manifest_path.display(), e))?;
    Ok(Some(entries))
}

fn remove_manifest(root: &Path) -> Result<(), String> {
    let manifest_path = manifest_path(root);
    if manifest_path.exists() {
        fs::remove_file(&manifest_path)
            .map_err(|e| format!("Impossible de supprimer le manifeste {}: {}", manifest_path.display(), e))?;
    }
    Ok(())
}

fn resolve_hidden_path(root: &Path, hidden: &str) -> PathBuf {
    let hidden_path = root.join(hidden);
    if hidden_path.exists() {
        return hidden_path;
    }

    let hidden_rel = Path::new(hidden);
    if let Some(file_name) = hidden_rel.file_name().and_then(|n| n.to_str()) {
        if !file_name.starts_with('.') {
            let alt = if let Some(parent) = hidden_rel.parent() {
                root.join(parent).join(format!(".{}", file_name))
            } else {
                root.join(format!(".{}", file_name))
            };
            if alt.exists() {
                return alt;
            }
        }
    }

    hidden_path
}

fn parcourir(dir: &Path, elements: &mut Vec<PathBuf>) -> Result<(), String> {
    let entries = match fs::read_dir(dir) {
        Ok(entries) => entries,
        Err(err) if err.kind() == std::io::ErrorKind::NotFound => {
            return Ok(());
        }
        Err(err) => {
            return Err(format!("Impossible de lire le dossier {}: {}", dir.display(), err));
        }
    };

    for entry in entries {
        let entry = match entry {
            Ok(entry) => entry,
            Err(err) => {
                return Err(format!("Impossible de lire un élément dans {}: {}", dir.display(), err));
            }
        };
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
            pieger_dossiers,
            demasquer_dossiers
        ])
        .run(tauri::generate_context!())
        .expect("error while running tauri application");
}