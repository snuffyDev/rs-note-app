use crate::core::utils::fs::write_atomically;
use crate::core::utils::json::to_json;

use serde::{Deserialize, Serialize};
use serde_json::Value;
use std::collections::HashMap;
use std::fs::{create_dir, read_dir};
use std::path::PathBuf;
use std::sync::{Arc, Mutex};
use tauri::{Config, State};
use uuid::Uuid;

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

pub enum InsertKind {
    Uuid(Uuid),
    String(String),
}

// #[serde(rename_all = "camelCase")]
#[derive(Serialize, Debug, Deserialize, Clone)]
pub struct NoteFile {
    pub file_path: PathBuf,
    pub uuid: Option<Uuid>,
    pub content: String,
}

impl NoteFile {
    pub fn new(file_path: &PathBuf, content: &String) -> Self {
        let uuid = Uuid::new_v4();
        Self {
            file_path: file_path
                .join(format!("{}.json", uuid.clone().to_string()))
                .to_path_buf(),
            content: content.to_string(),
            uuid: Some(uuid),
        }
    }
    // Load the note file from disk (currently unused)
    pub fn load(path: &PathBuf) -> Result<Self, String> {
        let note = match std::fs::read_to_string(path) {
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
    pub fn save(&mut self, buf: &[u8]) -> Result<Self, String> {
        self.content = String::from_utf8(buf.to_vec()).unwrap();
        match write_atomically(&self.file_path.to_path_buf(), to_json(self).unwrap()) {
            Ok(_) => {}
            Err(e) => throw!("File save error: {}", e.to_string()),
        }

        Ok(self.to_owned())
    }
}

impl Default for NoteFile {
    fn default() -> Self {
        Self {
            file_path: Default::default(),
            uuid: Default::default(),
            content: Default::default(),
        }
    }
}

pub trait KV {
    fn set(&mut self, uuid: InsertKind, content: &String);

    fn get(&self, uuid: Option<Uuid>) -> Option<NoteFile>;
    fn get_all(&self) -> Vec<NoteFile>;

    fn has_key(&self, uuid: &Option<Uuid>) -> bool;
}

#[derive(Serialize, Debug, Deserialize, Clone)]
pub struct Notes {
    pub data_path: PathBuf,
    pub entries: HashMap<Uuid, NoteFile>,
}

impl Notes {
    // Initialize Notes without reading the data directory
    pub fn new(data_path: &PathBuf) -> Notes {
        Self {
            entries: HashMap::<Uuid, NoteFile>::new(),
            data_path: data_path.to_path_buf(),
        }
    }
    // Initialize Notes from the data directory
    pub fn new_from_data_dir(data_path: &PathBuf) -> Notes {
        let mut entries = HashMap::new();
        for entry in read_dir(data_path).unwrap() {
            if let Ok(e) = entry {
                if let Ok(note) = NoteFile::load(&e.path()) {
                    entries.insert(note.uuid.unwrap().to_owned(), note);
                }
                {
                    eprintln!("ERROR! LOAD FROM DIR!!!");
                }
            };
        }
        Self {
            entries: entries,
            data_path: data_path.to_path_buf(),
        }
    }
    // Insert or update a note into the HashMap
    pub fn insert(&mut self, key: InsertKind, content: &str) {
        match key {
            InsertKind::Uuid(uuid) => {
                if self.entries.contains_key(&uuid) {
                    let entry = self.entries.entry(uuid).or_default();
                    *entry = entry.save(&content.as_bytes()).unwrap()
                };
            }
            InsertKind::String(_title) => {
                let content_bytes = &content.as_bytes();
                let mut new_note =
                    NoteFile::new(&self.data_path.to_path_buf(), &content.to_string());

                new_note
                    .save(content_bytes)
                    .expect("Error saving newly inserted note");

                self.entries
                    .insert(new_note.uuid.unwrap().to_owned(), new_note);
            }
        }
    }
}

impl KV for Notes {
    fn set(&mut self, uuid: InsertKind, content: &String) {
        match uuid {
            InsertKind::Uuid(uuid) => {
                self.entries
                    .insert(uuid, NoteFile::new(&self.data_path.to_path_buf(), content));
            }

            InsertKind::String(_string) => {}
        }
    }

    fn get(&self, uuid: Option<Uuid>) -> std::option::Option<NoteFile> {
        let result = self.entries.get(&uuid.unwrap()).unwrap();
        Some(result.to_owned())
    }

    fn get_all(&self) -> Vec<NoteFile> {
        let result = self.entries.values().map(|e| e.to_owned()).collect();
        result
    }

    fn has_key(&self, uuid: &Option<Uuid>) -> bool {
        if let Some(uuid) = uuid {
            let key_exists = self.entries.contains_key(&uuid);
            return key_exists;
        };
        false
    }
}

#[derive(Debug)]
pub struct Store {
    pub data_path: PathBuf,
    notes: Arc<Mutex<Notes>>,
}

impl Store {
    pub fn new(data_path: AppData) -> Store {
        if data_path.data_dir.is_dir() {
            Self {
                data_path: data_path.data_dir.clone(),
                notes: Arc::new(Mutex::new(Notes::new_from_data_dir(&data_path.data_dir))),
            }
        } else {
            create_dir(data_path.data_dir.clone()).unwrap();
            Self {
                data_path: data_path.data_dir.clone(),
                notes: Arc::new(Mutex::new(Notes::new(&data_path.data_dir))),
            }
        }
    }

    pub fn set(&self, key: InsertKind, content: String) {
        let mut data = self.notes.lock().unwrap();
        data.insert(key, &content.clone());
    }

    pub fn set_new(&self, key: String, content: String) {
        let mut data = self.notes.lock().unwrap();
        data.insert(InsertKind::String(key), &content);
    }

    pub fn get(&self, key: Option<Uuid>) -> NoteFile {
        let data = self.notes.lock().unwrap();
        let note = &data.get(key.to_owned()).unwrap();
        note.to_owned()
    }

    pub fn has_key(&self, key: Option<Uuid>) -> bool {
        let data = self.notes.lock().unwrap();
        let key_exists = match data.has_key(&key) {
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
pub fn save_file(
    file_name: String,
    content: String,
    uuid: Option<Uuid>,
    data: State<'_, Data>,
) -> Result<Value, String> {
    let cache = data.0.lock().unwrap();

    match cache.has_key(uuid) {
        true => cache.set(InsertKind::Uuid(uuid.unwrap()), content),
        _ => {
            cache.set(InsertKind::String(file_name), content.clone());
        }
    };

    to_json(&cache.get_all())
}

#[tauri::command]
pub fn get_files(data: State<'_, Data>) -> Result<Value, String> {
    let cache = data.0.lock().unwrap();

    to_json(&cache.get_all())
}
