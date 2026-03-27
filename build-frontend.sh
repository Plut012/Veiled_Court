#!/bin/bash
# Build script for Spirit Animals Go frontend

set -e

echo "Building frontend..."

# Copy frontend files to dist directory
cd frontend
rm -rf dist/*
cp -r *.html css js assets dist/

echo "Frontend built successfully!"
echo "Files are in: frontend/dist/"
