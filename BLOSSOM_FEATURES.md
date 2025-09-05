# Blossom Server Features

## 🌸 **Integrated File Storage for Premium Users**

The Blossom server implements the [BUD-01 Blossom protocol](https://github.com/hzrd149/blossom) for decentralized file storage, providing a comprehensive media storage solution for Premium tier subscribers.

## 🎯 **Key Features**

### **BUD-01 Protocol Compliance**
- ✅ **PUT /upload** - Upload files with authentication
- ✅ **GET /:sha256** - Retrieve files by SHA256 hash
- ✅ **HEAD /:sha256** - Get file metadata without downloading
- ✅ **DELETE /:sha256** - Delete owned files
- ✅ **File Deduplication** - Automatic deduplication via SHA256

### **Authentication & Authorization**
- **Nostr Authentication** - NIP-98 HTTP Auth support
- **Premium Subscription Required** - Only premium users can upload
- **User Ownership** - Files are tied to user public keys
- **Secure Deletion** - Only file owners can delete their uploads

### **Storage Backend**
- **MinIO Object Storage** - Scalable, S3-compatible storage
- **PostgreSQL Metadata** - File information and user associations
- **Redis Caching** - Fast file metadata retrieval
- **Automatic Backup** - Built-in replication and backup

### **File Management**
- **Multiple File Types** - Images, videos, audio, documents
- **Size Limits** - 100MB per file (configurable)
- **Smart Deduplication** - Same file uploaded once, referenced many times
- **Access Tracking** - Download counts and usage analytics

## 📋 **Supported File Types**

### **Images**
- JPEG, PNG, GIF, WebP
- Automatic metadata extraction
- Thumbnail generation (planned)

### **Videos**
- MP4, WebM
- Streaming-optimized delivery
- Preview generation (planned)

### **Audio**
- MP3, OGG, WAV
- Streaming support
- Waveform generation (planned)

### **Documents**
- PDF, TXT
- Content indexing (planned)

## 🔧 **API Endpoints**

### **Upload File**
```http
PUT /upload
Content-Type: multipart/form-data
Authorization: Nostr <base64-encoded-event>
X-User-Pubkey: <user-public-key>

file: <binary-file-data>
```

### **Get File**
```http
GET /:sha256
# Returns the actual file with appropriate headers
```

### **Get File Info**
```http
HEAD /:sha256
# Returns metadata headers without file content
```

### **Delete File**
```http
DELETE /:sha256
Authorization: Nostr <base64-encoded-event>
X-User-Pubkey: <user-public-key>
```

### **List User Files**
```http
GET /list
Authorization: Nostr <base64-encoded-event>
X-User-Pubkey: <user-public-key>
```

### **Storage Statistics**
```http
GET /stats
# Public endpoint showing aggregate statistics
```

## 🏗️ **Architecture**

```
┌─────────────────┐    ┌─────────────────┐    ┌─────────────────┐
│   Web Client    │    │  Blossom API    │    │  MinIO Storage  │
│                 │    │                 │    │                 │
│ • File Upload   │◄──►│ • Auth Check    │◄──►│ • Object Store  │
│ • File Access   │    │ • Deduplication │    │ • S3 Compatible │
│ • Management    │    │ • Metadata      │    │ • Replication   │
└─────────────────┘    └─────────────────┘    └─────────────────┘
                              │
                              ▼
                    ┌─────────────────┐
                    │   PostgreSQL    │
                    │                 │
                    │ • File Metadata │
                    │ • User Relations│
                    │ • Access Logs   │
                    └─────────────────┘
```

## 💡 **Use Cases**

### **For Content Creators**
- **Media Hosting** - Host images and videos for Nostr posts
- **Portfolio Storage** - Store and share creative work
- **Backup Solution** - Reliable storage for important files

### **For Developers**
- **App Storage** - Backend storage for Nostr applications
- **CDN Alternative** - Fast, reliable file delivery
- **Integration Ready** - Standard Blossom protocol compatibility

### **For Communities**
- **Shared Resources** - Community files and media
- **Event Photos** - Store and share community events
- **Educational Content** - Host tutorials and guides

## 🔐 **Security Features**

### **Content Validation**
- **File Type Checking** - Only allowed MIME types accepted
- **Size Limitations** - Prevent abuse with size limits
- **Hash Verification** - Ensure file integrity with SHA256
- **Virus Scanning** - Malware detection (planned)

### **Access Control**
- **Authentication Required** - All uploads require Nostr auth
- **Ownership Tracking** - Files linked to uploading user
- **Private Files** - Support for private file access (planned)
- **Rate Limiting** - Prevent spam and abuse

### **Data Protection**
- **Encryption at Rest** - MinIO server-side encryption
- **Secure Transfer** - HTTPS/TLS for all transfers
- **Backup & Recovery** - Automated backup procedures
- **GDPR Compliance** - Data deletion and privacy controls

## 📊 **Pricing & Limits**

### **Standard Tier** (3,500 sats/month)
- **Storage**: 1GB included
- **Upload Access**: View-only (no uploads)
- **Download**: Unlimited downloads of public files

### **Premium Tier** (6,500 sats/month)
- **Storage**: 10GB included
- **Upload Access**: Full upload/delete permissions
- **File Types**: All supported formats
- **Priority Support**: 24-hour response time
- **Advanced Features**: Early access to new features

## 🚀 **Development Status**

### ✅ **Completed**
- BUD-01 protocol implementation
- MinIO storage integration
- PostgreSQL metadata storage
- Basic authentication system
- File upload/download/delete
- Docker containerization

### 🔄 **In Progress**
- Nostr NIP-98 authentication
- Premium subscription verification
- Advanced file management UI
- Storage quota enforcement

### 📋 **Planned**
- Thumbnail generation
- Video transcoding
- Content indexing
- Advanced analytics
- CDN integration
- Mobile app support

## 🛠️ **Getting Started**

### **For Developers**
```bash
# Start the blossom server
make dev-blossom

# Test file upload
curl -X PUT http://localhost:8004/upload \
  -H "X-User-Pubkey: your-pubkey" \
  -F "file=@test-image.jpg"

# Get file info
curl -I http://localhost:8004/your-file-sha256
```

### **For Users**
1. **Upgrade to Premium** - Subscribe to Premium tier (6,500 sats/month)
2. **Access Dashboard** - Use the web interface for file management
3. **Upload Files** - Drag and drop files in the dashboard
4. **Share Links** - Get permanent links for your files
5. **Manage Storage** - Monitor usage and organize files

## 🌟 **Competitive Advantages**

### **vs Traditional File Hosts**
- ✅ **Decentralized** - No single point of failure
- ✅ **Nostr Native** - Integrated with Nostr ecosystem
- ✅ **User Owned** - Users control their data
- ✅ **Transparent** - Open source and auditable

### **vs Other Blossom Servers**
- ✅ **Integrated Service** - Part of complete Nostr package
- ✅ **Premium Support** - Professional support included
- ✅ **Reliable Infrastructure** - Enterprise-grade storage
- ✅ **Community Governed** - Democratic development process

This Blossom server integration makes Pleb.One a complete Nostr infrastructure solution, providing everything users need in one cohesive, community-owned package! 🌸
