# Environment Configuration Templates

This directory contains environment configuration templates for different deployment environments.

## Files

| File | Purpose | Use When |
|------|---------|----------|
| `.env.example` | Development configuration | Local development, Docker Compose |
| `.env.production.example` | Production configuration | Production deployments, staging |

## Quick Start

### Development Setup

```bash
# 1. Copy the development template
cp templates/env/.env.example .env

# 2. Customize values (database credentials, ports, etc.)
nano .env

# 3. Start services
make dev-up
```

### Production Setup

```bash
# 1. Copy the production template
cp templates/env/.env.production.example .env.production

# 2. Replace placeholders with actual secrets
# NEVER hardcode secrets - use secret management:
# - AWS Secrets Manager
# - HashiCorp Vault
# - Azure Key Vault
# - Kubernetes Secrets

# 3. Load secrets at runtime
# Example with AWS Secrets Manager:
export JWT_SECRET=$(aws secretsmanager get-secret-value --secret-id prod/jwt-secret --query SecretString --output text)
```

## Configuration Sections

### Database Configuration

**Development:**
```bash
DATABASE_URL=postgres://myuser:mypassword@localhost:5432/mydb
DATABASE_MAX_CONNECTIONS=10
```

**Production:**
```bash
DATABASE_URL=postgres://user:${DB_PASSWORD}@prod-db.example.com:5432/prod_db
DATABASE_MAX_CONNECTIONS=50
DATABASE_SSL_MODE=require
```

### Security Configuration

**CRITICAL:** Never use default or weak values in production.

**Generating Secure Secrets:**

```bash
# JWT Secret (32 bytes, base64)
openssl rand -base64 32

# API Key (32 bytes, hex)
openssl rand -hex 32

# Encryption Key (32 bytes for AES-256, hex)
openssl rand -hex 32

# Session Secret (64 bytes, base64)
openssl rand -base64 64
```

### CORS Configuration

**Development (permissive):**
```bash
ALLOWED_ORIGINS=http://localhost:4200
```

**Production (strict):**
```bash
# NEVER use * in production - critical CSRF vulnerability
ALLOWED_ORIGINS=https://app.example.com,https://www.example.com
```

### Logging Configuration

**Development (verbose):**
```bash
LOG_LEVEL=debug
RUST_LOG=debug,sqlx=warn
```

**Production (minimal):**
```bash
LOG_LEVEL=warn
RUST_LOG=warn,sqlx=error
```

### Rate Limiting

**Development (relaxed):**
```bash
RATE_LIMIT_REQUESTS_PER_MINUTE=60
```

**Production (strict):**
```bash
RATE_LIMIT_REQUESTS_PER_MINUTE=30
RATE_LIMIT_REQUESTS_PER_HOUR=500
```

## Stack-Specific Variables

### Rust

```bash
RUST_LOG=debug,sqlx=warn,tower_http=debug
RUST_BACKTRACE=1
CARGO_WATCH_ENABLED=true  # Development only
```

### Python (FastAPI/Django)

```bash
PYTHONUNBUFFERED=1
LOG_LEVEL=INFO
DEBUG=True  # Development only
```

### Node.js (Express/NestJS)

```bash
NODE_ENV=development
DEBUG=express:*
NODEMON_ENABLED=true  # Development only
```

## Security Best Practices

### ✅ DO

1. **Use strong, random secrets**
   ```bash
   JWT_SECRET=$(openssl rand -base64 32)
   ```

2. **Load secrets from secret management**
   ```bash
   # AWS Secrets Manager
   JWT_SECRET=$(aws secretsmanager get-secret-value --secret-id prod/jwt --query SecretString --output text)
   ```

3. **Use different secrets per environment**
   - Development: Can use simpler values
   - Staging: Use real secrets, separate from production
   - Production: Maximum security

4. **Rotate secrets regularly**
   - JWT secrets: Every 90 days
   - API keys: Every 90 days
   - Database passwords: Every 180 days

5. **Enable SSL in production**
   ```bash
   DATABASE_SSL_MODE=require
   REDIS_SSL_ENABLED=true
   SSL_ENABLED=true
   ```

### ❌ DON'T

1. **Don't commit .env to git**
   ```bash
   # .gitignore should include:
   .env
   .env.local
   .env.*.local
   ```

2. **Don't use weak secrets**
   ```bash
   # ❌ NEVER do this:
   JWT_SECRET=secret123
   API_KEY=mykey
   ```

3. **Don't use * for CORS in production**
   ```bash
   # ❌ NEVER do this in production:
   ALLOWED_ORIGINS=*
   ```

4. **Don't expose sensitive data in logs**
   ```bash
   # Set appropriate log levels in production
   LOG_LEVEL=warn
   ```

5. **Don't reuse secrets across environments**
   ```bash
   # ❌ NEVER use same secret in dev and prod
   ```

## Environment-Specific Files

### Development

```bash
.env                  # Main development config
.env.local           # Local overrides (gitignored)
.env.test            # Test environment config
```

### Production

```bash
.env.production      # Production config (secrets from vault)
.env.staging         # Staging config
```

## Docker Compose Integration

**.env file is automatically loaded by Docker Compose:**

```yaml
# docker-compose.yml
services:
  backend:
    env_file:
      - .env
    environment:
      - DATABASE_URL=${DATABASE_URL}
      - REDIS_URL=${REDIS_URL}
```

## Validation

**Check configuration is valid:**

```bash
# Development
make env-validate

# Production (ensure all required secrets are set)
make env-validate-prod
```

**Example validation script:**

```bash
#!/bin/bash
required_vars=(
  "DATABASE_URL"
  "REDIS_URL"
  "JWT_SECRET"
  "API_KEY"
)

for var in "${required_vars[@]}"; do
  if [ -z "${!var}" ]; then
    echo "ERROR: $var is not set"
    exit 1
  fi
done

echo "✓ All required environment variables are set"
```

## Troubleshooting

### Issue: Database connection fails

```bash
# Check DATABASE_URL format
echo $DATABASE_URL
# Should be: postgres://user:password@host:port/database

# Test connection
psql $DATABASE_URL -c "SELECT 1;"
```

### Issue: Redis connection fails

```bash
# Check REDIS_URL format
echo $REDIS_URL
# Should be: redis://host:port

# Test connection
redis-cli -u $REDIS_URL ping
```

### Issue: CORS errors in frontend

```bash
# Check ALLOWED_ORIGINS includes frontend URL
echo $ALLOWED_ORIGINS
# Should include: http://localhost:4200 (development)
```

## References

- [The Twelve-Factor App - Config](https://12factor.net/config)
- [OWASP Secrets Management Cheat Sheet](https://cheatsheetseries.owasp.org/cheatsheets/Secrets_Management_Cheat_Sheet.html)
- [AWS Secrets Manager Best Practices](https://docs.aws.amazon.com/secretsmanager/latest/userguide/best-practices.html)

## Support

For environment configuration issues:
1. Check this README
2. Verify all required variables are set
3. Check logs for specific error messages
4. Consult the main project documentation
