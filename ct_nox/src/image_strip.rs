use std::fs::File;
use std::io::BufWriter;

const IMAGE_SIZE: u32 = 256;
const STRIP_DEPTH: i32 = 8;

fn frame_columns() -> Vec<(u32, u32, i32, i32)> {
    let mut cols = Vec::with_capacity(127);
    let s = IMAGE_SIZE as i32 - 1;

    for x in 0..=(s - STRIP_DEPTH) {
        cols.push((x as u32, 0, 0, 1));
    }
    for y in 1..=(s - STRIP_DEPTH) {
        cols.push((s as u32, y as u32, -1, 0));
    }
    for x in (STRIP_DEPTH..=s).rev() {
        cols.push((x as u32, s as u32, 0, -1));
    }
    for y in (STRIP_DEPTH + 1..=s).rev() {
        cols.push((0, y as u32, 1, 0));
    }
    cols
}

fn overlay_logo(pixels: &mut [u8], logo_bytes: &[u8]) {
    let decoder = png::Decoder::new(std::io::Cursor::new(logo_bytes));
    let mut reader = decoder.read_info().unwrap();
    let mut buf = vec![0u8; reader.output_buffer_size()];
    let info = reader.next_frame(&mut buf).unwrap();
    let logo = &buf[..info.buffer_size()];
    let logo_w = info.width;
    let logo_h = info.height;
    let offset_x = (IMAGE_SIZE - logo_w) / 2;
    let offset_y = (IMAGE_SIZE - logo_h) / 2;
    for ly in 0..logo_h {
        for lx in 0..logo_w {
            let src_idx = ((ly * logo_w + lx) * 4) as usize;
            let r = logo[src_idx];
            let g = logo[src_idx + 1];
            let b = logo[src_idx + 2];
            let a = logo[src_idx + 3];
            if a == 0 {
                continue;
            }
            let px = offset_x + lx;
            let py = offset_y + ly;
            let dst_idx = ((py * IMAGE_SIZE + px) * 4) as usize;
            if a == 255 {
                pixels[dst_idx] = r;
                pixels[dst_idx + 1] = g;
                pixels[dst_idx + 2] = b;
                pixels[dst_idx + 3] = 255;
            } else {
                let alpha = a as f32 / 255.0;
                let inv = 1.0 - alpha;
                pixels[dst_idx] = (r as f32 * alpha + pixels[dst_idx] as f32 * inv) as u8;
                pixels[dst_idx + 1] = (g as f32 * alpha + pixels[dst_idx + 1] as f32 * inv) as u8;
                pixels[dst_idx + 2] = (b as f32 * alpha + pixels[dst_idx + 2] as f32 * inv) as u8;
                pixels[dst_idx + 3] = 255;
            }
        }
    }
}

pub fn encode_to_image(text: &str, output_path: &str) -> Result<(), Box<dyn std::error::Error>> {
    let cols = frame_columns();
    let max_chars = cols.len();
    if text.len() > max_chars {
        return Err(format!("Text too long: {} chars, max {}", text.len(), max_chars).into());
    }

    let mut pixels = vec![255u8; (IMAGE_SIZE * IMAGE_SIZE * 4) as usize];

    for (i, ch) in text.bytes().enumerate() {
        let (cx, cy, dx, dy) = cols[i];
        for bit in 0..8u32 {
            let set = (ch >> (7 - bit)) & 1 == 1;
            let px = cx as i32 + dx * bit as i32;
            let py = cy as i32 + dy * bit as i32;
            if px >= 0 && px < IMAGE_SIZE as i32 && py >= 0 && py < IMAGE_SIZE as i32 {
                let dst = ((py as u32 * IMAGE_SIZE + px as u32) * 4) as usize;
                if set {
                    pixels[dst] = 0;
                    pixels[dst + 1] = 0;
                    pixels[dst + 2] = 0;
                }
            }
        }
    }

    let logo_bytes = include_bytes!("../../test/assets/icons/ct64.png");
    overlay_logo(&mut pixels, logo_bytes);

    let file = File::create(output_path)?;
    let w = BufWriter::new(file);
    let mut encoder = png::Encoder::new(w, IMAGE_SIZE, IMAGE_SIZE);
    encoder.set_color(png::ColorType::Rgba);
    encoder.set_depth(png::BitDepth::Eight);
    let mut writer = encoder.write_header()?;
    writer.write_image_data(&pixels)?;
    Ok(())
}

pub fn decode_from_image(input_path: &str) -> Result<String, Box<dyn std::error::Error>> {
    let file = File::open(input_path)?;
    let decoder = png::Decoder::new(file);
    let mut reader = decoder.read_info()?;
    let mut buf = vec![0u8; reader.output_buffer_size()];
    let info = reader.next_frame(&mut buf)?;
    let pixels = &buf[..info.buffer_size()];

    let cols = frame_columns();
    let mut result = Vec::with_capacity(cols.len());

    for (i, &(cx, cy, dx, dy)) in cols.iter().enumerate() {
        let mut byte = 0u8;
        for bit in 0..8u32 {
            let px = cx as i32 + dx * bit as i32;
            let py = cy as i32 + dy * bit as i32;
            if px >= 0 && px < IMAGE_SIZE as i32 && py >= 0 && py < IMAGE_SIZE as i32 {
                let dst = ((py as u32 * info.width + px as u32) * 4) as usize;
                if pixels[dst] < 128 {
                    byte |= 1 << (7 - bit);
                }
            }
        }
        if byte == 0 {
            break;
        }
        result.push(byte);
    }

    Ok(String::from_utf8(result)?)
}
