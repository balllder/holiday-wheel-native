# Canary Deployment Strategy

## Philosophy

**Deploy changes gradually. Catch issues with 5% of traffic, not 100%.**

Canary deployments are the final validation layer. Even with perfect tests, production can surprise you. Canaries limit blast radius and enable automatic rollback.

---

## The Canary Process

### Visual Overview

```
Stable (v1.2.3)          Canary (v1.3.0)
     100%         →           5%            Monitor 15min
                              ↓
                         Metrics OK?
                              ↓
                    Yes ↙           ↘ No
                      25%         Rollback
                       ↓
                  Monitor 15min
                       ↓
                  Metrics OK?
                       ↓
                 Yes ↙     ↘ No
                   50%    Rollback
                    ↓
              Monitor 15min
                    ↓
               Metrics OK?
                    ↓
              Yes ↙     ↘ No
                100%   Rollback
                  ↓
            Full Rollout Complete
```

### Step-by-Step Process

**Step 1: Deploy Canary (5% traffic)**
```bash
make deploy-canary VERSION=v1.3.0
# Creates canary deployment with 5% traffic split
```

**Step 2: Monitor for 15 minutes**
- Watch error rates
- Watch latency (p50, p95, p99)
- Watch success rates
- Watch resource usage
- Watch business metrics

**Step 3: Decision Point**
- **Metrics OK?** → Increase to 25%
- **Metrics degraded?** → Automatic rollback

**Step 4: Gradual Increase**
- Repeat monitoring at each stage: 25% → 50% → 100%
- Each stage: 15 minutes of monitoring
- Any degradation: automatic rollback

**Step 5: Full Rollout**
- 100% of traffic on new version
- Keep monitoring for 24 hours
- Mark deployment as successful

---

## Canary Metrics (Success Criteria)

### Critical Metrics

**1. Error Rate**
```
Threshold: < 1% errors
Alert: error_rate > 1% for 5 minutes
Action: Automatic rollback
```

**2. Latency (p95)**
```
Threshold: < 500ms
Alert: p95_latency > 500ms for 5 minutes
Action: Automatic rollback
```

**3. Latency (p99)**
```
Threshold: < 1000ms
Alert: p99_latency > 1000ms for 5 minutes
Action: Warning, consider rollback
```

**4. Success Rate**
```
Threshold: > 99%
Alert: success_rate < 99% for 5 minutes
Action: Automatic rollback
```

**5. Resource Usage (CPU)**
```
Threshold: < 80%
Alert: cpu_usage > 80% for 5 minutes
Action: Warning, monitor closely
```

**6. Resource Usage (Memory)**
```
Threshold: < 80%
Alert: memory_usage > 80% for 5 minutes
Action: Warning, monitor memory leaks
```

### Business Metrics

**7. Conversion Rate**
```
Threshold: Not degraded by > 5%
Comparison: Canary vs Stable conversion rate
Action: Rollback if degraded
```

**8. Revenue Per User**
```
Threshold: Not degraded by > 5%
Comparison: Canary vs Stable RPU
Action: Rollback if degraded
```

**9. User Engagement**
```
Threshold: Not degraded by > 10%
Metrics: Session duration, pages per session
Action: Consider rollback if degraded
```

---

## Implementation Options

### Option 1: Kubernetes Native (Manual)

**Pros:**
- Simple, no additional tools
- Full control over traffic split

**Cons:**
- Manual monitoring required
- Manual rollback required

```yaml
# k8s/deployment-stable.yml
apiVersion: apps/v1
kind: Deployment
metadata:
  name: myapp-stable
spec:
  replicas: 9  # 90% of traffic
  selector:
    matchLabels:
      app: myapp
      version: stable
  template:
    metadata:
      labels:
        app: myapp
        version: stable
    spec:
      containers:
      - name: myapp
        image: myapp:v1.2.3
---
# k8s/deployment-canary.yml
apiVersion: apps/v1
kind: Deployment
metadata:
  name: myapp-canary
spec:
  replicas: 1  # 10% of traffic
  selector:
    matchLabels:
      app: myapp
      version: canary
  template:
    metadata:
      labels:
        app: myapp
        version: canary
    spec:
      containers:
      - name: myapp
        image: myapp:v1.3.0  # New version
---
# k8s/service.yml
apiVersion: v1
kind: Service
metadata:
  name: myapp
spec:
  selector:
    app: myapp  # Matches both stable and canary
  ports:
    - port: 80
      targetPort: 8080
```

