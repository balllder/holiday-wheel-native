# User Story [ID]: [Title]

## Status
- [ ] Not Started
- [ ] In Progress
- [ ] Completed

## Priority
**[P0 - Critical | P1 - High | P2 - Medium | P3 - Low]**

## User Story
**As a** [type of user],
**I want** [an action or feature],
**So that** [benefit or value].

## Acceptance Criteria
- [ ] [Specific, measurable criterion 1]
- [ ] [Specific, measurable criterion 2]
- [ ] [Specific, measurable criterion 3]
- [ ] E2E test passes
- [ ] Unit tests pass
- [ ] Code coverage ≥80%
- [ ] Documentation updated

## E2E Test Scenario

**CRITICAL: Write this test BEFORE implementation (TDD approach)**

```typescript
// e2e/[feature].spec.ts
test('[User Story Title]', async ({ page }) => {
  // Given: [Preconditions]
  await page.goto('/[route]');

  // When: [User action]
  await page.fill('[selector]', '[value]');
  await page.click('[button-selector]');

  // Then: [Expected outcome]
  await expect(page).toHaveURL('/[expected-route]');
  await expect(page.locator('[result-selector]')).toBeVisible();
});
```

## Technical Approach

### Backend Changes
- [Change 1]
- [Change 2]

### Frontend Changes
- [Change 1]
- [Change 2]

### API Changes
```
[Method] /api/v1/[endpoint]
Request: { ... }
Response: { ... }
```

## Database Changes
```sql
-- Migrations
[SQL for schema changes]
```

## Dependencies
- [ ] [Dependency 1]
- [ ] [Dependency 2]

## Testing Strategy

### Unit Tests
```typescript
// backend/tests/unit/[feature].test.ts
test('[Unit test description]', () => {
  const result = functionUnderTest(input);
  expect(result).toBe(expected);
});
```

### Integration Tests
```typescript
// backend/tests/integration/[feature].test.ts
test('[Integration test description]', async () => {
  const response = await request(app)
    .post('/api/v1/endpoint')
    .send(payload);

  expect(response.status).toBe(201);
  expect(response.body).toMatchObject(expected);
});
```

### E2E Tests
See "E2E Test Scenario" section above.

## UI/UX Design (if applicable)
- [Link to mockups/wireframes]
- [Design notes]

## Security Considerations
- [ ] [Security check 1]
- [ ] [Security check 2]
- [ ] Input validation
- [ ] Authorization checks
- [ ] Data sanitization

## Performance Considerations
- [Performance consideration 1]
- [Performance consideration 2]

## Monitoring Requirements

**CRITICAL: Monitoring is part of the feature, not optional.**

### Health Checks
- [ ] Health check endpoint added for new service/feature
- [ ] Liveness probe configured (K8s)
- [ ] Readiness probe configured (K8s)
- [ ] Startup probe configured (K8s)

### Metrics
- [ ] Application metrics instrumented (request count, latency, errors)
- [ ] Business metrics instrumented (conversions, revenue, user actions)
- [ ] Resource metrics monitored (CPU, memory, connections)

### Synthetic Monitoring
- [ ] Synthetic test added (reuses E2E test for production)
- [ ] Synthetic test runs every 5-15 minutes
- [ ] Synthetic test validates critical user workflow

### Alerts
- [ ] Error rate alert configured (> 1% errors)
- [ ] Latency alert configured (p95 > threshold)
- [ ] Success rate alert configured (< 99% success)
- [ ] Business metric alerts configured (conversion rate drop, etc.)
- [ ] Alert routing configured (Slack, PagerDuty)

### Dashboards
- [ ] Grafana dashboard created for feature
- [ ] Canary dashboard configured (stable vs canary comparison)
- [ ] Business metrics visualized

### Canary Deployment
- [ ] Canary deployment configuration created
- [ ] Canary success criteria defined
- [ ] Automatic rollback triggers configured
- [ ] Gradual rollout plan defined (5% → 25% → 50% → 100%)

### Feature Flags (if applicable)
- [ ] Feature flag implemented for gradual rollout
- [ ] Rollout percentage configurable
- [ ] User whitelist support added

**Monitoring Checklist:**
```yaml
monitoring:
  health_check: /api/v1/[feature]/health
  metrics:
    - [feature]_requests_total
    - [feature]_request_duration_seconds
    - [feature]_errors_total
    - [business_metric_name]
  alerts:
    - [feature]_error_rate_high
    - [feature]_latency_high
  synthetic_test: monitoring/synthetic/[feature].spec.ts
  dashboard: https://grafana.example.com/d/[feature]
  canary_config: k8s/canary-[feature].yml
```

## Documentation Requirements
- [ ] API documentation (OpenAPI annotations)
- [ ] User guide with screenshots
- [ ] Code comments for complex logic

## Definition of Done

### Development
- [ ] Code implemented
- [ ] All tests passing (unit, integration, E2E)
- [ ] Code coverage ≥80%
- [ ] Code reviewed
- [ ] Documentation updated
- [ ] OpenAPI schema regenerated
- [ ] TypeScript client regenerated

### Monitoring (CRITICAL)
- [ ] Health check endpoints implemented
- [ ] Application metrics instrumented
- [ ] Synthetic monitoring test created
- [ ] Alerts configured and tested
- [ ] Grafana dashboard created
- [ ] Canary deployment configuration ready

### Deployment
- [ ] Deployed to staging
- [ ] Staging validation passed
- [ ] QA approved
- [ ] Code merged to main
- [ ] **Canary deployed to production (5% traffic)**
- [ ] **Canary metrics validated (15 min monitoring)**
- [ ] **Gradual rollout completed (25% → 50% → 100%)**
- [ ] **Production monitoring validated (24 hour soak)**
- [ ] **No alerts firing**
- [ ] User story marked as Complete

## Notes
[Any additional context, links, or references]

---

**Related:**
- [Milestone](./details.md)
- [Previous User Story](./us-[n-1].md)
- [Next User Story](./us-[n+1].md)
