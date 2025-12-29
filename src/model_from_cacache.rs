use std::{
    collections::HashMap,
    sync::Arc,
    time::{Duration, Instant},
};

use cacache::Integrity;
use tokio::sync::Mutex;

#[non_exhaustive]
#[derive(Debug)]
pub enum FileCacacheOpCode {
    Succes,
    ErrorNoSpace,
    ErrorInvalidData,
}

struct ContentMetadata {
    hash: Integrity,
    timeout: Instant,
}

type FileMap = HashMap<String, ContentMetadata>;

pub struct CacheIntegrityMap {
    map: Arc<Mutex<FileMap>>,

    path: String,
}

impl CacheIntegrityMap {
    pub fn new(path: &str) -> Self {
        Self {
            map: Arc::new(Mutex::new(HashMap::new())),
            path: path.to_string(),
        }
    }

    pub async fn get(&self, key: &str) -> Result<Vec<u8>, FileCacacheOpCode> {
        let now = Instant::now();
        let mut hash_map = self.map.lock().await;

        if let Some(metadata) = hash_map.get(key) {
            if now < metadata.timeout {
                cacache::read_hash(&self.path, &metadata.hash)
                    .await
                    .map_err(|_| FileCacacheOpCode::ErrorInvalidData)
            } else {
                let exp = metadata.hash.clone();

                hash_map.remove(key);

                match cacache::remove_hash(&self.path, &exp).await {
                    _ => Err(FileCacacheOpCode::ErrorNoSpace),
                }
            }
        } else {
            Err(FileCacacheOpCode::ErrorNoSpace)
        }
    }

    pub async fn insert(
        &self,
        key: &str,
        data: &[u8],
        timeout: Option<Instant>,
    ) -> FileCacacheOpCode {
        let _timeout = timeout.unwrap_or_else(|| Instant::now() + Duration::from_secs(10));

        let old_hash = self.map.lock().await.remove(key).map(|meta| meta.hash);

        if let Some(_old_hash) = old_hash {
            let _ = cacache::remove_hash(&self.path, &_old_hash).await;
        }

        let new_hash = match cacache::write(&self.path, key, data).await {
            Ok(h) => h,
            Err(_) => return FileCacacheOpCode::ErrorInvalidData,
        };

        self.map.lock().await.insert(
            key.to_string(),
            ContentMetadata {
                hash: new_hash,
                timeout: _timeout,
            },
        );

        FileCacacheOpCode::Succes
    }

    pub(crate) async fn rehydrate(&self) -> () {
        let _lock = {
            let mut map = self.map.lock().await;

            let _hashes = cacache::list_sync(&self.path).for_each(|m| match m {
                Ok(_m) => {
                    map.insert(
                        _m.key,
                        ContentMetadata {
                            hash: _m.integrity,
                            timeout: Instant::now() + Duration::from_secs(10),
                        },
                    );
                }
                Err(_) => {
                    println!("error")
                }
            });
        };
    }

    pub(crate) fn list(&self) -> () {
        let _hashes = cacache::list_sync(&self.path);
        for a in _hashes {
            println!("{:#?}", a);
        }
    }
}
