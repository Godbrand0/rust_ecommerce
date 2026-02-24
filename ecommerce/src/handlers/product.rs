use axum::{
    extract::{Path, State},
    http::StatusCode,
    response::Json,
};
use serde_json::{json, Value};
use uuid::Uuid;
use crate::models::{Product, ProductResponse, CreateProduct};
use crate::services::DbPool;

pub async fn get_products(
    State(pool): State<DbPool>,
) -> Result<Json<Vec<ProductResponse>>, (StatusCode, Json<Value>)> {
    let products = sqlx::query_as::<_, Product>("SELECT * FROM products ORDER BY created_at DESC")
        .fetch_all(&pool)
        .await;

    match products {
        Ok(products) => {
            let product_responses: Vec<ProductResponse> = products
                .into_iter()
                .map(ProductResponse::from)
                .collect();

            Ok(Json(product_responses))
        }
        Err(e) => {
            tracing::error!("Error fetching products: {}", e);
            Err((
                StatusCode::INTERNAL_SERVER_ERROR,
                Json(json!({"error": "Failed to fetch products"})),
            ))
        }
    }
}

pub async fn get_product(
    Path(product_id): Path<Uuid>,
    State(pool): State<DbPool>,
) -> Result<Json<ProductResponse>, (StatusCode, Json<Value>)> {
    let product = sqlx::query_as::<_, Product>("SELECT * FROM products WHERE id = $1")
        .bind(product_id)
        .fetch_optional(&pool)
        .await;

    match product {
        Ok(Some(product)) => {
            let product_response = ProductResponse::from(product);
            Ok(Json(product_response))
        }
        Ok(None) => Err((
            StatusCode::NOT_FOUND,
            Json(json!({"error": "Product not found"})),
        )),
        Err(e) => {
            tracing::error!("Error fetching product: {}", e);
            Err((
                StatusCode::INTERNAL_SERVER_ERROR,
                Json(json!({"error": "Failed to fetch product"})),
            ))
        }
    }
}

pub async fn create_product(
    State(pool): State<DbPool>,
    Json(product_data): Json<CreateProduct>,
) -> Result<Json<ProductResponse>, (StatusCode, Json<Value>)> {
    let new_product_id = Uuid::new_v4();
    let now = chrono::Utc::now();

    let result = sqlx::query(
        r#"
        INSERT INTO products (id, name, price, created_at, updated_at)
        VALUES ($1, $2, $3, $4, $5)
        "#,
    )
    .bind(new_product_id)
    .bind(&product_data.name)
    .bind(product_data.price)
    .bind(now)
    .bind(now)
    .execute(&pool)
    .await;

    match result {
        Ok(_) => {
            let product = sqlx::query_as::<_, Product>("SELECT * FROM products WHERE id = $1")
                .bind(new_product_id)
                .fetch_one(&pool)
                .await;

            match product {
                Ok(product) => {
                    let product_response = ProductResponse::from(product);
                    Ok(Json(product_response))
                }
                Err(e) => {
                    tracing::error!("Error fetching created product: {}", e);
                    Err((
                        StatusCode::INTERNAL_SERVER_ERROR,
                        Json(json!({"error": "Failed to create product"})),
                    ))
                }
            }
        }
        Err(e) => {
            tracing::error!("Error creating product: {}", e);
            Err((
                StatusCode::INTERNAL_SERVER_ERROR,
                Json(json!({"error": "Failed to create product"})),
            ))
        }
    }
}
