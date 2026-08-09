use serde::{Deserialize, Serialize};
use std::fs;
use std::path::{Path, PathBuf};

const MANIFEST_NAME: &str = ".epsilon-hidden.json";

#[derive(Serialize, Deserialize)]
struct HiddenEntry {
    original: String,
    hidden: String,
}


// ═══════════════════════════════════════════════════════════════
// DOSSIERS CIBLES
// ═══════════════════════════════════════════════════════════════

fn dossiers_cibles() -> [PathBuf; 6] {
    [
        PathBuf::from("/storage/emulated/0/DCIM"),
        PathBuf::from("/storage/emulated/0/Pictures"),
        PathBuf::from("/storage/emulated/0/Movies"),
        PathBuf::from("/storage/emulated/0/Download"),
        PathBuf::from("/storage/emulated/0/WhatsAppResources"),
        PathBuf::from("/storage/emulated/0/MyAlbums"),
    ]
}


// ═══════════════════════════════════════════════════════════════
// MASQUER
// ═══════════════════════════════════════════════════════════════

#[tauri::command]
fn pieger_dossiers() -> Result<String, String> {
    let dossiers = dossiers_cibles();

    let mut total = 0;
    let mut errors: Vec<String> = Vec::new();
    let mut entries_by_root: Vec<(PathBuf, Vec<HiddenEntry>)> = Vec::new();

    for root in dossiers {
        if !root.exists() {
            errors.push(format!(
                "Dossier introuvable : {}",
                root.display()
            ));
            continue;
        }

        let mut elements: Vec<PathBuf> = Vec::new();

        if let Err(err) = parcourir_visible(&root, &mut elements) {
            errors.push(err);
            continue;
        }

        // Les éléments les plus profonds sont traités en premier.
        elements.sort_by_key(|p| std::cmp::Reverse(p.components().count()));

        let mut hidden_entries = Vec::new();

        for ancien in elements {
            let nom = match ancien.file_name().and_then(|n| n.to_str()) {
                Some(nom) => nom,
                None => continue,
            };

            // Déjà caché
            if nom.starts_with('.') {
                continue;
            }

            let parent = match ancien.parent() {
                Some(parent) => parent,
                None => continue,
            };

            let nouveau = parent.join(format!(".{}", nom));

            // Évite d'écraser un élément existant.
            if nouveau.exists() {
                errors.push(format!(
                    "Impossible de masquer {} : {} existe déjà",
                    ancien.display(),
                    nouveau.display()
                ));
                continue;
            }

            match fs::rename(&ancien, &nouveau) {
                Ok(_) => {
                    total += 1;

                    let original = ancien
                        .strip_prefix(&root)
                        .unwrap_or(&ancien)
                        .to_string_lossy()
                        .to_string();

                    let hidden = nouveau
                        .strip_prefix(&root)
                        .unwrap_or(&nouveau)
                        .to_string_lossy()
                        .to_string();

                    hidden_entries.push(HiddenEntry {
                        original,
                        hidden,
                    });

                    println!(
                        "MASQUÉ : {} -> {}",
                        ancien.display(),
                        nouveau.display()
                    );
                }

                Err(e) => {
                    errors.push(format!(
                        "Impossible de renommer {} : {}",
                        ancien.display(),
                        e
                    ));
                }
            }
        }

        // Le manifeste reste une sauvegarde.
        if !hidden_entries.is_empty() {
            if let Err(err) = save_manifest(&root, &hidden_entries) {
                errors.push(err);
            }
        }
    }

    if total == 0 && !errors.is_empty() {
        return Err(errors.join("\n"));
    }

    let mut message = if total == 0 {
        "Aucun élément à masquer.".to_string()
    } else {
        format!("{} éléments masqués.", total)
    };

    if !errors.is_empty() {
        message.push_str("\n\nErreurs :\n");
        message.push_str(&errors.join("\n"));
    }

    Ok(message)
}


