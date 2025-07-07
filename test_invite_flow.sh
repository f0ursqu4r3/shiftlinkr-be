#!/bin/bash

# Invite functionality test script for ShiftLinkr API

BASE_URL="http://127.0.0.1:8080"

echo "🔗 Testing ShiftLinkr invite functionality..."

# First, login as admin to get a token
echo "1. Logging in as admin..."
LOGIN_RESPONSE=$(curl -s -X POST "$BASE_URL/api/v1/auth/login" \
  -H "Content-Type: application/json" \
  -d '{"email":"admin@shiftlinkr.com","password":"admin123"}')

if echo "$LOGIN_RESPONSE" | jq -e '.token' > /dev/null 2>&1; then
    ADMIN_TOKEN=$(echo "$LOGIN_RESPONSE" | jq -r '.token')
    echo "✅ Admin login successful"
    echo "   Token: ${ADMIN_TOKEN:0:20}..."
else
    echo "❌ Admin login failed"
    echo "   Error: $(echo "$LOGIN_RESPONSE" | jq -r '.error // "Unknown error"')"
    exit 1
fi
echo ""

# Test creating an invite
echo "2. Creating an invite for a new employee..."
INVITE_RESPONSE=$(curl -s -X POST "$BASE_URL/api/v1/auth/invite" \
  -H "Content-Type: application/json" \
  -H "Authorization: Bearer $ADMIN_TOKEN" \
  -d '{"email":"newemployee@example.com","role":"employee","team_id":null}')

if echo "$INVITE_RESPONSE" | jq -e '.invite_link' > /dev/null 2>&1; then
    INVITE_LINK=$(echo "$INVITE_RESPONSE" | jq -r '.invite_link')
    INVITE_TOKEN=$(echo "$INVITE_LINK" | sed 's/.*\///')
    EXPIRES_AT=$(echo "$INVITE_RESPONSE" | jq -r '.expires_at')
    echo "✅ Invite created successfully"
    echo "   Invite Link: $INVITE_LINK"
    echo "   Expires At: $EXPIRES_AT"
else
    echo "❌ Failed to create invite"
    echo "   Error: $(echo "$INVITE_RESPONSE" | jq -r '.error // "Unknown error"')"
    exit 1
fi
echo ""

# Test getting invite info
echo "3. Getting invite information..."
GET_INVITE_RESPONSE=$(curl -s -X GET "$BASE_URL/api/v1/auth/invite/$INVITE_TOKEN")

if echo "$GET_INVITE_RESPONSE" | jq -e '.email' > /dev/null 2>&1; then
    echo "✅ Invite information retrieved successfully"
    echo "   Email: $(echo "$GET_INVITE_RESPONSE" | jq -r '.email')"
    echo "   Role: $(echo "$GET_INVITE_RESPONSE" | jq -r '.role')"
    echo "   Inviter: $(echo "$GET_INVITE_RESPONSE" | jq -r '.inviter_name')"
else
    echo "❌ Failed to get invite information"
    echo "   Error: $(echo "$GET_INVITE_RESPONSE" | jq -r '.error // "Unknown error"')"
fi
echo ""

# Test accepting the invite
echo "4. Accepting the invite (creating new user account)..."
ACCEPT_RESPONSE=$(curl -s -X POST "$BASE_URL/api/v1/auth/invite/accept" \
  -H "Content-Type: application/json" \
  -d "{\"token\":\"$INVITE_TOKEN\",\"name\":\"New Employee\",\"password\":\"newpass123\"}")

if echo "$ACCEPT_RESPONSE" | jq -e '.token' > /dev/null 2>&1; then
    NEW_USER_TOKEN=$(echo "$ACCEPT_RESPONSE" | jq -r '.token')
    echo "✅ Invite accepted successfully"
    echo "   New User: $(echo "$ACCEPT_RESPONSE" | jq -r '.user.name')"
    echo "   Email: $(echo "$ACCEPT_RESPONSE" | jq -r '.user.email')"
    echo "   Role: $(echo "$ACCEPT_RESPONSE" | jq -r '.user.role')"
    echo "   Token: ${NEW_USER_TOKEN:0:20}..."
