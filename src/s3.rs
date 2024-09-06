use crate::config::R2;
use s3::creds::Credentials;
use s3::error::S3Error;
use s3::{Bucket, Region};
use tokio::io::AsyncRead;

pub struct R2Uploader {
    bucket: Box<Bucket>,
}

impl R2Uploader {
    pub fn new(r2: &R2) -> Result<Self, S3Error> {
        let region = Region::R2 { account_id: r2.account_id.clone() };
        let credentials = Credentials {
            access_key: Some(r2.access_key.clone()),
            secret_key: Some(r2.secret_key.clone()),
            ..Credentials::default()?
        };
        let bucket = Bucket::new(&r2.bucket, region, credentials)?;
        Ok(Self { bucket })
    }

    pub async fn upload<R: AsyncRead + Unpin + ?Sized>(
        &self,
        name: &str,
        reader: &mut R,
    ) -> Result<(), S3Error> {
        self.bucket.put_object_stream(reader, name).await?;
        Ok(())
    }
}
