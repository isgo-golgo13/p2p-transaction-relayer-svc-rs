const WebSocket = require('ws');
const express = require('express');
const http = require('http');
const cors = require('cors');

const app = express();
app.use(cors());
app.use(express.json());

const server = http.createServer(app);
const wss = new WebSocket.Server({ server });

// Store connected peers
const peers = new Map();
const rooms = new Map();

console.log('Starting P2P Signaling Server...');

wss.on('connection', (ws, req) => {
    console.log(`New peer connected from ${req.socket.remoteAddress}`);
    
    ws.on('message', (message) => {
        try {
            const data = JSON.parse(message);
            console.log(`Received message type: ${data.type}`);
            handleMessage(ws, data);
        } catch (error) {
            console.error('Invalid message format:', error);
            ws.send(JSON.stringify({ 
                type: 'error', 
                message: 'Invalid message format' 
            }));
        }
    });

    ws.on('close', () => {
        console.log('Peer disconnected');
        cleanupPeer(ws);
    });

    ws.on('error', (error) => {
        console.error('WebSocket error:', error);
    });

    // Send welcome message
    ws.send(JSON.stringify({
        type: 'welcome',
        message: 'Connected to signaling server'
    }));
});

function handleMessage(ws, data) {
    switch (data.type) {
        case 'join':
            joinRoom(ws, data.roomId, data.peerId);
            break;
        case 'leave':
            leaveRoom(ws, data.roomId);
            break;
        case 'offer':
        case 'answer':
        case 'ice-candidate':
            relaySignalingMessage(ws, data);
            break;
        case 'transaction':
            broadcastTransaction(ws, data);
            break;
        case 'ping':
            ws.send(JSON.stringify({ type: 'pong' }));
            break;
        default:
            console.log(`Unknown message type: ${data.type}`);
    }
}

function joinRoom(ws, roomId, peerId) {
    if (!roomId || !peerId) {
        ws.send(JSON.stringify({ 
            type: 'error', 
            message: 'Room ID and Peer ID required' 
        }));
        return;
    }

    // Leave existing room if any
    if (ws.roomId) {
        leaveRoom(ws, ws.roomId);
    }

    if (!rooms.has(roomId)) {
        rooms.set(roomId, new Set());
    }
    
    const room = rooms.get(roomId);
    ws.peerId = peerId;
    ws.roomId = roomId;
    
    // Notify existing peers about new peer
    const existingPeers = [];
    room.forEach(peer => {
        peer.send(JSON.stringify({
            type: 'peer-joined',
            peerId: peerId,
            roomId: roomId
        }));
        existingPeers.push(peer.peerId);
    });
    
    room.add(ws);
    peers.set(peerId, ws);
    
    // Send room state to new peer
    ws.send(JSON.stringify({
        type: 'room-joined',
        roomId: roomId,
        peerId: peerId,
        peers: existingPeers
    }));

    console.log(`Peer ${peerId} joined room ${roomId}. Room size: ${room.size}`);
}

function leaveRoom(ws, roomId) {
    if (!roomId || !rooms.has(roomId)) return;
    
    const room = rooms.get(roomId);
    room.delete(ws);
    
    if (ws.peerId) {
        peers.delete(ws.peerId);
        
        // Notify remaining peers
        room.forEach(peer => {
            peer.send(JSON.stringify({
                type: 'peer-left',
                peerId: ws.peerId,
                roomId: roomId
            }));
        });
    }
    
    // Clean up empty room
    if (room.size === 0) {
        rooms.delete(roomId);
        console.log(`Room ${roomId} deleted (empty)`);
    }
    
    ws.roomId = null;
    ws.peerId = null;
}

function relaySignalingMessage(ws, data) {
    const { targetPeer, roomId } = data;
    
    if (!targetPeer || !roomId) {
        ws.send(JSON.stringify({ 
            type: 'error', 
            message: 'Target peer and room ID required for signaling' 
        }));
        return;
    }

    const targetWs = peers.get(targetPeer);
    if (targetWs && targetWs.roomId === roomId) {
        // Add sender info
        data.fromPeer = ws.peerId;
        targetWs.send(JSON.stringify(data));
        console.log(`Relayed ${data.type} from ${ws.peerId} to ${targetPeer}`);
    } else {
        ws.send(JSON.stringify({
            type: 'error',
            message: `Peer ${targetPeer} not found or not in same room`
        }));
    }
}

function broadcastTransaction(ws, data) {
    if (!ws.roomId || !rooms.has(ws.roomId)) {
        ws.send(JSON.stringify({ 
            type: 'error', 
            message: 'Not in a room' 
        }));
        return;
    }

    const room = rooms.get(ws.roomId);
    const broadcastData = {
        type: 'transaction-broadcast',
        transaction: data.transaction,
        fromPeer: ws.peerId,
        roomId: ws.roomId,
        timestamp: Date.now()
    };

    // Broadcast to all peers in room (including sender for confirmation)
    room.forEach(peer => {
        peer.send(JSON.stringify(broadcastData));
    });

    console.log(`Broadcasted transaction from ${ws.peerId} to ${room.size} peers`);
}

function cleanupPeer(ws) {
    if (ws.roomId) {
        leaveRoom(ws, ws.roomId);
    }
    if (ws.peerId) {
        peers.delete(ws.peerId);
    }
}

// Health check endpoint
app.get('/health', (req, res) => {
    res.json({ 
        status: 'healthy', 
        connections: wss.clients.size,
        rooms: rooms.size,
        timestamp: new Date().toISOString()
    });
});

// Stats endpoint
app.get('/stats', (req, res) => {
    const roomStats = Array.from(rooms.entries()).map(([roomId, peers]) => ({
        roomId,
        peerCount: peers.size,
        peers: Array.from(peers).map(peer => peer.peerId).filter(Boolean)
    }));

    res.json({
        totalConnections: wss.clients.size,
        totalRooms: rooms.size,
        rooms: roomStats
    });
});

const PORT = process.env.PORT || 8080;
server.listen(PORT, () => {
    console.log(`🚀 Signaling server running on port ${PORT}`);
    console.log(`📊 Health check: http://localhost:${PORT}/health`);
    console.log(`📈 Stats: http://localhost:${PORT}/stats`);
});

// Graceful shutdown
process.on('SIGTERM', () => {
    console.log('SIGTERM received, shutting down gracefully');
    server.close(() => {
        console.log('Server closed');
        process.exit(0);
    });
});