**Deploy canary:**
```bash
# Deploy canary
kubectl apply -f k8s/deployment-canary.yml

# Increase canary traffic (scale up canary, scale down stable)
kubectl scale deployment myapp-canary --replicas=3  # 30%
kubectl scale deployment myapp-stable --replicas=7  # 70%

# Promote canary (update stable image)
kubectl set image deployment/myapp-stable myapp=myapp:v1.3.0
kubectl delete deployment myapp-canary

# Rollback
kubectl delete deployment myapp-canary
```

---

### Option 2: Flagger (Automated - RECOMMENDED)

**Pros:**
- Automatic traffic shifting
- Automatic rollback on metric failures
- Integrates with Prometheus
- Production-ready

**Cons:**
- Requires Flagger installation
- Learning curve

**Installation:**
```bash
# Install Flagger
kubectl apply -k github.com/fluxcd/flagger//kustomize/linkerd

# Install Prometheus (for metrics)
helm install prometheus prometheus-community/prometheus
```

**Canary Configuration:**
```yaml
# k8s/flagger-canary.yml
apiVersion: flagger.app/v1beta1
kind: Canary
metadata:
  name: myapp
  namespace: production
spec:
  # Target deployment
  targetRef:
    apiVersion: apps/v1
    kind: Deployment
    name: myapp

  # Service configuration
  service:
    port: 80
    targetPort: 8080

  # Canary analysis
  analysis:
    # Interval between checks
    interval: 1m

    # Number of checks before rollout
    threshold: 5

    # Maximum traffic percentage for canary
    maxWeight: 50

    # Traffic increment step
    stepWeight: 10

    # Success criteria metrics
    metrics:
    - name: request-success-rate
      # Min 99% success rate
      thresholdRange:
        min: 99
      interval: 1m

    - name: request-duration
      # Max 500ms p95 latency
      thresholdRange:
        max: 500
      interval: 1m

    - name: cpu-usage
      thresholdRange:
        max: 80
      interval: 1m

    - name: memory-usage
      thresholdRange:
        max: 80
      interval: 1m

    # Webhooks for custom checks
    webhooks:
    - name: load-test
      url: http://flagger-loadtester/
      timeout: 5s
      metadata:
        type: cmd
        cmd: "hey -z 1m -q 10 -c 2 http://myapp-canary/"

    - name: acceptance-test
      url: http://flagger-loadtester/
      timeout: 30s
      metadata:
        type: bash
        cmd: "curl http://myapp-canary/health | grep healthy"

  # Automatic rollback on failure
  rollbackOnError: true

  # Skip analysis (for emergency deploys)
  skipAnalysis: false
```

**Prometheus Metrics for Flagger:**
```yaml
# monitoring/prometheus/flagger-queries.yml
- name: request-success-rate
  query: |
    sum(rate(http_requests_total{app="myapp",status!~"5.."}[1m]))
    /
    sum(rate(http_requests_total{app="myapp"}[1m]))
    * 100

- name: request-duration
  query: |
    histogram_quantile(0.95,
      sum(rate(http_request_duration_seconds_bucket{app="myapp"}[1m])) by (le)
    ) * 1000

- name: cpu-usage
  query: |
    sum(rate(container_cpu_usage_seconds_total{pod=~"myapp-.*"}[1m]))
    /
    sum(kube_pod_container_resource_limits{pod=~"myapp-.*",resource="cpu"})
    * 100

- name: memory-usage
  query: |
    sum(container_memory_usage_bytes{pod=~"myapp-.*"})
    /
    sum(kube_pod_container_resource_limits{pod=~"myapp-.*",resource="memory"})
    * 100
```

