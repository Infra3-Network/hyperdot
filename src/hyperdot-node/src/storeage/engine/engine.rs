use std::any::Any;

#[async_trait::async_trait]
pub trait DataEngine: Send + Sync {
    fn name(&self) -> String;

    async fn write_block(
        &self,
        chain: String,
        blocks: Vec<Box<dyn Any + Send + Sync>>,
    ) -> anyhow::Result<()>;
}
