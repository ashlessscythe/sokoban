# Sokoban Application

A web application built with Rocket, Tera, and SQLx for managing user statuses and checklists. This application handles user registration, authentication, and updates to user statuses and checklist items.

## Features

- User registration and authentication
- Management and display of user statuses
- Updating checklist statuses for users
- Rendering templates with user and department data

## Getting Started

### Prerequisites

- [Rust](https://www.rust-lang.org/)
- [Docker](https://www.docker.com/)
- [Docker Compose](https://docs.docker.com/compose/)

### Installation

1. Clone the repository:

   ```bash
   git clone https://github.com/yourusername/sokoban.git
   cd sokoban
   ```

2. Create a `.env` (you can use .env.example) file in the root of the project and add the following environment variables:

   ```env
   # SIMULATE_DB_OFFLINE=true # comment out to prevent db from being offline testing stuff
   LOCAL_DB=postgres://user:password@localhost:5432/dbname
   SECRET_KEY=secret
   DEV_DB=postgres://user:password@localhost:5432/dbname
   DATABASE_URL=postgres://user:password@localhost:5432/dbname
   COOKIE_TTL=30
   LAST_PUNCH_HOURS=24
   COMPANY_LOGO_URL=imgur/imgbb/logo/localfile
   UNSPLASH_ACCESS_KEY=your_unsplash_access_key_here
   ```

3. Modify the values in the `.env` file as necessary.

### Running the Application

1. Build and start the application using Docker Compose:

   ```bash
   docker-compose up --build
   ```

2. The application should now be running and accessible at `http://localhost:8000` (or the port specified in your Docker Compose file).

## Environment Variables

The application requires the following environment variables to be set in the `.env` file:

- `SIMULATE_DB_OFFLINE`: Comment out to prevent the database from being offline during testing.
- `LOCAL_DB`: The connection string for the local PostgreSQL database.
- `SECRET_KEY`: A secret key used for application security.
- `DEV_DB`: The connection string for the development PostgreSQL database.
- `DATABASE_URL`: The connection string for the main PostgreSQL database.
- `COOKIE_TTL`: The time-to-live for cookies in minutes.
- `LAST_PUNCH_HOURS`: The number of hours to consider for the last punch.
- `COMPANY_LOGO_URL`: The URL to the company logo image. (can be local file or URL)
- `UNSPLASH_ACCESS_KEY`: The access key for the Unsplash API to fetch random images.

## Prisma installation and setup

- npm install
- Make sure DATABASE_URL is set in your .env
- npx prisma generate
- OR
- npx prisma introspect ## to get the current db schema

## Instructions for Each Script

1. **`exportToCsv.js`**:

   - Exports data from a specified table to a CSV file.
   - Usage: `node prisma_scripts/exportToCsv.js <table_name>`
   - Example: `node prisma_scripts/exportToCsv.js users`

2. **`updateUserId.js`**:

   - Updates user IDs based on the contents of a CSV file.
   - CSV Format: Should have columns `old_id` and `new_id`.
   - Usage: `node prisma_scripts/updateUserId.js <csv_file_path>`
   - Example: `node prisma_scripts/updateUserId.js ./prisma_scripts/update_user.csv`

3. **`upsertFromCsv.js`**:
   - Upserts data into the specified table based on the CSV file name and its contents.
   - CSV Format: Should match the table's structure and have appropriate headers.
   - Usage: `node prisma_scripts/upsertFromCsv.js <csv_file_path>`
   - Example: `node prisma_scripts/upsertFromCsv.js ./prisma_scripts/users.csv`

## License

This project is licensed under the MIT License - see the [LICENSE](LICENSE) file for details.
