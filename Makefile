# Makefile for Holiday Wheel Native (Monorepo)
# React Native phone/TV apps + Rust backend

.PHONY: help setup dev test lint format clean

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

setup: ## Initial project setup (install all deps)
	@echo "Setting up Holiday Wheel Native monorepo..."
	@npm install
	@make backend-setup
	@echo "✓ Setup complete"

setup-hooks: ## Install Git hooks
	@echo "Installing Git hooks..."
	@mkdir -p .git/hooks
	@echo '#!/bin/sh\nmake pre-commit' > .git/hooks/pre-commit
	@chmod +x .git/hooks/pre-commit
	@echo "✓ Pre-commit hook installed"

backend-setup: ## Setup backend (Rust)
	@echo "Building Rust backend..."
	@cd apps/backend-rust && cargo build
	@echo "✓ Backend setup complete"

#=============================================================================
# Development
#=============================================================================

dev: ## Start all development servers (backend + phone Metro)
	@echo "Starting development environment..."
	@make -j2 dev-backend dev-phone

dev-backend: ## Start backend server
	@cd apps/backend-rust && cargo run

dev-phone: ## Start phone Metro bundler
	@cd apps/phone && npm start

dev-tv: ## Start TV Metro bundler
	@cd apps/tv && npm start

#=============================================================================
# Backend (Rust + Axum)
#=============================================================================

backend-build: ## Build backend
	@cd apps/backend-rust && cargo build

backend-build-release: ## Build backend (release mode)
	@cd apps/backend-rust && cargo build --release

backend-run: ## Run backend locally
	@cd apps/backend-rust && cargo run

backend-watch: ## Run backend with auto-reload (requires cargo-watch)
	@cd apps/backend-rust && cargo watch -x run

backend-test: ## Run backend tests
	@cd apps/backend-rust && cargo test

backend-lint: ## Lint backend code (clippy)
	@cd apps/backend-rust && cargo clippy -- -D warnings

backend-format: ## Format backend code
	@cd apps/backend-rust && cargo fmt

backend-format-check: ## Check backend code formatting
	@cd apps/backend-rust && cargo fmt -- --check

#=============================================================================
# Phone App (React Native)
#=============================================================================

phone-start: ## Start phone Metro bundler
	@cd apps/phone && npm start

phone-android: ## Run phone app on Android
	@cd apps/phone && npm run android

phone-ios: ## Run phone app on iOS
	@cd apps/phone && npm run ios

phone-test: ## Run phone app tests
	@cd apps/phone && npm test

phone-lint: ## Lint phone app
	@cd apps/phone && npm run lint

#=============================================================================
# TV App (React Native)
#=============================================================================

tv-start: ## Start TV Metro bundler
	@cd apps/tv && npm start

tv-android: ## Run TV app on Android TV
	@cd apps/tv && npm run android

tv-ios: ## Run TV app on Apple TV
	@cd apps/tv && npm run ios

tv-test: ## Run TV app tests
	@cd apps/tv && npm test

tv-lint: ## Lint TV app
	@cd apps/tv && npm run lint

#=============================================================================
# Shared Package
#=============================================================================

shared-build: ## Build shared package
	@cd packages/shared && npm run build

shared-test: ## Test shared package
	@cd packages/shared && npm test

shared-lint: ## Lint shared package
	@cd packages/shared && npm run lint

#=============================================================================
# Quality (All)
#=============================================================================

test: backend-test phone-test tv-test ## Run all tests
	@echo "✓ All tests passed"

lint: backend-lint ## Lint all code (backend + frontend)
	@npm run lint
	@echo "✓ All linting passed"

format: backend-format ## Format all code
	@npm run format
	@echo "✓ All code formatted"

format-check: backend-format-check ## Check code formatting
	@npm run format -- --check
	@echo "✓ Format check passed"

check-types: ## TypeScript type checking
	@npm run check-types
	@echo "✓ Type check passed"

#=============================================================================
# CI/CD
#=============================================================================

ci-quick: format-check lint check-types ## Fast CI checks (lint + type-check)
	@echo "✓ Quick CI checks passed"

ci: ci-quick test ## Full CI validation
	@echo "✓ Full CI passed"

pre-commit: ci-quick ## Pre-commit validation
	@echo "✓ Pre-commit checks passed"

#=============================================================================
# Build
#=============================================================================

build: ## Build all packages
	@npm run build
	@echo "✓ All packages built"

build-backend: backend-build ## Build backend only

build-shared: shared-build ## Build shared package only

#=============================================================================
# Health Checks
#=============================================================================

health: ## Check backend service health
	@echo "Checking backend health..."
	@curl -sf http://localhost:5000/health || (echo "✗ Backend unhealthy" && exit 1)
	@echo "✓ Backend is healthy"

#=============================================================================
# Clean
#=============================================================================

clean: ## Clean build artifacts
	@cd apps/backend-rust && cargo clean
	@rm -rf apps/phone/node_modules
	@rm -rf apps/tv/node_modules
	@rm -rf packages/shared/node_modules
	@rm -rf node_modules
	@echo "✓ Cleaned"

clean-backend: ## Clean backend only
	@cd apps/backend-rust && cargo clean
	@echo "✓ Backend cleaned"

#=============================================================================
# Utilities
#=============================================================================

turbo: ## Run Turbo command (usage: make turbo cmd="build")
	@npx turbo $(cmd)

pods-install: ## Install iOS CocoaPods
	@cd apps/phone/ios && pod install
	@cd apps/tv/ios && pod install
	@echo "✓ Pods installed"
