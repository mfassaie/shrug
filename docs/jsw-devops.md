# Jira Software: DevOps and Integrations

DevOps pipeline integration commands for builds, deployments, feature flags, development information, components, operations, remote links, and security information.

Product alias: `jira-software` (or `jsw`)

All DevOps integration endpoints require an `--Authorization` header parameter (OAuth 2.0 bearer token) unless noted otherwise.

---

## builds

Submit and manage CI/CD build data. 4 operations (3 CRUD-mapped, 1 extended).

### CRUD operations

#### create

Submit build data. Body fields: `properties`, `builds*` (required), `providerMetadata`.

```
shrug jsw builds create --Authorization TOKEN
```

| Parameter | Location | Required |
|---|---|---|
| `--Authorization` | header | yes |

**Example:**

```bash
echo '{"builds":[{"pipelineId":"pipe-1","buildNumber":42,"updateSequenceNumber":1,"displayName":"Build #42","url":"https://ci.example.com/42","state":"successful","lastUpdated":"2026-03-25T10:00:00Z","issueKeys":["PROJ-1"],"references":[]}]}' | shrug jsw builds create --Authorization "Bearer TOKEN"
```

#### get

Get a build by key.

```
shrug jsw builds get --Authorization TOKEN --pipelineId ID --buildNumber NUM
```

| Parameter | Location | Required |
|---|---|---|
| `--Authorization` | header | yes |
| `--pipelineId` | path | yes |
| `--buildNumber` | path | yes |

**Example:**

```bash
shrug jsw builds get --Authorization "Bearer TOKEN" --pipelineId pipe-1 --buildNumber 42
```

#### delete

Delete a build by key.

```
shrug jsw builds delete --Authorization TOKEN --pipelineId ID --buildNumber NUM
    [--_updateSequenceNumber N]
```

| Parameter | Location | Required |
|---|---|---|
| `--Authorization` | header | yes |
| `--pipelineId` | path | yes |
| `--buildNumber` | path | yes |
| `--_updateSequenceNumber` | query | no |

### Extended operations

#### delete-builds-by-property

DELETE. Delete builds by property.

```
shrug jsw builds delete-builds-by-property --Authorization TOKEN [--_updateSequenceNumber N]
```

| Parameter | Location | Required |
|---|---|---|
| `--Authorization` | header | yes |
| `--_updateSequenceNumber` | query | no |

---

## deployments

Submit and manage deployment data. 5 operations (3 CRUD-mapped, 2 extended).

### CRUD operations

#### create

Submit deployment data. Body fields: `properties`, `deployments*` (required), `providerMetadata`.

```
shrug jsw deployments create --Authorization TOKEN
```

| Parameter | Location | Required |
|---|---|---|
| `--Authorization` | header | yes |

#### get

Get a deployment by key.

```
shrug jsw deployments get --Authorization TOKEN --pipelineId ID --environmentId ID
    --deploymentSequenceNumber NUM
```

| Parameter | Location | Required |
|---|---|---|
| `--Authorization` | header | yes |
| `--pipelineId` | path | yes |
| `--environmentId` | path | yes |
| `--deploymentSequenceNumber` | path | yes |

**Example:**

```bash
shrug jsw deployments get --Authorization "Bearer TOKEN" --pipelineId pipe-1 --environmentId prod --deploymentSequenceNumber 5
```

#### delete

Delete a deployment by key.

```
shrug jsw deployments delete --Authorization TOKEN --pipelineId ID --environmentId ID
    --deploymentSequenceNumber NUM [--_updateSequenceNumber N]
```

| Parameter | Location | Required |
|---|---|---|
| `--Authorization` | header | yes |
| `--pipelineId` | path | yes |
| `--environmentId` | path | yes |
| `--deploymentSequenceNumber` | path | yes |
| `--_updateSequenceNumber` | query | no |

### Extended operations

#### delete-deployments-by-property

DELETE. Delete deployments by property.

```
shrug jsw deployments delete-deployments-by-property --Authorization TOKEN
    [--_updateSequenceNumber N]
```

| Parameter | Location | Required |
|---|---|---|
| `--Authorization` | header | yes |
| `--_updateSequenceNumber` | query | no |

#### get-deployment-gating-status-by-key

GET. Get deployment gating status by key.

```
shrug jsw deployments get-deployment-gating-status-by-key --pipelineId ID
    --environmentId ID --deploymentSequenceNumber NUM
```

| Parameter | Location | Required |
|---|---|---|
| `--pipelineId` | path | yes |
| `--environmentId` | path | yes |
| `--deploymentSequenceNumber` | path | yes |

---

## development information

Store and manage development information (repositories, commits, branches, pull requests). 6 operations (4 CRUD-mapped, 2 extended).

### CRUD operations

#### list

Check if data exists for the supplied properties.

```
shrug jsw "development information" list --Authorization TOKEN [--_updateSequenceId ID]
```

| Parameter | Location | Required |
|---|---|---|
| `--Authorization` | header | yes |
| `--_updateSequenceId` | query | no |

#### create

Store development information. Body fields: `repositories*` (required), `preventTransitions`, `operationType`, `properties`, `providerMetadata`.

