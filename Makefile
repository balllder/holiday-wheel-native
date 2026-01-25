# Makefile for Rust + Axum + React Project
# Generated from project-starter template

.PHONY: help setup dev-up dev-down dev-logs logs-backend logs-frontend logs-db logs-cache health health-ready health-all test lint format clean

# Default target
.DEFAULT_GOAL := help

#=============================================================================
# Help
#=============================================================================

help: ## Show this help message
	@echo "Available commands:"
	@grep -E '^[a-zA-Z_-]+:.*?## .*$$' $(MAKEFILE_LIST) | sort | awk 'BEGIN {FS = ":.*?## "}; {printf "\033[36m%-30s\033[0m %s\n", $$1, $$2}'

#=============================================================================
# Setup
#=============================================================================

setup: ## Initial project setup
	@echo "Setting up Rust + Axum + React project..."
	@make backend-setup
	@make frontend-setup
	@make setup-hooks
	@echo "✓ Setup complete"

setup-hooks: ## Install Git hooks
	@echo "Installing Git hooks..."
	@cp templates/git-hooks/pre-commit .git/hooks/pre-commit
	@chmod +x .git/hooks/pre-commit
	@echo "✓ Pre-commit hook installed"

backend-setup: ## Setup backend (Rust)
	@echo "Installing Rust dependencies..."
	@cd backend && cargo build
	@echo "✓ Backend setup complete"

frontend-setup: ## Setup frontend (React)
	@echo "Installing frontend dependencies..."
	@cd frontend && npm install
	@echo "✓ Frontend setup complete"

#=============================================================================
# Development
#=============================================================================

dev-up: ## Start development environment
	@echo "Starting development environment..."
	@DOCKER_BUILDKIT=1 COMPOSE_DOCKER_CLI_BUILD=1 docker-compose up -d
	@echo "✓ Services started"
	@echo "Backend: http://localhost:3000"
	@echo "Frontend: http://localhost:5173"
	@echo "Database: localhost:5432"

dev-up-watch: ## Start development with hot-reload
	@echo "Starting development with hot-reload..."
	@DOCKER_BUILDKIT=1 COMPOSE_DOCKER_CLI_BUILD=1 docker-compose -f docker-compose.yml -f docker-compose.dev.yml up -d backend-dev
	@echo "✓ Development environment started with hot-reload"
	@echo "Backend (hot-reload): http://localhost:3000"
	@echo "Edit src/ files - changes auto-reload!"

dev-down: ## Stop development environment
	@docker-compose down

dev-logs: ## Show all service logs
	@docker-compose logs -f

logs-backend: ## Show backend service logs only
	@docker-compose logs -f backend

logs-frontend: ## Show frontend service logs only
	@docker-compose logs -f frontend

logs-db: ## Show database logs only
	@docker-compose logs -f postgres

logs-cache: ## Show cache (Redis) logs only
	@docker-compose logs -f redis

dev-restart: dev-down dev-up ## Restart development environment

#=============================================================================
# Health Checks
#=============================================================================

health: ## Check service health (liveness probe)
	@echo "Checking service health..."
	@curl -sf http://localhost:3000/health || (echo "✗ Service unhealthy" && exit 1)
	@echo "✓ Service is healthy"

health-ready: ## Check service readiness (readiness probe)
	@echo "Checking service readiness..."
	@curl -sf http://localhost:3000/health/ready || (echo "✗ Service not ready" && exit 1)
	@echo "✓ Service is ready"

health-all: health health-ready ## Run all health checks

#=============================================================================
# Backend (Rust + Axum)
#=============================================================================

backend-build: ## Build backend
	@cd backend && cargo build

backend-build-release: ## Build backend (release mode)
	@cd backend && cargo build --release

backend-run: ## Run backend locally
	@cd backend && cargo run

backend-watch: ## Run backend with auto-reload
	@cd backend && cargo watch -x run

backend-test-unit: ## Run backend unit tests
	@cd backend && cargo test --lib

