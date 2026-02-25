use axum::{
    extract::{Path, State},
    http::StatusCode,
    response::Json,
};
use serde_json::{json, Value};
use uuid::Uuid;
use crate::models::{Order, OrderResponse, OrderItemResponse, CreateOrder};
use crate::services::DbPool;

#[derive(sqlx::FromRow)]
struct OrderItemRow {
    product_id: Uuid,
    product_name: String,
    product_price: i32,
    quantity: i32,
}

#[derive(sqlx::FromRow)]
struct CartItemForOrder {
    product_id: Uuid,
    quantity: i32,
    product_name: String,
    product_price: i32,
}

pub async fn get_orders(
    State(pool): State<DbPool>,
) -> Result<Json<Vec<OrderResponse>>, (StatusCode, Json<Value>)> {
    let orders = sqlx::query_as::<_, Order>("SELECT * FROM orders ORDER BY created_at DESC")
        .fetch_all(&pool)
        .await;

    match orders {
        Ok(orders) => {
            let mut order_responses = Vec::new();

            for order in orders {
                let order_items = sqlx::query_as::<_, OrderItemRow>(
                    r#"
                    SELECT oi.product_id, oi.product_name, oi.product_price, oi.quantity
                    FROM order_items oi
                    WHERE oi.order_id = $1
                    "#,
                )
                .bind(order.id)
                .fetch_all(&pool)
                .await;

                match order_items {
                    Ok(items) => {
                        let order_item_responses: Vec<OrderItemResponse> = items
                            .into_iter()
                            .map(|item| {
                                OrderItemResponse::new(
                                    item.product_id,
                                    item.product_name,
                                    item.product_price,
                                    item.quantity,
                                )
                            })
                            .collect();

                        let order_response = OrderResponse::new(order, order_item_responses);
                        order_responses.push(order_response);
                    }
                    Err(e) => {
                        tracing::error!("Error fetching order items: {}", e);
                        return Err((
                            StatusCode::INTERNAL_SERVER_ERROR,
                            Json(json!({"error": "Failed to fetch order items"})),
                        ));
                    }
                }
            }

            Ok(Json(order_responses))
        }
        Err(e) => {
            tracing::error!("Error fetching orders: {}", e);
            Err((
                StatusCode::INTERNAL_SERVER_ERROR,
                Json(json!({"error": "Failed to fetch orders"})),
            ))
        }
    }
}

pub async fn get_user_orders(
    Path(user_id): Path<Uuid>,
    State(pool): State<DbPool>,
) -> Result<Json<Vec<OrderResponse>>, (StatusCode, Json<Value>)> {
    let orders = sqlx::query_as::<_, Order>(
        "SELECT * FROM orders WHERE user_id = $1 ORDER BY created_at DESC",
    )
    .bind(user_id)
    .fetch_all(&pool)
    .await;

    match orders {
        Ok(orders) => {
            let mut order_responses = Vec::new();

            for order in orders {
                let order_items = sqlx::query_as::<_, OrderItemRow>(
                    r#"
                    SELECT oi.product_id, oi.product_name, oi.product_price, oi.quantity
                    FROM order_items oi
                    WHERE oi.order_id = $1
                    "#,
                )
                .bind(order.id)
                .fetch_all(&pool)
                .await;

                match order_items {
                    Ok(items) => {
                        let order_item_responses: Vec<OrderItemResponse> = items
                            .into_iter()
                            .map(|item| {
                                OrderItemResponse::new(
                                    item.product_id,
                                    item.product_name,
                                    item.product_price,
                                    item.quantity,
                                )
                            })
                            .collect();

                        let order_response = OrderResponse::new(order, order_item_responses);
                        order_responses.push(order_response);
                    }
                    Err(e) => {
                        tracing::error!("Error fetching order items: {}", e);
                        return Err((
                            StatusCode::INTERNAL_SERVER_ERROR,
                            Json(json!({"error": "Failed to fetch order items"})),
                        ));
                    }
                }
            }

            Ok(Json(order_responses))
        }
        Err(e) => {
            tracing::error!("Error fetching orders: {}", e);
            Err((
                StatusCode::INTERNAL_SERVER_ERROR,
                Json(json!({"error": "Failed to fetch orders"})),
            ))
        }
    }
}

