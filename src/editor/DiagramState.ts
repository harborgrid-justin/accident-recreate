/**
 * Diagram State Management
 * AccuScene Enterprise Accident Recreation Platform
 */

import {
  DiagramElement,
  DiagramState,
  HistoryEntry,
  Point,
  Transform,
  Measurement,
  Size,
} from '../types/diagram';

export class DiagramStateManager {
  private state: DiagramState;
  private history: HistoryEntry[] = [];
  private historyIndex: number = -1;
  private maxHistorySize: number = 50;
  private listeners: Set<(state: DiagramState) => void> = new Set();

  constructor(initialState?: Partial<DiagramState>) {
    this.state = {
      elements: [],
      measurements: [],
      selectedIds: [],
      scale: 20, // 20 pixels per meter
      gridVisible: true,
      gridSize: 1, // 1 meter
      backgroundColor: '#F3F4F6',
      canvasSize: { width: 1920, height: 1080 },
      ...initialState,
    };

    this.saveHistory('Initial state');
  }

  /**
   * Get current state
   */
  getState(): DiagramState {
    return { ...this.state };
  }

  /**
   * Subscribe to state changes
   */
  subscribe(listener: (state: DiagramState) => void): () => void {
    this.listeners.add(listener);
    return () => this.listeners.delete(listener);
  }

  /**
   * Notify all listeners
   */
  private notifyListeners(): void {
    this.listeners.forEach((listener) => listener(this.getState()));
  }

  /**
   * Save current state to history
   */
  private saveHistory(description: string): void {
    // Remove any history after current index
    this.history = this.history.slice(0, this.historyIndex + 1);

    // Add new history entry
    this.history.push({
      timestamp: Date.now(),
      state: JSON.parse(JSON.stringify(this.state)),
      description,
    });

    // Limit history size
    if (this.history.length > this.maxHistorySize) {
      this.history.shift();
    } else {
      this.historyIndex++;
    }
  }

  /**
   * Undo last action
   */
  undo(): boolean {
    if (this.historyIndex > 0) {
      this.historyIndex--;
      this.state = JSON.parse(JSON.stringify(this.history[this.historyIndex].state));
      this.notifyListeners();
      return true;
    }
    return false;
  }

  /**
   * Redo last undone action
   */
  redo(): boolean {
    if (this.historyIndex < this.history.length - 1) {
      this.historyIndex++;
      this.state = JSON.parse(JSON.stringify(this.history[this.historyIndex].state));
      this.notifyListeners();
      return true;
    }
    return false;
  }

  /**
   * Can undo
   */
  canUndo(): boolean {
    return this.historyIndex > 0;
  }

  /**
   * Can redo
   */
  canRedo(): boolean {
    return this.historyIndex < this.history.length - 1;
  }

  /**
   * Add element
   */
  addElement(element: DiagramElement): void {
    this.state.elements.push(element);
    this.saveHistory(`Added ${element.type}: ${element.subType}`);
    this.notifyListeners();
  }

  /**
   * Add multiple elements
   */
  addElements(elements: DiagramElement[]): void {
    this.state.elements.push(...elements);
    this.saveHistory(`Added ${elements.length} elements`);
    this.notifyListeners();
  }

  /**
   * Remove element
   */
  removeElement(id: string): void {
    const element = this.state.elements.find((e) => e.id === id);
    this.state.elements = this.state.elements.filter((e) => e.id !== id);
    this.state.selectedIds = this.state.selectedIds.filter((sid) => sid !== id);
    if (element) {
      this.saveHistory(`Removed ${element.type}: ${element.subType}`);
    }
    this.notifyListeners();
  }

  /**
   * Remove selected elements
   */
  removeSelectedElements(): void {
    const count = this.state.selectedIds.length;
    this.state.elements = this.state.elements.filter(
      (e) => !this.state.selectedIds.includes(e.id)
    );
    this.state.selectedIds = [];
    this.saveHistory(`Removed ${count} elements`);
    this.notifyListeners();
  }

  /**
   * Update element
   */
  updateElement(id: string, updates: Partial<DiagramElement>): void {
    const element = this.state.elements.find((e) => e.id === id);
    if (element) {
      Object.assign(element, updates);
      this.notifyListeners();
    }
  }

  /**
   * Update element transform
   */
  updateElementTransform(id: string, transform: Partial<Transform>): void {
    const element = this.state.elements.find((e) => e.id === id);
    if (element) {
      element.transform = { ...element.transform, ...transform };
      this.notifyListeners();
    }
  }

  /**
   * Update element position
   */
  updateElementPosition(id: string, position: Point): void {
    const element = this.state.elements.find((e) => e.id === id);
    if (element) {
      element.transform.position = position;
      this.notifyListeners();
    }
  }

  /**
   * Move element
   */
  moveElement(id: string, delta: Point): void {
    const element = this.state.elements.find((e) => e.id === id);
    if (element) {
      element.transform.position.x += delta.x;
      element.transform.position.y += delta.y;
      this.notifyListeners();
    }
  }

  /**
   * Move selected elements
   */
  moveSelectedElements(delta: Point, saveToHistory: boolean = false): void {
    this.state.selectedIds.forEach((id) => {
      const element = this.state.elements.find((e) => e.id === id);
      if (element && !element.locked) {
        element.transform.position.x += delta.x;
        element.transform.position.y += delta.y;
      }
    });

    if (saveToHistory) {
      this.saveHistory(`Moved ${this.state.selectedIds.length} elements`);
    }

    this.notifyListeners();
  }

