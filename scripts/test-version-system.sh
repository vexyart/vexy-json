#!/bin/bash
# this_file: scripts/test-version-system.sh
# Test script to validate the git-tag-based version management system
# Usage: ./scripts/test-version-system.sh

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

# Test results
TESTS_PASSED=0
TESTS_FAILED=0

# Function to run a test
run_test() {
    local test_name="$1"
    local test_command="$2"
    
    echo -e "${BLUE}🧪 Testing: $test_name${NC}"
    
    if eval "$test_command" > /dev/null 2>&1; then
        echo -e "${GREEN}✅ PASSED: $test_name${NC}"
        ((TESTS_PASSED++))
        return 0
    else
        echo -e "${RED}❌ FAILED: $test_name${NC}"
        ((TESTS_FAILED++))
        return 1
    fi
}

# Function to run a test with output
run_test_with_output() {
    local test_name="$1"
    local test_command="$2"
    local expected_pattern="$3"
    
    echo -e "${BLUE}🧪 Testing: $test_name${NC}"
    
    local output
    if output=$(eval "$test_command" 2>&1); then
        if [[ -n "$expected_pattern" ]] && [[ ! "$output" =~ $expected_pattern ]]; then
            echo -e "${RED}❌ FAILED: $test_name (output doesn't match expected pattern)${NC}"
            echo -e "${YELLOW}Expected pattern: $expected_pattern${NC}"
            echo -e "${YELLOW}Actual output: $output${NC}"
            ((TESTS_FAILED++))
            return 1
        else
            echo -e "${GREEN}✅ PASSED: $test_name${NC}"
            echo -e "${BLUE}Output: $output${NC}"
            ((TESTS_PASSED++))
            return 0
        fi
    else
        echo -e "${RED}❌ FAILED: $test_name (command failed)${NC}"
        echo -e "${YELLOW}Error output: $output${NC}"
        ((TESTS_FAILED++))
        return 1
    fi
}

# Function to create a temporary git tag for testing
create_test_tag() {
    local tag="$1"
    echo -e "${BLUE}Creating test tag: $tag${NC}"
    git tag "$tag" > /dev/null 2>&1 || true
}

# Function to remove a test tag
remove_test_tag() {
    local tag="$1"
    echo -e "${BLUE}Removing test tag: $tag${NC}"
    git tag -d "$tag" > /dev/null 2>&1 || true
}

# Function to get current git commit
get_current_commit() {
    git rev-parse HEAD
}

# Function to test version detection
test_version_detection() {
    echo -e "${PURPLE}=== Testing Version Detection ===${NC}"
    
    # Test 1: Check if get-version.sh exists and is executable
    run_test "get-version.sh exists and is executable" "[ -x ./scripts/get-version.sh ]"
    
    # Test 2: Check if get-version.sh returns a version
    run_test_with_output "get-version.sh returns a version" "./scripts/get-version.sh" "[0-9]+\.[0-9]+\.[0-9]+"
    
    # Test 3: Create a test tag and check if it's detected
    local test_tag="v99.99.99-test"
    create_test_tag "$test_tag"
    
    run_test_with_output "Version detection with test tag" "./scripts/get-version.sh" "99\.99\.99-test"
    
    # Clean up test tag
    remove_test_tag "$test_tag"
}

# Function to test version updating
test_version_updating() {
    echo -e "${PURPLE}=== Testing Version Updating ===${NC}"
    
    # Test 1: Check if update-versions.sh exists and is executable
    run_test "update-versions.sh exists and is executable" "[ -x ./scripts/update-versions.sh ]"
    
    # Test 2: Backup current state
    local backup_dir="/tmp/vexy-json-version-test-backup"
    mkdir -p "$backup_dir"
    
    # Copy important files to backup
    cp Cargo.toml "$backup_dir/"
    find crates -name "Cargo.toml" -exec cp {} "$backup_dir/" \; 2>/dev/null || true
    
    # Test 3: Set a test version and run update
    local test_version="99.99.99-test"
    export RELEASE_VERSION="$test_version"
    
    run_test "Version update script execution" "./scripts/update-versions.sh"
    
    # Test 4: Check if version was updated in main Cargo.toml
    run_test_with_output "Main Cargo.toml updated" "grep 'version = \"$test_version\"' Cargo.toml | head -1" "version = \"$test_version\""
    
    # Test 5: Check if version was updated in core crate
    if [ -f "crates/core/Cargo.toml" ]; then
        run_test_with_output "Core crate version updated" "grep 'version = \"$test_version\"' crates/core/Cargo.toml | head -1" "version = \"$test_version\""
    fi
    
    # Restore backup
    cp "$backup_dir/Cargo.toml" .
    find crates -name "Cargo.toml" -exec cp "$backup_dir/Cargo.toml" {} \; 2>/dev/null || true
    rm -rf "$backup_dir"
    
    unset RELEASE_VERSION
}

# Function to test build system
test_build_system() {
    echo -e "${PURPLE}=== Testing Build System ===${NC}"
    
    # Test 1: Check if build.sh exists and is executable
    run_test "build.sh exists and is executable" "[ -x ./build.sh ]"
    
    # Test 2: Test validate command
    run_test "Build system validation" "./build.sh validate"
    
    # Test 3: Test version command
    run_test_with_output "Build system version command" "./build.sh version" "[0-9]+\.[0-9]+\.[0-9]+"
    
    # Test 4: Test that help command works
    run_test "Build system help command" "./build.sh help"
}

