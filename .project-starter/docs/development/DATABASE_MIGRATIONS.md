# Database Migration Patterns

## Overview

This guide documents database migration patterns for different tech stacks. Each framework has its own conventions for organizing and managing schema changes.

---

## Quick Reference

| Stack | Framework | Directory | Naming Pattern |
|-------|-----------|-----------|----------------|
| **Rust** | SQLx | `backend/migrations/` | `YYYYMMDDHHMMSS_description.sql` |
| **Python** | Alembic | `backend/alembic/versions/` | `XXX_description.py` |
| **Node.js** | Prisma | `backend/prisma/migrations/` | `YYYYMMDDHHMMSS_description/` |
| **Node.js** | Knex | `backend/migrations/` | `YYYYMMDDHHMMSS_description.js` |

---

## Rust + SQLx

### Directory Structure

```
backend/
└── migrations/
    ├── 20240101000000_initial_schema.sql
    ├── 20240102120000_add_users_table.sql
    ├── 20240103093000_add_sessions_table.sql
    └── 20240104151500_add_user_roles.sql
```

### File Format

Each migration is a `.sql` file with timestamp-based naming:

```sql
-- 20240102120000_add_users_table.sql

-- Add up migration
CREATE TABLE users (
    id UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    email VARCHAR(255) NOT NULL UNIQUE,
    password_hash VARCHAR(255) NOT NULL,
    created_at TIMESTAMPTZ NOT NULL DEFAULT NOW(),
    updated_at TIMESTAMPTZ NOT NULL DEFAULT NOW()
);

CREATE INDEX idx_users_email ON users(email);

-- Optionally add down migration as comment
-- To revert:
-- DROP TABLE users;
```

### Commands

```bash
# Create new migration
sqlx migrate add add_users_table

# Run pending migrations
sqlx migrate run

# Revert last migration
sqlx migrate revert

# Check migration status
sqlx migrate info
```

### Makefile Integration

```makefile
.PHONY: migrate migrate-revert migrate-create migrate-status

migrate: ## Run database migrations
	@cd backend && sqlx migrate run

migrate-revert: ## Revert last migration
	@cd backend && sqlx migrate revert

migrate-create: ## Create new migration (usage: make migrate-create name=add_users)
	@cd backend && sqlx migrate add $(name)

migrate-status: ## Show migration status
	@cd backend && sqlx migrate info
```

### Configuration

In `backend/.env`:

```bash
DATABASE_URL=postgres://postgres:postgres@localhost:5432/mydb
```

### Best Practices

✅ **DO:**
- Use descriptive migration names
- Include timestamps in filename (auto-generated)
- Keep migrations small and focused
- Add indexes in same migration as tables
- Use transactions when possible
- Test migrations before committing

❌ **DON'T:**
- Modify existing migrations after deployment
- Mix schema changes with data migrations
- Use DROP TABLE without backup
- Forget to test rollback path

---

## Python + Alembic

### Directory Structure

```
backend/
└── alembic/
    ├── env.py                    # Configuration
    ├── script.py.mako            # Template
    ├── README                    # Alembic docs
    └── versions/
        ├── 001_initial_schema.py
        ├── 002_add_users_table.py
        ├── 003_add_sessions_table.py
        └── 004_add_user_roles.py
```

### File Format

Each migration is a Python file with `upgrade()` and `downgrade()`:

```python
# alembic/versions/002_add_users_table.py

"""add users table

Revision ID: 002
Revises: 001
Create Date: 2024-01-02 12:00:00
"""

from alembic import op
import sqlalchemy as sa
from sqlalchemy.dialects import postgresql

# revision identifiers
revision = '002'
down_revision = '001'
branch_labels = None
depends_on = None


def upgrade():
    op.create_table(
        'users',
        sa.Column('id', postgresql.UUID(as_uuid=True), primary_key=True),
        sa.Column('email', sa.String(255), nullable=False, unique=True),
        sa.Column('password_hash', sa.String(255), nullable=False),
        sa.Column('created_at', sa.DateTime(timezone=True), server_default=sa.func.now()),
        sa.Column('updated_at', sa.DateTime(timezone=True), server_default=sa.func.now())
    )

    op.create_index('idx_users_email', 'users', ['email'])


def downgrade():
    op.drop_index('idx_users_email')
    op.drop_table('users')
```

