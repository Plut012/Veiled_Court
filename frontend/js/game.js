// Game screen controller
// Handles board interactions, game controls, and connection lifecycle

document.addEventListener('DOMContentLoaded', () => {
  // Restore theme from session storage
  const selectedSpirit = sessionStorage.getItem('selectedSpirit');
  const selectedTheme = sessionStorage.getItem('selectedTheme');

  if (selectedTheme) {
    document.body.className = selectedTheme;
  }

  // Restore game context from session storage
  const playerColor = sessionStorage.getItem('playerColor') || 'black';
  const boardSize = parseInt(sessionStorage.getItem('boardSize')) || 19;

  // Apply to game client
  gameClient.playerColor = playerColor;
  gameClient.boardSize = boardSize;
  gameClient.spirit = selectedSpirit;

  // Initialize board renderer (behind the loading overlay)
  boardRenderer.init('board', boardSize);

  // --- Connection lifecycle: connect, init or resume, dismiss overlay ---
  const overlay = document.getElementById('loading-overlay');
  const existingSession = sessionStorage.getItem('sessionId');

  function dismissOverlay() {
    overlay.classList.add('done');
    setTimeout(() => overlay.remove(), 600);
  }

  // Intercept GameStarted to load board and dismiss overlay
  const origHandleGameStarted = gameClient.handleGameStarted.bind(gameClient);
  gameClient.handleGameStarted = function (msg) {
    origHandleGameStarted(msg);

    if (msg.board) {
      boardRenderer.setBoard(msg.board);
      const moveNum = msg.move_number || 0;
      document.getElementById('move-count').textContent = `Move ${moveNum}`;
    }

    gameClient.setTurnIndicator('player');
    dismissOverlay();
  };

  // Also handle BoardUpdate as the resume response — same shape, dismisses overlay
  const origHandleBoardUpdate = gameClient.handleBoardUpdate.bind(gameClient);
  gameClient.handleBoardUpdate = function (msg) {
    origHandleBoardUpdate(msg);
    dismissOverlay();
  };

  // Connect — if we have a saved session, onopen will send ResumeGame automatically.
  // Otherwise, wait for the socket to open and send InitGame.
  gameClient.connect();

  if (!existingSession) {
    const waitForOpen = setInterval(() => {
      if (gameClient.ws && gameClient.ws.readyState === WebSocket.OPEN) {
        clearInterval(waitForOpen);
        gameClient.initGame(selectedSpirit, boardSize, playerColor);
      }
    }, 100);
  }

  // --- Board interaction (unchanged) ---
  const canvas = document.getElementById('board');
  const submitBar = document.getElementById('submit-bar');
  let previewCoords = null;

  canvas.addEventListener('click', (e) => {
    const coords = boardRenderer.coordsFromClick(e);
    if (!coords) return;

    if (boardRenderer.board[coords.y][coords.x] !== 0) return;

    const currentColor = playerColor === 'black' ? 1 : 2;
    boardRenderer.setPreview(coords.x, coords.y, currentColor);
    previewCoords = coords;

    submitBar.style.display = 'block';
  });

  submitBar.addEventListener('click', () => {
    if (!previewCoords) return;
    const coord = gameClient.coordToGTP(previewCoords.x, previewCoords.y);
    gameClient.makeMove(coord);
    boardRenderer.clearPreview();
    previewCoords = null;
    submitBar.style.display = 'none';
  });

  document.addEventListener('keydown', (e) => {
    if (e.key === 'Enter' && previewCoords) {
      submitBar.click();
    }
  });

  const origSetBoard = boardRenderer.setBoard.bind(boardRenderer);
  boardRenderer.setBoard = function (data) {
    previewCoords = null;
    submitBar.style.display = 'none';
    origSetBoard(data);
  };

  document.getElementById('pass-btn').addEventListener('click', () => {
    if (confirm('Pass?')) {
      gameClient.pass();
      boardRenderer.clearPreview();
      previewCoords = null;
      submitBar.style.display = 'none';
    }
  });

  document.getElementById('resign-btn').addEventListener('click', () => {
    if (confirm('Resign?')) {
      gameClient.resign();
    }
  });

  window.addEventListener('resize', () => {
    boardRenderer.setupCanvasSize();
    boardRenderer.draw();
  });
});
