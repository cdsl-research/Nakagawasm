use async_trait::async_trait;

pub trait Metrics{}
/// 収集したメトリクスの永続化・再構築
#[async_trait]
pub trait CollectedMetricsRepository {
    async fn store(&self, metrics: &impl Metrics);
}
