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
use serde_json::json;
use sqlx::types::Json;

use crate::{
    config::db::Db,
    entity::{
        project::Project, project_content::ProjectContent,
        project_with_thumbnail::ProjectWithThumbnail,
    },
    error::to_error,
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
        println!("metadata: {:?}", metadata);
        let metadata_json = Json(json!(metadata));
        let now_utc: DateTime<Utc> = Utc::now();
        let sql =
            "INSERT INTO project(metadata, created_on, visible, adult) VALUES($1, $2, $3, $4) RETURNING *";
        let query = sqlx::query_as::<_, Project>(sql)
            .bind(metadata_json)
            .bind(now_utc)
            .bind(false)
            .bind(false);
        let project = ProjectDto::from(
            query
                .fetch_one(&self.db)
                .await
                .map_err(|sqlx_error| to_error(sqlx_error, None))?,
        );
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
        let content_entity = query
            .fetch_one(&self.db)
            .await
            .map_err(|sqlx_error| to_error(sqlx_error, Some(project_id.to_string())))?;
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
            .map_err(|sqlx_error| to_error(sqlx_error, Some(id.to_string())))?;
        Ok(ProjectDto::from(project))
    }

    async fn get_projects(&self) -> Result<Vec<ProjectDto>, Error> {
        let sql = "SELECT * FROM project";
        let query = sqlx::query_as::<_, Project>(sql);
        let projects = query
            .fetch_all(&self.db)
            .await
            .map_err(|sqlx_error| to_error(sqlx_error, None))?;
        Ok(projects
            .iter()
            .map(|p| ProjectDto::from(p.clone()))
            .collect())
    }

    async fn get_projects_with_filter(
        &self,
        page: i32,
        size: i32,
        is_adult: Option<bool>,
        is_visible: bool,
    ) -> Result<ProjectsDto, Error> {
        let adult_filter = match is_adult {
            Some(adult) => format!("AND p.adult = {}", adult),
            None => "".to_owned(),
        };

        // Use optional parameter syntax to conditionally include the `is_adult` parameter
        let sql = format!(
            "SELECT p.id, p.metadata, p.created_on, p.updated_on, p.description, p.visible, p.adult, ct.content AS thumbnail_content, ct.created_on AS thumbnail_created_on
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
            {}
            ORDER BY p.created_on DESC
            LIMIT $2 OFFSET $3",
            adult_filter
        );

        // Construct the total count SQL query
        let total_sql = format!(
            "SELECT COUNT(*)
        FROM project p
        WHERE p.visible = $1
        {}",
            adult_filter
        );

        let total_count: i64 = sqlx::query_scalar(&total_sql)
            .bind(is_visible)
            .bind(&is_adult)
            .fetch_one(&self.db)
            .await
            .map_err(|sqlx_error| to_error(sqlx_error, None))?;

        let rows = sqlx::query_as::<_, ProjectWithThumbnail>(&sql)
            .bind(is_visible)
            .bind(size)
            .bind((page - 1) * size)
            .fetch_all(&self.db)
            .await
            .map_err(|sqlx_error| to_error(sqlx_error, None))?;

        let projects = rows
            .iter()
            .map(|p| ProjectWithThumbnailDto::from(p.clone()))
            .collect();

        // Calculate the total number of pages based on the total count and page size
        let total_pages = ((total_count as f64) / (size as f64)).ceil() as i32;

        // Create a new ProjectsDto instance with the relevant information
        let projects_dto = ProjectsDto::new(page, size, total_pages, projects);

        Ok(projects_dto)
    }

    async fn update_project_entity(
        &self,
        project_id: i32,
        project: &ProjectDto,
    ) -> Result<(), Error> {
        let mut tx = self
            .db
            .begin()
            .await
            .map_err(|sqlx_error| to_error(sqlx_error, Some(project_id.to_string())))?;
        let now_utc: DateTime<Utc> = Utc::now();
        let ProjectDto {
            metadata,
            description,
            visible,
            adult,
            ..
        } = project.clone();
        sqlx::query("UPDATE project SET description = $1, metadata = $2, visible = $3, adult= $4, updated_on = $5 WHERE id = $6 ")
            .bind(description)
            .bind(metadata.map(|metadata| Json(json!(metadata))))
            .bind(visible)
            .bind(adult)
            .bind(now_utc)
            .bind(project_id)
            .execute(&mut tx)
            .await.map_err(|sqlx_error| to_error(sqlx_error, Some(project_id.to_string())))?;

        tx.commit()
            .await
            .map_err(|sqlx_error| to_error(sqlx_error, Some(project_id.to_string())))?;

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
            .map_err(|sqlx_error| to_error(sqlx_error, Some(project_id.to_string())))?;
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
            .map_err(|sqlx_error| to_error(sqlx_error, Some(project_id.to_string())))?;

        Ok(ProjectContentDto::from(content))
    }

    async fn delete_project_content_by_id(&self, project_id: i32, id: i32) -> Result<(), Error> {
        let mut tx = self
            .db
            .begin()
            .await
            .map_err(|sqlx_error| to_error(sqlx_error, Some(project_id.to_string())))?;
        sqlx::query("DELETE FROM project_content WHERE id = $1 and project_id = $2 ")
            .bind(id)
            .bind(project_id)
            .execute(&mut tx)
            .await
            .map_err(|sqlx_error| to_error(sqlx_error, Some(project_id.to_string())))?;
        tx.commit()
            .await
            .map_err(|sqlx_error| to_error(sqlx_error, Some(project_id.to_string())))?;
        Ok(())
    }

    async fn delete_project_by_id(&self, project_id: i32) -> Result<(), Error> {
        let mut tx = self
            .db
            .begin()
            .await
            .map_err(|sqlx_error| to_error(sqlx_error, Some(project_id.to_string())))?;
        sqlx::query("DELETE FROM project WHERE id = $1 ")
            .bind(project_id)
            .execute(&mut tx)
            .await
            .map_err(|sqlx_error| to_error(sqlx_error, Some(project_id.to_string())))?;
        tx.commit()
            .await
            .map_err(|sqlx_error| to_error(sqlx_error, Some(project_id.to_string())))?;
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
            .map_err(|sqlx_error| to_error(sqlx_error, Some(project_id.to_string())))?;
        Ok(contents
            .iter()
            .map(|c| ProjectContentDto::from(c.clone()))
            .collect())
    }
}
