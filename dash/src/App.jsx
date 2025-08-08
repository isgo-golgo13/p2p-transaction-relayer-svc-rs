import React, { useState, useEffect } from 'react';
import TxEndpointCard from './components/TxEndpointCard';
import TransactionGraph from './components/TransactionGraph';
import StatsPanel from './components/StatsPanel';
import './App.css';

function App() {
  const [transactions, setTransactions] = useState([]);
  const [endpoints, setEndpoints] = useState([
    { id: 'endpoint-1', status: 'disconnected', balance: 1000, url: 'http://localhost:8000' },
    { id: 'endpoint-2', status: 'disconnected', balance: 1000, url: 'http://localhost:8001' }
  ]);
  const [stats, setStats] = useState(null);
  const [loading, setLoading] = useState(true);
  const [error, setError] = useState('');

  useEffect(() => {
    initializeConnections();
    fetchData();
    
    // Set up polling for updates
    const interval = setInterval(fetchData, 2000);
    return () => clearInterval(interval);
  }, []);

  const initializeConnections = () => {
    // Connect to signaling server for real-time updates
    try {
      const signalingUrl = import.meta.env.VITE_SIGNALING_SERVER || 'ws://localhost:8080';
      const ws = new WebSocket(signalingUrl);
      
      ws.onopen = () => {
        console.log('Connected to signaling server');
        setError('');
        
        // Join room as observer
        ws.send(JSON.stringify({
          type: 'join',
          roomId: 'transaction-room',
          peerId: 'dashboard-observer'
        }));
      };

      ws.onmessage = (event) => {
        try {
          const data = JSON.parse(event.data);
          console.log('Dashboard received:', data);
          
          if (data.type === 'transaction-broadcast') {
            setTransactions(prev => {
              // Avoid duplicates
              const exists = prev.some(tx => tx.id === data.transaction.id);
              if (!exists) {
                return [data.transaction, ...prev].slice(0, 100); // Keep last 100
              }
              return prev;
            });
          }
          
          if (data.type === 'peer-joined' || data.type === 'peer-left') {
            updateEndpointStatus(data.peerId, data.type === 'peer-joined' ? 'connected' : 'disconnected');
          }
        } catch (err) {
          console.error('Error parsing WebSocket message:', err);
        }
      };

      ws.onclose = () => {
        console.log('Disconnected from signaling server');
        setError('Connection to signaling server lost');
        // Try to reconnect after 3 seconds
        setTimeout(initializeConnections, 3000);
      };

      ws.onerror = (error) => {
        console.error('WebSocket error:', error);
        setError('Failed to connect to signaling server');
      };

      return () => ws.close();
    } catch (err) {
      console.error('Failed to initialize WebSocket:', err);
      setError('Failed to initialize real-time connection');
    }
  };

  const updateEndpointStatus = (endpointId, status) => {
    setEndpoints(prev => prev.map(ep => 
      ep.id === endpointId ? { ...ep, status } : ep
    ));
  };

  const fetchData = async () => {
    try {
      const apiGateway = import.meta.env.VITE_API_GATEWAY || 'http://localhost:3001';
      
      // Fetch transactions
      const transactionsResponse = await fetch(`${apiGateway}/api/transactions?limit=50`);
      if (transactionsResponse.ok) {
        const transactionsData = await transactionsResponse.json();
        setTransactions(transactionsData);
      }
      
      // Fetch stats
      const statsResponse = await fetch(`${apiGateway}/api/stats`);
      if (statsResponse.ok) {
        const statsData = await statsResponse.json();
        setStats(statsData);
        
        // Update endpoint balances from stats
        setEndpoints(prev => prev.map(ep => {
          const endpointStats = statsData.endpoints.find(es => es.endpoint_id === ep.id);
          return endpointStats ? 
            { ...ep, balance: 1000 + endpointStats.balance_change } : ep;
        }));
      }
      
      setLoading(false);
    } catch (err) {
      console.error('Error fetching data:', err);
      setError('Failed to fetch data from API');
      setLoading(false);
    }
  };

  if (loading) {
    return (
      <div className="app-loading">
        <div className="loading-spinner"></div>
        <h2>Loading P2P Transaction Dashboard...</h2>
      </div>
    );
  }

  return (
    <div className="App">
      <header className="app-header">
        <h1>🔗 P2P Transaction System Dashboard</h1>
        <p>Real-time WebSocket P2P Transaction Monitoring</p>
        {error && (
          <div className="error-banner">
            ⚠️ {error}
          </div>
        )}
      </header>
      
      <div className="dashboard-container">
        {stats && <StatsPanel stats={stats} />}
        
        <div className="endpoints-section">
          <h2>Transaction Endpoints</h2>
          <div className="endpoints-container">
            {endpoints.map(endpoint => (
              <TxEndpointCard 
                key={endpoint.id}
                endpoint={endpoint}
                transactions={transactions.filter(tx => 
                  tx.from_endpoint === endpoint.id || tx.to_endpoint === endpoint.id
                )}
              />
            ))}
          </div>
        </div>
        
        <div className="graph-section">
          <h2>Live Transaction Flow</h2>
          <TransactionGraph 
            endpoints={endpoints}
            transactions={transactions}
          />
        </div>
        
        <div className="transactions-section">
          <h2>Recent Transactions ({transactions.length})</h2>
          <div className="transactions-list">
            {transactions.slice(0, 20).map(tx => (
              <div key={tx.id} className="transaction-item">
                <div className="transaction-header">
                  <span className="transaction-amount">${tx.amount}</span>
                  <span className={`transaction-status status-${tx.status}`}>
                    {tx.status}
                  </span>
                </div>
                <div className="transaction-details">
                  <span className="transaction-flow">
                    {tx.from_endpoint || tx.from} → {tx.to_endpoint || tx.to}
                  </span>
                  <span className="transaction-time">
                    {new Date(tx.timestamp).toLocaleTimeString()}
                  </span>
                </div>
                <div className="transaction-id">
                  ID: {tx.id.substring(0, 8)}...
                </div>
              </div>
            ))}
            
            {transactions.length === 0 && (
              <div className="no-transactions">
                No transactions yet. Start the endpoints to see activity!
              </div>
            )}
          </div>
        </div>
      </div>
    </div>
  );
}

