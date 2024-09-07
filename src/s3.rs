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
        let credentials =
            Credentials::new(Some(&r2.access_key), Some(&r2.secret_key), None, None, None)?;
        let bucket = Bucket::new(&r2.bucket, region, credentials)?;
        Ok(Self { bucket })
    }

    pub async fn upload<R: AsyncRead + Unpin>(
        &self,
        name: &str,
        reader: &mut R,
    ) -> Result<(), S3Error> {
        let content_type = if name.ends_with(".jpg") {
            "image/jpeg"
        } else if name.ends_with(".png") {
            "image/png"
        } else {
            unreachable!()
        };
        self.bucket.put_object_stream_with_content_type(reader, name, content_type).await?;
        Ok(())
    }
}
