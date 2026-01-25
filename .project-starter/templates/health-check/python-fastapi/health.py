"""
Health Check Endpoints

Provides liveness and readiness endpoints for monitoring and orchestration.

Usage:
    from health import router
    app.include_router(router)
"""

from fastapi import APIRouter, status, Response
from pydantic import BaseModel
from typing import Optional
import logging
from sqlalchemy import text
from sqlalchemy.ext.asyncio import AsyncSession
# from redis import asyncio as aioredis  # Uncomment if using Redis

logger = logging.getLogger(__name__)

router = APIRouter(prefix="/health", tags=["health"])


class HealthChecks(BaseModel):
    """Detailed health checks for dependencies"""
    database: str
    cache: Optional[str] = None


class HealthResponse(BaseModel):
    """Health check response structure"""
    status: str
    service: Optional[str] = None
    version: Optional[str] = None
    checks: Optional[HealthChecks] = None


@router.get("", response_model=HealthResponse, status_code=status.HTTP_200_OK)
async def health_check():
    """
    Liveness probe endpoint

    Returns 200 if the service is running.
    Use this for Kubernetes liveness probes.

    **Response:**
    ```json
    {
        "status": "healthy",
        "service": "my-service",
        "version": "0.1.0"
    }
    ```
    """
    return HealthResponse(
        status="healthy",
        service="my-service",  # Replace with actual service name
        version="0.1.0"  # Replace with actual version
    )


@router.get("/ready", response_model=HealthResponse)
async def ready_check(
    db: AsyncSession,
    # redis: aioredis.Redis,  # Uncomment if using Redis
    response: Response
):
    """
    Readiness probe endpoint

    Returns 200 if the service is ready to accept traffic.
    Checks database and cache connections.
    Use this for Kubernetes readiness probes and load balancer health checks.

    **Response (healthy):**
    ```json
    {
        "status": "ready",
        "checks": {
            "database": "ok",
            "cache": "ok"
        }
    }
    ```

    **Response (unhealthy):**
    ```json
    {
        "status": "unhealthy",
        "checks": {
            "database": "error"
        }
    }
    ```
    Returns 503 status code if any check fails.
    """
    checks = {}
    is_healthy = True

    # Check database connection
    try:
        await db.execute(text("SELECT 1"))
        checks["database"] = "ok"
    except Exception as e:
        logger.error(f"Database health check failed: {e}")
        checks["database"] = "error"
        is_healthy = False

    # Optional: Check Redis connection
    # Uncomment if using Redis
    """
    try:
        await redis.ping()
        checks["cache"] = "ok"
    except Exception as e:
        logger.error(f"Redis health check failed: {e}")
        checks["cache"] = "error"
        is_healthy = False
    """

    if not is_healthy:
        response.status_code = status.HTTP_503_SERVICE_UNAVAILABLE
        return HealthResponse(
            status="unhealthy",
            service="my-service",
            version="0.1.0",
            checks=HealthChecks(**checks)
        )

    return HealthResponse(
        status="ready",
        service="my-service",
        version="0.1.0",
        checks=HealthChecks(**checks)
    )


# Example dependency injection setup
"""
from fastapi import Depends
from sqlalchemy.ext.asyncio import AsyncSession
from database import get_db

@router.get("/ready")
async def ready_check(
    db: AsyncSession = Depends(get_db),
    response: Response = None
):
    # Implementation above
    pass
"""
