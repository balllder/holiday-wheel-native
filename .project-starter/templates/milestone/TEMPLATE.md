# Milestone [N]: [Milestone Name]

## Status
- [ ] Not Started
- [ ] In Progress
- [ ] Completed

## Dependencies
- Milestone [X]: [Previous Milestone] (status required: [complete/in-progress])

## Priority
**[P0 - Critical | P1 - High | P2 - Medium | P3 - Low]**

## Description
[2-3 sentences describing what this milestone delivers and why it matters]

## Goals
- [Goal 1]
- [Goal 2]
- [Goal 3]

## User Stories

| ID | Title | Status | Priority |
|----|-------|--------|----------|
| US-1 | [User Story Title] | Not Started | P0 |
| US-2 | [User Story Title] | Not Started | P1 |
| US-3 | [User Story Title] | Not Started | P1 |

## Deliverables
- [Deliverable 1]
- [Deliverable 2]
- [Deliverable 3]

## Acceptance Criteria
- [ ] [Criteria 1]
- [ ] [Criteria 2]
- [ ] [Criteria 3]
- [ ] All user stories completed
- [ ] E2E tests passing
- [ ] Documentation updated
- [ ] Code coverage ≥80%

## Technical Stack
- **Backend:** [Language/Framework]
- **Frontend:** [Language/Framework]
- **Database:** [Database]
- **Testing:** [Testing Framework]

## API Endpoints (if applicable)

| Method | Endpoint | Description |
|--------|----------|-------------|
| GET | `/api/v1/resource` | List resources |
| POST | `/api/v1/resource` | Create resource |
| GET | `/api/v1/resource/{id}` | Get resource |
| PUT | `/api/v1/resource/{id}` | Update resource |
| DELETE | `/api/v1/resource/{id}` | Delete resource |

## Database Schema Changes (if applicable)

```sql
-- New tables
CREATE TABLE resources (
    id UUID PRIMARY KEY,
    name VARCHAR(255) NOT NULL,
    created_at TIMESTAMP DEFAULT NOW()
);

-- Migrations
ALTER TABLE users ADD COLUMN role VARCHAR(50);
```

## Testing Requirements
- [ ] Unit tests for all business logic
- [ ] Integration tests for API endpoints
- [ ] E2E tests for user workflows
- [ ] Coverage ≥80%
- [ ] Performance tests (if applicable)

## Security Considerations
- [Security consideration 1]
- [Security consideration 2]

## Performance Targets
- [Performance target 1]
- [Performance target 2]

## Monitoring & Observability

**CRITICAL: Every milestone MUST include monitoring infrastructure.**

### Health Checks
- [ ] Health check endpoints for all new services
- [ ] Liveness/readiness/startup probes configured
- [ ] Dependency health checks (database, Redis, external APIs)

### Metrics Collection
- [ ] Application metrics instrumented
  - Request count, latency, error rates
  - Resource usage (CPU, memory, connections)
- [ ] Business metrics instrumented
  - Feature-specific KPIs
  - User engagement metrics
  - Revenue/conversion metrics

### Synthetic Monitoring
- [ ] Synthetic tests for critical user workflows
- [ ] Scheduled runs (every 5-15 minutes)
- [ ] Production validation tests

### Alerting
- [ ] Error rate alerts (> 1%)
- [ ] Latency alerts (p95 > threshold)
- [ ] Success rate alerts (< 99%)
- [ ] Business metric alerts (conversion drops, etc.)
- [ ] Alert routing (Slack, PagerDuty)

### Dashboards
- [ ] Grafana dashboard for milestone features
- [ ] Canary deployment dashboard
- [ ] Business metrics visualization
- [ ] SLO/SLI tracking dashboards

### Canary Deployment
- [ ] Canary deployment configuration
- [ ] Success criteria defined
- [ ] Automatic rollback triggers
- [ ] Gradual rollout plan (5% → 25% → 50% → 100%)

### Observability Stack
```yaml
monitoring:
  health_endpoints:
    - /api/v1/[service]/health
    - /api/v1/[service]/ready

  metrics:
    prometheus:
      - [service]_requests_total
      - [service]_request_duration_seconds
      - [service]_errors_total
      - [business_metric]_total

  synthetic_tests:
    - monitoring/synthetic/[critical-flow-1].spec.ts
    - monitoring/synthetic/[critical-flow-2].spec.ts

  alerts:
    - [service]_error_rate_high
    - [service]_latency_high
    - [service]_success_rate_low
    - [business_metric]_degraded

  dashboards:
    - https://grafana.example.com/d/[milestone-id]
    - https://grafana.example.com/d/canary-[milestone-id]

  canary:
    config: k8s/canary-[service].yml
    success_criteria:
      error_rate: < 1%
      p95_latency: < 500ms
      success_rate: > 99%
```

## Documentation Requirements
- [ ] API documentation (OpenAPI)
- [ ] User guide with screenshots
- [ ] Developer guide
- [ ] CLAUDE.md updated

## Risks & Mitigation

| Risk | Mitigation |
|------|------------|
| [Risk 1] | [Mitigation strategy] |
| [Risk 2] | [Mitigation strategy] |

## Timeline Estimate
[X weeks / Y days] - *Note: Avoid rigid estimates, focus on delivering value*

## Success Metrics
- [Metric 1]
- [Metric 2]
- [Metric 3]

---

**Related Files:**
- [US-1: User Story Title](./us-1.md)
- [US-2: User Story Title](./us-2.md)
- [US-3: User Story Title](./us-3.md)
