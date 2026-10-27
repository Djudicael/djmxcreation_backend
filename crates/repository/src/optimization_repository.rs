use crate::{
    config::{db::DatabaseConfig, storage::StorageClient},
    error::to_error,
};
use app_core::optimization::{IMediaProcessing, ImageOptimization, ImageSource};
use app_error::Error;
use async_trait::async_trait;
use aws_sdk_s3::{presigning::PresigningConfig, primitives::ByteStream};
use serde_json::json;
use std::{collections::HashMap, sync::Arc, time::Duration};
use uuid::Uuid;

#[derive(Clone)]
pub struct OptimizationRepository {
    pub db: Arc<DatabaseConfig>,
    pub storage: StorageClient,
}
pub struct OptimizationJob {
    pub id: Uuid,
    pub token: Uuid,
    pub step: i32,
    pub bucket: String,
    pub key: String,
}
pub struct VariantWrite {
    pub target: i32,
    pub width: i32,
    pub height: i32,
    pub extension: &'static str,
    pub mime: &'static str,
    pub bytes: Vec<u8>,
}

impl OptimizationRepository {
    pub fn new(db: Arc<DatabaseConfig>, storage: StorageClient) -> Self {
        // Bound storage calls below the serverless request/cleanup grace budget.
        // Persistent retries belong to PostgreSQL, not an SDK background loop.
        let config = storage
            .inner
            .config()
            .to_builder()
            .timeout_config(
                aws_sdk_s3::config::timeout::TimeoutConfig::builder()
                    .operation_timeout(Duration::from_secs(10))
                    .build(),
            )
            .retry_config(aws_sdk_s3::config::retry::RetryConfig::standard().with_max_attempts(1))
            .build();
        Self {
            db,
            storage: StorageClient {
                inner: aws_sdk_s3::Client::from_conf(config),
            },
        }
    }

    pub async fn claim(&self) -> Result<Option<OptimizationJob>, Error> {
        let token = Uuid::new_v4();
        let mut conn = self.db.connect().await.map_err(|e| to_error(e, None))?;
        conn.with_transaction(async |txn| {
            txn.execute("UPDATE media_optimization SET status='failed',reason='Processing budget exhausted; retry from the admin',lease_token=NULL,lease_until=NULL WHERE status='processing' AND lease_until<NOW() AND attempts>=5").await?;
            let rows=txn.query("SELECT pc.id,pc.content,j.step FROM project_content pc JOIN media_optimization j ON j.media_id=pc.id WHERE (j.status IN ('pending','failed') AND j.attempts<5 AND j.next_attempt_at<=NOW()) OR (j.status='processing' AND j.lease_until<NOW()) ORDER BY j.next_attempt_at,pc.created_on LIMIT 1 FOR UPDATE OF pc,j SKIP LOCKED").await?;
            let Some(row)=rows.into_rows().into_iter().next() else { return Ok(None); };
            let id:Uuid=row.get_by_name("id")?;
            let content:serde_json::Value=row.get_by_name("content")?;
            let bucket=content["bucket_name"].as_str().unwrap_or_default().to_owned();
            let key=content["file_name"].as_str().unwrap_or_default().to_owned();
            let step=row.get_by_name("step")?;
            txn.query_params("UPDATE media_optimization SET status='processing',attempts=attempts+1,lease_token=$2,lease_until=NOW()+INTERVAL '5 minutes' WHERE media_id=$1", &[&id,&token]).await?;
            // Register every potential output BEFORE any upload. Deleted jobs can
            // therefore clean even an object whose PUT succeeded before a crash.
            txn.query_params("INSERT INTO media_asset(media_id,bucket,object_key,kind,target_width,mime_type) SELECT $1::uuid,$2,'optimized/v1/'||($1::uuid)::text||'/'||w::text||'.'||ext,'variant',w,mime FROM (VALUES(400),(800),(1600),(2560)) widths(w) CROSS JOIN (VALUES('jpg','image/jpeg'),('png','image/png'),('webp','image/webp')) formats(ext,mime) ON CONFLICT DO NOTHING", &[&id,&bucket]).await?;
            Ok(Some(OptimizationJob {id,token,step,bucket,key}))
        }).await.map_err(|e|to_error(e,None))
    }

