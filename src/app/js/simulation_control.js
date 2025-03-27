// Implementation of SimulationControl class implementing MapLibre's IControl interface
class SimulationControl {
  constructor() {
    this._map = null;
    this._container = null;
    this._simulationPanel = null;
  }

  onAdd(map) {
    this._map = map;

    // Create container for the control
    this._container = document.createElement('div');
    this._container.className = 'maplibregl-ctrl maplibregl-ctrl-group';

    // Create the button
    this._button = document.createElement('button');
    this._button.className = 'maplibregl-ctrl-simulation';
    this._button.type = 'button';
    this._button.title = 'Simulation Controls';
    this._button.setAttribute('aria-label', 'Simulation Controls');
    this._button.innerHTML = '▶';

    // Create the simulation panel (initially hidden)
    this._simulationPanel = document.createElement('div');
    this._simulationPanel.className = 'simulation-panel';
    this._simulationPanel.style.display = 'none';

    // Add header
    const header = document.createElement('h3');
    header.textContent = 'TfL Vehicle Simulation';

    // Add controls
    const controls = document.createElement('div');
    controls.className = 'simulation-controls';
    
    const playPauseButton = document.createElement('button');
    playPauseButton.id = 'play-pause-simulation';
    playPauseButton.textContent = 'Play/Pause';
    
    const resetButton = document.createElement('button');
    resetButton.id = 'reset-simulation';
    resetButton.textContent = 'Reset';
    
    controls.appendChild(playPauseButton);
    controls.appendChild(resetButton);

    // Add close button
    const closeButton = document.createElement('button');
    closeButton.className = 'close-button';
    closeButton.textContent = 'Close';

    // Add components to the panel
    this._simulationPanel.appendChild(header);
    this._simulationPanel.appendChild(controls);
    this._simulationPanel.appendChild(closeButton);

    // Add panel to document body
    document.body.appendChild(this._simulationPanel);

    // Add button to container
    this._container.appendChild(this._button);

    // Add event listeners
    this._button.addEventListener('click', () => {
      this._simulationPanel.style.display = this._simulationPanel.style.display === 'none' ? 'block' : 'none';
    });

    closeButton.addEventListener('click', () => {
      this._simulationPanel.style.display = 'none';
    });

    playPauseButton.addEventListener('click', () => {
      if (window.SimulationController) {
        window.SimulationController.toggle();
      }
    });

    resetButton.addEventListener('click', () => {
      if (window.SimulationController) {
        window.SimulationController.reset();
      }
    });

    return this._container;
  }

  onRemove() {
    if (this._container && this._container.parentNode) {
      this._container.parentNode.removeChild(this._container);
    }

    if (this._simulationPanel && this._simulationPanel.parentNode) {
      this._simulationPanel.parentNode.removeChild(this._simulationPanel);
    }

    this._map = null;
  }

  getDefaultPosition() {
    return 'top-right';
  }
}

// Export the control constructor
window.SimulationControl = SimulationControl;