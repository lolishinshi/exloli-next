use reqwest::multipart::Part;
use telegraph_rs::{Error, Uploadable};

pub struct ImageBytes(pub Vec<u8>);

impl Uploadable for ImageBytes {
    fn part(&self) -> Result<Part, Error> {
        Ok(Part::bytes(self.0.clone()).file_name("image.jpg").mime_str("image/jpeg")?)
    }
}
