# Database Setup Guide

## Prerequisites
- PostgreSQL installed on your system
- Database created for the application

## Setup Steps

### 1. Install PostgreSQL
```bash
# On Ubuntu/Debian
sudo apt update
sudo apt install postgresql postgresql-contrib

# On macOS (using Homebrew)
brew install postgresql
brew services start postgresql

# On Windows
# Download and install from https://www.postgresql.org/download/windows/
```

### 2. Create Database and User
```bash
# Switch to postgres user
sudo -u postgres psql

# In PostgreSQL shell
CREATE DATABASE rust_ecommerce;
CREATE USER username WITH PASSWORD 'password';
GRANT ALL PRIVILEGES ON DATABASE rust_ecommerce TO username;
\q
```

### 3. Update .env File
Update the DATABASE_URL in your .env file with your actual database credentials:
```
DATABASE_URL=postgresql://username:password@localhost/rust_ecommerce
```

### 4. Run Migrations
```bash
cd ecommerce
cargo install sqlx-cli
sqlx migrate run --database-url "postgresql://username:password@localhost/rust_ecommerce"
```

### 5. Start PostgreSQL Service
```bash
# On Ubuntu/Debian
sudo systemctl start postgresql
sudo systemctl enable postgresql

# On macOS
brew services start postgresql

# On Windows
# Start the PostgreSQL service from Services or run:
net start postgresql-x64-14  # Version may vary
```

## Alternative: Use Docker
If you prefer using Docker:

```bash
docker run --name postgres-ecommerce \
  -e POSTGRES_DB=rust_ecommerce \
  -e POSTGRES_USER=username \
  -e POSTGRES_PASSWORD=password \
  -p 5432:5432 \
  -d postgres:latest
```

Then update your .env with:
```
DATABASE_URL=postgresql://username:password@localhost:5432/rust_ecommerce
```

## Troubleshooting
- If you get "Connection refused (os error 111)", ensure PostgreSQL is running
- Check if the database exists and credentials are correct
- Verify the port (default is 5432) is not blocked by firewall