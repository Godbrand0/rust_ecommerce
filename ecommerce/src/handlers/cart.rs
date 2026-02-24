use axum::{
    extract::{Path, State},
    http::StatusCode,
    response::Json,
};
use serde_json::{json, Value};
use uuid::Uuid;
use crate::models::{CartItemResponse, CartResponse, AddToCart, UpdateCartItem};
use crate::services::DbPool;

#[derive(sqlx::FromRow)]
struct CartItemRow {
    id: Uuid,
    product_id: Uuid,
    quantity: i32,
    product_name: String,
    product_price: i32,
}

#[derive(sqlx::FromRow)]
struct CartItemBasic {
    id: Uuid,
    quantity: i32,
}

pub async fn get_cart(
    Path(user_id): Path<Uuid>,
    State(pool): State<DbPool>,
) -> Result<Json<CartResponse>, (StatusCode, Json<Value>)> {
    let cart_items = sqlx::query_as::<_, CartItemRow>(
        r#"
        SELECT ci.id, ci.product_id, ci.quantity, p.name as product_name, p.price as product_price
        FROM cart_items ci
        JOIN products p ON ci.product_id = p.id
        WHERE ci.user_id = $1
        ORDER BY ci.created_at DESC
        "#,
    )
    .bind(user_id)
    .fetch_all(&pool)
    .await;

    match cart_items {
        Ok(items) => {
            let cart_item_responses: Vec<CartItemResponse> = items
                .into_iter()
                .map(|item| {
                    CartItemResponse::new(
                        item.id,
                        item.product_id,
                        item.product_name,
                        item.product_price,
                        item.quantity,
                    )
                })
                .collect();

            let cart_response = CartResponse::new(cart_item_responses);
            Ok(Json(cart_response))
        }
        Err(e) => {
            tracing::error!("Error fetching cart: {}", e);
            Err((
                StatusCode::INTERNAL_SERVER_ERROR,
                Json(json!({"error": "Failed to fetch cart"})),
            ))
        }
    }
}

pub async fn add_to_cart(
    Path(user_id): Path<Uuid>,
    State(pool): State<DbPool>,
    Json(cart_data): Json<AddToCart>,
) -> Result<Json<Value>, (StatusCode, Json<Value>)> {
    let now = chrono::Utc::now();

    let product_exists = sqlx::query("SELECT id FROM products WHERE id = $1")
        .bind(cart_data.product_id)
        .fetch_optional(&pool)
        .await;

    match product_exists {
        Ok(Some(_)) => {
            let existing_item = sqlx::query_as::<_, CartItemBasic>(
                "SELECT id, quantity FROM cart_items WHERE user_id = $1 AND product_id = $2",
            )
            .bind(user_id)
            .bind(cart_data.product_id)
            .fetch_optional(&pool)
            .await;

            match existing_item {
                Ok(Some(item)) => {
                    let new_quantity = item.quantity + cart_data.quantity;
                    let result = sqlx::query(
                        "UPDATE cart_items SET quantity = $1, updated_at = $2 WHERE id = $3",
                    )
                    .bind(new_quantity)
                    .bind(now)
                    .bind(item.id)
                    .execute(&pool)
                    .await;

                    match result {
                        Ok(_) => Ok(Json(json!({
                            "message": "Cart item updated successfully",
                            "quantity": new_quantity
                        }))),
                        Err(e) => {
                            tracing::error!("Error updating cart item: {}", e);
                            Err((
                                StatusCode::INTERNAL_SERVER_ERROR,
                                Json(json!({"error": "Failed to update cart item"})),
                            ))
                        }
                    }
                }
                Ok(None) => {
                    let new_cart_item_id = Uuid::new_v4();
                    let result = sqlx::query(
                        r#"
                        INSERT INTO cart_items (id, user_id, product_id, quantity, created_at, updated_at)
                        VALUES ($1, $2, $3, $4, $5, $6)
                        "#,
                    )
                    .bind(new_cart_item_id)
                    .bind(user_id)
                    .bind(cart_data.product_id)
                    .bind(cart_data.quantity)
                    .bind(now)
                    .bind(now)
                    .execute(&pool)
                    .await;

                    match result {
                        Ok(_) => Ok(Json(json!({
                            "message": "Item added to cart successfully",
                            "cart_item_id": new_cart_item_id
                        }))),
                        Err(e) => {
                            tracing::error!("Error adding to cart: {}", e);
                            Err((
                                StatusCode::INTERNAL_SERVER_ERROR,
                                Json(json!({"error": "Failed to add item to cart"})),
                            ))
                        }
                    }
                }
                Err(e) => {
                    tracing::error!("Error checking existing cart item: {}", e);
                    Err((
                        StatusCode::INTERNAL_SERVER_ERROR,
                        Json(json!({"error": "Failed to check cart"})),
                    ))
                }
            }
        }
        Ok(None) => Err((
            StatusCode::NOT_FOUND,
            Json(json!({"error": "Product not found"})),
        )),
        Err(e) => {
            tracing::error!("Error checking product: {}", e);
            Err((
                StatusCode::INTERNAL_SERVER_ERROR,
                Json(json!({"error": "Failed to check product"})),
            ))
        }
    }
}

