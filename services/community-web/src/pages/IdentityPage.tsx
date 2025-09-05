import { Key, UserCircle, Shield, QrCode, Fingerprint, Lock } from 'lucide-react'

export function IdentityPage() {
  const identities = [
    {
      name: "alice@pleb.one",
      npub: "npub1alice123...",
      verified: true,
      created: "2024-01-15",
      keyType: "Ed25519",
      status: "active"
    },
    {
      name: "alice-dev@pleb.one", 
      npub: "npub1alicedev456...",
      verified: false,
      created: "2024-02-20",
      keyType: "secp256k1",
      status: "pending"
    }
  ]

  const verificationMethods = [
    {
      name: "NIP-05 Verification",
      description: "Link your identity to a domain you control",
      icon: Shield,
      status: "Available",
      difficulty: "Easy"
    },
    {
      name: "GitHub Verification",
      description: "Verify your identity with your GitHub account",
      icon: UserCircle,
      status: "Available", 
      difficulty: "Easy"
    },
    {
      name: "Hardware Key",
      description: "Use a hardware security key for verification",
      icon: Key,
      status: "Coming Soon",
      difficulty: "Advanced"
    },
    {
      name: "Lightning Verification",
      description: "Verify ownership through Lightning Network transactions",
      icon: QrCode,
      status: "Beta",
      difficulty: "Medium"
    }
  ]

  const securityFeatures = [
    {
      title: "Key Rotation",
      description: "Regularly rotate your keys for enhanced security",
      icon: Fingerprint,
      enabled: true
    },
    {
      title: "Multi-Signature",
      description: "Require multiple signatures for important actions",
      icon: Lock,
      enabled: false
    },
    {
      title: "Recovery Keys",
      description: "Secure backup keys for account recovery",
      icon: Shield,
      enabled: true
    }
  ]

  return (
    <div className="space-y-8">
      {/* Header */}
      <div className="text-center">
        <h1 className="text-4xl font-bold text-gray-900">Identity Management</h1>
        <p className="mt-4 text-lg text-gray-600 max-w-2xl mx-auto">
          Secure, verifiable identity on the Nostr protocol. Take control of your 
          digital presence with cryptographic proof and community verification.
        </p>
      </div>

      {/* Current Identities */}
      <div>
        <div className="flex justify-between items-center mb-6">
          <h2 className="text-2xl font-bold text-gray-900">Your Identities</h2>
          <button className="btn-primary">Create New Identity</button>
        </div>
        <div className="space-y-4">
          {identities.map((identity, index) => (
            <div key={index} className="card">
              <div className="flex justify-between items-start">
                <div className="flex-1">
                  <div className="flex items-center mb-2">
                    <h3 className="text-lg font-semibold text-gray-900 mr-3">
                      {identity.name}
                    </h3>
                    {identity.verified && (
                      <span className="badge-green">Verified</span>
                    )}
                    {!identity.verified && (
                      <span className="badge-yellow">Pending</span>
                    )}
                  </div>
                  <p className="text-sm text-gray-600 mb-2">
                    <span className="font-mono">{identity.npub}</span>
                  </p>
                  <div className="flex space-x-4 text-sm text-gray-500">
                    <span>Created: {identity.created}</span>
                    <span>Key: {identity.keyType}</span>
                    <span>Status: {identity.status}</span>
                  </div>
                </div>
                <div className="flex space-x-2">
                  <button className="btn-secondary">Manage</button>
                  <button className="text-gray-400 hover:text-gray-600">
                    <Key className="h-4 w-4" />
                  </button>
                </div>
              </div>
            </div>
          ))}
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
                        method.status === 'Beta' ? 'bg-yellow-100 text-yellow-800' :
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
                    method.status === 'Beta' ? 'btn-secondary' : 
                    'btn-secondary opacity-50 cursor-not-allowed'
                  }`}
                  disabled={method.status === 'Coming Soon'}
                >
                  {method.status === 'Available' ? 'Setup Verification' :
                   method.status === 'Beta' ? 'Try Beta' : 'Coming Soon'}
                </button>
              </div>
            )
          })}
        </div>
      </div>

      {/* Security Features */}
      <div>
        <h2 className="text-2xl font-bold text-gray-900 mb-6">Security Features</h2>
        <div className="grid grid-cols-1 md:grid-cols-3 gap-6">
          {securityFeatures.map((feature, index) => {
            const Icon = feature.icon
            return (
              <div key={index} className="card">
                <div className="flex items-center justify-between mb-4">
                  <div className="flex items-center">
                    <Icon className="h-6 w-6 text-pleb-600 mr-3" />
                    <h3 className="text-lg font-semibold text-gray-900">{feature.title}</h3>
                  </div>
                  <div className={`w-12 h-6 ${feature.enabled ? 'bg-pleb-600' : 'bg-gray-300'} 
                    rounded-full relative cursor-pointer`}>
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
              <li>• Use hardware security keys for high-value identities</li>
              <li>• Regularly rotate your keys and update verification methods</li>
              <li>• Enable multiple verification methods for redundancy</li>
              <li>• Backup your recovery keys in a secure location</li>
            </ul>
          </div>
        </div>
      </div>
    </div>
  )
}
