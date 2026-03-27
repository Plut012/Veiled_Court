# Frontend Testing Checklist

## Phase 3: Frontend - Manual Testing

### Selection Screen Tests

1. **Load Selection Screen**
   - Navigate to http://localhost:3000
   - Verify 9 spirit cards display in grid
   - Verify each card shows emoji, name, and archetype

2. **Spirit Selection**
   - Click each spirit card
   - Verify card highlights with accent color
   - Verify theme changes (background, colors)
   - Verify "Start Game" button updates with spirit name

3. **Theme Preview**
   - Hover over each spirit card
   - Verify theme preview on hover
   - Verify theme restores on mouse leave

4. **Game Options**
   - Test board size selector (9×9, 13×13, 19×19)
   - Test color selector (Black/White)
   - Verify selections persist

5. **Start Game**
   - Click "Start Game" with no selection → should be disabled
   - Select a spirit, click "Start Game"
   - Verify navigation to game.html
   - Verify WebSocket connection established

### Game Screen Tests

1. **Board Rendering**
   - Verify board grid displays correctly
   - Verify star points (hoshi) display
   - Verify board size matches selection
   - Verify theme colors applied

2. **Stone Placement**
   - Click on empty intersection
   - Verify preview stone appears (semi-transparent)
   - Verify confirmation prompt
   - Verify stone placed after confirmation

3. **Controls**
   - Click "Pass" button → verify confirmation
   - Click "Resign" button → verify confirmation
   - Verify buttons disabled after game ends

4. **Info Display**
   - Verify move counter updates
   - Verify turn indicator updates
   - Verify last move indicator displays

### Theme System Tests

1. **All 9 Palettes**
   - Test each spirit's theme loads correctly
   - Verify color values match theming.md spec
   - Verify smooth transitions between themes

2. **Jaguar Dynamic Palette**
   - Start game as Jaguar
   - Play moves and observe palette drift
   - Verify gradual warm→cold transition
   - Verify snap to cold at move 120

3. **Crow Ko Dim**
   - Start game as Crow
   - Trigger ko situation
   - Verify board dims
   - Verify ko threats highlighted

### WebSocket Tests

1. **Connection**
   - Verify WebSocket connects on page load
   - Verify reconnection on disconnect
   - Verify error handling

2. **Message Handling**
   - Test InitGame message
   - Test Move message
   - Test Pass message
   - Test Resign message
   - Verify BoardUpdate received
   - Verify GameOver received

### Mobile/Responsive Tests

1. **Viewport**
   - Test on mobile viewport (375px)
   - Verify grid layout adjusts (2 columns)
   - Verify board scales correctly

2. **Touch Events**
   - Test touch on spirit cards
   - Test touch on board intersections
   - Verify preview and confirmation flow

## Known Issues / TODO

- [ ] Spirit portraits not yet generated (placeholder images)
- [ ] Ko detection not fully implemented (Crow middleware)
- [ ] Server-side KataGo integration pending
- [ ] Territory estimation not yet implemented

## Success Criteria

- [x] `frontend/` directory created with all files
- [x] Selection screen displays 9 spirits in grid
- [x] Game screen has canvas board + controls
- [x] All 9 palettes defined in themes.css
- [x] Board rendering works (grid, hoshi, stone preview)
- [x] WebSocket client structure complete
- [x] Server serves static files from frontend/dist
