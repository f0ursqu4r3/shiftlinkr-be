#!/bin/bash

echo "🧪 Running ShiftLinkr Backend Tests..."

# Set test environment
export RUST_LOG=debug

echo ""
echo "📋 Configuration Tests:"
cargo test config_tests --quiet

echo ""
echo "🗄️  Database Tests:"
cargo test database_tests --quiet

echo ""
echo "🔐 Authentication Tests:"
cargo test auth_tests --quiet

echo ""
echo "📊 Test Summary:"
cargo test --quiet

echo ""
echo "✅ All tests completed!"
