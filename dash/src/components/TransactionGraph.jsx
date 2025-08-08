import React, { useEffect, useRef } from 'react';
import * as d3 from 'd3';

function TransactionGraph({ endpoints, transactions }) {
  const svgRef = useRef();

  useEffect(() => {
    if (!endpoints.length) return;

    const svg = d3.select(svgRef.current);
    svg.selectAll("*").remove(); // Clear previous render
    
    const width = 900;
    const height = 500;
    const margin = { top: 20, right: 20, bottom: 20, left: 20 };
    
    svg.attr("width", width).attr("height", height);
    
    // Create gradient definitions
    const defs = svg.append("defs");
    
    const gradient = defs.append("linearGradient")
      .attr("id", "nodeGradient")
      .attr("x1", "0%").attr("y1", "0%")
      .attr("x2", "100%").attr("y2", "100%");
    
    gradient.append("stop")
      .attr("offset", "0%")
      .attr("stop-color", "#ff6b6b");
    
    gradient.append("stop")
      .attr("offset", "100%")
      .attr("stop-color", "#ee5a52");

    // Create nodes for endpoints
    const nodeRadius = 60;
    const nodes = endpoints.map((endpoint, i) => ({
      id: endpoint.id,
      x: 150 + i * (width - 300),
      y: height / 2,
      radius: nodeRadius,
      status: endpoint.status,
      balance: endpoint.balance
    }));
    
    // Draw connection line between endpoints
    if (nodes.length === 2) {
      svg.append("line")
        .attr("x1", nodes[0].x)
        .attr("y1", nodes[0].y)
        .attr("x2", nodes[1].x)
        .attr("y2", nodes[1].y)
        .attr("stroke", "#ddd")
        .attr("stroke-width", 4)
        .attr("stroke-dasharray", "10,5");
    }
    
    // Draw endpoint circles with enhanced styling
    const nodeGroups = svg.selectAll(".endpoint-node")
      .data(nodes)
      .enter()
      .append("g")
      .attr("class", "endpoint-node")
      .attr("transform", d => `translate(${d.x}, ${d.y})`);
    
    // Main circle
    nodeGroups.append("circle")
      .attr("r", d => d.radius)
      .attr("fill", "url(#nodeGradient)")
      .attr("stroke", "#fff")
      .attr("stroke-width", 4)
      .attr("filter", "drop-shadow(0 4px 8px rgba(0,0,0,0.2))")
      .style("transition", "all 0.3s ease");
    
    // Status indicator
    nodeGroups.append("circle")
      .attr("r", 8)
      .attr("cx", 35)
      .attr("cy", -35)
      .attr("fill", d => d.status === 'connected' ? '#4caf50' : '#f44336')
      .attr("stroke", "#fff")
      .attr("stroke-width", 2);
    
    // Endpoint ID labels
    nodeGroups.append("text")
      .attr("text-anchor", "middle")
      .attr("dy", "0.35em")
      .attr("fill", "white")
      .attr("font-weight", "bold")
      .attr("font-size", "14px")
      .text(d => d.id);
    
    // Balance labels
    nodeGroups.append("text")
      .attr("text-anchor", "middle")
      .attr("dy", "20px")
      .attr("fill", "white")
      .attr("font-size", "12px")
      .attr("opacity", "0.9")
      .text(d => `${d.balance.toFixed(2)}`);
    
    // Add legends
    const legend = svg.append("g")
      .attr("transform", `translate(${width - 150}, 30)`);
    
    legend.append("circle")
      .attr("r", 6)
      .attr("fill", "#4caf50");
    
    legend.append("text")
      .attr("x", 15)
      .attr("y", 5)
      .attr("font-size", "12px")
      .text("Connected");
    
    legend.append("circle")
      .attr("r", 6)
      .attr("cy", 20)
      .attr("fill", "#f44336");
    
    legend.append("text")
      .attr("x", 15)
      .attr("y", 25)
      .attr("font-size", "12px")
      .text("Disconnected");

    // Animate recent transactions
    const recentTx = transactions.slice(-5);
    
    recentTx.forEach((tx, i) => {
      const fromNode = nodes.find(n => n.id === (tx.from_endpoint || tx.from));
      const toNode = nodes.find(n => n.id === (tx.to_endpoint || tx.to));
      
      if (fromNode && toNode) {
        setTimeout(() => {
          animateTransaction(svg, fromNode, toNode, tx);
        }, i * 1500);
      }
    });
    
  }, [endpoints, transactions]);

  const animateTransaction = (svg, fromNode, toNode, tx) => {
    // Create a packet circle
    const packet = svg.append("circle")
      .attr("r", 0)
      .attr("fill", "#4caf50")
      .attr("stroke", "#fff")
      .attr("stroke-width", 2)
      .attr("cx", fromNode.x)
      .attr("cy", fromNode.y)
      .attr("filter", "drop-shadow(0 2px 4px rgba(0,0,0,0.3))");
    
    // Animate packet appearance
    packet.transition()
      .duration(200)
      .attr("r", 10);
    
    // Animate packet movement
    packet.transition()
      .delay(200)
      .duration(2000)
      .ease(d3.easeCubicInOut)
      .attr("cx", toNode.x)
      .attr("cy", toNode.y)
      .on("end", () => {
        // Show transaction amount briefly
        const label = svg.append("text")
          .attr("x", toNode.x)
          .attr("y", toNode.y - 40)
          .attr("text-anchor", "middle")
          .attr("fill", "#4caf50")
          .attr("font-weight", "bold")
          .attr("font-size", "16px")
          .text(`+${tx.amount}`)
          .style("opacity", 0);
        
        label.transition()
          .duration(300)
          .style("opacity", 1)
          .transition()
          .delay(1000)
          .duration(500)
          .style("opacity", 0)
          .on("end", () => label.remove());
        
        // Remove packet
        packet.transition()
          .duration(300)
          .attr("r", 0)
          .style("opacity", 0)
          .on("end", () => packet.remove());
      });
  };

  return (
    <div className="transaction-graph" style={{
      background: 'white',
      borderRadius: '12px',
      padding: '20px',
      boxShadow: '0 4px 15px rgba(0,0,0,0.1)',
      margin: '20px 0'
    }}>
      <svg ref={svgRef} style={{ width: '100%', height: 'auto' }}></svg>
    </div>
  );
}

export default TransactionGraph;
