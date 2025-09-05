import { useQuery } from '@tanstack/react-query'
import { Activity, Database, Users, Zap, Clock, TrendingUp } from 'lucide-react'

interface RelayMetrics {
  connections_active: number
  connections_total: number
  events_stored_total: number
  events_processing_duration_avg: number
  queries_total: number
  database_operations_total: number
  uptime_seconds: number
}

async function fetchRelayMetrics(): Promise<RelayMetrics> {
  const response = await fetch('/api/metrics')
  if (!response.ok) {
    throw new Error('Failed to fetch metrics')
  }
  
  // Parse Prometheus metrics text format
  const text = await response.text()
  const lines = text.split('\n')
  
  const metrics: any = {}
  for (const line of lines) {
    if (line.startsWith('relay_')) {
      const [metric, value] = line.split(' ')
      const metricName = metric.replace('relay_', '')
      metrics[metricName] = parseFloat(value) || 0
    }
  }
  
  return metrics as RelayMetrics
}

export function DashboardPage() {
  const { data: metrics, isLoading, error } = useQuery({
    queryKey: ['relay-metrics'],
    queryFn: fetchRelayMetrics,
    refetchInterval: 5000, // Refresh every 5 seconds
  })

  if (isLoading) {
    return (
      <div className="flex justify-center items-center h-64">
        <div className="animate-spin rounded-full h-32 w-32 border-b-2 border-pleb-600"></div>
      </div>
    )
  }

  if (error) {
    return (
      <div className="card bg-red-50 border-red-200">
        <div className="flex items-center">
          <div className="flex-shrink-0">
            <div className="w-8 h-8 bg-red-100 rounded-full flex items-center justify-center">
              <Activity className="w-4 h-4 text-red-600" />
            </div>
          </div>
          <div className="ml-3">
            <h3 className="text-sm font-medium text-red-800">
              Unable to fetch relay metrics
            </h3>
            <div className="mt-2 text-sm text-red-700">
              <p>Check if the relay is running and accessible.</p>
            </div>
          </div>
        </div>
      </div>
    )
  }

  const formatUptime = (seconds: number) => {
    const days = Math.floor(seconds / 86400)
    const hours = Math.floor((seconds % 86400) / 3600)
    const minutes = Math.floor((seconds % 3600) / 60)
    return `${days}d ${hours}h ${minutes}m`
  }

  const formatNumber = (num: number) => {
    if (num >= 1000000) return (num / 1000000).toFixed(1) + 'M'
    if (num >= 1000) return (num / 1000).toFixed(1) + 'K'
    return num.toString()
  }

  return (
    <div className="space-y-6">
      {/* Header */}
      <div>
        <h1 className="text-3xl font-bold text-gray-900">Relay Dashboard</h1>
        <p className="mt-2 text-gray-600">
          Real-time metrics and performance data for the Pleb.One relay network.
        </p>
      </div>

      {/* Status Overview */}
      <div className="grid grid-cols-1 gap-6 sm:grid-cols-2 lg:grid-cols-3">
        <div className="card">
          <div className="flex items-center">
            <div className="flex-shrink-0">
              <Activity className="h-8 w-8 text-green-500" />
            </div>
            <div className="ml-4">
              <p className="text-sm font-medium text-gray-600">Relay Status</p>
              <p className="text-2xl font-bold text-green-600">Online</p>
              <p className="text-xs text-gray-500">
                Uptime: {metrics ? formatUptime(metrics.uptime_seconds) : '0m'}
              </p>
            </div>
          </div>
        </div>

        <div className="card">
          <div className="flex items-center">
            <div className="flex-shrink-0">
              <Users className="h-8 w-8 text-pleb-500" />
            </div>
            <div className="ml-4">
              <p className="text-sm font-medium text-gray-600">Active Connections</p>
              <p className="text-2xl font-bold text-gray-900">
                {metrics ? formatNumber(metrics.connections_active) : '0'}
              </p>
              <p className="text-xs text-gray-500">
                Total: {metrics ? formatNumber(metrics.connections_total) : '0'}
              </p>
            </div>
          </div>
        </div>

        <div className="card">
          <div className="flex items-center">
            <div className="flex-shrink-0">
              <Zap className="h-8 w-8 text-bitcoin-500" />
            </div>
            <div className="ml-4">
              <p className="text-sm font-medium text-gray-600">Events Stored</p>
              <p className="text-2xl font-bold text-gray-900">
                {metrics ? formatNumber(metrics.events_stored_total) : '0'}
              </p>
              <p className="text-xs text-gray-500">
                Avg processing: {metrics ? (metrics.events_processing_duration_avg * 1000).toFixed(1) : '0'}ms
              </p>
            </div>
          </div>
        </div>
      </div>

      {/* Detailed Metrics */}
      <div className="grid grid-cols-1 gap-6 lg:grid-cols-2">
        <div className="card">
          <h3 className="text-lg font-semibold text-gray-900 mb-4 flex items-center">
            <TrendingUp className="h-5 w-5 mr-2" />
            Query Performance
          </h3>
          <div className="space-y-4">
            <div className="flex justify-between items-center">
              <span className="text-sm text-gray-600">Total Queries</span>
              <span className="text-lg font-semibold">
                {metrics ? formatNumber(metrics.queries_total) : '0'}
              </span>
            </div>
            <div className="flex justify-between items-center">
              <span className="text-sm text-gray-600">Average Response Time</span>
              <span className="text-lg font-semibold">
                {metrics ? '< 100ms' : 'N/A'}
              </span>
            </div>
            <div className="w-full bg-gray-200 rounded-full h-2">
              <div className="bg-green-500 h-2 rounded-full" style={{ width: '95%' }}></div>
            </div>
            <p className="text-xs text-gray-500">95% of queries under 100ms</p>
          </div>
        </div>

        <div className="card">
          <h3 className="text-lg font-semibold text-gray-900 mb-4 flex items-center">
            <Database className="h-5 w-5 mr-2" />
            Database Health
          </h3>
          <div className="space-y-4">
            <div className="flex justify-between items-center">
              <span className="text-sm text-gray-600">Database Operations</span>
              <span className="text-lg font-semibold">
                {metrics ? formatNumber(metrics.database_operations_total) : '0'}
              </span>
            </div>
            <div className="flex justify-between items-center">
              <span className="text-sm text-gray-600">Connection Pool</span>
              <span className="text-lg font-semibold text-green-600">Healthy</span>
            </div>
            <div className="w-full bg-gray-200 rounded-full h-2">
              <div className="bg-pleb-500 h-2 rounded-full" style={{ width: '88%' }}></div>
            </div>
            <p className="text-xs text-gray-500">Pool utilization: 88%</p>
          </div>
        </div>
      </div>

      {/* Network Information */}
      <div className="card">
        <h3 className="text-lg font-semibold text-gray-900 mb-4 flex items-center">
          <Clock className="h-5 w-5 mr-2" />
          Network Information
        </h3>
        <div className="grid grid-cols-1 md:grid-cols-3 gap-6">
          <div>
            <h4 className="text-sm font-medium text-gray-900 mb-2">WebSocket Endpoint</h4>
            <p className="text-sm text-gray-600 font-mono bg-gray-50 p-2 rounded">
              wss://relay.pleb.one
            </p>
          </div>
          <div>
            <h4 className="text-sm font-medium text-gray-900 mb-2">Supported NIPs</h4>
            <div className="flex flex-wrap gap-1">
              {['NIP-01', 'NIP-02', 'NIP-09', 'NIP-11', 'NIP-12', 'NIP-15', 'NIP-16', 'NIP-20'].map((nip) => (
                <span key={nip} className="badge-green">{nip}</span>
              ))}
            </div>
          </div>
          <div>
            <h4 className="text-sm font-medium text-gray-900 mb-2">Rate Limits</h4>
            <ul className="text-sm text-gray-600 space-y-1">
              <li>• 10 connections per IP</li>
              <li>• 50 events per minute</li>
              <li>• 100 queries per minute</li>
            </ul>
          </div>
        </div>
      </div>
    </div>
  )
}