```
shrug jsw "development information" create --Authorization TOKEN
```

| Parameter | Location | Required |
|---|---|---|
| `--Authorization` | header | yes |

#### get

Get repository.

```
shrug jsw "development information" get --repositoryId ID --Authorization TOKEN
```

| Parameter | Location | Required |
|---|---|---|
| `--repositoryId` | path | yes |
| `--Authorization` | header | yes |

#### delete

Delete repository.

```
shrug jsw "development information" delete --repositoryId ID --Authorization TOKEN
    [--_updateSequenceId ID]
```

| Parameter | Location | Required |
|---|---|---|
| `--repositoryId` | path | yes |
| `--_updateSequenceId` | query | no |
| `--Authorization` | header | yes |

### Extended operations

#### delete-by-properties

DELETE. Delete development information by properties.

```
shrug jsw "development information" delete-by-properties --Authorization TOKEN
    [--_updateSequenceId ID]
```

| Parameter | Location | Required |
|---|---|---|
| `--Authorization` | header | yes |
| `--_updateSequenceId` | query | no |

#### delete-entity

DELETE. Delete development information entity.

```
shrug jsw "development information" delete-entity --repositoryId ID --entityType TYPE
    --entityId ID --Authorization TOKEN [--_updateSequenceId ID]
```

| Parameter | Location | Required |
|---|---|---|
| `--repositoryId` | path | yes |
| `--entityType` | path | yes |
| `--entityId` | path | yes |
| `--_updateSequenceId` | query | no |
| `--Authorization` | header | yes |

---

## devops components

Submit and manage DevOps components. 4 operations (3 CRUD-mapped, 1 extended).

### CRUD operations

#### create

Submit DevOps components. Body fields: `properties`, `devopsComponents*` (required), `providerMetadata`.

```
shrug jsw "devops components" create --Authorization TOKEN
```

| Parameter | Location | Required |
|---|---|---|
| `--Authorization` | header | yes |

#### get

Get a component by ID.

```
shrug jsw "devops components" get --Authorization TOKEN --componentId ID
```

| Parameter | Location | Required |
|---|---|---|
| `--Authorization` | header | yes |
| `--componentId` | path | yes |

#### delete

Delete a component by ID.

```
shrug jsw "devops components" delete --Authorization TOKEN --componentId ID
```

| Parameter | Location | Required |
|---|---|---|
| `--Authorization` | header | yes |
| `--componentId` | path | yes |

### Extended operations

#### delete-components-by-property

DELETE. Delete DevOps components by property.

```
shrug jsw "devops components" delete-components-by-property --Authorization TOKEN
```

| Parameter | Location | Required |
|---|---|---|
| `--Authorization` | header | yes |

---

## feature flags

Submit and manage feature flag data. 4 operations (3 CRUD-mapped, 1 extended).

### CRUD operations

#### create

Submit feature flag data. Body fields: `properties`, `flags*` (required), `providerMetadata`.

```
shrug jsw "feature flags" create --Authorization TOKEN
```

| Parameter | Location | Required |
|---|---|---|
| `--Authorization` | header | yes |

#### get

Get a feature flag by ID.

```
shrug jsw "feature flags" get --Authorization TOKEN --featureFlagId ID
```

| Parameter | Location | Required |
|---|---|---|
| `--Authorization` | header | yes |
| `--featureFlagId` | path | yes |

#### delete

Delete a feature flag by ID.

```
shrug jsw "feature flags" delete --Authorization TOKEN --featureFlagId ID
    [--_updateSequenceId ID]
```

| Parameter | Location | Required |
|---|---|---|
| `--Authorization` | header | yes |
| `--featureFlagId` | path | yes |
| `--_updateSequenceId` | query | no |

### Extended operations

#### delete-feature-flags-by-property

DELETE. Delete feature flags by property.

```
shrug jsw "feature flags" delete-feature-flags-by-property --Authorization TOKEN
    [--_updateSequenceId ID]
```

| Parameter | Location | Required |
|---|---|---|
| `--Authorization` | header | yes |
| `--_updateSequenceId` | query | no |

---

## operations

Manage incidents, reviews, and operations workspaces. 9 operations (4 CRUD-mapped, 5 extended).

### CRUD operations

#### list

Get all operations workspace IDs or a specific operations workspace by ID.

```
shrug jsw operations list --Authorization TOKEN
```

| Parameter | Location | Required |
|---|---|---|
| `--Authorization` | header | yes |

#### create

Submit incident or review data. Body fields: `properties`, `providerMetadata`.

```
shrug jsw operations create --Authorization TOKEN
```

| Parameter | Location | Required |
|---|---|---|
| `--Authorization` | header | yes |

#### get

Get an incident by ID.

```
shrug jsw operations get --Authorization TOKEN --incidentId ID
```

| Parameter | Location | Required |
|---|---|---|
| `--Authorization` | header | yes |
| `--incidentId` | path | yes |

#### delete

Delete an incident by ID.

```
shrug jsw operations delete --Authorization TOKEN --incidentId ID
```

