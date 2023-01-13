use app_core::{
    dto::{
        content_dto::ContentDto, metadata_dto::MetadataDto, project_content_dto::ProjectContentDto,
        project_dto::ProjectDto,
    },
    project::project_repository::IProjectRepository,
};
use app_error::Error;
use async_trait::async_trait;
use chrono::{DateTime, Utc};
use serde_json::json;
use sqlx::types::Json;

use crate::{
    config::db::Db,
    entity::{project::Project, project_content::ProjectContent},
};

pub struct ProjectRepository {
    db: Db,
}

impl ProjectRepository {
    pub fn new(db: Db) -> Self {
        Self { db }
    }
}

#[async_trait]
impl IProjectRepository for ProjectRepository {
    async fn create(&self, metadata: &MetadataDto) -> Result<ProjectDto, Error> {
        let metadata_json = Json(json!(metadata));
        let now_utc: DateTime<Utc> = Utc::now();
        let sql =
            "INSERT INTO project(metadata, created_on, visible) VALUES($1, $2, $3) RETURNING *";
        let query = sqlx::query_as::<_, Project>(sql)
            .bind(metadata_json)
            .bind(now_utc)
            .bind(false);
        let project = ProjectDto::from(query.fetch_one(&self.db).await?);
        Ok(project)
    }

    async fn add_project_content(
        &self,
        project_id: i32,
        content: &ContentDto,
    ) -> Result<ProjectContentDto, Error> {
        let content_json = Json(json!(content));
        let now_utc: DateTime<Utc> = Utc::now();
        let sql = "INSERT INTO project_content(project_id, content, created_on) VALUES($1, $2, $3) RETURNING *";
        let query = sqlx::query_as::<_, ProjectContent>(sql)
            .bind(project_id)
            .bind(content_json)
            .bind(now_utc);
        let content_entity =
            query
                .fetch_one(&self.db)
                .await
                .map_err(|sqlx_error| match sqlx_error {
                    sqlx::Error::RowNotFound => Error::EntityNotFound(project_id.to_string()),
                    other => Error::Sqlx(other),
                })?;
        Ok(ProjectContentDto::from(content_entity))
    }

    async fn add_project_thumbnail(
        &self,
        _project_id: i32,
        _thumbnail: &ContentDto,
    ) -> Result<(), Error> {
        todo!()
    }

    async fn get_project_by_id(&self, id: i32) -> Result<ProjectDto, Error> {
        let sql = "SELECT * FROM project where id= $1";
        let query = sqlx::query_as::<_, Project>(sql).bind(id);
        let project = query
            .fetch_one(&self.db)
            .await
            .map_err(|sqlx_error| match sqlx_error {
                sqlx::Error::RowNotFound => Error::EntityNotFound(id.to_string()),
                other => Error::Sqlx(other),
            })?;
        Ok(ProjectDto::from(project))
    }

    async fn get_projects(&self) -> Result<Vec<ProjectDto>, Error> {
        let sql = "SELECT * FROM project";
        let query = sqlx::query_as::<_, Project>(sql);
        let projects = query.fetch_all(&self.db).await?;
        Ok(projects
            .iter()
            .map(|p| ProjectDto::from(p.clone()))
            .collect())
    }

    async fn update_project_entity(
        &self,
        project_id: i32,
        project: &ProjectDto,
    ) -> Result<(), Error> {
        let mut tx = self.db.begin().await?;
        let now_utc: DateTime<Utc> = Utc::now();
        sqlx::query("UPDATE project SET description = $1, metadata = $2, visible = $3, updated_on = $4 WHERE id = $5 ")
            .bind(project.clone().description)
            .bind(project.clone().metadata.map(|metadata| Json(json!(metadata))))
            .bind(project.visible)
            .bind(now_utc)
            .bind(project_id)
            .execute(&mut tx)
            .await .map_err(|sqlx_error| match sqlx_error {
                sqlx::Error::RowNotFound => Error::EntityNotFound(project_id.to_string()),
                other => Error::Sqlx(other),
            })?;

        tx.commit().await?;

        Ok(())
    }

    async fn get_projects_contents(
        &self,
        project_id: i32,
    ) -> Result<Vec<ProjectContentDto>, Error> {
        let sql = "SELECT * FROM project_content where project_id = $1";
        let query = sqlx::query_as::<_, ProjectContent>(sql).bind(project_id);
        let contents = query
            .fetch_all(&self.db)
            .await
            .map_err(|sqlx_error| match sqlx_error {
                sqlx::Error::RowNotFound => Error::EntityNotFound(project_id.to_string()),
                other => Error::Sqlx(other),
            })?;
        Ok(contents
            .iter()
            .map(|c| ProjectContentDto::from(c.clone()))
            .collect())
    }

    async fn get_projects_content_by_id(
        &self,
        project_id: i32,
        id: i32,
    ) -> Result<ProjectContentDto, Error> {
        let sql = "SELECT * FROM project_content where project_id = $1 and id= $2";
        let query = sqlx::query_as::<_, ProjectContent>(sql)
            .bind(project_id)
            .bind(id);
        let content = query
            .fetch_one(&self.db)
            .await
            .map_err(|sqlx_error| match sqlx_error {
                sqlx::Error::RowNotFound => Error::EntityNotFound(id.to_string()),
                other => Error::Sqlx(other),
            })?;

        Ok(ProjectContentDto::from(content))
    }

    async fn delete_project_content_by_id(&self, project_id: i32, id: i32) -> Result<(), Error> {
        let mut tx = self.db.begin().await?;
        sqlx::query("DELETE FROM project_content WHERE id = $1 and project_id = $2 ")
            .bind(id)
            .bind(project_id)
            .execute(&mut tx)
            .await
            .map_err(|sqlx_error| match sqlx_error {
                sqlx::Error::RowNotFound => Error::EntityNotFound(project_id.to_string()),
                other => Error::Sqlx(other),
            })?;
        tx.commit().await?;
        Ok(())
    }

    async fn delete_project_by_id(&self, project_id: i32) -> Result<(), Error> {
        let mut tx = self.db.begin().await?;
        sqlx::query("DELETE FROM project WHERE id = $1 ")
            .bind(project_id)
            .execute(&mut tx)
            .await
            .map_err(|sqlx_error| match sqlx_error {
                sqlx::Error::RowNotFound => Error::EntityNotFound(project_id.to_string()),
                other => Error::Sqlx(other),
            })?;
        tx.commit().await?;
        Ok(())
    }

    //TODO modify for thumbnail
    async fn get_projects_content_thumbnail(
        &self,
        project_id: i32,
    ) -> Result<Vec<ProjectContentDto>, Error> {
        let sql = "SELECT * FROM project_content where project_id = $1 FETCH FIRST ROW ONLY";
        let query = sqlx::query_as::<_, ProjectContent>(sql).bind(project_id);
        let contents = query
            .fetch_all(&self.db)
            .await
            .map_err(|sqlx_error| match sqlx_error {
                sqlx::Error::RowNotFound => Error::EntityNotFound(project_id.to_string()),
                other => Error::Sqlx(other),
            })?;
        Ok(contents
            .iter()
            .map(|c| ProjectContentDto::from(c.clone()))
            .collect())
    }
}
