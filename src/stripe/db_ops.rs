use super::models::StripeUpdate;
use crate::prelude::*;

pub async fn persist_update_op(
    db: impl PgExecutor<'_>,
    update: &StripeUpdate,
) -> Aresult<()> {
    query!(
        "update users set subscription_type_id = $1
        where stripe_customer_id = $2",
        update.subscription_type.as_int(),
        update.stripe_customer_id
    )
    .execute(db)
    .await?;
    Ok(())
}

pub async fn get_subscription_type(
    db: &PgPool,
    user_id: i32,
) -> Aresult<SubscriptionTypes> {
    struct Qres {
        subscription_type_id: i32,
    }
    let Qres {
        subscription_type_id,
    } = query_as!(
        Qres,
        "select subscription_type_id from users where id = $1",
        user_id
    )
    .fetch_one(db)
    .await?;

    Ok(SubscriptionTypes::from_int(subscription_type_id))
}
