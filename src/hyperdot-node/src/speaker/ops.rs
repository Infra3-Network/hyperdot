use crate::types::WriteBlockRequest;
use crate::types::WriteBlockResponse;

#[async_trait::async_trait]
pub trait SpeakerOps {
    /// Write out block.
    async fn write_block(&self, block: WriteBlockRequest) -> anyhow::Result<WriteBlockResponse>;
}