// ═══════════════════════════════════════════════════════════════
// DÉMASQUER
// ═══════════════════════════════════════════════════════════════
//
// IMPORTANT :
// Cette fonction NE dépend PAS du manifeste.
//
// Elle inspecte directement le stockage et recherche :
//
//     .photo.jpg
//     .video.mp4
//     .Screenshots
//     .WhatsApp
//     .n'importe_quoi
//
// puis les transforme en :
//
//     photo.jpg
//     video.mp4
//     Screenshots
//     WhatsApp
//     n'importe_quoi
//
// ═══════════════════════════════════════════════════════════════

#[tauri::command]
fn demasquer_dossiers() -> Result<String, String> {
    let dossiers = dossiers_cibles();

    let mut restored = 0;
    let mut errors: Vec<String> = Vec::new();

    for root in dossiers {
        if !root.exists() {
            continue;
        }

        println!("Analyse : {}", root.display());

        let mut elements: Vec<PathBuf> = Vec::new();

        // Ici, contrairement à parcourir_visible(),
        // ON ENTRE AUSSI DANS LES DOSSIERS CACHÉS.
        if let Err(err) = parcourir_tous(&root, &mut elements) {
            errors.push(err);
            continue;
        }

        // Les éléments les plus profonds d'abord.
        //
        // Exemple :
        //
        // .Dossier/.SousDossier/.image.jpg
        //
        // devient :
        //
        // Dossier/SousDossier/image.jpg
        //
        // Il faut donc traiter image.jpg avant les dossiers parents.
        elements.sort_by_key(|p| std::cmp::Reverse(p.components().count()));

        for hidden_path in elements {
            let nom = match hidden_path.file_name().and_then(|n| n.to_str()) {
                Some(nom) => nom,
                None => continue,
            };

            // On ne touche qu'aux noms commençant par "."
            if !nom.starts_with('.') {
                continue;
            }

            // Le manifeste est utilisé comme fichier interne.
            // On ne veut surtout pas le renommer.
            if nom == MANIFEST_NAME {
                continue;
            }

            // "." ou ".."
            if nom == "." || nom == ".." {
                continue;
            }

            // Retire uniquement le premier "."
            let nom_original = &nom[1..];

            if nom_original.is_empty() {
                continue;
            }

            let parent = match hidden_path.parent() {
                Some(parent) => parent,
                None => continue,
            };

            let original_path = parent.join(nom_original);

            // Un élément portant déjà le nom original existe.
            if original_path.exists() {
                errors.push(format!(
                    "CONFLIT : {} -> {} existe déjà",
                    hidden_path.display(),
                    original_path.display()
                ));
                continue;
            }

            match fs::rename(&hidden_path, &original_path) {
                Ok(_) => {
                    restored += 1;

                    println!(
                        "RESTAURÉ : {} -> {}",
                        hidden_path.display(),
                        original_path.display()
                    );
                }

                Err(e) => {
                    errors.push(format!(
                        "Impossible de restaurer {} : {}",
                        hidden_path.display(),
                        e
                    ));
                }
            }
        }

        // Le manifeste n'est plus nécessaire.
        // On le supprime simplement à la fin.
        if let Err(err) = remove_manifest(&root) {
            // On ignore seulement l'absence du manifeste.
            if !err.contains("introuvable") {
                errors.push(err);
            }
        }
    }

    if restored == 0 && errors.is_empty() {
        return Ok(
            "Aucun fichier ou dossier caché à restaurer.".to_string()
        );
    }

    let mut message = format!(
        "{} éléments restaurés.",
        restored
    );

    if !errors.is_empty() {
        message.push_str("\n\nErreurs :\n");
        message.push_str(&errors.join("\n"));
    }

    Ok(message)
}


// ═══════════════════════════════════════════════════════════════
// PARCOURS POUR LE MASQUAGE
// ═══════════════════════════════════════════════════════════════
//
// On ignore les éléments déjà cachés.
// Cela empêche le programme de transformer :
//
// photo.jpg
// → .photo.jpg
// → ..photo.jpg
// → ...photo.jpg
//
// ═══════════════════════════════════════════════════════════════

