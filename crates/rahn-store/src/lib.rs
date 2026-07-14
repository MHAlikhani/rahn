// SPDX-License-Identifier: Apache-2.0

//! Content-addressed local persistence (ADR 0004).
//!
//! Layout under the repository root (default `.rahn/`):
//!
//! ```text
//! .rahn/
//! ├── objects/<xx>/<62-hex>   write-once objects, named by SHA-256 of contents
//! ├── refs/heads/<name>       branch references: commit id (hex) + newline
//! ├── HEAD                    current branch name
//! ├── index                   working (uncommitted) state, canonical bytes
//! └── constitution            constitution requirements, one per line
//! ```
//!
//! Object integrity: on read, the file is re-hashed and MUST match its name;
//! mismatch is a loud `Corrupted` error. Silent repair is forbidden
//! (docs/spec/storage.md).

use std::fs;
use std::io;
use std::path::{Path, PathBuf};

use rahn_core::State;
use rahn_state::canonical::{canonical_bytes, parse_canonical, CanonicalError};
use rahn_state::commit::{canonical_commit_bytes, CommitId, CommitRecord};
use rahn_state::identity::StateId;
use sha2::{Digest, Sha256};

const STATE_TAG: u8 = b'S';
const COMMIT_TAG: u8 = b'C';

fn hash(data: &[u8]) -> [u8; 32] {
    let mut hasher = Sha256::new();
    hasher.update(data);
    hasher.finalize().into()
}

fn to_hex(bytes: &[u8]) -> String {
    let mut s = String::with_capacity(bytes.len() * 2);
    for b in bytes {
        s.push_str(&format!("{b:02x}"));
    }
    s
}

#[derive(Debug)]
pub enum StoreError {
    Io(io::Error),
    /// An object file's contents do not hash to its name.
    Corrupted {
        path: PathBuf,
        expected: String,
        found: String,
    },
    /// Canonical parse failure.
    Malformed(CanonicalError),
    /// A branch or ref name violates the identifier rules.
    InvalidRef(String),
    /// A referenced object of the expected type does not exist.
    NotFound(String),
}

impl std::fmt::Display for StoreError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            StoreError::Io(e) => write!(f, "storage I/O error: {e}"),
            StoreError::Corrupted {
                path,
                expected,
                found,
            } => write!(
                f,
                "CORRUPTED object {path:?}: name says {expected}, content hashes to {found}. \
                 The store refuses to load corrupted objects; do not repair silently."
            ),
            StoreError::Malformed(e) => write!(f, "malformed stored object: {e}"),
            StoreError::InvalidRef(name) => write!(f, "invalid branch name {name:?}"),
            StoreError::NotFound(what) => write!(f, "not found: {what}"),
        }
    }
}

impl std::error::Error for StoreError {}

impl From<io::Error> for StoreError {
    fn from(e: io::Error) -> Self {
        StoreError::Io(e)
    }
}

impl From<CanonicalError> for StoreError {
    fn from(e: CanonicalError) -> Self {
        StoreError::Malformed(e)
    }
}

/// The content-addressed store plus repository working files.
#[derive(Debug, Clone)]
pub struct Store {
    root: PathBuf,
}

impl Store {
    /// Create a fresh repository layout. Fails if the root already exists.
    pub fn init(root: impl AsRef<Path>) -> Result<Store, StoreError> {
        let root = root.as_ref();
        if root.exists() {
            return Err(StoreError::NotFound(format!(
                "refusing to initialize: {} already exists",
                root.display()
            )));
        }
        fs::create_dir_all(root.join("objects"))?;
        fs::create_dir_all(root.join("refs/heads"))?;
        fs::write(root.join("HEAD"), "main\n")?;
        fs::write(root.join("constitution"), "")?;
        // Empty working state.
        fs::write(root.join("index"), canonical_bytes(&State::empty()))?;
        Ok(Store {
            root: root.to_path_buf(),
        })
    }

