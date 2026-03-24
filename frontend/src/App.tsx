import { Routes, Route } from 'react-router-dom'
import Layout from './components/Layout'
import ProjectList from './pages/ProjectList'
import ProjectDetail from './pages/ProjectDetail'
import TopicDetail from './pages/TopicDetail'

export default function App() {
  return (
    <Routes>
      <Route element={<Layout />}>
        <Route path="/" element={<ProjectList />} />
        <Route path="/projects/:projectId" element={<ProjectDetail />} />
        <Route path="/projects/:projectId/topics/:topicId" element={<TopicDetail />} />
      </Route>
    </Routes>
  )
}
