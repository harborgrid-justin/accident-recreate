/**
 * AccuScene Enterprise v0.3.0 - CAD Tool State Management Hook
 */

import { useState, useCallback, useEffect } from 'react';
import { CADToolType, CADTool, ToolCategory } from '../types';

export interface UseCADToolReturn {
  activeTool: CADToolType;
  setActiveTool: (tool: CADToolType) => void;
  tools: CADTool[];
  getToolById: (id: string) => CADTool | undefined;
  getToolsByCategory: (category: ToolCategory) => CADTool[];
  isToolActive: (tool: CADToolType) => boolean;
  previousTool: CADToolType | null;
  restorePreviousTool: () => void;
}

const defaultTools: CADTool[] = [
  {
    id: 'select',
    type: CADToolType.SELECT,
    name: 'Select',
    icon: '⬚',
    shortcut: 'V',
    description: 'Select and manipulate objects',
    category: ToolCategory.SELECTION,
    enabled: true,
  },
  {
    id: 'pan',
    type: CADToolType.PAN,
    name: 'Pan',
    icon: '✋',
    shortcut: 'H',
    description: 'Pan the viewport',
    category: ToolCategory.NAVIGATION,
    enabled: true,
  },
  {
    id: 'zoom',
    type: CADToolType.ZOOM,
    name: 'Zoom',
    icon: '🔍',
    shortcut: 'Z',
    description: 'Zoom in and out',
    category: ToolCategory.NAVIGATION,
    enabled: true,
  },
  {
    id: 'line',
    type: CADToolType.LINE,
    name: 'Line',
    icon: '📏',
    shortcut: 'L',
    description: 'Draw a line segment',
    category: ToolCategory.DRAW,
    enabled: true,
  },
  {
    id: 'rectangle',
    type: CADToolType.RECTANGLE,
    name: 'Rectangle',
    icon: '▭',
    shortcut: 'R',
    description: 'Draw a rectangle',
    category: ToolCategory.DRAW,
    enabled: true,
  },
  {
    id: 'circle',
    type: CADToolType.CIRCLE,
    name: 'Circle',
    icon: '○',
    shortcut: 'C',
    description: 'Draw a circle',
    category: ToolCategory.DRAW,
    enabled: true,
  },
  {
    id: 'arc',
    type: CADToolType.ARC,
    name: 'Arc',
    icon: '◠',
    shortcut: 'A',
    description: 'Draw an arc',
    category: ToolCategory.DRAW,
    enabled: true,
  },
  {
    id: 'polyline',
    type: CADToolType.POLYLINE,
    name: 'Polyline',
    icon: '📐',
    shortcut: 'P',
    description: 'Draw connected line segments',
    category: ToolCategory.DRAW,
    enabled: true,
  },
  {
    id: 'polygon',
    type: CADToolType.POLYGON,
    name: 'Polygon',
    icon: '⬡',
    shortcut: 'Shift+P',
    description: 'Draw a polygon',
    category: ToolCategory.DRAW,
    enabled: true,
  },
  {
    id: 'text',
    type: CADToolType.TEXT,
    name: 'Text',
    icon: 'T',
    shortcut: 'T',
    description: 'Add text annotation',
    category: ToolCategory.ANNOTATION,
    enabled: true,
  },
  {
    id: 'measure-distance',
    type: CADToolType.MEASURE_DISTANCE,
    name: 'Measure Distance',
    icon: '↔',
    shortcut: 'M',
    description: 'Measure distance between points',
    category: ToolCategory.MEASURE,
    enabled: true,
  },
  {
    id: 'measure-angle',
    type: CADToolType.MEASURE_ANGLE,
    name: 'Measure Angle',
    icon: '∠',
    shortcut: 'Shift+M',
    description: 'Measure angle between lines',
    category: ToolCategory.MEASURE,
    enabled: true,
  },
  {
    id: 'move',
    type: CADToolType.MOVE,
    name: 'Move',
    icon: '↗',
    shortcut: 'M',
    description: 'Move selected objects',
    category: ToolCategory.MODIFY,
    enabled: true,
  },
  {
    id: 'rotate',
    type: CADToolType.ROTATE,
    name: 'Rotate',
    icon: '↻',
    shortcut: 'Ctrl+R',
    description: 'Rotate selected objects',
    category: ToolCategory.MODIFY,
    enabled: true,
  },
  {
    id: 'scale',
    type: CADToolType.SCALE,
    name: 'Scale',
    icon: '⤢',
    shortcut: 'S',
    description: 'Scale selected objects',
    category: ToolCategory.MODIFY,
    enabled: true,
  },
  {
    id: 'mirror',
    type: CADToolType.MIRROR,
    name: 'Mirror',
    icon: '⇄',
    shortcut: 'Ctrl+M',
    description: 'Mirror selected objects',
    category: ToolCategory.MODIFY,
    enabled: true,
  },
  {
    id: 'offset',
    type: CADToolType.OFFSET,
    name: 'Offset',
    icon: '⊶',
    shortcut: 'O',
    description: 'Create offset copy',
    category: ToolCategory.MODIFY,
    enabled: true,
  },
];

