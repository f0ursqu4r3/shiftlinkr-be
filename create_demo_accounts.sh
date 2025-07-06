#!/bin/bash

# Demo account creation script for ShiftLinkr API

BASE_URL="http://127.0.0.1:8080"

echo "🚀 Creating demo accounts for ShiftLinkr..."

# Test health endpoint first
echo "1. Testing health endpoint:"
curl -s "$BASE_URL/health" | jq .
echo ""

# Function to create and test an account
create_account() {
    local role=$1
    local email=$2
    local password=$3
    local name=$4
    
    echo "Creating ${role} account..."
    
    REGISTER_RESPONSE=$(curl -s -X POST "$BASE_URL/api/v1/auth/register" \
      -H "Content-Type: application/json" \
      -d "{\"email\":\"${email}\",\"password\":\"${password}\",\"name\":\"${name}\",\"role\":\"${role}\"}")
    
    if echo "$REGISTER_RESPONSE" | jq -e '.token' > /dev/null 2>&1; then
        echo "✅ ${role} account created successfully"
        echo "   Email: ${email}"
        echo "   Password: ${password}"
        echo "   Name: ${name}"
    else
        echo "❌ Failed to create ${role} account"
        echo "   Error: $(echo "$REGISTER_RESPONSE" | jq -r '.error // "Unknown error"')"
    fi
    echo ""
}

# Function to test login
test_login() {
    local role=$1
    local email=$2
    local password=$3
    
    echo "Testing login for ${role} (${email})..."
    
    LOGIN_RESPONSE=$(curl -s -X POST "$BASE_URL/api/v1/auth/login" \
      -H "Content-Type: application/json" \
      -d "{\"email\":\"${email}\",\"password\":\"${password}\"}")
    
    if echo "$LOGIN_RESPONSE" | jq -e '.token' > /dev/null 2>&1; then
        echo "✅ ${role} login successful"
        
        # Test the /me endpoint
        TOKEN=$(echo "$LOGIN_RESPONSE" | jq -r '.token')
        ME_RESPONSE=$(curl -s -X GET "$BASE_URL/api/v1/auth/me" \
          -H "Authorization: Bearer $TOKEN")
        
        if echo "$ME_RESPONSE" | jq -e '.user' > /dev/null 2>&1; then
            echo "   User info: $(echo "$ME_RESPONSE" | jq -r '.user.name') ($(echo "$ME_RESPONSE" | jq -r '.user.role'))"
        else
            echo "   ❌ Failed to get user info"
        fi
    else
        echo "❌ ${role} login failed"
        echo "   Error: $(echo "$LOGIN_RESPONSE" | jq -r '.error // "Unknown error"')"
    fi
    echo ""
}

# Create demo accounts
echo "2. Creating demo accounts:"
create_account "admin" "admin@shiftlinkr.com" "admin123" "Admin User"
create_account "manager" "manager@shiftlinkr.com" "manager123" "Manager Smith"
create_account "employee" "employee1@shiftlinkr.com" "employee123" "John Doe"
create_account "employee" "employee2@shiftlinkr.com" "employee123" "Jane Smith"
create_account "employee" "employee3@shiftlinkr.com" "employee123" "Mike Johnson"

echo "3. Testing login for each account:"
test_login "admin" "admin@shiftlinkr.com" "admin123"
test_login "manager" "manager@shiftlinkr.com" "manager123"
test_login "employee1" "employee1@shiftlinkr.com" "employee123"
test_login "employee2" "employee2@shiftlinkr.com" "employee123"
test_login "employee3" "employee3@shiftlinkr.com" "employee123"

echo "🎉 Demo account creation completed!"
echo ""
echo "📝 Available Demo Accounts:"
echo "┌──────────────────────────────────────────────────────────────────────┐"
echo "│ Role      │ Email                      │ Password     │ Name         │"
echo "├──────────────────────────────────────────────────────────────────────┤"
echo "│ Admin     │ admin@shiftlinkr.com       │ admin123     │ Admin User   │"
echo "│ Manager   │ manager@shiftlinkr.com     │ manager123   │ Manager Smith│"
echo "│ Employee  │ employee1@shiftlinkr.com   │ employee123  │ John Doe     │"
echo "│ Employee  │ employee2@shiftlinkr.com   │ employee123  │ Jane Smith   │"
echo "│ Employee  │ employee3@shiftlinkr.com   │ employee123  │ Mike Johnson │"
echo "└──────────────────────────────────────────────────────────────────────┘"
echo ""
echo "🌐 You can now use these accounts to test the frontend at:"
echo "   http://localhost:3000/auth/login"
echo ""
echo "🔐 Password Reset Testing:"
echo "To test the password reset flow, run:"
echo "   ./test_password_reset.sh"
echo ""
echo "Or manually test with any existing account:"
echo "   curl -X POST http://127.0.0.1:8080/api/v1/auth/forgot-password \\"
echo "     -H 'Content-Type: application/json' \\"
echo "     -d '{\"email\":\"admin@shiftlinkr.com\"}'"
