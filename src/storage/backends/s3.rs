use crate::storage::StorageOperations;
use anyhow::{Context, Result, anyhow, bail};
use aws_sdk_s3::Client;
use std::path::Path;
use tracing::debug;

#[derive(Debug)]
pub struct S3Storage {
    client: Client,
    bucket: String,
}

impl S3Storage {
    pub fn new(bucket: String) -> Result<Self> {
        let bucket_clone = bucket.clone();
        let client = match std::thread::spawn(move || {
            tokio::runtime::Runtime::new().unwrap().block_on(async {
                let config = aws_config::from_env().load().await;
                let client = Client::new(&config);
                if let Err(err) = client.head_bucket().bucket(&bucket_clone).send().await {
                    if err.as_service_error().is_some()
                        && err.as_service_error().unwrap().is_not_found()
                    {
                        client
                            .create_bucket()
                            .bucket(&bucket_clone)
                            .send()
                            .await
                            .unwrap();
                    } else {
                        bail!("Error while initialing S3 bucket for storage: {err:?}");
                    }
                }
                debug!(
                    "Initialised S3 client with endpoint {:?}",
                    config.endpoint_url()
                );
                Ok(client)
            })
        })
        .join()
        {
            Ok(result) => result,
            Err(panic_err) => {
                return Err(anyhow!("S3 client creation thread error: {:?}", panic_err));
            }
        }?;

        Ok(Self { client, bucket })
    }
}

impl StorageOperations for S3Storage {
    async fn read(&self, path: &Path) -> Result<Option<Vec<u8>>> {
        debug!("Reading {path:?} from bucket {}", self.bucket);
        match self
            .client
            .get_object()
            .bucket(&self.bucket)
            .key(path.to_str().context("failed to convert path to str")?)
            .send()
            .await
        {
            Ok(output) => {
                let data = output.body.collect().await?.into_bytes().to_vec();
                Ok(Some(data))
            }
            Err(err) => {
                if err.as_service_error().unwrap().is_no_such_key() {
                    Ok(None)
                } else {
                    Err(err.into())
                }
            }
        }
    }

    async fn exists(&self, path: &Path) -> Result<bool> {
        debug!("Checking if {path:?} exists in bucket {}", self.bucket);
        match self
            .client
            .head_object()
            .bucket(&self.bucket)
            .key(path.to_str().context("failed to convert path to str")?)
            .send()
            .await
        {
            Ok(_) => Ok(true),
            Err(err) => {
                if err.as_service_error().unwrap().is_not_found() {
                    Ok(false)
                } else {
                    Err(err.into())
                }
            }
        }
    }
}
