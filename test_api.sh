#!/bin/bash

# Test script for ShiftLinkr API

BASE_URL="http://127.0.0.1:8080"

echo "🧪 Testing ShiftLinkr API..."

# Test health endpoint
echo "1. Testing health endpoint:"
curl -s "$BASE_URL/health" | jq .
echo ""

# Test user registration
echo "2. Testing user registration:"
REGISTER_RESPONSE=$(curl -s -X POST "$BASE_URL/api/v1/auth/register" \
  -H "Content-Type: application/json" \
  -d '{
    "email": "test@example.com",
    "password": "password123",
    "name": "Test User",
    "role": "employee"
  }')

echo "$REGISTER_RESPONSE" | jq .
echo ""

# Extract token from registration response
TOKEN=$(echo "$REGISTER_RESPONSE" | jq -r '.token')

# Test login
echo "3. Testing user login:"
LOGIN_RESPONSE=$(curl -s -X POST "$BASE_URL/api/v1/auth/login" \
  -H "Content-Type: application/json" \
  -d '{
    "email": "test@example.com",
    "password": "password123"
  }')

echo "$LOGIN_RESPONSE" | jq .
echo ""

# Test me endpoint with token
echo "4. Testing protected /me endpoint:"
curl -s -X GET "$BASE_URL/api/v1/auth/me" \
  -H "Authorization: Bearer $TOKEN" | jq .
echo ""

echo "✅ API tests completed!"
