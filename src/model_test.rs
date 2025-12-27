#[cfg(test)]
mod tests {
    use crate::model::*;
    use std::fs::File;

    #[test]
    fn test_single_insert() {
        let path = "cache_single_run.bin";
        let file = File::create(path).unwrap();
        file.set_len(100).unwrap();

        let arena = FileArena::new(100, file);

        let data = "Hello World";
        let range = arena
            .insert("test_key".to_string(), data.as_bytes())
            .unwrap();

        assert_eq!(range.start, 0);
        assert_eq!(range.end, data.len());
        let mut verify_buf = vec![0u8; data.len()];

        #[cfg(unix)]
        {
            use std::os::unix::fs::FileExt;
            let f = std::fs::File::open(path).unwrap();
            f.read_exact_at(&mut verify_buf, range.start as u64)
                .unwrap();
        }

        #[cfg(windows)]
        {
            use std::os::windows::fs::FileExt;
            let f = std::fs::File::open(path).unwrap();
            f.seek_read(&mut verify_buf, range.start as u64).unwrap();
        }

        assert_eq!(data.as_bytes(), verify_buf);

        let _ = std::fs::remove_file(path);
    }

    #[test]
    fn test_single_insert_and_read() {
        use std::fs::OpenOptions;
        let path = "cache_single_run.bin";
        let file = OpenOptions::new()
            .read(true)
            .write(true)
            .create(true)
            .truncate(true)
            .open(path)
            .unwrap();

        file.set_len(100).unwrap();

        let arena = FileArena::new(100, file);

        let data = "Hello World";
        let range = arena
            .insert("test_key".to_string(), data.as_bytes())
            .unwrap();

        assert_eq!(range.start, 0);
        assert_eq!(range.end, data.len());

        let fetched_bytes = arena.fetch("test_key".to_string()).expect("fail to fetch");
        assert_eq!(fetched_bytes.as_slice(), data.as_bytes());

        let fetched_string = String::from_utf8(fetched_bytes).unwrap();
        assert_eq!(fetched_string, data);

        let _ = std::fs::remove_file(path);
    }
}