export function useCADTool(initialTool: CADToolType = CADToolType.SELECT): UseCADToolReturn {
  const [activeTool, setActiveToolState] = useState<CADToolType>(initialTool);
  const [previousTool, setPreviousTool] = useState<CADToolType | null>(null);
  const [tools] = useState<CADTool[]>(defaultTools);

  const setActiveTool = useCallback((tool: CADToolType) => {
    setActiveToolState((prev) => {
      if (prev !== tool) {
        setPreviousTool(prev);
      }
      return tool;
    });
  }, []);

  const getToolById = useCallback(
    (id: string): CADTool | undefined => {
      return tools.find((tool) => tool.id === id);
    },
    [tools]
  );

  const getToolsByCategory = useCallback(
    (category: ToolCategory): CADTool[] => {
      return tools.filter((tool) => tool.category === category);
    },
    [tools]
  );

  const isToolActive = useCallback(
    (tool: CADToolType): boolean => {
      return activeTool === tool;
    },
    [activeTool]
  );

  const restorePreviousTool = useCallback(() => {
    if (previousTool) {
      setActiveTool(previousTool);
    }
  }, [previousTool, setActiveTool]);

  // Keyboard shortcuts
  useEffect(() => {
    const handleKeyDown = (e: KeyboardEvent) => {
      // Don't trigger shortcuts if user is typing in an input
      if (
        e.target instanceof HTMLInputElement ||
        e.target instanceof HTMLTextAreaElement
      ) {
        return;
      }

      const key = e.key.toLowerCase();
      const ctrl = e.ctrlKey || e.metaKey;
      const shift = e.shiftKey;

      // Find matching tool by shortcut
      const matchingTool = tools.find((tool) => {
        if (!tool.shortcut) return false;

        const shortcut = tool.shortcut.toLowerCase();
        const hasCtrl = shortcut.includes('ctrl+');
        const hasShift = shortcut.includes('shift+');
        const baseKey = shortcut.replace('ctrl+', '').replace('shift+', '');

        return (
          baseKey === key &&
          (!hasCtrl || ctrl) &&
          (!hasShift || shift) &&
          (hasCtrl === ctrl) &&
          (hasShift === shift)
        );
      });

      if (matchingTool && matchingTool.enabled) {
        e.preventDefault();
        setActiveTool(matchingTool.type);
      }

      // ESC to return to select tool
      if (key === 'escape') {
        e.preventDefault();
        setActiveTool(CADToolType.SELECT);
      }
    };

    window.addEventListener('keydown', handleKeyDown);
    return () => window.removeEventListener('keydown', handleKeyDown);
  }, [tools, setActiveTool]);

  return {
    activeTool,
    setActiveTool,
    tools,
    getToolById,
    getToolsByCategory,
    isToolActive,
    previousTool,
    restorePreviousTool,
  };
}
