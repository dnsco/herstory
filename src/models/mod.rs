use std::sync::{Arc, Mutex};
use std::time;

mod request;
mod serialization;

use models::serialization::DeserializedPhotoset;
use models::request::{Request, CurlRequest};
use threadpool::Threadpool;

use Config;
use HasStatus;
use Status;
use Status::*;

pub type Images = Vec<Arc<Mutex<Image>>>;

#[derive(Debug)]
pub struct Photoset {
    pub name: String,
    pub images: Images,
}

impl Photoset {
    pub fn from_json(json: &str) -> Photoset {
        DeserializedPhotoset::from_json(json)
    }

    pub fn download_and_save(&self) {
        Threadpool::batch(4, &self.images, |image: &mut Image| image.spawn_request());
    }
}

#[derive(Debug)]
pub struct Image {
    pub index: i32,
    pub url: String,
    pub request: Option<Request<CurlRequest>>,
}

impl Image {
    fn filename(&self) -> String {
        let t = time::UNIX_EPOCH.elapsed().unwrap().as_secs();
        format!("{}/{}_{}", Config::DATA_DIR, self.index, t)
    }

    pub fn spawn_request(&mut self) {
        let mut request = Request::build(&self.url, &self.filename());
        request.perform_and_save().ok();
        self.request = Some(request);
    }
}

impl HasStatus for Image {
    fn status(&self) -> Status {
        match self.request {
            None => Pending,
            Some(ref req) => req.status(),
        }
    }
}