pub async fn update_cart_item(
    Path((user_id, item_id)): Path<(Uuid, Uuid)>,
    State(pool): State<DbPool>,
    Json(update_data): Json<UpdateCartItem>,
) -> Result<Json<Value>, (StatusCode, Json<Value>)> {
    let now = chrono::Utc::now();

    let cart_item = sqlx::query(
        "SELECT id FROM cart_items WHERE id = $1 AND user_id = $2",
    )
    .bind(item_id)
    .bind(user_id)
    .fetch_optional(&pool)
    .await;

    match cart_item {
        Ok(Some(_)) => {
            if update_data.quantity <= 0 {
                let result = sqlx::query("DELETE FROM cart_items WHERE id = $1")
                    .bind(item_id)
                    .execute(&pool)
                    .await;

                match result {
                    Ok(_) => Ok(Json(json!({"message": "Cart item removed successfully"}))),
                    Err(e) => {
                        tracing::error!("Error removing cart item: {}", e);
                        Err((
                            StatusCode::INTERNAL_SERVER_ERROR,
                            Json(json!({"error": "Failed to remove cart item"})),
                        ))
                    }
                }
            } else {
                let result = sqlx::query(
                    "UPDATE cart_items SET quantity = $1, updated_at = $2 WHERE id = $3",
                )
                .bind(update_data.quantity)
                .bind(now)
                .bind(item_id)
                .execute(&pool)
                .await;

                match result {
                    Ok(_) => Ok(Json(json!({
                        "message": "Cart item updated successfully",
                        "quantity": update_data.quantity
                    }))),
                    Err(e) => {
                        tracing::error!("Error updating cart item: {}", e);
                        Err((
                            StatusCode::INTERNAL_SERVER_ERROR,
                            Json(json!({"error": "Failed to update cart item"})),
                        ))
                    }
                }
            }
        }
        Ok(None) => Err((
            StatusCode::NOT_FOUND,
            Json(json!({"error": "Cart item not found"})),
        )),
        Err(e) => {
            tracing::error!("Error checking cart item: {}", e);
            Err((
                StatusCode::INTERNAL_SERVER_ERROR,
                Json(json!({"error": "Failed to check cart item"})),
            ))
        }
    }
}

pub async fn remove_from_cart(
    Path((user_id, item_id)): Path<(Uuid, Uuid)>,
    State(pool): State<DbPool>,
) -> Result<Json<Value>, (StatusCode, Json<Value>)> {
    let cart_item = sqlx::query(
        "SELECT id FROM cart_items WHERE id = $1 AND user_id = $2",
    )
    .bind(item_id)
    .bind(user_id)
    .fetch_optional(&pool)
    .await;

    match cart_item {
        Ok(Some(_)) => {
            let result = sqlx::query("DELETE FROM cart_items WHERE id = $1")
                .bind(item_id)
                .execute(&pool)
                .await;

            match result {
                Ok(_) => Ok(Json(json!({"message": "Cart item removed successfully"}))),
                Err(e) => {
                    tracing::error!("Error removing cart item: {}", e);
                    Err((
                        StatusCode::INTERNAL_SERVER_ERROR,
                        Json(json!({"error": "Failed to remove cart item"})),
                    ))
                }
            }
        }
        Ok(None) => Err((
            StatusCode::NOT_FOUND,
            Json(json!({"error": "Cart item not found"})),
        )),
        Err(e) => {
            tracing::error!("Error checking cart item: {}", e);
            Err((
                StatusCode::INTERNAL_SERVER_ERROR,
                Json(json!({"error": "Failed to check cart item"})),
            ))
        }
    }
}