    /// Open an existing repository.
    pub fn open(root: impl AsRef<Path>) -> Result<Store, StoreError> {
        let root = root.as_ref();
        for required in ["objects", "refs/heads", "HEAD"] {
            if !root.join(required).exists() {
                return Err(StoreError::NotFound(format!(
                    "not a rahn repository (missing {})",
                    root.join(required).display()
                )));
            }
        }
        Ok(Store {
            root: root.to_path_buf(),
        })
    }

    /// Repository root (`.rahn` directory). Read access for diagnostics.
    pub fn root(&self) -> &Path {
        &self.root
    }

    fn object_path(&self, hex: &str) -> PathBuf {
        let mut p = self.root.join("objects").join(&hex[..2]);
        p.push(&hex[2..]);
        p
    }

    fn put_object(&self, tag: u8, payload: &[u8]) -> Result<String, StoreError> {
        let mut bytes = Vec::with_capacity(payload.len() + 1);
        bytes.push(tag);
        bytes.extend_from_slice(payload);
        let hex = to_hex(&hash(&bytes));
        let path = self.object_path(&hex);
        if path.exists() {
            return Ok(hex); // write-once, content-addressed: identical is a no-op
        }
        fs::create_dir_all(path.parent().unwrap())?;
        let tmp = path.with_extension("tmp");
        fs::write(&tmp, &bytes)?;
        fs::rename(&tmp, &path)?;
        Ok(hex)
    }

    fn get_object(&self, hex: &str, expected_tag: u8) -> Result<Option<Vec<u8>>, StoreError> {
        let path = self.object_path(hex);
        if !path.exists() {
            return Ok(None);
        }
        let bytes = fs::read(&path)?;
        let found = to_hex(&hash(&bytes));
        if found != hex {
            return Err(StoreError::Corrupted {
                path,
                expected: hex.to_owned(),
                found,
            });
        }
        if bytes.first() != Some(&expected_tag) {
            return Err(StoreError::Corrupted {
                path,
                expected: format!("type tag {expected_tag}"),
                found: format!("tag {:?}", bytes.first()),
            });
        }
        Ok(Some(bytes[1..].to_vec()))
    }

    // -- states ------------------------------------------------------------

    pub fn put_state(&self, state: &State) -> Result<StateId, StoreError> {
        let hex = self.put_object(STATE_TAG, &canonical_bytes(state))?;
        Ok(StateId::from_hex(&hex).expect("store hash is valid hex"))
    }
    pub fn get_state(&self, id: StateId) -> Result<Option<State>, StoreError> {
        match self.get_object(&id.as_hex(), STATE_TAG)? {
            None => Ok(None),
            Some(payload) => Ok(Some(parse_canonical(&payload)?)),
        }
    }

    // -- commits -----------------------------------------------------------

    pub fn put_commit(&self, rec: &CommitRecord) -> Result<CommitId, StoreError> {
        let hex = self.put_object(COMMIT_TAG, &canonical_commit_bytes(rec))?;
        Ok(CommitId::from_hex(&hex).expect("store hash is valid hex"))
    }

    pub fn get_commit(&self, id: CommitId) -> Result<Option<CommitRecord>, StoreError> {
        match self.get_object(&id.as_hex(), COMMIT_TAG)? {
            None => Ok(None),
            Some(payload) => Ok(Some(CommitRecord::parse(&payload)?)),
        }
    }

    // -- refs / HEAD -------------------------------------------------------

    pub fn set_branch(&self, name: &str, commit: CommitId) -> Result<(), StoreError> {
        validate_ref_name(name).map_err(|_| StoreError::InvalidRef(name.to_owned()))?;
        let path = self.root.join("refs/heads").join(name);
        if let Some(parent) = path.parent() {
            fs::create_dir_all(parent)?;
        }
        fs::write(path, format!("{}\n", commit.as_hex()))?;
        Ok(())
    }

    pub fn get_branch(&self, name: &str) -> Result<Option<CommitId>, StoreError> {
        let path = self.root.join("refs/heads").join(name);
        match fs::read_to_string(path) {
            Ok(text) => {
                CommitId::from_hex(text.trim())
                    .map(Some)
                    .ok_or_else(|| StoreError::Corrupted {
                        path: self.root.join("refs/heads").join(name),
                        expected: "64 hex chars".to_owned(),
                        found: text.trim().to_owned(),
                    })
            }
            Err(e) if e.kind() == io::ErrorKind::NotFound => Ok(None),
            Err(e) => Err(e.into()),
        }
    }

