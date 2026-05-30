use std::collections::HashMap;

/// Intern table for deduplicating short strings (APID, CTID, ECU).
///
/// ID 0 is reserved for "absent" (empty string).
pub struct InternTable {
    map: HashMap<String, u16>,
    strings: Vec<String>,
}

impl InternTable {
    pub fn new() -> Self {
        let mut table = Self {
            map: HashMap::new(),
            strings: vec![String::new()], // ID 0 = absent
        };
        table.map.insert(String::new(), 0);
        table
    }

    /// Insert a string and return its interned ID.
    /// Returns the existing ID if the string was already interned.
    pub fn insert(&mut self, s: &str) -> u16 {
        if let Some(&id) = self.map.get(s) {
            return id;
        }
        let id = self.strings.len() as u16;
        self.strings.push(s.to_string());
        self.map.insert(s.to_string(), id);
        id
    }

    /// Resolve an interned ID back to its string.
    pub fn resolve(&self, id: u16) -> &str {
        &self.strings[id as usize]
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn id_zero_is_empty_string() {
        let table = InternTable::new();
        assert_eq!(table.resolve(0), "");
    }

    #[test]
    fn insert_and_resolve_roundtrip() {
        let mut table = InternTable::new();
        let id = table.insert("APP1");
        assert_eq!(table.resolve(id), "APP1");
    }

    #[test]
    fn duplicate_insert_returns_same_id() {
        let mut table = InternTable::new();
        let id1 = table.insert("APP1");
        let id2 = table.insert("APP1");
        assert_eq!(id1, id2);
    }

    #[test]
    fn different_strings_get_different_ids() {
        let mut table = InternTable::new();
        let id1 = table.insert("APP1");
        let id2 = table.insert("CTX1");
        assert_ne!(id1, id2);
    }

    #[test]
    fn empty_string_returns_id_zero() {
        let mut table = InternTable::new();
        let id = table.insert("");
        assert_eq!(id, 0);
    }

    #[test]
    fn first_non_empty_insert_gets_id_one() {
        let mut table = InternTable::new();
        let id = table.insert("ECU1");
        assert_eq!(id, 1);
    }
}
