/**
 * Editor Store - Manages diagram editor state
 */

import React, { createContext, useContext, useState, useCallback, ReactNode } from 'react';

export interface DiagramElement {
  id: string;
  type: 'vehicle' | 'road' | 'marker' | 'text' | 'arrow' | 'measurement';
  x: number;
  y: number;
  width?: number;
  height?: number;
  rotation?: number;
  data?: any;
  color?: string;
  label?: string;
}

interface EditorState {
  elements: DiagramElement[];
  selectedElements: string[];
  tool: string;
  zoom: number;
  pan: { x: number; y: number };
  gridEnabled: boolean;
  snapToGrid: boolean;
  history: DiagramElement[][];
  historyIndex: number;
  isDirty: boolean;
}

interface EditorContextValue extends EditorState {
  addElement: (element: DiagramElement) => void;
  updateElement: (id: string, updates: Partial<DiagramElement>) => void;
  deleteElement: (id: string) => void;
  selectElement: (id: string, multi?: boolean) => void;
  clearSelection: () => void;
  setTool: (tool: string) => void;
  setZoom: (zoom: number) => void;
  setPan: (pan: { x: number; y: number }) => void;
  toggleGrid: () => void;
  toggleSnapToGrid: () => void;
  undo: () => void;
  redo: () => void;
  clearDiagram: () => void;
  loadDiagram: (elements: DiagramElement[]) => void;
  saveDiagram: () => DiagramElement[];
  markClean: () => void;
}

const EditorContext = createContext<EditorContextValue | undefined>(undefined);

export const EditorProvider: React.FC<{ children: ReactNode }> = ({ children }) => {
  const [state, setState] = useState<EditorState>({
    elements: [],
    selectedElements: [],
    tool: 'select',
    zoom: 1,
    pan: { x: 0, y: 0 },
    gridEnabled: true,
    snapToGrid: true,
    history: [[]],
    historyIndex: 0,
    isDirty: false,
  });

  const addToHistory = useCallback((elements: DiagramElement[]) => {
    setState((prev) => {
      const newHistory = prev.history.slice(0, prev.historyIndex + 1);
      newHistory.push([...elements]);
      return {
        ...prev,
        history: newHistory,
        historyIndex: newHistory.length - 1,
        isDirty: true,
      };
    });
  }, []);

  const addElement = useCallback(
    (element: DiagramElement) => {
      setState((prev) => {
        const newElements = [...prev.elements, element];
        addToHistory(newElements);
        return {
          ...prev,
          elements: newElements,
          isDirty: true,
        };
      });
    },
    [addToHistory]
  );

  const updateElement = useCallback(
    (id: string, updates: Partial<DiagramElement>) => {
      setState((prev) => {
        const newElements = prev.elements.map((el) =>
          el.id === id ? { ...el, ...updates } : el
        );
        addToHistory(newElements);
        return {
          ...prev,
          elements: newElements,
          isDirty: true,
        };
      });
    },
    [addToHistory]
  );

  const deleteElement = useCallback(
    (id: string) => {
      setState((prev) => {
        const newElements = prev.elements.filter((el) => el.id !== id);
        addToHistory(newElements);
        return {
          ...prev,
          elements: newElements,
          selectedElements: prev.selectedElements.filter((selId) => selId !== id),
          isDirty: true,
        };
      });
    },
    [addToHistory]
  );

  const selectElement = useCallback((id: string, multi: boolean = false) => {
    setState((prev) => ({
      ...prev,
      selectedElements: multi
        ? prev.selectedElements.includes(id)
          ? prev.selectedElements.filter((selId) => selId !== id)
          : [...prev.selectedElements, id]
        : [id],
    }));
  }, []);

  const clearSelection = useCallback(() => {
    setState((prev) => ({ ...prev, selectedElements: [] }));
  }, []);

  const setTool = useCallback((tool: string) => {
    setState((prev) => ({ ...prev, tool }));
  }, []);

  const setZoom = useCallback((zoom: number) => {
    setState((prev) => ({ ...prev, zoom: Math.max(0.1, Math.min(5, zoom)) }));
  }, []);

  const setPan = useCallback((pan: { x: number; y: number }) => {
    setState((prev) => ({ ...prev, pan }));
  }, []);

  const toggleGrid = useCallback(() => {
    setState((prev) => ({ ...prev, gridEnabled: !prev.gridEnabled }));
  }, []);

  const toggleSnapToGrid = useCallback(() => {
    setState((prev) => ({ ...prev, snapToGrid: !prev.snapToGrid }));
  }, []);

  const undo = useCallback(() => {
    setState((prev) => {
      if (prev.historyIndex > 0) {
        const newIndex = prev.historyIndex - 1;
        return {
          ...prev,
          elements: [...prev.history[newIndex]!],
          historyIndex: newIndex,
          isDirty: true,
        };
      }
      return prev;
    });
  }, []);

  const redo = useCallback(() => {
    setState((prev) => {
      if (prev.historyIndex < prev.history.length - 1) {
        const newIndex = prev.historyIndex + 1;
        return {
          ...prev,
          elements: [...prev.history[newIndex]!],
          historyIndex: newIndex,
          isDirty: true,
        };
      }
      return prev;
    });
  }, []);

  const clearDiagram = useCallback(() => {
    setState((prev) => {
      addToHistory([]);
      return {
        ...prev,
        elements: [],
        selectedElements: [],
        isDirty: true,
      };
    });
  }, [addToHistory]);

  const loadDiagram = useCallback(
    (elements: DiagramElement[]) => {
      setState((prev) => {
        addToHistory(elements);
        return {
          ...prev,
          elements: [...elements],
          selectedElements: [],
          isDirty: false,
        };
      });
    },
    [addToHistory]
  );

  const saveDiagram = useCallback((): DiagramElement[] => {
    return [...state.elements];
  }, [state.elements]);

  const markClean = useCallback(() => {
    setState((prev) => ({ ...prev, isDirty: false }));
  }, []);

  const value: EditorContextValue = {
    ...state,
    addElement,
    updateElement,
    deleteElement,
    selectElement,
    clearSelection,
    setTool,
    setZoom,
    setPan,
    toggleGrid,
    toggleSnapToGrid,
    undo,
    redo,
    clearDiagram,
    loadDiagram,
    saveDiagram,
    markClean,
  };

  return <EditorContext.Provider value={value}>{children}</EditorContext.Provider>;
};

export const useEditorStore = (): EditorContextValue => {
  const context = useContext(EditorContext);
  if (context === undefined) {
    throw new Error('useEditorStore must be used within an EditorProvider');
  }
  return context;
};