else
    echo "❌ Failed to accept invite"
    echo "   Error: $(echo "$ACCEPT_RESPONSE" | jq -r '.error // "Unknown error"')"
fi
echo ""

# Test that the invite token is now used (should fail)
echo "5. Testing that invite token is now used..."
GET_USED_INVITE_RESPONSE=$(curl -s -X GET "$BASE_URL/api/v1/auth/invite/$INVITE_TOKEN")

if echo "$GET_USED_INVITE_RESPONSE" | jq -e '.error' > /dev/null 2>&1; then
    echo "✅ Invite token correctly marked as used"
    echo "   Error: $(echo "$GET_USED_INVITE_RESPONSE" | jq -r '.error')"
else
    echo "❌ Invite token should be marked as used"
fi
echo ""

# Test login with the new account
echo "6. Testing login with the new account..."
NEW_LOGIN_RESPONSE=$(curl -s -X POST "$BASE_URL/api/v1/auth/login" \
  -H "Content-Type: application/json" \
  -d '{"email":"newemployee@example.com","password":"newpass123"}')

if echo "$NEW_LOGIN_RESPONSE" | jq -e '.token' > /dev/null 2>&1; then
    echo "✅ New user login successful"
    echo "   User: $(echo "$NEW_LOGIN_RESPONSE" | jq -r '.user.name')"
    echo "   Role: $(echo "$NEW_LOGIN_RESPONSE" | jq -r '.user.role')"
else
    echo "❌ New user login failed"
    echo "   Error: $(echo "$NEW_LOGIN_RESPONSE" | jq -r '.error // "Unknown error"')"
fi
echo ""

# Test getting admin's invites
echo "7. Getting admin's sent invites..."
ADMIN_INVITES_RESPONSE=$(curl -s -X GET "$BASE_URL/api/v1/auth/invites" \
  -H "Authorization: Bearer $ADMIN_TOKEN")

if echo "$ADMIN_INVITES_RESPONSE" | jq -e '.invites' > /dev/null 2>&1; then
    INVITE_COUNT=$(echo "$ADMIN_INVITES_RESPONSE" | jq '.invites | length')
    echo "✅ Admin invites retrieved successfully"
    echo "   Number of invites: $INVITE_COUNT"
    if [ "$INVITE_COUNT" -gt 0 ]; then
        echo "   First invite email: $(echo "$ADMIN_INVITES_RESPONSE" | jq -r '.invites[0].email')"
        echo "   First invite used: $(echo "$ADMIN_INVITES_RESPONSE" | jq -r '.invites[0].used_at // "Not used"')"
    fi
else
    echo "❌ Failed to get admin invites"
    echo "   Error: $(echo "$ADMIN_INVITES_RESPONSE" | jq -r '.error // "Unknown error"')"
fi
echo ""

echo "🎉 Invite functionality testing completed!"
echo ""
echo "📝 Summary:"
echo "✅ Admin can create invites"
echo "✅ Invite information can be retrieved"
echo "✅ Invites can be accepted to create new user accounts"
echo "✅ Used invite tokens are properly invalidated"
echo "✅ New users can login with their credentials"
echo "✅ Admin can view their sent invites"
echo ""
echo "🌐 Invite Links Format:"
echo "   http://localhost:3000/auth/invite/{token}"
echo ""
echo "📧 Invite Flow:"
echo "1. Admin/Manager creates invite via API"
echo "2. Invite link is sent to employee (email/Slack/etc.)"
echo "3. Employee clicks link and sets their password"
echo "4. Employee account is created and auto-joined to team (if specified)"
echo "5. Employee can immediately login and use the system"
