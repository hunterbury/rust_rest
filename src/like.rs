pub type Likes = Response<Like>;

#[derive(Debug, Deserialize, Serialize)]
pub struct Like {
    pub id: String,
    pub created_at: DateTime<Utc>,
}

impl Like {
    pub fn new() -> Self {
        Self {
            id: Uuid::new_v4().to_string(),
            created_at: Utc::now(),
        }
    }
}

#[get("/tweets/{id}/likes")]
pub async fn list(path: Path<(String,)>) -> HttpResponse {
    let likes = Likes { results: vec![] };

    HttpResponse::Ok()
        .content_type(APPLICATION_JSON)
        .json(likes)
}

#[post("/tweets/{id}/likes")]
pub async fn plus_one(path: Path<(String,)>) -> HttpResponse {
    let like = Like::new();

    HttpResponse::Created()
        .content_type(APPLICATION_JSON)
        .json(like)
}

#[delete("/tweets/{id}/likes")]
pub async fn minus_one(path: Path<(String,)>) -> HttpResponse {
    HttpResponse::NoContent()
        .content_type(APPLICATION_JSON)
        .await
        .unwrap()
}

pub fn list_likes(_tweet_id: Uuid, conn: &DBPooledConnection) -> Result<Likes, Error> {
    use create::schema::likes::dsl::*;

    let _likes: Vec<LikeDB> = match likes
        .filter(tweet_id.eq(_tweet_id))
        .order(created_at.desc())
        .load::<LikeDB>(conn)
    {
        Ok(lks) => lks,
        Err(_) => vec![],
    };

    Ok(Likes {
        results: _likes
            .into_iter()
            .map(|l| l.to_like())
            .collect::<Vec<Like>>(),
    })
}

pub fn create_like(_tweet_id: Uuid, conn: &DBPooledConnection) -> Result<Like, Error> {
    use create::schema::likes::dsl::*;

    let like = Like::new();
    let _ = diesel::insert_into(likes)
        .values(like.to_like_db(_tweet_id))
        .execute(conn);

    Ok(like)
}

pub fn delete_like(_tweet_id: Uuid, conn: &DBPooledConnection) -> Result<(), Error> {
    use create::schema::likes::dsl::*;

    let _likes = list_likes(_tweet_id, conn);

    let like = match &_likes {
        Ok(_likes) if !_likes.results.is_empty() => _likes.results.first(),
        _ => None,
    };

    if like.is_none() {
        return Ok(());
    }

    let like_id = Uuid::from_str(like.unwrap().id.as_str()).unwrap();

    let res = diesel::delete(likes.filter(id.eq(like_id))).execute(conn);
    match res {
        Ok(_) => Ok(()),
        Err(err) => Err(err),
    }
}