### Commands

```bash
# Create new migration
alembic revision -m "add users table"

# Run all pending migrations
alembic upgrade head

# Revert one migration
alembic downgrade -1

# Revert to specific revision
alembic downgrade 001

# Show current revision
alembic current

# Show migration history
alembic history

# Auto-generate migration from models
alembic revision --autogenerate -m "add users table"
```

### Makefile Integration

```makefile
.PHONY: migrate migrate-revert migrate-create migrate-auto migrate-status

migrate: ## Run database migrations
	@cd backend && alembic upgrade head

migrate-revert: ## Revert last migration
	@cd backend && alembic downgrade -1

migrate-create: ## Create new migration (usage: make migrate-create name="add users")
	@cd backend && alembic revision -m "$(name)"

migrate-auto: ## Auto-generate migration from models
	@cd backend && alembic revision --autogenerate -m "$(name)"

migrate-status: ## Show current migration status
	@cd backend && alembic current
```

### Configuration

In `backend/alembic.ini`:

```ini
[alembic]
sqlalchemy.url = postgresql://postgres:postgres@localhost:5432/mydb
```

### Best Practices

✅ **DO:**
- Always write both upgrade() and downgrade()
- Review auto-generated migrations
- Use descriptive migration messages
- Handle data migrations separately
- Test both upgrade and downgrade paths

❌ **DON'T:**
- Trust auto-generate blindly (review changes!)
- Modify deployed migrations
- Use raw SQL when ORM methods exist
- Skip downgrade() implementation

---

## Node.js + Prisma

### Directory Structure

```
backend/
└── prisma/
    ├── schema.prisma              # Schema definition
    └── migrations/
        ├── 20240101000000_initial_schema/
        │   └── migration.sql
        ├── 20240102120000_add_users_table/
        │   └── migration.sql
        ├── 20240103093000_add_sessions_table/
        │   └── migration.sql
        └── migration_lock.toml    # Lock file
```

### File Format

Each migration is a directory with a `migration.sql` file:

```sql
-- 20240102120000_add_users_table/migration.sql

-- CreateTable
CREATE TABLE "users" (
    "id" UUID NOT NULL DEFAULT gen_random_uuid(),
    "email" VARCHAR(255) NOT NULL,
    "password_hash" VARCHAR(255) NOT NULL,
    "created_at" TIMESTAMP(3) NOT NULL DEFAULT CURRENT_TIMESTAMP,
    "updated_at" TIMESTAMP(3) NOT NULL,

    CONSTRAINT "users_pkey" PRIMARY KEY ("id")
);

-- CreateIndex
CREATE UNIQUE INDEX "users_email_key" ON "users"("email");
```

### Schema Definition

In `backend/prisma/schema.prisma`:

```prisma
datasource db {
  provider = "postgresql"
  url      = env("DATABASE_URL")
}

generator client {
  provider = "prisma-client-js"
}

model User {
  id            String   @id @default(uuid()) @db.Uuid
  email         String   @unique @db.VarChar(255)
  passwordHash  String   @map("password_hash") @db.VarChar(255)
  createdAt     DateTime @default(now()) @map("created_at")
  updatedAt     DateTime @updatedAt @map("updated_at")

  @@map("users")
}
```

### Commands

```bash
# Create migration from schema changes
npx prisma migrate dev --name add_users_table

# Run pending migrations (production)
npx prisma migrate deploy

# Reset database (dev only - DESTRUCTIVE)
npx prisma migrate reset

# Check migration status
npx prisma migrate status

# Resolve failed migration
npx prisma migrate resolve --applied <migration_name>
```

### Makefile Integration

