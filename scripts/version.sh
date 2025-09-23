#!/bin/bash

# Ledger SDK Rust - Version Management Script
# This script helps with version management and release preparation

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

# Get current version from Cargo.toml
get_current_version() {
    local crate="$1"
    if [ -z "$crate" ]; then
        # Get version from the first Cargo.toml found
        grep -r "^version = " --include="Cargo.toml" . | head -1 | sed 's/.*version = "\(.*\)"/\1/'
    else
        grep "^version = " "$crate/Cargo.toml" | sed 's/.*version = "\(.*\)"/\1/'
    fi
}

# Update version in a specific Cargo.toml
update_version() {
    local file="$1"
    local version="$2"
    sed -i.bak "s/^version = \".*\"/version = \"$version\"/" "$file"
    rm "$file.bak"
}

# Bump version
bump_version() {
    local current_version="$1"
    local bump_type="$2"
    
    # Split version into parts
    IFS='.' read -ra VERSION_PARTS <<< "$current_version"
    local major="${VERSION_PARTS[0]}"
    local minor="${VERSION_PARTS[1]}"
    local patch="${VERSION_PARTS[2]}"
    
    case "$bump_type" in
        "major")
            major=$((major + 1))
            minor=0
            patch=0
            ;;
        "minor")
            minor=$((minor + 1))
            patch=0
            ;;
        "patch")
            patch=$((patch + 1))
            ;;
        *)
            print_error "Invalid bump type: $bump_type. Use major, minor, or patch."
            exit 1
            ;;
    esac
    
    echo "$major.$minor.$patch"
}

# Create git tag
create_tag() {
    local version="$1"
    local tag="v$version"
    
    print_status "Creating git tag: $tag"
    
    if git tag -l | grep -q "^$tag$"; then
        print_warning "Tag $tag already exists"
        read -p "Do you want to delete and recreate it? (y/N): " -n 1 -r
        echo
        if [[ $REPLY =~ ^[Yy]$ ]]; then
            git tag -d "$tag"
            git push origin ":refs/tags/$tag" 2>/dev/null || true
        else
            print_error "Tag creation cancelled"
            exit 1
        fi
    fi
    
    git tag -a "$tag" -m "Release version $version"
    print_success "Tag $tag created"
}

# Push tag to remote
push_tag() {
    local version="$1"
    local tag="v$version"
    
    print_status "Pushing tag to remote..."
    git push origin "$tag"
    print_success "Tag pushed to remote"
}

# Create release commit
create_release_commit() {
    local version="$1"
    local commit_message="Release version $version"
    
    print_status "Creating release commit..."
    git add .
    git commit -m "$commit_message"
    print_success "Release commit created"
}

# Main function
main() {
    local command="$1"
    local arg="$2"
    
    case "$command" in
        "current")
            local version=$(get_current_version)
            echo "Current version: $version"
            ;;
        "bump")
            if [ -z "$arg" ]; then
                print_error "Bump type is required (major, minor, patch)"
                echo "Usage: $0 bump <major|minor|patch>"
                exit 1
            fi
            
            local current_version=$(get_current_version)
            local new_version=$(bump_version "$current_version" "$arg")
            
            print_status "Bumping version from $current_version to $new_version"
            
            # Update all Cargo.toml files
            find . -name "Cargo.toml" -not -path "./target/*" -not -path "./examples/*" | while read -r file; do
                update_version "$file" "$new_version"
            done
            
            print_success "Version bumped to $new_version"
            ;;
        "tag")
            if [ -z "$arg" ]; then
                local version=$(get_current_version)
            else
                local version="$arg"
            fi
            
            create_tag "$version"
            ;;
        "release")
            if [ -z "$arg" ]; then
                print_error "Version is required for release command"
                echo "Usage: $0 release <version>"
                exit 1
            fi
            
            local version="$arg"
            
            # Update all Cargo.toml files
            find . -name "Cargo.toml" -not -path "./target/*" -not -path "./examples/*" | while read -r file; do
                update_version "$file" "$version"
            done
            
            create_release_commit "$version"
            create_tag "$version"
            push_tag "$version"
            
            print_success "Release $version created and pushed"
            ;;
        "full-release")
            if [ -z "$arg" ]; then
                print_error "Bump type is required (major, minor, patch)"
                echo "Usage: $0 full-release <major|minor|patch>"
                exit 1
            fi
            
            local current_version=$(get_current_version)
            local new_version=$(bump_version "$current_version" "$arg")
            
            print_status "Creating full release: $current_version -> $new_version"
            
            # Update all Cargo.toml files
            find . -name "Cargo.toml" -not -path "./target/*" -not -path "./examples/*" | while read -r file; do
                update_version "$file" "$new_version"
            done
            
            create_release_commit "$new_version"
            create_tag "$new_version"
            push_tag "$new_version"
            
            print_success "Full release $new_version created and pushed"
            print_status "GitHub Actions will now automatically publish to crates.io"
            ;;
        *)
            echo "Ledger SDK Rust - Version Management Script"
            echo ""
            echo "Usage: $0 <command> [argument]"
            echo ""
            echo "Commands:"
            echo "  current                     Show current version"
            echo "  bump <major|minor|patch>    Bump version in all Cargo.toml files"
            echo "  tag [version]               Create git tag (uses current version if not specified)"
            echo "  release <version>           Create release commit and tag"
            echo "  full-release <major|minor|patch>  Bump version, commit, tag, and push (triggers CI)"
            echo ""
            echo "Examples:"
            echo "  $0 current"
            echo "  $0 bump patch"
            echo "  $0 tag 0.1.0"
            echo "  $0 release 0.1.0"
            echo "  $0 full-release patch"
            exit 1
            ;;
    esac
}

# Run main function with all arguments
main "$@"
