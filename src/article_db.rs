
use sqlx::{sqlite::SqlitePool, Result};

use crate::model;


pub async fn get_articles(
    db: SqlitePool
) -> Result<Vec<model::ArticleObject>> {
    // TODO: implement
    return Ok(vec!());
}

pub async fn get_article(
    db: SqlitePool,
    article_id: Option<i32>,
) -> Result<Option<Box<model::ArticleObject>>> {
    return match article_id {
        Some(aid) => {
            let article_chain = sqlx::query_as::<_, model::ArticleEntity>(
                "WITH article_chain AS (
					SELECT id, precursor_id, name, barcode, amount, active, created, usage_count
					FROM article WHERE id = ?
					UNION
					SELECT p.id, p.precursor_id, p.name, p.barcode, p.amount, p.active, p.created, p.usage_count
					FROM article p
						INNER JOIN article_chain o 
							ON o.precursor_id = p.id
				)
				SELECT * FROM article_chain;",
            )
            .bind(aid)
            .fetch_all(&db)
            .await;

            article_chain.map(build_article_chain)
        }
        None => Ok(None),
    };
}

fn build_article_chain(chain: Vec<model::ArticleEntity>) -> Option<Box<model::ArticleObject>> {
    // reverse list, so we can build the list starting at the end
    return chain.into_iter().rev().fold(None, |acc, next| {
        Some(Box::new(model::ArticleObject {
            entity: next,
            precursor: acc,
        }))
    });
}
