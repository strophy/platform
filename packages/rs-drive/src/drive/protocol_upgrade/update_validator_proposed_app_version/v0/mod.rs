use crate::drive::grove_operations::BatchInsertApplyType;
use crate::drive::object_size_info::PathKeyElementInfo;

use crate::drive::batch::grovedb_op_batch::GroveDbOpBatchV0Methods;
use crate::drive::protocol_upgrade::{desired_version_for_validators_path, versions_counter_path};
use crate::drive::Drive;
use crate::error::drive::DriveError;
use crate::error::Error;
use crate::error::Error::GroveDB;
use crate::fee::op::LowLevelDriveOperation;

use dpp::util::deserializer::ProtocolVersion;
use dpp::version::drive_versions::DriveVersion;

use grovedb::{Element, TransactionArg};
use integer_encoding::VarInt;

impl Drive {
    /// Update the validator proposed app version
    /// returns true if the value was changed, or is new
    /// returns false if it was not changed
    pub(super) fn update_validator_proposed_app_version_v0(
        &self,
        validator_pro_tx_hash: [u8; 32],
        version: ProtocolVersion,
        transaction: TransactionArg,
        drive_version: &DriveVersion,
    ) -> Result<bool, Error> {
        let mut batch_operations: Vec<LowLevelDriveOperation> = vec![];
        let inserted = self.update_validator_proposed_app_version_operations(
            validator_pro_tx_hash,
            version,
            transaction,
            &mut batch_operations,
            drive_version,
        )?;

        let grove_db_operations =
            LowLevelDriveOperation::grovedb_operations_batch(&batch_operations);
        if !grove_db_operations.is_empty() {
            self.apply_batch_grovedb_operations(
                None,
                transaction,
                grove_db_operations,
                &mut vec![],
                drive_version,
            )?;
        }
        Ok(inserted)
    }
    /// Update the validator proposed app version
    /// returns true if the value was changed, or is new
    /// returns false if it was not changed
    pub(crate) fn update_validator_proposed_app_version_operations_v0(
        &self,
        validator_pro_tx_hash: [u8; 32],
        version: ProtocolVersion,
        transaction: TransactionArg,
        drive_operations: &mut Vec<LowLevelDriveOperation>,
        drive_version: &DriveVersion,
    ) -> Result<bool, Error> {
        let mut cache = self.cache.write().unwrap();
        let maybe_version_counter = &mut cache.protocol_versions_counter;

        let version_counter = if let Some(version_counter) = maybe_version_counter {
            version_counter
        } else {
            *maybe_version_counter =
                Some(self.fetch_versions_with_counter(transaction, drive_version)?);
            maybe_version_counter.as_mut().unwrap()
        };

        let path = desired_version_for_validators_path();
        let version_bytes = version.encode_var_vec();
        let version_element = Element::new_item(version_bytes.clone());

        let (value_changed, previous_element) = self.batch_insert_if_changed_value(
            PathKeyElementInfo::PathFixedSizeKeyRefElement((
                path,
                validator_pro_tx_hash.as_slice(),
                version_element,
            )),
            BatchInsertApplyType::StatefulBatchInsert,
            transaction,
            drive_operations,
            drive_version,
        )?;

        // if we will insert we need to add it to the version counter
        if value_changed {
            // if we had a different previous version we need to remove it from the version counter
            if let Some(previous_element) = previous_element {
                let previous_version_bytes = previous_element.as_item_bytes().map_err(GroveDB)?;
                let previous_version = ProtocolVersion::decode_var(previous_version_bytes)
                    .ok_or(Error::Drive(DriveError::CorruptedElementType(
                        "encoded value could not be decoded",
                    )))
                    .map(|(value, _)| value)?;
                //we should remove 1 from the previous version
                let previous_count =
                    version_counter
                        .get_mut(&previous_version)
                        .ok_or(Error::Drive(DriveError::CorruptedCacheState(
                            "trying to lower the count of a version from cache that is not found"
                                .to_string(),
                        )))?;
                if previous_count == &0 {
                    return Err(Error::Drive(DriveError::CorruptedCacheState(
                        "trying to lower the count of a version from cache that is already at 0"
                            .to_string(),
                    )));
                }
                *previous_count -= 1;
                self.batch_insert(
                    PathKeyElementInfo::PathFixedSizeKeyRefElement((
                        versions_counter_path(),
                        previous_version_bytes,
                        Element::new_item(previous_count.encode_var_vec()),
                    )),
                    drive_operations,
                    drive_version,
                )?;
            }

            let version_count = version_counter.entry(version).or_default();
            if version_count == &u64::MAX {
                return Err(Error::Drive(DriveError::CorruptedCacheState(
                    "trying to raise the count of a version from cache that is already at max"
                        .to_string(),
                )));
            }
            *version_count += 1;
            self.batch_insert(
                PathKeyElementInfo::PathFixedSizeKeyRefElement((
                    versions_counter_path(),
                    version_bytes.as_slice(),
                    Element::new_item(version_count.encode_var_vec()),
                )),
                drive_operations,
                drive_version,
            )?;
        }

        Ok(value_changed)
    }
}
