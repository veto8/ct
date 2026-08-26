use std::fs::File;
use std::io::BufWriter;

const IMAGE_SIZE: u32 = 256;
const LOGO_SIZE: u32 = 64;

fn clockwise_frame_coords() -> Vec<(u32, u32)> {
    let mut coords = Vec::with_capacity(1020);
    let s = IMAGE_SIZE - 1;
    for x in 0..=s {
        coords.push((x, 0));
    }
    for y in 1..=s {
        coords.push((s, y));
    }
    for x in (0..=s - 1).rev() {
        coords.push((x, s));
    }
    for y in (1..=s - 1).rev() {
        coords.push((0, y));
    }
    coords
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
    let coords = clockwise_frame_coords();
    let max_chars = coords.len() / 8;
    if text.len() > max_chars {
        return Err(format!("Text too long: {} chars, max {} for 256x256 frame", text.len(), max_chars).into());
    }

    let mut pixels = vec![255u8; (IMAGE_SIZE * IMAGE_SIZE * 4) as usize];
    for (i, ch) in text.bytes().enumerate() {
        for bit in 0..8 {
            let pixel_idx = i * 8 + bit;
            let (x, y) = coords[pixel_idx];
            let set = (ch >> (7 - bit)) & 1 == 1;
            let dst = ((y * IMAGE_SIZE + x) * 4) as usize;
            if set {
                pixels[dst] = 0;
                pixels[dst + 1] = 0;
                pixels[dst + 2] = 0;
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

    let coords = clockwise_frame_coords();
    let num_chars = coords.len() / 8;
    let mut result = Vec::with_capacity(num_chars);

    for i in 0..num_chars {
        let mut byte = 0u8;
        for bit in 0..8 {
            let pixel_idx = i * 8 + bit;
            let (x, y) = coords[pixel_idx];
            let dst = ((y * info.width + x) * 4) as usize;
            if pixels[dst] < 128 {
                byte |= 1 << (7 - bit);
            }
        }
        if byte == 0 {
            break;
        }
        result.push(byte);
    }

    Ok(String::from_utf8(result)?)
}
