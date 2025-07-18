#!/bin/bash
# this_file: scripts/dev-workflow.sh
# Development workflow helper script for vexy_json
# Usage: ./scripts/dev-workflow.sh [command]

set -e

# Colors for output
RED='\033[0;31m'
GREEN='\033[0;32m'
YELLOW='\033[1;33m'
BLUE='\033[0;34m'
PURPLE='\033[0;35m'
NC='\033[0m' # No Color

# Script directory
SCRIPT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)"
PROJECT_ROOT="$(cd "$SCRIPT_DIR/.." && pwd)"
cd "$PROJECT_ROOT"

# Function to print usage
usage() {
    echo -e "${BLUE}🚀 Vexy JSON Development Workflow Helper${NC}"
    echo "=============================================="
    echo
    echo "Usage: $0 [command]"
    echo
    echo "Commands:"
    echo "  setup        - Set up development environment"
    echo "  check        - Quick development checks (format, lint, test)"
    echo "  format       - Format code and fix common issues"
    echo "  pre-commit   - Run pre-commit checks"
    echo "  bump-version - Bump version for testing (creates local tag)"
    echo "  clean-tags   - Clean up test tags"
    echo "  status       - Show project status and version info"
    echo "  help         - Show this help message"
    echo
}

# Function to set up development environment
setup_dev_env() {
    echo -e "${BLUE}🔧 Setting up development environment...${NC}"
    
    # Check if Rust is installed
    if ! command -v cargo &> /dev/null; then
        echo -e "${RED}❌ Rust is not installed${NC}"
        echo -e "${YELLOW}Install Rust from: https://rustup.rs/${NC}"
        exit 1
    fi
    
    # Check if git is installed
    if ! command -v git &> /dev/null; then
        echo -e "${RED}❌ Git is not installed${NC}"
        exit 1
    fi
    
    # Install additional tools
    echo -e "${BLUE}Installing development tools...${NC}"
    
    # Install cargo-watch for file watching
    if ! command -v cargo-watch &> /dev/null; then
        echo -e "${BLUE}Installing cargo-watch...${NC}"
        cargo install cargo-watch
    fi
    
    # Install cargo-expand for macro expansion
    if ! command -v cargo-expand &> /dev/null; then
        echo -e "${BLUE}Installing cargo-expand...${NC}"
        cargo install cargo-expand
    fi
    
    # Install wasm-pack for WebAssembly builds
    if ! command -v wasm-pack &> /dev/null; then
        echo -e "${BLUE}Installing wasm-pack...${NC}"
        curl https://rustwasm.github.io/wasm-pack/installer/init.sh -sSf | sh
    fi
    
    # Set up git hooks
    echo -e "${BLUE}Setting up git hooks...${NC}"
    if [ ! -f ".git/hooks/pre-commit" ]; then
        cat > .git/hooks/pre-commit << 'EOF'
#!/bin/bash
# Pre-commit hook for vexy_json
./scripts/dev-workflow.sh pre-commit
EOF
        chmod +x .git/hooks/pre-commit
        echo -e "${GREEN}✅ Pre-commit hook installed${NC}"
    fi
    
    echo -e "${GREEN}✅ Development environment setup completed${NC}"
}

# Function to run quick development checks
quick_check() {
    echo -e "${BLUE}🔍 Running quick development checks...${NC}"
    
    # Format check
    echo -e "${BLUE}Checking code format...${NC}"
    if ! cargo fmt --all -- --check; then
        echo -e "${YELLOW}⚠️  Code format issues found. Run './scripts/dev-workflow.sh format' to fix.${NC}"
    else
        echo -e "${GREEN}✅ Code format is correct${NC}"
    fi
    
    # Clippy check
    echo -e "${BLUE}Running clippy...${NC}"
    if ! cargo clippy --workspace --all-features -- -D warnings; then
        echo -e "${RED}❌ Clippy found issues${NC}"
        exit 1
    else
        echo -e "${GREEN}✅ Clippy checks passed${NC}"
    fi
    
    # Quick test
    echo -e "${BLUE}Running quick tests...${NC}"
    if ! cargo test --workspace --lib; then
        echo -e "${RED}❌ Quick tests failed${NC}"
        exit 1
    else
        echo -e "${GREEN}✅ Quick tests passed${NC}"
    fi
    
    echo -e "${GREEN}✅ All quick checks passed${NC}"
}

