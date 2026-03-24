import { Routes, Route } from 'react-router-dom'
import AppShell from './components/shell/AppShell'
import ProjectList from './pages/ProjectList'
import ProjectDetail from './pages/ProjectDetail'
import TopicDetail from './pages/TopicDetail'

export default function App() {
  return (
    <Routes>
      <Route path="/" element={<AppShellWrapper><ProjectList /></AppShellWrapper>} />
      <Route path="/projects/:projectId" element={<AppShellWrapper><ProjectDetail /></AppShellWrapper>} />
      <Route path="/projects/:projectId/topics/:topicId" element={<AppShellWrapper><TopicDetail /></AppShellWrapper>} />
    </Routes>
  )
}

function AppShellWrapper({ children }: { children: React.ReactNode }) {
  return <AppShell>{children}</AppShell>
}
