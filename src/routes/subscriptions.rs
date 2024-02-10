use actix_web::{web, HttpResponse};
use sqlx::PgPool;
use sqlx::{types::chrono::Utc, types::uuid::Uuid};

#[derive(serde::Deserialize)]
pub struct SubscribeFormData {
    name: String,
    email: String,
}

pub async fn post_subscribe(
    form: web::Form<SubscribeFormData>,
    db_pool: web::Data<PgPool>,
) -> HttpResponse {
    let pg_query_result = sqlx::query!(
        r#"
INSERT INTO subscriptions (id, email, name, subscribed_at)
VALUES ($1, $2, $3, $4)
"#,
        Uuid::new_v4(),
        form.email,
        form.name,
        Utc::now()
    )
    // We use `get_ref` to get an immutable reference to the `PgConnection`
    // wrapped by `web::Data`.
    .execute(db_pool.get_ref())
    .await;
    match pg_query_result {
        Ok(_) => HttpResponse::Ok().finish(),
        Err(e) => {
            eprintln!("Error when running query: {e}");
            HttpResponse::InternalServerError().finish()
        }
    }
}
