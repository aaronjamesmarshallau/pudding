use rusoto_core::Region;
use rusoto_credential::StaticProvider;

/// Represents all the metadata require to successfully create and access a
/// bucket
pub struct BucketMetadata {
    pub bucket_name: String,
    pub credentials: StaticProvider,
    pub region: Region,
}
