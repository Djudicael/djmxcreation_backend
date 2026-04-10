use std::sync::{Arc, Mutex};

use app_core::storage::storage_repository::{DynIStorageRepository, IStorageRepository};
use app_error::Error;
use async_trait::async_trait;

struct State {
    object_url: String,
    uploaded_files: Mutex<Vec<(String, String, Vec<u8>)>>,
    removed_objects: Mutex<Vec<(String, String)>>,
    url_requests: Mutex<Vec<(String, String)>>,
}

struct FakeStorageRepository {
    state: Arc<State>,
}

pub struct StorageRepositoryProbe {
    state: Arc<State>,
}

impl StorageRepositoryProbe {
    pub fn uploaded_files(&self) -> Vec<(String, String, Vec<u8>)> {
        self.state
            .uploaded_files
            .lock()
            .expect("storage upload lock poisoned")
            .clone()
    }

    pub fn removed_objects(&self) -> Vec<(String, String)> {
        self.state
            .removed_objects
            .lock()
            .expect("storage remove lock poisoned")
            .clone()
    }

    pub fn url_requests(&self) -> Vec<(String, String)> {
        self.state
            .url_requests
            .lock()
            .expect("storage url lock poisoned")
            .clone()
    }
}

#[async_trait]
impl IStorageRepository for FakeStorageRepository {
    async fn upload_file(
        &self,
        bucket_name: &str,
        file_name: &str,
        file: &[u8],
    ) -> Result<(), Error> {
        self.state
            .uploaded_files
            .lock()
            .expect("storage upload lock poisoned")
            .push((
                bucket_name.to_string(),
                file_name.to_string(),
                file.to_vec(),
            ));
        Ok(())
    }

    async fn get_object_url(&self, bucket_name: &str, file_name: &str) -> Result<String, Error> {
        self.state
            .url_requests
            .lock()
            .expect("storage url lock poisoned")
            .push((bucket_name.to_string(), file_name.to_string()));
        Ok(self.state.object_url.clone())
    }

    async fn remove_object(&self, bucket_name: &str, file_name: &str) -> Result<(), Error> {
        self.state
            .removed_objects
            .lock()
            .expect("storage remove lock poisoned")
            .push((bucket_name.to_string(), file_name.to_string()));
        Ok(())
    }
}

pub fn create_storage_repository(
    object_url: impl Into<String>,
) -> (DynIStorageRepository, StorageRepositoryProbe) {
    let state = Arc::new(State {
        object_url: object_url.into(),
        uploaded_files: Mutex::new(Vec::new()),
        removed_objects: Mutex::new(Vec::new()),
        url_requests: Mutex::new(Vec::new()),
    });

    let repository: DynIStorageRepository = Arc::new(FakeStorageRepository {
        state: state.clone(),
    });

    (repository, StorageRepositoryProbe { state })
}
