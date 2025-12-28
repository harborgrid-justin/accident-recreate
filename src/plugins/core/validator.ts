/**
 * AccuScene Enterprise v0.2.0 - Plugin Validator
 * Validation of plugin manifests, permissions, and dependencies
 */

import {
  PluginManifest,
  PluginPermission,
  PluginDependency,
  ValidationResult,
  ValidationError,
  ValidationWarning,
  PluginCapability,
} from '../types';

const SEMVER_REGEX = /^(\d+)\.(\d+)\.(\d+)(?:-([0-9A-Za-z-]+(?:\.[0-9A-Za-z-]+)*))?(?:\+([0-9A-Za-z-]+(?:\.[0-9A-Za-z-]+)*))?$/;
const ID_REGEX = /^[a-z0-9-]+(\.[a-z0-9-]+)*$/;

export class PluginValidatorImpl {
  private knownPlugins = new Map<string, string>(); // id -> version

  /**
   * Validate a plugin manifest
   */
  validate(manifest: PluginManifest): ValidationResult {
    const errors: ValidationError[] = [];
    const warnings: ValidationWarning[] = [];

    // Validate required fields
    this.validateRequiredFields(manifest, errors);

    // Validate ID format
    this.validateId(manifest.id, errors);

    // Validate version format
    this.validateVersion(manifest.version, errors);

    // Validate main entry point
    this.validateMainEntry(manifest.main, errors);

    // Validate dependencies
    if (manifest.dependencies) {
      this.validateDependencies(manifest.dependencies, errors, warnings);
    }

    // Validate permissions
    if (manifest.permissions) {
      this.validatePermissions(manifest.permissions, errors, warnings);
    }

    // Validate capabilities
    if (manifest.capabilities) {
      this.validateCapabilities(manifest.capabilities, errors, warnings);
    }

    // Validate contributions
    if (manifest.contributes) {
      this.validateContributions(manifest, errors, warnings);
    }

    return {
      valid: errors.length === 0,
      errors,
      warnings,
    };
  }

  /**
   * Validate permissions
   */
  validatePermissions(permissions: PluginPermission[]): ValidationResult {
    const errors: ValidationError[] = [];
    const warnings: ValidationWarning[] = [];

    // Check for duplicate permissions
    const seen = new Set<PluginPermission>();
    for (const permission of permissions) {
      if (seen.has(permission)) {
        warnings.push({
          code: 'DUPLICATE_PERMISSION',
          message: `Duplicate permission: ${permission}`,
          field: 'permissions',
        });
      }
      seen.add(permission);
    }

    // Check for dangerous permission combinations
    if (
      permissions.includes(PluginPermission.WRITE_FILE) &&
      permissions.includes(PluginPermission.EXECUTE_COMMAND)
    ) {
      warnings.push({
        code: 'DANGEROUS_PERMISSIONS',
        message: 'Plugin requests both file write and command execution - this combination may be dangerous',
        field: 'permissions',
      });
    }

    return { valid: errors.length === 0, errors, warnings };
  }

  /**
   * Validate dependencies
   */
  validateDependencies(dependencies: PluginDependency[]): ValidationResult {
    const errors: ValidationError[] = [];
    const warnings: ValidationWarning[] = [];

    for (const dep of dependencies) {
      // Validate dependency ID
      if (!ID_REGEX.test(dep.id)) {
        errors.push({
          code: 'INVALID_DEPENDENCY_ID',
          message: `Invalid dependency ID: ${dep.id}`,
          field: 'dependencies',
        });
      }

      // Validate version constraint
      if (!this.isValidVersionConstraint(dep.version)) {
        errors.push({
          code: 'INVALID_VERSION_CONSTRAINT',
          message: `Invalid version constraint for ${dep.id}: ${dep.version}`,
          field: 'dependencies',
        });
      }

      // Check if dependency is available
      if (this.knownPlugins.has(dep.id)) {
        const availableVersion = this.knownPlugins.get(dep.id)!;
        if (!this.satisfiesVersion(availableVersion, dep.version)) {
          errors.push({
            code: 'DEPENDENCY_VERSION_MISMATCH',
            message: `Dependency ${dep.id} version ${availableVersion} does not satisfy ${dep.version}`,
            field: 'dependencies',
          });
        }
      } else if (!dep.optional) {
        warnings.push({
          code: 'MISSING_DEPENDENCY',
          message: `Required dependency ${dep.id} is not available`,
          field: 'dependencies',
        });
      }
    }

    // Check for circular dependencies
    const circular = this.detectCircularDependencies(dependencies);
    if (circular.length > 0) {
      errors.push({
        code: 'CIRCULAR_DEPENDENCY',
        message: `Circular dependency detected: ${circular.join(' -> ')}`,
        field: 'dependencies',
      });
    }

    return { valid: errors.length === 0, errors, warnings };
  }

  /**
   * Register a known plugin for dependency checking
   */
  registerKnownPlugin(id: string, version: string): void {
    this.knownPlugins.set(id, version);
  }

  /**
   * Unregister a known plugin
   */
  unregisterKnownPlugin(id: string): void {
    this.knownPlugins.delete(id);
  }

