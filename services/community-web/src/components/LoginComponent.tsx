import { useState } from 'react'
import { Key, UserPlus, Zap, AlertCircle } from 'lucide-react'
import { useAuth } from '../contexts/AuthContext'

export function LoginComponent() {
  const { loginWithExtension, loginWithNpub, generateNewAccount, loading, error } = useAuth()
  const [npubInput, setNpubInput] = useState('')
  const [activeTab, setActiveTab] = useState<'extension' | 'npub' | 'generate'>('extension')

  const handleNpubSubmit = async (e: React.FormEvent) => {
    e.preventDefault()
    if (npubInput.trim()) {
      await loginWithNpub(npubInput.trim())
    }
  }

  const handleGenerateAccount = async () => {
    try {
      await generateNewAccount()
    } catch (err) {
      console.error('Failed to generate account:', err)
    }
  }

  return (
    <div className="max-w-md mx-auto bg-white rounded-lg shadow-lg p-6">
      <div className="text-center mb-6">
        <h2 className="text-2xl font-bold text-gray-900">Connect to Pleb-R1</h2>
        <p className="text-gray-600 mt-2">Choose how you'd like to authenticate</p>
      </div>

      {/* Tab Navigation */}
      <div className="flex space-x-1 mb-6 bg-gray-100 p-1 rounded-lg">
        <button
          onClick={() => setActiveTab('extension')}
          className={`flex-1 py-2 px-3 rounded-md text-sm font-medium transition-colors ${
            activeTab === 'extension'
              ? 'bg-white text-pleb-600 shadow-sm'
              : 'text-gray-500 hover:text-gray-700'
          }`}
        >
          <Zap className="h-4 w-4 inline mr-1" />
          Extension
        </button>
        <button
          onClick={() => setActiveTab('npub')}
          className={`flex-1 py-2 px-3 rounded-md text-sm font-medium transition-colors ${
            activeTab === 'npub'
              ? 'bg-white text-pleb-600 shadow-sm'
              : 'text-gray-500 hover:text-gray-700'
          }`}
        >
          <Key className="h-4 w-4 inline mr-1" />
          npub
        </button>
        <button
          onClick={() => setActiveTab('generate')}
          className={`flex-1 py-2 px-3 rounded-md text-sm font-medium transition-colors ${
            activeTab === 'generate'
              ? 'bg-white text-pleb-600 shadow-sm'
              : 'text-gray-500 hover:text-gray-700'
          }`}
        >
          <UserPlus className="h-4 w-4 inline mr-1" />
          New
        </button>
      </div>

      {/* Error Display */}
      {error && (
        <div className="mb-4 p-3 bg-red-50 border border-red-200 rounded-md">
          <div className="flex items-center">
            <AlertCircle className="h-4 w-4 text-red-500 mr-2" />
            <span className="text-red-700 text-sm">{error}</span>
          </div>
        </div>
      )}

      {/* Tab Content */}
      <div className="space-y-4">
        {activeTab === 'extension' && (
          <div className="space-y-4">
            <div className="text-center">
              <Zap className="h-12 w-12 text-pleb-600 mx-auto mb-3" />
              <h3 className="text-lg font-semibold text-gray-900 mb-2">Browser Extension</h3>
              <p className="text-gray-600 text-sm mb-4">
                Use your NIP-07 compatible browser extension like Alby, nos2x, or Flamingo
              </p>
            </div>
            <button
              onClick={loginWithExtension}
              disabled={loading}
              className="w-full btn-primary disabled:opacity-50 disabled:cursor-not-allowed"
            >
              {loading ? 'Connecting...' : 'Connect with Extension'}
            </button>
            <div className="text-xs text-gray-500 space-y-1">
              <p>✓ Most secure option</p>
              <p>✓ Keys stay in your browser</p>
              <p>✓ Can sign transactions</p>
            </div>
          </div>
        )}

        {activeTab === 'npub' && (
          <div className="space-y-4">
            <div className="text-center">
              <Key className="h-12 w-12 text-pleb-600 mx-auto mb-3" />
              <h3 className="text-lg font-semibold text-gray-900 mb-2">Enter npub</h3>
              <p className="text-gray-600 text-sm mb-4">
                Enter your Nostr public key (npub) to connect in read-only mode
              </p>
            </div>
            <form onSubmit={handleNpubSubmit} className="space-y-4">
              <div>
                <label htmlFor="npub" className="block text-sm font-medium text-gray-700 mb-1">
                  Nostr Public Key (npub)
                </label>
                <input
                  id="npub"
                  type="text"
                  value={npubInput}
                  onChange={(e) => setNpubInput(e.target.value)}
                  placeholder="npub1..."
                  className="w-full px-3 py-2 border border-gray-300 rounded-md focus:outline-none focus:ring-2 focus:ring-pleb-500 focus:border-transparent font-mono text-sm"
                  required
                />
              </div>
              <button
                type="submit"
                disabled={loading || !npubInput.trim()}
                className="w-full btn-primary disabled:opacity-50 disabled:cursor-not-allowed"
              >
                {loading ? 'Connecting...' : 'Connect with npub'}
              </button>
            </form>
            <div className="text-xs text-gray-500 space-y-1">
              <p>⚠️ Read-only mode</p>
              <p>✓ No private key required</p>
              <p>✓ Safe for public computers</p>
            </div>
          </div>
        )}

        {activeTab === 'generate' && (
          <div className="space-y-4">
            <div className="text-center">
              <UserPlus className="h-12 w-12 text-pleb-600 mx-auto mb-3" />
              <h3 className="text-lg font-semibold text-gray-900 mb-2">Generate New Account</h3>
              <p className="text-gray-600 text-sm mb-4">
                Create a brand new Nostr identity. Perfect for newcomers to the protocol.
              </p>
            </div>
            <div className="bg-yellow-50 border border-yellow-200 rounded-md p-3 mb-4">
              <div className="flex items-start">
                <AlertCircle className="h-4 w-4 text-yellow-500 mr-2 mt-0.5" />
                <div className="text-sm text-yellow-700">
                  <p className="font-medium mb-1">Important Security Notice:</p>
                  <p>Your private key will be generated and shown once. Make sure to save it securely!</p>
                </div>
              </div>
            </div>
            <button
              onClick={handleGenerateAccount}
              disabled={loading}
              className="w-full btn-primary disabled:opacity-50 disabled:cursor-not-allowed"
            >
              {loading ? 'Generating...' : 'Generate New Nostr Account'}
            </button>
            <div className="text-xs text-gray-500 space-y-1">
              <p>✓ Fresh identity</p>
              <p>✓ Full control</p>
              <p>⚠️ Backup your keys!</p>
            </div>
          </div>
        )}
      </div>

      {/* Help Section */}
      <div className="mt-6 pt-4 border-t border-gray-200">
        <details className="text-sm text-gray-600">
          <summary className="cursor-pointer font-medium text-gray-700 hover:text-gray-900">
            Need help getting started?
          </summary>
          <div className="mt-2 space-y-2">
            <p><strong>New to Nostr?</strong> Generate a new account to get started immediately.</p>
            <p><strong>Have an extension?</strong> Use Alby, nos2x, or Flamingo for the best experience.</p>
            <p><strong>Have an npub?</strong> Enter it to connect in read-only mode.</p>
            <p><strong>Need an extension?</strong> Visit <a href="https://github.com/nostr-protocol/nips/blob/master/07.md" target="_blank" rel="noopener noreferrer" className="text-pleb-600 hover:underline">NIP-07 extensions</a></p>
          </div>
        </details>
      </div>
    </div>
  )
}
