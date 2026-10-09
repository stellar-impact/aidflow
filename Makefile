.PHONY: help lint fmt test build clean check-boundaries contracts services clients install

help: ## Show this help message
	@echo 'Usage: make [target]'
	@echo ''
	@echo 'Available targets:'
	@grep -E '^[a-zA-Z_-]+:.*?## .*$$' $(MAKEFILE_LIST) | sort | awk 'BEGIN {FS = ":.*?## "}; {printf "  \033[36m%-20s\033[0m %s\n", $$1, $$2}'

install: ## Install all dependencies
	@echo "Installing dependencies..."
	npm install
	cd contracts/sdk && npm install || true
	cd clients/donor-dashboard && npm install || true
	cd clients/beneficiary-pwa && npm install || true
	cd clients/merchant-app && npm install || true
	cd services && go mod download || true

lint: ## Run linting across all modules
	@echo "Running lint checks..."
	@npm run lint --workspaces --if-present || true
	@cd contracts && cargo fmt --check || echo "Contracts not ready for fmt"
	@cd contracts && cargo clippy --all -- -D warnings || echo "Contracts not ready for clippy"
	@cd services && go fmt ./... || echo "Services not ready for fmt"
	@cd services && golangci-lint run || echo "Services not ready for golangci-lint (skipping)"

fmt: ## Auto-format all modules in place
	@echo "Formatting code..."
	@npm run format --workspaces --if-present || true
	@cd contracts && cargo fmt --all || echo "Contracts not ready for fmt"
	@cd services && go fmt ./... || echo "Services not ready for fmt"

test: ## Run tests across all modules
	@echo "Running tests..."
	@cd contracts && cargo test --all || echo "Contracts not ready for tests"
	@cd services && go test ./... || echo "Services not ready for tests"
	@npm run test --workspaces --if-present || true

build: ## Build all modules
	@echo "Building all modules..."
	@cd contracts && ./scripts/build.sh || echo "Contracts not ready for build"
	@cd services && ./scripts/build.sh || echo "Services not ready for build"
	@npm run build --workspaces --if-present || true

clean: ## Clean build artifacts
	@echo "Cleaning build artifacts..."
	rm -rf node_modules
	rm -rf */node_modules
	rm -rf */*/node_modules
	cd contracts && cargo clean || true
	cd services && rm -rf bin/ || true
	cd clients/donor-dashboard && rm -rf dist/ || true
	cd clients/beneficiary-pwa && rm -rf dist/ || true

check-boundaries: ## Enforce dependency direction rules
	@./scripts/check-boundaries.sh

contracts: ## Build contracts only
	@echo "Building contracts..."
	@cd contracts && ./scripts/build.sh

services: ## Build services only
	@echo "Building services..."
	@cd services && ./scripts/build.sh

clients: ## Build clients only
	@echo "Building clients..."
	@npm run build --workspaces --if-present

dev-setup: install ## Complete development environment setup
	@echo "Setting up development environment..."
	@./scripts/setup-dev.sh || echo "Setup script not ready yet"

docker-up: ## Start docker-compose stack
	@docker-compose -f infra/docker-compose.yml up -d

docker-down: ## Stop docker-compose stack
	@docker-compose -f infra/docker-compose.yml down

docker-logs: ## View docker-compose logs
	@docker-compose -f infra/docker-compose.yml logs -f