  /**
   * Rotate element
   */
  rotateElement(id: string, angle: number, saveToHistory: boolean = false): void {
    const element = this.state.elements.find((e) => e.id === id);
    if (element && !element.locked) {
      element.transform.rotation = angle;

      if (saveToHistory) {
        this.saveHistory(`Rotated ${element.type}`);
      }

      this.notifyListeners();
    }
  }

  /**
   * Scale element
   */
  scaleElement(id: string, scale: Point, saveToHistory: boolean = false): void {
    const element = this.state.elements.find((e) => e.id === id);
    if (element && !element.locked) {
      element.transform.scale = scale;

      if (saveToHistory) {
        this.saveHistory(`Scaled ${element.type}`);
      }

      this.notifyListeners();
    }
  }

  /**
   * Get element by ID
   */
  getElement(id: string): DiagramElement | undefined {
    return this.state.elements.find((e) => e.id === id);
  }

  /**
   * Get selected elements
   */
  getSelectedElements(): DiagramElement[] {
    return this.state.elements.filter((e) => this.state.selectedIds.includes(e.id));
  }

  /**
   * Select element
   */
  selectElement(id: string, addToSelection: boolean = false): void {
    if (addToSelection) {
      if (!this.state.selectedIds.includes(id)) {
        this.state.selectedIds.push(id);
      }
    } else {
      this.state.selectedIds = [id];
    }
    this.notifyListeners();
  }

  /**
   * Select multiple elements
   */
  selectElements(ids: string[], addToSelection: boolean = false): void {
    if (addToSelection) {
      ids.forEach((id) => {
        if (!this.state.selectedIds.includes(id)) {
          this.state.selectedIds.push(id);
        }
      });
    } else {
      this.state.selectedIds = ids;
    }
    this.notifyListeners();
  }

  /**
   * Deselect element
   */
  deselectElement(id: string): void {
    this.state.selectedIds = this.state.selectedIds.filter((sid) => sid !== id);
    this.notifyListeners();
  }

  /**
   * Clear selection
   */
  clearSelection(): void {
    this.state.selectedIds = [];
    this.notifyListeners();
  }

  /**
   * Select all elements
   */
  selectAll(): void {
    this.state.selectedIds = this.state.elements.map((e) => e.id);
    this.notifyListeners();
  }

  /**
   * Bring element to front
   */
  bringToFront(id: string): void {
    const element = this.state.elements.find((e) => e.id === id);
    if (element) {
      const maxZIndex = Math.max(...this.state.elements.map((e) => e.zIndex), 0);
      element.zIndex = maxZIndex + 1;
      this.saveHistory('Brought element to front');
      this.notifyListeners();
    }
  }

  /**
   * Send element to back
   */
  sendToBack(id: string): void {
    const element = this.state.elements.find((e) => e.id === id);
    if (element) {
      const minZIndex = Math.min(...this.state.elements.map((e) => e.zIndex), 0);
      element.zIndex = minZIndex - 1;
      this.saveHistory('Sent element to back');
      this.notifyListeners();
    }
  }

  /**
   * Toggle grid visibility
   */
  toggleGrid(): void {
    this.state.gridVisible = !this.state.gridVisible;
    this.notifyListeners();
  }

  /**
   * Set grid size
   */
  setGridSize(size: number): void {
    this.state.gridSize = size;
    this.notifyListeners();
  }

  /**
   * Set scale (pixels per meter)
   */
  setScale(scale: number): void {
    this.state.scale = Math.max(1, Math.min(100, scale));
    this.notifyListeners();
  }

  /**
   * Set canvas size
   */
  setCanvasSize(size: Size): void {
    this.state.canvasSize = size;
    this.notifyListeners();
  }

  /**
   * Add measurement
   */
  addMeasurement(measurement: Measurement): void {
    this.state.measurements.push(measurement);
    this.saveHistory(`Added ${measurement.type} measurement`);
    this.notifyListeners();
  }

  /**
   * Remove measurement
   */
  removeMeasurement(id: string): void {
    this.state.measurements = this.state.measurements.filter((m) => m.id !== id);
    this.saveHistory('Removed measurement');
    this.notifyListeners();
  }

  /**
   * Clear all measurements
   */
  clearMeasurements(): void {
    this.state.measurements = [];
    this.saveHistory('Cleared all measurements');
    this.notifyListeners();
  }

  /**
   * Serialize state to JSON
   */
  serialize(): string {
    return JSON.stringify(this.state, null, 2);
  }

  /**
   * Deserialize state from JSON
   */
  deserialize(json: string): boolean {
    try {
      const newState = JSON.parse(json);
      this.state = newState;
      this.saveHistory('Loaded diagram');
      this.notifyListeners();
      return true;
    } catch (error) {
      console.error('Failed to deserialize state:', error);
      return false;
    }
  }

  /**
   * Clear diagram
   */
  clear(): void {
    this.state.elements = [];
    this.state.measurements = [];
    this.state.selectedIds = [];
    this.saveHistory('Cleared diagram');
    this.notifyListeners();
  }

  /**
   * Get history
   */
  getHistory(): HistoryEntry[] {
    return [...this.history];
  }

  /**
   * Get current history index
   */
  getHistoryIndex(): number {
    return this.historyIndex;
  }
}

/**
 * Create a singleton instance
 */
export const diagramStateManager = new DiagramStateManager();
