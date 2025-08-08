import React from 'react';

function StatsPanel({ stats }) {
  const formatCurrency = (amount) => `${amount.toFixed(2)}`;
  
  return (
    <div className="stats-panel" style={{
      display: 'grid',
      gridTemplateColumns: 'repeat(auto-fit, minmax(200px, 1fr))',
      gap: '20px',
      margin: '20px 0',
      padding: '20px',
      background: 'linear-gradient(135deg, #667eea 0%, #764ba2 100%)',
      borderRadius: '15px',
      color: 'white'
    }}>
      <div className="stat-card" style={{
        background: 'rgba(255,255,255,0.1)',
        padding: '20px',
        borderRadius: '10px',
        textAlign: 'center',
        backdropFilter: 'blur(10px)'
      }}>
        <h3 style={{ margin: '0 0 10px 0', fontSize: '0.9rem', opacity: '0.9' }}>
          Total Transactions
        </h3>
        <div style={{ fontSize: '2rem', fontWeight: 'bold' }}>
          {stats.total_transactions}
        </div>
      </div>
      
      <div className="stat-card" style={{
        background: 'rgba(255,255,255,0.1)',
        padding: '20px',
        borderRadius: '10px',
        textAlign: 'center',
        backdropFilter: 'blur(10px)'
      }}>
        <h3 style={{ margin: '0 0 10px 0', fontSize: '0.9rem', opacity: '0.9' }}>
          Total Volume
        </h3>
        <div style={{ fontSize: '2rem', fontWeight: 'bold' }}>
          {formatCurrency(stats.total_volume)}
        </div>
      </div>
      
      <div className="stat-card" style={{
        background: 'rgba(255,255,255,0.1)',
        padding: '20px',
        borderRadius: '10px',
        textAlign: 'center',
        backdropFilter: 'blur(10px)'
      }}>
        <h3 style={{ margin: '0 0 10px 0', fontSize: '0.9rem', opacity: '0.9' }}>
          Average Transaction
        </h3>
        <div style={{ fontSize: '2rem', fontWeight: 'bold' }}>
          {formatCurrency(stats.average_transaction)}
        </div>
      </div>
      
      <div className="stat-card" style={{
        background: 'rgba(255,255,255,0.1)',
        padding: '20px',
        borderRadius: '10px',
        textAlign: 'center',
        backdropFilter: 'blur(10px)'
      }}>
        <h3 style={{ margin: '0 0 10px 0', fontSize: '0.9rem', opacity: '0.9' }}>
          Active Endpoints
        </h3>
        <div style={{ fontSize: '2rem', fontWeight: 'bold' }}>
          {stats.endpoints.length}
        </div>
      </div>
    </div>
  );
}

export default StatsPanel;

