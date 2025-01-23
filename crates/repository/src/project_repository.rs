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
use serde_json::json;
use tokio::sync::Mutex;
use tokio_postgres::{Row, Transaction};

use crate::{
    config::db::ClientV2,
    entity::{
        project::Project, project_content::ProjectContent,
        project_with_thumbnail::ProjectWithThumbnail,
    },
    error::{handle_serde_json_error, to_error},
};

pub struct ProjectRepository {
    client: Arc<Mutex<ClientV2>>,
}

impl ProjectRepository {
    pub fn new(client: ClientV2) -> Self {
        Self {
            client: Arc::new(Mutex::new(client)),
        }
    }

    fn map_create_row_to_project(row: &Row) -> Result<Project, Error> {
        let metadata: Option<serde_json::Value> = row
            .get::<_, Option<String>>(1)
            .map(|s| serde_json::from_str(&s).map_err(|e| handle_serde_json_error(e)))
            .transpose()?;
        let description: Option<serde_json::Value> = row
            .get::<_, Option<String>>(1)
            .map(|s| serde_json::from_str(&s).map_err(|e| handle_serde_json_error(e)))
            .transpose()?;
        let thumbnail_content: Option<serde_json::Value> = row
            .get::<_, Option<String>>(7)
            .map(|s| serde_json::from_str(&s).map_err(|e| handle_serde_json_error(e)))
            .transpose()?;
        let updated_on: Option<SystemTime> = row.get(3);
        let updated_on: Option<DateTime<Utc>> = updated_on.map(|time| time.into());
        let created_on: Option<SystemTime> = row.get(2);
        let created_on: Option<DateTime<Utc>> = created_on.map(|time| time.into());
        Ok(Project {
            id: row.get(0),
            metadata,
            created_on,
            updated_on,
            description,
            visible: row.get(5),
            adult: row.get(6),
            contents: vec![],
            thumbnail_content,
        })
    }

    fn map_row_to_project_with_thumbnail(row: &Row) -> Result<ProjectWithThumbnail, Error> {
        let metadata: Option<serde_json::Value> = row
            .get::<_, Option<String>>(1)
            .map(|s| serde_json::from_str(&s).map_err(|e| handle_serde_json_error(e)))
            .transpose()?;
        let description: Option<serde_json::Value> = row
            .get::<_, Option<String>>(1)
            .map(|s| serde_json::from_str(&s).map_err(|e| handle_serde_json_error(e)))
            .transpose()?;
        let thumbnail_content: Option<serde_json::Value> = row
            .get::<_, Option<String>>(7)
            .map(|s| serde_json::from_str(&s).map_err(|e| handle_serde_json_error(e)))
            .transpose()?;
        let updated_on: Option<SystemTime> = row.get(3);
        let updated_on: Option<DateTime<Utc>> = updated_on.map(|time| time.into());
        let created_on: SystemTime = row.get(2);
        let created_on: DateTime<Utc> = created_on.into();
        let thumbnail_created_on: Option<SystemTime> = row.get(8);
        let thumbnail_created_on: Option<DateTime<Utc>> =
            thumbnail_created_on.map(|time| time.into());
        Ok(ProjectWithThumbnail {
            id: row.get(0),
            metadata,
            created_on,
            updated_on,
            description,
            visible: row.get(5),
            adult: row.get(6),
            thumbnail_content,
            thumbnail_created_on,
        })
    }

