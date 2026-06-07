//! Core logic for a todo list with JSON persistence.
//!
//! Design notes:
//! * The list logic (`TodoList`) knows nothing about files — it is a
//!   pure data structure, trivially testable.
//! * Persistence is two small functions at the edge. Serde derives do
//!   the heavy lifting: the structs ARE the file format.
//! * Items get stable ids so "done 3" still means the same item after
//!   other items are removed.

use std::fs;
use std::io;
use std::path::Path;

use serde::{Deserialize, Serialize};

/// Serialize/Deserialize are derived: serde generates the conversion
/// code from the struct shape at compile time. Renaming a field here
/// changes the JSON schema — with old saved files that is a breaking
/// change (see `#[serde(default)]` below for the escape hatch).
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct Item {
    pub id: u64,
    pub title: String,
    pub done: bool,
}

#[derive(Debug, Default, Serialize, Deserialize)]
pub struct TodoList {
    /// Next id to hand out; never reused, even after removals.
    /// `#[serde(default)]` makes files written before this field
    /// existed still load (missing field -> 0 -> fixed up in `load`).
    #[serde(default)]
    next_id: u64,
    items: Vec<Item>,
}

impl TodoList {
    pub fn new() -> Self {
        TodoList {
            next_id: 1,
            items: Vec::new(),
        }
    }

    /// Returns the id so callers can refer to the new item.
    pub fn add(&mut self, title: impl Into<String>) -> u64 {
        let id = self.next_id;
        self.next_id += 1;
        self.items.push(Item {
            id,
            title: title.into(),
            done: false,
        });
        id
    }

    /// Option, not panic: "no such id" is an expected condition the
    /// CLI turns into a user-facing message.
    pub fn mark_done(&mut self, id: u64) -> Option<&Item> {
        let item = self.items.iter_mut().find(|item| item.id == id)?;
        item.done = true;
        Some(item) // reborrow as shared for the caller
    }

    /// Returns the removed item — ownership moves out to the caller,
    /// which is exactly what "remove" should mean.
    pub fn remove(&mut self, id: u64) -> Option<Item> {
        let index = self.items.iter().position(|item| item.id == id)?;
        Some(self.items.remove(index))
    }

    pub fn items(&self) -> &[Item] {
        &self.items
    }

    pub fn pending(&self) -> impl Iterator<Item = &Item> {
        self.items.iter().filter(|item| !item.done)
    }
}

/// Errors from the persistence edge: either the file system failed or
/// the file held invalid JSON. Two variants, two very different user
/// remedies — which is why they stay distinct.
#[derive(Debug)]
pub enum StorageError {
    Io(io::Error),
    Format(serde_json::Error),
}

impl std::fmt::Display for StorageError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            StorageError::Io(e) => write!(f, "file error: {e}"),
            StorageError::Format(e) => write!(f, "corrupt todo file: {e}"),
        }
    }
}

impl std::error::Error for StorageError {}

// The From impls let load/save use `?` on both io and serde results.
impl From<io::Error> for StorageError {
    fn from(e: io::Error) -> Self {
        StorageError::Io(e)
    }
}

impl From<serde_json::Error> for StorageError {
    fn from(e: serde_json::Error) -> Self {
        StorageError::Format(e)
    }
}

/// A missing file is NOT an error — it just means a fresh list. This
/// is the "first run" experience, handled by matching on the io error
/// kind instead of bubbling it up.
pub fn load(path: &Path) -> Result<TodoList, StorageError> {
    let json = match fs::read_to_string(path) {
        Ok(json) => json,
        Err(e) if e.kind() == io::ErrorKind::NotFound => return Ok(TodoList::new()),
        Err(e) => return Err(e.into()),
    };
    let mut list: TodoList = serde_json::from_str(&json)?;
    // Repair files saved before next_id existed (serde defaulted it
    // to 0): never hand out an id twice.
    let max_id = list.items.iter().map(|i| i.id).max().unwrap_or(0);
    list.next_id = list.next_id.max(max_id + 1);
    Ok(list)
}

pub fn save(list: &TodoList, path: &Path) -> Result<(), StorageError> {
    // Pretty-printing keeps the file diffable and hand-editable —
    // worth the extra bytes for a personal tool.
    let json = serde_json::to_string_pretty(list)?;
    fs::write(path, json)?;
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn add_assigns_sequential_ids() {
        let mut list = TodoList::new();
        assert_eq!(list.add("buy milk"), 1);
        assert_eq!(list.add("learn rust"), 2);
        assert_eq!(list.items().len(), 2);
    }

    #[test]
    fn done_and_remove_use_stable_ids() {
        let mut list = TodoList::new();
        let milk = list.add("buy milk");
        let rust = list.add("learn rust");

        list.remove(milk);
        // `rust`'s id still works even though indices shifted.
        assert!(list.mark_done(rust).is_some());
        assert!(list.items()[0].done);
    }

    #[test]
    fn missing_ids_are_none_not_panic() {
        let mut list = TodoList::new();
        assert!(list.mark_done(99).is_none());
        assert!(list.remove(99).is_none());
    }

    #[test]
    fn ids_are_never_reused() {
        let mut list = TodoList::new();
        let first = list.add("a");
        list.remove(first);
        let second = list.add("b");
        assert_ne!(first, second);
    }

    #[test]
    fn pending_filters_out_done_items() {
        let mut list = TodoList::new();
        let a = list.add("a");
        list.add("b");
        list.mark_done(a);
        let pending: Vec<_> = list.pending().map(|i| i.title.as_str()).collect();
        assert_eq!(pending, ["b"]);
    }

    #[test]
    fn save_load_round_trip() {
        let path = std::env::temp_dir().join("todo_cli_test_round_trip.json");

        let mut list = TodoList::new();
        list.add("persisted");
        let done_id = list.add("and done");
        list.mark_done(done_id);
        save(&list, &path).expect("save");

        let loaded = load(&path).expect("load");
        assert_eq!(loaded.items(), list.items());
        // After loading, new ids must continue, not collide.
        assert_eq!(loaded.next_id, list.next_id);

        let _ = std::fs::remove_file(&path); // tidy up; failure is fine
    }

    #[test]
    fn loading_a_missing_file_gives_a_fresh_list() {
        let path = std::env::temp_dir().join("todo_cli_test_does_not_exist.json");
        let _ = std::fs::remove_file(&path);
        let list = load(&path).expect("missing file is not an error");
        assert!(list.items().is_empty());
    }

    #[test]
    fn corrupt_files_are_reported_not_swallowed() {
        let path = std::env::temp_dir().join("todo_cli_test_corrupt.json");
        std::fs::write(&path, "not json at all").expect("write");
        assert!(matches!(load(&path), Err(StorageError::Format(_))));
        let _ = std::fs::remove_file(&path);
    }
}

// Exercises
// ---------
// 1. Add a `priority: Priority` enum field (Low/Medium/High) with
//    `#[serde(default)]` so existing files keep loading.
// 2. Add `fn clear_done(&mut self) -> usize` returning how many items
//    were removed (hint: retain + a length comparison).
// 3. Make `save` atomic: write to a temp file, then rename over the
//    original. What failure does this protect against?
