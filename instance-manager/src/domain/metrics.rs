use async_trait::async_trait;
use tokio::process::Child;

macro_rules! regex {
    ($re:literal $(,)?) => {{
        static RE: once_cell::sync::OnceCell<regex::Regex> = once_cell::sync::OnceCell::new();
        RE.get_or_init(|| regex::Regex::new($re).unwrap())
    }};
}

#[async_trait]
pub trait MemoryUsage {
    async fn memory_usage(&self) -> anyhow::Result<u32>;
}

#[async_trait]
impl MemoryUsage for Child {
    async fn memory_usage(&self) -> anyhow::Result<u32> {
        let pid = self.id().ok_or_else(|| anyhow::anyhow!(""))?;
        let s = tokio::fs::read_to_string(format!("/proc/{}/smaps", pid)).await?;
        Ok(regex!(r"Private_((Clean)|(Dirty)):\s*(\d+)\skB")
            .captures_iter(&s)
            .map(|cap| cap.get(4).unwrap())
            .map(|m| m.as_str().parse::<u32>().unwrap())
            .sum())
    }
}
