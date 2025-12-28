/**
 * useCases Hook - Convenience hook for case management
 */

import { useCasesStore, CasesContextValue } from '../store/casesStore';

export type { CasesContextValue };

export const useCases = (): CasesContextValue => {
  return useCasesStore();
};

export default useCases;
