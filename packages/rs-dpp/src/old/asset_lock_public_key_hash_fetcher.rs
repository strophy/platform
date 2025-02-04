use std::sync::Arc;

use crate::identity::state_transition::asset_lock_proof::asset_lock_transaction_output_fetcher::AssetLockTransactionOutputFetcher;
use crate::identity::state_transition::asset_lock_proof::AssetLockProof;
use crate::state_repository::StateRepositoryLike;
use crate::state_transition::state_transition_execution_context::StateTransitionExecutionContext;
use crate::util::vec::vec_to_array;
use crate::DPPError;

pub struct AssetLockPublicKeyHashFetcher<SR> {
    state_repository: Arc<SR>,
    asset_lock_transaction_output_fetcher: Arc<AssetLockTransactionOutputFetcher<SR>>,
}

impl<SR> AssetLockPublicKeyHashFetcher<SR>
where
    SR: StateRepositoryLike,
{
    pub fn new(
        state_repository: Arc<SR>,
        asset_lock_transaction_output_fetcher: Arc<AssetLockTransactionOutputFetcher<SR>>,
    ) -> Self {
        Self {
            state_repository,
            asset_lock_transaction_output_fetcher,
        }
    }

    pub async fn fetch_public_key_hash(
        &self,
        asset_lock_proof: AssetLockProof,
        execution_context: &StateTransitionExecutionContext,
    ) -> Result<[u8; 20], DPPError> {
        let output = self
            .asset_lock_transaction_output_fetcher
            .fetch(&asset_lock_proof, execution_context)
            .await?;

        if output.script_pubkey.is_op_return() {
            let public_key_hash = &output.script_pubkey.as_bytes()[2..];
            vec_to_array(public_key_hash).map_err(|_| DPPError::WrongPublicKeyHashSize)
        } else {
            Err(DPPError::WrongBurnOutputType)
        }
    }
}
