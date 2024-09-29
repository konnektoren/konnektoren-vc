use axum::{
    body::Body,
    extract::State,
    http::{header, StatusCode},
    response::{IntoResponse, Response},
    routing::get,
    Router,
};
use chrono::Utc;
use hyper::body::Bytes;
use image::{ImageBuffer, Rgb};
use qrcodegen::{QrCode, QrCodeEcc};
use std::io::Cursor;

use crate::services::{CertificateData, CertificateService};
use crate::storage::SharedStorage;

async fn generate_example_qr_image(
    State(storage): State<SharedStorage>,
) -> Result<impl IntoResponse, StatusCode> {
    // Create an example certificate
    let certificate_data = CertificateData {
        game_path_name: "Example Game".to_string(),
        total_challenges: 10,
        solved_challenges: 8,
        performance_percentage: 80,
        profile_name: "John Doe".to_string(),
        date: Utc::now(),
    };

    let service = CertificateService::new(storage);

    let qr_url = service
        .generate_offer_url(&certificate_data)
        .map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)?;

    // Generate QR code from the URL
    let qr = QrCode::encode_text(&qr_url, QrCodeEcc::Medium)
        .map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)?;

    // Set the scale factor to make each QR module larger
    let scale = 10;
    let padding = 40; // Add padding around the QR code
    let size = qr.size() as u32;
    let img_size = size * scale + 2 * padding;

    // Create a larger image from the QR code with padding
    let mut img: ImageBuffer<Rgb<u8>, Vec<u8>> = ImageBuffer::new(img_size, img_size);

    // Fill the entire image with white (including padding)
    for y in 0..img_size {
        for x in 0..img_size {
            img.put_pixel(x, y, Rgb([255u8, 255u8, 255u8]));
        }
    }

    // Draw the QR code on the image (with padding offset)
    for y in 0..size {
        for x in 0..size {
            if qr.get_module(x as i32, y as i32) {
                for dy in 0..scale {
                    for dx in 0..scale {
                        img.put_pixel(
                            x * scale + dx + padding,
                            y * scale + dy + padding,
                            Rgb([0u8, 0u8, 0u8]),
                        );
                    }
                }
            }
        }
    }

    // Convert the image to PNG
    let mut png_data = Vec::new();
    img.write_to(
        &mut Cursor::new(&mut png_data),
        image::ImageOutputFormat::Png,
    )
    .map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)?;

    // Convert png_data (Vec<u8>) to Body
    let body = Body::from(Bytes::from(png_data));

    // Create the response
    let response = Response::builder()
        .status(StatusCode::OK)
        .header(header::CONTENT_TYPE, "image/png")
        .body(body)
        .map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)?;

    Ok(response)
}

pub fn create_example_router() -> Router<SharedStorage> {
    Router::new().route("/qr", get(generate_example_qr_image))
}
