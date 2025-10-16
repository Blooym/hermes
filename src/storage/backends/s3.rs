use crate::storage::{FileMetadata, StorageOperations};
use anyhow::{Context, Result, anyhow, bail};
use aws_sdk_s3::Client;
use std::path::Path;
use tokio::io::AsyncRead;
use tracing::debug;

#[derive(Debug)]
pub struct S3Storage {
    client: Client,
    bucket: Box<str>,
}

impl S3Storage {
    pub fn new<B: Into<Box<str>>>(bucket: B) -> Result<Self> {
        let bucket = bucket.into();
        let client = std::thread::spawn({
            let bucket = bucket.clone();
            move || {
                tokio::runtime::Runtime::new()
                    .context("Failed to create Tokio runtime")?
                    .block_on(async move {
                        let config = aws_config::from_env().load().await;
                        let client = Client::new(&config);
                        if let Err(err) = client.head_bucket().bucket(&*bucket).send().await {
                            if err.as_service_error().map(|e| e.is_not_found()) == Some(true) {
                                client
                                    .create_bucket()
                                    .bucket(&*bucket)
                                    .send()
                                    .await
                                    .context("Failed to create S3 bucket")?;
                            } else {
                                bail!("Error while initializing S3 bucket: {err:?}");
                            }
                        }
                        debug!(
                            "Initialized S3 client with endpoint {:?}",
                            config.endpoint_url()
                        );
                        Ok(client)
                    })
            }
        })
        .join()
        .map_err(|e| anyhow!("S3 client thread panicked: {e:?}"))??;
        Ok(Self { client, bucket })
    }
}

impl StorageOperations for S3Storage {
    async fn read_stream(&self, path: &Path) -> Result<Option<Box<dyn AsyncRead + Unpin + Send>>> {
        debug!("Opening stream for {path:?} from bucket {}", self.bucket);
        match self
            .client
            .get_object()
            .bucket(&*self.bucket)
            .key(path.to_str().context("failed to convert path to str")?)
            .send()
            .await
        {
            Ok(output) => Ok(Some(Box::new(output.body.into_async_read()))),
            Err(err) => {
                if err.as_service_error().map(|e| e.is_no_such_key()) == Some(true) {
                    Ok(None)
                } else {
                    Err(err.into())
                }
            }
        }
    }

    async fn metadata(&self, path: &Path) -> Result<Option<FileMetadata>> {
        debug!("Checking if {path:?} exists in bucket {}", self.bucket);
        match self
            .client
            .head_object()
            .bucket(&*self.bucket)
            .key(path.to_str().context("failed to convert path to str")?)
            .send()
            .await
        {
            Ok(data) => Ok(Some(FileMetadata {
                file_size: data.content_length.unwrap_or_default().try_into()?,
            })),
            Err(err) => {
                if err.as_service_error().map(|e| e.is_not_found()) == Some(true) {
                    Ok(None)
                } else {
                    Err(err.into())
                }
            }
        }
    }
}
