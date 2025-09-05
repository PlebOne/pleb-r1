import { Users, MessageCircle, Vote, Shield, Heart, Star } from 'lucide-react'

export function CommunityPage() {
  const councils = [
    {
      name: "Technical Council",
      description: "Reviews technical proposals and protocol improvements",
      members: 12,
      proposals: 8,
      icon: Shield,
    },
    {
      name: "Moderation Council", 
      description: "Handles content moderation and community guidelines",
      members: 8,
      proposals: 3,
      icon: Users,
    },
    {
      name: "Treasury Council",
      description: "Manages community funds and resource allocation",
      members: 6,
      proposals: 5,
      icon: Heart,
    },
  ]

  const proposals = [
    {
      id: 1,
      title: "Implement NIP-50 Search Functionality",
      author: "alice@pleb.one",
      council: "Technical",
      votes: { for: 156, against: 23, abstain: 12 },
      status: "active",
      timeLeft: "3 days",
    },
    {
      id: 2,
      title: "Update Community Moderation Guidelines",
      author: "bob@pleb.one", 
      council: "Moderation",
      votes: { for: 89, against: 45, abstain: 8 },
      status: "active",
      timeLeft: "1 week",
    },
    {
      id: 3,
      title: "Fund Open Source Development Bounties",
      author: "charlie@pleb.one",
      council: "Treasury", 
      votes: { for: 234, against: 12, abstain: 15 },
      status: "passed",
      timeLeft: "Completed",
    },
  ]

  const getStatusColor = (status: string) => {
    switch (status) {
      case 'active': return 'badge-yellow'
      case 'passed': return 'badge-green'
      case 'rejected': return 'badge-red'
      default: return 'badge-gray'
    }
  }

  return (
    <div className="space-y-8">
      {/* Header */}
      <div className="text-center">
        <h1 className="text-4xl font-bold text-gray-900">Community Governance</h1>
        <p className="mt-4 text-lg text-gray-600 max-w-2xl mx-auto">
          Democratic decision making for the Pleb.One network. Your voice matters in 
          shaping the future of community-owned infrastructure.
        </p>
      </div>

      {/* Pleb Councils */}
      <div>
        <h2 className="text-2xl font-bold text-gray-900 mb-6">Pleb Councils</h2>
        <div className="grid grid-cols-1 md:grid-cols-3 gap-6">
          {councils.map((council, index) => {
            const Icon = council.icon
            return (
              <div key={index} className="card hover:shadow-md transition-shadow">
                <div className="flex items-center mb-4">
                  <Icon className="h-8 w-8 text-pleb-600 mr-3" />
                  <h3 className="text-lg font-semibold text-gray-900">{council.name}</h3>
                </div>
                <p className="text-gray-600 mb-4">{council.description}</p>
                <div className="flex justify-between text-sm text-gray-500">
                  <span>{council.members} members</span>
                  <span>{council.proposals} active proposals</span>
                </div>
                <button className="mt-4 w-full btn-secondary">
                  Join Council
                </button>
              </div>
            )
          })}
        </div>
      </div>

      {/* Active Proposals */}
      <div>
        <div className="flex justify-between items-center mb-6">
          <h2 className="text-2xl font-bold text-gray-900">Community Proposals</h2>
          <button className="btn-primary">Submit Proposal</button>
        </div>
        <div className="space-y-4">
          {proposals.map((proposal) => (
            <div key={proposal.id} className="card">
              <div className="flex justify-between items-start mb-4">
                <div>
                  <h3 className="text-lg font-semibold text-gray-900 mb-1">
                    {proposal.title}
                  </h3>
                  <p className="text-sm text-gray-600">
                    by {proposal.author} • {proposal.council} Council
                  </p>
                </div>
                <div className="flex items-center space-x-3">
                  <span className={getStatusColor(proposal.status)}>
                    {proposal.status}
                  </span>
                  <span className="text-sm text-gray-500">{proposal.timeLeft}</span>
                </div>
              </div>
              
              {/* Voting Results */}
              <div className="mb-4">
                <div className="flex justify-between text-sm text-gray-600 mb-2">
                  <span>Votes</span>
                  <span>
                    {proposal.votes.for + proposal.votes.against + proposal.votes.abstain} total
                  </span>
                </div>
                <div className="w-full bg-gray-200 rounded-full h-3">
                  <div className="flex h-3 rounded-full overflow-hidden">
                    <div 
                      className="bg-green-500"
                      style={{ width: `${(proposal.votes.for / (proposal.votes.for + proposal.votes.against + proposal.votes.abstain)) * 100}%` }}
                    ></div>
                    <div 
                      className="bg-red-500"
                      style={{ width: `${(proposal.votes.against / (proposal.votes.for + proposal.votes.against + proposal.votes.abstain)) * 100}%` }}
                    ></div>
                    <div 
                      className="bg-gray-400"
                      style={{ width: `${(proposal.votes.abstain / (proposal.votes.for + proposal.votes.against + proposal.votes.abstain)) * 100}%` }}
                    ></div>
                  </div>
                </div>
                <div className="flex justify-between text-xs text-gray-500 mt-1">
                  <span>👍 {proposal.votes.for}</span>
                  <span>👎 {proposal.votes.against}</span>
                  <span>🤷 {proposal.votes.abstain}</span>
                </div>
              </div>
              
              {proposal.status === 'active' && (
                <div className="flex space-x-3">
                  <button className="flex-1 bg-green-600 hover:bg-green-700 text-white py-2 px-4 rounded-lg">
                    Vote For
                  </button>
                  <button className="flex-1 bg-red-600 hover:bg-red-700 text-white py-2 px-4 rounded-lg">
                    Vote Against
                  </button>
                  <button className="flex-1 btn-secondary">
                    Abstain
                  </button>
                </div>
              )}
            </div>
          ))}
        </div>
      </div>

      {/* Community Stats */}
      <div className="grid grid-cols-1 md:grid-cols-2 lg:grid-cols-4 gap-6">
        <div className="card text-center">
          <Users className="h-8 w-8 text-pleb-600 mx-auto mb-2" />
          <p className="text-2xl font-bold text-gray-900">1,234</p>
          <p className="text-sm text-gray-600">Community Members</p>
        </div>
        <div className="card text-center">
          <Vote className="h-8 w-8 text-pleb-600 mx-auto mb-2" />
          <p className="text-2xl font-bold text-gray-900">16</p>
          <p className="text-sm text-gray-600">Active Proposals</p>
        </div>
        <div className="card text-center">
          <MessageCircle className="h-8 w-8 text-pleb-600 mx-auto mb-2" />
          <p className="text-2xl font-bold text-gray-900">89%</p>
          <p className="text-sm text-gray-600">Participation Rate</p>
        </div>
        <div className="card text-center">
          <Star className="h-8 w-8 text-pleb-600 mx-auto mb-2" />
          <p className="text-2xl font-bold text-gray-900">4.8</p>
          <p className="text-sm text-gray-600">Community Rating</p>
        </div>
      </div>

      {/* How It Works */}
      <div className="card bg-gradient-to-r from-pleb-600 to-pleb-700 text-white">
        <h2 className="text-2xl font-bold mb-6 text-center">How Community Governance Works</h2>
        <div className="grid grid-cols-1 md:grid-cols-3 gap-6">
          <div className="text-center">
            <div className="w-12 h-12 bg-white bg-opacity-20 rounded-full flex items-center justify-center mx-auto mb-4">
              <span className="text-xl font-bold">1</span>
            </div>
            <h3 className="text-lg font-semibold mb-2">Propose</h3>
            <p className="text-pleb-100 text-sm">
              Community members submit proposals for network improvements, 
              policy changes, or resource allocation.
            </p>
          </div>
          <div className="text-center">
            <div className="w-12 h-12 bg-white bg-opacity-20 rounded-full flex items-center justify-center mx-auto mb-4">
              <span className="text-xl font-bold">2</span>
            </div>
            <h3 className="text-lg font-semibold mb-2">Discuss</h3>
            <p className="text-pleb-100 text-sm">
              Proposals are reviewed by the relevant council and discussed 
              openly with community feedback and input.
            </p>
          </div>
          <div className="text-center">
            <div className="w-12 h-12 bg-white bg-opacity-20 rounded-full flex items-center justify-center mx-auto mb-4">
              <span className="text-xl font-bold">3</span>
            </div>
            <h3 className="text-lg font-semibold mb-2">Vote</h3>
            <p className="text-pleb-100 text-sm">
              Community members vote democratically, with transparent 
              results and immediate implementation of passed proposals.
            </p>
          </div>
        </div>
      </div>
    </div>
  )
}
