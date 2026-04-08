// Review screen controller
// Handles game replay with analysis overlays

document.addEventListener('DOMContentLoaded', () => {
  // Restore theme
  const selectedTheme = sessionStorage.getItem('selectedTheme');
  if (selectedTheme) {
    document.body.className = selectedTheme;
  }

  // Load review data
  const raw = sessionStorage.getItem('reviewData');
  if (!raw) {
    window.location.href = '/';
    return;
  }

  const data = JSON.parse(raw);
  const analysis = data.analysis;   // [{move_number, winrate, score_lead, ownership, best_move, played_move}, ...]
  const boards = data.boards;       // [board_state_at_move_0, board_state_at_move_1, ...]
  const boardSize = data.board_size;
  const totalMoves = boards.length - 1; // boards[0] = empty board

  // Init board
  boardRenderer.init('board', boardSize);

  // State
  let currentMove = 0;
  let autoAdvanceTimer = null;
  let holdTimer = null;
  const HOLD_THRESHOLD = 300;
  const AUTO_ADVANCE_INTERVAL = 800;
  const CRITICAL_THRESHOLD = 5;
  const MODERATE_THRESHOLD = 2;

  // Ownership wash fade state
  let washFadeId = null;
  let washStartTime = 0;
  const WASH_DURATION = 2000;

  // Elements
  const verdictEl = document.getElementById('verdict');
  const timelineFill = document.getElementById('timeline-fill');
  const timelineBar = document.getElementById('timeline-bar');

  // --- Build timeline critical dots ---
  for (let i = 1; i < analysis.length; i++) {
    const delta = Math.abs(analysis[i].score_lead - analysis[i - 1].score_lead);
    if (delta >= CRITICAL_THRESHOLD) {
      const dot = document.createElement('span');
      dot.className = 'timeline-dot';
      dot.style.left = `${(i / totalMoves) * 100}%`;
      timelineBar.appendChild(dot);
    }
  }

  // --- Rendering ---

  function getImpact(moveNum) {
    if (moveNum <= 0 || moveNum >= analysis.length) return 0;
    return analysis[moveNum].score_lead - analysis[moveNum - 1].score_lead;
  }

  function renderPosition(moveNum) {
    currentMove = moveNum;

    // Set board state
    boardRenderer.setBoard(boards[moveNum]);

    // Set last move marker (the played move at this position)
    boardRenderer.lastMove = null;
    if (moveNum > 0 && analysis[moveNum]) {
      const played = analysis[moveNum].played_move;
      if (played && played.toLowerCase() !== 'pass') {
        const coord = parseGTPCoord(played, boardSize);
        if (coord) {
          boardRenderer.lastMove = coord;
        }
      }
    }

    // Compute impact
    const rawDelta = getImpact(moveNum);
    const absDelta = Math.abs(rawDelta);

    // Draw overlays
    drawReviewOverlays(moveNum, rawDelta, absDelta);

    // Update verdict
    updateVerdict(moveNum, rawDelta, absDelta);

    // Update timeline
    timelineFill.style.width = `${(moveNum / totalMoves) * 100}%`;
  }

  function drawReviewOverlays(moveNum, rawDelta, absDelta) {
    // The base draw() was called by setBoard — now add overlays
    const displaySize = parseInt(boardRenderer.canvas.style.width);
    const cs = displaySize / (boardSize + 1);
    const m = cs;
    const ctx = boardRenderer.ctx;

    // 1. Move impact dot (scaled)
    if (moveNum > 0 && boardRenderer.lastMove) {
      const lm = boardRenderer.lastMove;
      const cx = m + lm.x * cs;
      const cy = m + lm.y * cs;
      const stoneColor = boardRenderer.board[lm.y][lm.x];

      const t = Math.min(absDelta / 10, 1);
      const radius = cs * (0.06 + t * 0.10);
      const alpha = 0.2 + t * 0.8;

      ctx.beginPath();
      ctx.arc(cx, cy, radius, 0, Math.PI * 2);
      ctx.fillStyle = stoneColor === 1 ? '#e8e0d4' : '#1a1815';
      ctx.globalAlpha = alpha;
      ctx.fill();
      ctx.globalAlpha = 1.0;
    }

    // 2. Ghost stone (best move, shown on moderate/critical)
    if (moveNum > 0 && absDelta >= MODERATE_THRESHOLD && analysis[moveNum]) {
      const bestMove = analysis[moveNum].best_move;
      const playedMove = analysis[moveNum].played_move;

      if (bestMove && bestMove !== playedMove && bestMove.toLowerCase() !== 'pass') {
        const coord = parseGTPCoord(bestMove, boardSize);
        if (coord && boardRenderer.board[coord.y][coord.x] === 0) {
          const cx = m + coord.x * cs;
          const cy = m + coord.y * cs;
          const r = cs * 0.45;

          // Ghost stone body
          ctx.beginPath();
          ctx.arc(cx, cy, r, 0, Math.PI * 2);
          ctx.fillStyle = '#888';
          ctx.globalAlpha = 0.3;
          ctx.fill();
          ctx.globalAlpha = 1.0;

          // Accent outline
          const accent = getComputedStyle(document.body).getPropertyValue('--accent').trim();
          ctx.beginPath();
          ctx.arc(cx, cy, r, 0, Math.PI * 2);
          ctx.strokeStyle = accent;
          ctx.lineWidth = 1.5;
          ctx.globalAlpha = 0.6;
          ctx.stroke();
          ctx.globalAlpha = 1.0;
        }
      }
    }

    // 3. Ownership wash (with fade animation)
    if (moveNum > 0 && absDelta >= MODERATE_THRESHOLD && analysis[moveNum]) {
      startOwnershipWash(moveNum);
    }
  }

  function startOwnershipWash(moveNum) {
    if (washFadeId) cancelAnimationFrame(washFadeId);
    washStartTime = performance.now();

    const ownership = analysis[moveNum].ownership;
    if (!ownership || ownership.length === 0) return;

    function fadeFrame(now) {
      const elapsed = now - washStartTime;
      const progress = Math.max(0, 1 - elapsed / WASH_DURATION);

      if (progress <= 0) return;

      // Redraw base board + overlays without wash
      boardRenderer.draw();
      // Redraw impact dot and ghost (simplified — just the dot)
      if (boardRenderer.lastMove) {
        const displaySize = parseInt(boardRenderer.canvas.style.width);
        const cs = displaySize / (boardSize + 1);
        const m = cs;
        const ctx = boardRenderer.ctx;
        const lm = boardRenderer.lastMove;
        const cx = m + lm.x * cs;
        const cy = m + lm.y * cs;
        const stoneColor = boardRenderer.board[lm.y][lm.x];
        const absDelta = Math.abs(getImpact(currentMove));
        const t = Math.min(absDelta / 10, 1);
        const radius = cs * (0.06 + t * 0.10);
        ctx.beginPath();
        ctx.arc(cx, cy, radius, 0, Math.PI * 2);
        ctx.fillStyle = stoneColor === 1 ? '#e8e0d4' : '#1a1815';
        ctx.globalAlpha = 0.2 + t * 0.8;
        ctx.fill();
        ctx.globalAlpha = 1.0;
      }

      // Draw ownership wash
      drawOwnershipLayer(ownership, progress);

      washFadeId = requestAnimationFrame(fadeFrame);
    }

    washFadeId = requestAnimationFrame(fadeFrame);
  }

  function drawOwnershipLayer(ownership, fadeProgress) {
    const displaySize = parseInt(boardRenderer.canvas.style.width);
    const cs = displaySize / (boardSize + 1);
    const m = cs;
    const ctx = boardRenderer.ctx;

    for (let y = 0; y < boardSize; y++) {
      for (let x = 0; x < boardSize; x++) {
        const val = ownership[y * boardSize + x];
        if (Math.abs(val) < 0.05) continue;

        const cx = m + x * cs;
        const cy = m + y * cs;
        const r = cs * 0.35;

        ctx.beginPath();
        ctx.arc(cx, cy, r, 0, Math.PI * 2);
        ctx.fillStyle = val > 0 ? '#1a1815' : '#e8e0d4';
        ctx.globalAlpha = Math.abs(val) * 0.2 * fadeProgress;
        ctx.fill();
      }
    }
    ctx.globalAlpha = 1.0;
  }

  function updateVerdict(moveNum, rawDelta, absDelta) {
    if (moveNum === 0) {
      verdictEl.textContent = '';
      verdictEl.className = '';
      return;
    }

    // Determine if this move was good or bad for the player who made it
    // score_lead is from Black's perspective
    // If Black played and score_lead went down, that's bad for Black
    // If White played and score_lead went up, that's bad for White
    const isBlackMove = analysis[moveNum].played_move && (moveNum % 2 === 1);
    const playerDelta = isBlackMove ? rawDelta : -rawDelta;

    const sign = playerDelta >= 0 ? '+' : '';
    const pts = `${sign}${playerDelta.toFixed(1)} pts`;

    if (absDelta >= CRITICAL_THRESHOLD) {
      verdictEl.textContent = playerDelta < 0 ? `Critical mistake  ${pts}` : `Brilliant move  ${pts}`;
      verdictEl.className = 'critical';
    } else if (absDelta >= MODERATE_THRESHOLD) {
      verdictEl.textContent = `Inaccuracy  ${pts}`;
      verdictEl.className = 'moderate';
    } else {
      verdictEl.textContent = `Correct  ${pts}`;
      verdictEl.className = '';
    }
  }

  // --- Navigation ---

  function stepForward() {
    if (currentMove < totalMoves) {
      renderPosition(currentMove + 1);
    }
  }

  function stepBackward() {
    if (currentMove > 0) {
      renderPosition(currentMove - 1);
    }
  }

  function startAutoAdvance(direction) {
    stopAutoAdvance();
    autoAdvanceTimer = setInterval(() => {
      const nextMove = currentMove + direction;
      if (nextMove < 0 || nextMove > totalMoves) {
        stopAutoAdvance();
        return;
      }

      renderPosition(nextMove);

      // Pause on critical moves
      const absDelta = Math.abs(getImpact(nextMove));
      if (absDelta >= CRITICAL_THRESHOLD) {
        stopAutoAdvance();
      }
    }, AUTO_ADVANCE_INTERVAL);
  }

  function stopAutoAdvance() {
    if (autoAdvanceTimer) {
      clearInterval(autoAdvanceTimer);
      autoAdvanceTimer = null;
    }
  }

  // --- Button interaction (tap vs hold) ---

  function setupButton(btn, direction) {
    const stepFn = direction === 1 ? stepForward : stepBackward;

    btn.addEventListener('pointerdown', (e) => {
      e.preventDefault();
      holdTimer = setTimeout(() => {
        holdTimer = null;
        startAutoAdvance(direction);
      }, HOLD_THRESHOLD);
    });

    btn.addEventListener('pointerup', (e) => {
      e.preventDefault();
      if (holdTimer) {
        // Was a tap (released before hold threshold)
        clearTimeout(holdTimer);
        holdTimer = null;
        stepFn();
      } else {
        // Was a hold — stop auto-advance
        stopAutoAdvance();
      }
    });

    btn.addEventListener('pointerleave', () => {
      if (holdTimer) {
        clearTimeout(holdTimer);
        holdTimer = null;
      }
      stopAutoAdvance();
    });
  }

  setupButton(document.getElementById('nav-left'), -1);
  setupButton(document.getElementById('nav-right'), 1);

  // Keyboard navigation
  const keyState = {};
  document.addEventListener('keydown', (e) => {
    if (keyState[e.key]) return; // prevent key repeat
    keyState[e.key] = true;

    if (e.key === 'ArrowRight') {
      stepForward();
      holdTimer = setTimeout(() => startAutoAdvance(1), HOLD_THRESHOLD);
    } else if (e.key === 'ArrowLeft') {
      stepBackward();
      holdTimer = setTimeout(() => startAutoAdvance(-1), HOLD_THRESHOLD);
    }
  });

  document.addEventListener('keyup', (e) => {
    keyState[e.key] = false;
    if (e.key === 'ArrowRight' || e.key === 'ArrowLeft') {
      if (holdTimer) {
        clearTimeout(holdTimer);
        holdTimer = null;
      }
      stopAutoAdvance();
    }
  });

  // Resize handler
  window.addEventListener('resize', () => {
    boardRenderer.setupCanvasSize();
    renderPosition(currentMove);
  });

  // --- Utility ---

  function parseGTPCoord(coordStr, size) {
    if (!coordStr || coordStr.length < 2) return null;
    let col = coordStr.charCodeAt(0) - 65;
    if (col > 7) col--;
    const row = size - parseInt(coordStr.substring(1));
    if (col >= 0 && col < size && row >= 0 && row < size) {
      return { x: col, y: row };
    }
    return null;
  }

  // --- Start ---
  renderPosition(0);
});
