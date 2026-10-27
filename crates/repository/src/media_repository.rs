use crate::{
    config::{db::DatabaseConfig, storage::StorageClient},
    error::to_error,
};
use app_core::media::{
    IMediaRepository, MAX_VIDEO_SIZE, MediaDetails, MediaOrder, PART_SIZE, UploadRequest,
    UploadSession,
};
use app_error::Error;
use async_trait::async_trait;
use aws_sdk_s3::{
    presigning::PresigningConfig,
    types::{CompletedMultipartUpload, CompletedPart},
};
use serde_json::json;
use std::{sync::Arc, time::Duration};
use uuid::Uuid;

pub struct MediaRepository {
    db: Arc<DatabaseConfig>,
    storage: StorageClient,
    bucket: String,
}

struct Session {
    key: String,
    upload: String,
    size: i64,
    mime: String,
    status: String,
}

#[async_trait]
impl IMediaRepository for MediaRepository {
    async fn reorder(&self, project: Uuid, order: MediaOrder) -> Result<(), Error> {
        use std::collections::HashSet;
        let ids: HashSet<_> = order.ids.iter().copied().collect();
        if order.ids.len() > 5000
            || ids.len() != order.ids.len()
            || ids != order.previous.iter().copied().collect()
            || order.ids.len() != order.previous.len()
        {
            return Err(invalid(
                "The order must contain each project media exactly once.",
            ));
        }
        let data = json!(order.ids);
        let mut conn = self.db.connect().await.map_err(|e| to_error(e, None))?;
        let saved = conn.with_transaction(async |txn| {
            let project_rows = txn.query_params("SELECT id FROM project WHERE id=$1 FOR UPDATE", &[&project]).await?;
            if project_rows.into_rows().is_empty() { return Ok(false); }
            let rows = txn.query_params("SELECT id FROM project_content WHERE project_id=$1 ORDER BY COALESCE((content->'details'->>'position')::int,0),created_on,id FOR UPDATE", &[&project]).await?;
            let current: Vec<Uuid> = rows.into_rows().iter().map(|row| row.get_by_name("id")).collect::<Result<_,_>>()?;
            if current != order.previous { return Ok(false); }
            txn.query_params("UPDATE project_content pc SET content=jsonb_set(pc.content,'{details}',COALESCE(pc.content->'details','{}'::jsonb)||jsonb_build_object('position',ord.position-1)) FROM jsonb_array_elements_text($2) WITH ORDINALITY AS ord(id,position) WHERE pc.project_id=$1 AND pc.id::text=ord.id", &[&project,&data]).await?;
            Ok(true)
        }).await.map_err(|e| to_error(e, None))?;
        if !saved {
            return Err(invalid(
                "Media changed since this page was loaded. Reload before reordering.",
            ));
        }
        Ok(())
    }
    async fn begin(&self, project: Uuid, request: UploadRequest) -> Result<UploadSession, Error> {
        self.begin_inner(project, request).await
    }
    async fn part_url(&self, project: Uuid, session: Uuid, part: i32) -> Result<String, Error> {
        self.part_url_inner(project, session, part).await
    }
    async fn edit(&self, project: Uuid, content: Uuid, details: MediaDetails) -> Result<(), Error> {
        self.edit_inner(project, content, details).await
    }
    async fn complete(&self, project: Uuid, session: Uuid) -> Result<Uuid, Error> {
        let mut conn = self.db.connect().await.map_err(|e| to_error(e, None))?;
        let lock = session.to_string();
        conn.with_transaction(async |txn| {
            txn.query_params(
                "SELECT pg_advisory_xact_lock(hashtextextended($1,0))",
                &[&lock],
            )
            .await?;
            Ok(self.complete_inner(project, session).await)
        })
        .await
        .map_err(|e| to_error(e, None))?
    }
    async fn abort(&self, project: Uuid, session: Uuid) -> Result<(), Error> {
        let mut conn = self.db.connect().await.map_err(|e| to_error(e, None))?;
        let lock = session.to_string();
        conn.with_transaction(async |txn| {
            txn.query_params(
                "SELECT pg_advisory_xact_lock(hashtextextended($1,0))",
                &[&lock],
            )
            .await?;
            Ok(self.abort_inner(project, session).await)
        })
        .await
        .map_err(|e| to_error(e, None))?
    }
}

