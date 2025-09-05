import { Book, Code, Play, ExternalLink, Download, Terminal } from 'lucide-react'

export function EducationPage() {
  const tutorials = [
    {
      title: "What is Nostr?",
      description: "Learn the basics of the Nostr protocol and why it matters for decentralized communication.",
      level: "Beginner",
      duration: "10 min read",
      icon: Book,
      tags: ["Protocol", "Basics"],
    },
    {
      title: "Setting Up Your First Nostr Client",
      description: "Step-by-step guide to connect to Nostr relays and start publishing events.",
      level: "Beginner", 
      duration: "15 min",
      icon: Play,
      tags: ["Client", "Setup"],
    },
    {
      title: "Self-Hosting a Nostr Relay",
      description: "Deploy your own Pleb.One relay instance using Docker and contribute to the network.",
      level: "Intermediate",
      duration: "30 min",
      icon: Terminal,
      tags: ["Self-hosting", "Docker"],
    },
    {
      title: "Understanding NIPs",
      description: "Dive deep into Nostr Implementation Possibilities and how they extend the protocol.",
      level: "Advanced",
      duration: "20 min read",
      icon: Code,
      tags: ["NIPs", "Development"],
    },
    {
      title: "Lightning Integration",
      description: "Learn how to integrate Lightning Network payments with your Nostr applications.",
      level: "Advanced",
      duration: "45 min",
      icon: ExternalLink,
      tags: ["Lightning", "Payments"],
    },
    {
      title: "Contributing to Pleb.One",
      description: "How to contribute to the open source Pleb.One relay and community tools.",
      level: "Intermediate",
      duration: "25 min",
      icon: Download,
      tags: ["Open Source", "Contributing"],
    },
  ]

  const quickStart = {
    connect: `// Connect to Pleb.One relay
const relay = relayInit('wss://relay.pleb.one')
relay.on('connect', () => {
  console.log('Connected to Pleb.One!')
})
await relay.connect()`,
    
    publish: `// Publish an event
const event = {
  kind: 1,
  created_at: Math.floor(Date.now() / 1000),
  tags: [],
  content: 'Hello from Pleb.One!',
}
const signedEvent = await window.nostr.signEvent(event)
relay.publish(signedEvent)`,
    
    subscribe: `// Subscribe to events
const sub = relay.sub([
  {
    kinds: [1],
    limit: 10
  }
])
sub.on('event', (event) => {
  console.log('Received event:', event)
})`
  }

  const getLevelColor = (level: string) => {
    switch (level) {
      case 'Beginner': return 'badge-green'
      case 'Intermediate': return 'badge-yellow'
      case 'Advanced': return 'badge-red'
      default: return 'badge-gray'
    }
  }

  return (
    <div className="space-y-8">
      {/* Header */}
      <div className="text-center">
        <h1 className="text-4xl font-bold text-gray-900">Educational Hub</h1>
        <p className="mt-4 text-lg text-gray-600 max-w-2xl mx-auto">
          Learn Nostr, Bitcoin, and self-sovereign technology. From basics to advanced topics,
          all open source and community-driven.
        </p>
      </div>

      {/* Quick Start Section */}
      <div className="card">
        <h2 className="text-2xl font-bold text-gray-900 mb-6">Quick Start Code Examples</h2>
        <div className="grid grid-cols-1 lg:grid-cols-3 gap-6">
          <div>
            <h3 className="text-lg font-semibold text-gray-900 mb-3">1. Connect to Relay</h3>
            <pre className="bg-gray-900 text-gray-100 p-4 rounded-lg text-sm overflow-x-auto">
              <code>{quickStart.connect}</code>
            </pre>
          </div>
          <div>
            <h3 className="text-lg font-semibold text-gray-900 mb-3">2. Publish Event</h3>
            <pre className="bg-gray-900 text-gray-100 p-4 rounded-lg text-sm overflow-x-auto">
              <code>{quickStart.publish}</code>
            </pre>
          </div>
          <div>
            <h3 className="text-lg font-semibold text-gray-900 mb-3">3. Subscribe to Events</h3>
            <pre className="bg-gray-900 text-gray-100 p-4 rounded-lg text-sm overflow-x-auto">
              <code>{quickStart.subscribe}</code>
            </pre>
          </div>
        </div>
      </div>

      {/* Tutorials Grid */}
      <div>
        <h2 className="text-2xl font-bold text-gray-900 mb-6">Tutorials & Guides</h2>
        <div className="grid grid-cols-1 md:grid-cols-2 lg:grid-cols-3 gap-6">
          {tutorials.map((tutorial, index) => {
            const Icon = tutorial.icon
            return (
              <div key={index} className="card hover:shadow-md transition-shadow cursor-pointer">
                <div className="flex items-center mb-4">
                  <Icon className="h-6 w-6 text-pleb-600 mr-3" />
                  <span className={`${getLevelColor(tutorial.level)}`}>
                    {tutorial.level}
                  </span>
                </div>
                <h3 className="text-lg font-semibold text-gray-900 mb-2">
                  {tutorial.title}
                </h3>
                <p className="text-gray-600 mb-4 text-sm">
                  {tutorial.description}
                </p>
                <div className="flex items-center justify-between">
                  <span className="text-sm text-gray-500">{tutorial.duration}</span>
                  <div className="flex gap-1">
                    {tutorial.tags.map((tag) => (
                      <span key={tag} className="text-xs bg-gray-100 text-gray-600 px-2 py-1 rounded">
                        {tag}
                      </span>
                    ))}
                  </div>
                </div>
              </div>
            )
          })}
        </div>
      </div>

      {/* Resources Section */}
      <div className="grid grid-cols-1 lg:grid-cols-2 gap-8">
        <div className="card">
          <h3 className="text-xl font-semibold text-gray-900 mb-4 flex items-center">
            <Book className="h-5 w-5 mr-2" />
            Essential Resources
          </h3>
          <ul className="space-y-3">
            <li>
              <a href="https://github.com/nostr-protocol/nostr" className="flex items-center text-pleb-600 hover:text-pleb-700">
                <ExternalLink className="h-4 w-4 mr-2" />
                Official Nostr Protocol Specification
              </a>
            </li>
            <li>
              <a href="https://github.com/aljazceru/awesome-nostr" className="flex items-center text-pleb-600 hover:text-pleb-700">
                <ExternalLink className="h-4 w-4 mr-2" />
                Awesome Nostr - Curated Resources
              </a>
            </li>
            <li>
              <a href="https://nostr.com" className="flex items-center text-pleb-600 hover:text-pleb-700">
                <ExternalLink className="h-4 w-4 mr-2" />
                Nostr.com - Directory of Clients & Relays
              </a>
            </li>
            <li>
              <a href="#" className="flex items-center text-pleb-600 hover:text-pleb-700">
                <Download className="h-4 w-4 mr-2" />
                Pleb.One Docker Compose Setup
              </a>
            </li>
          </ul>
        </div>

        <div className="card">
          <h3 className="text-xl font-semibold text-gray-900 mb-4 flex items-center">
            <Code className="h-5 w-5 mr-2" />
            Development Tools
          </h3>
          <ul className="space-y-3">
            <li>
              <a href="https://github.com/nbd-wtf/nostr-tools" className="flex items-center text-pleb-600 hover:text-pleb-700">
                <ExternalLink className="h-4 w-4 mr-2" />
                nostr-tools - JavaScript Library
              </a>
            </li>
            <li>
              <a href="https://github.com/rust-nostr/nostr" className="flex items-center text-pleb-600 hover:text-pleb-700">
                <ExternalLink className="h-4 w-4 mr-2" />
                rust-nostr - Rust Implementation
              </a>
            </li>
            <li>
              <a href="https://websocketking.com" className="flex items-center text-pleb-600 hover:text-pleb-700">
                <ExternalLink className="h-4 w-4 mr-2" />
                WebSocket Testing Tool
              </a>
            </li>
            <li>
              <a href="#" className="flex items-center text-pleb-600 hover:text-pleb-700">
                <Terminal className="h-4 w-4 mr-2" />
                Pleb.One CLI Tools
              </a>
            </li>
          </ul>
        </div>
      </div>

      {/* Community Section */}
      <div className="card bg-gradient-to-r from-pleb-600 to-pleb-700 text-white">
        <div className="text-center">
          <h2 className="text-2xl font-bold mb-4">Join the Learning Community</h2>
          <p className="text-pleb-100 mb-6 max-w-2xl mx-auto">
            Connect with other plebs learning Nostr and Bitcoin. Share knowledge, 
            ask questions, and build together.
          </p>
          <div className="flex flex-col sm:flex-row gap-4 justify-center">
            <a href="#" className="bg-white text-pleb-700 hover:bg-gray-100 font-medium py-3 px-6 rounded-lg transition-colors">
              Join Discord
            </a>
            <a href="#" className="border border-white text-white hover:bg-white hover:text-pleb-700 font-medium py-3 px-6 rounded-lg transition-colors">
              Follow on Nostr
            </a>
          </div>
        </div>
      </div>
    </div>
  )
}
