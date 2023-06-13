use crate::types::rpc::WriteBlockRequest;
use crate::types::rpc::WriteBlockResponse;

#[async_trait::async_trait]
pub trait SpeakerOps {
    /// Write out block.
    async fn write_block<T: Clone + serde::Serialize>(&self, request: WriteBlockRequest<T>) -> anyhow::Result<WriteBlockResponse>;
}