backend-test-integration: ## Run backend integration tests
	@cd backend && cargo test --test '*'

backend-test: backend-test-unit backend-test-integration ## Run all backend tests

backend-test-coverage: ## Run backend tests with coverage
	@cd backend && cargo tarpaulin --out Html --output-dir coverage

backend-lint: ## Lint backend code
	@cd backend && cargo clippy -- -D warnings

backend-format: ## Format backend code
	@cd backend && cargo fmt

backend-format-check: ## Check backend code formatting
	@cd backend && cargo fmt -- --check

#=============================================================================
# Frontend (React + TypeScript)
#=============================================================================

frontend-build: ## Build frontend
	@cd frontend && npm run build

frontend-dev: ## Run frontend dev server
	@cd frontend && npm run dev

frontend-test: ## Run frontend tests
	@cd frontend && npm test

frontend-test-watch: ## Run frontend tests in watch mode
	@cd frontend && npm test -- --watch

frontend-lint: ## Lint frontend code
	@cd frontend && npm run lint

frontend-format: ## Format frontend code
	@cd frontend && npm run format

frontend-format-check: ## Check frontend formatting
	@cd frontend && npm run format:check

frontend-type-check: ## TypeScript type checking
	@cd frontend && npm run type-check

#=============================================================================
# Code Generation (OpenAPI)
#=============================================================================

openapi-generate: ## Generate OpenAPI schema from backend
	@echo "Generating OpenAPI schema..."
	@cd backend && cargo run --bin generate-openapi > ../frontend/src/api/openapi.json
	@echo "✓ OpenAPI schema generated"

codegen-frontend: openapi-generate ## Generate TypeScript client
	@echo "Generating TypeScript client..."
	@cd frontend && npm run codegen
	@echo "✓ TypeScript client generated"

codegen-all: openapi-generate codegen-frontend ## Generate all code

codegen-check: ## Check if generated code is up-to-date
	@make codegen-all
	@if git diff --quiet frontend/src/api/; then \
		echo "✓ Generated code is up-to-date"; \
	else \
		echo "ERROR: Generated code is out of date. Run 'make codegen-all'"; \
		exit 1; \
	fi

#=============================================================================
# Testing (All)
#=============================================================================

test-unit: backend-test-unit frontend-test ## Run all unit tests

test-integration: backend-test-integration ## Run integration tests

test-e2e: ## Run E2E tests
	@cd frontend && npx playwright test

test-e2e-ui: ## Run E2E tests with UI
	@cd frontend && npx playwright test --ui

test-e2e-debug: ## Debug E2E tests
	@cd frontend && npx playwright test --debug

test-e2e-report: ## Show E2E test report
	@cd frontend && npx playwright show-report

test-e2e-codegen: ## Generate E2E test code
	@cd frontend && npx playwright codegen http://localhost:4200

test: test-unit test-integration test-e2e ## Run all tests

test-coverage: backend-test-coverage ## Run tests with coverage
	@cd frontend && npm test -- --coverage

#=============================================================================
# Quality Checks
#=============================================================================

lint: backend-lint frontend-lint ## Lint all code

format: backend-format frontend-format ## Format all code

format-check: backend-format-check frontend-format-check ## Check code formatting

type-check: frontend-type-check ## Type check TypeScript

quality: format-check lint type-check ## Run all quality checks

#=============================================================================
# CI/CD
#=============================================================================

ci-quick: format-check lint test-unit ## Fast CI checks (< 3 min)

ci: quality test codegen-check ## Full CI validation (< 10 min)

pre-commit: ci-quick ## Pre-commit hook

#=============================================================================
# Database Migrations
#=============================================================================
# See docs/development/DATABASE_MIGRATIONS.md for detailed patterns

# Rust + SQLx (default)
migrate: ## Run database migrations
	@cd backend && sqlx migrate run

migrate-revert: ## Revert last migration
	@cd backend && sqlx migrate revert