    pub fn list_branches(&self) -> Result<Vec<String>, StoreError> {
        let mut out = Vec::new();
        let dir = self.root.join("refs/heads");
        collect_ref_names(&dir, String::new(), &mut out)?;
        out.sort();
        Ok(out)
    }

    pub fn set_head(&self, branch: &str) -> Result<(), StoreError> {
        validate_ref_name(branch).map_err(|_| StoreError::InvalidRef(branch.to_owned()))?;
        fs::write(self.root.join("HEAD"), format!("{branch}\n"))?;
        Ok(())
    }

    pub fn head(&self) -> Result<String, StoreError> {
        let text = fs::read_to_string(self.root.join("HEAD"))?;
        Ok(text.trim().to_owned())
    }

    // -- working state (index) ---------------------------------------------

    pub fn load_index(&self) -> Result<State, StoreError> {
        let bytes = fs::read(self.root.join("index"))?;
        Ok(parse_canonical(&bytes)?)
    }

    pub fn save_index(&self, state: &State) -> Result<(), StoreError> {
        Ok(fs::write(self.root.join("index"), canonical_bytes(state))?)
    }

    // -- observations (append-only log; model belongs to rahn-state) -------

    pub fn observations(&self) -> Result<rahn_state::obs::log::ObsLog, StoreError> {
        rahn_state::obs::log::ObsLog::open(&self.root).map_err(|e| {
            StoreError::Malformed(CanonicalError {
                message: e.to_string(),
                offset: 0,
            })
        })
    }

    // -- causal log (append-only; model belongs to rahn-state) -------------

    pub fn causal(&self) -> Result<rahn_state::causal_log::CausalLog, StoreError> {
        rahn_state::causal_log::CausalLog::open(&self.root).map_err(|e| {
            StoreError::Malformed(CanonicalError {
                message: e.to_string(),
                offset: 0,
            })
        })
    }

    // -- constitution (raw text; parsing belongs to rahn-verify) ------------

    pub fn load_constitution_text(&self) -> Result<String, StoreError> {
        Ok(fs::read_to_string(self.root.join("constitution"))?)
    }

    pub fn save_constitution_text(&self, text: &str) -> Result<(), StoreError> {
        Ok(fs::write(self.root.join("constitution"), text)?)
    }
}

fn collect_ref_names(dir: &Path, prefix: String, out: &mut Vec<String>) -> Result<(), StoreError> {
    for entry in fs::read_dir(dir)? {
        let entry = entry?;
        let path = entry.path();
        let name = entry.file_name().to_string_lossy().into_owned();
        if path.is_dir() {
            collect_ref_names(&path, format!("{prefix}{name}/"), out)?;
        } else {
            out.push(format!("{prefix}{name}"));
        }
    }
    Ok(())
}

