# Makefile for Rust + Axum + React Project
# Generated from project-starter template

.PHONY: help setup dev-up dev-down test lint format clean

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
	@echo "✓ Setup complete"

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
	@docker-compose up -d
	@echo "✓ Services started"
	@echo "Backend: http://localhost:3000"
	@echo "Frontend: http://localhost:5173"
	@echo "Database: localhost:5432"

dev-down: ## Stop development environment
	@docker-compose down

dev-logs: ## Show development logs
	@docker-compose logs -f

dev-restart: dev-down dev-up ## Restart development environment

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
# Database
#=============================================================================

db-migrate: ## Run database migrations
	@cd backend && cargo run --bin migrate

db-seed: ## Seed database with test data
	@cd backend && cargo run --bin seed

db-reset: ## Reset database (drop + migrate + seed)
	@docker-compose down -v
	@docker-compose up -d postgres
	@sleep 2
	@make db-migrate
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

docker-build: ## Build Docker images
	@docker-compose build

docker-up: ## Start services with Docker
	@docker-compose up -d

docker-down: ## Stop Docker services
	@docker-compose down

docker-logs: ## View Docker logs
	@docker-compose logs -f

docker-clean: ## Clean Docker resources
	@docker-compose down -v
	@docker system prune -f
