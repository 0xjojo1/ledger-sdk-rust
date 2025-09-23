#!/bin/bash

# Ledger SDK Rust - Publish Script
# This script helps with local testing and manual publishing

set -e

# Colors for output
RED='\033[0;31m'
GREEN='\033[0;32m'
YELLOW='\033[1;33m'
BLUE='\033[0;34m'
NC='\033[0m' # No Color

# Function to print colored output
print_status() {
    echo -e "${BLUE}[INFO]${NC} $1"
}

print_success() {
    echo -e "${GREEN}[SUCCESS]${NC} $1"
}

print_warning() {
    echo -e "${YELLOW}[WARNING]${NC} $1"
}

print_error() {
    echo -e "${RED}[ERROR]${NC} $1"
}

# Function to check if command exists
command_exists() {
    command -v "$1" >/dev/null 2>&1
}

# Check prerequisites
check_prerequisites() {
    print_status "Checking prerequisites..."
    
    if ! command_exists cargo; then
        print_error "cargo is not installed. Please install Rust first."
        exit 1
    fi
    
    if ! command_exists git; then
        print_error "git is not installed."
        exit 1
    fi
    
    print_success "Prerequisites check passed"
}

# Run code quality checks
run_checks() {
    print_status "Running code quality checks..."
    
    print_status "Running cargo fmt..."
    if ! cargo fmt --all -- --check; then
        print_error "Code formatting check failed. Run 'cargo fmt' to fix."
        exit 1
    fi
    
    print_status "Running cargo clippy..."
    if ! cargo clippy --all-targets --all-features -- -D warnings; then
        print_error "Clippy check failed. Please fix the warnings."
        exit 1
    fi
    
    print_status "Running tests..."
    if ! cargo test --all; then
        print_error "Tests failed. Please fix the failing tests."
        exit 1
    fi
    
    print_success "All checks passed"
}

# Update versions in all Cargo.toml files
update_versions() {
    local version="$1"
    print_status "Updating versions to $version..."
    
    find . -name "Cargo.toml" -not -path "./target/*" -not -path "./examples/*" | while read -r file; do
        sed -i.bak "s/^version = \".*\"/version = \"$version\"/" "$file"
        rm "$file.bak"
    done
    
    print_success "Versions updated to $version"
}

# Update dependencies to use crates.io versions
update_dependencies() {
    local version="$1"
    print_status "Updating dependencies to use crates.io versions..."
    
    # Update ledger-transport dependencies
    sed -i.bak "s|ledger-apdu = { path = \"../ledger-apdu\" }|ledger-apdu = \"$version\"|" ledger-transport/Cargo.toml
    rm ledger-transport/Cargo.toml.bak
    
    # Update ledger-device-base dependencies
    sed -i.bak "s|ledger-transport = { path = \"../ledger-transport\" }|ledger-transport = \"$version\"|" ledger-device-base/Cargo.toml
    rm ledger-device-base/Cargo.toml.bak
    
    # Update ledger-transport-hid dependencies
    sed -i.bak "s|ledger-transport = { path = \"../ledger-transport\" }|ledger-transport = \"$version\"|" ledger-transport-hid/Cargo.toml
    rm ledger-transport-hid/Cargo.toml.bak
    
    # Update ledger-eth-app dependencies
    sed -i.bak "s|ledger-transport = { path = \"../ledger-transport\" }|ledger-transport = \"$version\"|" ledger-eth-app/Cargo.toml
    sed -i.bak "s|ledger-device-base = { path = \"../ledger-device-base\" }|ledger-device-base = \"$version\"|" ledger-eth-app/Cargo.toml
    rm ledger-eth-app/Cargo.toml.bak
    
    print_success "Dependencies updated"
}

# Revert dependencies back to local paths
revert_dependencies() {
    print_status "Reverting dependencies to local paths..."
    
    # Revert ledger-transport dependencies
    sed -i.bak "s|ledger-apdu = \"[^\"]*\"|ledger-apdu = { path = \"../ledger-apdu\" }|" ledger-transport/Cargo.toml
    rm ledger-transport/Cargo.toml.bak
    
    # Revert ledger-device-base dependencies
    sed -i.bak "s|ledger-transport = \"[^\"]*\"|ledger-transport = { path = \"../ledger-transport\" }|" ledger-device-base/Cargo.toml
    rm ledger-device-base/Cargo.toml.bak
    
    # Revert ledger-transport-hid dependencies
    sed -i.bak "s|ledger-transport = \"[^\"]*\"|ledger-transport = { path = \"../ledger-transport\" }|" ledger-transport-hid/Cargo.toml
    rm ledger-transport-hid/Cargo.toml.bak
    
    # Revert ledger-eth-app dependencies
    sed -i.bak "s|ledger-transport = \"[^\"]*\"|ledger-transport = { path = \"../ledger-transport\" }|" ledger-eth-app/Cargo.toml
    sed -i.bak "s|ledger-device-base = \"[^\"]*\"|ledger-device-base = { path = \"../ledger-device-base\" }|" ledger-eth-app/Cargo.toml
    rm ledger-eth-app/Cargo.toml.bak
    
    print_success "Dependencies reverted to local paths"
}

# Publish crates in dependency order
publish_crates() {
    local version="$1"
    print_status "Publishing crates to crates.io..."
    
    # Publish in dependency order
    local crates=("ledger-apdu" "ledger-transport" "ledger-device-base" "ledger-transport-hid" "ledger-eth-app")
    
    for crate in "${crates[@]}"; do
        print_status "Publishing $crate..."
        if cargo publish -p "$crate" --no-verify; then
            print_success "$crate published successfully"
        else
            print_error "Failed to publish $crate"
            exit 1
        fi
    done
    
    print_success "All crates published successfully!"
}

# Main function
main() {
    local command="$1"
    local version="$2"
    
    case "$command" in
        "check")
            check_prerequisites
            run_checks
            ;;
        "prepare")
            if [ -z "$version" ]; then
                print_error "Version is required for prepare command"
                echo "Usage: $0 prepare <version>"
                exit 1
            fi
            check_prerequisites
            run_checks
            update_versions "$version"
            update_dependencies "$version"
            print_success "Project prepared for publishing version $version"
            ;;
        "publish")
            if [ -z "$version" ]; then
                print_error "Version is required for publish command"
                echo "Usage: $0 publish <version>"
                exit 1
            fi
            check_prerequisites
            update_versions "$version"
            update_dependencies "$version"
            publish_crates "$version"
            ;;
        "revert")
            revert_dependencies
            print_success "Dependencies reverted to local paths"
            ;;
        *)
            echo "Ledger SDK Rust - Publish Script"
            echo ""
            echo "Usage: $0 <command> [version]"
            echo ""
            echo "Commands:"
            echo "  check                    Run code quality checks"
            echo "  prepare <version>        Prepare project for publishing (update versions and dependencies)"
            echo "  publish <version>        Publish all crates to crates.io"
            echo "  revert                   Revert dependencies back to local paths"
            echo ""
            echo "Examples:"
            echo "  $0 check"
            echo "  $0 prepare 0.1.0"
            echo "  $0 publish 0.1.0"
            echo "  $0 revert"
            exit 1
            ;;
    esac
}

# Run main function with all arguments
main "$@"