  private validateRequiredFields(manifest: PluginManifest, errors: ValidationError[]): void {
    const required = ['id', 'name', 'version', 'description', 'author', 'license', 'main'];

    for (const field of required) {
      if (!manifest[field as keyof PluginManifest]) {
        errors.push({
          code: 'MISSING_REQUIRED_FIELD',
          message: `Missing required field: ${field}`,
          field,
        });
      }
    }
  }

  private validateId(id: string, errors: ValidationError[]): void {
    if (!ID_REGEX.test(id)) {
      errors.push({
        code: 'INVALID_ID',
        message: `Invalid plugin ID format: ${id}. Must be lowercase alphanumeric with dots or hyphens.`,
        field: 'id',
      });
    }
  }

  private validateVersion(version: string, errors: ValidationError[]): void {
    if (!SEMVER_REGEX.test(version)) {
      errors.push({
        code: 'INVALID_VERSION',
        message: `Invalid version format: ${version}. Must follow semantic versioning.`,
        field: 'version',
      });
    }
  }

  private validateMainEntry(main: string, errors: ValidationError[]): void {
    if (!main.endsWith('.js') && !main.endsWith('.ts') && !main.endsWith('.mjs')) {
      errors.push({
        code: 'INVALID_MAIN_ENTRY',
        message: `Main entry must be a JavaScript/TypeScript file: ${main}`,
        field: 'main',
      });
    }
  }

  private validateCapabilities(
    capabilities: PluginCapability[],
    errors: ValidationError[],
    warnings: ValidationWarning[]
  ): void {
    // Check for capabilities that require specific permissions
    if (capabilities.includes(PluginCapability.NATIVE_MODULE)) {
      warnings.push({
        code: 'NATIVE_MODULE_CAPABILITY',
        message: 'Plugin uses native modules - ensure compatibility across platforms',
        field: 'capabilities',
      });
    }

    if (
      capabilities.includes(PluginCapability.WORKER_THREAD) ||
      capabilities.includes(PluginCapability.WEB_WORKER)
    ) {
      warnings.push({
        code: 'WORKER_CAPABILITY',
        message: 'Plugin uses worker threads - ensure proper resource management',
        field: 'capabilities',
      });
    }
  }

  private validateContributions(
    manifest: PluginManifest,
    errors: ValidationError[],
    warnings: ValidationWarning[]
  ): void {
    const { contributes } = manifest;

    if (!contributes) {
      return;
    }

    // Validate that plugin has necessary permissions for its contributions
    const permissions = manifest.permissions || [];

    if (contributes.commands && !permissions.includes(PluginPermission.REGISTER_COMMAND)) {
      errors.push({
        code: 'MISSING_PERMISSION_FOR_CONTRIBUTION',
        message: 'Plugin contributes commands but lacks REGISTER_COMMAND permission',
        field: 'permissions',
      });
    }

    if (
      (contributes.menus || contributes.toolbars || contributes.panels) &&
      !permissions.includes(PluginPermission.UI_MODIFY)
    ) {
      errors.push({
        code: 'MISSING_PERMISSION_FOR_CONTRIBUTION',
        message: 'Plugin contributes UI elements but lacks UI_MODIFY permission',
        field: 'permissions',
      });
    }
  }

  private isValidVersionConstraint(constraint: string): boolean {
    // Support various version constraint formats
    const patterns = [
      /^\d+\.\d+\.\d+$/, // Exact version: 1.2.3
      /^\^\d+\.\d+\.\d+$/, // Caret range: ^1.2.3
      /^~\d+\.\d+\.\d+$/, // Tilde range: ~1.2.3
      /^>=?\d+\.\d+\.\d+$/, // Greater than: >=1.2.3
      /^<=?\d+\.\d+\.\d+$/, // Less than: <=1.2.3
      /^\*$/, // Any version: *
    ];

    return patterns.some(pattern => pattern.test(constraint));
  }

  private satisfiesVersion(version: string, constraint: string): boolean {
    // Simplified version satisfaction check
    // In production, use a library like semver

    if (constraint === '*') {
      return true;
    }

    if (constraint.startsWith('^')) {
      const constraintVersion = constraint.slice(1);
      return version >= constraintVersion;
    }

    if (constraint.startsWith('~')) {
      const constraintVersion = constraint.slice(1);
      return version >= constraintVersion;
    }

    if (constraint.startsWith('>=')) {
      const constraintVersion = constraint.slice(2);
      return version >= constraintVersion;
    }

    if (constraint.startsWith('<=')) {
      const constraintVersion = constraint.slice(2);
      return version <= constraintVersion;
    }

    return version === constraint;
  }

  private detectCircularDependencies(dependencies: PluginDependency[]): string[] {
    // Simplified circular dependency detection
    // In production, use a proper graph traversal algorithm

    const visited = new Set<string>();
    const stack = new Set<string>();

    const visit = (id: string): string[] => {
      if (stack.has(id)) {
        return [id];
      }

      if (visited.has(id)) {
        return [];
      }

      visited.add(id);
      stack.add(id);

      const dep = dependencies.find(d => d.id === id);
      if (dep) {
        // In a real implementation, we'd traverse the dependency tree
      }

      stack.delete(id);
      return [];
    };

    for (const dep of dependencies) {
      const cycle = visit(dep.id);
      if (cycle.length > 0) {
        return cycle;
      }
    }

    return [];
  }
}

export const createPluginValidator = (): PluginValidatorImpl => {
  return new PluginValidatorImpl();
};
