#!/bin/bash

# Test Nostr relay WebSocket connection with a simple REQ message
echo "Testing Nostr relay at wss://r1.pleb.one"

# Create a simple REQ message (requesting events)
echo '["REQ","test-sub",{"kinds":[1],"limit":1}]' | websocat wss://r1.pleb.one --exit-on-eof --oneshot

echo "Test completed"
