use opencv::{prelude::*, videoio, Result};

pub struct Camera {
    cap: videoio::VideoCapture,
    width: i32,
    height: i32,
}

impl Camera {
    pub fn new(device_id: i32, width: i32, height: i32) -> Result<Self> {
        let mut cap = videoio::VideoCapture::new(device_id, videoio::CAP_V4L2)?;
        
        cap.set(videoio::CAP_PROP_FRAME_WIDTH, width as f64)?;
        cap.set(videoio::CAP_PROP_FRAME_HEIGHT, height as f64)?;

        let actual_width = cap.get(videoio::CAP_PROP_FRAME_WIDTH)? as i32;
        let actual_height = cap.get(videoio::CAP_PROP_FRAME_HEIGHT)? as i32;

        println!("Camera initialized with resolution: {}x{}", actual_width, actual_height);

        Ok(Self { cap, width: actual_width, height: actual_height })
    }

    pub fn read(&mut self, frame: &mut Mat) -> Result<bool> {
        self.cap.read(frame)
    }

    pub fn width(&self) -> i32 { self.width }
    pub fn height(&self) -> i32 { self.height }
}
