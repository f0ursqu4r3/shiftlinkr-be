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
    local email=$1
    local password=$2
    local name=$3
    
    echo "Creating account for ${name}..."
    
    REGISTER_RESPONSE=$(curl -s -X POST "$BASE_URL/api/v1/auth/register" \
      -H "Content-Type: application/json" \
      -d "{\"email\":\"${email}\",\"password\":\"${password}\",\"name\":\"${name}\"}")
    
    if echo "$REGISTER_RESPONSE" | jq -e '.token' > /dev/null 2>&1; then
        echo "✅ Account created successfully"
        echo "   Email: ${email}"
        echo "   Password: ${password}"
        echo "   Name: ${name}"
        
        # Extract user ID for later use
        USER_ID=$(echo "$REGISTER_RESPONSE" | jq -r '.user.id')
        echo "   User ID: ${USER_ID}"
        
        return 0
    else
        echo "❌ Failed to create account"
        echo "   Error: $(echo "$REGISTER_RESPONSE" | jq -r '.error // "Unknown error"')"
    fi
    echo ""
}

# Function to create a demo company
create_demo_company() {
    local admin_email=$1
    local admin_password=$2
    
    echo "Creating demo company..."
    
    # First, login as the admin to get a token
    LOGIN_RESPONSE=$(curl -s -X POST "$BASE_URL/api/v1/auth/login" \
      -H "Content-Type: application/json" \
      -d "{\"email\":\"${admin_email}\",\"password\":\"${admin_password}\"}")
    
    if echo "$LOGIN_RESPONSE" | jq -e '.token' > /dev/null 2>&1; then
        TOKEN=$(echo "$LOGIN_RESPONSE" | jq -r '.token')
        
        # Create the company
        COMPANY_RESPONSE=$(curl -s -X POST "$BASE_URL/api/v1/companies" \
          -H "Content-Type: application/json" \
          -H "Authorization: Bearer $TOKEN" \
          -d '{
            "name": "ShiftLinkr Demo Company",
            "description": "A demo company for testing ShiftLinkr features",
            "website": "https://demo.shiftlinkr.com",
            "phone": "+1-555-DEMO-123",
            "email": "contact@demo.shiftlinkr.com",
            "address": "123 Demo Street, Demo City, DC 12345",
            "timezone": "America/New_York"
          }')
        
        if echo "$COMPANY_RESPONSE" | jq -e '.id' > /dev/null 2>&1; then
            COMPANY_ID=$(echo "$COMPANY_RESPONSE" | jq -r '.id')
            echo "✅ Demo company created successfully"
            echo "   Company ID: ${COMPANY_ID}"
            echo "   Company Name: ShiftLinkr Demo Company"
            return 0
        else
            echo "❌ Failed to create demo company"
            echo "   Error: $(echo "$COMPANY_RESPONSE" | jq -r '.error // "Unknown error"')"
            return 1
        fi
    else
        echo "❌ Failed to login as admin to create company"
        echo "   Error: $(echo "$LOGIN_RESPONSE" | jq -r '.error // "Unknown error"')"
        return 1
    fi
    echo ""
}

# Function to add user to company
add_user_to_company() {
    local company_id=$1
    local user_id=$2
    local role=$3
    local admin_token=$4
    local user_name=$5
    
    echo "Adding ${user_name} to company as ${role}..."
    
    ADD_RESPONSE=$(curl -s -X POST "$BASE_URL/api/v1/companies/${company_id}/employees" \
      -H "Content-Type: application/json" \
      -H "Authorization: Bearer $admin_token" \
      -d "{
        \"user_id\": \"${user_id}\",
        \"role\": \"${role}\",
        \"is_primary\": true
      }")
    
    if [ $? -eq 0 ]; then
        echo "✅ ${user_name} added to company as ${role}"
    else
        echo "❌ Failed to add ${user_name} to company"
        echo "   Error: $(echo "$ADD_RESPONSE" | jq -r '.error // "Unknown error"')"
    fi
    echo ""
}

