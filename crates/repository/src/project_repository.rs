use std::{future::Future, pin::Pin, sync::Arc, time::SystemTime};

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
use deadpool_postgres::PoolError;
use serde_json::{json, Value};

use tokio_postgres::{types::Json, Row, Transaction};
use uuid::Uuid;

use crate::{
    config::db::DatabasePool,
    entity::{
        project::Project, project_content::ProjectContent,
        project_with_thumbnail::ProjectWithThumbnail,
    },
    error::to_error,
};

pub struct ProjectRepository {
    client: Arc<DatabasePool>,
}

impl ProjectRepository {
    pub fn new(client: Arc<DatabasePool>) -> Self {
        Self { client }
    }

    fn map_create_row_to_project_without_thumbnail_content(row: &Row) -> Result<Project, Error> {
        let metadata: Option<serde_json::Value> = row
            .try_get::<_, Option<Json<Value>>>("metadata")
            .map_err(|e| to_error(PoolError::Backend(e), None))?
            .map(|json| json.0);

        let description: Option<serde_json::Value> = row
            .try_get::<_, Option<Json<Value>>>("description")
            .map_err(|e| to_error(PoolError::Backend(e), None))?
            .map(|json| json.0);

        let updated_on: Option<SystemTime> = row
            .try_get("updated_on")
            .map_err(|e| to_error(PoolError::Backend(e), None))?;
        let updated_on: Option<DateTime<Utc>> = updated_on.map(|time| time.into());

        let created_on: Option<SystemTime> = row
            .try_get("created_on")
            .map_err(|e| to_error(PoolError::Backend(e), None))?;
        let created_on: Option<DateTime<Utc>> = created_on.map(|time| time.into());

        let id: Uuid = row
            .try_get("id")
            .map_err(|e| to_error(PoolError::Backend(e), None))?;

        let visible: bool = row
            .try_get("visible")
            .map_err(|e| to_error(PoolError::Backend(e), None))?;

        let adult: bool = row
            .try_get("adult")
            .map_err(|e| to_error(PoolError::Backend(e), None))?;
        Ok(Project {
            id: Some(id),
            metadata,
            created_on,
            updated_on,
            description,
            visible,
            adult,
            contents: vec![],
            thumbnail_content: None,
        })
    }
    fn map_create_row_to_project(row: &Row) -> Result<Project, Error> {
        let metadata: Option<serde_json::Value> = row
            .try_get::<_, Option<Json<Value>>>("metadata")
            .map_err(|e| to_error(PoolError::Backend(e), None))?
            .map(|json| json.0);

        let description: Option<serde_json::Value> = row
            .try_get::<_, Option<Json<Value>>>("description")
            .map_err(|e| to_error(PoolError::Backend(e), None))?
            .map(|json| json.0);

        let thumbnail_content: Option<serde_json::Value> = row
            .try_get::<_, Option<Json<Value>>>("thumbnail_content")
            .map_err(|e| to_error(PoolError::Backend(e), None))?
            .map(|json| json.0);

        let updated_on: Option<SystemTime> = row
            .try_get("updated_on")
            .map_err(|e| to_error(PoolError::Backend(e), None))?;
        let updated_on: Option<DateTime<Utc>> = updated_on.map(|time| time.into());

        let created_on: Option<SystemTime> = row
            .try_get("created_on")
            .map_err(|e| to_error(PoolError::Backend(e), None))?;
        let created_on: Option<DateTime<Utc>> = created_on.map(|time| time.into());

        let id: Uuid = row
            .try_get("id")
            .map_err(|e| to_error(PoolError::Backend(e), None))?;

        let visible: bool = row
            .try_get("visible")
            .map_err(|e| to_error(PoolError::Backend(e), None))?;

        let adult: bool = row
            .try_get("adult")
            .map_err(|e| to_error(PoolError::Backend(e), None))?;
        Ok(Project {
            id: Some(id),
            metadata,
            created_on,
            updated_on,
            description,
            visible,
            adult,
            contents: vec![],
            thumbnail_content,
        })
    }

