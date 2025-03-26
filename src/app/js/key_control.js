// Implementation of the KeyControl class implementing MapLibre's IControl interface
class KeyControl {
  constructor() {
    this._map = null;
    this._container = null;
    this._button = null;
    this._keyPanel = null;
  }

  onAdd(map) {
    this._map = map;

    // Create container for the control
    this._container = document.createElement('div');
    this._container.className = 'maplibregl-ctrl maplibregl-ctrl-group';

    // Create the button
    this._button = document.createElement('button');
    this._button.className = 'maplibregl-ctrl-icon oim-key-control';
    this._button.type = 'button';
    this._button.title = 'Show map key';
    this._button.setAttribute('aria-label', 'Show map key');

    // Create the key panel (initially hidden)
    this._keyPanel = document.createElement('div');
    this._keyPanel.className = 'oim-key-panel';

    // Add header
    const header = document.createElement('div');
    header.className = 'oim-key-header';

    const title = document.createElement('h2');
    title.textContent = 'Key';

    const closeButton = document.createElement('button');
    closeButton.className = 'oim-key-close';
    closeButton.textContent = '×';

    header.appendChild(title);
    header.appendChild(closeButton);

    // Add body
    const body = document.createElement('div');
    body.className = 'oim-key-body';

    // Populate key content
    this._populateKeyContent(body);

    // Add components to the panel
    this._keyPanel.appendChild(header);
    this._keyPanel.appendChild(body);

    // Add panel to document body
    document.body.appendChild(this._keyPanel);

    // Add button to container
    this._container.appendChild(this._button);

    // Add event listeners
    this._button.addEventListener('click', () => {
      this._keyPanel.classList.add('visible');
    });

    closeButton.addEventListener('click', () => {
      this._keyPanel.classList.remove('visible');
    });

    return this._container;
  }

  onRemove() {
    if (this._container && this._container.parentNode) {
      this._container.parentNode.removeChild(this._container);
    }

    if (this._keyPanel && this._keyPanel.parentNode) {
      this._keyPanel.parentNode.removeChild(this._keyPanel);
    }

    this._map = null;
  }

  getDefaultPosition() {
    return 'top-right';
  }

  _populateKeyContent(container) {
    // Add Underground Lines section
    container.appendChild(this._createSection('Underground Lines', [
      { name: 'Bakerloo', className: 'color-line bakerloo' },
      { name: 'Central', className: 'color-line central' },
      { name: 'Circle', className: 'color-line circle' },
      { name: 'District', className: 'color-line district' },
      { name: 'Hammersmith & City', className: 'color-line hammersmith' },
      { name: 'Jubilee', className: 'color-line jubilee' },
      { name: 'Metropolitan', className: 'color-line metropolitan' },
      { name: 'Northern', className: 'color-line northern' },
      { name: 'Piccadilly', className: 'color-line piccadilly' },
      { name: 'Victoria', className: 'color-line victoria' },
      { name: 'Waterloo & City', className: 'color-line waterloo' }
    ]));

    // Add Other Rail section
    container.appendChild(this._createSection('Other Rail', [
      { name: 'Overground', className: 'color-line overground' },
      { name: 'DLR', className: 'color-line dlr' },
      { name: 'Elizabeth Line', className: 'color-line elizabeth' },
      { name: 'Trams', className: 'color-line tram' },
      { name: 'Cable Car', className: 'color-line cablecar' }
    ]));

    // Add Other Features section
    container.appendChild(this._createSection('Other Features', [
      { name: 'Station', className: 'map-symbol station' },
      { name: 'Interchange', className: 'map-symbol interchange' },
      { name: 'Depot', className: 'map-symbol depot' }
    ]));
  }

  _createSection(title, items) {
    const section = document.createElement('div');

    const heading = document.createElement('h3');
    heading.textContent = title;
    section.appendChild(heading);

    const table = document.createElement('table');

    items.forEach(item => {
      const row = document.createElement('tr');

      const labelCell = document.createElement('td');
      labelCell.textContent = item.name;

      const symbolCell = document.createElement('td');
      const symbolDiv = document.createElement('div');
      symbolDiv.className = item.className;
      symbolCell.appendChild(symbolDiv);

      row.appendChild(labelCell);
      row.appendChild(symbolCell);
      table.appendChild(row);
    });

    section.appendChild(table);
    return section;
  }
}

// Export the control constructor
window.KeyControl = KeyControl;