/// Branch names follow identifier rules plus `/` separators (no `..`,
/// no leading/trailing or doubled slashes).
fn validate_ref_name(name: &str) -> Result<(), ()> {
    if name.is_empty() || name.len() > 128 {
        return Err(());
    }
    if name.starts_with('/') || name.ends_with('/') || name.contains("//") || name.contains("..") {
        return Err(());
    }
    for seg in name.split('/') {
        if seg.is_empty() {
            return Err(());
        }
        for ch in seg.chars() {
            let ok = ch.is_ascii_alphanumeric() || ch == '-' || ch == '_' || ch == '.';
            if !ok {
                return Err(());
            }
        }
    }
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;
    use rahn_core::{Metadata, Network};
    use rahn_state::transition::Operation;

    fn temp_root(tag: &str) -> std::path::PathBuf {
        let dir = std::env::temp_dir().join(format!(
            "rahn-store-test-{tag}-{}-{}",
            std::process::id(),
            std::time::SystemTime::now()
                .duration_since(std::time::UNIX_EPOCH)
                .unwrap()
                .as_nanos()
        ));
        dir
    }

    fn state_with(ids: &[&str]) -> State {
        let mut net = Network::empty();
        for id in ids {
            net.add_node(id, Metadata::new()).unwrap();
        }
        State { network: net }
    }

    #[test]
    fn state_round_trip_and_dedup() {
        let root = temp_root("roundtrip");
        let store = Store::init(&root).unwrap();
        let s = state_with(&["a", "b"]);
        let id1 = store.put_state(&s).unwrap();
        let id2 = store.put_state(&s).unwrap();
        assert_eq!(id1, id2, "identical states deduplicate");
        let loaded = store.get_state(id1).unwrap().unwrap();
        assert_eq!(loaded, s);
        let missing = store
            .get_state(StateId::from_hex(&"0".repeat(64)).unwrap())
            .unwrap();
        assert!(missing.is_none());
        std::fs::remove_dir_all(&root).ok();
    }

    #[test]
    fn commit_round_trip() {
        let root = temp_root("commit");
        let store = Store::init(&root).unwrap();
        let s = state_with(&["a"]);
        let state_id = store.put_state(&s).unwrap();
        let rec = CommitRecord {
            state_id,
            parents: vec![],
            operations: vec![Operation::AddNode {
                id: "a".into(),
                metadata: Metadata::new(),
            }],
            message: "root".into(),
            verification: rahn_core::VerificationSummary::passed(),
        };
        let id = store.put_commit(&rec).unwrap();
        let loaded = store.get_commit(id).unwrap().unwrap();
        assert_eq!(loaded, rec);
        std::fs::remove_dir_all(&root).ok();
    }

    #[test]
    fn corrupted_object_is_refused() {
        let root = temp_root("corrupt");
        let store = Store::init(&root).unwrap();
        let s = state_with(&["a", "b"]);
        let id = store.put_state(&s).unwrap();
        // Corrupt the stored object in place.
        let hex = id.as_hex();
        let mut path = root.join("objects").join(&hex[..2]);
        path.push(&hex[2..]);
        let bytes = fs::read(&path).unwrap();
        let mut corrupted = bytes.clone();
        let last = corrupted.len() - 1;
        corrupted[last] ^= 0xFF;
        fs::write(&path, &corrupted).unwrap();
        let err = store.get_state(id).unwrap_err();
        assert!(matches!(err, StoreError::Corrupted { .. }), "{err}");
        assert!(err.to_string().contains("CORRUPTED"));
        std::fs::remove_dir_all(&root).ok();
    }

    #[test]
    fn refs_and_head() {
        let root = temp_root("refs");
        let store = Store::init(&root).unwrap();
        assert_eq!(store.head().unwrap(), "main");
        let s = state_with(&["a"]);
        let state_id = store.put_state(&s).unwrap();
        let rec = CommitRecord {
            state_id,
            parents: vec![],
            operations: vec![],
            message: "root".into(),
            verification: rahn_core::VerificationSummary::passed(),
        };
        let id = store.put_commit(&rec).unwrap();
        store.set_branch("main", id).unwrap();
        store.set_branch("experiment", id).unwrap();
        assert_eq!(store.get_branch("experiment").unwrap(), Some(id));
        let branches = store.list_branches().unwrap();
        assert_eq!(branches, vec!["experiment".to_string(), "main".to_string()]);
        // Invalid ref names rejected.
        assert!(store.set_branch("../evil", id).is_err());
        assert!(store.set_branch("", id).is_err());
        assert!(store.set_branch("a//b", id).is_err());
        std::fs::remove_dir_all(&root).ok();
    }

    #[test]
    fn index_round_trips() {
        let root = temp_root("index");
        let store = Store::init(&root).unwrap();
        assert_eq!(
            store.load_index().unwrap(),
            State::empty(),
            "init writes an empty index"
        );
        let s = state_with(&["a"]);
        store.save_index(&s).unwrap();
        assert_eq!(store.load_index().unwrap(), s);
        std::fs::remove_dir_all(&root).ok();
    }

    #[test]
    fn init_refuses_existing_repository() {
        let root = temp_root("reinit");
        Store::init(&root).unwrap();
        assert!(Store::init(&root).is_err());
        Store::open(&root).unwrap();
        std::fs::remove_dir_all(&root).ok();
    }
}
