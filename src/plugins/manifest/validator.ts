/**
 * AccuScene Enterprise v0.2.0 - Manifest Validator
 * Extended validation for plugin manifests
 */

import { PluginManifest, ValidationResult, ValidationError, ValidationWarning } from '../types';
import { validateManifestSchema } from './schema';

export class ManifestValidator {
  /**
   * Validate a plugin manifest comprehensively
   */
  validate(manifest: PluginManifest): ValidationResult {
    const errors: ValidationError[] = [];
    const warnings: ValidationWarning[] = [];

    // Schema validation
    const schemaResult = validateManifestSchema(manifest);
    if (!schemaResult.valid) {
      schemaResult.errors.forEach(error => {
        errors.push({
          code: 'SCHEMA_VALIDATION_ERROR',
          message: error,
        });
      });
    }

    // Semantic validation
    this.validateId(manifest.id, errors, warnings);
    this.validateVersion(manifest.version, errors, warnings);
    this.validateLicense(manifest.license, errors, warnings);
    this.validateUrls(manifest, errors, warnings);
    this.validateDependencies(manifest, errors, warnings);
    this.validatePermissions(manifest, errors, warnings);
    this.validateContributions(manifest, errors, warnings);

    return {
      valid: errors.length === 0,
      errors,
      warnings,
    };
  }

  private validateId(id: string, errors: ValidationError[], warnings: ValidationWarning[]): void {
    // Check for reserved prefixes
    const reservedPrefixes = ['accuscene', 'system', 'core', 'builtin'];
    const prefix = id.split('.')[0];

    if (reservedPrefixes.includes(prefix)) {
      warnings.push({
        code: 'RESERVED_ID_PREFIX',
        message: `Plugin ID uses reserved prefix: ${prefix}`,
        field: 'id',
      });
    }

    // Check length
    if (id.length > 100) {
      errors.push({
        code: 'ID_TOO_LONG',
        message: 'Plugin ID must not exceed 100 characters',
        field: 'id',
      });
    }
  }

  private validateVersion(
    version: string,
    errors: ValidationError[],
    warnings: ValidationWarning[]
  ): void {
    const parts = version.split('.');

    if (parts.length < 3) {
      errors.push({
        code: 'INVALID_VERSION',
        message: 'Version must have at least major.minor.patch',
        field: 'version',
      });
      return;
    }

    // Check for pre-release versions
    if (version.includes('-')) {
      warnings.push({
        code: 'PRERELEASE_VERSION',
        message: 'Plugin uses a pre-release version',
        field: 'version',
      });
    }
  }

  private validateLicense(
    license: string,
    errors: ValidationError[],
    warnings: ValidationWarning[]
  ): void {
    // Common open source licenses
    const commonLicenses = [
      'MIT',
      'Apache-2.0',
      'GPL-3.0',
      'BSD-3-Clause',
      'ISC',
      'LGPL-3.0',
      'MPL-2.0',
    ];

    if (!commonLicenses.includes(license) && license !== 'UNLICENSED') {
      warnings.push({
        code: 'UNCOMMON_LICENSE',
        message: `License '${license}' is not a common SPDX identifier`,
        field: 'license',
      });
    }
  }

  private validateUrls(
    manifest: PluginManifest,
    errors: ValidationError[],
    warnings: ValidationWarning[]
  ): void {
    const urlFields = ['homepage', 'repository'] as const;

    for (const field of urlFields) {
      const url = manifest[field];
      if (url) {
        try {
          new URL(url);
        } catch {
          errors.push({
            code: 'INVALID_URL',
            message: `Invalid URL in ${field}`,
            field,
          });
        }
      }
    }
  }

  private validateDependencies(
    manifest: PluginManifest,
    errors: ValidationError[],
    warnings: ValidationWarning[]
  ): void {
    const deps = manifest.dependencies || [];
    const peerDeps = manifest.peerDependencies || [];

    // Check for duplicates between dependencies and peerDependencies
    const depIds = new Set(deps.map(d => d.id));
    for (const peerDep of peerDeps) {
      if (depIds.has(peerDep.id)) {
        warnings.push({
          code: 'DUPLICATE_DEPENDENCY',
          message: `Dependency ${peerDep.id} is listed in both dependencies and peerDependencies`,
          field: 'dependencies',
        });
      }
    }

    // Check for self-dependency
    if (deps.some(d => d.id === manifest.id) || peerDeps.some(d => d.id === manifest.id)) {
      errors.push({
        code: 'SELF_DEPENDENCY',
        message: 'Plugin cannot depend on itself',
        field: 'dependencies',
      });
    }
  }

  private validatePermissions(
    manifest: PluginManifest,
    errors: ValidationError[],
    warnings: ValidationWarning[]
  ): void {
    const permissions = manifest.permissions || [];

    // Check for overly permissive combinations
    const hasFileWrite = permissions.includes('file:write' as any);
    const hasCommandExecute = permissions.includes('command:execute' as any);
    const hasNetwork = permissions.includes('network' as any);

    if (hasFileWrite && hasCommandExecute) {
      warnings.push({
        code: 'DANGEROUS_PERMISSION_COMBO',
        message: 'Plugin has both file:write and command:execute permissions - potential security risk',
        field: 'permissions',
      });
    }

    if (hasFileWrite && hasNetwork) {
      warnings.push({
        code: 'DATA_EXFILTRATION_RISK',
        message: 'Plugin has both file:write and network permissions - potential data exfiltration risk',
        field: 'permissions',
      });
    }
  }

  private validateContributions(
    manifest: PluginManifest,
    errors: ValidationError[],
    warnings: ValidationWarning[]
  ): void {
    const contributes = manifest.contributes;

    if (!contributes) {
      warnings.push({
        code: 'NO_CONTRIBUTIONS',
        message: 'Plugin does not contribute any functionality',
        field: 'contributes',
      });
      return;
    }

    // Check if plugin has necessary permissions for its contributions
    const permissions = manifest.permissions || [];

    if (contributes.commands && !permissions.includes('command:register' as any)) {
      errors.push({
        code: 'MISSING_PERMISSION',
        message: 'Plugin contributes commands but lacks command:register permission',
        field: 'permissions',
      });
    }

    if (
      (contributes.menus || contributes.toolbars || contributes.panels) &&
      !permissions.includes('ui:modify' as any)
    ) {
      errors.push({
        code: 'MISSING_PERMISSION',
        message: 'Plugin contributes UI elements but lacks ui:modify permission',
        field: 'permissions',
      });
    }
  }
}

export const createManifestValidator = (): ManifestValidator => {
  return new ManifestValidator();
};
