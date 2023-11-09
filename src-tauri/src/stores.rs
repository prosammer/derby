use std::path::PathBuf;
use log::{error, info, warn};
use serde_json::Value;
use tauri::{AppHandle, Manager, Wry};
use tauri_plugin_store::{StoreCollection, with_store};

pub fn get_from_store(handle: &AppHandle, key: &str) -> Option<String>{
    let stores = handle.state::<StoreCollection<Wry>>();
    let path = PathBuf::from(".settings.dat");

    if path.exists() {
        info!("The settings file exists.");
    } else {
        warn!("The settings file does not exist.");
    }
    let mut retrieved: Option<String> = None;

    with_store(handle.clone(), stores, path, |store| {
        if let Some(stored_value) = store.get(key) {
            info!("Retrieved value from store: {}", stored_value);
            retrieved = Some(stored_value.to_string());
        } else {
            warn!("Key not found in store");
        }
        Ok(())
    }).expect("Failed to interact with the store");

    retrieved
}

pub fn set_in_store(handle: &AppHandle, key: String, value: Value) {
    let stores = handle.state::<StoreCollection<Wry>>();
    let path = PathBuf::from(".settings.dat");

    if path.exists() {
        info!("The settings file exists.");
    } else {
        warn!("The settings file does not exist.");
    }

    with_store(handle.clone(), stores, path, |store| {
        match store.insert(key.clone(), value) {
            Ok(_) => info!("Inserted key '{}' in store successfully.", key),
            Err(e) => error!("Failed to insert key '{}' in store: {}", key, e),
        }
        Ok(())
    }).expect("Failed to interact with the store");
}