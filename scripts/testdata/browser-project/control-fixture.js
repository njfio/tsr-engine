// Authored harness fixture, never an engine-produced game or player evidence.
// Exercises a playable-scale state/render/input loop without external resources.
(() => {
  'use strict';
  const start = document.querySelector('#start');
  const board = document.querySelector('canvas');
  const context = board.getContext('2d');
  const readout = document.querySelector('#readout');
  const history = document.querySelector('#history');
  const state = {
    running: false, paused: false, x: 2, y: 2, score: 0, moves: 0,
    targets: [{x: 3, y: 2}, {x: 3, y: 3}, {x: 2, y: 3}],
    visited: [], lastInput: 'none'
  };
  const grid = {columns: 12, rows: 8, size: 24};
  function describe() {
    const phase = !state.running ? 'Ready' : state.paused ? 'Paused' : 'Active';
    readout.textContent = `${phase}; moves ${state.moves}; score ${state.score}; input ${state.lastInput}`;
    history.textContent = state.visited.map(position => `(${position.x},${position.y})`).join(' ');
    board.setAttribute('aria-label', `Position ${state.x}, ${state.y}. Score ${state.score}`);
  }
  function drawCell(x, y, color) {
    context.fillStyle = color;
    context.fillRect(x * grid.size + 2, y * grid.size + 2, grid.size - 4, grid.size - 4);
  }
  function render() {
    context.fillStyle = '#15202a';
    context.fillRect(0, 0, board.width, board.height);
    for (let y = 0; y < grid.rows; y++) {
      for (let x = 0; x < grid.columns; x++) {
        context.strokeStyle = '#314151';
        context.strokeRect(x * grid.size, y * grid.size, grid.size, grid.size);
      }
    }
    state.targets.forEach(target => drawCell(target.x, target.y, '#f2cb56'));
    drawCell(state.x, state.y, state.paused ? '#777777' : '#6bdbed');
    describe();
  }
  function move(dx, dy) {
    if (!state.running || state.paused) return;
    const nextX = state.x + dx;
    const nextY = state.y + dy;
    if (nextX < 0 || nextX >= grid.columns || nextY < 0 || nextY >= grid.rows) return;
    state.x = nextX;
    state.y = nextY;
    state.moves++;
    state.visited.push({x: state.x, y: state.y});
    if (state.visited.length > 12) state.visited.shift();
    const target = state.targets.findIndex(item => item.x === state.x && item.y === state.y);
    if (target >= 0) {
      state.targets.splice(target, 1);
      state.score++;
    }
  }
  start.addEventListener('click', () => {
    Object.assign(state, {running: true, paused: false, x: 2, y: 2, score: 0, moves: 0, visited: [], lastInput: 'start'});
    state.targets = [{x: 3, y: 2}, {x: 3, y: 3}, {x: 2, y: 3}];
    start.textContent = 'Play';
    render();
  });
  window.addEventListener('keydown', event => {
    const directions = {ArrowLeft: [-1, 0], ArrowRight: [1, 0], ArrowUp: [0, -1], ArrowDown: [0, 1]};
    if (directions[event.key]) {
      event.preventDefault();
      state.lastInput = event.key;
      move(...directions[event.key]);
    } else if (event.code === 'Space') {
      event.preventDefault();
      state.lastInput = 'pause';
      if (state.running) state.paused = !state.paused;
    }
    render();
  });
  render();
})();
