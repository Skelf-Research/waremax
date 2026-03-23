#!/bin/bash
# publish-crates.sh - Publish waremax crates to crates.io with checks and delays
#
# Usage:
#   ./scripts/publish-crates.sh [--dry-run] [--skip-tests] [--skip-checks]
#
# Options:
#   --dry-run      Run cargo publish with --dry-run flag (no actual publishing)
#   --skip-tests   Skip running tests before publishing
#   --skip-checks  Skip pre-flight checks (git status, formatting, clippy)

set -e

# Colors for output
RED='\033[0;31m'
GREEN='\033[0;32m'
YELLOW='\033[1;33m'
BLUE='\033[0;34m'
NC='\033[0m' # No Color

# Configuration
DELAY_BETWEEN_CRATES=30  # seconds to wait between publishing crates (crates.io rate limiting)
DELAY_AFTER_FAILURE=60   # seconds to wait before retry after rate limit error

# Parse arguments
DRY_RUN=false
SKIP_TESTS=false
SKIP_CHECKS=false

for arg in "$@"; do
    case $arg in
        --dry-run)
            DRY_RUN=true
            shift
            ;;
        --skip-tests)
            SKIP_TESTS=true
            shift
            ;;
        --skip-checks)
            SKIP_CHECKS=true
            shift
            ;;
        *)
            echo -e "${RED}Unknown option: $arg${NC}"
            echo "Usage: $0 [--dry-run] [--skip-tests] [--skip-checks]"
            exit 1
            ;;
    esac
done

# Crates in dependency order (leaf dependencies first)
# This order respects the internal dependency graph
CRATES=(
    "waremax-core"
    "waremax-map"
    "waremax-storage"
    "waremax-entities"
    "waremax-config"
    "waremax-policies"
    "waremax-metrics"
    "waremax-analysis"
    "waremax-sim"
    "waremax-testing"
    "waremax-ui"
)

log_info() {
    echo -e "${BLUE}[INFO]${NC} $1"
}

log_success() {
    echo -e "${GREEN}[SUCCESS]${NC} $1"
}

log_warning() {
    echo -e "${YELLOW}[WARNING]${NC} $1"
}

log_error() {
    echo -e "${RED}[ERROR]${NC} $1"
}

# Change to project root
cd "$(dirname "$0")/.."
PROJECT_ROOT=$(pwd)

log_info "Publishing waremax crates from: $PROJECT_ROOT"
log_info "Dry run: $DRY_RUN"

# ============================================================================
# Pre-flight Checks
# ============================================================================

if [ "$SKIP_CHECKS" = false ]; then
    log_info "Running pre-flight checks..."

    # Check for uncommitted changes
    log_info "Checking git status..."
    if ! git diff --quiet || ! git diff --cached --quiet; then
        log_error "You have uncommitted changes. Please commit or stash them before publishing."
        git status --short
        exit 1
    fi
    log_success "Git working directory is clean"

    # Check for untracked files (warning only)
    UNTRACKED=$(git ls-files --others --exclude-standard)
    if [ -n "$UNTRACKED" ]; then
        log_warning "You have untracked files:"
        echo "$UNTRACKED"
    fi

    # Check formatting
    log_info "Checking code formatting..."
    if ! cargo fmt --all -- --check; then
        log_error "Code is not formatted. Run 'cargo fmt --all' first."
        exit 1
    fi
    log_success "Code formatting check passed"

    # Run clippy
    log_info "Running clippy..."
    if ! cargo clippy --all-targets --all-features -- -D warnings; then
        log_error "Clippy found issues. Please fix them before publishing."
        exit 1
    fi
    log_success "Clippy check passed"
fi

# ============================================================================
# Run Tests
# ============================================================================

if [ "$SKIP_TESTS" = false ]; then
    log_info "Running tests..."
    if ! cargo test --all; then
        log_error "Tests failed. Please fix them before publishing."
        exit 1
    fi
    log_success "All tests passed"
fi

# ============================================================================
# Verify Crate Metadata
# ============================================================================

log_info "Verifying crate metadata..."

