import React, { useState, useEffect, useRef } from 'react'

interface NostrEvent {
  id: string
  pubkey: string
  created_at: number
  kind: number
  tags: string[][]
  content: string
  sig: string
}

interface Profile {
  name?: string
  display_name?: string
  about?: string
  picture?: string
  nip05?: string
}

interface FeedItem {
  event: NostrEvent
  profile: Profile | null
  timeAgo: string
}

const FeedPage: React.FC = () => {
  const [feedItems, setFeedItems] = useState<FeedItem[]>([])
  const [isConnected, setIsConnected] = useState(false)
  const [connectionStatus, setConnectionStatus] = useState('Connecting...')
  const wsRef = useRef<WebSocket | null>(null)
  const profileCache = useRef<Map<string, Profile>>(new Map())

  useEffect(() => {
    connectToRelay()
    return () => {
      if (wsRef.current) {
        wsRef.current.close()
      }
    }
  }, [])

  const connectToRelay = () => {
    try {
      const ws = new WebSocket('wss://r1.pleb.one')
      wsRef.current = ws

      ws.onopen = () => {
        setIsConnected(true)
        setConnectionStatus('Connected')
        
        // Subscribe to all recent events (text notes, long-form content, etc.)
        const subscription = JSON.stringify([
          "REQ",
          "feed-sub",
          {
            "kinds": [0, 1, 30023], // metadata, text notes, long-form content
            "limit": 100
          }
        ])
        ws.send(subscription)
      }

      ws.onmessage = (event) => {
        try {
          const message = JSON.parse(event.data)
          handleRelayMessage(message)
        } catch (error) {
          console.error('Error parsing relay message:', error)
        }
      }

      ws.onclose = () => {
        setIsConnected(false)
        setConnectionStatus('Disconnected')
        // Attempt to reconnect after 3 seconds
        setTimeout(connectToRelay, 3000)
      }

      ws.onerror = (error) => {
        console.error('WebSocket error:', error)
        setConnectionStatus('Connection error')
      }
    } catch (error) {
      console.error('Failed to connect to relay:', error)
      setConnectionStatus('Failed to connect')
    }
  }

  const handleRelayMessage = (message: any[]) => {
    if (message[0] === 'EVENT') {
      const event: NostrEvent = message[2]
      
      // Handle metadata events (kind 0) for profile info
      if (event.kind === 0) {
        try {
          const profile: Profile = JSON.parse(event.content)
          profileCache.current.set(event.pubkey, profile)
          
          // Update existing feed items with this profile
          setFeedItems(prev => prev.map(item => 
            item.event.pubkey === event.pubkey 
              ? { ...item, profile }
              : item
          ))
        } catch (error) {
          console.error('Error parsing profile metadata:', error)
        }
      }
      
      // Handle content events (kind 1 and 30023)
      if (event.kind === 1 || event.kind === 30023) {
        const profile = profileCache.current.get(event.pubkey) || null
        const timeAgo = formatTimeAgo(event.created_at)
        
        const newFeedItem: FeedItem = {
          event,
          profile,
          timeAgo
        }
        
        setFeedItems(prev => {
          // Avoid duplicates
          if (prev.some(item => item.event.id === event.id)) {
            return prev
          }
          
          // Add new item and sort by created_at (newest first)
          const updated = [newFeedItem, ...prev]
          return updated
            .sort((a, b) => b.event.created_at - a.event.created_at)
            .slice(0, 100) // Keep only latest 100 items
        })
        
        // Request profile if we don't have it
        if (!profile && wsRef.current?.readyState === WebSocket.OPEN) {
          const profileReq = JSON.stringify([
            "REQ",
            `profile-${event.pubkey}`,
            {
              "kinds": [0],
              "authors": [event.pubkey],
              "limit": 1
            }
          ])
          wsRef.current.send(profileReq)
        }
      }
    }
  }

  const formatTimeAgo = (timestamp: number): string => {
    const now = Math.floor(Date.now() / 1000)
    const diff = now - timestamp
    
    if (diff < 60) return `${diff}s ago`
    if (diff < 3600) return `${Math.floor(diff / 60)}m ago`
    if (diff < 86400) return `${Math.floor(diff / 3600)}h ago`
    if (diff < 604800) return `${Math.floor(diff / 86400)}d ago`
    return new Date(timestamp * 1000).toLocaleDateString()
  }

  const getDisplayName = (profile: Profile | null, pubkey: string): string => {
    if (profile?.display_name) return profile.display_name
    if (profile?.name) return profile.name
    return `${pubkey.slice(0, 8)}...${pubkey.slice(-4)}`
  }

  const getProfilePicture = (profile: Profile | null): string => {
    return profile?.picture || `https://api.dicebear.com/7.x/identicon/svg?seed=${Math.random()}`
  }

  const truncateContent = (content: string, maxLength: number = 280): string => {
    if (content.length <= maxLength) return content
    return content.slice(0, maxLength) + '...'
  }

  const getEventTypeLabel = (kind: number): string => {
    switch (kind) {
      case 1: return 'Note'
      case 30023: return 'Article'
      default: return `Kind ${kind}`
    }
  }

  const getEventTitle = (event: NostrEvent): string => {
    if (event.kind === 30023) {
      const titleTag = event.tags.find(tag => tag[0] === 'title')
      return titleTag?.[1] || 'Untitled Article'
    }
    return ''
  }

  return (
    <div className="min-h-screen bg-gray-50">
      {/* Header */}
      <div className="bg-white shadow-sm border-b">
        <div className="max-w-4xl mx-auto px-4 py-4">
          <div className="flex items-center justify-between">
            <div>
              <h1 className="text-2xl font-bold text-gray-900">Live Feed</h1>
              <p className="text-sm text-gray-600">Real-time posts from r1.pleb.one</p>
            </div>
            <div className="flex items-center space-x-2">
              <div className={`w-3 h-3 rounded-full ${isConnected ? 'bg-green-500' : 'bg-red-500'}`}></div>
              <span className="text-sm text-gray-600">{connectionStatus}</span>
            </div>
          </div>
        </div>
      </div>

      {/* Feed */}
      <div className="max-w-4xl mx-auto px-4 py-6">
        {feedItems.length === 0 ? (
          <div className="text-center py-12">
            <div className="text-gray-500">
              {isConnected ? 'Waiting for posts...' : 'Connecting to relay...'}
            </div>
          </div>
        ) : (
          <div className="space-y-4">
            {feedItems.map((item) => (
              <div key={item.event.id} className="bg-white rounded-lg shadow-sm border p-6">
                {/* Header */}
                <div className="flex items-center space-x-3 mb-4">
                  <img
                    src={getProfilePicture(item.profile)}
                    alt="Profile"
                    className="w-12 h-12 rounded-full bg-gray-200"
                    onError={(e) => {
                      e.currentTarget.src = `https://api.dicebear.com/7.x/identicon/svg?seed=${item.event.pubkey}`
                    }}
                  />
                  <div className="flex-1 min-w-0">
                    <div className="flex items-center space-x-2">
                      <h3 className="text-sm font-semibold text-gray-900 truncate">
                        {getDisplayName(item.profile, item.event.pubkey)}
                      </h3>
                      <span className="inline-flex items-center px-2 py-1 rounded-full text-xs font-medium bg-blue-100 text-blue-800">
                        {getEventTypeLabel(item.event.kind)}
                      </span>
                    </div>
                    <div className="flex items-center space-x-2 text-xs text-gray-500">
                      <span>{item.timeAgo}</span>
                      {item.profile?.nip05 && (
                        <>
                          <span>•</span>
                          <span className="text-blue-600">✓ {item.profile.nip05}</span>
                        </>
                      )}
                    </div>
                  </div>
                </div>

                {/* Content */}
                <div className="space-y-3">
                  {item.event.kind === 30023 && getEventTitle(item.event) && (
                    <h2 className="text-lg font-semibold text-gray-900">
                      {getEventTitle(item.event)}
                    </h2>
                  )}
                  
                  <div className="text-gray-700 whitespace-pre-wrap">
                    {truncateContent(item.event.content)}
                  </div>

                  {/* Metadata */}
                  <div className="flex items-center justify-between pt-2 text-xs text-gray-500 border-t">
                    <span>Event ID: {item.event.id.slice(0, 16)}...</span>
                    <span>Size: {item.event.content.length} chars</span>
                  </div>
                </div>
              </div>
            ))}
          </div>
        )}
      </div>
    </div>
  )
}

export default FeedPage
