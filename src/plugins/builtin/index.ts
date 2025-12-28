/**
 * AccuScene Enterprise v0.2.0 - Built-in Plugins
 * Exports for built-in plugins
 */

export * from './measurements';
export * from './annotations';
export * from './exports';

import { PluginContext } from '../types';
import { createMeasurementsPlugin } from './measurements';
import { createAnnotationsPlugin } from './annotations';
import { createExportsPlugin } from './exports';

/**
 * Get all built-in plugins
 */
export const getBuiltinPlugins = () => {
  return [
    createMeasurementsPlugin,
    createAnnotationsPlugin,
    createExportsPlugin,
  ];
};
