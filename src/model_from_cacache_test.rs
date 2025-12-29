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

    #[tokio::test]
    async fn test_multiple_insert_and_list() {
        let this_path = "./list-cache.bin";

        let fc = CacheIntegrityMap::new(this_path);
        let keys = ["a", "b", "c"];
        let data = [b"hello", b"world", b"foooo"];

        for i in 0..keys.len() {
            fc.insert(
                keys[i],
                data[i],
                Some(Instant::now() + Duration::from_secs(10)),
            )
            .await;
        }

        fc.list();

        let _ = std::fs::remove_dir_all(this_path);
    }

    #[tokio::test]
    async fn test_resync() {
        let this_path = "./resync-cache.bin";

        let fc = CacheIntegrityMap::new(this_path);
        let keys = ["a", "b", "c"];
        let data = [b"hello", b"world", b"foooo"];

        for i in 0..keys.len() {
            fc.insert(
                keys[i],
                data[i],
                Some(Instant::now() + Duration::from_secs(10)),
            )
            .await;
        }

        let fc_2 = CacheIntegrityMap::new(this_path);
        fc_2.rehydrate().await;

        let res_1 = fc.get(keys[0]).await;
        let res_2 = fc_2.get(keys[0]).await;

        let _ = std::fs::remove_dir_all(this_path);

        assert_eq!(res_1.unwrap(), res_2.unwrap());
    }
}
