import { BrowserRouter, Routes, Route } from 'react-router-dom';
import Dashboard from './components/Dashboard';
import Home from './components/Home';
import DetectionEngine from './components/DetectionEngine/DetectionEngine';
import CircuitBreaker from './components/CircuitBreaker/CircuitBreaker';
import AlertRegistry from './components/AlertRegistry/AlertRegistry';

function App() {
  return (
    <BrowserRouter>
      <Routes>
        <Route path="/" element={<Dashboard />}>
          <Route index element={<Home />} />
          <Route path="detection-engine" element={<DetectionEngine />} />
          <Route path="circuit-breaker" element={<CircuitBreaker />} />
          <Route path="alerts" element={<AlertRegistry />} />
        </Route>
      </Routes>
    </BrowserRouter>
  );
}

export default App;
