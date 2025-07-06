#!/bin/bash

# Test script for password reset functionality

BASE_URL="http://127.0.0.1:8080"

echo "🔐 Testing Password Reset Flow..."

# Test health endpoint first
echo "1. Testing health endpoint:"
curl -s "$BASE_URL/health" | jq .
echo ""

# Create a test user first
echo "2. Creating test user for password reset:"
REGISTER_RESPONSE=$(curl -s -X POST "$BASE_URL/api/v1/auth/register" \
  -H "Content-Type: application/json" \
  -d '{
    "email": "reset-test@shiftlinkr.com",
    "password": "oldpassword123",
    "name": "Reset Test User",
    "role": "employee"
  }')

if echo "$REGISTER_RESPONSE" | jq -e '.token' > /dev/null 2>&1; then
    echo "✅ Test user created successfully"
else
    echo "ℹ️  User might already exist, continuing with test..."
fi
echo ""

# Test forgot password
echo "3. Testing forgot password endpoint:"
FORGOT_RESPONSE=$(curl -s -X POST "$BASE_URL/api/v1/auth/forgot-password" \
  -H "Content-Type: application/json" \
  -d '{
    "email": "reset-test@shiftlinkr.com"
  }')

echo "$FORGOT_RESPONSE" | jq .

if echo "$FORGOT_RESPONSE" | jq -e '.message' > /dev/null 2>&1; then
    echo "✅ Forgot password request successful"
else
    echo "❌ Forgot password request failed"
    exit 1
fi
echo ""

# Note: In a real scenario, the user would get the reset token via email
# For testing, the token will be printed to the backend console
echo "4. Testing with invalid reset token:"
INVALID_RESET_RESPONSE=$(curl -s -X POST "$BASE_URL/api/v1/auth/reset-password" \
  -H "Content-Type: application/json" \
  -d '{
    "token": "invalid-token-12345",
    "new_password": "newpassword123"
  }')

echo "$INVALID_RESET_RESPONSE" | jq .

if echo "$INVALID_RESET_RESPONSE" | jq -e '.error' > /dev/null 2>&1; then
    echo "✅ Invalid token properly rejected"
else
    echo "❌ Invalid token was not rejected"
fi
echo ""

echo "🔍 To complete the test:"
echo "1. Check the backend console for the reset token"
echo "2. Use the token in a reset-password request like this:"
echo ""
echo "curl -X POST \"$BASE_URL/api/v1/auth/reset-password\" \\"
echo "  -H \"Content-Type: application/json\" \\"
echo "  -d '{"
echo "    \"token\": \"YOUR_TOKEN_HERE\","
echo "    \"new_password\": \"newpassword123\""
echo "  }'"
echo ""
echo "3. Then test login with the new password:"
echo ""
echo "curl -X POST \"$BASE_URL/api/v1/auth/login\" \\"
echo "  -H \"Content-Type: application/json\" \\"
echo "  -d '{"
echo "    \"email\": \"reset-test@shiftlinkr.com\","
echo "    \"password\": \"newpassword123\""
echo "  }'"

echo ""
echo "🎉 Password reset flow test completed!"
