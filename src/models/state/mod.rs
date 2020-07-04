use s3::creds::Credentials;
use s3::region::Region;

/// Represents all the metadata require to successfully create and access a
/// bucket
pub struct BucketMetadata {
    pub bucket_name: &'static str,
    pub credentials: Credentials,
    pub region: Region,
}
