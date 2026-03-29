# GitHub Project Coordination Skill

This skill enables you to set up and coordinate GitHub repositories for team projects, including creating repos, inviting collaborators, setting up branch protection, and managing issues/PRs.

## When to Use

Use this skill when users ask you to:
- Create a new GitHub repository for a project
- Add collaborators to a repository
- Set up branch protection rules
- Create issues for task tracking
- Help with PR workflow and reviews
- Monitor CI/CD status

## Prerequisites

The following environment variables must be set:
- `GH_TOKEN` or `GITHUB_TOKEN`: GitHub personal access token with repo scope
- `GITHUB_USERNAME`: Your GitHub username

The `gh` CLI is used for all GitHub operations.

## Capabilities

### 1. Create a Repository

```bash
# Create a private repository
gh repo create project-name --private --description "Project description"

# Create a public repository
gh repo create project-name --public --description "Project description"

# Create with a template
gh repo create project-name --template owner/template-repo --private

# Create and clone locally
gh repo create project-name --private --clone
```

### 2. Invite Collaborators

```bash
# Add a collaborator with write access
gh api repos/OWNER/REPO/collaborators/USERNAME -X PUT -f permission=push

# Add with admin access
gh api repos/OWNER/REPO/collaborators/USERNAME -X PUT -f permission=admin

# Add with read-only access
gh api repos/OWNER/REPO/collaborators/USERNAME -X PUT -f permission=pull

# List current collaborators
gh api repos/OWNER/REPO/collaborators --jq '.[].login'
```

### 3. Set Up Branch Protection

```bash
# Require PR reviews before merging to main
gh api repos/OWNER/REPO/branches/main/protection -X PUT \
  -F required_pull_request_reviews='{"required_approving_review_count":1}' \
  -F enforce_admins=false \
  -F required_status_checks=null \
  -F restrictions=null

# More strict protection (require reviews + status checks)
gh api repos/OWNER/REPO/branches/main/protection -X PUT \
  -F required_pull_request_reviews='{"required_approving_review_count":1,"dismiss_stale_reviews":true}' \
  -F required_status_checks='{"strict":true,"contexts":["ci/test"]}' \
  -F enforce_admins=true \
  -F restrictions=null
```

### 4. Create Issues for Task Tracking

```bash
# Create an issue
gh issue create --repo OWNER/REPO \
  --title "Implement feature X" \
  --body "Description of what needs to be done" \
  --assignee alice,bob \
  --label "enhancement"

# List open issues
gh issue list --repo OWNER/REPO

# Assign an issue
gh issue edit ISSUE_NUMBER --repo OWNER/REPO --add-assignee USERNAME
```

### 5. Create and Manage Pull Requests

```bash
# Create a PR
gh pr create --repo OWNER/REPO \
  --title "Add feature X" \
  --body "Closes #123" \
  --base main \
  --head feature-branch

# Request reviewers
gh pr edit PR_NUMBER --repo OWNER/REPO --add-reviewer alice,bob

# Check PR status
gh pr status --repo OWNER/REPO

# List PRs awaiting review
gh pr list --repo OWNER/REPO --state open --json number,title,author,reviewDecision
```

### 6. Monitor CI/CD

```bash
# Check workflow runs
gh run list --repo OWNER/REPO --limit 5

# View a specific run
gh run view RUN_ID --repo OWNER/REPO

# Watch a run in progress
gh run watch RUN_ID --repo OWNER/REPO
```

## Example Workflows

### Setting Up a New Project Repository

When a user says: "Create a GitHub repo for our CS 101 project and add Alice, Bob, and Carol"

1. **Create the repository:**
```bash
gh repo create cs101-final-project --private --description "CS 101 Final Project - Group 5"
```

2. **Invite collaborators:**
```bash
gh api repos/YOUR_USERNAME/cs101-final-project/collaborators/alice --method PUT -f permission=push
gh api repos/YOUR_USERNAME/cs101-final-project/collaborators/bob --method PUT -f permission=push
gh api repos/YOUR_USERNAME/cs101-final-project/collaborators/carol --method PUT -f permission=push
```

3. **Set up branch protection:**
```bash
gh api repos/YOUR_USERNAME/cs101-final-project/branches/main/protection -X PUT \
  -F required_pull_request_reviews='{"required_approving_review_count":1}' \
  -F enforce_admins=false
```

4. **Create initial issues:**
```bash
gh issue create --repo YOUR_USERNAME/cs101-final-project \
  --title "Set up project structure" \
  --body "Create initial folder structure and README"
```

5. **Notify the team:**
Send a message via Discord/Slack with:
- Repository URL
- Clone instructions
- Branch protection rules explanation
- How to contribute (create branch, make PR)

### Helping with PR Workflow

When a user needs help with a PR:

1. **Check current status:**
```bash
gh pr view PR_NUMBER --repo OWNER/REPO
```

2. **If CI is failing:**
```bash
gh run view RUN_ID --repo OWNER/REPO --log-failed
```

3. **Request reviews:**
```bash
gh pr edit PR_NUMBER --repo OWNER/REPO --add-reviewer teammate
```

4. **Remind reviewers:**
Send a Discord/Slack message to the reviewer

### Responding to GitHub Issues

When you receive a GitHub notification about an issue:

1. **Read the issue:**
```bash
gh issue view ISSUE_NUMBER --repo OWNER/REPO
```

2. **Create a branch and fix:**
```bash
git checkout -b fix/issue-NUMBER
# Make changes
git add .
git commit -m "Fix issue #NUMBER: description"
git push -u origin fix/issue-NUMBER
```

3. **Create a PR:**
```bash
gh pr create --title "Fix: issue description" --body "Closes #NUMBER"
```

## Best Practices

1. **Branch naming**: Use prefixes like `feature/`, `fix/`, `docs/` for clarity
2. **PR descriptions**: Always reference related issues with "Closes #X" or "Relates to #X"
3. **Review requests**: Request reviews from team members after creating a PR
4. **Small PRs**: Encourage breaking large changes into smaller, reviewable PRs
5. **Commit messages**: Use clear, descriptive commit messages

## Common Issues

### Permission Denied
- Check if the token has the correct scopes
- Verify you're using the correct repository owner/name

### Collaborator Invitation Pending
- Collaborators must accept the invitation before they can push
- You can check pending invitations:
```bash
gh api repos/OWNER/REPO/invitations
```

### Branch Protection Conflicts
- Users cannot push directly to protected branches
- Must use pull requests for changes
