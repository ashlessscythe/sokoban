#!/bin/bash

# Get the last 30 commits
echo "Last 30 commits:"
git log -n 30 --pretty=format:"%h - %an, %ar : %s"
echo ""

# Get the detailed statistics for the last 30 commits
echo "Summary of changes for the last 30 commits:"
git log -n 30 --pretty=format:"" --numstat | awk '
{
    added += $1;
    removed += $2;
    files[$3]++;
}
END {
    print "Files changed:", length(files);
    print "Lines added:", added;
    print "Lines removed:", removed;
}
'

# Get the list of files modified in the last 30 commits
echo ""
echo "Files modified in the last 30 commits:"
git log -n 30 --pretty=format:"" --name-only | sort | uniq -c | sort -nr
