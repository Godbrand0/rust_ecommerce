use crate::models::Order;
use crate::services::{DbPool, PaystackService};
use axum::{extract::State, http::StatusCode, response::Json};
use serde_json::{json, Value};
use uuid::Uuid;

#[derive(serde::Deserialize)]
pub struct InitializePaymentRequest {
    pub order_id: Uuid,
    pub email: String,
}

#[derive(serde::Deserialize)]
pub struct VerifyPaymentRequest {
    pub reference: String,
}

pub async fn initialize_payment(
    State(pool): State<DbPool>,
    State(paystack_service): State<PaystackService>,
    Json(payment_data): Json<InitializePaymentRequest>,
) -> Result<Json<Value>, (StatusCode, Json<Value>)> {
    let order_id = payment_data.order_id;
    let email = &payment_data.email;

    let order = sqlx::query_as::<_, Order>("SELECT * FROM orders WHERE id = $1")
        .bind(order_id)
        .fetch_optional(&pool)
        .await;

    match order {
        Ok(Some(order)) => {
            if order.status != "pending" {
                return Err((
                    StatusCode::BAD_REQUEST,
                    Json(json!({"error": "Order is not pending payment"})),
                ));
            }

            let reference = PaystackService::generate_reference();

            match paystack_service
                .initialize_payment(email, order.total_amount, &reference)
                .await
            {
                Ok(response) => {
                    if response.status {
                        // Use a transaction to ensure data consistency
                        let mut tx = match pool.begin().await {
                            Ok(tx) => tx,
                            Err(e) => {
                                tracing::error!("Failed to begin database transaction: {}", e);
                                return Err((
                                    StatusCode::SERVICE_UNAVAILABLE,
                                    Json(json!({"error": "Database service unavailable"})),
                                ));
                            }
                        };

                        let update_result = sqlx::query(
                            "UPDATE orders SET payment_reference = $1 WHERE id = $2",
                        )
                        .bind(&reference)
                        .bind(order_id)
                        .execute(&mut *tx)
                        .await;

                        match update_result {
                            Ok(_) => {
                                // Commit the transaction
                                if let Err(e) = tx.commit().await {
                                    tracing::error!("Failed to commit transaction: {}", e);
                                    return Err((
                                        StatusCode::INTERNAL_SERVER_ERROR,
                                        Json(json!({"error": "Failed to initialize payment"})),
                                    ));
                                }

                                Ok(Json(json!({
                                    "status": true,
                                    "message": "Payment initialized successfully",
                                    "data": {
                                        "authorization_url": response.data.authorization_url,
                                        "access_code": response.data.access_code,
                                        "reference": response.data.reference
                                    }
                                })))
                            }
                            Err(e) => {
                                // Rollback the transaction
                                if let Err(rollback_err) = tx.rollback().await {
                                    tracing::error!(
                                        "Failed to rollback transaction: {}",
                                        rollback_err
                                    );
                                }
                                tracing::error!(
                                    "Error updating order with payment reference: {}",
                                    e
                                );
                                Err((
                                    StatusCode::INTERNAL_SERVER_ERROR,
                                    Json(json!({"error": "Failed to initialize payment"})),
                                ))
                            }
                        }
                    } else {
                        Err((
                            StatusCode::BAD_REQUEST,
                            Json(json!({"error": response.message})),
                        ))
                    }
                }
                Err(e) => {
                    tracing::error!("Error initializing payment: {}", e);
                    Err((
                        StatusCode::INTERNAL_SERVER_ERROR,
                        Json(json!({"error": "Failed to initialize payment"})),
                    ))
                }
            }
        }
        Ok(None) => Err((
            StatusCode::NOT_FOUND,
            Json(json!({"error": "Order not found"})),
        )),
        Err(e) => {
            tracing::error!("Error fetching order: {}", e);
            Err((
                StatusCode::INTERNAL_SERVER_ERROR,
                Json(json!({"error": "Failed to fetch order"})),
            ))
        }
    }
}

pub async fn verify_payment(
    State(pool): State<DbPool>,
    State(paystack_service): State<PaystackService>,
    Json(payment_data): Json<VerifyPaymentRequest>,
) -> Result<Json<Value>, (StatusCode, Json<Value>)> {
    let reference = &payment_data.reference;

    match paystack_service.verify_payment(reference).await {
        Ok(response) => {
            if response.status && response.data.status == "success" {
                // Use a transaction to ensure data consistency
                let mut tx = match pool.begin().await {
                    Ok(tx) => tx,
                    Err(e) => {
                        tracing::error!("Failed to begin database transaction: {}", e);
                        return Err((
                            StatusCode::SERVICE_UNAVAILABLE,
                            Json(json!({"error": "Database service unavailable"})),
                        ));
                    }
                };

                let order =
                    sqlx::query_as::<_, Order>("SELECT * FROM orders WHERE payment_reference = $1")
                        .bind(reference)
                        .fetch_optional(&mut *tx)
                        .await;

                match order {
                    Ok(Some(order)) => {
                        let update_result = sqlx::query(
                            "UPDATE orders SET status = $1 WHERE id = $2",
                        )
                        .bind("paid")
                        .bind(order.id)
                        .execute(&mut *tx)
                        .await;

                        match update_result {
                            Ok(_) => {
                                // Commit the transaction
                                if let Err(e) = tx.commit().await {
                                    tracing::error!("Failed to commit transaction: {}", e);
                                    return Err((
                                        StatusCode::INTERNAL_SERVER_ERROR,
                                        Json(json!({"error": "Failed to verify payment"})),
                                    ));
                                }

                                Ok(Json(json!({
                                    "status": true,
                                    "message": "Payment verified successfully",
                                    "data": {
                                        "order_id": order.id,
                                        "amount": response.data.amount,
                                        "paid_at": response.data.paid_at
                                    }
                                })))
                            }
                            Err(e) => {
                                // Rollback the transaction
                                if let Err(rollback_err) = tx.rollback().await {
                                    tracing::error!(
                                        "Failed to rollback transaction: {}",
                                        rollback_err
                                    );
                                }
                                tracing::error!("Error updating order status: {}", e);
                                Err((
                                    StatusCode::INTERNAL_SERVER_ERROR,
                                    Json(json!({"error": "Failed to verify payment"})),
                                ))
                            }
                        }
                    }
                    Ok(None) => {
                        // Rollback the transaction
                        if let Err(rollback_err) = tx.rollback().await {
                            tracing::error!("Failed to rollback transaction: {}", rollback_err);
                        }
                        Err((
                            StatusCode::NOT_FOUND,
                            Json(json!({"error": "Order not found for this payment reference"})),
                        ))
                    }
                    Err(e) => {
                        // Rollback the transaction
                        if let Err(rollback_err) = tx.rollback().await {
                            tracing::error!("Failed to rollback transaction: {}", rollback_err);
                        }
                        tracing::error!("Error fetching order: {}", e);
                        Err((
                            StatusCode::INTERNAL_SERVER_ERROR,
                            Json(json!({"error": "Failed to verify payment"})),
                        ))
                    }
                }
            } else {
                Err((
                    StatusCode::BAD_REQUEST,
                    Json(json!({"error": "Payment was not successful"})),
                ))
            }
        }
        Err(e) => {
            tracing::error!("Error verifying payment: {}", e);
            Err((
                StatusCode::INTERNAL_SERVER_ERROR,
                Json(json!({"error": "Failed to verify payment"})),
            ))
        }
    }
}
