// Game screen controller
// Handles board interactions and game controls

document.addEventListener('DOMContentLoaded', () => {
  // Restore theme from session storage
  const selectedSpirit = sessionStorage.getItem('selectedSpirit');
  const selectedTheme = sessionStorage.getItem('selectedTheme');

  if (selectedTheme) {
    document.body.className = selectedTheme;
  }

  // Initialize board renderer
  boardRenderer.init('board', 19); // Default to 19x19, will update from server

  // Set up canvas click handler
  const canvas = document.getElementById('board');
  let previewCoords = null;

  canvas.addEventListener('click', (e) => {
    const coords = boardRenderer.coordsFromClick(e);
    if (!coords) return;

    // Check if spot is empty
    if (boardRenderer.board[coords.y][coords.x] !== 0) {
      console.log('Position occupied');
      return;
    }

    // Set preview
    const currentColor = gameClient.playerColor === 'black' ? 1 : 2;
    boardRenderer.setPreview(coords.x, coords.y, currentColor);
    previewCoords = coords;

    // Show submit confirmation (for mobile UX)
    showSubmitConfirmation(coords);
  });

  // Pass button
  document.getElementById('pass-btn').addEventListener('click', () => {
    if (confirm('Are you sure you want to pass?')) {
      gameClient.pass();
      boardRenderer.clearPreview();
      previewCoords = null;
    }
  });

  // Resign button
  document.getElementById('resign-btn').addEventListener('click', () => {
    if (confirm('Are you sure you want to resign?')) {
      gameClient.resign();
    }
  });

  // Initialize WebSocket connection
  gameClient.connect();
});

function showSubmitConfirmation(coords) {
  // Simple confirmation for mobile UX
  if (confirm(`Play stone at ${gameClient.coordToGTP(coords.x, coords.y)}?`)) {
    const coord = gameClient.coordToGTP(coords.x, coords.y);
    gameClient.makeMove(coord);
    boardRenderer.clearPreview();
  } else {
    boardRenderer.clearPreview();
  }
}
