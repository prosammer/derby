use std::path::PathBuf;
use serde_json::Value;
use tauri::{AppHandle, Manager, Wry};
use tauri_plugin_store::{StoreCollection, with_store};

pub fn get_from_store(handle: AppHandle, key: &str) -> Option<String>{
    let stores = handle.state::<StoreCollection<Wry>>();
    let path = PathBuf::from(".settings.dat");

    let mut retrieved: Option<String> = None;

    with_store(handle.clone(), stores, path, |store| {
        if let Some(stored_value) = store.get(key) {
            println!("Retrieved value from store: {}", stored_value);
            retrieved = Some(stored_value.to_string());
        } else {
            println!("Key not found in store");
        }
        Ok(())
    }).expect("Failed to interact with the store");

    retrieved
}

pub fn set_in_store(handle: AppHandle, key: String, value: Value) {
    let stores = handle.state::<StoreCollection<Wry>>();
    let path = PathBuf::from(".settings.dat");

    with_store(handle.clone(), stores, path, |store| {
        store.insert(key, value)
    }).expect("Failed to interact with the store");
}