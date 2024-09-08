use crate::config::S3;
use s3::creds::Credentials;
use s3::error::S3Error;
use s3::{Bucket, Region};
use tokio::io::AsyncRead;

pub struct S3Uploader {
    bucket: Box<Bucket>,
}

impl S3Uploader {
    pub fn new(s3: &S3) -> Result<Self, S3Error> {
        let region = Region::Custom { region: s3.region.clone(), endpoint: s3.endpoint.clone() };
        let credentials =
            Credentials::new(Some(&s3.access_key), Some(&s3.secret_key), None, None, None)?;
        let bucket = Bucket::new(&s3.bucket, region, credentials)?;
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
