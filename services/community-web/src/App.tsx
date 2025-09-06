import { Routes, Route } from 'react-router-dom'
import { AuthProvider } from './contexts/AuthContext'
import { Layout } from './components/Layout'
import { HomePage } from './pages/HomePage'
import { DashboardPage } from './pages/DashboardPage'
import { EducationPage } from './pages/EducationPage'
import { CommunityPage } from './pages/CommunityPage'
import { IdentityPage } from './pages/IdentityPage'
import FeedPage from './pages/FeedPage'

function App() {
  return (
    <AuthProvider>
      <Layout>
        <Routes>
          <Route path="/" element={<HomePage />} />
          <Route path="/dashboard" element={<DashboardPage />} />
          <Route path="/feed" element={<FeedPage />} />
          <Route path="/education" element={<EducationPage />} />
          <Route path="/community" element={<CommunityPage />} />
          <Route path="/identity" element={<IdentityPage />} />
        </Routes>
      </Layout>
    </AuthProvider>
  )
}

export default App
