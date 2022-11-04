use std::io::BufRead;
use std::{error::Error, io::BufReader, env::args};
use std::fs::File;

use pixel_canvas::{Canvas, Color};

#[repr(u8)]
enum ColorBase {
    Binary,
    Decimal,
    Hexadecimal,
    Base64,
}

fn handle_file(name: String) -> Result<(), Box<dyn Error>> {
    let file = File::open(&name)?;
    let data = BufReader::new(file);
    let mut width = 0;
    let mut height = 0;
    let mut pixel_width = 0;
    let mut color_base = ColorBase::Binary;
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
                color_base = match line.as_str() {
                    "2" => ColorBase::Binary,
                    "10" => ColorBase::Decimal,
                    "16" => ColorBase::Hexadecimal,
                    "64" => ColorBase::Base64,
                    _ => ColorBase::Binary,
                }
            }
            _ => break,
        }
    }
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
                    let mut color = vec![0; pixel_width];
                    for v in c.take(pixel_width) {
                        match &color_base {
                            Binary => {
                                match v {
                                    '1' => color = vec![255, 255, 255],
                                    _ => {},
                                }
                            }
                            Decimal => {
                            }
                            _ => unreachable!(),
                        }
                    }
                    // match pixel_width {
                    //     1 => {
                    //
                    // *px = Color {
                    //     r: color[0],
                    //     g: color[1],
                    //     b: color[2],
                    // };
                    //     }
                    // }
                }
            }
        }
    });
    Ok(())
}

fn main() -> Result<(), Box<dyn Error>> {
    let files: Vec<_> = args().skip(1).map(handle_file).collect();
    Ok(())
}
