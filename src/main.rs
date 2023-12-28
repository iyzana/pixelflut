use image::{DynamicImage, GenericImageView, Rgba};
use rand::{seq::SliceRandom, thread_rng};
use std::{
    io,
    io::prelude::*,
    net::TcpStream,
    sync::Arc,
    thread::{self, JoinHandle},
    time::Duration, process::exit,
};

const ADDR: &str = "151.217.15.90:1337";
const OFFSET: &str = "OFFSET 90 400\n";

fn main() {
    let image = match load_image() {
        Some(img) => img,
        None => return,
    };

    let data = build_commands(&image).into_bytes();
    spawn_fluts(data)
        .into_iter()
        .for_each(|thread| thread.join().unwrap());
}

fn load_image() -> Option<DynamicImage> {
    let image_path = std::env::args()
        .nth(1)
        .unwrap_or_else(|| "image.png".to_string());

    match image::open(&image_path) {
        Ok(img) => Some(img),
        Err(e) => {
            println!("could not load {}: {:?}", image_path, e);
            None
        }
    }
}

fn build_commands(image: &DynamicImage) -> String {
    let (width, height) = image.dimensions();
    let mut pixels = (0..width)
        .flat_map(|x| (0..height).map(move |y| (x, y)))
        .collect::<Vec<_>>();
    pixels.shuffle(&mut thread_rng());
    pixels
        .iter()
        .filter_map(|&(x, y)| {
            let Rgba([r, g, b, a]) = image.get_pixel(x, y);
            if a > 64 {
                Some(format!("PX {} {} {:02x}{:02x}{:02x}\n", x, y, r, g, b))
            } else {
                None
            }
        })
        .collect()
}

fn spawn_fluts(data: Vec<u8>) -> Vec<JoinHandle<()>> {
    let data = Arc::new(data);
    (0..1).map(|_| spawn_flut(data.clone())).collect()
}

fn spawn_flut(data: Arc<Vec<u8>>) -> JoinHandle<()> {
    thread::spawn(move || {
        // randomize timings up to two seconds
        thread::sleep(Duration::from_secs_f64(rand::random::<f64>() * 2.0));
        if let Err(e) = flut(&data) {
            println!("could not flut: {:?}", e);
        }
    })
}

fn flut(data: &[u8]) -> io::Result<()> {
    let mut tcp = TcpStream::connect(ADDR)?;
    tcp.set_nodelay(true).unwrap();
    tcp.write_all(OFFSET.as_bytes()).unwrap();

    println!("fluting...");
    // print!("{}", String::from_utf8_lossy(OFFSET.as_bytes()));
    // print!("{}", String::from_utf8_lossy(data));
    // exit(0);
    loop {
        if let Err(e) = tcp.write_all(data) {
            println!("{:?}, reconnecting", e);
            tcp = TcpStream::connect(ADDR)?;
            tcp.write_all(OFFSET.as_bytes()).unwrap();
        }
    }
}