# Function to test login
test_login() {
    local email=$1
    local password=$2
    local name=$3
    
    echo "Testing login for ${name} (${email})..."
    
    LOGIN_RESPONSE=$(curl -s -X POST "$BASE_URL/api/v1/auth/login" \
      -H "Content-Type: application/json" \
      -d "{\"email\":\"${email}\",\"password\":\"${password}\"}")
    
    if echo "$LOGIN_RESPONSE" | jq -e '.token' > /dev/null 2>&1; then
        echo "✅ Login successful for ${name}"
        
        # Test the /me endpoint
        TOKEN=$(echo "$LOGIN_RESPONSE" | jq -r '.token')
        ME_RESPONSE=$(curl -s -X GET "$BASE_URL/api/v1/auth/me" \
          -H "Authorization: Bearer $TOKEN")
        
        if echo "$ME_RESPONSE" | jq -e '.user' > /dev/null 2>&1; then
            echo "   User info: $(echo "$ME_RESPONSE" | jq -r '.user.name')"
            
            # Test companies endpoint
            COMPANIES_RESPONSE=$(curl -s -X GET "$BASE_URL/api/v1/companies" \
              -H "Authorization: Bearer $TOKEN")
            
            if echo "$COMPANIES_RESPONSE" | jq -e '.' > /dev/null 2>&1; then
                COMPANY_COUNT=$(echo "$COMPANIES_RESPONSE" | jq 'length')
                echo "   Companies: ${COMPANY_COUNT}"
                if [ "$COMPANY_COUNT" -gt 0 ]; then
                    echo "   Primary company: $(echo "$COMPANIES_RESPONSE" | jq -r '.[0].name') (Role: $(echo "$COMPANIES_RESPONSE" | jq -r '.[0].role'))"
                fi
            fi
        else
            echo "   ❌ Failed to get user info"
        fi
    else
        echo "❌ Login failed for ${name}"
        echo "   Error: $(echo "$LOGIN_RESPONSE" | jq -r '.error // "Unknown error"')"
    fi
    echo ""
}

# Create demo accounts
echo "2. Creating demo accounts:"

# Store user IDs for later company assignment
ADMIN_USER_ID=""
MANAGER_USER_ID=""
EMPLOYEE1_USER_ID=""
EMPLOYEE2_USER_ID=""
EMPLOYEE3_USER_ID=""

echo "Creating admin account..."
if create_account "admin@shiftlinkr.com" "admin123" "Admin User"; then
    # Get the admin user ID
    LOGIN_RESPONSE=$(curl -s -X POST "$BASE_URL/api/v1/auth/login" \
      -H "Content-Type: application/json" \
      -d '{"email":"admin@shiftlinkr.com","password":"admin123"}')
    
    if echo "$LOGIN_RESPONSE" | jq -e '.token' > /dev/null 2>&1; then
        ADMIN_TOKEN=$(echo "$LOGIN_RESPONSE" | jq -r '.token')
        ME_RESPONSE=$(curl -s -X GET "$BASE_URL/api/v1/auth/me" \
          -H "Authorization: Bearer $ADMIN_TOKEN")
        ADMIN_USER_ID=$(echo "$ME_RESPONSE" | jq -r '.user.id')
        echo "   Admin User ID: $ADMIN_USER_ID"
    fi
fi

echo "Creating manager account..."
if create_account "manager@shiftlinkr.com" "manager123" "Manager Smith"; then
    LOGIN_RESPONSE=$(curl -s -X POST "$BASE_URL/api/v1/auth/login" \
      -H "Content-Type: application/json" \
      -d '{"email":"manager@shiftlinkr.com","password":"manager123"}')
    
    if echo "$LOGIN_RESPONSE" | jq -e '.token' > /dev/null 2>&1; then
        ME_RESPONSE=$(curl -s -X GET "$BASE_URL/api/v1/auth/me" \
          -H "Authorization: Bearer $(echo "$LOGIN_RESPONSE" | jq -r '.token')")
        MANAGER_USER_ID=$(echo "$ME_RESPONSE" | jq -r '.user.id')
        echo "   Manager User ID: $MANAGER_USER_ID"
    fi
fi

echo "Creating employee accounts..."
if create_account "employee1@shiftlinkr.com" "employee123" "John Doe"; then
    LOGIN_RESPONSE=$(curl -s -X POST "$BASE_URL/api/v1/auth/login" \
      -H "Content-Type: application/json" \
      -d '{"email":"employee1@shiftlinkr.com","password":"employee123"}')
    
    if echo "$LOGIN_RESPONSE" | jq -e '.token' > /dev/null 2>&1; then
        ME_RESPONSE=$(curl -s -X GET "$BASE_URL/api/v1/auth/me" \
          -H "Authorization: Bearer $(echo "$LOGIN_RESPONSE" | jq -r '.token')")
        EMPLOYEE1_USER_ID=$(echo "$ME_RESPONSE" | jq -r '.user.id')
        echo "   Employee 1 User ID: $EMPLOYEE1_USER_ID"
    fi
fi

if create_account "employee2@shiftlinkr.com" "employee123" "Jane Smith"; then
    LOGIN_RESPONSE=$(curl -s -X POST "$BASE_URL/api/v1/auth/login" \
      -H "Content-Type: application/json" \
      -d '{"email":"employee2@shiftlinkr.com","password":"employee123"}')
    
    if echo "$LOGIN_RESPONSE" | jq -e '.token' > /dev/null 2>&1; then
        ME_RESPONSE=$(curl -s -X GET "$BASE_URL/api/v1/auth/me" \
          -H "Authorization: Bearer $(echo "$LOGIN_RESPONSE" | jq -r '.token')")
        EMPLOYEE2_USER_ID=$(echo "$ME_RESPONSE" | jq -r '.user.id')
        echo "   Employee 2 User ID: $EMPLOYEE2_USER_ID"
    fi
