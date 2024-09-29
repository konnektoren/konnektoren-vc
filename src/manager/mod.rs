mod custom_credential_issuer;

use crate::storage::MemoryStorage;
use oid4vc_manager::managers::credential_issuer::CredentialIssuerManager;
use oid4vci::credential_format_profiles::{CredentialFormats, WithParameters};

pub use custom_credential_issuer::ConfigurableManager;

pub type ManagerType = CredentialIssuerManager<MemoryStorage, CredentialFormats<WithParameters>>;