verify_crate_metadata() {
    local crate_name=$1
    local crate_path="crates/${crate_name}"
    local cargo_toml="$crate_path/Cargo.toml"

    if [ ! -f "$cargo_toml" ]; then
        log_error "Cargo.toml not found for $crate_name at $cargo_toml"
        return 1
    fi

    # Check for description
    if ! grep -q "^description" "$cargo_toml"; then
        log_warning "$crate_name is missing 'description' field"
    fi

    return 0
}

for crate in "${CRATES[@]}"; do
    verify_crate_metadata "$crate"
done

log_success "Metadata verification complete"

# ============================================================================
# Build all crates first
# ============================================================================

log_info "Building all crates..."
if ! cargo build --all --release; then
    log_error "Build failed. Please fix build errors before publishing."
    exit 1
fi
log_success "Build completed successfully"

# ============================================================================
# Publish Crates
# ============================================================================

publish_crate() {
    local crate_name=$1
    local crate_path="crates/${crate_name}"
    local attempt=1
    local max_attempts=3

    log_info "Publishing $crate_name..."

    PUBLISH_ARGS=""
    if [ "$DRY_RUN" = true ]; then
        PUBLISH_ARGS="--dry-run"
    fi

    while [ $attempt -le $max_attempts ]; do
        if cargo publish -p "$crate_name" $PUBLISH_ARGS --allow-dirty 2>&1 | tee /tmp/publish_output.txt; then
            log_success "Successfully published $crate_name"
            return 0
        fi

        # Check if it's a rate limit error
        if grep -q "rate limit" /tmp/publish_output.txt 2>/dev/null; then
            log_warning "Rate limited. Waiting ${DELAY_AFTER_FAILURE}s before retry (attempt $attempt/$max_attempts)..."
            sleep $DELAY_AFTER_FAILURE
            attempt=$((attempt + 1))
        # Check if already published
        elif grep -q "already uploaded" /tmp/publish_output.txt 2>/dev/null; then
            log_warning "$crate_name version already published, skipping..."
            return 0
        else
            log_error "Failed to publish $crate_name"
            cat /tmp/publish_output.txt
            return 1
        fi
    done

    log_error "Failed to publish $crate_name after $max_attempts attempts"
    return 1
}

log_info "Starting crate publication..."
log_info "Will publish ${#CRATES[@]} crates with ${DELAY_BETWEEN_CRATES}s delay between each"

PUBLISHED_COUNT=0
FAILED_CRATES=()

for i in "${!CRATES[@]}"; do
    crate="${CRATES[$i]}"

    echo ""
    log_info "=========================================="
    log_info "Publishing crate $((i + 1))/${#CRATES[@]}: $crate"
    log_info "=========================================="

    if publish_crate "$crate"; then
        PUBLISHED_COUNT=$((PUBLISHED_COUNT + 1))
    else
        FAILED_CRATES+=("$crate")
        log_error "Stopping publication due to failure"
        break
    fi

    # Wait between crates (except for the last one)
    if [ $((i + 1)) -lt ${#CRATES[@]} ]; then
        if [ "$DRY_RUN" = false ]; then
            log_info "Waiting ${DELAY_BETWEEN_CRATES}s before next crate (crates.io rate limiting)..."
            sleep $DELAY_BETWEEN_CRATES
        else
            log_info "(Dry run: skipping delay)"
        fi
    fi
done

# ============================================================================
# Summary
# ============================================================================

echo ""
log_info "=========================================="
log_info "Publication Summary"
log_info "=========================================="

if [ ${#FAILED_CRATES[@]} -eq 0 ]; then
    log_success "All $PUBLISHED_COUNT crates published successfully!"

    if [ "$DRY_RUN" = false ]; then
        echo ""
        log_info "Next steps:"
        log_info "  1. Create a git tag: git tag -a v\$(cargo metadata --format-version 1 | jq -r '.packages[] | select(.name==\"waremax-core\") | .version') -m 'Release'"
        log_info "  2. Push the tag: git push origin --tags"
    fi
else
    log_error "Publication failed!"
    log_error "Successfully published: $PUBLISHED_COUNT crates"
    log_error "Failed crates: ${FAILED_CRATES[*]}"
    exit 1
fi
