use sqlx::{sqlite::SqlitePool, Sqlite, Transaction};
use std::iter::Iterator;

use crate::{error::DbError, model};

pub async fn get_articles(
    db: &SqlitePool,
    limit: i32,
    offset: i32,
    active: bool,
    ancestor: bool,
) -> std::result::Result<Vec<model::ArticleObject>, DbError> {
    let mut tx = db.begin().await?;
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
    .fetch_all(&mut tx).await?;

    let mut result = Vec::new();
    for parent in article_entities_result {
        let child = match ancestor {
            true => get_article_tx(&mut tx, parent.precursor_id).await?,
            false => None,
        };
        let o = model::ArticleObject {
            entity: parent,
            precursor: child,
        };
        result.push(o);
    }

    tx.commit().await?;

    return Ok(result);
}

pub async fn num_active(db: &SqlitePool) -> std::result::Result<i32, DbError> {
    let mut tx = db.begin().await?;
    let count_result = sqlx::query_scalar::<_, i32>(
        "SELECT count(*)
		FROM article
		WHERE active IS TRUE",
    )
    .fetch_one(&mut tx)
    .await?;
    tx.commit().await?;

    return Ok(count_result);
}

// pub async fn get_article(
//     db: &SqlitePool,
//     article_id: Option<i32>,
// ) -> std::result::Result<Option<Box<model::ArticleObject>>, DbError> {
//     let mut tx = db.begin().await?;
//     let article = get_article_tx(&mut tx, article_id).await?;
//     tx.commit().await?;
//     return Ok(article);
// }

pub async fn get_article_tx(
    tx: &mut Transaction<'static, Sqlite>,
    article_id: Option<i32>,
) -> std::result::Result<Option<Box<model::ArticleObject>>, DbError> {
    match article_id {
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
            .fetch_all(tx)
            .await?;

            Ok(build_article_chain(article_chain))
        }
        None => Ok(None),
    }
}

pub async fn get_article_or_error(
    db: &SqlitePool,
    article_id: i32,
) -> std::result::Result<model::ArticleObject, DbError> {
    let mut tx = db.begin().await?;
    let article = get_article_or_error_tx(&mut tx, article_id).await?;
    tx.commit().await?;
    return Ok(article);
}

pub async fn get_article_or_error_tx(
    tx: &mut Transaction<'static, Sqlite>,
    article_id: i32,
) -> std::result::Result<model::ArticleObject, DbError> {
    return match get_article_tx(tx, Some(article_id)).await {
        Ok(Some(article)) => Ok(*article),
        Ok(None) => Err(DbError::EntityNotFound(("Article").to_string())),
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
) -> std::result::Result<model::ArticleObject, DbError> {
    let mut tx = db.begin().await?;
    let article_entity = sqlx::query_as::<_, model::ArticleEntity>(
        "INSERT INTO article (name, barcode, amount, active, created, usage_count)
		VALUES(?, ?, ?, TRUE, datetime('now', 'localtime'), 0);

        SELECT id, precursor_id, name, barcode, amount, active, created, usage_count
        FROM article WHERE id = last_insert_rowid();",
    )
    .bind(name)
    .bind(barcode)
    .bind(amount)
    .fetch_one(&mut tx)
    .await?;
    tx.commit().await?;

    return Ok(model::ArticleObject {
        entity: article_entity,
        precursor: None,
    });
}

pub async fn update_article(
    db: &SqlitePool,
    precursor_id: i32,
    name: &str,
    barcode: Option<&str>,
    amount: i32,
) -> std::result::Result<model::ArticleObject, DbError> {
    let mut tx = db.begin().await?;
    let child = get_article_tx(&mut tx, Some(precursor_id))
        .await?
        .map(|mut a| {
            a.entity.active = false;
            a
        });

    let article_entity = sqlx::query_as::<_, model::ArticleEntity>(
        "-- try to insert, trigger prevents updating inactive articles
		INSERT INTO article (precursor_id, name, barcode, amount, active, created, usage_count)
		SELECT id, ?, ?, ?, TRUE, datetime('now', 'localtime'), usage_count
		FROM article WHERE id = ?;

		-- deactivate old article if it had been active to make sure the transaction does not fail
		UPDATE article SET active = FALSE
		WHERE id = ?;

		SELECT id, precursor_id, name, barcode, amount, active, created, usage_count
        FROM article WHERE id = last_insert_rowid();",
    )
    .bind(name)
    .bind(barcode)
    .bind(amount)
    .bind(precursor_id)
    .bind(precursor_id)
    .fetch_one(&mut tx)
    .await?;
    tx.commit().await?;

    return Ok(model::ArticleObject {
        entity: article_entity,
        precursor: child,
    });
}

pub async fn delete_article(
    db: &SqlitePool,
    article_id: i32,
) -> std::result::Result<model::ArticleObject, DbError> {
    let mut tx = db.begin().await?;
    sqlx::query("UPDATE article SET active = FALSE WHERE id = ?;")
        .bind(article_id)
        .execute(&mut tx)
        .await?;
    let article_result = get_article_tx(&mut tx, Some(article_id)).await?;
    tx.commit().await?;

    return match article_result {
        Some(a) => Ok(*a),
        None => Err(DbError::EntityNotFound("Article".to_string())),
    };
}
