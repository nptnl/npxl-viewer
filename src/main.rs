use std::fs::File;
use std::io::BufRead;
use std::path::Path;
use std::sync::atomic::AtomicBool;
use std::time::Instant;
use std::{env::args, error::Error, io::BufReader};
use std::io::prelude::*;

use notify::{Event, RecursiveMode, Watcher};
use pixel_canvas::{Canvas, Color};

static VERBOSE: AtomicBool = AtomicBool::new(false);

fn handle_file(name: String) -> Result<(), Box<dyn Error>> {
    let (width, height, pixel_width, color_base) = read_header(&name)?;
    // create a window with the correct width and height
    let canvas = Canvas::new(width, height).show_ms(true).hidpi(true).title(&name);
    canvas.render(move |_, image| {
        let file = File::open(&name).expect("failed to read file");
        // read the image one line at a time and skip the header
        let mut data = BufReader::new(file).lines().skip(2);
        // Modify the `image` based on your state.
        let width = image.width() as usize;
        let proportion = (255 / color_base) as u8;
        let mut image_data = Vec::with_capacity(width * height * 3);
        // iterate over the actual image window
        for (_y, row) in image.chunks(width).enumerate() {
            if let Some(Ok(v)) = data.next() {
                let mut c = v.chars();
                for (_x, _px) in row.iter().enumerate() {
                    let mut color: Vec<u8> = vec![0; pixel_width];
                    // if it is 1 number per a pixel, we can add the number for all values of each
                    // color and multiply by the proportion so it's not dark
                    if pixel_width == 1 {
                        if let Some(v) = c.next() {
                            if let Some(v) = v.to_digit(color_base) {
                                color[0] = v as u8;
                            }
                        }
                        image_data.push(Color {
                            r: color[0] * proportion,
                            g: color[0] * proportion,
                            b: color[0] * proportion,
                        });
                        continue;
                    }
                    for i in color.iter_mut() {
                        if let Some(v) = c.next() {
                            if let Some(v) = v.to_digit(color_base) {
                                *i = v as u8;
                            }
                        }
                    }
                    image_data.push(Color {
                        r: color[0] * proportion,
                        g: color[1] * proportion,
                        b: color[2] * proportion,
                    });
                }
            }
        }
        image.copy_from_slice(&image_data);
    });
    Ok(())
}

fn read_header<P: AsRef<Path>>(path: P) -> Result<(usize, usize, usize, u32), Box<dyn Error>> {
    let file = File::open(&path)?;
    // bufreader returns an iterator which is lazy, aka it doesn't do any work until we want it to,
    // this prevents from loading the entire file into memory at once
    let data = BufReader::new(file);
    let mut width = 0;
    let mut height = 0;
    let mut pixel_width = 0;
    let mut color_base = 2;
    for (i, line) in data.lines().enumerate() {
        let line = line?;
        // we onyl have to check the first two lines and parse them
        match i {
            0 => {
                for (pos, c) in line.split_whitespace().enumerate() {
                    let c = c.parse()?;
                    match pos {
                        0 => width = c,
                        1 => height = c,
                        _ => {}
                    }
                }
            }
            1 => {
                for (pos, c) in line.split_whitespace().enumerate() {
                    let c = c.parse()?;
                    match pos {
                        0 => color_base = c as u32,
                        1 => pixel_width = c,
                        _ => {}
                    }
                }
            }
            _ => break,
        }
    }
    Ok((width, height, pixel_width, color_base))
}

fn build_multi() -> Result<(), Box<dyn Error>> {
    let mut writeto = File::create("./full.npxl").expect("no header file in this directory!");
    let mut filenumber: u32 = 0;

    loop {
        // standard for npxlb outputs is just 3.npxl ... 7.npxl etc
        // the 0 file contains the header
        let path = format!("./{filenumber}.npxl");
        let file = File::open(path.as_str());
        if let Err(_) = file { break; }
        // the files used for building dont include headers, they're just straight lines of pixels
        let data = BufReader::new(file?).lines();
        for line in data {
            writeto.write_all(line?.as_bytes()).expect("cannot write line");
            writeto.write_all("\n".as_bytes()).expect("cannot slash N");
        }
        filenumber += 1;
    }
    Ok(())
}

