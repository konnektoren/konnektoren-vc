use crate::certificate_data::CertificateData;
use crate::routes::v1;
use utoipa::OpenApi;

#[derive(OpenApi)]
#[openapi(
    paths(
        v1::send_certificate_and_get_qr,
    ),
    components(
        schemas(CertificateData)
    ),
    tags(
        (name = "certificate_v1", description = "Certificate issuance operations")
    )
)]
pub struct ApiDoc;