fn invalid(message: &str) -> Error {
    Error::InvalidInput(message.into())
}

impl MediaRepository {
    pub fn new(db: Arc<DatabaseConfig>, storage: StorageClient, bucket: String) -> Self {
        Self {
            db,
            storage,
            bucket,
        }
    }

    async fn session(&self, project: Uuid, id: Uuid) -> Result<Session, Error> {
        let mut conn = self.db.connect().await.map_err(|e| to_error(e, None))?;
        let rows = conn.query_params("SELECT * FROM media_upload WHERE id=$1 AND project_id=$2 AND (expires_at > NOW() OR status='complete')", &[&id, &project]).await.map_err(|e| to_error(e, None))?;
        let row = rows
            .into_rows()
            .into_iter()
            .next()
            .ok_or_else(|| Error::EntityNotFound("Upload expired or not found".into()))?;
        Ok(Session {
            key: row
                .get_by_name("object_key")
                .map_err(|e| to_error(e, None))?,
            upload: row
                .get_by_name("upload_id")
                .map_err(|e| to_error(e, None))?,
            size: row
                .get_by_name("file_size")
                .map_err(|e| to_error(e, None))?,
            mime: row
                .get_by_name("mime_type")
                .map_err(|e| to_error(e, None))?,
            status: row.get_by_name("status").map_err(|e| to_error(e, None))?,
        })
    }

    async fn cleanup_expired(&self) -> Result<(), Error> {
        let mut conn = self.db.connect().await.map_err(|e| to_error(e, None))?;
        let rows = conn.query("SELECT id,object_key,upload_id FROM media_upload WHERE status='pending' AND expires_at <= NOW() LIMIT 20").await.map_err(|e| to_error(e, None))?;
        for row in rows.into_rows() {
            let id: Uuid = row.get_by_name("id").map_err(|e| to_error(e, None))?;
            let key: String = row
                .get_by_name("object_key")
                .map_err(|e| to_error(e, None))?;
            let upload: String = row
                .get_by_name("upload_id")
                .map_err(|e| to_error(e, None))?;
            if self
                .storage
                .inner
                .abort_multipart_upload()
                .bucket(&self.bucket)
                .key(&key)
                .upload_id(&upload)
                .send()
                .await
                .is_ok()
            {
                conn.query_params(
                    "UPDATE media_upload SET status='cancelled' WHERE id=$1 AND status='pending'",
                    &[&id],
                )
                .await
                .map_err(|e| to_error(e, None))?;
            }
        }
        Ok(())
    }
}

impl MediaRepository {
    async fn begin_inner(
        &self,
        project: Uuid,
        request: UploadRequest,
    ) -> Result<UploadSession, Error> {
        let video = matches!(request.mime_type.as_str(), "video/mp4" | "video/webm");
        let image = matches!(
            request.mime_type.as_str(),
            "image/jpeg" | "image/png" | "image/webp" | "image/avif" | "image/gif"
        );
        if !video && !image {
            return Err(invalid(
                "Use MP4/WebM video or JPEG/PNG/WebP/AVIF/GIF images.",
            ));
        }
        if request.size <= 0
            || request.size
                > if video {
                    MAX_VIDEO_SIZE
                } else {
                    32 * 1024 * 1024
                }
        {
            return Err(invalid(
                "Maximum file size: 2 GiB for video, 32 MiB for images.",
            ));
        }
        let mut conn = self.db.connect().await.map_err(|e| to_error(e, None))?;
        if conn
            .query_params("SELECT id FROM project WHERE id=$1", &[&project])
            .await
            .map_err(|e| to_error(e, None))?
            .into_rows()
            .is_empty()
        {
            return Err(Error::EntityNotFound("Project not found".into()));
        }
        // Best effort bounded cleanup; storage lifecycle rules handle abandoned sessions as well.
        if let Err(error) = self.cleanup_expired().await {
            tracing::warn!(%error, "upload cleanup failed");
        }
        let id = Uuid::new_v4();
        let key = format!("projects/{project}/{id}/{}", request.file_name);
        let created = self
            .storage
            .inner
            .create_multipart_upload()
            .bucket(&self.bucket)
            .key(&key)
            .content_type(&request.mime_type)
            .metadata("upload-session", id.to_string())
            .send()
            .await
            .map_err(|_| Error::StorageUpload)?;
        let upload = created.upload_id().ok_or(Error::StorageUpload)?.to_owned();
        let result = conn.query_params("INSERT INTO media_upload(id,project_id,object_key,upload_id,mime_type,file_size) VALUES($1,$2,$3,$4,$5,$6)", &[&id,&project,&key,&upload,&request.mime_type,&request.size]).await;
        if let Err(error) = result {
            let _ = self
                .storage
                .inner
                .abort_multipart_upload()
                .bucket(&self.bucket)
                .key(&key)
                .upload_id(&upload)
                .send()
                .await;
            return Err(to_error(error, None));
        }
        Ok(UploadSession {
            id,
            part_size: PART_SIZE,
            part_count: ((request.size + PART_SIZE - 1) / PART_SIZE) as i32,
        })
    }