fi

if create_account "employee3@shiftlinkr.com" "employee123" "Mike Johnson"; then
    LOGIN_RESPONSE=$(curl -s -X POST "$BASE_URL/api/v1/auth/login" \
      -H "Content-Type: application/json" \
      -d '{"email":"employee3@shiftlinkr.com","password":"employee123"}')
    
    if echo "$LOGIN_RESPONSE" | jq -e '.token' > /dev/null 2>&1; then
        ME_RESPONSE=$(curl -s -X GET "$BASE_URL/api/v1/auth/me" \
          -H "Authorization: Bearer $(echo "$LOGIN_RESPONSE" | jq -r '.token')")
        EMPLOYEE3_USER_ID=$(echo "$ME_RESPONSE" | jq -r '.user.id')
        echo "   Employee 3 User ID: $EMPLOYEE3_USER_ID"
    fi
fi

echo ""
echo "3. Creating demo company and assigning roles:"

# Create company (this automatically makes the admin user an admin of the company)
if create_demo_company "admin@shiftlinkr.com" "admin123"; then
    # Add other users to the company
    if [ -n "$ADMIN_TOKEN" ] && [ -n "$COMPANY_ID" ]; then
        # Note: Admin is already added as admin when company is created
        
        # Add manager
        if [ -n "$MANAGER_USER_ID" ]; then
            add_user_to_company "$COMPANY_ID" "$MANAGER_USER_ID" "manager" "$ADMIN_TOKEN" "Manager Smith"
        fi
        
        # Add employees
        if [ -n "$EMPLOYEE1_USER_ID" ]; then
            add_user_to_company "$COMPANY_ID" "$EMPLOYEE1_USER_ID" "employee" "$ADMIN_TOKEN" "John Doe"
        fi
        
        if [ -n "$EMPLOYEE2_USER_ID" ]; then
            add_user_to_company "$COMPANY_ID" "$EMPLOYEE2_USER_ID" "employee" "$ADMIN_TOKEN" "Jane Smith"
        fi
        
        if [ -n "$EMPLOYEE3_USER_ID" ]; then
            add_user_to_company "$COMPANY_ID" "$EMPLOYEE3_USER_ID" "employee" "$ADMIN_TOKEN" "Mike Johnson"
        fi
    fi
fi

echo ""
echo "4. Testing login for each account:"
test_login "admin@shiftlinkr.com" "admin123" "Admin User"
test_login "manager@shiftlinkr.com" "manager123" "Manager Smith"
test_login "employee1@shiftlinkr.com" "employee123" "John Doe"
test_login "employee2@shiftlinkr.com" "employee123" "Jane Smith"
test_login "employee3@shiftlinkr.com" "employee123" "Mike Johnson"

echo "🎉 Demo account creation completed!"
echo ""
echo "📝 Available Demo Accounts (All part of 'ShiftLinkr Demo Company'):"
echo "┌─────────────────────────────────────────────────────────────────────────────┐"
echo "│ Role      │ Email                      │ Password     │ Name             │"
echo "├─────────────────────────────────────────────────────────────────────────────┤"
echo "│ Admin     │ admin@shiftlinkr.com       │ admin123     │ Admin User       │"
echo "│ Manager   │ manager@shiftlinkr.com     │ manager123   │ Manager Smith    │"
echo "│ Employee  │ employee1@shiftlinkr.com   │ employee123  │ John Doe         │"
echo "│ Employee  │ employee2@shiftlinkr.com   │ employee123  │ Jane Smith       │"
echo "│ Employee  │ employee3@shiftlinkr.com   │ employee123  │ Mike Johnson     │"
echo "└─────────────────────────────────────────────────────────────────────────────┘"
echo ""
echo "� Demo Company Details:"
echo "   Name: ShiftLinkr Demo Company"
echo "   Description: A demo company for testing ShiftLinkr features"
echo "   Website: https://demo.shiftlinkr.com"
echo "   Email: contact@demo.shiftlinkr.com"
echo ""
echo "�🌐 You can now use these accounts to test the frontend at:"
echo "   http://localhost:3000/auth/login"
echo ""
echo "✨ All users are automatically assigned to the demo company with appropriate roles!"
echo ""
echo "🔐 Password Reset Testing:"
echo "To test the password reset flow, run:"
echo "   ./test_password_reset.sh"