    pub async fn original(&self, job: &OptimizationJob) -> Result<Vec<u8>, Error> {
        let object = self
            .storage
            .inner
            .get_object()
            .bucket(&job.bucket)
            .key(&job.key)
            .send()
            .await
            .map_err(|_| Error::StorageGetObjectUrl)?;
        if object
            .content_length()
            .is_none_or(|length| !(1..=32 * 1024 * 1024).contains(&length))
        {
            return Err(Error::InvalidInput(
                "Original exceeds processing limits".into(),
            ));
        }
        let mut stream = object.body;
        let mut bytes = Vec::new();
        while let Some(chunk) = stream
            .try_next()
            .await
            .map_err(|_| Error::StorageGetObjectUrl)?
        {
            if bytes.len() + chunk.len() > 32 * 1024 * 1024 {
                return Err(Error::InvalidInput(
                    "Original exceeds processing limits".into(),
                ));
            }
            bytes.extend_from_slice(&chunk);
        }
        Ok(bytes)
    }

    pub async fn store(&self, job: &OptimizationJob, output: VariantWrite) -> Result<bool, Error> {
        let key = format!(
            "optimized/v1/{}/{}.{}",
            job.id, output.target, output.extension
        );
        let mut conn = self.db.connect().await.map_err(|e| to_error(e, None))?;
        conn.with_transaction(async |txn| {
            // DELETE must wait for this short write transaction; CPU processing
            // happens outside it. A deleted/expired job can never publish a PUT.
            let row=txn.query_params("SELECT pc.id FROM project_content pc JOIN media_optimization j ON j.media_id=pc.id WHERE pc.id=$1 AND j.lease_token=$2 AND j.status='processing' AND j.lease_until>NOW() FOR UPDATE OF pc,j", &[&job.id,&job.token]).await?;
            if row.into_rows().is_empty() { return Ok(Ok(false)); }
            if self.storage.inner.put_object().bucket(&job.bucket).key(&key).content_type(output.mime).body(ByteStream::from(output.bytes)).send().await.is_err() { return Ok(Err(Error::StorageUpload)); }
            txn.query_params("UPDATE media_asset SET ready=TRUE,width=$3,height=$4 WHERE bucket=$1 AND object_key=$2", &[&job.bucket,&key,&output.width,&output.height]).await?;
            Ok(Ok(true))
        }).await.map_err(|e|to_error(e,None))?
    }

    pub async fn finish(
        &self,
        job: &OptimizationJob,
        done: bool,
        width: i32,
        height: i32,
    ) -> Result<(), Error> {
        let status = if done { "ready" } else { "pending" };
        let mut conn = self.db.connect().await.map_err(|e| to_error(e, None))?;
        conn.query_params("UPDATE media_optimization SET status=$3,step=step+1,attempts=0,lease_token=NULL,lease_until=NULL,next_attempt_at=NOW(),reason=NULL,original_width=$4,original_height=$5 WHERE media_id=$1 AND lease_token=$2 AND status='processing'", &[&job.id,&job.token,&status,&width,&height]).await.map_err(|e|to_error(e,None))?;
        Ok(())
    }

    pub async fn fail(&self, job: &OptimizationJob, reason: &str, skip: bool) -> Result<(), Error> {
        let status = if skip { "skipped" } else { "failed" };
        let reason: String = reason.chars().take(200).collect();
        let mut conn = self.db.connect().await.map_err(|e| to_error(e, None))?;
        conn.query_params("UPDATE media_optimization SET status=$3,reason=$4,lease_token=NULL,lease_until=NULL,next_attempt_at=NOW()+INTERVAL '30 seconds'*attempts WHERE media_id=$1 AND lease_token=$2", &[&job.id,&job.token,&status,&reason]).await.map_err(|e|to_error(e,None))?;
        Ok(())
    }