# Function to format code
format_code() {
    echo -e "${BLUE}🎨 Formatting code...${NC}"
    
    # Format Rust code
    cargo fmt --all
    
    # Fix common clippy issues
    cargo clippy --workspace --all-features --fix --allow-dirty --allow-staged || true
    
    echo -e "${GREEN}✅ Code formatting completed${NC}"
}

# Function to run pre-commit checks
pre_commit_check() {
    echo -e "${BLUE}🔍 Running pre-commit checks...${NC}"
    
    # Run quick checks
    quick_check
    
    # Check for common issues
    echo -e "${BLUE}Checking for common issues...${NC}"
    
    # Check for TODO/FIXME comments in staged files
    if git diff --cached --name-only | grep -E "\.(rs|md|toml)$" | xargs grep -l "TODO\|FIXME" 2>/dev/null; then
        echo -e "${YELLOW}⚠️  Found TODO/FIXME comments in staged files${NC}"
    fi
    
    # Check for debug prints
    if git diff --cached --name-only | grep -E "\.rs$" | xargs grep -l "println!\|dbg!\|eprintln!" 2>/dev/null; then
        echo -e "${YELLOW}⚠️  Found debug prints in staged files${NC}"
    fi
    
    echo -e "${GREEN}✅ Pre-commit checks completed${NC}"
}

# Function to bump version for testing
bump_version() {
    echo -e "${BLUE}🔖 Bumping version for testing...${NC}"
    
    # Get current version
    current_version=$(./scripts/get-version.sh)
    echo -e "${BLUE}Current version: $current_version${NC}"
    
    # Parse version components
    if [[ "$current_version" =~ ^([0-9]+)\.([0-9]+)\.([0-9]+)(-.*)?$ ]]; then
        major="${BASH_REMATCH[1]}"
        minor="${BASH_REMATCH[2]}"
        patch="${BASH_REMATCH[3]}"
        suffix="${BASH_REMATCH[4]}"
        
        # Increment patch version
        new_patch=$((patch + 1))
        new_version="$major.$minor.$new_patch-dev"
        
        # Create test tag
        test_tag="v$new_version"
        
        echo -e "${BLUE}Creating test tag: $test_tag${NC}"
        git tag "$test_tag"
        
        echo -e "${GREEN}✅ Test version created: $new_version${NC}"
        echo -e "${YELLOW}To clean up, run: ./scripts/dev-workflow.sh clean-tags${NC}"
    else
        echo -e "${RED}❌ Could not parse current version: $current_version${NC}"
        exit 1
    fi
}

# Function to clean up test tags
clean_tags() {
    echo -e "${BLUE}🧹 Cleaning up test tags...${NC}"
    
    # Find and remove test tags
    test_tags=$(git tag -l | grep -E "(dev|test|tmp)" || true)
    
    if [ -n "$test_tags" ]; then
        echo -e "${BLUE}Found test tags:${NC}"
        echo "$test_tags"
        
        read -p "Remove these tags? (y/N): " -n 1 -r
        echo
        if [[ $REPLY =~ ^[Yy]$ ]]; then
            echo "$test_tags" | xargs -I {} git tag -d {}
            echo -e "${GREEN}✅ Test tags cleaned up${NC}"
        else
            echo -e "${YELLOW}⚠️  Clean up cancelled${NC}"
        fi
    else
        echo -e "${GREEN}✅ No test tags found${NC}"
    fi
}

# Function to show project status
show_status() {
    echo -e "${PURPLE}📊 Vexy JSON Project Status${NC}"
    echo "=============================================="
    echo
    
    # Version information
    echo -e "${BLUE}Version Information:${NC}"
    ./build.sh version
    
    # Git status
    echo -e "${BLUE}Git Status:${NC}"
    git status --short
    
    # Recent commits
    echo -e "${BLUE}Recent Commits:${NC}"
    git log --oneline -5
    
    # Workspace status
    echo -e "${BLUE}Workspace Status:${NC}"
    cargo tree --workspace | head -10
    
    # Dependencies that need updating
    echo -e "${BLUE}Outdated Dependencies:${NC}"
    cargo outdated --root-deps-only --exit-code 0 || echo "No outdated dependencies found"
}

# Main script logic
case "${1:-help}" in
    setup)
        setup_dev_env
        ;;
    check)
        quick_check
        ;;
    format)
        format_code
        ;;
    pre-commit)
        pre_commit_check
        ;;
    bump-version)
        bump_version
        ;;
    clean-tags)
        clean_tags
        ;;
    status)
        show_status
        ;;
    help | --help | -h)
        usage
        ;;
    *)
        echo -e "${RED}❌ Unknown command: $1${NC}"
        echo
        usage
        exit 1
        ;;
esac