**Deploy with Flagger:**
```bash
# Deploy canary configuration
kubectl apply -f k8s/flagger-canary.yml

# Trigger canary by updating deployment
kubectl set image deployment/myapp myapp=myapp:v1.3.0

# Flagger automatically:
# 1. Creates canary deployment
# 2. Shifts traffic 10% → 20% → 30% → ... → 50%
# 3. Monitors metrics at each step
# 4. Rolls back if metrics fail
# 5. Promotes to production if successful

# Watch canary progress
watch kubectl get canary myapp

# View canary events
kubectl describe canary myapp
```

---

### Option 3: Istio / Service Mesh

**Pros:**
- Fine-grained traffic control
- Advanced routing (header-based, geo-based)
- Built-in observability

**Cons:**
- Complex setup
- Performance overhead
- Steep learning curve

```yaml
# k8s/istio-virtual-service.yml
apiVersion: networking.istio.io/v1beta1
kind: VirtualService
metadata:
  name: myapp
spec:
  hosts:
  - myapp
  http:
  - match:
    - headers:
        x-canary:
          exact: "true"
    route:
    - destination:
        host: myapp
        subset: canary
  - route:
    - destination:
        host: myapp
        subset: stable
      weight: 95
    - destination:
        host: myapp
        subset: canary
      weight: 5
---
apiVersion: networking.istio.io/v1beta1
kind: DestinationRule
metadata:
  name: myapp
spec:
  host: myapp
  subsets:
  - name: stable
    labels:
      version: stable
  - name: canary
    labels:
      version: canary
```

---

## Rollback Strategies

### Automatic Rollback

**Flagger automatically rolls back when:**
```yaml
rollback_triggers:
  - error_rate > 1%
  - p95_latency > 500ms
  - success_rate < 99%
  - cpu_usage > 90%
  - memory_usage > 90%
  - custom_metric_threshold_exceeded
```

**Rollback process:**
1. Stop canary traffic immediately (0% to canary)
2. Route 100% traffic to stable version
3. Alert team via Slack/PagerDuty
4. Keep canary pods running for debugging
5. Collect logs and metrics from canary
6. Delete canary deployment after investigation

### Manual Rollback

```bash
# Flagger
kubectl rollout undo deployment/myapp

# Kubernetes native
kubectl delete deployment myapp-canary
kubectl scale deployment myapp-stable --replicas=10

# Istio
kubectl apply -f k8s/istio-virtual-service-stable-only.yml
```

### Rollback Communication

**Slack notification template:**
```
🚨 CANARY ROLLBACK: myapp v1.3.0

Reason: Error rate exceeded 1% (actual: 2.3%)
Traffic: Reverted to 100% stable (v1.2.3)
Duration: Canary ran for 12 minutes
Impact: ~5% of users affected

Action Required:
1. Investigate error logs
2. Fix issue in v1.3.1
3. Redeploy canary with fix

Metrics: https://grafana.example.com/d/canary/myapp
Logs: https://kibana.example.com/app/discover
```

---

## Canary Best Practices

### 1. Start Small
- Start with 5% traffic, not 10%
- Smaller blast radius = safer deployments

### 2. Monitor Longer
- 15 minutes per stage minimum
- Some issues take time to surface (memory leaks)

### 3. Use Real Traffic
- Don't rely only on synthetic tests
- Real users expose real issues

### 4. Compare Apples to Apples
- Compare canary metrics to current stable metrics
- Not to historical metrics (traffic patterns change)

### 5. Have Multiple Canary Stages
- 5% → 25% → 50% → 100%
- Not 10% → 100% (too risky)

### 6. Test Rollback Process
- Practice rollbacks in staging
- Ensure team knows rollback procedure

### 7. Keep Canary Pods for Debugging
- Don't delete immediately after rollback
- Collect logs, heap dumps, metrics

### 8. Use Feature Flags for High-Risk Changes
- Canary deployment + feature flag = double safety
- Can disable feature without redeployment

---

## Monitoring During Canary

### Grafana Canary Dashboard

**Panels to include:**

1. **Traffic Split**
   - Requests to stable vs canary
   - Visual traffic percentage

2. **Error Rate Comparison**
   - Stable error rate
   - Canary error rate
   - Difference highlighted

