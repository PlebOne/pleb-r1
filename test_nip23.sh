#!/bin/bash

# Test NIP-23 long-form content event
echo "Testing NIP-23 long-form content support..."

# Read the test event
TEST_EVENT=$(cat test_nip23.json)

# Create the EVENT message
EVENT_MESSAGE='["EVENT",'$TEST_EVENT']'

echo "Sending test event:"
echo "$EVENT_MESSAGE" | jq '.'

# Send to relay using websocat
echo "$EVENT_MESSAGE" | timeout 10 ./websocat wss://r1.pleb.one/ || echo "Test completed"
