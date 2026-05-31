//! 条码/二维码生成

use ::image::{DynamicImage, ImageBuffer, Luma};
use printcraft_core::elements::BarcodeType;
use printcraft_core::error::{PrintCraftError, Result};
use qrcode::QrCode;

/// 生成条码/二维码图片
pub fn generate_barcode_image(code: &str, barcode_type: BarcodeType) -> Result<DynamicImage> {
    match barcode_type {
        BarcodeType::QRCode => generate_qrcode(code),
        _ => generate_placeholder(code),
    }
}

fn generate_qrcode(code: &str) -> Result<DynamicImage> {
    let qr = QrCode::new(code.as_bytes())
        .map_err(|e| PrintCraftError::Render(format!("QR code generation failed: {e}")))?;

    let qr_width = qr.width();
    let module_size = (200.0 / qr_width as f64).ceil() as u32;
    let img_size = qr_width as u32 * module_size;
    let colors = qr.to_colors();

    let mut img = ImageBuffer::from_pixel(img_size, img_size, Luma([255u8]));

    for y in 0..qr_width {
        for x in 0..qr_width {
            let color = colors[y * qr_width + x];
            if color == qrcode::Color::Dark {
                let px_start = x as u32 * module_size;
                let py_start = y as u32 * module_size;
                for dy in 0..module_size {
                    for dx in 0..module_size {
                        let px = px_start + dx;
                        let py = py_start + dy;
                        if px < img_size && py < img_size {
                            img.put_pixel(px, py, Luma([0u8]));
                        }
                    }
                }
            }
        }
    }

    Ok(DynamicImage::ImageLuma8(img))
}

fn generate_placeholder(code: &str) -> Result<DynamicImage> {
    let width = 200u32;
    let height = 80u32;
    let mut img = ImageBuffer::from_pixel(width, height, Luma([255u8]));

    let label = format!("[{code}]");
    render_ascii_text(&mut img, &label, 10, 30);
    Ok(DynamicImage::ImageLuma8(img))
}

fn render_ascii_text(img: &mut ImageBuffer<Luma<u8>, Vec<u8>>, text: &str, x: u32, y: u32) {
    for (i, ch) in text.chars().enumerate() {
        let glyph = get_glyph(ch);
        for gy in 0..5u32 {
            for gx in 0..3u32 {
                if glyph[gy as usize][gx as usize] {
                    let px = x + i as u32 * 4 + gx;
                    let py = y + gy;
                    if px < img.width() && py < img.height() {
                        img.put_pixel(px, py, Luma([0u8]));
                    }
                }
            }
        }
    }
}

fn get_glyph(ch: char) -> [[bool; 3]; 5] {
    match ch {
        '0' => [
            [true, true, true],
            [true, false, true],
            [true, false, true],
            [true, false, true],
            [true, true, true],
        ],
        '1' => [
            [false, true, false],
            [true, true, false],
            [false, true, false],
            [false, true, false],
            [true, true, true],
        ],
        '2' => [
            [true, true, true],
            [false, false, true],
            [true, true, true],
            [true, false, false],
            [true, true, true],
        ],
        '3' => [
            [true, true, true],
            [false, false, true],
            [true, true, true],
            [false, false, true],
            [true, true, true],
        ],
        '4' => [
            [true, false, true],
            [true, false, true],
            [true, true, true],
            [false, false, true],
            [false, false, true],
        ],
        '5' => [
            [true, true, true],
            [true, false, false],
            [true, true, true],
            [false, false, true],
            [true, true, true],
        ],
        '6' => [
            [true, true, true],
            [true, false, false],
            [true, true, true],
            [true, false, true],
            [true, true, true],
        ],
        '7' => [
            [true, true, true],
            [false, false, true],
            [false, true, false],
            [false, true, false],
            [false, true, false],
        ],
        '8' => [
            [true, true, true],
            [true, false, true],
            [true, true, true],
            [true, false, true],
            [true, true, true],
        ],
        '9' => [
            [true, true, true],
            [true, false, true],
            [true, true, true],
            [false, false, true],
            [true, true, true],
        ],
        '[' => [
            [true, true, false],
            [true, false, false],
            [true, false, false],
            [true, false, false],
            [true, true, false],
        ],
        ']' => [
            [false, true, true],
            [false, false, true],
            [false, false, true],
            [false, false, true],
            [false, true, true],
        ],
        _ => [[true; 3]; 5],
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_qrcode_generation() {
        let img = generate_barcode_image("https://example.com", BarcodeType::QRCode).unwrap();
        assert!(img.width() > 0);
        assert!(img.height() > 0);
    }

    #[test]
    fn test_placeholder_generation() {
        let img = generate_barcode_image("12345", BarcodeType::Code128).unwrap();
        assert_eq!(img.width(), 200);
        assert_eq!(img.height(), 80);
    }
}
