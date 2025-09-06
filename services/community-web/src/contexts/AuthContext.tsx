import { createContext, useContext, useState, useEffect, ReactNode } from 'react'
import { generatePrivateKey, getPublicKey, nip19, SimplePool, Filter, Event } from 'nostr-tools'

interface NostrUser {
  npub: string
  pubkey: string
  privkey?: string
  profile?: {
    name?: string
    about?: string
    picture?: string
    nip05?: string
    lud06?: string
    lud16?: string
    banner?: string
    website?: string
  }
}

interface AuthContextType {
  user: NostrUser | null
  isAuthenticated: boolean
  loginWithExtension: () => Promise<void>
  loginWithNpub: (npub: string) => Promise<void>
  generateNewAccount: () => Promise<NostrUser>
  logout: () => void
  loading: boolean
  error: string | null
}

const AuthContext = createContext<AuthContextType | undefined>(undefined)

export function useAuth() {
  const context = useContext(AuthContext)
  if (context === undefined) {
    throw new Error('useAuth must be used within an AuthProvider')
  }
  return context
}

interface AuthProviderProps {
  children: ReactNode
}

export function AuthProvider({ children }: AuthProviderProps) {
  const [user, setUser] = useState<NostrUser | null>(null)
  const [loading, setLoading] = useState(false)
  const [error, setError] = useState<string | null>(null)

  const isAuthenticated = user !== null

  // Define relay URLs to fetch profile from
  const relayUrls = [
    'wss://relay.damus.io',
    'wss://nos.lol',
    'wss://relay.primal.net',
    'wss://relay.nostr.band',
    'wss://r1.pleb.one'
  ]

  // Function to fetch profile information from relays
  const fetchProfile = async (pubkey: string) => {
    const pool = new SimplePool()
    
    try {
      console.log('Fetching profile for pubkey:', pubkey)
      console.log('Using relay URLs:', relayUrls)
      
      // Create filter for metadata events (kind 0)
      const filter: Filter = {
        kinds: [0],
        authors: [pubkey],
        limit: 1
      }

      console.log('Profile filter:', filter)

      // Fetch from relays with timeout
      const events = await pool.list(relayUrls, [filter])
      console.log('Received events:', events.length)
      
      if (events.length > 0) {
        // Get the most recent profile event
        const profileEvent = events.sort((a: Event, b: Event) => b.created_at - a.created_at)[0]
        console.log('Most recent profile event:', profileEvent)
        
        try {
          const profile = JSON.parse(profileEvent.content)
          console.log('Parsed profile:', profile)
          return profile
        } catch (err) {
          console.warn('Failed to parse profile content:', err)
          return {}
        }
      }
      
      console.log('No profile events found')
      return {}
    } catch (err) {
      console.warn('Failed to fetch profile from relays:', err)
      return {}
    } finally {
      pool.close(relayUrls)
    }
  }

  // Check for existing session on mount
  useEffect(() => {
    const savedUser = localStorage.getItem('nostr-user')
    if (savedUser) {
      try {
        setUser(JSON.parse(savedUser))
      } catch (err) {
        console.error('Failed to parse saved user:', err)
        localStorage.removeItem('nostr-user')
      }
    }
  }, [])

  const loginWithExtension = async () => {
    setLoading(true)
    setError(null)
    
    try {
      // Check if NIP-07 extension is available
      if (!(window as any).nostr) {
        throw new Error('No Nostr browser extension found. Please install a NIP-07 compatible extension like Alby, nos2x, or Flamingo.')
      }

      // Get public key from extension
      const pubkey = await (window as any).nostr.getPublicKey()
      const npub = nip19.npubEncode(pubkey)

      // Fetch profile information from relays
      console.log('Fetching profile for pubkey:', pubkey)
      const profile = await fetchProfile(pubkey)
      console.log('Fetched profile:', profile)

      const newUser: NostrUser = {
        npub,
        pubkey,
        profile
      }

      setUser(newUser)
      localStorage.setItem('nostr-user', JSON.stringify(newUser))
    } catch (err) {
      setError(err instanceof Error ? err.message : 'Failed to connect with extension')
    } finally {
      setLoading(false)
    }
  }

  const loginWithNpub = async (npub: string) => {
    setLoading(true)
    setError(null)

    try {
      // Validate npub format
      if (!npub.startsWith('npub1')) {
        throw new Error('Invalid npub format. Must start with "npub1"')
      }

      const pubkey = nip19.decode(npub).data as string

      // Fetch profile information from relays
      console.log('Fetching profile for pubkey:', pubkey)
      const profile = await fetchProfile(pubkey)
      console.log('Fetched profile:', profile)

      const newUser: NostrUser = {
        npub,
        pubkey,
        profile
      }

      setUser(newUser)
      localStorage.setItem('nostr-user', JSON.stringify(newUser))
    } catch (err) {
      setError('Invalid npub format or unable to decode')
    } finally {
      setLoading(false)
    }
  }

  const generateNewAccount = async (): Promise<NostrUser> => {
    setLoading(true)
    setError(null)

    try {
      const privkey = generatePrivateKey()
      const pubkey = getPublicKey(privkey)
      const npub = nip19.npubEncode(pubkey)
      const nsec = nip19.nsecEncode(privkey)

      const newUser: NostrUser = {
        npub,
        pubkey,
        privkey,
        profile: {}
      }

      setUser(newUser)
      localStorage.setItem('nostr-user', JSON.stringify(newUser))

      // Show the private key to user for backup
      alert(`🔐 IMPORTANT: Save your private key (nsec)!\n\n${nsec}\n\nThis is the only time it will be shown. Store it safely!`)

      return newUser
    } catch (err) {
      setError('Failed to generate new account')
      throw err
    } finally {
      setLoading(false)
    }
  }

  const logout = () => {
    setUser(null)
    localStorage.removeItem('nostr-user')
    setError(null)
  }

  const value: AuthContextType = {
    user,
    isAuthenticated,
    loginWithExtension,
    loginWithNpub,
    generateNewAccount,
    logout,
    loading,
    error
  }

  return (
    <AuthContext.Provider value={value}>
      {children}
    </AuthContext.Provider>
  )
}
