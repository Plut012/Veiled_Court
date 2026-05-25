// Spirit selection screen logic
// Handles spirit card selection and game initialization

let selectedSpirit = null;
let selectedTheme = null;
let podReady = false;

// Summon the GPU pod as soon as the selection page loads.
// The pod boots while the player browses spirits.
let podUrl = null;

fetch('/summon', { method: 'POST' })
  .then(r => r.json())
  .then(data => {
    if (data.status === 'ready' && data.url) {
      podReady = true;
      podUrl = data.url;
    } else {
      pollStatus();
    }
  })
  .catch(() => {
    // Running without coordinator (direct to game server) — that's fine
    podReady = true;
  });

function pollStatus() {
  const poll = setInterval(() => {
    fetch('/status').then(r => r.json()).then(data => {
      if (data.ready && data.url) {
        podReady = true;
        podUrl = data.url;
        clearInterval(poll);
      }
    }).catch(() => {});
  }, 3000);
}

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
        document.body.className = '';
      }
    });
  });

  // Size picker
  document.querySelectorAll('.size-btn').forEach(btn => {
    btn.addEventListener('click', () => {
      document.querySelectorAll('.size-btn').forEach(b => b.classList.remove('selected'));
      btn.classList.add('selected');
    });
  });

  // Color picker
  document.querySelectorAll('.color-btn').forEach(btn => {
    btn.addEventListener('click', () => {
      document.querySelectorAll('.color-btn').forEach(b => b.classList.remove('selected'));
      btn.classList.add('selected');
    });
  });

  // Secret: click title for local pass-and-play
  document.querySelector('#selection-screen h1').addEventListener('click', () => {
    window.location.href = '/local.html';
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
  if (!selectedSpirit) return;

  // Get game options from picker buttons
  const activeSize = document.querySelector('.size-btn.selected');
  const activeColor = document.querySelector('.color-btn.selected');
  const boardSize = parseInt(activeSize ? activeSize.dataset.size : '19');
  const playerColor = activeColor ? activeColor.dataset.color : 'black';

  // Store selections
  sessionStorage.setItem('selectedSpirit', selectedSpirit);
  sessionStorage.setItem('selectedTheme', selectedTheme);
  sessionStorage.setItem('playerColor', playerColor);
  sessionStorage.setItem('boardSize', boardSize);

  // Encode game config as URL params (survives cross-domain redirect)
  const params = new URLSearchParams({
    spirit: selectedSpirit,
    theme: selectedTheme,
    color: playerColor,
    size: boardSize,
  });

  if (podUrl) {
    window.location.href = podUrl + '/game.html?' + params;
  } else {
    // Pass params in URL so loading page can forward them to the pod
    window.location.href = '/loading.html?' + params;
  }
}

// Load theme script for hover previews
const themeScript = document.createElement('script');
themeScript.src = 'js/theme.js';
document.head.appendChild(themeScript);