    fn map_row_to_project_content(row: &Row) -> Result<ProjectContent, Error> {
        let content: Option<serde_json::Value> = row
            .get::<_, Option<String>>(2)
            .map(|s| serde_json::from_str(&s).map_err(|e| handle_serde_json_error(e)))
            .transpose()?;
        let created_on: Option<SystemTime> = row.get(3);
        let created_on: Option<DateTime<Utc>> = created_on.map(|time| time.into());
        Ok(ProjectContent::new(
            row.get(0),
            row.get(1),
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
        let mut client = self.client.lock().await;
        let transaction = client.transaction().await.map_err(|e| to_error(e, None))?;
        let result = f(&transaction).await?;
        transaction.commit().await.map_err(|e| to_error(e, None))?;
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
                        .query_one(
                            sql,
                            &[
                                &metadata_json.to_string(),
                                &now_utc.to_string(),
                                &true,
                                &false,
                            ],
                        )
                        .await
                        .map_err(|e| to_error(e, None))?;
                    ProjectRepository::map_create_row_to_project(&row).map(ProjectDto::from)
                })
            })
            .await?;

        Ok(project)
    }

    async fn add_project_content(
        &self,
        project_id: i32,
        content: &ContentDto,
    ) -> Result<ProjectContentDto, Error> {
        let content_json = json!(content);
        let now_utc: DateTime<Utc> = Utc::now();
        let sql = "INSERT INTO project_content(project_id, content, created_on) VALUES($1, $2, $3) RETURNING *";
        let content_entity = self
            .with_transaction(|tx| {
                Box::pin(async move {
                    let row = tx
                        .query_one(
                            sql,
                            &[&project_id, &content_json.to_string(), &now_utc.to_string()],
                        )
                        .await
                        .map_err(|e| to_error(e, Some(project_id.to_string())))?;
                    ProjectRepository::map_row_to_project_content(&row)
                })
            })
            .await?;
        Ok(ProjectContentDto::from(content_entity))
    }

    async fn add_project_thumbnail(
        &self,
        project_id: i32,
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
                        .query_one(
                            sql,
                            &[
                                &thumbnail_json.to_string(),
                                &project_id,
                                &now_utc.to_string(),
                            ],
                        )
                        .await
                        .map_err(|e| to_error(e, Some(project_id.to_string())))?;
                    ProjectRepository::map_row_to_project_content(&row)
                })
            })
            .await?;
        Ok(ProjectContentDto::from(content_entity))
    }

    async fn get_project_by_id(&self, id: i32) -> Result<ProjectDto, Error> {
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

        let client = self.client.lock().await;
        let row = client
            .query_one(sql, &[&id])
            .await
            .map_err(|e| to_error(e, None))?;

        let project = ProjectRepository::map_create_row_to_project(&row)?;

        Ok(ProjectDto::from(project))
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

        let client = self.client.lock().await;
        let rows = client
            .query(sql, &[])
            .await
            .map_err(|e| to_error(e, None))?;

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
        page: i32,
        size: i32,
        is_adult: Option<bool>,
        is_visible: bool,
    ) -> Result<ProjectsDto, Error> {
        let adult_filter = match is_adult {
            Some(adult) => format!("AND p.adult = {adult}"),
            None => "".to_owned(),
        };

        // Use optional parameter syntax to conditionally include the `is_adult` parameter
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
        LIMIT $2 OFFSET $3
        "
        );

        // Construct the total count SQL query
        let total_sql = format!(
            "SELECT COUNT(*)
        FROM project p
        WHERE p.visible = $1
        {adult_filter}"
        );

        let client = self.client.lock().await;

        // Execute the total count SQL query to get the total number of projects

        let total_count: i64 = client
            .query_one(&total_sql, &[&is_visible])
            .await
            .map_err(|e| to_error(e, None))?
            .get(0);

        // Execute the main SQL query to get the projects for the current page

        let rows = client
            .query(&sql, &[&is_visible, &size, &((page - 1) * size)])
            .await
            .map_err(|e| to_error(e, None))?;

        let projects = rows
            .iter()
            .map(|row| ProjectRepository::map_row_to_project_with_thumbnail(row))
            .map(|r| r.map(ProjectWithThumbnailDto::from))
            .collect::<Result<Vec<ProjectWithThumbnailDto>, Error>>()?;

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
        let now_utc: DateTime<Utc> = Utc::now();
        let ProjectDto {
            metadata,
            description,
            visible,
            adult,
            ..
        } = project.clone();

        let sql="UPDATE project SET description = $1, metadata = $2, visible = $3, adult= $4, updated_on = $5 WHERE id = $6";
        let client = self.client.lock().await;
        client
            .execute(
                sql,
                &[
                    &description.map(|v| v.to_string()),
                    &metadata.map(|v| json!(v).to_string()),
                    &visible,
                    &adult,
                    &now_utc.to_string(),
                    &project_id,
                ],
            )
            .await
            .map_err(|e| to_error(e, Some(project_id.to_string())))?;

        Ok(())
    }

    async fn get_projects_contents(
        &self,
        project_id: i32,
    ) -> Result<Vec<ProjectContentDto>, Error> {
        let sql = "SELECT * FROM project_content where project_id = $1";
        let client = self.client.lock().await;
        let row = client
            .query(sql, &[&project_id])
            .await
            .map_err(|e| to_error(e, Some(project_id.to_string())))?;
        let contents = row
            .iter()
            .map(|r| ProjectRepository::map_row_to_project_content(r))
            .map(|r| r.map(ProjectContentDto::from))
            .collect::<Result<Vec<ProjectContentDto>, Error>>()?;
        Ok(contents)
    }

    async fn get_projects_content_by_id(
        &self,
        project_id: i32,
        id: i32,
    ) -> Result<ProjectContentDto, Error> {
        let sql = "SELECT * FROM project_content where project_id = $1 and id= $2";

        let client = self.client.lock().await;
        let row = client
            .query_one(sql, &[&project_id, &id])
            .await
            .map_err(|e| to_error(e, Some(project_id.to_string())))?;
        let content = ProjectRepository::map_row_to_project_content(&row)?;

        Ok(ProjectContentDto::from(content))
    }

    async fn delete_project_content_by_id(&self, project_id: i32, id: i32) -> Result<(), Error> {
        let sql = "DELETE FROM project_content WHERE id = $1 and project_id = $2 ";
        let client = self.client.lock().await;
        client
            .execute(sql, &[&id, &project_id])
            .await
            .map_err(|e| to_error(e, Some(project_id.to_string())))?;
        Ok(())
    }

    async fn delete_project_by_id(&self, project_id: i32) -> Result<(), Error> {
        let sql = "DELETE FROM project WHERE id = $1 ";
        let client = self.client.lock().await;
        client
            .execute(sql, &[&project_id])
            .await
            .map_err(|e| to_error(e, Some(project_id.to_string())))?;
        Ok(())
    }

    //TODO modify for thumbnail
    async fn get_projects_content_thumbnail(
        &self,
        project_id: i32,
    ) -> Result<Vec<ProjectContentDto>, Error> {
        let sql = "SELECT * FROM project_content where project_id = $1 FETCH FIRST ROW ONLY";

        let client = self.client.lock().await;
        let row = client
            .query(sql, &[&project_id])
            .await
            .map_err(|e| to_error(e, Some(project_id.to_string())))?;
        let contents = row
            .iter()
            .map(|r| ProjectRepository::map_row_to_project_content(r))
            .collect::<Result<Vec<ProjectContent>, Error>>()?;

        Ok(contents
            .iter()
            .map(|c| ProjectContentDto::from(c.clone()))
            .collect())
    }

    async fn delete_thumbnail_by_id(&self, project_id: i32, id: i32) -> Result<(), Error> {
        let sql = "DELETE FROM project_content_thumbnail WHERE id = $1 and project_id = $2";

        let client = self.client.lock().await;
        client
            .execute(sql, &[&id, &project_id])
            .await
            .map_err(|e| to_error(e, Some(project_id.to_string())))?;
        Ok(())
    }

    async fn get_thumbnail_by_id(
        &self,
        project_id: i32,
        id: i32,
    ) -> Result<Option<ProjectContentDto>, Error> {
        let sql = "SELECT * FROM project_content_thumbnail AS pt
        WHERE pt.content ->> 'id' = CAST($1 AS TEXT) and pt.project_id = $2
        ";

        let client = self.client.lock().await;
        let row = client
            .query_one(sql, &[&id, &project_id])
            .await
            .map_err(|e| to_error(e, Some(project_id.to_string())))?;
        let content = ProjectRepository::map_row_to_project_content(&row)?;

        Ok(Some(ProjectContentDto::from(content)))
    }
}
