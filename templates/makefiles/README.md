# Makefile Templates

**Default:** Rust + Axum + React (already included in root `Makefile`)

To use a different tech stack, copy the appropriate Makefile to your project root.

## Available Templates

### 1. Rust + Axum + React (DEFAULT)
**File:** `rust-axum/Makefile`

**Stack:**
- Backend: Rust + Axum
- Frontend: React 19 + TypeScript + Vite
- Database: PostgreSQL
- OpenAPI: utoipa
- Testing: cargo test + Playwright

**Already included in root `Makefile` - no action needed.**

---

### 2. Python + FastAPI + React
**File:** `python-fastapi/Makefile`

**Stack:**
- Backend: Python + FastAPI
- Frontend: React 19 + TypeScript + Vite
- Database: PostgreSQL
- OpenAPI: Built-in (Pydantic)
- Testing: pytest + Playwright

**Usage:**
```bash
cp templates/makefiles/python-fastapi/Makefile ./Makefile
make setup
```

---

### 3. Node.js + Express + React
**File:** `nodejs-express/Makefile`

**Stack:**
- Backend: Node.js + TypeScript + Express
- Frontend: React 19 + TypeScript + Vite
- Database: PostgreSQL
- OpenAPI: tsoa
- Testing: Jest + Playwright

**Usage:**
```bash
cp templates/makefiles/nodejs-express/Makefile ./Makefile
make setup
```

---

## Full-Stack Monorepo Structure

All templates assume this directory structure:

```
project-root/
├── backend/           # Backend code
│   ├── src/
│   ├── tests/
│   └── ...
├── frontend/          # Frontend code
│   ├── src/
│   ├── e2e/
│   └── ...
├── docs/              # Documentation
├── Makefile           # Copied from template
├── docker-compose.yml # Dev environment
└── README.md
```

---

## Related Documents

- [Dev-First Approach](../../docs/methodology/DEV_FIRST_APPROACH.md)
- [Testing Guide](../../docs/testing/TESTING_GUIDE.md)
- [API Testing Requirements](../../docs/testing/API_TESTING_REQUIREMENTS.md)
