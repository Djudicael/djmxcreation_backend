use std::sync::Arc;

use app_core::{
    dto::{
        content_dto::ContentDto, metadata_dto::MetadataDto, project_content_dto::ProjectContentDto,
        project_dto::ProjectDto, project_with_thumbnail_dto::ProjectWithThumbnailDto,
        projects_dto::ProjectsDto,
    },
    project::project_repository::IProjectRepository,
};
use app_error::Error;
use async_trait::async_trait;

use chrono::{DateTime, Utc};
use serde_json::{Value, json};

use uuid::Uuid;

use wasi_pg_client::Row;

use crate::{
    config::db::DatabaseConfig,
    entity::{
        project::Project, project_content::ProjectContent,
        project_with_thumbnail::ProjectWithThumbnail,
    },
    error::to_error,
};

pub struct ProjectRepository {
    config: Arc<DatabaseConfig>,
}

/// Extract an optional JSON column from a row.
fn json_opt(row: &Row, col: &str) -> Result<Option<Value>, Error> {
    row.get_by_name::<Option<Value>>(col)
        .map_err(|e| to_error(e, None))
}

/// Extract a column value from a row.
fn col<T: wasi_pg_client::pg_types::FromSql>(row: &Row, col_name: &str) -> Result<T, Error> {
    row.get_by_name(col_name).map_err(|e| to_error(e, None))
}

/// Extract an optional `DateTime<Utc>` column.
fn timestamp_opt(row: &Row, col_name: &str) -> Result<Option<DateTime<Utc>>, Error> {
    row.get_by_name::<Option<DateTime<Utc>>>(col_name)
        .map_err(|e| to_error(e, None))
}

/// Extract a required `DateTime<Utc>` column.
fn timestamp(row: &Row, col_name: &str) -> Result<DateTime<Utc>, Error> {
    row.get_by_name(col_name).map_err(|e| to_error(e, None))
}

impl ProjectRepository {
    pub fn new(config: Arc<DatabaseConfig>) -> Self {
        Self { config }
    }

    fn map_row_to_project(row: &Row, with_thumbnail: bool) -> Result<Project, Error> {
        let thumbnail_content = if with_thumbnail {
            json_opt(row, "thumbnail_content")?
        } else {
            None
        };

        Ok(Project {
            id: Some(col(row, "id")?),
            metadata: json_opt(row, "metadata")?,
            created_on: timestamp_opt(row, "created_on")?,
            updated_on: timestamp_opt(row, "updated_on")?,
            description: json_opt(row, "description")?,
            visible: col(row, "visible")?,
            adult: col(row, "adult")?,
            contents: vec![],
            thumbnail_content,
        })
    }

    fn map_row_to_project_with_thumbnail(row: &Row) -> Result<ProjectWithThumbnail, Error> {
        Ok(ProjectWithThumbnail {
            id: Some(col(row, "id")?),
            metadata: json_opt(row, "metadata")?,
            created_on: timestamp(row, "created_on")?,
            updated_on: timestamp_opt(row, "updated_on")?,
            description: json_opt(row, "description")?,
            visible: col(row, "visible")?,
            adult: col(row, "adult")?,
            thumbnail_content: json_opt(row, "thumbnail_content")?,
            thumbnail_created_on: timestamp_opt(row, "thumbnail_created_on")?,
        })
    }

    fn map_row_to_project_content(row: &Row) -> Result<ProjectContent, Error> {
        Ok(ProjectContent::new(
            Some(col(row, "id")?),
            col(row, "project_id")?,
            json_opt(row, "content")?,
            timestamp_opt(row, "created_on")?,
        ))
    }
}

