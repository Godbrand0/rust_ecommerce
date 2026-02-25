# Rust E-commerce Backend

A minimalist e-commerce backend built with Rust and Actix-web, featuring product management, shopping cart functionality, and Paystack payment integration.

## Features

- Product catalog (name and price)
- Shopping cart management
- Order placement and management
- Paystack payment integration
- RESTful API design

## Tech Stack

- **Backend**: Rust with Actix-web framework
- **Database**: PostgreSQL with SQLx
- **Payment**: Paystack API
- **Environment**: dotenvy for configuration management

## Project Structure

```
src/
├── main.rs              # Application entry point
├── models/              # Data models
│   ├── mod.rs
│   ├── user.rs
│   ├── product.rs
│   ├── cart.rs
│   └── order.rs
├── handlers/            # Request handlers
│   ├── mod.rs
│   ├── product.rs
│   ├── cart.rs
│   ├── order.rs
│   └── payment.rs
├── routes/              # Route definitions
│   ├── mod.rs
│   ├── product.rs
│   ├── cart.rs
│   ├── order.rs
│   └── payment.rs
└── services/            # Business logic
    ├── mod.rs
    ├── database.rs
    └── payment.rs
```

## API Endpoints

### Products
- `GET /api/products` - List all products
- `GET /api/products/{id}` - Get product details

### Cart
- `POST /api/cart` - Add item to cart
- `GET /api/cart` - View cart contents
- `PUT /api/cart/{id}` - Update cart item
- `DELETE /api/cart/{id}` - Remove item from cart

### Orders
- `POST /api/orders` - Place an order
- `GET /api/orders/{id}` - Get order details

### Payment
- `POST /api/payment/initialize` - Initialize payment
- `POST /api/payment/verify` - Verify payment

## Setup

1. Clone the repository
2. Copy `.env.example` to `.env` and update with your configuration
3. Set up your PostgreSQL database
4. Run `cargo build` to build the project
5. Run `cargo run` to start the server

## Environment Variables

- `PORT`: Server port (default: 3000)
- `HOST`: Server host (default: 127.0.0.1)
- `DATABASE_URL`: PostgreSQL connection string
- `PAYSTACK_SECRET_KEY`: Your Paystack secret key
- `PAYSTACK_PUBLIC_KEY`: Your Paystack public key

## Database Schema

The application uses the following main entities:
- Users (for order tracking)
- Products (name and price)
- Cart Items (product reference and quantity)
- Orders (user reference and items)
- Payments (order reference and status)

## Payment Flow

1. User adds items to cart
2. User places an order
3. Payment is initialized with Paystack
4. User completes payment on Paystack's platform
5. Payment is verified and order is confirmed

## Contributing

1. Fork the repository
2. Create a feature branch
3. Make your changes
4. Add tests if applicable
5. Submit a pull request

## License

This project is licensed under the MIT License.