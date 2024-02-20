use actix_web::{web, HttpResponse};
use sqlx::PgPool;
use sqlx::{types::chrono::Utc, types::uuid::Uuid};

use crate::domain::{NewSubscriber, SubscriberEmail, SubscriberName};

#[derive(serde::Deserialize, Debug)]
pub struct SubscribeFormData {
    name: String,
    email: String,
}

impl TryFrom<SubscribeFormData> for NewSubscriber {
    type Error = String;

    fn try_from(form: SubscribeFormData) -> Result<Self, Self::Error> {
        let name = SubscriberName::parse(form.name)?;
        let email = SubscriberEmail::parse(form.email)?;
        Ok(NewSubscriber { name, email })
    }
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
    let new_subscriber = match form.0.try_into() {
        Ok(name) => name,
        Err(_) => return HttpResponse::BadRequest().finish(),
    };

    match insert_subscriber(&db_pool, &new_subscriber).await {
        Ok(_) => HttpResponse::Ok().finish(),
        Err(e) => {
            tracing::error!("Error when running query: {:?}", e);
            HttpResponse::InternalServerError().finish()
        }
    }
}

#[tracing::instrument(
    name = "Saving new subscriber details in DB",
    skip(db_pool, new_subscriber)
)]
pub async fn insert_subscriber(
    db_pool: &PgPool,
    new_subscriber: &NewSubscriber,
) -> Result<(), sqlx::Error> {
    sqlx::query!(
        r#"
INSERT INTO subscriptions (id, email, name, subscribed_at)
VALUES ($1, $2, $3, $4)
"#,
        Uuid::new_v4(),
        new_subscriber.email.as_ref(),
        new_subscriber.name.as_ref(),
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
