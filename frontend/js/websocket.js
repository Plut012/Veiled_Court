// WebSocket client for Spirit Animals Go
// Handles all server communication

class GameClient {
  constructor() {
    this.ws = null;
    this.sessionId = null;
    this.spirit = null;
    this.boardSize = 19;
    this.playerColor = 'white';
    this.currentTurn = 1; // 1 = black, 2 = white
    this.moveNumber = 0;
    this.reconnectAttempts = 0;
    this.maxReconnectAttempts = 5;
  }

  connect() {
    const protocol = window.location.protocol === 'https:' ? 'wss:' : 'ws:';
    const wsUrl = `${protocol}//${window.location.host}/ws`;

    console.log('Connecting to WebSocket:', wsUrl);

    this.ws = new WebSocket(wsUrl);

    this.ws.onopen = () => {
      console.log('Connected to game server');
      this.reconnectAttempts = 0;
    };

    this.ws.onmessage = (event) => {
      try {
        const msg = JSON.parse(event.data);
        console.log('Received message:', msg);
        this.handleMessage(msg);
      } catch (err) {
        console.error('Failed to parse message:', err);
      }
    };

    this.ws.onerror = (error) => {
      console.error('WebSocket error:', error);
    };

    this.ws.onclose = () => {
      console.log('Disconnected from game server');
      this.attemptReconnect();
    };
  }

  attemptReconnect() {
    if (this.reconnectAttempts < this.maxReconnectAttempts) {
      this.reconnectAttempts++;
      const delay = Math.min(1000 * Math.pow(2, this.reconnectAttempts), 10000);
      console.log(`Reconnecting in ${delay}ms (attempt ${this.reconnectAttempts})`);

      setTimeout(() => {
        this.connect();
      }, delay);
    } else {
      console.error('Max reconnection attempts reached');
      this.showError('Connection lost. Please refresh the page.');
    }
  }

  initGame(spirit, boardSize, playerColor) {
    this.spirit = spirit;
    this.boardSize = boardSize;
    this.playerColor = playerColor;

    const message = {
      type: 'InitGame',
      spirit: spirit,
      board_size: boardSize,
      player_color: playerColor
    };

    console.log('Initializing game:', message);
    this.send(message);
  }

  makeMove(coord) {
    const message = {
      type: 'Move',
      coord: coord
    };

    console.log('Making move:', message);
    this.send(message);
  }

  pass() {
    const message = { type: 'Pass' };
    console.log('Passing');
    this.send(message);
  }

  resign() {
    const message = { type: 'Resign' };
    console.log('Resigning');
    this.send(message);
  }

  send(message) {
    if (this.ws && this.ws.readyState === WebSocket.OPEN) {
      this.ws.send(JSON.stringify(message));
    } else {
      console.error('WebSocket not connected');
      this.showError('Not connected to server');
    }
  }

  handleMessage(msg) {
    switch (msg.type) {
      case 'GameStarted':
        this.handleGameStarted(msg);
        break;

      case 'BoardUpdate':
        this.handleBoardUpdate(msg);
        break;

      case 'KoActive':
        this.handleKoActive(msg);
        break;

      case 'GameOver':
        this.handleGameOver(msg);
        break;

      case 'Error':
        this.handleError(msg);
        break;

      default:
        console.warn('Unknown message type:', msg.type);
    }
  }

  handleGameStarted(msg) {
    console.log('Game started:', msg);
    this.sessionId = msg.session_id;
    this.boardSize = msg.board_size;

    // Navigate to game screen
    window.location.href = `/game.html?session=${msg.session_id}`;
  }

  handleBoardUpdate(msg) {
    console.log('Board update:', msg);

    // Update board state
    if (window.boardRenderer) {
      boardRenderer.setBoard(msg.board);

      // Update last move indicator
      if (msg.last_move) {
        const coord = this.parseCoord(msg.last_move);
        if (coord) {
          boardRenderer.setLastMove(coord.x, coord.y);
        }
      }

      // Update move counter
      this.moveNumber = msg.move_number || 0;
      document.getElementById('move-count').textContent = `Move ${this.moveNumber}`;

      // Update turn indicator
      this.currentTurn = (this.moveNumber % 2 === 0) ? 1 : 2;
      const turnText = this.currentTurn === 1 ? 'Black to play' : 'White to play';
      document.getElementById('current-turn').textContent = turnText;

      // Update Jaguar palette if needed
      if (this.spirit === 'jaguar' && window.updateJaguarPalette) {
        updateJaguarPalette(this.moveNumber);
      }
    }
  }

  handleKoActive(msg) {
    console.log('Ko active:', msg);

    // Apply Crow ko-dim effect
    if (this.spirit === 'crow') {
      document.body.classList.add('ko-dim');

      // Highlight ko threats
      if (msg.threats && window.boardRenderer) {
        // TODO: Implement ko threat highlighting on board
        console.log('Ko threats:', msg.threats);
      }
    }
  }

  handleGameOver(msg) {
    console.log('Game over:', msg);

    // Display winner
    const infoDiv = document.getElementById('info');
    const resultText = document.createElement('div');
    resultText.style.fontSize = '24px';
    resultText.style.color = getComputedStyle(document.body).getPropertyValue('--accent');
    resultText.textContent = `Game Over - ${msg.winner} wins!`;
    infoDiv.appendChild(resultText);

    // Disable controls
    document.getElementById('pass-btn').disabled = true;
    document.getElementById('resign-btn').disabled = true;
  }

  handleError(msg) {
    console.error('Server error:', msg);
    this.showError(msg.message || 'An error occurred');
  }

  parseCoord(coordStr) {
    // Parse GTP coordinate like "Q16" into {x, y}
    if (!coordStr || coordStr.length < 2) return null;

    const col = coordStr.charCodeAt(0) - 65; // A=0, B=1, etc.
    const row = parseInt(coordStr.substring(1)) - 1;

    if (col >= 0 && col < this.boardSize && row >= 0 && row < this.boardSize) {
      return { x: col, y: row };
    }
    return null;
  }

  coordToGTP(x, y) {
    // Convert {x, y} to GTP coordinate like "Q16"
    const col = String.fromCharCode(65 + x); // 0=A, 1=B, etc.
    const row = y + 1;
    return `${col}${row}`;
  }

  showError(message) {
    // Simple error display
    alert(message);
  }
}

// Create global instance
const gameClient = new GameClient();
window.gameClient = gameClient;
