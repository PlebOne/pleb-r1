import { Book, Code, Play, ExternalLink, Download, Terminal } from 'lucide-react'

export function EducationPage() {
  const tutorials = [
    {
      title: "WTF is Nostr? (For Normies)",
      description: "A no-BS explanation of why Nostr exists and why Big Tech hates it. Spoiler: They can't control it.",
      level: "Pleb",
      duration: "10 min read",
      icon: Book,
      tags: ["Protocol", "Awakening"],
    },
    {
      title: "Escape the Cage: Your First Nostr Client",
      description: "Break free from Zuck's surveillance empire. Step-by-step guide to sovereign communication.",
      level: "Pleb", 
      duration: "15 min",
      icon: Play,
      tags: ["Freedom", "Setup"],
    },
    {
      title: "Become a Chad: Self-Host Your Relay",
      description: "Don't just use the network, BE the network. Deploy your own Pleb-R1 relay and join the rebellion.",
      level: "Chad",
      duration: "30 min",
      icon: Terminal,
      tags: ["Self-hosting", "Rebellion"],
    },
    {
      title: "NIPs Deep Dive: The Pleb Protocols",
      description: "Understanding Nostr Implementation Possibilities. From basic events to zaps and everything in between.",
      level: "Gigachad",
      duration: "20 min read",
      icon: Code,
      tags: ["NIPs", "Development"],
    },
    {
      title: "Orange Pill Lightning Integration",
      description: "Stack sats while you relay. Integrate Lightning payments and watch fiat minds explode.",
      level: "Gigachad",
      duration: "45 min",
      icon: ExternalLink,
      tags: ["Lightning", "Bitcoin"],
    },
    {
      title: "Join the Pleb Army: Contributing",
      description: "Help build the future of free speech. Contribute to Pleb-R1 and stick it to the censors.",
      level: "Chad",
      duration: "25 min",
      icon: Download,
      tags: ["Open Source", "Revolution"],
    },
    {
      title: "Database Pilled: PostgreSQL Mastery",
      description: "Scale your relay like a pro. Advanced database optimization for handling millions of plebs.",
      level: "Gigachad",
      duration: "60 min",
      icon: Terminal,
      tags: ["Database", "Performance"],
    },
    {
      title: "Fiat Destroyer: Monetization Strategies",
      description: "Turn your relay into a Bitcoin machine. Paid relays, premium features, and pleb economics.",
      level: "Chad",
      duration: "35 min",
      icon: ExternalLink,
      tags: ["Business", "Bitcoin"],
    },
    {
      title: "Opsec for Paranoid Plebs",
      description: "Protect yourself from glowies. Security, privacy, and operational security for relay operators.",
      level: "Gigachad",
      duration: "40 min",
      icon: Book,
      tags: ["Security", "Privacy"],
    },
    {
      title: "Tor & I2P Integration",
      description: "Run your relay through anonymity networks. Because surveillance is cringe.",
      level: "Gigachad", 
      duration: "50 min",
      icon: Terminal,
      tags: ["Privacy", "Tor"],
    },
    {
      title: "Backup & Disaster Recovery",
      description: "Never lose a pleb's data. Comprehensive backup strategies for relay operators.",
      level: "Chad",
      duration: "30 min",
      icon: Download,
      tags: ["Backup", "Operations"],
    },
    {
      title: "Monitoring & Alerting Setup",
      description: "Watch your relay like a hawk. Prometheus, Grafana, and custom alerting for based operators.",
      level: "Chad",
      duration: "45 min", 
      icon: Code,
      tags: ["Monitoring", "DevOps"],
    },
  ]

  const quickStart = {
    connect: `// Escape Big Tech surveillance - Connect to Pleb-R1!
const relay = relayInit('wss://relay.pleb-r1.com')
relay.on('connect', () => {
  console.log('🎯 Connected to the rebellion!')
  console.log('Zuck in shambles 📉')
})
await relay.connect()`,
    
    publish: `// Speak freely without algorithm manipulation
const event = {
  kind: 1,
  created_at: Math.floor(Date.now() / 1000),
  tags: [['r', 'pleb-r1.com']], // Support the rebellion!
  content: 'GM plebs! Just escaped the Twitter gulag 🚁',
}
const signedEvent = await window.nostr.signEvent(event)
relay.publish(signedEvent) // No shadow banning here!`,
    
    subscribe: `// Actually see ALL messages (revolutionary concept!)
const sub = relay.sub([
  {
    kinds: [1], // Text notes
    '#r': ['pleb-r1.com'], // Our based community
    limit: 50
  }
])
sub.on('event', (event) => {
  console.log('Uncensored truth:', event.content)
  // No "community guidelines" violations here 😎
})`
  }

  const getLevelColor = (level: string) => {
    switch (level) {
      case 'Pleb': return 'badge-green'
      case 'Chad': return 'badge-yellow'
      case 'Gigachad': return 'badge-red'
      default: return 'badge-gray'
    }
  }

  return (
    <div className="space-y-8">
      {/* Header */}
      <div className="text-center">
        <h1 className="text-4xl font-bold text-gray-900">🎓 Pleb University</h1>
        <p className="mt-4 text-lg text-gray-600 max-w-2xl mx-auto">
          Learn to escape Big Tech tyranny. Master Nostr, Bitcoin, and self-sovereignty. 
          Because freedom isn't free, but these tutorials are! 😎
        </p>
      </div>

      {/* Quick Start Section */}
      <div className="card">
        <h2 className="text-2xl font-bold text-gray-900 mb-6">⚡ Quick Start: Join the Rebellion</h2>
        <div className="grid grid-cols-1 lg:grid-cols-3 gap-6">
          <div>
            <h3 className="text-lg font-semibold text-gray-900 mb-3">1. 🏃‍♂️ Escape the Matrix</h3>
            <pre className="bg-gray-900 text-gray-100 p-4 rounded-lg text-sm overflow-x-auto">
              <code>{quickStart.connect}</code>
            </pre>
          </div>
          <div>
            <h3 className="text-lg font-semibold text-gray-900 mb-3">2. 🚀 Launch Your Voice</h3>
            <pre className="bg-gray-900 text-gray-100 p-4 rounded-lg text-sm overflow-x-auto">
              <code>{quickStart.publish}</code>
            </pre>
          </div>
          <div>
            <h3 className="text-lg font-semibold text-gray-900 mb-3">3. 👂 Hear the Truth</h3>
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
            Essential Pleb Resources
          </h3>
          <ul className="space-y-3">
            <li>
              <a href="https://github.com/nostr-protocol/nostr" className="flex items-center text-pleb-600 hover:text-pleb-700">
                <ExternalLink className="h-4 w-4 mr-2" />
                Official Nostr Protocol (The Sacred Texts)
              </a>
            </li>
            <li>
              <a href="https://github.com/aljazceru/awesome-nostr" className="flex items-center text-pleb-600 hover:text-pleb-700">
                <ExternalLink className="h-4 w-4 mr-2" />
                Awesome Nostr - Curated Freedom Tools
              </a>
            </li>
            <li>
              <a href="https://nostrgraph.com" className="flex items-center text-pleb-600 hover:text-pleb-700">
                <ExternalLink className="h-4 w-4 mr-2" />
                Nostr Network Visualizer (See the Web of Freedom)
              </a>
            </li>
            <li>
              <a href="#" className="flex items-center text-pleb-600 hover:text-pleb-700">
                <Download className="h-4 w-4 mr-2" />
                Pleb-R1 Docker Stack (One-Click Freedom)
              </a>
            </li>
            <li>
              <a href="https://21ideas.org" className="flex items-center text-pleb-600 hover:text-pleb-700">
                <ExternalLink className="h-4 w-4 mr-2" />
                21 Ideas - Bitcoin Education for Plebs
              </a>
            </li>
          </ul>
        </div>

        <div className="card">
          <h3 className="text-xl font-semibold text-gray-900 mb-4 flex items-center">
            <Code className="h-5 w-5 mr-2" />
            Chad Developer Arsenal
          </h3>
          <ul className="space-y-3">
            <li>
              <a href="https://github.com/nbd-wtf/nostr-tools" className="flex items-center text-pleb-600 hover:text-pleb-700">
                <ExternalLink className="h-4 w-4 mr-2" />
                nostr-tools - JavaScript Library (Based AF)
              </a>
            </li>
            <li>
              <a href="https://github.com/rust-nostr/nostr" className="flex items-center text-pleb-600 hover:text-pleb-700">
                <ExternalLink className="h-4 w-4 mr-2" />
                rust-nostr - Rust Implementation (Memory Safe Rebellion)
              </a>
            </li>
            <li>
              <a href="https://websocketking.com" className="flex items-center text-pleb-600 hover:text-pleb-700">
                <ExternalLink className="h-4 w-4 mr-2" />
                WebSocket Testing Tool (Debug Like a Pro)
              </a>
            </li>
            <li>
              <a href="#" className="flex items-center text-pleb-600 hover:text-pleb-700">
                <Terminal className="h-4 w-4 mr-2" />
                Pleb-R1 CLI Tools (Command Line Chad Mode)
              </a>
            </li>
            <li>
              <a href="https://github.com/lnbits/lnbits" className="flex items-center text-pleb-600 hover:text-pleb-700">
                <ExternalLink className="h-4 w-4 mr-2" />
                LNbits - Lightning Wallet Stack (Orange Pill Everything)
              </a>
            </li>
          </ul>
        </div>
      </div>

      {/* Community Section */}
      <div className="card bg-gradient-to-r from-pleb-600 to-pleb-700 text-white">
        <div className="text-center">
          <h2 className="text-2xl font-bold mb-4">🤝 Join the Pleb Resistance</h2>
          <p className="text-pleb-100 mb-6 max-w-2xl mx-auto">
            Connect with based plebs building the future of free speech. No safe spaces, 
            just raw truth and revolutionary technology. Stack sats, learn Nostr, resist tyranny.
          </p>
          <div className="flex flex-col sm:flex-row gap-4 justify-center">
            <a href="#" className="bg-white text-pleb-700 hover:bg-gray-100 font-medium py-3 px-6 rounded-lg transition-colors">
              🚁 Join Telegram
            </a>
            <a href="#" className="border border-white text-white hover:bg-white hover:text-pleb-700 font-medium py-3 px-6 rounded-lg transition-colors">
              🤙 Follow on Nostr
            </a>
            <a href="#" className="border border-white text-white hover:bg-white hover:text-pleb-700 font-medium py-3 px-6 rounded-lg transition-colors">
              📢 Matrix Chat
            </a>
          </div>
        </div>
      </div>
    </div>
  )
}
