use futures::{
    stream::{self, StreamExt},
    FutureExt,
};

use crate::{
    app_error::Error,
    domain::{
        content::Content, metadata::Metadata, project_content_entity::ProjectContentEntity,
        project_entity::ProjectEntity,
    },
    mapper::project_mapper::to_view,
    repository::{
        project_repository::{
            add_project_content, create, delete_project_by_id, delete_project_content_by_id,
            get_project_by_id, get_projects, get_projects_content_by_id,
            get_projects_content_thumbnail, get_projects_contents, update_project_entity,
        },
        storage_repository::{get_object_metadata, get_object_url, remove_object, upload_file},
    },
    view::{content_view::ContentView, project_view::ProjectView},
};

pub async fn create_project(metadata: &Metadata) -> Result<ProjectView, Error> {
    let project = create(metadata).await?;
    let contents: Vec<ContentView> = vec![];
    let project_view = to_view(&contents, &project);
    Ok(project_view)
}

pub async fn add_project(id: i32, file_name: String, file: &[u8]) -> Result<ContentView, Error> {
    let _ = get_project_by_id(id).await?;
    let key = format!("{}/{}", "portfolio", file_name);
    let bucket = "portfolio";
    let content = Content::new(None, bucket.to_owned(), key.clone(), None);
    upload_file(bucket, key.as_str(), file).await?;
    let content_entity = add_project_content(id, &content).await?;
    let content = content_entity
        .content()
        .map(|photo| &photo.0)
        .map(to_content);
    let (url, mime_type) = match content {
        Some(photo) => {
            let url = get_object_url(photo.bucket_name(), photo.file_name()).await?;
            let head = get_object_metadata(photo.bucket_name(), photo.file_name()).await?;
            (Some(url), head.content_type)
        }
        None => (None, None),
    };

    let content_view = ContentView::new(content_entity.id().copied(), mime_type, url);

    Ok(content_view)
}

pub async fn update_project(id: i32, project: &ProjectEntity) -> Result<(), Error> {
    let _ = get_project_by_id(id).await?;

    update_project_entity(id, project).await?;

    Ok(())
}

pub async fn find_project(id: i32) -> Result<ProjectView, Error> {
    let project_entity = get_project_by_id(id).await?;

    let project_contents = get_projects_contents(id).await?;

    let mut contents: Vec<ContentView> = vec![];

    for content_entity in project_contents {
        let content = content_entity
            .content()
            .map(|photo| &photo.0)
            .map(to_content);
        let (url, mime_type) = match content {
            Some(photo) => {
                let url = get_object_url(photo.bucket_name(), photo.file_name()).await?;
                let head = get_object_metadata(photo.bucket_name(), photo.file_name()).await?;
                (Some(url), head.content_type)
            }
            None => (None, None),
        };

        let content_view = ContentView::new(content_entity.id().copied(), mime_type, url);
        contents.push(content_view);
    }
    let project_view = to_view(&contents, &project_entity);
    Ok(project_view)
}

pub async fn delete_project(id: i32) -> Result<(), Error> {
    let _ = get_project_by_id(id).await?;
    let project_contents = get_projects_contents(id).await?;
    delete_project_by_id(id).await?;
    for content_entity in project_contents {
        let content = content_entity
            .content()
            .map(|photo| &photo.0)
            .map(to_content);

        if let Some(content) = content {
            remove_object(content.bucket_name(), content.file_name()).await?
        }
    }

    Ok(())
}

pub async fn delete_project_content(project_id: i32, content_id: i32) -> Result<(), Error> {
    let _ = get_project_by_id(project_id).await?;
    let content_entity = get_projects_content_by_id(project_id, content_id).await?;
    let content = content_entity
        .content()
        .map(|photo| &photo.0)
        .map(to_content);

    delete_project_content_by_id(project_id, content_id).await?;

    if let Some(content) = content {
        remove_object(content.bucket_name(), content.file_name()).await?
    }

    Ok(())
}

// TODO faire la récupération de tous les projet avec 1 image pour le thumbnail

pub async fn get_portfolio_projects() -> Result<Vec<ProjectView>, Error> {
    let projects = get_projects().await?;

    let result = stream::iter(projects)
        .fold(Vec::new(), |mut vec, data| async move {
            let contents = match data.id().copied() {
                Some(id) => {
                    let thumb = get_projects_content_thumbnail(id).await.unwrap();
                    let thumb_views = to_contents(&thumb).await.unwrap();
                    thumb_views
                }
                None => vec![],
            };
            let project_view = to_view(&contents, &data);
            vec.push(project_view);
            vec
        })
        .map(move |vec| vec)
        .await;

    Ok(result)
}

fn to_content(value: &serde_json::Value) -> Content {
    serde_json::from_value(value.clone()).unwrap()
}

async fn to_contents(
    project_contents: &Vec<ProjectContentEntity>,
) -> Result<Vec<ContentView>, Error> {
    let mut contents: Vec<ContentView> = vec![];
    for content_entity in project_contents {
        let content = content_entity
            .content()
            .map(|photo| &photo.0)
            .map(to_content);
        let (url, mime_type) = match content {
            Some(photo) => {
                let url = get_object_url(photo.bucket_name(), photo.file_name()).await?;
                let head = get_object_metadata(photo.bucket_name(), photo.file_name()).await?;
                (Some(url), head.content_type)
            }
            None => (None, None),
        };
        let content_view = ContentView::new(content_entity.id().copied(), mime_type, url);
        contents.push(content_view);
    }
    Ok(contents)
}