fn to_png<P: AsRef<Path>>(path: P) -> Result<(), Box<dyn Error>> {
    // if the extension is not npxl then we can ignore this file
    if let Some(v) = path.as_ref().extension() {
        if v.to_string_lossy() != "npxl" {
            return Ok(());
        }
    } else {
        return Ok(());
    }
    // get the file header information
    let (width, height, pixel_width, color_base) = read_header(&path)?;
    let frametime = Instant::now();
    let file = File::open(&path).expect("failed to read file");
    // skip the file header
    let data = BufReader::new(file).lines().skip(2);
    // when we take in binary for example we will get 0 or 1, putting these numbers directly in the
    // png data Vec will be very dark as we are using u8 (0-255), so we must determine the
    // proportion we multiply by to allow the max value of that base to be 255
    // we need to subtract one because for example binary is denoted by 2 in the file header, but
    // 255 / 2 * 1 is not 255 obviously, to account for the bases starting at 0 we can just
    // subtract 1
    let proportion = (255 / color_base - 1) as u8;
    // pre reserve the space in memory adding these pixels will take up
    let mut pixels: Vec<u8> = Vec::with_capacity(width * height * 3);
    for line in data {
        let line = line?;
        // to digit takes in the radix (base) of the value we want to convert
        if pixel_width == 1 {
            for p in line.chars() {
                // if we only have 1 number per a pixel we gotta add it 3 times for the rgb values
                for _ in 0..3 {
                    pixels.push(p.to_digit(color_base).unwrap_or_default() as u8 * proportion);
                }
            }
            continue;
        }
        for p in line.chars() {
            pixels.push(p.to_digit(color_base).unwrap_or_default() as u8 * proportion);
        }
    }
    let image = image::DynamicImage::ImageRgb8(
        image::ImageBuffer::from_raw(width as u32, height as u32, pixels)
            .expect("invalid image buffer"),
    );
    // write the resulting png
    if let Some(v) = path.as_ref().file_stem() {
        image.save(format!("{}.png", v.to_string_lossy()))?;
    }
    // long the time it took if we have verbose enabled
    if VERBOSE.load(std::sync::atomic::Ordering::Acquire) {
        println!(
            "completed {} in {} ms",
            path.as_ref().to_string_lossy(),
            frametime.elapsed().as_millis()
        );
    }
    Ok(())
}

fn main() -> Result<(), Box<dyn Error>> {
    // collect program arguments, the first element is always the file path of the binary so we can
    // ignore that for this program by skipping
    let args: Vec<_> = args().skip(1).collect();
    // set verbose flag if it's in arguments
    if args.contains(&String::from("--verbose")) {
        VERBOSE.swap(true, std::sync::atomic::Ordering::Release);
    }
    if args.contains(&String::from("--convert")) {
        for arg in args.iter().skip(1) {
            if arg == "--verbose" { continue }
            match to_png(arg) {
                Ok(_) => {}
                _ => println!("encountered error while transforming {arg}"),
            };
        }
        std::process::exit(0);
    }
    if args.contains(&String::from("--build")) {
        match build_multi() {
            Ok(_) => {return Ok(())},
            _ => println!("building error"),
        };
    }
    // if we are in watch mode, then we can have a loop to wait for changes
    // we only care if we modify the image or create a file
    if args.contains(&String::from("--watch")) {
        let mut watcher =
            notify::recommended_watcher(|res: Result<Event, notify::Error>| match res {
                Ok(e) => {
                    type E = notify::EventKind;
                    match e.kind {
                        E::Modify(_) | E::Create(_) => {
                            for file in &e.paths {
                                match to_png(file) {
                                    Ok(_) => {}
                                    _ => println!(
                                        "encountered error while transforming {}",
                                        file.to_string_lossy()
                                    ),
                                };
                            }
                        }
                        _ => {}
                    }
                }
                Err(e) => println!("watch error: {:?}", e),
            })?;
        loop {
            // watch current directory and sub directories
            watcher.watch(Path::new("."), RecursiveMode::Recursive)?;
        }
    }
    // if we are not in watch mode then we need to process each file given as an argument
    let files = args.into_iter().map(handle_file);
    for file in files {
        file?;
    }
    println!("finish");
    Ok(())
}
