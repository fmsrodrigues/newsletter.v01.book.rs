use actix_web::{web, HttpResponse};
use actix_web_flash_messages::FlashMessage;
use anyhow::Context;
use sqlx::PgPool;

use crate::authentication::UserId;
use crate::utils::{e500, see_other};
use crate::{domain::SubscriberEmail, email_client::EmailClient};

#[derive(serde::Deserialize)]
pub struct FormData {
    title: String,
    text_content: String,
    html_content: String,
}

#[tracing::instrument(
    name = "Publish a newsletter issue",
    skip(form, pool, email_client, user_id),
    fields(
        user_id=%*user_id
    )
)]
pub async fn publish_newsletter(
    form: web::Form<FormData>,
    pool: web::Data<PgPool>,
    email_client: web::Data<EmailClient>,
    user_id: web::ReqData<UserId>,
) -> Result<HttpResponse, actix_web::Error> {
    let subscribers = get_confirmed_subscriber(&pool).await.map_err(e500)?;

    // This could be refactored to send all emails on one request
    // due to how the MailTrap API works but,
    // I want to follow the book as close as possible,
    // so I will do how it was shown with the Postmark API
    for subscriber in subscribers {
        match subscriber {
            Ok(subscriber) => {
                email_client
                    .send_email(
                        &subscriber.email,
                        &form.title,
                        &form.html_content,
                        &form.text_content,
                    )
                    .await
                    .with_context(|| {
                        format!("Failed to send newsletter issue to {}", subscriber.email)
                    })
                    .map_err(e500)?;
            }
            Err(error) => {
                tracing::warn!(
                    error.cause_chain = ?error,
                    error.message = %error,
                    "Skipping invalid subscriber. Their stored contact details are invalid"
                )
            }
        }
    }

    FlashMessage::info("The newsletter issue has been published!").send();
    Ok(see_other("/admin/newsletters"))
}

struct ConfirmedSubscriber {
    email: SubscriberEmail,
}

#[tracing::instrument(name = "Get confirmed subscribers", skip(pool))]
async fn get_confirmed_subscriber(
    pool: &PgPool,
) -> Result<Vec<Result<ConfirmedSubscriber, anyhow::Error>>, anyhow::Error> {
    let rows = sqlx::query!(
        r#"
        SELECT email
        FROM subscriptions
        WHERE status = 'confirmed'
        "#,
    )
    .fetch_all(pool)
    .await?;

    let confirmed_subscribers = rows
        .into_iter()
        .map(|r| match SubscriberEmail::parse(r.email) {
            Ok(email) => Ok(ConfirmedSubscriber { email }),
            Err(error) => Err(anyhow::anyhow!(error)),
        })
        .collect();

    Ok(confirmed_subscribers)
}