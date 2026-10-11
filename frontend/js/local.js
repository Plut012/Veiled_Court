// Local pass-and-play UI
// Rules live in rules.js (goEngine), loaded before this file

// --- UI Controller ---

document.addEventListener('DOMContentLoaded', () => {
  const boardSize = 19;
  goEngine.init(boardSize);
  boardRenderer.init('board', boardSize);

  const submitTop = document.getElementById('submit-bar-top');
  const submitBottom = document.getElementById('submit-bar-bottom');
  const canvas = document.getElementById('board');
  let previewCoords = null;

  function hideSubmit() {
    submitTop.style.display = 'none';
    submitBottom.style.display = 'none';
  }

  function showSubmit() {
    submitTop.style.display = 'block';
    submitBottom.style.display = 'block';
  }

  function syncBoard() {
    // Convert engine board (numeric) to renderer format
    boardRenderer.board = goEngine.board.map(row => [...row]);
    boardRenderer.draw();
  }

  canvas.addEventListener('click', (e) => {
    const coords = boardRenderer.coordsFromClick(e);
    if (!coords) return;
    if (goEngine.board[coords.y][coords.x] !== 0) return;

    // Validate before showing preview
    const test = goEngine.tryMove(coords.x, coords.y);
    if (!test.valid) return;

    boardRenderer.setPreview(coords.x, coords.y, goEngine.currentPlayer);
    previewCoords = coords;
    showSubmit();
  });

  function submitMove() {
    if (!previewCoords) return;
    const result = goEngine.playMove(previewCoords.x, previewCoords.y);
    if (!result.valid) return;

    boardRenderer.clearPreview();
    boardRenderer.lastMove = { x: previewCoords.x, y: previewCoords.y };
    previewCoords = null;
    hideSubmit();
    syncBoard();
  }

  submitTop.addEventListener('click', submitMove);
  submitBottom.addEventListener('click', submitMove);

  document.addEventListener('keydown', (e) => {
    if (e.key === 'Enter' && previewCoords) submitMove();
  });

  document.getElementById('pass-btn').addEventListener('click', () => {
    const gameOver = goEngine.pass();
    previewCoords = null;
    hideSubmit();
    boardRenderer.clearPreview();
    boardRenderer.lastMove = null;
    syncBoard();

    if (gameOver) {
      // Both passed — game over
      const controls = document.getElementById('local-controls');
      controls.innerHTML = '<span style="color: var(--accent); font-family: Cormorant Garamond, serif; font-style: italic; opacity: 0.7;">Game complete</span>';
    }
  });

  document.getElementById('undo-btn').addEventListener('click', () => {
    if (goEngine.undo()) {
      previewCoords = null;
      hideSubmit();
      boardRenderer.clearPreview();
      boardRenderer.lastMove = null;
      syncBoard();
    }
  });

  window.addEventListener('resize', () => {
    boardRenderer.setupCanvasSize();
    boardRenderer.draw();
  });
});
