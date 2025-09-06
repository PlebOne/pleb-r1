import { Key, UserCircle, Shield, QrCode, Fingerprint, Lock, LogOut, Copy, Eye, EyeOff } from 'lucide-react'
import { useState } from 'react'
import { useAuth } from '../contexts/AuthContext'
import { LoginComponent } from '../components/LoginComponent'

export function IdentityPage() {
  const { user, isAuthenticated, logout } = useAuth()
  const [showPrivateKey, setShowPrivateKey] = useState(false)

  // If not authenticated, show login component
  if (!isAuthenticated) {
    return (
      <div className="space-y-8">
        <div className="text-center">
          <h1 className="text-4xl font-bold text-gray-900">Identity Management</h1>
          <p className="mt-4 text-lg text-gray-600 max-w-2xl mx-auto">
            Connect with your Nostr identity to access the Pleb-R1 community features.
          </p>
        </div>
        <LoginComponent />
      </div>
    )
  }

  const copyToClipboard = (text: string, label: string) => {
    navigator.clipboard.writeText(text)
    alert(`${label} copied to clipboard!`)
  }

  const verificationMethods = [
    {
      name: "NIP-05 Verification",
      description: "Link your identity to a domain you control",
      icon: Shield,
      status: "Available",
      difficulty: "Easy"
    },
    {
      name: "Lightning Verification", 
      description: "Verify ownership through Lightning Network transactions",
      icon: QrCode,
      status: "Coming Soon",
      difficulty: "Medium"
    }
  ]

  const securityFeatures = [
    {
      title: "Key Security",
      description: "Your keys are stored securely in your browser",
      icon: Fingerprint,
      enabled: true
    },
    {
      title: "Connection Method",
      description: user?.privkey ? "Generated Account" : "Extension/npub",
      icon: Lock,
      enabled: true
    }
  ]

  return (
    <div className="space-y-8">
      {/* Header */}
      <div className="text-center">
        <h1 className="text-4xl font-bold text-gray-900">Identity Management</h1>
        <p className="mt-4 text-lg text-gray-600 max-w-2xl mx-auto">
          Manage your Nostr identity and verification methods on the Pleb-R1 network.
        </p>
      </div>

      {/* Current Identity */}
      <div>
        <div className="flex justify-between items-center mb-6">
          <h2 className="text-2xl font-bold text-gray-900">Your Identity</h2>
          <button 
            onClick={logout}
            className="btn-secondary flex items-center space-x-2"
          >
            <LogOut className="h-4 w-4" />
            <span>Logout</span>
          </button>
        </div>
        
        <div className="card">
          <div className="flex justify-between items-start">
            <div className="flex-1">
              <div className="flex items-center mb-6">
                {user?.profile?.picture ? (
                  <img 
                    src={user.profile.picture} 
                    alt="Profile" 
                    className="h-16 w-16 rounded-full mr-4 object-cover"
                    onError={(e) => {
                      (e.target as HTMLImageElement).style.display = 'none'
                    }}
                  />
                ) : (
                  <UserCircle className="h-16 w-16 text-pleb-600 mr-4" />
                )}
                <div className="flex-1">
                  <h3 className="text-xl font-semibold text-gray-900 mb-1">
                    {user?.profile?.name || 'Anonymous'}
                  </h3>
                  {user?.profile?.nip05 && (
                    <div className="flex items-center mb-2">
                      <Shield className="h-4 w-4 text-green-600 mr-1" />
                      <span className="text-sm text-green-600">{user.profile.nip05}</span>
                    </div>
                  )}
                  <span className="badge-green">Connected</span>
                </div>
              </div>

              {/* Profile Information */}
              {(user?.profile?.about || user?.profile?.website || user?.profile?.lud16 || user?.profile?.lud06) && (
                <div className="mb-6 p-4 bg-gray-50 rounded-lg">
                  <h4 className="text-sm font-medium text-gray-700 mb-3">Profile Information</h4>
                  <div className="space-y-2">
                    {user?.profile?.about && (
                      <div>
                        <span className="text-xs text-gray-500 uppercase tracking-wider">About</span>
                        <p className="text-sm text-gray-700 mt-1">{user.profile.about}</p>
                      </div>
                    )}
                    {user?.profile?.website && (
                      <div>
                        <span className="text-xs text-gray-500 uppercase tracking-wider">Website</span>
                        <p className="text-sm text-gray-700 mt-1">
                          <a href={user.profile.website} target="_blank" rel="noopener noreferrer" className="text-pleb-600 hover:underline">
                            {user.profile.website}
                          </a>
                        </p>
                      </div>
                    )}
                    {(user?.profile?.lud16 || user?.profile?.lud06) && (
                      <div>
                        <span className="text-xs text-gray-500 uppercase tracking-wider">Lightning Address</span>
                        <p className="text-sm text-gray-700 mt-1">{user.profile.lud16 || user.profile.lud06}</p>
                      </div>
                    )}
                  </div>
                </div>
              )}
              
              <div className="space-y-3">
                <div>
                  <label className="block text-sm font-medium text-gray-700 mb-1">
                    Public Key (npub)
                  </label>
                  <div className="flex items-center space-x-2">
                    <code className="flex-1 text-sm bg-gray-100 p-2 rounded font-mono break-all">
                      {user?.npub}
                    </code>
                    <button
                      onClick={() => copyToClipboard(user?.npub || '', 'Public key')}
                      className="btn-secondary p-2"
                      title="Copy npub"
                    >
                      <Copy className="h-4 w-4" />
                    </button>
                  </div>
                </div>

                {user?.privkey && (
                  <div>
                    <label className="block text-sm font-medium text-gray-700 mb-1">
                      Private Key (nsec) - Keep Safe!
                    </label>
                    <div className="flex items-center space-x-2">
                      <code className="flex-1 text-sm bg-red-50 p-2 rounded font-mono break-all border border-red-200">
                        {showPrivateKey ? user.privkey : '•'.repeat(64)}
                      </code>
                      <button
                        onClick={() => setShowPrivateKey(!showPrivateKey)}
                        className="btn-secondary p-2"
                        title={showPrivateKey ? "Hide private key" : "Show private key"}
                      >
                        {showPrivateKey ? <EyeOff className="h-4 w-4" /> : <Eye className="h-4 w-4" />}
                      </button>
                      {showPrivateKey && (
                        <button
                          onClick={() => copyToClipboard(user.privkey || '', 'Private key')}
                          className="btn-secondary p-2"
                          title="Copy nsec"
                        >
                          <Copy className="h-4 w-4" />
                        </button>
                      )}
                    </div>
                    <p className="text-xs text-red-600 mt-1">
                      ⚠️ Never share your private key. Store it safely!
                    </p>
                  </div>
                )}
              </div>
            </div>
          </div>
        </div>
      </div>
      {/* Verification Methods */}
      <div>
        <h2 className="text-2xl font-bold text-gray-900 mb-6">Verification Methods</h2>
        <div className="grid grid-cols-1 md:grid-cols-2 gap-6">
          {verificationMethods.map((method, index) => {
            const Icon = method.icon
            return (
              <div key={index} className="card hover:shadow-md transition-shadow">
                <div className="flex items-center mb-4">
                  <Icon className="h-8 w-8 text-pleb-600 mr-3" />
                  <div>
                    <h3 className="text-lg font-semibold text-gray-900">{method.name}</h3>
                    <div className="flex items-center space-x-2">
                      <span className={`text-xs px-2 py-1 rounded-full ${
                        method.status === 'Available' ? 'bg-green-100 text-green-800' :
                        'bg-gray-100 text-gray-800'
                      }`}>
                        {method.status}
                      </span>
                      <span className="text-xs text-gray-500">{method.difficulty}</span>
                    </div>
                  </div>
                </div>
                <p className="text-gray-600 mb-4">{method.description}</p>
                <button 
                  className={`w-full ${
                    method.status === 'Available' ? 'btn-primary' : 
                    'btn-secondary opacity-50 cursor-not-allowed'
                  }`}
                  disabled={method.status !== 'Available'}
                >
                  {method.status === 'Available' ? 'Setup Verification' : 'Coming Soon'}
                </button>
              </div>
            )
          })}
        </div>
      </div>

      {/* Security Features */}
      <div>
        <h2 className="text-2xl font-bold text-gray-900 mb-6">Security Status</h2>
        <div className="grid grid-cols-1 md:grid-cols-2 gap-6">
          {securityFeatures.map((feature, index) => {
            const Icon = feature.icon
            return (
              <div key={index} className="card">
                <div className="flex items-center justify-between mb-4">
                  <div className="flex items-center">
                    <Icon className="h-6 w-6 text-pleb-600 mr-3" />
                    <h3 className="text-lg font-semibold text-gray-900">{feature.title}</h3>
                  </div>
                  <div className={`w-12 h-6 ${feature.enabled ? 'bg-green-500' : 'bg-gray-300'} 
                    rounded-full relative`}>
                    <div className={`w-5 h-5 bg-white rounded-full absolute top-0.5 transition-transform 
                      ${feature.enabled ? 'transform translate-x-6' : 'translate-x-0.5'}`}></div>
                  </div>
                </div>
                <p className="text-gray-600 text-sm">{feature.description}</p>
              </div>
            )
          })}
        </div>
      </div>

      {/* How Identity Works */}
      <div className="card bg-gradient-to-r from-pleb-600 to-pleb-700 text-white">
        <h2 className="text-2xl font-bold mb-6 text-center">How Nostr Identity Works</h2>
        <div className="grid grid-cols-1 md:grid-cols-4 gap-6">
          <div className="text-center">
            <div className="w-12 h-12 bg-white bg-opacity-20 rounded-full flex items-center justify-center mx-auto mb-4">
              <Key className="h-6 w-6" />
            </div>
            <h3 className="text-lg font-semibold mb-2">Generate Keys</h3>
            <p className="text-pleb-100 text-sm">
              Create a cryptographic key pair that serves as your unique identity.
            </p>
          </div>
          <div className="text-center">
            <div className="w-12 h-12 bg-white bg-opacity-20 rounded-full flex items-center justify-center mx-auto mb-4">
              <Shield className="h-6 w-6" />
            </div>
            <h3 className="text-lg font-semibold mb-2">Verify</h3>
            <p className="text-pleb-100 text-sm">
              Link your identity to external verification methods for trust.
            </p>
          </div>
          <div className="text-center">
            <div className="w-12 h-12 bg-white bg-opacity-20 rounded-full flex items-center justify-center mx-auto mb-4">
              <UserCircle className="h-6 w-6" />
            </div>
            <h3 className="text-lg font-semibold mb-2">Publish</h3>
            <p className="text-pleb-100 text-sm">
              Share your verified identity across the Nostr network.
            </p>
          </div>
          <div className="text-center">
            <div className="w-12 h-12 bg-white bg-opacity-20 rounded-full flex items-center justify-center mx-auto mb-4">
              <Lock className="h-6 w-6" />
            </div>
            <h3 className="text-lg font-semibold mb-2">Control</h3>
            <p className="text-pleb-100 text-sm">
              Maintain full ownership and control of your digital identity.
            </p>
          </div>
        </div>
      </div>

      {/* Security Best Practices */}
      <div className="card border-yellow-200 bg-yellow-50">
        <div className="flex items-start">
          <Shield className="h-6 w-6 text-yellow-600 mr-3 mt-1" />
          <div>
            <h3 className="text-lg font-semibold text-yellow-800 mb-2">Security Best Practices</h3>
            <ul className="space-y-2 text-sm text-yellow-700">
              <li>• Keep your private keys secure and never share them</li>
              <li>• Use browser extensions for enhanced security</li>
              <li>• Backup your keys in a secure location</li>
              <li>• Enable verification methods when available</li>
              <li>• Use separate keys for different purposes</li>
            </ul>
          </div>
        </div>
      </div>
    </div>
  )
}
