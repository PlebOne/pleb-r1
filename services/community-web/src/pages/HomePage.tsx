import { Link } from 'react-router-dom'
import { Activity, Users, Book, Zap, ArrowRight, Github, Shield, Globe } from 'lucide-react'

export function HomePage() {
  return (
    <div className="space-y-16">
      {/* Hero Section */}
      <div className="text-center">
        <div className="flex justify-center mb-6">
          <Zap className="h-16 w-16 text-pleb-600" />
        </div>
        <h1 className="text-4xl font-bold text-gray-900 sm:text-6xl">
          By Plebs, For Plebs
        </h1>
        <p className="mt-6 text-lg leading-8 text-gray-600 max-w-2xl mx-auto">
          Community-owned Nostr infrastructure that empowers the grassroots. 
          Open source, transparent, and built for sovereignty. Watch the live feed 
          of posts flowing through the relay in real-time.
        </p>
        <div className="mt-10 flex items-center justify-center gap-x-6">
          <Link to="/feed" className="btn-primary">
            View Live Feed
          </Link>
          <Link to="/dashboard" className="btn-secondary">
            View Dashboard
          </Link>
          <Link to="/education" className="flex items-center text-sm font-semibold leading-6 text-gray-900">
            Learn More <ArrowRight className="ml-1 h-4 w-4" />
          </Link>
        </div>
      </div>

      {/* Status Cards */}
      <div className="grid grid-cols-1 gap-6 sm:grid-cols-2 lg:grid-cols-4">
        <div className="card">
          <div className="flex items-center">
            <Activity className="h-8 w-8 text-green-500" />
            <div className="ml-4">
              <p className="text-sm font-medium text-gray-600">Relay Status</p>
              <p className="text-2xl font-bold text-gray-900">Online</p>
            </div>
          </div>
        </div>
        
        <div className="card">
          <div className="flex items-center">
            <Users className="h-8 w-8 text-pleb-500" />
            <div className="ml-4">
              <p className="text-sm font-medium text-gray-600">Active Connections</p>
              <p className="text-2xl font-bold text-gray-900">1,234</p>
            </div>
          </div>
        </div>
        
        <div className="card">
          <div className="flex items-center">
            <Zap className="h-8 w-8 text-bitcoin-500" />
            <div className="ml-4">
              <p className="text-sm font-medium text-gray-600">Events Today</p>
              <p className="text-2xl font-bold text-gray-900">56.7k</p>
            </div>
          </div>
        </div>
        
        <div className="card">
          <div className="flex items-center">
            <Globe className="h-8 w-8 text-purple-500" />
            <div className="ml-4">
              <p className="text-sm font-medium text-gray-600">Network Health</p>
              <p className="text-2xl font-bold text-gray-900">99.9%</p>
            </div>
          </div>
        </div>
      </div>

      {/* Features */}
      <div className="grid grid-cols-1 gap-8 lg:grid-cols-3">
        <div className="card">
          <Shield className="h-12 w-12 text-pleb-600 mb-4" />
          <h3 className="text-lg font-semibold text-gray-900 mb-2">
            Community Owned
          </h3>
          <p className="text-gray-600">
            100% open source with transparent governance. No corporate overlords, 
            just community-driven development and decision making.
          </p>
        </div>
        
        <div className="card">
          <Book className="h-12 w-12 text-pleb-600 mb-4" />
          <h3 className="text-lg font-semibold text-gray-900 mb-2">
            Educational First
          </h3>
          <p className="text-gray-600">
            Learn Nostr and Bitcoin while using the service. Comprehensive guides, 
            tutorials, and documentation to empower your journey.
          </p>
        </div>
        
        <div className="card">
          <Github className="h-12 w-12 text-pleb-600 mb-4" />
          <h3 className="text-lg font-semibold text-gray-900 mb-2">
            Radical Transparency
          </h3>
          <p className="text-gray-600">
            Public metrics, open development, and clear roadmap. Inspect the code, 
            contribute features, and shape the future together.
          </p>
        </div>
      </div>

      {/* Call to Action */}
      <div className="card bg-gradient-to-r from-pleb-600 to-pleb-700 text-white">
        <div className="text-center">
          <h2 className="text-3xl font-bold mb-4">Ready to Join the Revolution?</h2>
          <p className="text-pleb-100 mb-6 max-w-2xl mx-auto">
            Start using community-owned Nostr infrastructure today. No signups, 
            no KYC, just sovereignty and freedom.
          </p>
          <div className="flex flex-col sm:flex-row gap-4 justify-center">
            <Link to="/identity" className="bg-white text-pleb-700 hover:bg-gray-100 font-medium py-3 px-6 rounded-lg transition-colors">
              Get @pleb.one Identity
            </Link>
            <Link to="/education" className="border border-white text-white hover:bg-white hover:text-pleb-700 font-medium py-3 px-6 rounded-lg transition-colors">
              Learn How to Self-Host
            </Link>
          </div>
        </div>
      </div>
    </div>
  )
}