migrate-create: ## Create new migration (usage: make migrate-create name=add_users)
	@cd backend && sqlx migrate add $(name)

migrate-status: ## Show migration status
	@cd backend && sqlx migrate info

# Python + Alembic (uncomment to use)
# migrate: ## Run database migrations
# 	@cd backend && alembic upgrade head
#
# migrate-revert: ## Revert last migration
# 	@cd backend && alembic downgrade -1
#
# migrate-create: ## Create new migration
# 	@cd backend && alembic revision -m "$(name)"
#
# migrate-auto: ## Auto-generate migration from models
# 	@cd backend && alembic revision --autogenerate -m "$(name)"
#
# migrate-status: ## Show current migration status
# 	@cd backend && alembic current

# Node.js + Prisma (uncomment to use)
# migrate: ## Run database migrations
# 	@cd backend && npx prisma migrate deploy
#
# migrate-create: ## Create migration from schema
# 	@cd backend && npx prisma migrate dev --name $(name)
#
# migrate-reset: ## Reset database (DEV ONLY - destructive!)
# 	@cd backend && npx prisma migrate reset
#
# migrate-status: ## Check migration status
# 	@cd backend && npx prisma migrate status

# Node.js + Knex (uncomment to use)
# migrate: ## Run database migrations
# 	@cd backend && npx knex migrate:latest
#
# migrate-revert: ## Rollback last migration
# 	@cd backend && npx knex migrate:rollback
#
# migrate-create: ## Create new migration
# 	@cd backend && npx knex migrate:make $(name)
#
# migrate-status: ## Show migration status
# 	@cd backend && npx knex migrate:list

#=============================================================================
# Database Utilities
#=============================================================================

db-seed: ## Seed database with test data
	@cd backend && cargo run --bin seed
	# Python: @cd backend && python scripts/seed.py
	# Node.js: @cd backend && npm run seed

db-reset: ## Reset database (drop + migrate + seed)
	@docker-compose down -v
	@docker-compose up -d postgres
	@sleep 2
	@make migrate
	@make db-seed

#=============================================================================
# Documentation
#=============================================================================

docs-build: ## Build documentation
	@cd docs && npm run build

docs-serve: ## Serve documentation locally
	@cd docs && npm run serve

docs-screenshots: ## Generate documentation screenshots
	@cd frontend && npx playwright test --grep @screenshot

#=============================================================================
# Clean
#=============================================================================

clean: ## Clean build artifacts
	@cd backend && cargo clean
	@cd frontend && rm -rf dist node_modules
	@rm -rf target
	@echo "✓ Cleaned"

#=============================================================================
# Docker
#=============================================================================

docker-build: ## Build Docker images (optimized with BuildKit)
	@echo "Building with BuildKit optimizations..."
	@DOCKER_BUILDKIT=1 COMPOSE_DOCKER_CLI_BUILD=1 docker-compose build --parallel

docker-build-fast: ## Build with maximum caching (fastest rebuilds)
	@echo "Building with aggressive caching..."
	@DOCKER_BUILDKIT=1 docker build \
		--cache-from project-starter-api:latest \
		--build-arg BUILDKIT_INLINE_CACHE=1 \
		-t project-starter-api:latest \
		-f backend/Dockerfile \
		backend/

docker-build-no-cache: ## Build from scratch (clean build)
	@DOCKER_BUILDKIT=1 docker build --no-cache -t project-starter-api:latest -f backend/Dockerfile backend/

docker-up: ## Start services with Docker
	@docker-compose up -d

docker-down: ## Stop Docker services
	@docker-compose down

docker-logs: ## View Docker logs
	@docker-compose logs -f

docker-clean: ## Clean Docker resources
	@docker-compose down -v
	@docker system prune -f

docker-clean-cache: ## Clean Docker build cache (reclaim space)
	@docker builder prune -af

docker-stats: ## Show build cache statistics
	@docker system df
	@echo "\nBuildKit cache info:"
	@docker buildx du --verbose 2>/dev/null || echo "BuildKit cache info not available"
