mod v0;

use crate::drive::Drive;
use crate::error::drive::DriveError;
use crate::error::Error;
use crate::fee::op::LowLevelDriveOperation;
use dpp::identity::IdentityPublicKey;

use dpp::version::drive_versions::DriveVersion;
use platform_version::version::PlatformVersion;

impl Drive {
    /// Generates a vector of operations for inserting key to storage.
    ///
    /// # Arguments
    ///
    /// * `identity_id` - An array of bytes representing the identity id.
    /// * `identity_key` - A reference to the `IdentityPublicKey` struct.
    /// * `key_id_bytes` - The byte representation of the key id.
    /// * `drive_operations` - A mutable reference to a vector of `LowLevelDriveOperation` objects.
    /// * `drive_version` - The version of the drive.
    ///
    /// # Returns
    ///
    /// * `Result<(), Error>` - If successful, returns unit (`()`). If an error occurs during the operation, returns an `Error`.
    ///
    /// # Errors
    ///
    /// This function may return an `Error` if the operation creation process fails or if the drive version does not match any of the implemented method versions.
    pub fn insert_key_to_storage_operations(
        &self,
        identity_id: [u8; 32],
        identity_key: &IdentityPublicKey,
        key_id_bytes: &[u8],
        drive_operations: &mut Vec<LowLevelDriveOperation>,
        platform_version: &PlatformVersion,
    ) -> Result<(), Error> {
        match platform_version
            .drive
            .methods
            .identity
            .keys
            .insert
            .insert_key_to_storage
        {
            0 => self.insert_key_to_storage_operations_v0(
                identity_id,
                identity_key,
                key_id_bytes,
                drive_operations,
                platform_version,
            ),
            version => Err(Error::Drive(DriveError::UnknownVersionMismatch {
                method: "insert_key_to_storage_operations".to_string(),
                known_versions: vec![0],
                received: version,
            })),
        }
    }
}