    fn map_row_to_project_with_thumbnail(row: &Row) -> Result<ProjectWithThumbnail, Error> {
        let metadata: Option<Value> = row
            .try_get::<_, Option<Json<Value>>>("metadata")
            .map_err(|e| to_error(PoolError::Backend(e), None))?
            .map(|json| json.0);

        let description: Option<Value> = row
            .try_get::<_, Option<Json<Value>>>("description")
            .map_err(|e| to_error(PoolError::Backend(e), None))?
            .map(|json| json.0);

        let thumbnail_content: Option<Value> = row
            .try_get::<_, Option<Json<Value>>>("thumbnail_content")
            .map_err(|e| to_error(PoolError::Backend(e), None))?
            .map(|json| json.0);

        let updated_on: Option<SystemTime> = row
            .try_get("updated_on")
            .map_err(|e| to_error(PoolError::Backend(e), None))?;
        let updated_on: Option<DateTime<Utc>> = updated_on.map(|time| time.into());

        let created_on: SystemTime = row
            .try_get("created_on")
            .map_err(|e| to_error(PoolError::Backend(e), None))?;
        let created_on: DateTime<Utc> = created_on.into();

        let thumbnail_created_on: Option<SystemTime> = row
            .try_get("thumbnail_created_on")
            .map_err(|e| to_error(PoolError::Backend(e), None))?;
        let thumbnail_created_on: Option<DateTime<Utc>> =
            thumbnail_created_on.map(|time| time.into());

        let id: Uuid = row
            .try_get("id")
            .map_err(|e| to_error(PoolError::Backend(e), None))?;

        let visible: bool = row
            .try_get("visible")
            .map_err(|e| to_error(PoolError::Backend(e), None))?;

        let adult: bool = row
            .try_get("adult")
            .map_err(|e| to_error(PoolError::Backend(e), None))?;

        Ok(ProjectWithThumbnail {
            id: Some(id),
            metadata,
            created_on,
            updated_on,
            description,
            visible,
            adult,
            thumbnail_content,
            thumbnail_created_on,
        })
    }

    fn map_row_to_project_content(row: &Row) -> Result<ProjectContent, Error> {
        let content: Option<Value> = row
            .try_get::<_, Option<Json<Value>>>("content")
            .map_err(|e| to_error(PoolError::Backend(e), None))?
            .map(|json| json.0);

        let created_on: Option<SystemTime> = row
            .try_get("created_on")
            .map_err(|e| to_error(PoolError::Backend(e), None))?;
        let created_on: Option<DateTime<Utc>> = created_on.map(|time| time.into());

        let id: Uuid = row
            .try_get("id")
            .map_err(|e| to_error(PoolError::Backend(e), None))?;

        let project_id: Uuid = row
            .try_get("project_id")
            .map_err(|e| to_error(PoolError::Backend(e), None))?;

        Ok(ProjectContent::new(
            Some(id),
            project_id,
            content,
            created_on,
        ))
    }

    async fn with_transaction<F, T>(&self, f: F) -> Result<T, Error>
    where
        F: for<'a> FnOnce(
            &'a Transaction<'a>,
        ) -> Pin<Box<dyn Future<Output = Result<T, Error>> + Send + 'a>>,
    {
        let mut client = self
            .client
            .get_client()
            .await
            .map_err(|e| to_error(e, None))?;

        let transaction = client
            .build_transaction()
            .start()
            .await
            .map_err(|e| to_error(PoolError::Backend(e), None))?;

        let result = f(&transaction).await?;

        transaction
            .commit()
            .await
            .map_err(|e| to_error(PoolError::Backend(e), None))?;

        Ok(result)
    }
}

