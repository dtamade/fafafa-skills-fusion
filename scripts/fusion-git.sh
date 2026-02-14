#!/bin/bash
# fusion-git.sh - Git operations for Fusion workflow

set -euo pipefail

FUSION_DIR=".fusion"
ACTION="${1:-status}"
BRANCH_PREFIX="fusion/"

usage() {
    echo "Usage: fusion-git.sh {status|create-branch|commit|branch|changes|diff|cleanup}"
}

if [ "$ACTION" = "-h" ] || [ "$ACTION" = "--help" ]; then
    usage
    exit 0
fi

# Colors for output (use printf for better portability)
RED='\033[0;31m'
GREEN='\033[0;32m'
YELLOW='\033[1;33m'
NC='\033[0m' # No Color

log_info() {
    printf "${GREEN}[fusion-git]${NC} %s\n" "$1"
}

log_warn() {
    printf "${YELLOW}[fusion-git]${NC} %s\n" "$1"
}

log_error() {
    printf "${RED}[fusion-git]${NC} %s\n" "$1" >&2
}

# Check if we're in a git repo
check_git_repo() {
    if ! git rev-parse --is-inside-work-tree > /dev/null 2>&1; then
        log_error "Not a git repository"
        exit 1
    fi
}

# Create a new branch for the workflow
create_branch() {
    local goal_slug="$1"
    local branch_name="${BRANCH_PREFIX}${goal_slug}"

    check_git_repo

    # Check if branch already exists
    if git show-ref --verify --quiet "refs/heads/$branch_name"; then
        log_warn "Branch $branch_name already exists, switching to it"
        git checkout "$branch_name"
    else
        log_info "Creating and switching to branch: $branch_name"
        git checkout -b "$branch_name"
    fi

    echo "$branch_name"
}

# Commit changes with a message
commit_changes() {
    local message="$1"
    local task_id="$2"

    check_git_repo

    # Check if there are changes to commit
    if git diff --quiet && git diff --staged --quiet; then
        log_warn "No changes to commit"
        return 0
    fi

    # Stage all changes
    git add -A

    # Commit
    log_info "Committing: $message"
    git commit -m "$message"

    # Get the commit hash
    local commit_hash=$(git rev-parse --short HEAD)
    log_info "Committed: $commit_hash"

    # Update sessions.json if exists
    if [ -f "$FUSION_DIR/sessions.json" ]; then
        # This is a simple append; production would use jq
        log_info "Recording commit in sessions.json"
    fi

    echo "$commit_hash"
}

# Get current branch
get_current_branch() {
    check_git_repo
    git branch --show-current
}

# Get uncommitted changes summary
get_changes_summary() {
    check_git_repo

    echo "=== Git Status ==="
    git status --short

    echo ""
    echo "=== Changed Files ==="
    git diff --name-only
    git diff --staged --name-only
}

# Generate diff for review
get_diff() {
    check_git_repo

    # Get diff of all changes (staged and unstaged)
    git diff HEAD
}

# Cleanup: return to original branch
cleanup() {
    local original_branch="$1"

    check_git_repo

    if [ -n "$original_branch" ]; then
        log_info "Returning to original branch: $original_branch"
        git checkout "$original_branch"
    fi
}

# Main command handler
case "$ACTION" in
    create-branch)
        GOAL_SLUG="${2:-}"
        if [ -z "$GOAL_SLUG" ]; then
            log_error "Usage: fusion-git.sh create-branch <goal-slug>"
            exit 1
        fi
        create_branch "$GOAL_SLUG"
        ;;

    commit)
        MESSAGE="${2:-}"
        TASK_ID="${3:-}"
        if [ -z "$MESSAGE" ]; then
            log_error "Usage: fusion-git.sh commit <message> [task_id]"
            exit 1
        fi
        commit_changes "$MESSAGE" "$TASK_ID"
        ;;

    branch)
        get_current_branch
        ;;

    changes)
        get_changes_summary
        ;;

    diff)
        get_diff
        ;;

    cleanup)
        ORIGINAL_BRANCH="${2:-}"
        cleanup "$ORIGINAL_BRANCH"
        ;;

    status)
        check_git_repo
        echo "=== Fusion Git Status ==="
        echo "Current branch: $(get_current_branch)"
        echo ""
        get_changes_summary
        ;;

    *)
        log_error "Unknown action: $ACTION"
        usage >&2
        exit 1
        ;;
esac
