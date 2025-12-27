use std::fs::File;

#[cfg(unix)]
use std::os::unix::fs::FileExt;

#[cfg(windows)]
use std::os::windows::fs::FileExt;

use std::{collections::HashMap, sync::Mutex};

/// this file describes the prototype of the file cache variants
/// it should only contain the object and their memory/access pattern declarations
///
/// end of docstring

#[non_exhaustive]
#[derive(Debug)]
pub enum FileArenaOpCode {
    Succes,
    ErrorNoSpace,
    ErrorInvalidData,
}

pub trait FileArenaMethods {
    fn new(max_offset: usize, file: File) -> FileArena;
    fn insert(&self, name: String, data: &[u8]) -> Result<RegisterRange, FileArenaOpCode>;
    fn fetch(&self, name: String) -> Result<Vec<u8>, FileArenaOpCode>;
}

pub struct FileArena {
    arena: Mutex<ArenaState>, // mutex wrapper for safe FileArena access

    // the pointer to the file, which ideally should use mmap to get a file handle as pointer
    file_ptr: File,
}

pub(crate) struct ArenaState {
    // determines the current size of offset
    pub(crate) offset: usize,

    // maximum file size
    pub(crate) max_offset: usize,

    // location of each cache register
    pub(crate) registry: HashMap<String, RegisterRange>,
}

// locator object
#[derive(Copy, Debug, Clone)]
pub struct RegisterRange {
    pub(crate) start: usize,
    pub(crate) end: usize,
}

impl FileArenaMethods for FileArena {
    fn new(max_offset: usize, file: File) -> FileArena {
        let __arena = Mutex::new(ArenaState {
            offset: 0,
            max_offset: max_offset,
            registry: HashMap::new(),
        });

        Self {
            arena: __arena,
            file_ptr: file,
        }
    }
    fn insert(&self, name: String, data: &[u8]) -> Result<RegisterRange, FileArenaOpCode> {
        let range = {
            let mut state = self.arena.lock().unwrap();

            if state.offset + data.len() > state.max_offset {
                return Err(FileArenaOpCode::ErrorNoSpace);
            }

            let r = RegisterRange {
                start: state.offset,
                end: state.offset + data.len(),
            };

            state.offset += data.len();
            state.registry.insert(name, r);
            r
        };

        let offset = range.start as u64;

        #[cfg(unix)]
        {
            self.file_ptr
                .write_at(data, offset)
                .map_err(|_| FileArenaOpCode::ErrorInvalidData)?;
        }

        #[cfg(windows)]
        {
            self.file_ptr
                .seek_write(data, offset)
                .map_err(|_| FileArenaOpCode::ErrorInvalidData)?;
        }

        Ok(range)
    }

    fn fetch(&self, name: String) -> Result<Vec<u8>, FileArenaOpCode> {
        let range = {
            let state = self.arena.lock().unwrap();

            state
                .registry
                .get(&name)
                .copied()
                .ok_or(FileArenaOpCode::ErrorNoSpace)?
        };

        let offset = range.start;
        let stride = range.end - offset;

        let mut buffer = vec![0u8; stride];

        #[cfg(unix)]
        {
            self.file_ptr
                .read_exact_at(&mut buffer, offset as u64)
                .map_err(|_| FileArenaOpCode::ErrorInvalidData)?;
        }

        #[cfg(windows)]
        {
            self.file_ptr
                .seek_read(&mut buffer, offset)
                .map_err(|_| FileArenaOpCode::ErrorInvalidData)?;
        }

        Ok(buffer)
    }
}