#[cfg_attr(not(target_arch = "wasm32"), async_trait)]
#[cfg_attr(target_arch = "wasm32", async_trait(?Send))]
impl IProjectRepository for ProjectRepository {
    async fn create(&self, metadata: &MetadataDto) -> Result<ProjectDto, Error> {
        let metadata_json = json!(metadata);
        let now_utc: DateTime<Utc> = Utc::now();
        let sql = "INSERT INTO project (metadata, created_on, visible, adult) VALUES ($1, $2, $3, $4) RETURNING *";

        let mut conn = self.config.connect().await.map_err(|e| to_error(e, None))?;
        let project = conn
            .with_transaction(async |txn| {
                let row = txn
                    .query_params(sql, &[&metadata_json, &now_utc, &true, &false])
                    .await?
                    .into_rows()
                    .into_iter()
                    .next()
                    .ok_or_else(|| wasi_pg_client::PgError::UnexpectedNull {
                        column: "id".to_string(),
                    })?;

                let project = ProjectRepository::map_row_to_project(&row, false).map_err(|e| {
                    wasi_pg_client::PgError::TypeConversion(
                        wasi_pg_client::pg_types::Error::Conversion(e.to_string()),
                    )
                })?;
                Ok(ProjectDto::from(project))
            })
            .await
            .map_err(|e| to_error(e, None))?;

        Ok(project)
    }

    async fn add_project_content(
        &self,
        project_id: Uuid,
        content: &ContentDto,
    ) -> Result<ProjectContentDto, Error> {
        let content_json = json!(content);
        let now_utc: DateTime<Utc> = Utc::now();
        let sql = "INSERT INTO project_content(project_id, content, created_on) VALUES($1, $2, $3) RETURNING *";

        let mut conn = self.config.connect().await.map_err(|e| to_error(e, None))?;
        let content_entity = conn
            .with_transaction(async |txn| {
                let row = txn
                    .query_params(sql, &[&project_id, &content_json, &now_utc])
                    .await?
                    .into_rows()
                    .into_iter()
                    .next()
                    .ok_or_else(|| wasi_pg_client::PgError::UnexpectedNull {
                        column: "id".to_string(),
                    })?;

                ProjectRepository::map_row_to_project_content(&row).map_err(|e| {
                    wasi_pg_client::PgError::TypeConversion(
                        wasi_pg_client::pg_types::Error::Conversion(e.to_string()),
                    )
                })
            })
            .await
            .map_err(|e| to_error(e, Some(project_id.to_string())))?;

        Ok(ProjectContentDto::from(content_entity))
    }

    async fn add_project_thumbnail(
        &self,
        project_id: Uuid,
        thumbnail: &ContentDto,
    ) -> Result<ProjectContentDto, Error> {
        let thumbnail_json = json!(thumbnail);
        let now_utc: DateTime<Utc> = Utc::now();
        let sql = "INSERT INTO project_content_thumbnail (content, project_id, created_on)
        VALUES ($1, $2, $3)
        ON CONFLICT (project_id) DO UPDATE
        SET content = EXCLUDED.content, created_on = EXCLUDED.created_on
        RETURNING *;
        ";

        let mut conn = self.config.connect().await.map_err(|e| to_error(e, None))?;
        let content_entity = conn
            .with_transaction(async |txn| {
                let row = txn
                    .query_params(sql, &[&thumbnail_json, &project_id, &now_utc])
                    .await?
                    .into_rows()
                    .into_iter()
                    .next()
                    .ok_or_else(|| wasi_pg_client::PgError::UnexpectedNull {
                        column: "id".to_string(),
                    })?;

                ProjectRepository::map_row_to_project_content(&row).map_err(|e| {
                    wasi_pg_client::PgError::TypeConversion(
                        wasi_pg_client::pg_types::Error::Conversion(e.to_string()),
                    )
                })
            })
            .await
            .map_err(|e| to_error(e, Some(project_id.to_string())))?;

        Ok(ProjectContentDto::from(content_entity))
    }

    async fn get_project_by_id(&self, id: Uuid) -> Result<Option<ProjectDto>, Error> {
        let sql = "SELECT
        p.id,
        p.metadata,
        p.created_on,
        p.updated_on,
        p.description,
        p.visible,
        p.adult,
        c.content AS thumbnail_content,
        array_agg(json_build_object('id', ct.id, 'content', ct.content, 'project_id',ct.project_id)) AS contents
    FROM
        project p
    LEFT JOIN
        project_content_thumbnail c ON c.project_id = p.id
    LEFT JOIN
        project_content ct ON ct.project_id = p.id
    WHERE
        p.id = $1
    GROUP BY
        p.id,
        c.content";

        let mut conn = self.config.connect().await.map_err(|e| to_error(e, None))?;

        let row = conn
            .query_params(sql, &[&id])
            .await
            .map_err(|e| to_error(e, Some(format!("Project not found for id: {}", id))))?
            .into_rows()
            .into_iter()
            .next();

        match row {
            Some(row) => {
                let project = ProjectRepository::map_row_to_project(&row, true)?;
                Ok(Some(ProjectDto::from(project)))
            }
            None => Ok(None),
        }
    }

