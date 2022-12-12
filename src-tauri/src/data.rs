use serde::{Deserialize, Serialize};
use serde_json::Value;
use std::collections::HashMap;
use std::fs::{create_dir, read_dir, read_to_string};
use std::path::PathBuf;
use std::sync::{Arc, Mutex};
use tauri::{Config, Runtime, State};

use crate::core::utils::fs::write_atomically;
use crate::core::utils::json::to_json;

pub struct AppData {
    pub app_dir: PathBuf,
    pub data_dir: PathBuf,
}

impl AppData {
    pub fn initialize_from_config(config_file: &Config) -> Self {
        let app_dir = tauri::api::path::app_data_dir(config_file).unwrap();

        AppData {
            app_dir: app_dir.clone(),
            data_dir: app_dir.join("data"),
        }
    }
}

#[derive(Serialize, Debug, Deserialize, Default, Clone)]
#[serde(rename_all = "camelCase")]
pub struct NoteFile {
    pub file_path: PathBuf,
    pub content: String,
}

impl NoteFile {
    pub fn new(file_path: PathBuf, content: String) -> NoteFile {
        NoteFile { file_path, content }
    }
    // Load the note file from disk (currently unused)
    pub fn load(&mut self) -> Result<Self, String> {
        let note = match std::fs::read_to_string(&self.file_path) {
            Ok(note_str) => {
                let note_file: NoteFile = match serde_json::from_str(&note_str) {
                    Ok(note) => note,
                    Err(e) => throw!("Could not parse reminders file: {}", e),
                };
                note_file
            }
            Err(e) => match e.kind() {
                _ => throw!("{}", e.to_string()),
            },
        };
        Ok(note)
    }
    // Save the note file to disk and update self
    pub fn save(&self, buf: &[u8]) -> Result<(), String> {
        match write_atomically(&self.file_path, &buf) {
            Ok(_) => {}
            Err(e) => throw!("File save error: {}", e.to_string()),
        }

        Ok(())
    }
}

pub trait KV {
    fn set(&mut self, key: PathBuf, content: String);

    fn get(&mut self, key: PathBuf) -> Option<NoteFile>;
    fn get_all(&self) -> Vec<NoteFile>;

    fn has_key(&self, key: PathBuf) -> bool;
}

#[derive(Default, Serialize, Deserialize, Clone)]
pub struct Notes {
    pub entries: HashMap<PathBuf, Option<NoteFile>>,
}

impl Notes {
    // Initialize Notes without reading the data directory
    pub fn new() -> Self {
        Self {
            entries: HashMap::<PathBuf, Option<NoteFile>>::new(),
        }
    }
    // Initialize Notes from the data directory
    pub fn new_from_data_dir(data_path: &PathBuf) -> Self {
        let mut entries = HashMap::new();
        for entry in read_dir(data_path).unwrap() {
            if let Ok(e) = entry {
                let content = read_to_string(&e.path()).unwrap();
                entries.insert(
                    e.path().to_owned(),
                    Some(NoteFile {
                        file_path: e.path(),
                        content: content.to_string(),
                    }),
                );
            };
        }
        Self { entries: entries }
    }
    // Insert or update a note into the HashMap
    pub fn insert(&mut self, key: PathBuf, content: &str) {
        match self.entries.contains_key(&key) {
            true => {
                let data = self.entries.get_mut(&key).unwrap();
                let mut note = &mut data.as_ref().unwrap();
                note.save(&content.as_bytes()).unwrap();
            }
            false => {
                let new_note = NoteFile {
                    file_path: key.clone(),
                    content: content.to_string(),
                };
                new_note
                    .save(&content.as_bytes())
                    .expect("Error saving newly inserted note");

                self.entries.insert(key, Some(new_note));
            }
        };
    }
}

impl KV for Notes {
    fn set(&mut self, key: PathBuf, content: String) {
        self.entries
            .insert(key.clone(), Some(NoteFile::new(key.clone(), content)));
    }

    fn get(&mut self, key: PathBuf) -> Option<NoteFile> {
        let result = self.entries.get(&key).unwrap();
        result.to_owned()
    }
    fn has_key(&self, key: PathBuf) -> bool {
        let key_exists = self.entries.contains_key(&key);
        key_exists
    }

    fn get_all(&self) -> Vec<NoteFile> {
        let result = self.entries.values().map(|e| e.clone().unwrap()).collect();

        result
    }
}

#[derive(Default)]
pub struct Store {
    pub data_path: PathBuf,
    notes: Arc<Mutex<Notes>>,
}

impl Store {
    pub fn new(data_path: AppData) -> Store {
        if data_path.data_dir.is_dir() {
            Store {
                data_path: data_path.data_dir.clone(),
                notes: Arc::new(Mutex::new(Notes::new_from_data_dir(&data_path.data_dir))),
            }
        } else {
            create_dir(data_path.data_dir.clone()).unwrap();
            Store {
                data_path: data_path.data_dir.clone(),
                notes: Arc::new(Mutex::new(Notes::new())),
            }
        }
    }

    pub fn set(&self, key: &PathBuf, content: String) {
        let mut data = self.notes.lock().unwrap();
        data.insert(key.clone(), &content.clone());
    }

    pub fn get(&self, key: &PathBuf) -> Option<NoteFile> {
        let mut data = self.notes.lock().unwrap();
        let note = data.get(key.to_path_buf());
        note
    }

    pub fn has_key(&self, key: &PathBuf) -> bool {
        let data = self.notes.lock().unwrap();
        let key_exists = match data.has_key(self.data_path.join(key.to_path_buf())) {
            true => true,
            false => false,
        };
        key_exists
    }

    pub fn get_all(&self) -> Vec<NoteFile> {
        let lock = self.notes.lock();
        let data = lock.unwrap();

        let result: Vec<NoteFile> = data.get_all();
        result
    }
}

pub struct Data(pub Mutex<Store>);

#[tauri::command]
pub fn save_file<R: Runtime>(
    app: tauri::AppHandle<R>,
    file_name: PathBuf,
    content: String,
    data: State<'_, Data>,
) -> Result<Value, String> {
    let cache = data.0.lock().unwrap();

    match cache.has_key(&file_name) {
        true => {
            cache
                .get(&file_name)
                .unwrap()
                .save(&content.as_bytes())
                .unwrap();
        }
        false => {
            cache.set(&file_name, content.clone());
        }
    };

    to_json(&cache.get_all())
}

#[tauri::command]
pub fn get_files<R: Runtime>(
    app: tauri::AppHandle<R>,
    data: State<'_, Data>,
) -> Result<Value, String> {
    let cache = data.0.lock().unwrap();

    to_json(&cache.get_all())
}