```makefile
.PHONY: migrate migrate-create migrate-deploy migrate-reset migrate-status

migrate: migrate-deploy ## Run database migrations (production)

migrate-create: ## Create migration from schema (usage: make migrate-create name=add_users)
	@cd backend && npx prisma migrate dev --name $(name)

migrate-deploy: ## Deploy migrations (production)
	@cd backend && npx prisma migrate deploy

migrate-reset: ## Reset database (DEV ONLY - destroys data!)
	@cd backend && npx prisma migrate reset

migrate-status: ## Check migration status
	@cd backend && npx prisma migrate status
```

### Configuration

In `backend/.env`:

```bash
DATABASE_URL=postgresql://postgres:postgres@localhost:5432/mydb
```

### Best Practices

✅ **DO:**
- Edit schema.prisma, then generate migration
- Use `migrate dev` in development
- Use `migrate deploy` in production
- Commit migration files to git
- Review generated SQL before applying

❌ **DON'T:**
- Edit migration.sql files manually (use Prisma Client)
- Use `migrate reset` in production (DESTRUCTIVE!)
- Skip `migrate deploy` in CI/CD
- Forget to regenerate Prisma Client after migrations

---

## Node.js + Knex

### Directory Structure

```
backend/
└── migrations/
    ├── 20240101000000_initial_schema.js
    ├── 20240102120000_add_users_table.js
    ├── 20240103093000_add_sessions_table.js
    └── 20240104151500_add_user_roles.js
```

### File Format

Each migration is a JavaScript file with `up()` and `down()`:

```javascript
// 20240102120000_add_users_table.js

exports.up = function(knex) {
  return knex.schema.createTable('users', (table) => {
    table.uuid('id').primary().defaultTo(knex.raw('gen_random_uuid()'));
    table.string('email', 255).notNullable().unique();
    table.string('password_hash', 255).notNullable();
    table.timestamp('created_at').notNullable().defaultTo(knex.fn.now());
    table.timestamp('updated_at').notNullable().defaultTo(knex.fn.now());

    table.index('email');
  });
};

exports.down = function(knex) {
  return knex.schema.dropTable('users');
};
```

### Commands

```bash
# Create new migration
npx knex migrate:make add_users_table

# Run all pending migrations
npx knex migrate:latest

# Rollback last batch
npx knex migrate:rollback

# Rollback all migrations
npx knex migrate:rollback --all

# Check current version
npx knex migrate:currentVersion

# List completed and pending migrations
npx knex migrate:list
```

### Makefile Integration

```makefile
.PHONY: migrate migrate-revert migrate-create migrate-status

migrate: ## Run database migrations
	@cd backend && npx knex migrate:latest

migrate-revert: ## Rollback last migration batch
	@cd backend && npx knex migrate:rollback

migrate-create: ## Create new migration (usage: make migrate-create name=add_users)
	@cd backend && npx knex migrate:make $(name)

migrate-status: ## Show migration status
	@cd backend && npx knex migrate:list
```

### Configuration

In `backend/knexfile.js`:

```javascript
module.exports = {
  development: {
    client: 'postgresql',
    connection: {
      host: 'localhost',
      port: 5432,
      database: 'mydb',
      user: 'postgres',
      password: 'postgres'
    },
    migrations: {
      directory: './migrations',
      tableName: 'knex_migrations'
    }
  },

  production: {
    client: 'postgresql',
    connection: process.env.DATABASE_URL,
    migrations: {
      directory: './migrations',
      tableName: 'knex_migrations'
    }
  }
};
```

### Best Practices

✅ **DO:**
- Implement both up() and down() functions
- Use Knex schema builder (not raw SQL)
- Test rollback before deploying
- Group related changes in one migration
- Use transactions for data migrations

❌ **DON'T:**
- Modify deployed migrations
- Mix schema and data changes
- Forget to handle rollback
- Use `migrate:rollback --all` in production

---

## Common Patterns

### Pattern 1: Add Column

**SQLx (SQL):**
```sql
ALTER TABLE users ADD COLUMN phone VARCHAR(20);
```

