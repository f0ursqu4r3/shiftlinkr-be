#!/bin/bash

# ShiftLinkr Backend Test Runner
# This script runs all backend tests with proper isolation

echo "🧪 Running ShiftLinkr Backend Tests..."
echo "======================================="

# Run core functionality tests
echo "🔐 Running Authentication Tests..."
if cargo test --test auth_tests --quiet; then
    echo "✅ Authentication tests PASSED"
else
    echo "❌ Authentication tests FAILED"
    exit 1
fi

echo ""
echo "� Running Password Reset Tests..."
if cargo test --test password_reset_tests --quiet; then
    echo "✅ Password Reset tests PASSED"
else
    echo "❌ Password Reset tests FAILED"
    exit 1
fi

echo ""
echo "🌐 Running Integration Tests..."
if cargo test --test integration_tests --quiet; then
    echo "✅ Integration tests PASSED"
else
    echo "❌ Integration tests FAILED"
    exit 1
fi

echo ""
echo "⚙️ Running Config Tests (single-threaded)..."
if cargo test --test config_tests --quiet -- --test-threads=1; then
    echo "✅ Config tests PASSED"
else
    echo "❌ Config tests FAILED"
    exit 1
fi

echo ""
echo "🎉 All Tests PASSED!"
echo "======================================="
echo "📊 Test Summary:"
echo "   - Authentication: ✅"
echo "   - Password Reset: ✅"
echo "   - Integration: ✅"
echo "   - Configuration: ✅"
echo ""
echo "🚀 Backend is ready for production!"
