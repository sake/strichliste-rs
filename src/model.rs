use serde::{Deserialize, Serialize};

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
    pub active: bool,
    pub created: String,
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
    pub user: Option<UserEntity>,
    pub article: Option<ArticleObject>,
    pub recipient_transaction: Option<Box<TransactionObject>>,
    pub sender_transaction: Option<Box<TransactionObject>>,
}

//
// request objects
//

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
pub struct TransactionAddReq {
    pub amount: i32,
    pub quantity: Option<i32>,
    pub comment: Option<String>,
    #[serde(rename(serialize = "recipientId", deserialize = "recipientId"))]
    pub recipient_id: Option<i32>,
    #[serde(rename(serialize = "articleId", deserialize = "articleId"))]
    pub article_id: Option<i32>,
}

#[derive(Debug, Deserialize, Serialize, Clone)]
pub struct ArticleAddReq {
    pub name: String,
    pub barcode: Option<String>,
    pub amount: i32,
    #[serde(rename(serialize = "isActive", deserialize = "isActive"))]
    pub active: bool,
    pub precursor: Option<ArticleObject>,
}

#[derive(Debug, Deserialize, Serialize, Clone)]
pub struct ArticlesResp {
    pub count: usize,
    pub articles: Vec<ArticleObject>,
}