    pub async fn enqueue_existing(&self, limit: i64) -> Result<usize, Error> {
        if !app_core::optimization::enabled() {
            return Ok(0);
        }
        let mut conn = self.db.connect().await.map_err(|e| to_error(e, None))?;
        let rows=conn.query_params("SELECT enqueue_media_optimization(pc.id) FROM project_content pc LEFT JOIN media_optimization j ON j.media_id=pc.id WHERE j.media_id IS NULL AND (pc.content->>'mime_type' LIKE 'image/%' OR (pc.content->>'mime_type' IS NULL AND pc.content->>'file_name' ~* '[.](png|jpe?g|webp|avif|gif)$')) ORDER BY pc.created_on LIMIT $1", &[&limit.clamp(1,100)]).await.map_err(|e|to_error(e,None))?;
        Ok(rows.into_rows().len())
    }

    pub async fn enqueue_project(&self, project: Uuid) -> Result<usize, Error> {
        if !app_core::optimization::enabled() {
            return Ok(0);
        }
        let mut conn = self.db.connect().await.map_err(|e| to_error(e, None))?;
        let rows=conn.query_params("INSERT INTO media_optimization(media_id) SELECT id FROM project_content WHERE project_id=$1 AND (content->>'mime_type' LIKE 'image/%' OR (content->>'mime_type' IS NULL AND content->>'file_name' ~* '[.](png|jpe?g|webp|avif|gif)$')) ON CONFLICT(media_id) DO UPDATE SET status='pending',attempts=0,next_attempt_at=NOW(),lease_token=NULL,lease_until=NULL WHERE media_optimization.status='failed' RETURNING media_id", &[&project]).await.map_err(|e|to_error(e,None))?;
        Ok(rows.into_rows().len())
    }
}

#[async_trait]
impl IMediaProcessing for OptimizationRepository {
    async fn images(&self, ids: &[Uuid]) -> Result<HashMap<Uuid, ImageOptimization>, Error> {
        if !app_core::optimization::enabled() || ids.is_empty() {
            return Ok(HashMap::new());
        }
        let mut conn = self.db.connect().await.map_err(|e| to_error(e, None))?;
        let ids = json!(ids);
        let rows=conn.query_params("SELECT j.media_id,j.status,j.reason,j.original_width,j.original_height,a.object_key,a.bucket,a.width,a.height,a.mime_type FROM media_optimization j LEFT JOIN media_asset a ON a.media_id=j.media_id AND a.kind='variant' AND a.ready AND j.status='ready' WHERE j.media_id::text IN (SELECT jsonb_array_elements_text($1)) ORDER BY a.width", &[&ids]).await.map_err(|e|to_error(e,None))?;
        let mut result = HashMap::new();
        for row in rows.into_rows() {
            let id: Uuid = row.get_by_name("media_id").map_err(|e| to_error(e, None))?;
            let entry = result.entry(id).or_insert(ImageOptimization {
                reason: row.get_by_name("reason").map_err(|e| to_error(e, None))?,
                status: row.get_by_name("status").map_err(|e| to_error(e, None))?,
                width: row
                    .get_by_name("original_width")
                    .map_err(|e| to_error(e, None))?,
                height: row
                    .get_by_name("original_height")
                    .map_err(|e| to_error(e, None))?,
                sources: vec![],
            });
            let key: Option<String> = row
                .get_by_name("object_key")
                .map_err(|e| to_error(e, None))?;
            if let Some(key) = key {
                let bucket: String = row.get_by_name("bucket").map_err(|e| to_error(e, None))?;
                let signed = self
                    .storage
                    .inner
                    .get_object()
                    .bucket(bucket)
                    .key(key)
                    .presigned(
                        PresigningConfig::expires_in(Duration::from_secs(8640))
                            .map_err(|_| Error::StorageGetObjectUrl)?,
                    )
                    .await
                    .map_err(|_| Error::StorageGetObjectUrl)?;
                entry.sources.push(ImageSource {
                    url: signed.uri().into(),
                    width: row.get_by_name("width").map_err(|e| to_error(e, None))?,
                    height: row.get_by_name("height").map_err(|e| to_error(e, None))?,
                    mime_type: row
                        .get_by_name("mime_type")
                        .map_err(|e| to_error(e, None))?,
                });
            }
        }
        Ok(result)
    }

