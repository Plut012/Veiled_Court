// Go rules engine shared by local pass-and-play (local.js) and friend rooms (room.js)
// Handles game rules, captures, ko, and undo

const goEngine = {
  boardSize: 19,
  board: null,       // 2D array: 0=empty, 1=black, 2=white
  currentPlayer: 1,  // 1=black, 2=white
  history: [],       // array of {board, lastMove, captured, player}
  koPoint: null,     // {x,y} forbidden by ko, or null
  passes: 0,

  init(size) {
    this.boardSize = size;
    this.board = this.emptyBoard(size);
    this.currentPlayer = 1;
    this.history = [];
    this.koPoint = null;
    this.passes = 0;
  },

  emptyBoard(size) {
    return Array.from({ length: size }, () => Array(size).fill(0));
  },

  copyBoard(board) {
    return board.map(row => [...row]);
  },

  // Find all stones in the group connected to (x,y)
  getGroup(board, x, y) {
    const color = board[y][x];
    if (color === 0) return { stones: [], liberties: 0 };

    const visited = new Set();
    const stones = [];
    let liberties = 0;
    const stack = [[x, y]];

    while (stack.length > 0) {
      const [cx, cy] = stack.pop();
      const key = `${cx},${cy}`;
      if (visited.has(key)) continue;
      visited.add(key);

      if (board[cy][cx] === 0) {
        liberties++;
        continue;
      }
      if (board[cy][cx] !== color) continue;

      stones.push({ x: cx, y: cy });

      for (const [nx, ny] of this.neighbors(cx, cy)) {
        const nk = `${nx},${ny}`;
        if (!visited.has(nk)) stack.push([nx, ny]);
      }
    }

    return { stones, liberties };
  },

  neighbors(x, y) {
    const n = [];
    if (x > 0) n.push([x - 1, y]);
    if (x < this.boardSize - 1) n.push([x + 1, y]);
    if (y > 0) n.push([x, y - 1]);
    if (y < this.boardSize - 1) n.push([x, y + 1]);
    return n;
  },

  // Try to play a move. Returns { valid, captured } or { valid: false, reason }
  tryMove(x, y) {
    if (this.board[y][x] !== 0) return { valid: false, reason: 'occupied' };
    if (this.koPoint && this.koPoint.x === x && this.koPoint.y === y) {
      return { valid: false, reason: 'ko' };
    }

    const opponent = this.currentPlayer === 1 ? 2 : 1;
    const testBoard = this.copyBoard(this.board);
    testBoard[y][x] = this.currentPlayer;

    // Check for captures of opponent groups
    let captured = [];
    for (const [nx, ny] of this.neighbors(x, y)) {
      if (testBoard[ny][nx] === opponent) {
        const group = this.getGroup(testBoard, nx, ny);
        if (group.liberties === 0) {
          for (const s of group.stones) {
            testBoard[s.y][s.x] = 0;
            captured.push({ x: s.x, y: s.y });
          }
        }
      }
    }

    // Check for suicide
    const selfGroup = this.getGroup(testBoard, x, y);
    if (selfGroup.liberties === 0) {
      return { valid: false, reason: 'suicide' };
    }

    return { valid: true, captured, testBoard };
  },

  playMove(x, y) {
    const result = this.tryMove(x, y);
    if (!result.valid) return result;

    // Save history for undo
    this.history.push({
      board: this.copyBoard(this.board),
      player: this.currentPlayer,
      koPoint: this.koPoint,
      passes: this.passes,
    });

    // Apply move
    this.board = result.testBoard;
    this.passes = 0;

    // Ko detection: if exactly one stone captured, that point is ko
    if (result.captured.length === 1) {
      this.koPoint = { x: result.captured[0].x, y: result.captured[0].y };
    } else {
      this.koPoint = null;
    }

    this.currentPlayer = this.currentPlayer === 1 ? 2 : 1;
    return { valid: true };
  },

  pass() {
    this.history.push({
      board: this.copyBoard(this.board),
      player: this.currentPlayer,
      koPoint: this.koPoint,
      passes: this.passes,
    });

    this.passes++;
    this.koPoint = null;
    this.currentPlayer = this.currentPlayer === 1 ? 2 : 1;

    return this.passes >= 2;
  },

  undo() {
    if (this.history.length === 0) return false;
    const prev = this.history.pop();
    this.board = prev.board;
    this.currentPlayer = prev.player;
    this.koPoint = prev.koPoint;
    this.passes = prev.passes;
    return true;
  },
};
