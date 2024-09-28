use std::collections::HashMap;
use std::sync::{Arc, Mutex};

// Define the Storage trait
pub trait Storage: Send + Sync {
    fn store(&mut self, key: String, value: String) -> Result<(), String>;
    fn retrieve(&self, key: &str) -> Option<String>;
}

// Implement an in-memory storage solution
pub struct MemoryStorage {
    data: HashMap<String, String>,
}

impl MemoryStorage {
    pub fn new() -> Self {
        MemoryStorage {
            data: HashMap::new(),
        }
    }
}

impl Storage for MemoryStorage {
    fn store(&mut self, key: String, value: String) -> Result<(), String> {
        self.data.insert(key, value);
        Ok(())
    }

    fn retrieve(&self, key: &str) -> Option<String> {
        self.data.get(key).cloned()
    }
}

// Create a type alias for convenience
pub type SharedStorage = Arc<Mutex<dyn Storage>>;

// Function to create a new shared storage instance
pub fn new_shared_storage() -> SharedStorage {
    Arc::new(Mutex::new(MemoryStorage::new()))
}
