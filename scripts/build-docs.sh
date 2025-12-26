#!/bin/sh
# Create docs directory structure
mkdir -p docs
cd docs

# Copy the artifact files into the docs directory
# conf.py, index.rst, installation.rst, etc.

# Install Sphinx and theme
#pip install sphinx sphinx-rtd-theme

# Build HTML documentation
make html

# View the docs
#open _build/html/index.html  # macOS
#xdg-open _build/html/index.html  # Linux
#start _build/html/index.html  # Windows
