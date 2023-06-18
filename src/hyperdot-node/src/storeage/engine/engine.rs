use std::any::Any;

use hyperdot_common_config::Chain;

#[async_trait::async_trait]
pub trait DataEngine: Send + Sync {
    fn name(&self) -> String;

    async fn write_block(
        &self,
        chain: Chain,
        blocks: Vec<Box<dyn Any + Send + Sync>>,
    ) -> anyhow::Result<()>;
}