export default App;

### dashboard/src/components/TxEndpointCard.jsx
```javascript
import React from 'react';

function TxEndpointCard({ endpoint, transactions }) {
  const recentTransactions = transactions.slice(-5);
  const totalSent = transactions
    .filter(tx => (tx.from_endpoint || tx.from) === endpoint.id)
    .reduce((sum, tx) => sum + tx.amount, 0);
  const totalReceived = transactions
    .filter(tx => (tx.to_endpoint || tx.to) === endpoint.id)
    .reduce((sum, tx) => sum + tx.amount, 0);
  
  return (
    <div className="tx-endpoint-card" style={{
      background: 'linear-gradient(135deg, #ff6b6b, #ee5a52)',
      color: 'white',
      padding: '25px',
      borderRadius: '15px',
      margin: '15px',
      minWidth: '320px',
      boxShadow: '0 8px 25px rgba(238, 90, 82, 0.3)',
      transition: 'transform 0.3s ease, box-shadow 0.3s ease',
      cursor: 'pointer'
    }}
    onMouseEnter={(e) => {
      e.currentTarget.style.transform = 'translateY(-5px)';
      e.currentTarget.style.boxShadow = '0 15px 35px rgba(238, 90, 82, 0.4)';
    }}
    onMouseLeave={(e) => {
      e.currentTarget.style.transform = 'translateY(0)';
      e.currentTarget.style.boxShadow = '0 8px 25px rgba(238, 90, 82, 0.3)';
    }}>
      
      <div className="card-header" style={{ marginBottom: '20px' }}>
        <h3 style={{ margin: '0 0 10px 0', fontSize: '1.4rem' }}>{endpoint.id}</h3>
        
        <div className="status-indicator" style={{
          display: 'flex',
          alignItems: 'center',
          marginBottom: '15px'
        }}>
          <div style={{
            width: '12px',
            height: '12px',
            borderRadius: '50%',
            background: endpoint.status === 'connected' ? '#4caf50' : '#f44336',
            marginRight: '10px',
            boxShadow: endpoint.status === 'connected' ? '0 0 10px #4caf50' : '0 0 10px #f44336'
          }}></div>
          <span style={{ fontWeight: '600', textTransform: 'capitalize' }}>
            {endpoint.status}
          </span>
        </div>
      </div>
      
      <div className="balance-section" style={{ marginBottom: '20px' }}>
        <div style={{ 
          background: 'rgba(255,255,255,0.2)', 
          padding: '15px', 
          borderRadius: '10px',
          backdropFilter: 'blur(10px)'
        }}>
          <h4 style={{ margin: '0 0 8px 0', fontSize: '0.9rem', opacity: '0.9' }}>
            Current Balance
          </h4>
          <div style={{ fontSize: '1.8rem', fontWeight: 'bold' }}>
            ${endpoint.balance.toFixed(2)}
          </div>
        </div>
      </div>

      <div className="transaction-summary" style={{ marginBottom: '20px' }}>
        <div style={{ display: 'grid', gridTemplateColumns: '1fr 1fr', gap: '10px' }}>
          <div style={{ 
            background: 'rgba(255,255,255,0.15)', 
            padding: '10px', 
            borderRadius: '8px',
            textAlign: 'center'
          }}>
            <div style={{ fontSize: '0.8rem', opacity: '0.9' }}>Sent</div>
            <div style={{ fontSize: '1.2rem', fontWeight: 'bold' }}>
              ${totalSent.toFixed(2)}
            </div>
          </div>
          <div style={{ 
            background: 'rgba(255,255,255,0.15)', 
            padding: '10px', 
            borderRadius: '8px',
            textAlign: 'center'
          }}>
            <div style={{ fontSize: '0.8rem', opacity: '0.9' }}>Received</div>
            <div style={{ fontSize: '1.2rem', fontWeight: 'bold' }}>
              ${totalReceived.toFixed(2)}
            </div>
          </div>
        </div>
      </div>
      
      <div className="recent-transactions">
        <h5 style={{ margin: '0 0 10px 0', fontSize: '1rem' }}>
          Recent Activity ({transactions.length})
        </h5>
        <div style={{ maxHeight: '180px', overflowY: 'auto' }}>
          {recentTransactions.length > 0 ? (
            recentTransactions.map(tx => (
              <div key={tx.id} style={{
                background: 'rgba(255,255,255,0.1)',
                padding: '10px',
                margin: '5px 0',
                borderRadius: '6px',
                fontSize: '0.85em',
                border: '1px solid rgba(255,255,255,0.2)'
              }}>
                <div style={{ display: 'flex', justifyContent: 'space-between', alignItems: 'center' }}>
                  <span style={{ fontWeight: 'bold' }}>
                    {(tx.from_endpoint || tx.from) === endpoint.id ? '📤' : '📥'} 
                    ${tx.amount}
                  </span>
                  <span style={{ fontSize: '0.7rem', opacity: '0.8' }}>
                    {new Date(tx.timestamp).toLocaleTimeString()}
                  </span>
                </div>
                <div style={{ fontSize: '0.75rem', opacity: '0.8', marginTop: '4px' }}>
                  {tx.from_endpoint || tx.from} → {tx.to_endpoint || tx.to}
                </div>
              </div>
            ))
          ) : (
            <div style={{ 
              textAlign: 'center', 
              padding: '20px', 
              opacity: '0.7',
              fontSize: '0.9rem'
            }}>
              No transactions yet
            </div>
          )}
        </div>
      </div>
      
      <div className="endpoint-actions" style={{ marginTop: '15px' }}>
        <button 
          onClick={() => window.open(endpoint.url, '_blank')}
          style={{
            background: 'rgba(255,255,255,0.2)',
            color: 'white',
            border: '1px solid rgba(255,255,255,0.3)',
            padding: '8px 16px',
            borderRadius: '6px',
            cursor: 'pointer',
            fontSize: '0.9rem',
            width: '100%',
            fontWeight: '600',
            transition: 'all 0.3s ease'
          }}
          onMouseEnter={(e) => {
            e.target.style.background = 'rgba(255,255,255,0.3)';
          }}
          onMouseLeave={(e) => {
            e.target.style.background = 'rgba(255,255,255,0.2)';
          }}
        >
          Open Endpoint Interface
        </button>
      </div>
    </div>
  );
}

export default TxEndpointCard;

