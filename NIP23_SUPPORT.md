# NIP-23 Long-form Content Support

The Pleb-R1 relay now supports NIP-23 for long-form content (kind 30023).

## Features

### Event Validation
- **Content Length**: 100 bytes minimum, 500KB maximum
- **Required Tags**: 
  - `d` tag (identifier) - required for parameterized replaceable events
  - `title` tag - recommended
- **Optional Tags**:
  - `summary` - article summary (max 1000 chars)
  - `published_at` - publication timestamp
  - Other standard Nostr tags

### Content Filtering
- Spam detection using keyword analysis
- Content moderation capabilities
- Mention extraction support

## Event Structure

```json
{
  "id": "<event-id>",
  "pubkey": "<author-pubkey>", 
  "created_at": <timestamp>,
  "kind": 30023,
  "tags": [
    ["d", "<unique-identifier>"],
    ["title", "<article-title>"],
    ["summary", "<article-summary>"],
    ["published_at", "<publish-timestamp>"]
  ],
  "content": "# Article Title\n\nArticle content in markdown format...",
  "sig": "<signature>"
}
```

## Usage

Long-form content can be published by Nostr clients that support NIP-23, such as:
- Habla.news
- YakiHonne  
- Blogstr
- Alexandria

## Relay Information

- **WebSocket URL**: `wss://r1.pleb.one`
- **Supported Event Kinds**: 30023 (long-form content)
- **Rate Limits**: Standard relay rate limiting applies
- **Authentication**: Not required for reading, optional for publishing

## Related NIPs

This implementation also supports the following related NIPs:
- NIP-01: Basic protocol flow
- NIP-02: Contact Lists
- NIP-09: Event Deletion  
- NIP-11: Relay Information Document
- NIP-12: Generic Tag Queries
- NIP-15: End of Stored Events Notice
- NIP-16: Event Treatment
- NIP-20: Command Results