#[async_trait]
impl IProjectRepository for ProjectRepository {
    async fn create(&self, metadata: &MetadataDto) -> Result<ProjectDto, Error> {
        let metadata_json = json!(metadata);
        let now_utc: DateTime<Utc> = Utc::now();
        let sql = "INSERT INTO project (metadata, created_on, visible, adult) VALUES ($1, $2, $3, $4) RETURNING *";

        let project = self
            .with_transaction(|tx| {
                Box::pin(async move {
                    let row = tx
                        .query_one(sql, &[&Json(metadata_json), &now_utc, &true, &false])
                        .await
                        .map_err(|e| to_error(PoolError::Backend(e), None))?;
                    ProjectRepository::map_create_row_to_project_without_thumbnail_content(&row)
                        .map(ProjectDto::from)
                })
            })
            .await?;

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
        let content_entity = self
            .with_transaction(|tx| {
                Box::pin(async move {
                    let row = tx
                        .query_one(sql, &[&project_id, &Json(content_json), &now_utc])
                        .await
                        .map_err(|e| {
                            to_error(PoolError::Backend(e), Some(project_id.to_string()))
                        })?;
                    ProjectRepository::map_row_to_project_content(&row)
                })
            })
            .await?;
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
        let content_entity = self
            .with_transaction(|tx| {
                Box::pin(async move {
                    let row = tx
                        .query_one(sql, &[&Json(thumbnail_json), &project_id, &now_utc])
                        .await
                        .map_err(|e| {
                            to_error(PoolError::Backend(e), Some(project_id.to_string()))
                        })?;
                    ProjectRepository::map_row_to_project_content(&row)
                })
            })
            .await?;
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

        let client = self
            .client
            .get_client()
            .await
            .map_err(|e| to_error(e, None))?;
        match client.query_opt(sql, &[&id]).await.map_err(|e| {
            to_error(
                PoolError::Backend(e),
                Some(format!("Project not found for id: {}", id)),
            )
        })? {
            Some(row) => {
                let project = ProjectRepository::map_create_row_to_project(&row)?;
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

        let client = self
            .client
            .get_client()
            .await
            .map_err(|e| to_error(e, None))?;
        let rows = client
            .query(sql, &[])
            .await
            .map_err(|e| to_error(PoolError::Backend(e), None))?;

        let projects = rows
            .iter()
            .map(|row| ProjectRepository::map_create_row_to_project(row))
            .collect::<Result<Vec<Project>, Error>>()?;
        Ok(projects
            .iter()
            .map(|p| ProjectDto::from(p.clone()))
            .collect())
    }

    async fn get_projects_with_filter(
        &self,
        page: i64,
        size: i64,
        is_adult: Option<bool>,
        is_visible: bool,
    ) -> Result<ProjectsDto, Error> {
        // Validate input parameters
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
        let adult_filter = match is_adult {
            Some(adult) => format!("AND p.adult = {}", adult),
            None => "".to_owned(),
        };

        // Rest of the SQL query remains the same
        let sql = format!(
            "SELECT p.id, p.metadata, p.created_on, p.updated_on, p.description, p.visible, p.adult,
            COALESCE(c.content, ct.content) AS thumbnail_content,
            COALESCE(c.created_on, ct.created_on) AS thumbnail_created_on
        FROM project p
        LEFT JOIN project_content_thumbnail c ON c.project_id = p.id
        LEFT JOIN project_content ct ON ct.project_id = p.id AND ct.id = (
            SELECT id
            FROM project_content
            WHERE project_id = p.id
            ORDER BY created_on ASC
            LIMIT 1
        )
        WHERE p.visible = $1
        {adult_filter}
        AND (SELECT COUNT(*) FROM project_content WHERE project_id = p.id) > 0
        ORDER BY p.created_on DESC
        LIMIT $2 OFFSET $3"
        );

        // Rest of the implementation remains the same
        let total_sql = format!(
            "SELECT COUNT(*)
        FROM project p
        WHERE p.visible = $1
        {adult_filter}"
        );

        let client = self
            .client
            .get_client()
            .await
            .map_err(|e| to_error(e, None))?;

        let total_count: i64 = client
            .query_one(&total_sql, &[&is_visible])
            .await
            .map_err(|e| to_error(PoolError::Backend(e), None))?
            .get(0);

        let rows = client
            .query(&sql, &[&is_visible, &size, &((page - 1) * size)])
            .await
            .map_err(|e| to_error(PoolError::Backend(e), None))?;

        let projects = rows
            .iter()
            .map(|row| ProjectRepository::map_row_to_project_with_thumbnail(row))
            .map(|r| r.map(ProjectWithThumbnailDto::from))
            .collect::<Result<Vec<ProjectWithThumbnailDto>, Error>>()?;

        // Calculate total pages using i64 instead of f64
        let total_pages = if size <= 0 {
            0
        } else if total_count == 0 {
            0
        } else {
            (total_count + size - 1) / size
        };

        let projects_dto = ProjectsDto::new(page, size, total_pages, projects);

        Ok(projects_dto)
    }

    async fn update_project_entity(
        &self,
        project_id: Uuid,
        project: &ProjectDto,
    ) -> Result<(), Error> {
        let now_utc: DateTime<Utc> = Utc::now();
        let ProjectDto {
            metadata,
            description,
            visible,
            adult,
            ..
        } = project.clone();

        let sql="UPDATE project SET description = $1, metadata = $2, visible = $3, adult= $4, updated_on = $5 WHERE id = $6";
        let client = self
            .client
            .get_client()
            .await
            .map_err(|e| to_error(e, None))?;
        client
            .execute(
                sql,
                &[
                    &Json(description),
                    &Json(metadata),
                    &visible,
                    &adult,
                    &now_utc,
                    &project_id,
                ],
            )
            .await
            .map_err(|e| to_error(PoolError::Backend(e), Some(project_id.to_string())))?;

        Ok(())
    }

    async fn get_projects_contents(
        &self,
        project_id: Uuid,
    ) -> Result<Vec<ProjectContentDto>, Error> {
        let sql = "SELECT * FROM project_content where project_id = $1";
        let client = self
            .client
            .get_client()
            .await
            .map_err(|e| to_error(e, None))?;
        let row = client
            .query(sql, &[&project_id])
            .await
            .map_err(|e| to_error(PoolError::Backend(e), Some(project_id.to_string())))?;
        let contents = row
            .iter()
            .map(|r| ProjectRepository::map_row_to_project_content(r))
            .map(|r| r.map(ProjectContentDto::from))
            .collect::<Result<Vec<ProjectContentDto>, Error>>()?;
        Ok(contents)
    }

    async fn get_projects_content_by_id(
        &self,
        project_id: Uuid,
        id: Uuid,
    ) -> Result<Option<ProjectContentDto>, Error> {
        let sql = "SELECT * FROM project_content where project_id = $1 and id= $2";

        let client = self
            .client
            .get_client()
            .await
            .map_err(|e| to_error(e, None))?;
        match client
            .query_opt(sql, &[&project_id, &id])
            .await
            .map_err(|e| {
                to_error(
                    PoolError::Backend(e),
                    Some(format!("Content not found for id: {}", id)),
                )
            })? {
            Some(row) => {
                let content = ProjectRepository::map_row_to_project_content(&row)?;
                Ok(Some(ProjectContentDto::from(content)))
            }
            None => Ok(None),
        }
    }

    async fn delete_project_content_by_id(&self, project_id: Uuid, id: Uuid) -> Result<(), Error> {
        let sql = "DELETE FROM project_content WHERE id = $1 and project_id = $2 ";
        let client = self
            .client
            .get_client()
            .await
            .map_err(|e| to_error(e, None))?;
        client
            .execute(sql, &[&id, &project_id])
            .await
            .map_err(|e| to_error(PoolError::Backend(e), Some(project_id.to_string())))?;
        Ok(())
    }

    async fn delete_project_by_id(&self, project_id: Uuid) -> Result<(), Error> {
        let sql = "DELETE FROM project WHERE id = $1 ";
        let client = self
            .client
            .get_client()
            .await
            .map_err(|e| to_error(e, None))?;
        client
            .execute(sql, &[&project_id])
            .await
            .map_err(|e| to_error(PoolError::Backend(e), Some(project_id.to_string())))?;
        Ok(())
    }

    //TODO modify for thumbnail
    async fn get_projects_content_thumbnail(
        &self,
        project_id: Uuid,
    ) -> Result<Vec<ProjectContentDto>, Error> {
        let sql = "SELECT * FROM project_content where project_id = $1 FETCH FIRST ROW ONLY";

        let client = self
            .client
            .get_client()
            .await
            .map_err(|e| to_error(e, None))?;
        let row = client
            .query(sql, &[&project_id])
            .await
            .map_err(|e| to_error(PoolError::Backend(e), Some(project_id.to_string())))?;
        let contents = row
            .iter()
            .map(|r| ProjectRepository::map_row_to_project_content(r))
            .collect::<Result<Vec<ProjectContent>, Error>>()?;

        Ok(contents
            .iter()
            .map(|c| ProjectContentDto::from(c.clone()))
            .collect())
    }

    async fn delete_thumbnail_by_id(&self, project_id: Uuid, id: Uuid) -> Result<(), Error> {
        let sql = "DELETE FROM project_content_thumbnail WHERE id = $1 and project_id = $2";

        let client = self
            .client
            .get_client()
            .await
            .map_err(|e| to_error(e, None))?;
        client
            .execute(sql, &[&id, &project_id])
            .await
            .map_err(|e| to_error(PoolError::Backend(e), Some(project_id.to_string())))?;
        Ok(())
    }

    async fn get_thumbnail_by_id(
        &self,
        project_id: Uuid,
        id: Uuid,
    ) -> Result<Option<ProjectContentDto>, Error> {
        let sql = "SELECT * FROM project_content_thumbnail 
        WHERE id = $1 AND project_id = $2";

        let client = self
            .client
            .get_client()
            .await
            .map_err(|e| to_error(e, None))?;
        match client
            .query_opt(sql, &[&id, &project_id])
            .await
            .map_err(|e| {
                to_error(
                    PoolError::Backend(e),
                    Some(format!("Thumbnail not found for id: {}", id)),
                )
            })? {
            Some(row) => {
                let content = ProjectRepository::map_row_to_project_content(&row)?;
                Ok(Some(ProjectContentDto::from(content)))
            }
            None => Ok(None),
        }
    }
}