**Alembic (Python):**
```python
def upgrade():
    op.add_column('users', sa.Column('phone', sa.String(20)))

def downgrade():
    op.drop_column('users', 'phone')
```

**Prisma (Schema):**
```prisma
model User {
  // ... existing fields
  phone String? @db.VarChar(20)
}
```

**Knex (JavaScript):**
```javascript
exports.up = (knex) => knex.schema.table('users', (table) => {
  table.string('phone', 20);
});

exports.down = (knex) => knex.schema.table('users', (table) => {
  table.dropColumn('phone');
});
```

### Pattern 2: Create Index

**SQLx (SQL):**
```sql
CREATE INDEX idx_users_email ON users(email);
```

**Alembic (Python):**
```python
def upgrade():
    op.create_index('idx_users_email', 'users', ['email'])

def downgrade():
    op.drop_index('idx_users_email')
```

**Prisma (Schema):**
```prisma
model User {
  email String @unique @db.VarChar(255)

  @@index([email])
}
```

**Knex (JavaScript):**
```javascript
exports.up = (knex) => knex.schema.table('users', (table) => {
  table.index('email');
});

exports.down = (knex) => knex.schema.table('users', (table) => {
  table.dropIndex('email');
});
```

### Pattern 3: Data Migration

**SQLx (SQL):**
```sql
-- Separate migration file for data
UPDATE users SET role = 'user' WHERE role IS NULL;
```

**Alembic (Python):**
```python
from sqlalchemy import table, column

def upgrade():
    users = table('users', column('role'))
    op.execute(
        users.update().values(role='user').where(users.c.role.is_(None))
    )
```

**Prisma (Custom Script):**
```typescript
// Run separate script, not in migration
import { PrismaClient } from '@prisma/client'

const prisma = new PrismaClient()

await prisma.user.updateMany({
  where: { role: null },
  data: { role: 'user' }
})
```

**Knex (JavaScript):**
```javascript
exports.up = async (knex) => {
  await knex('users').whereNull('role').update({ role: 'user' });
};
```

---

## Migration Workflow Best Practices

### Development Workflow

1. **Make schema change** (edit models/schema)
2. **Create migration** (`make migrate-create`)
3. **Review generated migration** (check SQL)
4. **Test migration** (`make migrate`)
5. **Test rollback** (`make migrate-revert`)
6. **Commit migration files** (`git add migrations/`)

### Production Deployment

1. **Backup database** (always!)
2. **Run migrations** (`make migrate`)
3. **Verify schema** (check tables/indexes)
4. **Monitor application** (check errors)
5. **Rollback if needed** (`make migrate-revert`)

### Troubleshooting

**Problem:** Migration fails halfway through

**Solution:**
- Check migration status
- Manually fix inconsistent state
- Use framework's resolve command
- Re-run or rollback

**Problem:** Need to modify deployed migration

**Solution:**
- DON'T modify existing migration
- Create new migration to fix issue
- Document why fix was needed

---

## Version Control

### What to Commit

✅ **Always commit:**
- Migration files
- Schema definitions
- Migration configuration

❌ **Never commit:**
- Database dumps
- Local database files (.db, .sqlite)
- Connection credentials (.env)

### Git Workflow

```bash
# After creating migration
git add backend/migrations/
git commit -m "feat: add users table migration"

# In pull request
# Reviewers should verify:
# - Migration syntax is correct
# - Rollback path exists
# - No hardcoded credentials
```

---

## References

### Framework Documentation

- **SQLx:** https://github.com/launchbadge/sqlx
- **Alembic:** https://alembic.sqlalchemy.org/
- **Prisma:** https://www.prisma.io/docs/concepts/components/prisma-migrate
- **Knex:** https://knexjs.org/guide/migrations.html

### Best Practices

- [Evolutionary Database Design](https://martinfowler.com/articles/evodb.html)
- [Database Refactoring](https://databaserefactoring.com/)
- [Zero-Downtime Deployments](https://stripe.com/blog/online-migrations)

---

**Last Updated:** 2026-01-24
**Applies to:** project-starter v1.0+
