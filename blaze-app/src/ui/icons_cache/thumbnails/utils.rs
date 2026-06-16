use crate::ui::icons_cache::thumbnails::thumbnails_manager::ThumbError;

pub fn resolve_tiff_data(
    buffer: tiff::decoder::DecodingResult,
    w: u32,
    h: u32,
) -> Result<Vec<u8>, ThumbError> {
    let rgb_data = match buffer {
        tiff::decoder::DecodingResult::U8(data) => {
            if data.len() == (w * h * 4) as usize {
                data
            } else if data.len() == (w * h * 3) as usize {
                let mut rgba = Vec::with_capacity((w * h * 4) as usize);
                for chunk in data.chunks_exact(3) {
                    rgba.extend_from_slice(&[chunk[0], chunk[1], chunk[2], 255]);
                }
                rgba
            } else if data.len() == (w * h) as usize {
                let mut rgba = Vec::with_capacity((w * h * 4) as usize);
                for &pixel in &data {
                    rgba.extend_from_slice(&[pixel, pixel, pixel, 255]);
                }
                rgba
            } else {
                return Err(ThumbError::ImageError);
            }
        }
        tiff::decoder::DecodingResult::U16(data) => {
            let mut rgba = Vec::with_capacity((w * h * 4) as usize);
            for &pixel in &data {
                let p = (pixel >> 8) as u8;
                rgba.extend_from_slice(&[p, p, p, 255]);
            }
            rgba
        }
        tiff::decoder::DecodingResult::U32(data) => {
            let mut rgba = Vec::with_capacity((w * h * 4) as usize);
            for &pixel in &data {
                let p = (pixel / 16843009) as u8;
                rgba.extend_from_slice(&[p, p, p, 255]);
            }
            rgba
        }
        tiff::decoder::DecodingResult::U64(data) => {
            let mut rgba = Vec::with_capacity((w * h * 4) as usize);
            for &pixel in &data {
                let p = (pixel / 72340172838076673) as u8;
                rgba.extend_from_slice(&[p, p, p, 255]);
            }
            rgba
        }
        tiff::decoder::DecodingResult::F32(data) => {
            let mut rgba = Vec::with_capacity((w * h * 4) as usize);
            for &pixel in &data {
                let p = (pixel * 255.0) as u8;
                rgba.extend_from_slice(&[p, p, p, 255]);
            }
            rgba
        }
        tiff::decoder::DecodingResult::F64(data) => {
            let mut rgba = Vec::with_capacity((w * h * 4) as usize);
            for &pixel in &data {
                let p = (pixel * 255.0) as u8;
                rgba.extend_from_slice(&[p, p, p, 255]);
            }
            rgba
        }
        tiff::decoder::DecodingResult::I8(data) => {
            let mut rgba = Vec::with_capacity((w * h * 4) as usize);
            for &pixel in &data {
                let p = (pixel as i16 + 128) as u8;
                rgba.extend_from_slice(&[p, p, p, 255]);
            }
            rgba
        }
        tiff::decoder::DecodingResult::I16(data) => {
            let mut rgba = Vec::with_capacity((w * h * 4) as usize);
            for &pixel in &data {
                let p = ((pixel as i32 + 32768) / 257) as u8;
                rgba.extend_from_slice(&[p, p, p, 255]);
            }
            rgba
        }
        tiff::decoder::DecodingResult::I32(data) => {
            let mut rgba = Vec::with_capacity((w * h * 4) as usize);
            for &pixel in &data {
                let p = ((pixel as i64 + 2147483648) / 16843009) as u8;
                rgba.extend_from_slice(&[p, p, p, 255]);
            }
            rgba
        }
        tiff::decoder::DecodingResult::I64(data) => {
            let mut rgba = Vec::with_capacity((w * h * 4) as usize);
            for &pixel in &data {
                let p = ((pixel as i128 + 9223372036854775808) / 72340172838076673) as u8;
                rgba.extend_from_slice(&[p, p, p, 255]);
            }
            rgba
        }
        tiff::decoder::DecodingResult::F16(data) => {
            let mut rgba = Vec::with_capacity((w * h * 4) as usize);
            for &pixel in &data {
                let f32_val = pixel.to_f32();
                let clamped = f32_val.clamp(0.0, 1.0);
                let p = (clamped * 255.0) as u8;
                rgba.extend_from_slice(&[p, p, p, 255]);
            }
            rgba
        }
    };

    Ok(rgb_data)
}