    async fn part_url_inner(&self, project: Uuid, id: Uuid, part: i32) -> Result<String, Error> {
        let session = self.session(project, id).await?;
        if session.status != "pending"
            || part < 1
            || i64::from(part) > (session.size + PART_SIZE - 1) / PART_SIZE
        {
            return Err(invalid("Invalid upload part or session state."));
        }
        let length = PART_SIZE.min(session.size - i64::from(part - 1) * PART_SIZE);
        let signed = self
            .storage
            .inner
            .upload_part()
            .bucket(&self.bucket)
            .key(&session.key)
            .upload_id(&session.upload)
            .part_number(part)
            .content_length(length)
            .presigned(
                PresigningConfig::expires_in(Duration::from_secs(900))
                    .map_err(|_| Error::StorageUpload)?,
            )
            .await
            .map_err(|_| Error::StorageUpload)?;
        Ok(signed.uri().to_owned())
    }

    async fn complete_inner(&self, project: Uuid, id: Uuid) -> Result<Uuid, Error> {
        let session = self.session(project, id).await?;
        if session.status == "complete" {
            return Ok(id);
        }
        if session.status != "pending" {
            return Err(invalid("Upload was cancelled."));
        }
        // HEAD also recovers a completed object when the previous DB commit/response failed.
        let existing = self
            .storage
            .inner
            .head_object()
            .bucket(&self.bucket)
            .key(&session.key)
            .send()
            .await;
        if existing.is_err() {
            let listed = self
                .storage
                .inner
                .list_parts()
                .bucket(&self.bucket)
                .key(&session.key)
                .upload_id(&session.upload)
                .send()
                .await
                .map_err(|_| Error::StorageUpload)?;
            let count = (session.size + PART_SIZE - 1) / PART_SIZE;
            if listed.is_truncated() == Some(true) || listed.parts().len() as i64 != count {
                return Err(invalid("Upload is incomplete. Retry the missing parts."));
            }
            let mut parts = Vec::new();
            for (index, part) in listed.parts().iter().enumerate() {
                let number = index as i32 + 1;
                let expected = PART_SIZE.min(session.size - i64::from(number - 1) * PART_SIZE);
                if part.part_number() != Some(number)
                    || part.size() != Some(expected)
                    || part.e_tag().is_none()
                {
                    return Err(invalid(
                        "Uploaded part size does not match the declared file.",
                    ));
                }
                parts.push(
                    CompletedPart::builder()
                        .part_number(number)
                        .e_tag(part.e_tag().unwrap_or_default())
                        .build(),
                );
            }
            self.storage
                .inner
                .complete_multipart_upload()
                .bucket(&self.bucket)
                .key(&session.key)
                .upload_id(&session.upload)
                .multipart_upload(
                    CompletedMultipartUpload::builder()
                        .set_parts(Some(parts))
                        .build(),
                )
                .send()
                .await
                .map_err(|_| Error::StorageUpload)?;
        }
        let head = self
            .storage
            .inner
            .head_object()
            .bucket(&self.bucket)
            .key(&session.key)
            .send()
            .await
            .map_err(|_| Error::StorageUpload)?;
        if head.content_length() != Some(session.size)
            || head.content_type() != Some(session.mime.as_str())
        {
            return Err(invalid("Uploaded object does not match the declared file."));
        }
        let content = json!({ "bucket_name": self.bucket, "file_name": session.key, "mime_type": session.mime });
        let mut conn = self.db.connect().await.map_err(|e| to_error(e, None))?;
        let saved = conn.with_transaction(async |txn| {
            txn.query_params("INSERT INTO project_content(id,project_id,content,created_on) SELECT id,project_id,$3,NOW() FROM media_upload WHERE id=$1 AND project_id=$2 AND status='pending' ON CONFLICT(id) DO NOTHING", &[&id,&project,&content]).await?;
            let rows = txn.query_params("UPDATE media_upload SET status='complete' WHERE id=$1 AND project_id=$2 AND status='pending' RETURNING id", &[&id,&project]).await?;
            Ok(!rows.into_rows().is_empty())
        }).await.map_err(|e| to_error(e, None))?;
        if !saved {
            let _ = self
                .storage
                .inner
                .delete_object()
                .bucket(&self.bucket)
                .key(&session.key)
                .send()
                .await;
            return Err(Error::EntityNotFound(
                "Project or upload no longer exists".into(),
            ));
        }
        Ok(id)
    }

