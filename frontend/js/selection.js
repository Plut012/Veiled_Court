// Spirit selection screen logic
// Handles spirit card selection and game initialization

let selectedSpirit = null;
let selectedTheme = null;

// Set up event listeners when DOM loads
document.addEventListener('DOMContentLoaded', () => {
  const spiritCards = document.querySelectorAll('.spirit-card');
  const startBtn = document.getElementById('start-game-btn');

  // Add click handlers to spirit cards
  spiritCards.forEach(card => {
    card.addEventListener('click', () => {
      selectSpirit(card);
    });

    // Hover effect: preview theme
    card.addEventListener('mouseenter', () => {
      const theme = card.dataset.theme;
      previewTheme(theme);
    });

    card.addEventListener('mouseleave', () => {
      // Restore selected theme or default
      if (selectedTheme) {
        switchTheme(selectedSpirit);
      } else {
        document.body.className = 'theme-dragon'; // default
      }
    });
  });

  // Start game button
  startBtn.addEventListener('click', () => {
    if (selectedSpirit) {
      startGame();
    }
  });
});

function selectSpirit(card) {
  // Remove previous selection
  document.querySelectorAll('.spirit-card').forEach(c => {
    c.classList.remove('selected');
  });

  // Mark as selected
  card.classList.add('selected');

  // Store selection
  selectedSpirit = card.dataset.spirit;
  selectedTheme = card.dataset.theme;

  // Apply theme
  switchTheme(selectedSpirit);

  // Enable start button
  const startBtn = document.getElementById('start-game-btn');
  startBtn.disabled = false;
  startBtn.textContent = `Play as ${card.querySelector('h3').textContent}`;

  console.log('Selected spirit:', selectedSpirit);
}

function previewTheme(themeClass) {
  // Temporarily apply theme for preview
  document.body.className = themeClass;
}

function switchTheme(spiritName) {
  // Remove all theme classes
  document.body.classList.remove(
    'theme-dragon',
    'theme-mantis-shrimp',
    'theme-crane',
    'theme-spider',
    'theme-eagle',
    'theme-lion',
    'theme-praying-mantis',
    'theme-jaguar',
    'theme-jaguar-cold',
    'theme-crow'
  );

  // Add new theme class
  const themeClass = `theme-${spiritName.replace('_', '-')}`;
  document.body.classList.add(themeClass);
}

function startGame() {
  if (!selectedSpirit) {
    alert('Please select a spirit first');
    return;
  }

  // Get game options
  const boardSize = parseInt(document.getElementById('board-size').value);
  const playerColor = document.getElementById('player-color').value;

  console.log('Starting game:', {
    spirit: selectedSpirit,
    boardSize: boardSize,
    playerColor: playerColor
  });

  // Show loading state
  document.getElementById('start-game-btn').textContent = 'Connecting...';
  document.getElementById('start-game-btn').disabled = true;

  // Store selections in session storage for game screen
  sessionStorage.setItem('selectedSpirit', selectedSpirit);
  sessionStorage.setItem('selectedTheme', selectedTheme);

  // Create WebSocket client instance
  const client = new GameClient();
  window.gameClient = client;

  // Connect to server
  client.connect();

  // Wait for connection to open, then init game
  const checkConnection = setInterval(() => {
    if (client.ws && client.ws.readyState === WebSocket.OPEN) {
      clearInterval(checkConnection);

      // Initialize game
      client.initGame(selectedSpirit, boardSize, playerColor);
    }
  }, 100);

  // Timeout after 10 seconds
  setTimeout(() => {
    if (!client.sessionId) {
      clearInterval(checkConnection);
      alert('Connection timeout. Please try again.');
      document.getElementById('start-game-btn').textContent = `Play as ${selectedSpirit}`;
      document.getElementById('start-game-btn').disabled = false;
    }
  }, 10000);
}

// Load WebSocket client script
const script = document.createElement('script');
script.src = 'js/websocket.js';
document.head.appendChild(script);

// Load theme script
const themeScript = document.createElement('script');
themeScript.src = 'js/theme.js';
document.head.appendChild(themeScript);
