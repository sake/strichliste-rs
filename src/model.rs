use serde::{Deserialize, Serialize};
use warp::Reply;

//
// Helper to provide typed JSON replies
//

#[derive(Clone)]
pub struct JsonReply<T: Serialize>(T);

pub fn json_reply<T: Serialize>(val: T) -> JsonReply<T> {
    JsonReply(val)
}

impl<T: Serialize + Send> Reply for JsonReply<T> {
    fn into_response(self) -> warp::reply::Response {
        warp::reply::json(&self.0).into_response()
    }
}

//
// DB entities
//

#[derive(Debug, Deserialize, Serialize, Clone, sqlx::FromRow)]
pub struct UserEntity {
    pub id: i32,
    pub name: String,
    pub email: Option<String>,
    pub balance: i32,
    #[serde(rename(serialize = "isActive", deserialize = "isActive"))]
    pub active: bool,
    #[serde(rename(serialize = "isDisabled", deserialize = "isDisabled"))]
    pub disabled: bool,
    pub created: String,
    pub updated: Option<String>,
}

#[derive(Debug, Deserialize, Serialize, Clone, sqlx::FromRow)]
pub struct ArticleEntity {
    pub id: i32,
    #[serde(skip)]
    pub precursor_id: Option<i32>,
    pub name: String,
    pub barcode: Option<String>,
    pub amount: i32,
    #[serde(rename(serialize = "isActive", deserialize = "isActive"))]
    pub active: bool,
    pub created: String,
    #[serde(rename(serialize = "usageCount", deserialize = "usageCount"))]
    pub usage_count: i32,
}

#[derive(Debug, Deserialize, Serialize, Clone, sqlx::FromRow)]
pub struct TransactionEntity {
    pub id: i32,
    #[serde(skip)]
    pub user_id: i32,
    #[serde(skip)]
    pub article_id: Option<i32>,
    #[serde(skip)]
    pub recipient_transaction_id: Option<i32>,
    #[serde(skip)]
    pub sender_transaction_id: Option<i32>,
    pub quantity: Option<i32>,
    pub comment: Option<String>,
    pub amount: i32,
    pub deleted: bool,
    pub created: String,
}

#[derive(Debug, Deserialize, Serialize, Clone, sqlx::FromRow)]
pub struct TransactionStatsEntity {
    pub count: i32,
    pub amount: i32,
}


//
// complete data objects
//

#[derive(Debug, Deserialize, Serialize, Clone)]
pub struct ArticleObject {
    #[serde(flatten)]
    pub entity: ArticleEntity,
    pub precursor: Option<Box<ArticleObject>>,
}

#[derive(Debug, Deserialize, Serialize, Clone)]
pub struct TransactionObject {
    #[serde(flatten)]
    pub entity: TransactionEntity,
    pub user: UserEntity,
    pub article: Option<ArticleObject>,
    pub recipient: Option<UserEntity>,
    pub sender: Option<UserEntity>,
}

#[derive(Debug, Deserialize, Serialize, Clone)]
pub struct DailyTransaction {
    pub date: String,
    pub transactions: i32,
    #[serde(rename(serialize = "distinctUsers", deserialize = "distinctUsers"))]
    pub distinct_users: i32,
    pub balance: i32,
    pub charged: TransactionSum,
    pub spent: TransactionSum,
}

#[derive(Debug, Deserialize, Serialize, Clone)]
pub struct TransactionSum {
    pub amount: i32,
    pub transactions: i32,
}

//
// request objects
//

#[derive(Debug, Deserialize, Serialize, Clone)]
pub struct UserAddReq {
    pub name: String,
    pub email: Option<String>,
}

#[derive(Debug, Deserialize, Serialize, Clone)]
pub struct UserUpdateReq {
    pub name: String,
    pub email: Option<String>,
    #[serde(rename(serialize = "isDisabled", deserialize = "isDisabled"))]
    pub is_disabled: bool,
}

#[derive(Debug, Deserialize, Serialize, Clone)]
pub struct ArticleAddReq {
    pub name: String,
    pub barcode: Option<String>,
    pub amount: i32,
}

#[derive(Debug, Deserialize, Serialize, Clone)]
pub struct TransactionAddReq {
    pub amount: Option<f32>,
    pub quantity: Option<i32>,
    pub comment: Option<String>,
    #[serde(rename(serialize = "recipientId", deserialize = "recipientId"))]
    pub recipient_id: Option<i32>,
    #[serde(rename(serialize = "articleId", deserialize = "articleId"))]
    pub article_id: Option<i32>,
}

//
// response objects
//

#[derive(Debug, Deserialize, Serialize, Clone)]
pub struct UsersResp {
    pub count: usize,
    pub users: Vec<UserEntity>,
}

#[derive(Debug, Deserialize, Serialize, Clone)]
pub struct UserResp {
    pub user: UserEntity,
}

#[derive(Debug, Deserialize, Serialize, Clone)]
pub struct ArticleResp {
    pub article: ArticleObject,
}

#[derive(Debug, Deserialize, Serialize, Clone)]
pub struct ArticlesResp {
    pub count: usize,
    pub articles: Vec<ArticleObject>,
}

#[derive(Debug, Deserialize, Serialize, Clone)]
pub struct TransactionResp {
    pub transaction: TransactionObject,
}

#[derive(Debug, Deserialize, Serialize, Clone)]
pub struct TransactionsResp {
    pub count: usize,
    pub transactions: Vec<TransactionObject>,
}

#[derive(Debug, Deserialize, Serialize, Clone)]
pub struct SystemMetrics {
    pub balance: i32,
    #[serde(rename(serialize = "transactionCount", deserialize = "transactionCount"))]
    pub transaction_count: i32,
    #[serde(rename(serialize = "userCount", deserialize = "userCount"))]
    pub user_count: i32,
    pub articles: Vec<ArticleObject>,
    pub days: Vec<DailyTransaction>,
}

#[derive(Debug, Deserialize, Serialize, Clone)]
pub struct UserArticles {
    pub article: ArticleObject,
    pub count: i32,
    pub amount: i32,
}

#[derive(Debug, Deserialize, Serialize, Clone)]
pub struct UserTransactions {
    pub count: i32,
    pub outgoing: TransactionStatsEntity,
    pub incoming: TransactionStatsEntity,
}

#[derive(Debug, Deserialize, Serialize, Clone)]
pub struct UserMetrics {
    pub balance: i32,
    pub articles: Vec<UserArticles>,
    pub transactions: UserTransactions,
}
