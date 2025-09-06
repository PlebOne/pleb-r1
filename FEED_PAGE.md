# Live Feed Page

The Live Feed page displays real-time posts flowing through the Pleb.One Nostr relay in a firehose format.

## Features

### Real-time Updates
- **WebSocket Connection**: Direct connection to `wss://r1.pleb.one`
- **Auto-reconnect**: Automatically reconnects if connection is lost
- **Live Status**: Connection indicator shows current status

### Event Types Displayed
- **Text Notes (Kind 1)**: Regular short posts and notes
- **Long-form Content (Kind 30023)**: Articles and blog posts with titles
- **Profile Metadata (Kind 0)**: User profile information for displaying names and pictures

### User Profiles
- **Profile Pictures**: Shows user avatars from their profile metadata
- **Display Names**: Shows preferred name, username, or shortened pubkey
- **NIP-05 Verification**: Displays verification badges for verified users
- **Profile Caching**: Efficiently caches profile data to reduce relay requests

### Content Display
- **Truncated Content**: Long posts are automatically truncated for readability
- **Time Stamps**: Shows relative time (e.g., "2m ago", "1h ago")
- **Event Metadata**: Displays event ID and content length
- **Article Titles**: Shows titles for long-form content (NIP-23)

### Technical Implementation

#### WebSocket Management
```typescript
// Establishes WebSocket connection
const ws = new WebSocket('wss://r1.pleb.one')

// Subscribes to recent events
const subscription = JSON.stringify([
  "REQ",
  "feed-sub", 
  {
    "kinds": [0, 1, 30023], // metadata, notes, articles
    "limit": 100
  }
])
```

#### Profile Resolution
- Automatically requests profile metadata for unknown users
- Maintains local cache to avoid duplicate requests
- Updates existing feed items when profile data arrives

#### Event Processing
- Filters and validates incoming events
- Prevents duplicate events from appearing
- Sorts events by creation time (newest first)
- Maintains a maximum of 100 items for performance

## Usage

### Accessing the Feed
- Navigate to `/feed` in the web app
- Click "View Live Feed" from the homepage
- Use the "Feed" link in the navigation

### Real-time Monitoring
- Watch posts appear as they're published to the relay
- See user profiles resolve in real-time
- Monitor connection status with the indicator

### Content Types
- **Notes**: Short-form posts appear with user info
- **Articles**: Long-form content shows with titles
- **Profiles**: User metadata updates existing posts

## Privacy & Security

### No Authentication Required
- Reading the feed requires no login
- All displayed content is public relay data
- No personal information is collected

### External Resources
- Profile pictures loaded from user-provided URLs
- Fallback avatars generated using DiceBear API
- No tracking or analytics implemented

## Performance

### Optimization Features
- WebSocket connection reuse
- Profile data caching
- Limited feed history (100 items)
- Automatic content truncation
- Efficient event deduplication

### Resource Management
- Automatic connection cleanup on page exit
- Memory-conscious profile cache
- Throttled profile requests

## Browser Compatibility

- Modern browsers with WebSocket support
- Responsive design for mobile and desktop
- Graceful fallbacks for connection failures

## Related Features

- **Dashboard**: View relay metrics and statistics
- **Identity**: Manage your Nostr identity
- **Community**: Connect with other users

The Live Feed provides transparency into relay activity and demonstrates the real-time nature of the Nostr protocol.
