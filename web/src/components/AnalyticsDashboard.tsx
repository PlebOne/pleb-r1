'use client';

import { useState, useEffect } from 'react';
import { Card, CardContent, CardDescription, CardHeader, CardTitle } from '@/components/ui/card';
import { Button } from '@/components/ui/button';
import { Tabs, TabsContent, TabsList, TabsTrigger } from '@/components/ui/tabs';
import { Badge } from '@/components/ui/badge';
import { Alert, AlertDescription } from '@/components/ui/alert';
import { 
  LineChart, 
  Line, 
  XAxis, 
  YAxis, 
  CartesianGrid, 
  Tooltip, 
  ResponsiveContainer,
  BarChart,
  Bar,
  PieChart,
  Pie,
  Cell
} from 'recharts';
import { Download, RefreshCw, Activity, Users, Zap, HardDrive } from 'lucide-react';

interface RealtimeMetrics {
  active_connections: number;
  events_per_second: number;
  total_events_today: number;
  unique_users_today: number;
  memory_usage_mb: number;
  cpu_usage_percent: number;
  disk_usage_mb: number;
  top_event_types: Array<{ event_type: string; count: number }>;
  recent_errors: Array<{
    timestamp: string;
    event_type: string;
    error_code: string;
    client_id: string;
    message: string;
  }>;
}

interface TrafficReport {
  date: string;
  total_events: number;
  unique_users: number;
  event_types: Record<string, number>;
  avg_response_ms: number;
}

const COLORS = ['#0088FE', '#00C49F', '#FFBB28', '#FF8042', '#8884d8'];

