// Art nouveau line work on the selection screen (ornament.css).
// Each spirit card gets four corner tendrils, hidden until the card is chosen;
// they draw themselves in around the chosen spirit, in its colour.

(() => {
  // Top-left corner; the other three are mirrored in CSS.
  const CORNER = `
    <svg class="flourish" viewBox="0 0 40 40" aria-hidden="true">
      <path pathLength="1" d="M4 38 C4 22 7 11 15 7 C21 4 28 4 38 4"/>
      <path pathLength="1" d="M15 7 C11 12 13.5 18 18 16.5 C21 15.5 20.5 11.5 17.8 12 C16.3 12.3 16.4 14 17.4 14.2"/>
      <path pathLength="1" d="M28 4.2 C30.5 1.2 34.5 0.8 36.5 2.6"/>
    </svg>`;

  document.querySelectorAll('.spirit-card').forEach(card => {
    const frame = document.createElement('div');
    frame.className = 'flourishes';
    frame.innerHTML = CORNER.repeat(4);
    card.appendChild(frame);
  });
})();
