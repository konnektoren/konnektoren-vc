use crate::storage::MemoryStorage;
use oid4vc_manager::managers::credential_issuer::CredentialIssuerManager;
use oid4vci::credential_format_profiles::{CredentialFormats, WithParameters};

pub type ManagerType = CredentialIssuerManager<MemoryStorage, CredentialFormats<WithParameters>>;