3. **Latency Comparison (p50, p95, p99)**
   - Side-by-side stable vs canary
   - Threshold lines marked

4. **Success Rate**
   - Stable success rate
   - Canary success rate

5. **Resource Usage**
   - CPU: stable vs canary
   - Memory: stable vs canary

6. **Business Metrics**
   - Conversion rate comparison
   - Revenue per user comparison

7. **Canary Progress**
   - Current rollout stage
   - Time in current stage
   - Next action (promote or rollback)

---

## Makefile Commands

```makefile
# Canary deployment commands
.PHONY: canary-*

canary-deploy: ## Deploy canary version (5% traffic)
	@echo "Deploying canary version $(VERSION)..."
	@kubectl set image deployment/myapp myapp=myapp:$(VERSION)
	@echo "Monitor at: https://grafana.example.com/d/canary"

canary-promote: ## Promote canary to production (100% traffic)
	@echo "Promoting canary to production..."
	@kubectl rollout status deployment/myapp
	@echo "Canary promoted successfully"

canary-rollback: ## Rollback canary deployment
	@echo "Rolling back canary..."
	@kubectl rollout undo deployment/myapp
	@echo "Canary rolled back to stable version"

canary-status: ## Check canary deployment status
	@kubectl get canary myapp
	@kubectl describe canary myapp

canary-traffic: ## Show current traffic split
	@kubectl get canary myapp -o jsonpath='{.status.canaryWeight}'
	@echo "% to canary"
```

---

## Canary Deployment Checklist

### Pre-Deployment
- [ ] All tests passing (unit, integration, E2E)
- [ ] Staging deployment successful
- [ ] Synthetic tests passing on staging
- [ ] Grafana canary dashboard ready
- [ ] Alert rules configured
- [ ] Team notified of deployment
- [ ] Rollback plan documented

### During Deployment
- [ ] Canary deployed (5% traffic)
- [ ] Monitor error rates
- [ ] Monitor latency
- [ ] Monitor resource usage
- [ ] Monitor business metrics
- [ ] No alerts firing

### Each Stage (5% → 25% → 50% → 100%)
- [ ] Wait 15 minutes minimum
- [ ] Check all metrics
- [ ] Compare canary vs stable
- [ ] Verify no degradation
- [ ] Increase traffic or rollback

### Post-Deployment
- [ ] 100% traffic on new version
- [ ] Monitor for 24 hours
- [ ] Review metrics and alerts
- [ ] Document any issues
- [ ] Update runbook if needed

---

## Emergency Procedures

### Emergency Rollback

**Immediate rollback if:**
- Critical production incident
- Data corruption detected
- Security vulnerability discovered
- Cascading failures observed

**Process:**
```bash
# 1. Immediate rollback (don't wait for automatic)
make canary-rollback

# 2. Alert team
# Post to #incidents Slack channel

# 3. Investigate
# Collect logs, metrics, error traces

# 4. Fix and redeploy
# Fix issue, test thoroughly, redeploy canary
```

### Emergency Skip Canary

**For critical hotfixes only:**
```bash
# Skip canary, deploy directly to 100%
kubectl set image deployment/myapp myapp=myapp:v1.3.1-hotfix
kubectl annotate canary/myapp flagger.app/skip-analysis="true"

# Monitor VERY closely
# Have rollback plan ready
```

---

## Success Metrics

**Canary deployments are successful when:**

- ✅ 95%+ of canaries complete without rollback
- ✅ Rollbacks happen automatically, not manually
- ✅ Zero production incidents without prior canary detection
- ✅ Mean time to detection (MTTD) < 5 minutes during canary
- ✅ Mean time to rollback (MTTR) < 2 minutes
- ✅ Team trusts canary process (not bypassed)

---

## Related Documents

- [Continuous Monitoring](./CONTINUOUS_MONITORING.md) - Full monitoring strategy
- [Testing Guide](../testing/TESTING_GUIDE.md) - Testing before deployment
- [User Story Template](../../templates/user-story/TEMPLATE.md) - Monitoring requirements

---

**Remember: Canary deployments are your last line of defense. Trust the metrics, rollback early.**
