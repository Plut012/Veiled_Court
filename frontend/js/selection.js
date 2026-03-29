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

  // Store selections and navigate immediately — game.html handles the connection
  sessionStorage.setItem('selectedSpirit', selectedSpirit);
  sessionStorage.setItem('selectedTheme', selectedTheme);
  sessionStorage.setItem('playerColor', playerColor);
  sessionStorage.setItem('boardSize', boardSize);

  window.location.href = '/game.html';
}

// Load theme script for hover previews
const themeScript = document.createElement('script');
themeScript.src = 'js/theme.js';
document.head.appendChild(themeScript);
