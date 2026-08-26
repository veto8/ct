use std::fs::File;
use std::io::BufWriter;

const STRIP_DEPTH: i32 = 8;

fn calc_image_size(text_len: usize) -> u32 {
    let mut size: u32 = 256;
    loop {
        let cols = frame_columns(size);
        if text_len <= cols.len() {
            return size;
        }
        size += STRIP_DEPTH as u32 * 2;
    }
}

fn frame_columns(image_size: u32) -> Vec<(u32, u32, i32, i32)> {
    let mut cols = Vec::new();
    let mut d: i32 = 0;
    let s = image_size as i32 - 1;

    loop {
        let min = d;
        let max = s - d;
        if max - min < STRIP_DEPTH {
            break;
        }
        for x in min..=max - STRIP_DEPTH {
            cols.push((x as u32, min as u32, 0, 1));
        }
        for y in min + 1..=max - STRIP_DEPTH {
            cols.push((max as u32, y as u32, -1, 0));
        }
        for x in (min + STRIP_DEPTH..=max).rev() {
            cols.push((x as u32, max as u32, 0, -1));
        }
        for y in (min + STRIP_DEPTH + 1..=max).rev() {
            cols.push((min as u32, y as u32, 1, 0));
        }
        d += STRIP_DEPTH;
    }
    cols
}

fn overlay_logo(pixels: &mut [u8], logo_bytes: &[u8], image_size: u32) {
    let decoder = png::Decoder::new(std::io::Cursor::new(logo_bytes));
    let mut reader = decoder.read_info().unwrap();
    let mut buf = vec![0u8; reader.output_buffer_size()];
    let info = reader.next_frame(&mut buf).unwrap();
    let logo = &buf[..info.buffer_size()];
    let logo_w = info.width;
    let logo_h = info.height;
    let offset_x = (image_size - logo_w) / 2;
    let offset_y = (image_size - logo_h) / 2;
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
            let dst_idx = ((py * image_size + px) * 4) as usize;
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
    let image_size = calc_image_size(text.len());
    let cols = frame_columns(image_size);

    let mut pixels = vec![255u8; (image_size * image_size * 4) as usize];

    for (i, ch) in text.bytes().enumerate() {
        let (cx, cy, dx, dy) = cols[i];
        for bit in 0..8u32 {
            let set = (ch >> (7 - bit)) & 1 == 1;
            let px = cx as i32 + dx * bit as i32;
            let py = cy as i32 + dy * bit as i32;
            if px >= 0 && px < image_size as i32 && py >= 0 && py < image_size as i32 {
                let dst = ((py as u32 * image_size + px as u32) * 4) as usize;
                if set {
                    pixels[dst] = 0;
                    pixels[dst + 1] = 0;
                    pixels[dst + 2] = 0;
                }
            }
        }
    }

    let logo_bytes = include_bytes!("../../test/assets/icons/ct64.png");
    overlay_logo(&mut pixels, logo_bytes, image_size);

    let file = File::create(output_path)?;
    let w = BufWriter::new(file);
    let mut encoder = png::Encoder::new(w, image_size, image_size);
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
    let image_size = info.width;

    let cols = frame_columns(image_size);
    let mut result = Vec::with_capacity(cols.len());

    for &(cx, cy, dx, dy) in &cols {
        let mut byte = 0u8;
        for bit in 0..8u32 {
            let px = cx as i32 + dx * bit as i32;
            let py = cy as i32 + dy * bit as i32;
            if px >= 0 && px < image_size as i32 && py >= 0 && py < image_size as i32 {
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