    async fn abort_inner(&self, project: Uuid, id: Uuid) -> Result<(), Error> {
        let session = self.session(project, id).await?;
        if session.status == "complete" {
            return Err(invalid(
                "Upload already completed. Delete the media instead.",
            ));
        }
        if session.status == "cancelled" {
            return Ok(());
        }
        if self
            .storage
            .inner
            .head_object()
            .bucket(&self.bucket)
            .key(&session.key)
            .send()
            .await
            .is_ok()
        {
            return Err(invalid(
                "The file has already arrived. Retry finishing the upload before deleting the media.",
            ));
        }
        self.storage
            .inner
            .abort_multipart_upload()
            .bucket(&self.bucket)
            .key(&session.key)
            .upload_id(&session.upload)
            .send()
            .await
            .map_err(|_| Error::StorageUpload)?;
        let mut conn = self.db.connect().await.map_err(|e| to_error(e, None))?;
        conn.query_params(
            "UPDATE media_upload SET status='cancelled' WHERE id=$1 AND project_id=$2",
            &[&id, &project],
        )
        .await
        .map_err(|e| to_error(e, None))?;
        Ok(())
    }

    async fn edit_inner(
        &self,
        project: Uuid,
        id: Uuid,
        details: MediaDetails,
    ) -> Result<(), Error> {
        if details.caption.chars().count() > 2000
            || details.alt.chars().count() > 1000
            || !(0..=100_000).contains(&details.position)
        {
            return Err(invalid(
                "Caption: 2000 characters maximum; alternative text: 1000; position: 0–100000.",
            ));
        }
        let data = json!(details);
        let mut conn = self.db.connect().await.map_err(|e| to_error(e, None))?;
        let result = conn.query_params("UPDATE project_content SET content=jsonb_set(content,'{details}',$3) WHERE project_id=$1 AND id=$2 AND ($4::uuid IS NULL OR EXISTS(SELECT 1 FROM project_content poster WHERE poster.project_id=$1 AND poster.id=$4 AND (poster.content->>'mime_type' LIKE 'image/%' OR (poster.content->>'mime_type' IS NULL AND poster.content->>'file_name' ~* '\\.(png|jpe?g|gif|webp|avif|bmp)$')))) RETURNING id", &[&project,&id,&data,&details.poster_id]).await.map_err(|e| to_error(e, None))?;
        if result.into_rows().is_empty() {
            return Err(invalid(
                "Media not found, or the poster is not an image from this project.",
            ));
        }
        Ok(())
    }
}
