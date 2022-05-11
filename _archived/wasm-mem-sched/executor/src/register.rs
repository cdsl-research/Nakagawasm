use std::{fmt::Debug, path::PathBuf};

use crate::instance::{InstanceSpec, InstanceStatus};
use async_trait::async_trait;
use chrono::Local;
use r2d2::Pool;
use redis::{Client, Commands};

const EXPIRE_SEC: usize = 60;

#[async_trait]
pub trait InstanceRegister: Debug + 'static + Send + Sync {
    async fn exists(&self, spec: &InstanceSpec) -> anyhow::Result<bool>;
    async fn register(&self, spec: &InstanceSpec) -> anyhow::Result<()>;
    async fn update(&self, uid: &uuid::Uuid, status: InstanceStatus) -> anyhow::Result<()>;
    async fn expire(&self, uid: &uuid::Uuid) -> anyhow::Result<()>;
}

#[derive(Debug, Clone)]
pub struct RedisRegister {
    pool: Pool<Client>,
}

#[derive(Debug, Clone)]
struct InstanceRegisterEntry {
    module_id: String,
    module_path: PathBuf,
    instance_id: String,
    status: InstanceStatus,
    created_at: chrono::DateTime<Local>,
}

impl InstanceRegisterEntry {
    pub fn new(
        module_id: String,
        module_path: PathBuf,
        instance_id: String,
        status: InstanceStatus,
        created_at: chrono::DateTime<Local>,
    ) -> Self {
        Self {
            module_id,
            module_path,
            instance_id,
            status,
            created_at,
        }
    }

    pub fn from_spec(spec: &InstanceSpec) -> Self {
        Self::new(
            spec.module.digest.clone(),
            spec.module.path.clone(),
            spec.uid.to_string(),
            InstanceStatus::Starting,
            chrono::Local::now(),
        )
    }

    pub fn into_vec_tup(&self) -> Vec<(String, String)> {
        vec![
            ("module_id".into(), self.module_id.clone()),
            (
                "module_path".into(),
                self.module_path.to_str().unwrap().to_string(),
            ),
            ("instance_id".into(), self.instance_id.clone()),
            ("status".into(), self.status.to_string()),
            ("created_at".into(), self.created_at.to_rfc3339()),
        ]
    }
}

impl RedisRegister {
    const PREFIX: &'static str = "instance:";

    pub fn new(params: &str) -> anyhow::Result<Self> {
        Ok(Self {
            pool: Pool::builder().build(Client::open(params)?)?,
        })
    }

    fn to_key(&self, uid: &uuid::Uuid) -> String {
        format!("{}{}", Self::PREFIX, uid)
    }
}

#[async_trait]
impl InstanceRegister for RedisRegister {
    async fn exists(&self, spec: &InstanceSpec) -> anyhow::Result<bool> {
        let pool = self.pool.clone();
        let key = self.to_key(&spec.uid);

        tokio::task::spawn_blocking(move || {
            let mut conn = pool.get()?;
            Ok(conn.exists(key)?)
        })
        .await?
    }

    async fn register(&self, spec: &InstanceSpec) -> anyhow::Result<()> {
        let pool = self.pool.clone();
        let key = self.to_key(&spec.uid);
        let entry = InstanceRegisterEntry::from_spec(&spec);

        tokio::task::spawn_blocking(move || {
            let mut conn = pool.get()?;
            let _ = conn.hset_multiple(key, &entry.into_vec_tup())?;
            Ok(())
        })
        .await?
    }

    async fn update(&self, uid: &uuid::Uuid, status: InstanceStatus) -> anyhow::Result<()> {
        let pool = self.pool.clone();
        let key = self.to_key(uid);

        tokio::task::spawn_blocking(move || {
            let mut conn = pool.get()?;
            let _ = conn.hset(key, "status", status.to_string())?;
            Ok(())
        })
        .await?
    }

    async fn expire(&self, uid: &uuid::Uuid) -> anyhow::Result<()> {
        let pool = self.pool.clone();
        let key = self.to_key(uid);

        tokio::task::spawn_blocking(move || {
            let mut conn = pool.get()?;
            let _ = conn.expire(key, EXPIRE_SEC)?;
            Ok(())
        })
        .await?
    }
}
