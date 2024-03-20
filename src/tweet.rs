use actix_web::web::{Data, Json, Path};
use actix_web::{web, HttpResponse};
use chrono::{DateTime, NaiveDateTime, TimeZone, Utc};
use diesel::result::Error;
use diesel::{ExpressionMethods, Insertable, Queryable, RunQueryDsl};
use serde::{Deserialize, Serialize};
use uuid::Uuid;

use crate::constants::{APPLICATION_JSON, CONNECTION_POOL_ERROR};
use crate::like::{list_likes, Like};
use crate::response::Response;
use crate::{DBPool, DBPooledConnection};

use super::schema::tweets;
use diesel::query_dsl::methods::{FilterDsl, LimitDsl, OrderDsl};
use std::str::FromStr;

pub type Tweets = Response<Tweet>;

#[derive(Debug, Deserialize, Serialize)]
pub struct Tweet {
    pub id: String,
    pub created_at: DateTime<Utc>,
    pub message: String,
    pub likes: Vec<Like>,
}

impl Tweet {
    pub fn new(message: String) -> Self {
        Self {
            id: Uuid::new_v4().to_string(),
            created_at: Utc::now(),
            message,
            likes: vec![],
        }
    }
}

#[derive(Debug, Deserialize, Serialize)]
pub struct TweetRequest {
    pub message: Option<String>,
}

impl TweetRequest {
    pub fn to_tweet(&self) -> Option<Tweet> {
        match &self.message {
            Some(message) => Some(Tweet::new(message.to_string())),
            None => None,
        }
    }
}

#[get("/tweets")]
pub async fn list() -> HttpResponse {
    // TODO find tweets

    let tweets = Tweets { results: vec![] };

    HttpResponse::Ok()
        .content_type(APPLICATION_JSON)
        .json(tweets)
}

#[post("/tweets")]
pub async fn create(tweet_req: Json<TweetRequest>) -> HttpResponse {
    HttpResponse::Created()
        .content_type(APPLICATION_JSON)
        .json(tweet_req.to_tweet())
}

#[get("/tweets/{id}")]
pub async fn get(path: Path<(String,)>) -> HttpResponse {
    let found_tweet: Option<Tweet> = None;

    match found_tweet {
        Some(tweet) => HttpResponse::Ok()
            .content_type(APPLICATION_JSON)
            .json(tweet),
        None => HttpResponse::NoContent()
            .content_type(APPLICATION_JSON)
            .await
            .unwrap(),
    }
}

#[delete("/tweets/{id}")]
pub async fn delete(path: Path<(String,)>) -> HttpResponse {
    HttpResponse::NoContent()
        .content_type(APPLICATION_JSON)
        .await
        .unwrap()
}

fn list_tweets(total_tweets: i64, conn: &DBPooledConnection) -> Result<Tweets, Error> {
    use crate::schema::tweets::dsl::*;

    let _tweets = match tweets
        .order(created_at.desc())
        .limit(total_tweets)
        .load::<TweetDB>(conn)
    {
        Ok(tws) => tws,
        Err(_) => vec![]
    };

    Ok(Tweets {
        results: _tweets
            .into_iter()
            .map(|t| t.to_tweet())
            .collect::<Vec<Tweet>>(),
    })
}

fn find_tweet(_id: Uuid, conn: &DBPooledConnection) -> Result<Tweet, Error> {
    use crate::schema::tweets::dsl::*;

    let res = tweets.filter(id.eq(_id)).load::<TweetDB>(conn);
    match res {
        Ok(tweets_db) => match tweets_db.first() {
            Some(tweet_db) => Ok(tweet_db.to_tweet()),
            _ => Err(Error::NotFound),
        },
        Err(err) => Err(err),
    }
}

fn create_tweet(tweet: Tweet, conn: &DBPooledConnection) -> Result<Tweet, Error> {
    use crate::schema::tweets::dsl::*;

    let tweet_db = tweet::to_tweet_db();
    let _ = diesel::insert_into(tweets).values(&tweet_db).execute(conn);

    Ok(tweet_db.to_tweet())
}

fn delete_tweet(_id: Uuid, conn: &DBPooledConnection) -> Result<(), Error> {
    use crate::schema::tweets::dsl::*;

    let res = diesel::delete(tweets.filter(id.eq(_id))).execute(conn);
    match res {
        Ok(_) => Ok(()),
        Err(err) => Err(err),
    }
}