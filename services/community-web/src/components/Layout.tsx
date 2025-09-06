import { ReactNode } from 'react'
import { Link, useLocation } from 'react-router-dom'
import { Activity, Book, Users, User, Home, Github, Zap, Rss } from 'lucide-react'

interface LayoutProps {
  children: ReactNode
}

export function Layout({ children }: LayoutProps) {
  const location = useLocation()

  const navigation = [
    { name: 'Home', href: '/', icon: Home },
    { name: 'Dashboard', href: '/dashboard', icon: Activity },
    { name: 'Feed', href: '/feed', icon: Rss },
    { name: 'Education', href: '/education', icon: Book },
    { name: 'Community', href: '/community', icon: Users },
    { name: 'Identity', href: '/identity', icon: User },
  ]

  return (
    <div className="min-h-screen bg-gray-50">
      {/* Header */}
      <header className="bg-white shadow-sm border-b border-gray-200">
        <div className="max-w-7xl mx-auto px-4 sm:px-6 lg:px-8">
          <div className="flex justify-between items-center h-16">
            {/* Logo */}
            <div className="flex items-center">
              <Link to="/" className="flex items-center space-x-2">
                <Zap className="h-8 w-8 text-pleb-600" />
                <span className="text-xl font-bold text-gray-900">Pleb.One</span>
              </Link>
            </div>

            {/* Navigation */}
            <nav className="hidden md:flex space-x-8">
              {navigation.map((item) => {
                const Icon = item.icon
                const isActive = location.pathname === item.href
                return (
                  <Link
                    key={item.name}
                    to={item.href}
                    className={`flex items-center space-x-1 px-3 py-2 rounded-md text-sm font-medium transition-colors ${
                      isActive
                        ? 'text-pleb-600 bg-pleb-50'
                        : 'text-gray-700 hover:text-pleb-600 hover:bg-gray-100'
                    }`}
                  >
                    <Icon className="h-4 w-4" />
                    <span>{item.name}</span>
                  </Link>
                )
              })}
            </nav>

            {/* Actions */}
            <div className="flex items-center space-x-4">
              <a
                href="https://github.com/your-repo/NrelayOne"
                target="_blank"
                rel="noopener noreferrer"
                className="text-gray-500 hover:text-gray-700"
              >
                <Github className="h-5 w-5" />
              </a>
              <button className="btn-primary">Connect Wallet</button>
            </div>
          </div>
        </div>
      </header>

      {/* Main Content */}
      <main className="max-w-7xl mx-auto py-6 px-4 sm:px-6 lg:px-8">
        {children}
      </main>

      {/* Footer */}
      <footer className="bg-white border-t border-gray-200 mt-16">
        <div className="max-w-7xl mx-auto py-8 px-4 sm:px-6 lg:px-8">
          <div className="grid grid-cols-1 md:grid-cols-4 gap-8">
            <div>
              <h3 className="text-sm font-semibold text-gray-900 uppercase tracking-wider">
                🤝 Pleb Army
              </h3>
              <ul className="mt-4 space-y-4">
                <li>
                  <a href="#" className="text-base text-gray-500 hover:text-gray-900">
                    🚁 Telegram (No Censorship)
                  </a>
                </li>
                <li>
                  <a href="#" className="text-base text-gray-500 hover:text-gray-900">
                    🤙 Nostr (Obviously)
                  </a>
                </li>
                <li>
                  <a href="#" className="text-base text-gray-500 hover:text-gray-900">
                    📢 Matrix (For the Paranoid)
                  </a>
                </li>
              </ul>
            </div>
            <div>
              <h3 className="text-sm font-semibold text-gray-900 uppercase tracking-wider">
                🛠️ Freedom Tech
              </h3>
              <ul className="mt-4 space-y-4">
                <li>
                  <Link to="/education" className="text-base text-gray-500 hover:text-gray-900">
                    📚 Pleb University
                  </Link>
                </li>
                <li>
                  <a href="#" className="text-base text-gray-500 hover:text-gray-900">
                    🔧 API Docs (For Chads)
                  </a>
                </li>
                <li>
                  <a href="https://pleb.one" className="text-base text-gray-500 hover:text-gray-900">
                    🌐 The Mothership
                  </a>
                </li>
              </ul>
            </div>
            <div>
              <h3 className="text-sm font-semibold text-gray-900 uppercase tracking-wider">
                🏴‍☠️ Open Source Rebellion
              </h3>
              <ul className="mt-4 space-y-4">
                <li>
                  <a href="https://github.com/PlebOne/pleb-r1" className="text-base text-gray-500 hover:text-gray-900">
                    🐙 GitHub (Star Us!)
                  </a>
                </li>
                <li>
                  <a href="#" className="text-base text-gray-500 hover:text-gray-900">
                    🚀 Contributing (Join the Fight)
                  </a>
                </li>
                <li>
                  <a href="#" className="text-base text-gray-500 hover:text-gray-900">
                    ⚖️ License (MIT = Freedom)
                  </a>
                </li>
              </ul>
            </div>
            <div>
              <h3 className="text-sm font-semibold text-gray-900 uppercase tracking-wider">
                💰 Support the Cause
              </h3>
              <ul className="mt-4 space-y-4">
                <li>
                  <a href="#" className="text-base text-gray-500 hover:text-gray-900">
                    ⚡ Zap Us Sats
                  </a>
                </li>
                <li>
                  <a href="#" className="text-base text-gray-500 hover:text-gray-900">
                    📊 Relay Status
                  </a>
                </li>
                <li>
                  <a href="#" className="text-base text-gray-500 hover:text-gray-900">
                    📞 Contact (No Karens)
                  </a>
                </li>
              </ul>
            </div>
          </div>
          
          {/* Built by section */}
          <div className="mt-8 border-t border-gray-200 pt-6">
            <div className="text-center">
              <p className="text-base text-gray-600 mb-2">
                🚀 <strong>Built by Freedom Tech Rebels</strong>
              </p>
              <p className="text-sm text-gray-500 mb-4">
                Pleb-R1 is crafted with love, caffeine, and a healthy disdain for Big Tech 
                censorship by the mad scientists at <a href="https://pleb.one" className="text-pleb-600 hover:text-pleb-700">pleb.one</a> 🧪
              </p>
              <p className="text-xs text-gray-400 italic mb-4">
                "Why use one relay when you can help destroy the surveillance state?" - Ancient Pleb Wisdom
              </p>
            </div>
          </div>

          {/* Copyright and links */}
          <div className="border-t border-gray-200 pt-6">
            <div className="flex flex-col md:flex-row justify-between items-center">
              <p className="text-base text-gray-400 mb-4 md:mb-0">
                © 2025 <a href="https://pleb.one" className="text-pleb-600 hover:text-pleb-700">pleb.one</a> - Because freedom isn't free, but our software is 🏴‍☠️
              </p>
              <div className="flex space-x-6 text-sm">
                <a href="https://github.com/PlebOne" className="text-gray-400 hover:text-gray-500">🐙 GitHub</a>
                <a href="https://pleb.one" className="text-gray-400 hover:text-gray-500">🌐 Website</a>
                <a href="#" className="text-gray-400 hover:text-gray-500">📚 Help</a>
                <a href="#" className="text-gray-400 hover:text-gray-500">⚖️ Legal Mumbo Jumbo</a>
              </div>
            </div>
            <p className="text-center text-xs text-gray-400 mt-4">
              Made with ❤️, ☕, and a sprinkle of anarchist pixie dust ✨<br/>
              <em>"Side effects may include: sovereignty, privacy, and the urge to orange-pill friends" 🟠</em>
            </p>
          </div>
        </div>
      </footer>
    </div>
  )
}