    async fn cleanup(&self, limit: usize) -> Result<usize, Error> {
        let token = Uuid::new_v4();
        let limit = limit.clamp(1, 32) as i64;
        let mut conn = self.db.connect().await.map_err(|e| to_error(e, None))?;
        let rows=conn.query_params("WITH due AS (SELECT bucket,object_key FROM media_cleanup WHERE available_at<=NOW() AND (lease_until IS NULL OR lease_until<NOW()) ORDER BY available_at LIMIT $1 FOR UPDATE SKIP LOCKED) UPDATE media_cleanup c SET lease_token=$2,lease_until=NOW()+INTERVAL '5 minutes' FROM due WHERE c.bucket=due.bucket AND c.object_key=due.object_key RETURNING c.bucket,c.object_key", &[&limit,&token]).await.map_err(|e|to_error(e,None))?;
        let mut deleted = 0;
        let started = std::time::Instant::now();
        for row in rows.into_rows() {
            // Unvisited claims become eligible again when their lease expires.
            if started.elapsed() >= Duration::from_secs(5) {
                break;
            }
            let bucket: String = row.get_by_name("bucket").map_err(|e| to_error(e, None))?;
            let key: String = row
                .get_by_name("object_key")
                .map_err(|e| to_error(e, None))?;
            if self
                .storage
                .inner
                .delete_object()
                .bucket(&bucket)
                .key(&key)
                .send()
                .await
                .is_ok()
            {
                conn.with_transaction(async |txn| {
                    txn.query_params("DELETE FROM media_asset WHERE bucket=$1 AND object_key=$2 AND EXISTS(SELECT 1 FROM media_cleanup WHERE bucket=$1 AND object_key=$2 AND lease_token=$3)", &[&bucket,&key,&token]).await?;
                    txn.query_params("DELETE FROM media_cleanup WHERE bucket=$1 AND object_key=$2 AND lease_token=$3", &[&bucket,&key,&token]).await?;
                    Ok(())
                }).await.map_err(|e|to_error(e,None))?;
                deleted += 1;
            } else {
                conn.query_params("UPDATE media_cleanup SET attempts=attempts+1,available_at=NOW()+INTERVAL '30 seconds'*LEAST(attempts+1,120),lease_token=NULL,lease_until=NULL WHERE bucket=$1 AND object_key=$2 AND lease_token=$3", &[&bucket,&key,&token]).await.map_err(|e|to_error(e,None))?;
            }
        }
        Ok(deleted)
    }

    async fn wake(&self) -> Result<bool, Error> {
        if !app_core::optimization::enabled() {
            return Ok(false);
        }
        let Ok(url) = std::env::var("MEDIA_WORKER_URL") else {
            return Ok(false);
        };
        let key = std::env::var("MEDIA_PROCESSING_TOKEN")
            .map_err(|_| Error::InvalidInput("Missing media processing token".into()))?;
        if key.len() < 32 {
            return Err(Error::InvalidInput(
                "Media processing token must contain at least 32 characters".into(),
            ));
        }
        #[cfg(not(target_arch = "wasm32"))]
        {
            let response = reqwest::Client::builder()
                .timeout(Duration::from_secs(60))
                .redirect(reqwest::redirect::Policy::none())
                .build()
                .map_err(|_| Error::StorageUpload)?
                .post(format!(
                    "{}/internal/media/process-due",
                    url.trim_end_matches('/')
                ))
                .header("X-Media-Worker-Key", key)
                .send()
                .await
                .map_err(|_| Error::StorageUpload)?;
            Ok(response.status().is_success())
        }
        #[cfg(target_arch = "wasm32")]
        {
            use wstd::http::{Client, Request, body::Body};
            let mut client = Client::new();
            client.set_connect_timeout(Duration::from_secs(5));
            client.set_first_byte_timeout(Duration::from_secs(60));
            let request = Request::post(format!(
                "{}/internal/media/process-due",
                url.trim_end_matches('/')
            ))
            .header("X-Media-Worker-Key", key)
            .body(Body::empty())
            .map_err(|_| Error::StorageUpload)?;
            Ok(client
                .send(request)
                .await
                .map_err(|_| Error::StorageUpload)?
                .status()
                .is_success())
        }
    }
}
