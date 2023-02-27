#![cfg_attr(
    all(not(debug_assertions), target_os = "windows"),
    windows_subsystem = "windows"
    )]
    
use std::fs::File;
use std::io;
use std::path::{Path, PathBuf};
use std::env;
use std::fs;
use zip_extensions;

fn find_ksp2_install_path() -> Option<PathBuf> {

    // Look for the game in Steam library folders
    let mut steam_install_folder = PathBuf::from(env::var("ProgramFiles(x86)").unwrap())
        .join("Steam")
        .join("steamapps")
        .join("common")
        .join("Kerbal Space Program 2");

    // Look for the game in default installation path
    if !steam_install_folder.exists() {
        let default_install_folder = Path::new(env::var("ProgramFiles").unwrap().as_str())
            .join("Private Division")
            .join("Kerbal Space Program 2");

        if default_install_folder.exists() {
            steam_install_folder = default_install_folder.to_path_buf();
        }
    }

    if steam_install_folder.exists() {
        Some(steam_install_folder)
    } else {
        None
    }
}

async fn get_latest_release() -> String {
    let client = reqwest::Client::new();
    let res = client
        .get("https://api.github.com/repos/X606/SpaceWarp/releases/latest")
        .header(reqwest::header::USER_AGENT, "spacewarpsetup-script-sinon")
        .send()
        .await
        .expect("failed to get response")
        .text()
        .await
        .expect("failed to get payload");
    let json: serde_json::Value =
        serde_json::from_str(&res).expect("JSON was not well-formatted");
    let json_assets = json.get("assets").unwrap().to_string();

    let mut zip_url = "".to_string();

    for asset in serde_json::from_str::<Vec<serde_json::Value>>(&json_assets).unwrap() {
        if asset.get("browser_download_url").unwrap().to_string().contains("space-warp-release") {
            if asset.get("browser_download_url").unwrap().to_string().contains("zip") {
                zip_url = asset.get("browser_download_url").unwrap().to_string();
            }
        }
    }
    return zip_url.to_string();
}

async fn get_latest_bepinex_release() -> String {
    let client = reqwest::Client::new();
    let res = client
        .get("https://api.github.com/repos/X606/SpaceWarp/releases/latest")
        .header(reqwest::header::USER_AGENT, "spacewarpsetup-script-sinon")
        .send()
        .await
        .expect("failed to get response")
        .text()
        .await
        .expect("failed to get payload");
    let json: serde_json::Value =
        serde_json::from_str(&res).expect("JSON was not well-formatted");
    let json_assets = json.get("assets").unwrap().to_string();

    let mut zip_url = "".to_string();

    for asset in serde_json::from_str::<Vec<serde_json::Value>>(&json_assets).unwrap() {
        if asset.get("browser_download_url").unwrap().to_string().contains("bepinex.zip") {
            zip_url = asset.get("browser_download_url").unwrap().to_string();
        }
    }
    return zip_url.to_string();
}



fn rem_first_and_last(value: &str) -> &str {
    let mut chars = value.chars();
    chars.next();
    chars.next_back();
    chars.as_str()
}

#[tauri::command]
async fn download_file(ksp_given_path: Option<String>) -> String {

    println!("KSP Path: {:?}", ksp_given_path);

    let ksp_path = match ksp_given_path {
        Some(path) => PathBuf::from(path),
        None => match find_ksp2_install_path() {
            Some(path) => path,
            None => {
                return "notfound".to_string()
            }
        }
    };

    if !ksp_path.join("KSP2_x64.exe").exists() {
        return "not-valid".to_string();
    }

    if ksp_path.join("SpaceWarp").exists() {
        return "exists".to_string();
    }

    let latest_release_url = get_latest_release().await;

    let mut latest_release_url_str = latest_release_url.as_str();
    
    latest_release_url_str = rem_first_and_last(latest_release_url_str);

    println!("Downloading from: {}", latest_release_url_str);

    let resp = reqwest::get(latest_release_url_str).await.expect("request failed");

    let body = resp.bytes().await.expect("body invalid");
    let mut out = File::create(ksp_path.join("space_warp_temp.zip")).expect("failed to create file");
    io::copy(&mut body.as_ref(), &mut out).expect("failed to copy content");

    let _ = zip_extensions::read::zip_extract(
        &PathBuf::from(&ksp_path.join("space_warp_temp.zip")),
        &PathBuf::from(&ksp_path),
    );

    fs::remove_file(&ksp_path.join("space_warp_temp.zip")).expect("failed to delete file");

    fs::create_dir(&ksp_path.join("SpaceWarp").join("Mods")).expect("failed to create directory");

    return "Success".to_string();
}

#[tauri::command]
async fn download_bepinex(ksp_given_path: Option<String>) -> String {

    println!("KSP Path: {:?}", ksp_given_path);

    let ksp_path = match ksp_given_path {
        Some(path) => PathBuf::from(path),
        None => match find_ksp2_install_path() {
            Some(path) => path,
            None => {
                return "notfound".to_string()
            }
        }
    };

    if !ksp_path.join("BepInEx").exists() {
        return "notbep".to_string();
    }

    if !ksp_path.join("KSP2_x64.exe").exists() {
        return "not-valid".to_string();
    }

    let latest_release_url = get_latest_bepinex_release().await;
    let mut latest_release_url_str = latest_release_url.as_str();
    latest_release_url_str = rem_first_and_last(latest_release_url_str);

    println!("Downloading from: {}", latest_release_url_str);

    let resp = reqwest::get(latest_release_url_str).await.expect("request failed");
    let body = resp.bytes().await.expect("body invalid");
    let mut out = File::create(ksp_path.join("space_warp_bepinex_temp.zip")).expect("failed to create file");
    io::copy(&mut body.as_ref(), &mut out).expect("failed to copy content");

    let _ = zip_extensions::read::zip_extract(
        &PathBuf::from(&ksp_path.join("space_warp_bepinex_temp.zip")),
        &PathBuf::from(&ksp_path),
    );

    fs::remove_file(&ksp_path.join("space_warp_bepinex_temp.zip")).expect("failed to delete file");
    return "Success".to_string();
}

#[tauri::command]
async fn uninstall() -> String {

    let ksp_path = match find_ksp2_install_path() {
        Some(path) => path,
        None => {
            return "KSP2 path not found\nGo delete it yourself :3".to_string()
        }
    };

    fs::remove_dir_all(&ksp_path.join("SpaceWarp")).expect("failed to delete directory");
    fs::remove_file(&ksp_path.join("winhttp.dll")).expect("failed to delete file");
    fs::remove_file(&ksp_path.join("doorstop_config.ini")).expect("failed to delete file");
    fs::remove_file(&ksp_path.join(".doorstop_version")).expect("failed to delete file");

    println!("KSP Path: {:?}", ksp_path);

    return "Success".to_string();

}

#[tauri::command]
async fn uninstall_bepinex() -> String {

    let ksp_path = match find_ksp2_install_path() {
        Some(path) => path,
        None => {
            return "KSP2 path not found\nGo delete it yourself :3".to_string()
        }
    };

    fs::remove_dir_all(&ksp_path.join("BepInEx").join("plugins").join("SpaceWarp")).expect("failed to delete directory");

    println!("KSP Path: {:?}", ksp_path);

    return "Success".to_string();
}

fn main() {
    tauri::Builder::default()
        .invoke_handler(tauri::generate_handler![download_file, uninstall, download_bepinex, uninstall_bepinex])
        .run(tauri::generate_context!())
        .expect("error while running tauri application");
}
    