#!/bin/bash

# Generate some test events for the feed
echo "Generating test events for the feed..."

# Function to send an event
send_event() {
    local content="$1"
    local kind="$2"
    local pubkey="$3"
    local name="$4"
    local picture="$5"
    
    # First send profile metadata (kind 0)
    if [ ! -z "$name" ]; then
        local profile_content='{"name":"'$name'","about":"Test user for Pleb.One relay","picture":"'$picture'"}'
        local profile_event='["EVENT",{"id":"profile_'$pubkey'","pubkey":"'$pubkey'","created_at":'$(date +%s)',"kind":0,"tags":[],"content":"'$profile_content'","sig":"testsig_profile_'$pubkey'"}]'
        echo "$profile_event" | timeout 3 ./websocat wss://r1.pleb.one/ 2>/dev/null || true
        sleep 1
    fi
    
    # Then send the content event
    local event='["EVENT",{"id":"test_'$(date +%s%N)'","pubkey":"'$pubkey'","created_at":'$(date +%s)',"kind":'$kind',"tags":[],"content":"'$content'","sig":"testsig_'$(date +%s)'"}]'
    echo "$event" | timeout 3 ./websocat wss://r1.pleb.one/ 2>/dev/null || true
    echo "Sent: $content"
}

# Generate test users and posts
send_event "Welcome to Pleb.One! This is a test of our new relay infrastructure. #nostr #bitcoin" 1 "user1234567890abcdef" "Alice" "https://api.dicebear.com/7.x/avataaars/svg?seed=Alice"

sleep 2

send_event "Just published my first long-form article on Nostr! The future of decentralized publishing is here. 🚀" 1 "user2345678901bcdefg" "Bob" "https://api.dicebear.com/7.x/avataaars/svg?seed=Bob"

sleep 2

send_event "GM Nostr! Beautiful day to build on the most important protocol of our time. ☀️ #gm #nostr" 1 "user3456789012cdefgh" "Charlie" "https://api.dicebear.com/7.x/avataaars/svg?seed=Charlie"

sleep 2

send_event "Testing the relay with some technical content. Excited to see how NIP-23 long-form content works! 📝" 1 "user4567890123defghi" "Diana" "https://api.dicebear.com/7.x/avataaars/svg?seed=Diana"

echo "Test events generated! Check the feed page to see them."
