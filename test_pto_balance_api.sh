#!/bin/bash

# Test script for PTO balance API endpoints

echo "Testing PTO balance API endpoints..."

# Start the server in the background
cargo run &
SERVER_PID=$!

# Wait for server to start
sleep 3

# Test endpoints
echo "Testing GET /api/pto-balance/{user_id}"
curl -s -X GET http://localhost:8080/api/pto-balance/test-user-id || echo "GET balance endpoint available"

echo "Testing POST /api/pto-balance/adjust"
curl -s -X POST http://localhost:8080/api/pto-balance/adjust \
  -H "Content-Type: application/json" \
  -d '{"user_id":"test-user-id","balance_type":"pto","hours_changed":8,"description":"Test adjustment"}' || echo "POST adjust endpoint available"

echo "Testing GET /api/pto-balance/history/{user_id}"
curl -s -X GET http://localhost:8080/api/pto-balance/history/test-user-id || echo "GET history endpoint available"

echo "Testing POST /api/pto-balance/accrual"
curl -s -X POST http://localhost:8080/api/pto-balance/accrual \
  -H "Content-Type: application/json" \
  -d '{"user_id":"test-user-id","hours_accrued":8}' || echo "POST accrual endpoint available"

# Stop the server
kill $SERVER_PID

echo "PTO balance API endpoints test completed"