    async fn get_projects(&self) -> Result<Vec<ProjectDto>, Error> {
        let sql = "SELECT
        p.id,
        p.metadata,
        p.created_on,
        p.updated_on,
        p.description,
        p.visible,
        p.adult,
        c.content AS thumbnail_content,
        array_agg(json_build_object('id', ct.id, 'content', ct.content, 'project_id',ct.project_id)) AS contents
    FROM
        project p
    LEFT JOIN
        project_content_thumbnail c ON c.project_id = p.id
    LEFT JOIN
        project_content ct ON ct.project_id = p.id
    GROUP BY
        p.id,
        c.content";

        let mut conn = self.config.connect().await.map_err(|e| to_error(e, None))?;
        let result = conn.query(sql).await.map_err(|e| to_error(e, None))?;

        result
            .iter()
            .map(|row| ProjectRepository::map_row_to_project(row, true).map(ProjectDto::from))
            .collect()
    }

    async fn get_projects_with_filter(
        &self,
        page: i64,
        size: i64,
        is_adult: Option<bool>,
        is_visible: bool,
    ) -> Result<ProjectsDto, Error> {
        if size <= 0 {
            return Err(Error::InvalidInput(
                "Page size must be greater than 0".to_string(),
            ));
        }
        if page <= 0 {
            return Err(Error::InvalidInput(
                "Page number must be greater than 0".to_string(),
            ));
        }

        let sql = "SELECT p.id, p.metadata, p.created_on, p.updated_on, p.description, p.visible, p.adult,
            COALESCE(c.content, ct.content) AS thumbnail_content,
            COALESCE(c.created_on, ct.created_on) AS thumbnail_created_on
        FROM project p
        LEFT JOIN project_content_thumbnail c ON c.project_id = p.id
        LEFT JOIN project_content ct ON ct.project_id = p.id AND ct.id = (
            SELECT id FROM project_content
            WHERE project_id = p.id ORDER BY created_on ASC LIMIT 1
        )
        WHERE p.visible = $1
          AND ($4::boolean IS NULL OR p.adult = $4)
          AND (SELECT COUNT(*) FROM project_content WHERE project_id = p.id) > 0
        ORDER BY p.created_on DESC
        LIMIT $2 OFFSET $3";

        let total_sql = "SELECT COUNT(*) FROM project p
        WHERE p.visible = $1
          AND ($2::boolean IS NULL OR p.adult = $2)";

        let mut conn = self.config.connect().await.map_err(|e| to_error(e, None))?;

        let total_count: i64 = conn
            .query_params(total_sql, &[&is_visible, &is_adult])
            .await
            .map_err(|e| to_error(e, None))?
            .into_rows()
            .first()
            .and_then(|r| r.get(0).ok())
            .unwrap_or(0);

        let result = conn
            .query_params(sql, &[&is_visible, &size, &((page - 1) * size), &is_adult])
            .await
            .map_err(|e| to_error(e, None))?;

        let projects = result
            .iter()
            .map(|row| ProjectRepository::map_row_to_project_with_thumbnail(row))
            .map(|r| r.map(ProjectWithThumbnailDto::from))
            .collect::<Result<Vec<ProjectWithThumbnailDto>, Error>>()?;

        let total_pages = if size <= 0 || total_count == 0 {
            0
        } else {
            (total_count + size - 1) / size
        };

        Ok(ProjectsDto::new(page, size, total_pages, projects))
    }

    async fn update_project_entity(
        &self,
        project_id: Uuid,
        project: &ProjectDto,
    ) -> Result<(), Error> {
        let now_utc: DateTime<Utc> = Utc::now();
        let metadata_json = json!(&project.metadata);

        let sql = "UPDATE project SET description = $1, metadata = $2, visible = $3, adult= $4, updated_on = $5 WHERE id = $6";

        let mut conn = self.config.connect().await.map_err(|e| to_error(e, None))?;
        conn.execute_params(
            sql,
            &[
                &project.description,
                &metadata_json,
                &project.visible,
                &project.adult,
                &now_utc,
                &project_id,
            ],
        )
        .await
        .map_err(|e| to_error(e, Some(project_id.to_string())))?;

        Ok(())
    }

