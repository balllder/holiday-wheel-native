# Makefile for Python + FastAPI + React Project
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
	@echo "Setting up Python + FastAPI + React project..."
	@make backend-setup
	@make frontend-setup
	@echo "✓ Setup complete"

backend-setup: ## Setup backend (Python)
	@echo "Setting up Python virtual environment..."
	@cd backend && python -m venv venv
	@cd backend && . venv/bin/activate && pip install -r requirements.txt
	@cd backend && . venv/bin/activate && pip install -r requirements-dev.txt
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
	@echo "Backend: http://localhost:8000"
	@echo "Frontend: http://localhost:5173"
	@echo "Database: localhost:5432"
	@echo "API Docs: http://localhost:8000/docs"

dev-down: ## Stop development environment
	@docker-compose down

dev-logs: ## Show development logs
	@docker-compose logs -f

dev-restart: dev-down dev-up ## Restart development environment

#=============================================================================
# Backend (Python + FastAPI)
#=============================================================================

backend-run: ## Run backend locally
	@cd backend && . venv/bin/activate && uvicorn main:app --reload --host 0.0.0.0 --port 8000

backend-test-unit: ## Run backend unit tests
	@cd backend && . venv/bin/activate && pytest tests/unit -v

backend-test-integration: ## Run backend integration tests
	@cd backend && . venv/bin/activate && pytest tests/integration -v

backend-test: backend-test-unit backend-test-integration ## Run all backend tests

backend-test-coverage: ## Run backend tests with coverage
	@cd backend && . venv/bin/activate && pytest --cov=app --cov-report=html --cov-report=term

backend-lint: ## Lint backend code
	@cd backend && . venv/bin/activate && ruff check .
	@cd backend && . venv/bin/activate && mypy .

backend-format: ## Format backend code
	@cd backend && . venv/bin/activate && black .
	@cd backend && . venv/bin/activate && ruff check --fix .

backend-format-check: ## Check backend code formatting
	@cd backend && . venv/bin/activate && black --check .
	@cd backend && . venv/bin/activate && ruff check .

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
	@cd backend && . venv/bin/activate && python -c "import json; from main import app; print(json.dumps(app.openapi()))" > ../frontend/src/api/openapi.json
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
	@cd backend && . venv/bin/activate && alembic upgrade head

db-downgrade: ## Downgrade database one revision
	@cd backend && . venv/bin/activate && alembic downgrade -1

db-seed: ## Seed database with test data
	@cd backend && . venv/bin/activate && python scripts/seed.py

db-reset: ## Reset database (drop + migrate + seed)
	@docker-compose down -v
	@docker-compose up -d postgres
	@sleep 2
	@make db-migrate
	@make db-seed

db-revision: ## Create new migration
	@cd backend && . venv/bin/activate && alembic revision --autogenerate -m "$(message)"

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
	@find backend -type d -name __pycache__ -exec rm -rf {} +
	@find backend -type f -name "*.pyc" -delete
	@cd frontend && rm -rf dist node_modules
	@rm -rf .pytest_cache
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
