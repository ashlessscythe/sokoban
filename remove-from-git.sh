#!/bin/bash

# Check if a file path is provided
if [ "$#" -ne 1 ]; then
    echo "Usage: $0 <file_path>"
    exit 1
fi

FILE_PATH="$1"

# Function to ask for confirmation
confirm() {
    read -r -p "${1:-Are you sure? [y/N]} " response
    case "$response" in
        [yY][eE][sS]|[yY]) 
            true
            ;;
        *)
            false
            ;;
    esac
}

# Check if file exists
if [ ! -f "$FILE_PATH" ]; then
    echo "File $FILE_PATH does not exist."
    exit 1
fi

# Add to .gitignore
echo "Adding $FILE_PATH to .gitignore..."
echo "$FILE_PATH" >> .gitignore
git add .gitignore
git commit -m "Add $FILE_PATH to .gitignore"

# Remove file from git tracking
echo "Removing $FILE_PATH from git tracking..."
git rm --cached "$FILE_PATH"
git commit -m "Remove $FILE_PATH from repository"

# Push changes
if confirm "Do you want to push these changes to the remote repository? [y/N]"; then
    git push
fi

# Remove file from git history
if confirm "WARNING: This will rewrite git history. Are you sure you want to proceed? [y/N]"; then
    git filter-branch --force --index-filter \
    "git rm --cached --ignore-unmatch $FILE_PATH" \
    --prune-empty --tag-name-filter cat -- --all

    # Force push
    if confirm "Do you want to force push these changes to the remote repository? This will overwrite history. [y/N]"; then
        git push origin --force --all
        git push origin --force --tags
    fi

    # Clean up
    if confirm "Do you want to expire the reflog and run garbage collection? [y/N]"; then
        git reflog expire --expire=now --all
        git gc --prune=now --aggressive
    fi
fi

echo "Process completed. Please review the changes and ensure everything is correct."
echo "Remember to recreate your $FILE_PATH locally if needed, and consider rotating any exposed secrets."