    async fn get_projects_contents(
        &self,
        project_id: Uuid,
    ) -> Result<Vec<ProjectContentDto>, Error> {
        let sql = "SELECT * FROM project_content where project_id = $1";

        let mut conn = self.config.connect().await.map_err(|e| to_error(e, None))?;
        let result = conn
            .query_params(sql, &[&project_id])
            .await
            .map_err(|e| to_error(e, Some(project_id.to_string())))?;

        result
            .iter()
            .map(|r| ProjectRepository::map_row_to_project_content(r))
            .map(|r| r.map(ProjectContentDto::from))
            .collect()
    }

    async fn get_projects_content_by_id(
        &self,
        project_id: Uuid,
        id: Uuid,
    ) -> Result<Option<ProjectContentDto>, Error> {
        let sql = "SELECT * FROM project_content where project_id = $1 and id= $2";

        let mut conn = self.config.connect().await.map_err(|e| to_error(e, None))?;

        let row = conn
            .query_params(sql, &[&project_id, &id])
            .await
            .map_err(|e| to_error(e, Some(format!("Content not found for id: {}", id))))?
            .into_rows()
            .into_iter()
            .next();

        match row {
            Some(row) => {
                let content = ProjectRepository::map_row_to_project_content(&row)?;
                Ok(Some(ProjectContentDto::from(content)))
            }
            None => Ok(None),
        }
    }

    async fn delete_project_content_by_id(&self, project_id: Uuid, id: Uuid) -> Result<(), Error> {
        let sql = "DELETE FROM project_content WHERE id = $1 and project_id = $2 ";

        let mut conn = self.config.connect().await.map_err(|e| to_error(e, None))?;
        conn.execute_params(sql, &[&id, &project_id])
            .await
            .map_err(|e| to_error(e, Some(project_id.to_string())))?;

        Ok(())
    }

    async fn delete_project_by_id(&self, project_id: Uuid) -> Result<(), Error> {
        let sql = "DELETE FROM project WHERE id = $1 ";

        let mut conn = self.config.connect().await.map_err(|e| to_error(e, None))?;
        conn.execute_params(sql, &[&project_id])
            .await
            .map_err(|e| to_error(e, Some(project_id.to_string())))?;

        Ok(())
    }

    async fn get_projects_content_thumbnail(
        &self,
        project_id: Uuid,
    ) -> Result<Vec<ProjectContentDto>, Error> {
        let sql = "SELECT * FROM project_content where project_id = $1 FETCH FIRST ROW ONLY";

        let mut conn = self.config.connect().await.map_err(|e| to_error(e, None))?;
        let result = conn
            .query_params(sql, &[&project_id])
            .await
            .map_err(|e| to_error(e, Some(project_id.to_string())))?;

        result
            .iter()
            .map(|r| ProjectRepository::map_row_to_project_content(r).map(ProjectContentDto::from))
            .collect()
    }

    async fn delete_thumbnail_by_id(&self, project_id: Uuid, id: Uuid) -> Result<(), Error> {
        let sql = "DELETE FROM project_content_thumbnail WHERE id = $1 and project_id = $2";

        let mut conn = self.config.connect().await.map_err(|e| to_error(e, None))?;
        conn.execute_params(sql, &[&id, &project_id])
            .await
            .map_err(|e| to_error(e, Some(project_id.to_string())))?;

        Ok(())
    }

    async fn get_thumbnail_by_id(
        &self,
        project_id: Uuid,
        id: Uuid,
    ) -> Result<Option<ProjectContentDto>, Error> {
        let sql = "SELECT * FROM project_content_thumbnail
        WHERE id = $1 AND project_id = $2";

        let mut conn = self.config.connect().await.map_err(|e| to_error(e, None))?;

        let row = conn
            .query_params(sql, &[&id, &project_id])
            .await
            .map_err(|e| to_error(e, Some(format!("Thumbnail not found for id: {}", id))))?
            .into_rows()
            .into_iter()
            .next();

        match row {
            Some(row) => {
                let content = ProjectRepository::map_row_to_project_content(&row)?;
                Ok(Some(ProjectContentDto::from(content)))
            }
            None => Ok(None),
        }
    }
}