# Function to test release system
test_release_system() {
    echo -e "${PURPLE}=== Testing Release System ===${NC}"
    
    # Test 1: Check if release.sh exists and is executable
    run_test "release.sh exists and is executable" "[ -x ./scripts/release.sh ]"
    
    # Test 2: Test dry-run mode
    run_test "Release script dry-run mode" "./scripts/release.sh --version 99.99.99-test --dry-run"
}

# Function to test git integration
test_git_integration() {
    echo -e "${PURPLE}=== Testing Git Integration ===${NC}"
    
    # Test 1: Check if we're in a git repository
    run_test "In git repository" "git rev-parse --git-dir"
    
    # Test 2: Check if we can create and read tags
    local test_tag="v99.99.98-integration-test"
    create_test_tag "$test_tag"
    
    run_test "Can create and read git tags" "git tag -l '$test_tag' | grep -q '$test_tag'"
    
    # Test 3: Check if version detection works with our test tag
    run_test_with_output "Version detection with integration test tag" "./scripts/get-version.sh" "99\.99\.98-integration-test"
    
    # Clean up test tag
    remove_test_tag "$test_tag"
}

# Function to test cargo workspace
test_cargo_workspace() {
    echo -e "${PURPLE}=== Testing Cargo Workspace ===${NC}"
    
    # Test 1: Check if Cargo.toml exists
    run_test "Cargo.toml exists" "[ -f Cargo.toml ]"
    
    # Test 2: Check if workspace is valid
    run_test "Cargo workspace is valid" "cargo metadata --format-version 1 > /dev/null"
    
    # Test 3: Check if we can build the workspace
    run_test "Cargo workspace builds" "cargo check --workspace"
}

# Function to test version consistency
test_version_consistency() {
    echo -e "${PURPLE}=== Testing Version Consistency ===${NC}"
    
    # Get current version
    local current_version
    current_version=$(./scripts/get-version.sh)
    
    echo -e "${BLUE}Current version: $current_version${NC}"
    
    # Test 1: Check if version is consistent across key files
    if [ -f "Cargo.toml" ]; then
        local cargo_version
        cargo_version=$(grep -E '^version = ".*"' Cargo.toml | head -1 | sed 's/version = "\(.*\)"/\1/')
        
        if [[ "$cargo_version" =~ ^[0-9]+\.[0-9]+\.[0-9]+ ]]; then
            echo -e "${GREEN}✅ PASSED: Main Cargo.toml has valid version format: $cargo_version${NC}"
            ((TESTS_PASSED++))
        else
            echo -e "${RED}❌ FAILED: Main Cargo.toml has invalid version format: $cargo_version${NC}"
            ((TESTS_FAILED++))
        fi
    fi
    
    # Test 2: Check if all crate versions are reasonable
    local crate_count=0
    local version_mismatch=0
    
    for crate_toml in crates/*/Cargo.toml; do
        if [ -f "$crate_toml" ]; then
            ((crate_count++))
            local crate_version
            crate_version=$(grep -E '^version = ".*"' "$crate_toml" | head -1 | sed 's/version = "\(.*\)"/\1/')
            
            if [[ ! "$crate_version" =~ ^[0-9]+\.[0-9]+\.[0-9]+ ]]; then
                ((version_mismatch++))
                echo -e "${YELLOW}⚠️  $crate_toml has invalid version format: $crate_version${NC}"
            fi
        fi
    done
    
    if [ "$version_mismatch" -eq 0 ]; then
        echo -e "${GREEN}✅ PASSED: All $crate_count crates have valid version formats${NC}"
        ((TESTS_PASSED++))
    else
        echo -e "${RED}❌ FAILED: $version_mismatch out of $crate_count crates have invalid version formats${NC}"
        ((TESTS_FAILED++))
    fi
}

# Main test function
main() {
    echo -e "${PURPLE}🧪 Vexy JSON Version Management System Test Suite${NC}"
    echo "=============================================="
    echo
    
    # Run all test categories
    test_version_detection
    echo
    test_version_updating
    echo
    test_build_system
    echo
    test_release_system
    echo
    test_git_integration
    echo
    test_cargo_workspace
    echo
    test_version_consistency
    echo
    
    # Print summary
    echo -e "${PURPLE}=== Test Summary ===${NC}"
    echo -e "${GREEN}✅ Tests Passed: $TESTS_PASSED${NC}"
    echo -e "${RED}❌ Tests Failed: $TESTS_FAILED${NC}"
    echo -e "${BLUE}Total Tests: $((TESTS_PASSED + TESTS_FAILED))${NC}"
    echo
    
    if [ "$TESTS_FAILED" -eq 0 ]; then
        echo -e "${GREEN}🎉 All tests passed! Version management system is working correctly.${NC}"
        exit 0
    else
        echo -e "${RED}❌ Some tests failed. Please review the output above.${NC}"
        exit 1
    fi
}

# Run the test suite
main "$@"