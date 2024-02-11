use actix_web::{web, HttpResponse};
use sqlx::PgPool;
use sqlx::{types::chrono::Utc, types::uuid::Uuid};

#[derive(serde::Deserialize)]
pub struct SubscribeFormData {
    name: String,
    email: String,
}

#[tracing::instrument(
    name = "Adding a new subscriber"
    skip(form, db_pool),
    fields(
        subscriber_name = %form.name,
        subscriber_email = %form.email,
    )
)]
pub async fn post_subscribe(
    form: web::Form<SubscribeFormData>,
    db_pool: web::Data<PgPool>,
) -> HttpResponse {
    match insert_subscriber(&db_pool, &form).await {
        Ok(_) => HttpResponse::Ok().finish(),
        Err(e) => {
            tracing::error!("Error when running query: {:?}", e);
            HttpResponse::InternalServerError().finish()
        }
    }
}

#[tracing::instrument(name = "Saving new subscriber details in DB", skip(db_pool, form))]
pub async fn insert_subscriber(
    db_pool: &PgPool,
    form: &SubscribeFormData,
) -> Result<(), sqlx::Error> {
    sqlx::query!(
        r#"
INSERT INTO subscriptions (id, email, name, subscribed_at)
VALUES ($1, $2, $3, $4)
"#,
        Uuid::new_v4(),
        form.email,
        form.name,
        Utc::now()
    )
    .execute(db_pool)
    .await
    .map_err(|e| {
        tracing::error!("Failed to exec query: {:?}", e);
        return e;
    })?;

    Ok(())
}
