#!/bin/sh

git config core.hooksPath .githooks
chmod +x .githooks/pre-commit
echo "✓ Git hooks Instalado"