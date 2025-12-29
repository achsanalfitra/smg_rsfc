#[cfg(test)]
mod tests {
    use std::time::{Duration, Instant};

    use crate::model_from_cacache::CacheIntegrityMap;

    #[tokio::test]
    async fn test_single_insert_and_read() {
        let path = "./cache.bin";

        let fc = CacheIntegrityMap::new(path);
        let key = "my_key";
        let data = b"Hello world";

        fc.insert(key, data, Some(Instant::now() + Duration::from_secs(10)))
            .await;

        let result = fc.get(key).await;

        let _ = std::fs::remove_dir_all(path);

        assert_eq!(result.unwrap(), data);
    }
}
