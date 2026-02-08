#!/bin/bash
# Push kdbg to GitHub

echo "=== kdbg - Ready to push to GitHub ==="
echo ""
echo "Repository is initialized and committed locally."
echo ""
echo "To push to GitHub, run these commands:"
echo ""
echo "1. Create the repo on GitHub:"
echo "   Go to: https://github.com/new"
echo "   Repository name: kdbg"
echo "   Description: Kubernetes pod debugger - Fast kubectl wrapper with fuzzy matching"
echo "   Public repo"
echo "   Don't initialize with README (we already have one)"
echo ""
echo "2. Then push:"
echo "   cd ~/kdbg"
echo "   git remote set-url origin git@github.com:JackFurton/kdbg.git"
echo "   git push -u origin main"
echo ""
echo "Or if you have gh CLI installed:"
echo "   cd ~/kdbg"
echo "   gh repo create kdbg --public --source=. --description='Kubernetes pod debugger - Fast kubectl wrapper' --push"
echo ""
echo "Current status:"
cd ~/kdbg
git log --oneline -1
echo ""
echo "Files ready to push:"
git ls-files