pub async fn get_order(
    Path((user_id, order_id)): Path<(Uuid, Uuid)>,
    State(pool): State<DbPool>,
) -> Result<Json<OrderResponse>, (StatusCode, Json<Value>)> {
    let order = sqlx::query_as::<_, Order>(
        "SELECT * FROM orders WHERE id = $1 AND user_id = $2",
    )
    .bind(order_id)
    .bind(user_id)
    .fetch_optional(&pool)
    .await;

    match order {
        Ok(Some(order)) => {
            let order_items = sqlx::query_as::<_, OrderItemRow>(
                r#"
                SELECT oi.product_id, oi.product_name, oi.product_price, oi.quantity
                FROM order_items oi
                WHERE oi.order_id = $1
                "#,
            )
            .bind(order.id)
            .fetch_all(&pool)
            .await;

            match order_items {
                Ok(items) => {
                    let order_item_responses: Vec<OrderItemResponse> = items
                        .into_iter()
                        .map(|item| {
                            OrderItemResponse::new(
                                item.product_id,
                                item.product_name,
                                item.product_price,
                                item.quantity,
                            )
                        })
                        .collect();

                    let order_response = OrderResponse::new(order, order_item_responses);
                    Ok(Json(order_response))
                }
                Err(e) => {
                    tracing::error!("Error fetching order items: {}", e);
                    Err((
                        StatusCode::INTERNAL_SERVER_ERROR,
                        Json(json!({"error": "Failed to fetch order items"})),
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

pub async fn create_order(
    State(pool): State<DbPool>,
    Json(order_data): Json<CreateOrder>,
) -> Result<Json<OrderResponse>, (StatusCode, Json<Value>)> {
    let user_id = order_data.user_id;
    let now = chrono::Utc::now();

    let cart_items = sqlx::query_as::<_, CartItemForOrder>(
        r#"
        SELECT ci.product_id, ci.quantity, p.name as product_name, p.price as product_price
        FROM cart_items ci
        JOIN products p ON ci.product_id = p.id
        WHERE ci.user_id = $1
        "#,
    )
    .bind(user_id)
    .fetch_all(&pool)
    .await;

    match cart_items {
        Ok(items) => {
            if items.is_empty() {
                return Err((
                    StatusCode::BAD_REQUEST,
                    Json(json!({"error": "Cart is empty"})),
                ));
            }

            let total_amount: i32 = items.iter().map(|item| item.product_price * item.quantity).sum();

            let new_order_id = Uuid::new_v4();
            let result = sqlx::query(
                r#"
                INSERT INTO orders (id, user_id, total_amount, status, created_at, updated_at)
                VALUES ($1, $2, $3, $4, $5, $6)
                "#,
            )
            .bind(new_order_id)
            .bind(user_id)
            .bind(total_amount)
            .bind("pending")
            .bind(now)
            .bind(now)
            .execute(&pool)
            .await;

            match result {
                Ok(_) => {
                    for item in items {
                        let subtotal = item.product_price * item.quantity;
                        let order_item_id = Uuid::new_v4();

                        let result = sqlx::query(
                            r#"
                            INSERT INTO order_items (id, order_id, product_id, product_name, product_price, quantity, subtotal)
                            VALUES ($1, $2, $3, $4, $5, $6, $7)
                            "#,
                        )
                        .bind(order_item_id)
                        .bind(new_order_id)
                        .bind(item.product_id)
                        .bind(&item.product_name)
                        .bind(item.product_price)
                        .bind(item.quantity)
                        .bind(subtotal)
                        .execute(&pool)
                        .await;

                        if let Err(e) = result {
                            tracing::error!("Error creating order item: {}", e);
                            return Err((
                                StatusCode::INTERNAL_SERVER_ERROR,
                                Json(json!({"error": "Failed to create order items"})),
                            ));
                        }
                    }

                    let clear_cart_result =
                        sqlx::query("DELETE FROM cart_items WHERE user_id = $1")
                            .bind(user_id)
                            .execute(&pool)
                            .await;

                    if let Err(e) = clear_cart_result {
                        tracing::error!("Error clearing cart: {}", e);
                    }

                    let order = sqlx::query_as::<_, Order>("SELECT * FROM orders WHERE id = $1")
                        .bind(new_order_id)
                        .fetch_one(&pool)
                        .await;

                    match order {
                        Ok(order) => {
                            let order_items = sqlx::query_as::<_, OrderItemRow>(
                                r#"
                                SELECT oi.product_id, oi.product_name, oi.product_price, oi.quantity
                                FROM order_items oi
                                WHERE oi.order_id = $1
                                "#,
                            )
                            .bind(order.id)
                            .fetch_all(&pool)
                            .await;

                            match order_items {
                                Ok(items) => {
                                    let order_item_responses: Vec<OrderItemResponse> = items
                                        .into_iter()
                                        .map(|item| {
                                            OrderItemResponse::new(
                                                item.product_id,
                                                item.product_name,
                                                item.product_price,
                                                item.quantity,
                                            )
                                        })
                                        .collect();

                                    let order_response =
                                        OrderResponse::new(order, order_item_responses);
                                    Ok(Json(order_response))
                                }
                                Err(e) => {
                                    tracing::error!("Error fetching order items: {}", e);
                                    Err((
                                        StatusCode::INTERNAL_SERVER_ERROR,
                                        Json(json!({"error": "Failed to fetch order items"})),
                                    ))
                                }
                            }
                        }
                        Err(e) => {
                            tracing::error!("Error fetching created order: {}", e);
                            Err((
                                StatusCode::INTERNAL_SERVER_ERROR,
                                Json(json!({"error": "Failed to create order"})),
                            ))
                        }
                    }
                }
                Err(e) => {
                    tracing::error!("Error creating order: {}", e);
                    Err((
                        StatusCode::INTERNAL_SERVER_ERROR,
                        Json(json!({"error": "Failed to create order"})),
                    ))
                }
            }
        }
        Err(e) => {
            tracing::error!("Error fetching cart items: {}", e);
            Err((
                StatusCode::INTERNAL_SERVER_ERROR,
                Json(json!({"error": "Failed to fetch cart items"})),
            ))
        }
    }
}
