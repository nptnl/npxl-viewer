use std::io::BufRead;
use std::{error::Error, io::BufReader, env::args};
use std::fs::File;

use pixel_canvas::{Canvas, Color};

fn handle_file(name: String) -> Result<(), Box<dyn Error>> {
    let file = File::open(&name)?;
    let data = BufReader::new(file);
    let mut width = 0;
    let mut height = 0;
    let mut pixel_width = 0;
    let mut color_base = 2;
    for (i, line) in data.lines().enumerate() {
        let line = line?;
        match i {
            0 => {
                for (pos, c) in line.split_whitespace().enumerate() {
                    let c = c.parse()?;
                    match pos {
                        0 => width = c,
                        1 => height = c,
                        2 => pixel_width = c,
                        _ => panic!("invalid format"),
                    }
                }
            }
            1 => {
                color_base = line.parse()?;
            }
            _ => break,
        }
    }
    println!("{pixel_width}");
    let canvas = Canvas::new(width, height)
        .title(&name);
    canvas.render(move |_, image| {
        let file = File::open(&name).expect("failed to read file");
        let mut data = BufReader::new(file).lines();
        // Modify the `image` based on your state.
        let width = image.width() as usize;
        for (_y, row) in image.chunks_mut(width).enumerate() {
            if let Some(Ok(v)) = data.next() {
                let mut c = v.chars();
                for (_x, px) in row.iter_mut().enumerate() {
                    let mut color: Vec<u8> = vec![0; pixel_width];
                    if pixel_width == 1 {
                        if let Some(v) = c.next() {
                            if let Some(v) = v.to_digit(color_base) {
                                color[0] = v as u8;
                            }
                        }
                        *px = Color {
                            r: color[0],
                            g: color[0],
                            b: color[0],
                        };
                        continue;
                    }
                    for i in color.iter_mut() {
                        if let Some(v) = c.next() {
                            if let Some(v) = v.to_digit(color_base) {
                                *i = v as u8;
                            }
                        }
                    }
                    *px = Color {
                        r: color[0] * 25,
                        g: color[1] * 25,
                        b: color[2] * 25,
                    };
                }
            }
        }
    });
    Ok(())
}

fn main() -> Result<(), Box<dyn Error>> {
    let files: Vec<_> = args().skip(1).map(handle_file).collect();
    for file in files {
        file?;
    }
    Ok(())
}
