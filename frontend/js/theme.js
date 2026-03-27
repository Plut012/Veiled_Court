// Theme switching and Jaguar palette drift
// Handles dynamic color changes

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

  console.log('Switched to theme:', themeClass);

  // Redraw board with new colors if board renderer exists
  if (window.boardRenderer) {
    boardRenderer.draw();
  }
}

function updateJaguarPalette(moveNumber) {
  // Jaguar palette drifts from warm to cold over 120 moves

  if (moveNumber >= 120) {
    // Snap to cold palette at move 120
    document.body.classList.remove('theme-jaguar');
    document.body.classList.add('theme-jaguar-cold');

    // Update portrait if available
    const portrait = document.getElementById('spirit-portrait');
    if (portrait) {
      portrait.src = 'assets/portraits/jaguar_cold.png';
    }
  } else {
    // Linear interpolation from warm to cold (0 to 120)
    const progress = moveNumber / 120;

    // Warm colors
    const warmPrimary = { r: 28, g: 21, b: 8 };      // #1C1508
    const warmSecondary = { r: 92, g: 61, b: 26 };   // #5C3D1A
    const warmAccent = { r: 201, g: 168, b: 112 };   // #C9A870

    // Cold colors
    const coldPrimary = { r: 10, g: 12, b: 16 };     // #0A0C10
    const coldSecondary = { r: 42, g: 48, b: 64 };   // #2A3040
    const coldAccent = { r: 176, g: 196, b: 216 };   // #B0C4D8

    // Interpolate
    const boardPrimary = interpolateColor(warmPrimary, coldPrimary, progress);
    const boardSecondary = interpolateColor(warmSecondary, coldSecondary, progress);
    const accent = interpolateColor(warmAccent, coldAccent, progress);

    // Apply to CSS custom properties
    const root = document.documentElement;
    root.style.setProperty('--board-primary', rgbToHex(boardPrimary));
    root.style.setProperty('--board-secondary', rgbToHex(boardSecondary));
    root.style.setProperty('--accent', rgbToHex(accent));

    console.log(`Jaguar palette at move ${moveNumber}:`, {
      primary: rgbToHex(boardPrimary),
      secondary: rgbToHex(boardSecondary),
      accent: rgbToHex(accent)
    });
  }

  // Redraw board with updated colors
  if (window.boardRenderer) {
    boardRenderer.draw();
  }
}

function interpolateColor(color1, color2, progress) {
  return {
    r: Math.round(color1.r + (color2.r - color1.r) * progress),
    g: Math.round(color1.g + (color2.g - color1.g) * progress),
    b: Math.round(color1.b + (color2.b - color1.b) * progress)
  };
}

function rgbToHex(color) {
  const r = color.r.toString(16).padStart(2, '0');
  const g = color.g.toString(16).padStart(2, '0');
  const b = color.b.toString(16).padStart(2, '0');
  return `#${r}${g}${b}`;
}

function hexToRgb(hex) {
  const result = /^#?([a-f\d]{2})([a-f\d]{2})([a-f\d]{2})$/i.exec(hex);
  return result ? {
    r: parseInt(result[1], 16),
    g: parseInt(result[2], 16),
    b: parseInt(result[3], 16)
  } : null;
}

// Make functions globally available
window.switchTheme = switchTheme;
window.updateJaguarPalette = updateJaguarPalette;