fn parcourir_visible(
    dir: &Path,
    elements: &mut Vec<PathBuf>,
) -> Result<(), String> {

    let entries = match fs::read_dir(dir) {
        Ok(entries) => entries,

        Err(err) if err.kind() == std::io::ErrorKind::NotFound => {
            return Ok(());
        }

        Err(err) => {
            return Err(format!(
                "Impossible de lire {} : {}",
                dir.display(),
                err
            ));
        }
    };

    for entry in entries {
        let entry = match entry {
            Ok(entry) => entry,

            Err(err) => {
                return Err(format!(
                    "Impossible de lire un élément dans {} : {}",
                    dir.display(),
                    err
                ));
            }
        };

        let path = entry.path();

        let nom = match path.file_name().and_then(|n| n.to_str()) {
            Some(nom) => nom,
            None => continue,
        };

        // Ignore les éléments déjà cachés.
        if nom.starts_with('.') {
            continue;
        }

        if path.is_dir() {
            parcourir_visible(&path, elements)?;
        }

        elements.push(path);
    }

    Ok(())
}


// ═══════════════════════════════════════════════════════════════
// PARCOURS COMPLET POUR LE DÉMASQUAGE
// ═══════════════════════════════════════════════════════════════
//
// CONTRAIREMENT À parcourir_visible() :
//
// .Dossier
// └── .image.jpg
//
// est entièrement parcouru.
//
// ═══════════════════════════════════════════════════════════════

fn parcourir_tous(
    dir: &Path,
    elements: &mut Vec<PathBuf>,
) -> Result<(), String> {

    let entries = match fs::read_dir(dir) {
        Ok(entries) => entries,

        Err(err) if err.kind() == std::io::ErrorKind::NotFound => {
            return Ok(());
        }

        Err(err) => {
            return Err(format!(
                "Impossible de lire {} : {}",
                dir.display(),
                err
            ));
        }
    };

    for entry in entries {
        let entry = match entry {
            Ok(entry) => entry,

            Err(err) => {
                return Err(format!(
                    "Impossible de lire un élément dans {} : {}",
                    dir.display(),
                    err
                ));
            }
        };

        let path = entry.path();

        let nom = match path.file_name().and_then(|n| n.to_str()) {
            Some(nom) => nom,
            None => continue,
        };

        // Le manifeste est ignoré.
        if nom == MANIFEST_NAME {
            continue;
        }

        // Si c'est un dossier, on y entre même s'il est caché.
        if path.is_dir() {
            parcourir_tous(&path, elements)?;
        }

        elements.push(path);
    }

    Ok(())
}


// ═══════════════════════════════════════════════════════════════
// MANIFEST
// ═══════════════════════════════════════════════════════════════

fn manifest_path(root: &Path) -> PathBuf {
    root.join(MANIFEST_NAME)
}


fn save_manifest(
    root: &Path,
    entries: &[HiddenEntry],
) -> Result<(), String> {

    let path = manifest_path(root);

    let file = fs::File::create(&path)
        .map_err(|e| {
            format!(
                "Impossible d'écrire le manifeste {} : {}",
                path.display(),
                e
            )
        })?;

    serde_json::to_writer_pretty(file, entries)
        .map_err(|e| {
            format!(
                "Impossible de sérialiser le manifeste {} : {}",
                path.display(),
                e
            )
        })
}


fn remove_manifest(root: &Path) -> Result<(), String> {

    let path = manifest_path(root);

    match fs::remove_file(&path) {
        Ok(_) => Ok(()),

        Err(e) if e.kind() == std::io::ErrorKind::NotFound => Ok(()),

        Err(e) => Err(format!(
            "Impossible de supprimer le manifeste {} : {}",
            path.display(),
            e
        )),
    }
}


// ═══════════════════════════════════════════════════════════════
// TAURI
// ═══════════════════════════════════════════════════════════════

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
