use sqlx::{sqlite::SqlitePool, Result};
use std::iter::Iterator;

use crate::model;

pub async fn get_articles(
    db: &SqlitePool,
    limit: i32,
    offset: i32,
    active: bool,
) -> Result<Vec<model::ArticleObject>> {
    let article_entities_result = sqlx::query_as::<_, model::ArticleEntity>(
		"SELECT a1.id, a1.precursor_id, a1.name, a1.barcode, a1.amount, a1.active, a1.created, a1.usage_count
		FROM article AS a1
		LEFT JOIN article AS a2 ON a1.id = a2.precursor_id
		WHERE a1.active IS ? AND a2.id IS NULL
		ORDER BY a1.name LIMIT ? OFFSET ?"
	)
	.bind(active)
	.bind(limit)
	.bind(offset)
	.fetch_all(db).await?;

    let mut result = Vec::new();
    for parent in article_entities_result {
        let child = get_article(&db, parent.precursor_id).await?;
        let o = model::ArticleObject {
            entity: parent,
            precursor: child,
        };
        result.push(o);
    }

    return Ok(result);
}

pub async fn num_active(db: &SqlitePool) -> Result<i32> {
    let count_result = sqlx::query_scalar::<_, i32>(
        "SELECT count(*)
		FROM article
		WHERE active IS TRUE",
    )
    .fetch_one(db);

    return count_result.await;
}

pub async fn get_article(
    db: &SqlitePool,
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
            .fetch_all(db)
            .await;

            article_chain.map(build_article_chain)
        }
        None => Ok(None),
    };
}

pub async fn get_article_or_error(
    db: &SqlitePool,
    article_id: i32,
) -> Result<model::ArticleObject> {
    return match get_article(db, Some(article_id)).await {
        Ok(Some(article)) => Ok(*article),
        Ok(None) => Err(sqlx::Error::RowNotFound),
        Err(e) => Err(e),
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

pub async fn add_article(
    db: &SqlitePool,
    name: &str,
    barcode: Option<&str>,
    amount: i32,
) -> Result<model::ArticleObject> {
    let article_entity_result = sqlx::query_as::<_, model::ArticleEntity>(
        "BEGIN TRANSACTION;
        INSERT INTO article (name, barcode, amount, active, created, usage_count)
		VALUES(?, ?, ?, TRUE, datetime('now'), 0);

        SELECT id, precursor_id, name, barcode, amount, active, created, usage_count
        FROM article WHERE id = last_insert_rowid();
        END TRANSACTION;",
    )
    .bind(name)
    .bind(barcode)
    .bind(amount)
    .fetch_one(db)
    .await;

    return article_entity_result.map(|a| model::ArticleObject {
        entity: a,
        precursor: None,
    });
}

pub async fn update_article(
    db: &SqlitePool,
    precursor_id: i32,
    name: &str,
    barcode: Option<&str>,
    amount: i32,
) -> Result<model::ArticleObject> {
    let child = get_article(db, Some(precursor_id)).await?;
    let article_entity = sqlx::query_as::<_, model::ArticleEntity>(
        "BEGIN TRANSACTION;
        -- try to insert, trigger prevents updating inactive articles
		INSERT INTO article (precursor_id, name, barcode, amount, active, created, usage_count)
		SELECT id, ?, ?, ?, TRUE, datetime('now'), usage_count
		FROM article WHERE id = ?;

		-- deactivate old article if it had been active to make sure the transaction does not fail
		UPDATE article SET active = FALSE
		WHERE id = ?;

		SELECT id, precursor_id, name, barcode, amount, active, created, usage_count
        FROM article WHERE id = last_insert_rowid();
        END TRANSACTION;",
    )
    .bind(name)
    .bind(barcode)
    .bind(amount)
    .bind(precursor_id)
    .bind(precursor_id)
    .fetch_one(db)
    .await?;

    return Ok(model::ArticleObject {
        entity: article_entity,
        precursor: child,
    });
}

pub async fn delete_article(db: &SqlitePool, article_id: i32) -> Result<model::ArticleObject> {
    sqlx::query("BEGIN TRANSACTION;
            UPDATE article SET active = FALSE WHERE id = ?;
            END TRANSACTION;")
        .bind(article_id)
        .execute(db)
        .await?;
    let article_result = get_article(db, Some(article_id)).await?;

    return match article_result {
        Some(a) => Ok(*a),
        None => Err(sqlx::error::Error::RowNotFound),
    };
}