export default function AnalyticsDashboard() {
  const [realtimeMetrics, setRealtimeMetrics] = useState<RealtimeMetrics | null>(null);
  const [trafficData, setTrafficData] = useState<TrafficReport[]>([]);
  const [timeseriesData, setTimeseriesData] = useState<Array<{ timestamp: string; events: number }>>([]);
  const [loading, setLoading] = useState(true);
  const [error, setError] = useState<string | null>(null);
  const [selectedTimeRange, setSelectedTimeRange] = useState('24h');

  useEffect(() => {
    fetchRealtimeMetrics();
    fetchTrafficReport();
    fetchTimeseriesData();
    
    // Set up real-time updates
    const interval = setInterval(fetchRealtimeMetrics, 5000); // Update every 5 seconds
    return () => clearInterval(interval);
  }, [selectedTimeRange]);

  const fetchRealtimeMetrics = async () => {
    try {
      const response = await fetch('/api/v1/analytics/dashboard/realtime');
      if (!response.ok) throw new Error('Failed to fetch realtime metrics');
      const data = await response.json();
      setRealtimeMetrics(data);
    } catch (err) {
      setError(err instanceof Error ? err.message : 'Unknown error');
    }
  };

  const fetchTrafficReport = async () => {
    try {
      const endDate = new Date().toISOString().split('T')[0];
      const startDate = new Date(Date.now() - 7 * 24 * 60 * 60 * 1000).toISOString().split('T')[0];
      
      const response = await fetch(`/api/v1/analytics/reports/traffic?start_date=${startDate}&end_date=${endDate}&granularity=day`);
      if (!response.ok) throw new Error('Failed to fetch traffic report');
      const data = await response.json();
      setTrafficData(data);
    } catch (err) {
      setError(err instanceof Error ? err.message : 'Unknown error');
    }
  };

  const fetchTimeseriesData = async () => {
    try {
      const hours = selectedTimeRange === '24h' ? 24 : selectedTimeRange === '7d' ? 168 : 720;
      const response = await fetch(`/api/v1/analytics/timeseries/events?hours=${hours}`);
      if (!response.ok) throw new Error('Failed to fetch timeseries data');
      const data = await response.json();
      setTimeseriesData(data);
      setLoading(false);
    } catch (err) {
      setError(err instanceof Error ? err.message : 'Unknown error');
      setLoading(false);
    }
  };

  const exportData = async (format: 'csv' | 'json') => {
    try {
      const endDate = new Date().toISOString().split('T')[0];
      const startDate = new Date(Date.now() - 7 * 24 * 60 * 60 * 1000).toISOString().split('T')[0];
      
      const response = await fetch(`/api/v1/analytics/export/${format}?start_date=${startDate}&end_date=${endDate}`);
      if (!response.ok) throw new Error(`Failed to export ${format.toUpperCase()}`);
      
      const blob = await response.blob();
      const url = window.URL.createObjectURL(blob);
      const a = document.createElement('a');
      a.style.display = 'none';
      a.href = url;
      a.download = `nrelay_analytics_${startDate}_to_${endDate}.${format}`;
      document.body.appendChild(a);
      a.click();
      window.URL.revokeObjectURL(url);
    } catch (err) {
      setError(err instanceof Error ? err.message : 'Export failed');
    }
  };

  if (loading) {
    return (
      <div className="flex items-center justify-center h-64">
        <RefreshCw className="h-8 w-8 animate-spin" />
        <span className="ml-2">Loading analytics...</span>
      </div>
    );
  }

  if (error) {
    return (
      <Alert className="mb-6">
        <AlertDescription>
          Error loading analytics: {error}
          <Button 
            variant="outline" 
            size="sm" 
            className="ml-2"
            onClick={() => window.location.reload()}
          >
            Retry
          </Button>
        </AlertDescription>
      </Alert>
    );
  }

  return (
    <div className="space-y-6">
      {/* Header */}
      <div className="flex justify-between items-center">
        <div>
          <h1 className="text-3xl font-bold">Analytics Dashboard</h1>
          <p className="text-muted-foreground">Monitor your relay performance and traffic</p>
        </div>
        <div className="flex gap-2">
          <Button variant="outline" onClick={() => exportData('csv')}>
            <Download className="h-4 w-4 mr-2" />
            Export CSV
          </Button>
          <Button variant="outline" onClick={() => exportData('json')}>
            <Download className="h-4 w-4 mr-2" />
            Export JSON
          </Button>
          <Button onClick={fetchRealtimeMetrics}>
            <RefreshCw className="h-4 w-4 mr-2" />
            Refresh
          </Button>
        </div>
      </div>

      {/* Real-time Metrics */}
      {realtimeMetrics && (
        <div className="grid grid-cols-1 md:grid-cols-2 lg:grid-cols-4 gap-4">
          <Card>
            <CardHeader className="flex flex-row items-center justify-between space-y-0 pb-2">
              <CardTitle className="text-sm font-medium">Active Connections</CardTitle>
              <Activity className="h-4 w-4 text-muted-foreground" />
            </CardHeader>
            <CardContent>
              <div className="text-2xl font-bold">{realtimeMetrics.active_connections}</div>
              <p className="text-xs text-muted-foreground">
                {realtimeMetrics.events_per_second.toFixed(1)} events/sec
              </p>
            </CardContent>
          </Card>

          <Card>
            <CardHeader className="flex flex-row items-center justify-between space-y-0 pb-2">
              <CardTitle className="text-sm font-medium">Events Today</CardTitle>
              <Zap className="h-4 w-4 text-muted-foreground" />
            </CardHeader>
            <CardContent>
              <div className="text-2xl font-bold">{realtimeMetrics.total_events_today.toLocaleString()}</div>
              <p className="text-xs text-muted-foreground">
                {realtimeMetrics.unique_users_today} unique users
              </p>
            </CardContent>
          </Card>

          <Card>
            <CardHeader className="flex flex-row items-center justify-between space-y-0 pb-2">
              <CardTitle className="text-sm font-medium">System Load</CardTitle>
              <HardDrive className="h-4 w-4 text-muted-foreground" />
            </CardHeader>
            <CardContent>
              <div className="text-2xl font-bold">{realtimeMetrics.cpu_usage_percent.toFixed(1)}%</div>
              <p className="text-xs text-muted-foreground">
                {(realtimeMetrics.memory_usage_mb).toLocaleString()} MB RAM
              </p>
            </CardContent>
          </Card>

          <Card>
            <CardHeader className="flex flex-row items-center justify-between space-y-0 pb-2">
              <CardTitle className="text-sm font-medium">Storage</CardTitle>
              <HardDrive className="h-4 w-4 text-muted-foreground" />
            </CardHeader>
            <CardContent>
              <div className="text-2xl font-bold">{(realtimeMetrics.disk_usage_mb / 1024).toFixed(1)} GB</div>
              <p className="text-xs text-muted-foreground">
                Disk usage
              </p>
            </CardContent>
          </Card>
        </div>
      )}

      <Tabs value={selectedTimeRange} onValueChange={setSelectedTimeRange} className="space-y-4">
        <TabsList>
          <TabsTrigger value="24h">Last 24 Hours</TabsTrigger>
          <TabsTrigger value="7d">Last 7 Days</TabsTrigger>
          <TabsTrigger value="30d">Last 30 Days</TabsTrigger>
        </TabsList>

        <TabsContent value={selectedTimeRange} className="space-y-4">
          {/* Event Timeline */}
          <Card>
            <CardHeader>
              <CardTitle>Event Timeline</CardTitle>
              <CardDescription>Events processed over time</CardDescription>
            </CardHeader>
            <CardContent>
              <ResponsiveContainer width="100%" height={300}>
                <LineChart data={timeseriesData}>
                  <CartesianGrid strokeDasharray="3 3" />
                  <XAxis 
                    dataKey="timestamp" 
                    tick={{ fontSize: 12 }}
                    tickFormatter={(value) => new Date(value).toLocaleTimeString([], { hour: '2-digit', minute: '2-digit' })}
                  />
                  <YAxis />
                  <Tooltip 
                    labelFormatter={(value) => new Date(value).toLocaleString()}
                    formatter={(value: number) => [value, 'Events']}
                  />
                  <Line type="monotone" dataKey="events" stroke="#8884d8" strokeWidth={2} />
                </LineChart>
              </ResponsiveContainer>
            </CardContent>
          </Card>

          {/* Traffic Report */}
          <div className="grid grid-cols-1 lg:grid-cols-2 gap-4">
            <Card>
              <CardHeader>
                <CardTitle>Daily Traffic</CardTitle>
                <CardDescription>Events and users per day</CardDescription>
              </CardHeader>
              <CardContent>
                <ResponsiveContainer width="100%" height={300}>
                  <BarChart data={trafficData}>
                    <CartesianGrid strokeDasharray="3 3" />
                    <XAxis dataKey="date" tick={{ fontSize: 12 }} />
                    <YAxis />
                    <Tooltip />
                    <Bar dataKey="total_events" fill="#8884d8" name="Events" />
                    <Bar dataKey="unique_users" fill="#82ca9d" name="Users" />
                  </BarChart>
                </ResponsiveContainer>
              </CardContent>
            </Card>

            {/* Event Types Distribution */}
            {realtimeMetrics && (
              <Card>
                <CardHeader>
                  <CardTitle>Event Types</CardTitle>
                  <CardDescription>Distribution of event types (last hour)</CardDescription>
                </CardHeader>
                <CardContent>
                  <ResponsiveContainer width="100%" height={300}>
                    <PieChart>
                      <Pie
                        data={realtimeMetrics.top_event_types}
                        cx="50%"
                        cy="50%"
                        labelLine={false}
                        label={({ event_type, percent }) => `${event_type}: ${(percent * 100).toFixed(0)}%`}
                        outerRadius={80}
                        fill="#8884d8"
                        dataKey="count"
                        nameKey="event_type"
                      >
                        {realtimeMetrics.top_event_types.map((entry, index) => (
                          <Cell key={`cell-${index}`} fill={COLORS[index % COLORS.length]} />
                        ))}
                      </Pie>
                      <Tooltip />
                    </PieChart>
                  </ResponsiveContainer>
                </CardContent>
              </Card>
            )}
          </div>

          {/* Recent Errors */}
          {realtimeMetrics && realtimeMetrics.recent_errors.length > 0 && (
            <Card>
              <CardHeader>
                <CardTitle>Recent Errors</CardTitle>
                <CardDescription>Latest errors and issues</CardDescription>
              </CardHeader>
              <CardContent>
                <div className="space-y-2">
                  {realtimeMetrics.recent_errors.map((error, index) => (
                    <div key={index} className="flex items-center justify-between p-3 border rounded">
                      <div className="flex-1">
                        <div className="flex items-center gap-2">
                          <Badge variant="destructive">{error.error_code}</Badge>
                          <span className="font-medium">{error.event_type}</span>
                        </div>
                        <p className="text-sm text-muted-foreground mt-1">{error.message}</p>
                      </div>
                      <div className="text-right text-xs text-muted-foreground">
                        <div>{new Date(error.timestamp).toLocaleString()}</div>
                        <div>Client: {error.client_id || 'Unknown'}</div>
                      </div>
                    </div>
                  ))}
                </div>
              </CardContent>
            </Card>
          )}
        </TabsContent>
      </Tabs>
    </div>
  );
}