| Parameter | Location | Required |
|---|---|---|
| `--Authorization` | header | yes |
| `--incidentId` | path | yes |

### Extended operations

#### submit-operations-workspaces

POST. Submit operations workspace IDs.

```
shrug jsw operations submit-operations-workspaces --Authorization TOKEN
```

| Parameter | Location | Required |
|---|---|---|
| `--Authorization` | header | yes |

#### delete-workspaces

DELETE. Delete operations workspaces by ID.

```
shrug jsw operations delete-workspaces --Authorization TOKEN
```

| Parameter | Location | Required |
|---|---|---|
| `--Authorization` | header | yes |

#### delete-entity-by-property

DELETE. Delete incidents or reviews by property.

```
shrug jsw operations delete-entity-by-property --Authorization TOKEN
```

| Parameter | Location | Required |
|---|---|---|
| `--Authorization` | header | yes |

#### get-review-by-id

GET. Get a review by ID.

```
shrug jsw operations get-review-by-id --Authorization TOKEN --reviewId ID
```

| Parameter | Location | Required |
|---|---|---|
| `--Authorization` | header | yes |
| `--reviewId` | path | yes |

#### delete-review-by-id

DELETE. Delete a review by ID.

```
shrug jsw operations delete-review-by-id --Authorization TOKEN --reviewId ID
```

| Parameter | Location | Required |
|---|---|---|
| `--Authorization` | header | yes |
| `--reviewId` | path | yes |

---

## remote links

Submit and manage remote link data. 4 operations (3 CRUD-mapped, 1 extended).

### CRUD operations

#### create

Submit remote link data. Body fields: `properties`, `remoteLinks*` (required), `providerMetadata`.

```
shrug jsw "remote links" create --Authorization TOKEN
```

| Parameter | Location | Required |
|---|---|---|
| `--Authorization` | header | yes |

#### get

Get a remote link by ID.

```
shrug jsw "remote links" get --Authorization TOKEN --remoteLinkId ID
```

| Parameter | Location | Required |
|---|---|---|
| `--Authorization` | header | yes |
| `--remoteLinkId` | path | yes |

#### delete

Delete a remote link by ID.

```
shrug jsw "remote links" delete --Authorization TOKEN --remoteLinkId ID
    [--_updateSequenceNumber N]
```

| Parameter | Location | Required |
|---|---|---|
| `--Authorization` | header | yes |
| `--remoteLinkId` | path | yes |
| `--_updateSequenceNumber` | query | no |

### Extended operations

#### delete-remote-links-by-property

DELETE. Delete remote links by property.

```
shrug jsw "remote links" delete-remote-links-by-property --Authorization TOKEN
    [--_updateSequenceNumber N] [--params PARAMS]
```

| Parameter | Location | Required |
|---|---|---|
| `--Authorization` | header | yes |
| `--_updateSequenceNumber` | query | no |
| `--params` | query | no |

---

## security information

Manage security workspaces and vulnerability data. 8 operations (4 CRUD-mapped, 4 extended).

### CRUD operations

#### list

Get linked security workspaces.

```
shrug jsw "security information" list --Authorization TOKEN
```

| Parameter | Location | Required |
|---|---|---|
| `--Authorization` | header | yes |

#### create

Submit vulnerability data. Body fields: `operationType`, `properties`, `vulnerabilities*` (required), `providerMetadata`.

```
shrug jsw "security information" create --Authorization TOKEN
```

| Parameter | Location | Required |
|---|---|---|
| `--Authorization` | header | yes |

#### get

Get a linked security workspace by ID.

```
shrug jsw "security information" get --Authorization TOKEN --workspaceId ID
```

| Parameter | Location | Required |
|---|---|---|
| `--Authorization` | header | yes |
| `--workspaceId` | path | yes |

#### delete

Delete a vulnerability by ID.

```
shrug jsw "security information" delete --Authorization TOKEN --vulnerabilityId ID
```

| Parameter | Location | Required |
|---|---|---|
| `--Authorization` | header | yes |
| `--vulnerabilityId` | path | yes |

### Extended operations

#### submit-workspaces

POST. Submit security workspaces to link.

```
shrug jsw "security information" submit-workspaces --Authorization TOKEN
```

| Parameter | Location | Required |
|---|---|---|
| `--Authorization` | header | yes |

#### delete-linked-workspaces

DELETE. Delete linked security workspaces.

```
shrug jsw "security information" delete-linked-workspaces --Authorization TOKEN
```

| Parameter | Location | Required |
|---|---|---|
| `--Authorization` | header | yes |

#### delete-vulnerabilities-by-property

DELETE. Delete vulnerabilities by property.

```
shrug jsw "security information" delete-vulnerabilities-by-property --Authorization TOKEN
```

| Parameter | Location | Required |
|---|---|---|
| `--Authorization` | header | yes |

#### get-vulnerability-by-id

GET. Get a vulnerability by ID.

```
shrug jsw "security information" get-vulnerability-by-id --Authorization TOKEN
    --vulnerabilityId ID
```

| Parameter | Location | Required |
|---|---|---|
| `--Authorization` | header | yes |
| `--vulnerabilityId` | path | yes |
