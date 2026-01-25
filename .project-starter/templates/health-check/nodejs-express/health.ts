/**
 * Health Check Endpoints
 *
 * Provides liveness and readiness endpoints for monitoring and orchestration.
 *
 * Usage:
 * ```typescript
 * import { healthRouter } from './health';
 * app.use('/health', healthRouter);
 * ```
 */

import { Router, Request, Response } from 'express';
import { Pool } from 'pg';
// import { createClient, RedisClientType } from 'redis';  // Uncomment if using Redis

const router = Router();

/**
 * Health check response structure
 */
interface HealthResponse {
  status: string;
  service?: string;
  version?: string;
  checks?: {
    database?: string;
    cache?: string;
  };
}

/**
 * Liveness probe endpoint
 *
 * Returns 200 if the service is running.
 * Use this for Kubernetes liveness probes.
 *
 * GET /health
 *
 * Response:
 * ```json
 * {
 *   "status": "healthy",
 *   "service": "my-service",
 *   "version": "0.1.0"
 * }
 * ```
 */
router.get('/', (req: Request, res: Response) => {
  const response: HealthResponse = {
    status: 'healthy',
    service: process.env.npm_package_name || 'my-service',
    version: process.env.npm_package_version || '0.1.0',
  };

  res.status(200).json(response);
});

/**
 * Readiness probe endpoint
 *
 * Returns 200 if the service is ready to accept traffic.
 * Checks database and cache connections.
 * Use this for Kubernetes readiness probes and load balancer health checks.
 *
 * GET /health/ready
 *
 * Response (healthy):
 * ```json
 * {
 *   "status": "ready",
 *   "checks": {
 *     "database": "ok",
 *     "cache": "ok"
 *   }
 * }
 * ```
 *
 * Response (unhealthy):
 * ```json
 * {
 *   "status": "unhealthy",
 *   "checks": {
 *     "database": "error"
 *   }
 * }
 * ```
 * Returns 503 status code if any check fails.
 */
router.get('/ready', async (req: Request, res: Response) => {
  const checks: { database?: string; cache?: string } = {};
  let isHealthy = true;

  // Get database pool from app locals (set during app initialization)
  const dbPool: Pool = req.app.locals.dbPool;

  // Check database connection
  try {
    await dbPool.query('SELECT 1');
    checks.database = 'ok';
  } catch (error) {
    console.error('Database health check failed:', error);
    checks.database = 'error';
    isHealthy = false;
  }

  // Optional: Check Redis connection
  // Uncomment if using Redis
  /*
  const redisClient: RedisClientType = req.app.locals.redisClient;
  try {
    await redisClient.ping();
    checks.cache = 'ok';
  } catch (error) {
    console.error('Redis health check failed:', error);
    checks.cache = 'error';
    isHealthy = false;
  }
  */

  const response: HealthResponse = {
    status: isHealthy ? 'ready' : 'unhealthy',
    service: process.env.npm_package_name || 'my-service',
    version: process.env.npm_package_version || '0.1.0',
    checks,
  };

  const statusCode = isHealthy ? 200 : 503;
  res.status(statusCode).json(response);
});

export { router as healthRouter };

/**
 * Example setup in main app file:
 *
 * ```typescript
 * import express from 'express';
 * import { Pool } from 'pg';
 * import { healthRouter } from './health';
 *
 * const app = express();
 *
 * // Initialize database pool
 * const dbPool = new Pool({
 *   connectionString: process.env.DATABASE_URL,
 * });
 *
 * // Store in app locals for health check access
 * app.locals.dbPool = dbPool;
 *
 * // Mount health check router
 * app.use('/health', healthRouter);
 *
 * // Optional: Redis setup
 * // import { createClient } from 'redis';
 * // const redisClient = createClient({ url: process.env.REDIS_URL });
 * // await redisClient.connect();
 * // app.locals.redisClient = redisClient;
 * ```
 */
