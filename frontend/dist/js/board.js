// Board rendering logic adapted from phone_go
// Handles canvas rendering, stone placement, and preview

const boardRenderer = {
  canvas: null,
  ctx: null,
  board: null,
  boardSize: 19,
  previewMove: null, // {x, y} for move being previewed
  lastMove: null, // {x, y} for last move indicator

  init(canvasId, size = 19) {
    this.canvas = document.getElementById(canvasId);
    this.ctx = this.canvas.getContext('2d');
    this.boardSize = size;
    this.board = this.createEmptyBoard(size);
    this.setupCanvasSize();
    this.draw();
  },

  createEmptyBoard(size) {
    const board = [];
    for (let i = 0; i < size; i++) {
      board[i] = [];
      for (let j = 0; j < size; j++) {
        board[i][j] = 0; // 0 = empty, 1 = black, 2 = white
      }
    }
    return board;
  },

  setupCanvasSize() {
    const container = this.canvas.parentElement;
    const displaySize = Math.min(container.clientWidth, container.clientHeight) - 20;
    const dpr = window.devicePixelRatio || 1;

    this.canvas.width = displaySize * dpr;
    this.canvas.height = displaySize * dpr;
    this.canvas.style.width = displaySize + 'px';
    this.canvas.style.height = displaySize + 'px';
    this.ctx.setTransform(dpr, 0, 0, dpr, 0, 0);
  },

  draw() {
    const displaySize = parseInt(this.canvas.style.width);
    const cs = displaySize / (this.boardSize + 1); // cell size
    const m = cs; // margin

    // Get colors from CSS custom properties
    const styles = getComputedStyle(document.body);
    const boardPrimary = styles.getPropertyValue('--board-primary').trim();
    const boardSecondary = styles.getPropertyValue('--board-secondary').trim();
    const accent = styles.getPropertyValue('--accent').trim();

    // Clear canvas
    this.ctx.fillStyle = boardPrimary;
    this.ctx.fillRect(0, 0, displaySize, displaySize);

    // Draw grid lines
    this.ctx.strokeStyle = boardSecondary;
    this.ctx.lineWidth = 1;
    for (let i = 0; i < this.boardSize; i++) {
      // Vertical lines
      this.ctx.beginPath();
      this.ctx.moveTo(m + i * cs, m);
      this.ctx.lineTo(m + i * cs, m + (this.boardSize - 1) * cs);
      this.ctx.stroke();

      // Horizontal lines
      this.ctx.beginPath();
      this.ctx.moveTo(m, m + i * cs);
      this.ctx.lineTo(m + (this.boardSize - 1) * cs, m + i * cs);
      this.ctx.stroke();
    }

    // Draw star points (hoshi)
    this.drawStarPoints(cs, m, boardSecondary);

    // Draw stones
    for (let y = 0; y < this.boardSize; y++) {
      for (let x = 0; x < this.boardSize; x++) {
        const stone = this.board[y][x];
        if (stone) {
          this.drawStone(x, y, stone, cs, m);
        }
      }
    }

    // Draw last move indicator
    if (this.lastMove) {
      const cx = m + this.lastMove.x * cs;
      const cy = m + this.lastMove.y * cs;
      const r = cs * 0.3;
      const stoneColor = this.board[this.lastMove.y][this.lastMove.x];

      this.ctx.beginPath();
      this.ctx.arc(cx, cy, r, 0, Math.PI * 2);
      this.ctx.fillStyle = stoneColor === 1 ? accent : boardSecondary;
      this.ctx.globalAlpha = 0.7;
      this.ctx.fill();
      this.ctx.globalAlpha = 1.0;
    }

    // Draw preview move (semi-transparent)
    if (this.previewMove) {
      const px = this.previewMove.x;
      const py = this.previewMove.y;

      // Only draw if position is empty
      if (this.board[py][px] === 0) {
        const cx = m + px * cs;
        const cy = m + py * cs;
        const r = cs * 0.45;

        // Determine preview color (based on next player)
        const previewColor = this.previewMove.color || 1; // default to black

        this.ctx.beginPath();
        this.ctx.arc(cx, cy, r, 0, Math.PI * 2);
        this.ctx.fillStyle = previewColor === 1 ? '#1a1815' : '#e8e0d4';
        this.ctx.globalAlpha = 0.5;
        this.ctx.fill();
        this.ctx.globalAlpha = 1.0;
      }
    }
  },

  drawStarPoints(cs, m, color) {
    this.ctx.fillStyle = color;

    const starPoints = this.getStarPoints();

    for (const [x, y] of starPoints) {
      this.ctx.beginPath();
      this.ctx.arc(m + x * cs, m + y * cs, cs * 0.1, 0, Math.PI * 2);
      this.ctx.fill();
    }
  },

  getStarPoints() {
    if (this.boardSize === 19) {
      return [
        [3, 3], [3, 9], [3, 15],
        [9, 3], [9, 9], [9, 15],
        [15, 3], [15, 9], [15, 15]
      ];
    } else if (this.boardSize === 13) {
      return [
        [3, 3], [3, 9],
        [6, 6],
        [9, 3], [9, 9]
      ];
    } else if (this.boardSize === 9) {
      return [
        [2, 2], [2, 6],
        [4, 4],
        [6, 2], [6, 6]
      ];
    }
    return [];
  },

  drawStone(x, y, color, cs, m) {
    const cx = m + x * cs;
    const cy = m + y * cs;
    const r = cs * 0.45;

    // Shadow
    this.ctx.beginPath();
    this.ctx.arc(cx + 1.5, cy + 1.5, r, 0, Math.PI * 2);
    this.ctx.fillStyle = 'rgba(15, 10, 8, 0.4)';
    this.ctx.fill();

    // Stone with gradient
    const grad = this.ctx.createRadialGradient(
      cx - r * 0.3, cy - r * 0.3, r * 0.1,
      cx, cy, r
    );

    if (color === 1) {
      // Black stone
      grad.addColorStop(0, '#4a4540');
      grad.addColorStop(0.7, '#2a2725');
      grad.addColorStop(1, '#1a1815');
    } else {
      // White stone
      grad.addColorStop(0, '#f5f0e8');
      grad.addColorStop(0.5, '#e8e0d4');
      grad.addColorStop(1, '#d8cfc0');
    }

    this.ctx.beginPath();
    this.ctx.arc(cx, cy, r, 0, Math.PI * 2);
    this.ctx.fillStyle = grad;
    this.ctx.fill();
  },

  setBoard(boardData) {
    this.board = boardData;
    this.draw();
  },

  setPreview(x, y, color) {
    this.previewMove = { x, y, color };
    this.draw();
  },

  clearPreview() {
    this.previewMove = null;
    this.draw();
  },

  setLastMove(x, y) {
    this.lastMove = { x, y };
    this.draw();
  },

  coordsFromClick(event) {
    const rect = this.canvas.getBoundingClientRect();
    const displaySize = rect.width;
    const cs = displaySize / (this.boardSize + 1);
    const m = cs;

    const x = Math.round((event.clientX - rect.left - m) / cs);
    const y = Math.round((event.clientY - rect.top - m) / cs);

    if (x >= 0 && x < this.boardSize && y >= 0 && y < this.boardSize) {
      return { x, y };
    }
    return null;
  }
};

// Make it globally available
window.boardRenderer = boardRenderer;
