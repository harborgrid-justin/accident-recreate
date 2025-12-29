/**
 * AccuScene Enterprise v0.3.0 - Layer Management Hook
 */

import { useState, useCallback } from 'react';
import { Layer } from '../types';

export interface UseLayerManagementReturn {
  layers: Layer[];
  activeLayer: string | null;
  addLayer: (name: string, color?: string) => Layer;
  removeLayer: (id: string) => void;
  updateLayer: (id: string, updates: Partial<Layer>) => void;
  setActiveLayer: (id: string) => void;
  toggleLayerVisibility: (id: string) => void;
  toggleLayerLock: (id: string) => void;
  reorderLayers: (sourceId: string, targetId: string) => void;
  duplicateLayer: (id: string) => Layer;
  getLayerById: (id: string) => Layer | undefined;
  getVisibleLayers: () => Layer[];
}

const defaultLayers: Layer[] = [
  {
    id: 'layer-default',
    name: 'Layer 0',
    visible: true,
    locked: false,
    color: '#2563eb',
    opacity: 1,
    order: 0,
    objectCount: 0,
    selected: true,
  },
];

export function useLayerManagement(
  initialLayers: Layer[] = defaultLayers
): UseLayerManagementReturn {
  const [layers, setLayers] = useState<Layer[]>(initialLayers);
  const [activeLayer, setActiveLayerState] = useState<string | null>(
    initialLayers.length > 0 ? initialLayers[0].id : null
  );

  const addLayer = useCallback(
    (name: string, color: string = '#2563eb'): Layer => {
      const newLayer: Layer = {
        id: `layer-${Date.now()}-${Math.random().toString(36).substr(2, 9)}`,
        name,
        visible: true,
        locked: false,
        color,
        opacity: 1,
        order: layers.length,
        objectCount: 0,
        selected: false,
      };

      setLayers((prev) => [...prev, newLayer]);
      return newLayer;
    },
    [layers.length]
  );

  const removeLayer = useCallback((id: string) => {
    setLayers((prev) => {
      const filtered = prev.filter((layer) => layer.id !== id);
      // Reorder remaining layers
      return filtered.map((layer, index) => ({ ...layer, order: index }));
    });

    // If removed layer was active, set first layer as active
    setActiveLayerState((prev) => {
      if (prev === id) {
        return layers.length > 1 ? layers[0].id : null;
      }
      return prev;
    });
  }, [layers]);

  const updateLayer = useCallback((id: string, updates: Partial<Layer>) => {
    setLayers((prev) =>
      prev.map((layer) => (layer.id === id ? { ...layer, ...updates } : layer))
    );
  }, []);

  const setActiveLayer = useCallback((id: string) => {
    setActiveLayerState(id);
    setLayers((prev) =>
      prev.map((layer) => ({ ...layer, selected: layer.id === id }))
    );
  }, []);

  const toggleLayerVisibility = useCallback((id: string) => {
    setLayers((prev) =>
      prev.map((layer) =>
        layer.id === id ? { ...layer, visible: !layer.visible } : layer
      )
    );
  }, []);

  const toggleLayerLock = useCallback((id: string) => {
    setLayers((prev) =>
      prev.map((layer) =>
        layer.id === id ? { ...layer, locked: !layer.locked } : layer
      )
    );
  }, []);

  const reorderLayers = useCallback((sourceId: string, targetId: string) => {
    setLayers((prev) => {
      const sourceIndex = prev.findIndex((l) => l.id === sourceId);
      const targetIndex = prev.findIndex((l) => l.id === targetId);

      if (sourceIndex === -1 || targetIndex === -1) return prev;

      const newLayers = [...prev];
      const [removed] = newLayers.splice(sourceIndex, 1);
      newLayers.splice(targetIndex, 0, removed);

      // Update order property
      return newLayers.map((layer, index) => ({ ...layer, order: index }));
    });
  }, []);

  const duplicateLayer = useCallback(
    (id: string): Layer => {
      const layerToDuplicate = layers.find((l) => l.id === id);
      if (!layerToDuplicate) {
        throw new Error(`Layer ${id} not found`);
      }

      const newLayer: Layer = {
        ...layerToDuplicate,
        id: `layer-${Date.now()}-${Math.random().toString(36).substr(2, 9)}`,
        name: `${layerToDuplicate.name} Copy`,
        order: layers.length,
        selected: false,
        objectCount: 0,
      };

      setLayers((prev) => [...prev, newLayer]);
      return newLayer;
    },
    [layers]
  );

  const getLayerById = useCallback(
    (id: string): Layer | undefined => {
      return layers.find((layer) => layer.id === id);
    },
    [layers]
  );

  const getVisibleLayers = useCallback((): Layer[] => {
    return layers.filter((layer) => layer.visible);
  }, [layers]);

  return {
    layers,
    activeLayer,
    addLayer,
    removeLayer,
    updateLayer,
    setActiveLayer,
    toggleLayerVisibility,
    toggleLayerLock,
    reorderLayers,
    duplicateLayer,
    getLayerById,
    getVisibleLayers,
  };
}
