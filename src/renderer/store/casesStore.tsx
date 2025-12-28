/**
 * Cases Store - Manages case data and operations
 */

import React, { createContext, useContext, useState, useCallback, ReactNode } from 'react';
import { api, CaseData } from '../services/api';

interface CasesState {
  cases: CaseData[];
  currentCase: CaseData | null;
  isLoading: boolean;
  error: string | null;
  total: number;
  filters: {
    status?: string;
    priority?: string;
    search?: string;
  };
}

export interface CasesContextValue extends CasesState {
  fetchCases: (params?: any) => Promise<void>;
  fetchCase: (id: string) => Promise<void>;
  createCase: (caseData: CaseData) => Promise<CaseData | null>;
  updateCase: (id: string, caseData: Partial<CaseData>) => Promise<boolean>;
  deleteCase: (id: string) => Promise<boolean>;
  updateCaseStatus: (id: string, status: string) => Promise<boolean>;
  setFilters: (filters: CasesState['filters']) => void;
  clearError: () => void;
  clearCurrentCase: () => void;
}

const CasesContext = createContext<CasesContextValue | undefined>(undefined);

export const CasesProvider: React.FC<{ children: ReactNode }> = ({ children }) => {
  const [state, setState] = useState<CasesState>({
    cases: [],
    currentCase: null,
    isLoading: false,
    error: null,
    total: 0,
    filters: {},
  });

  const fetchCases = useCallback(async (params?: any) => {
    setState((prev) => ({ ...prev, isLoading: true, error: null }));

    try {
      const response = await api.getCases(params || state.filters);

      if (response.success && response.data) {
        setState((prev) => ({
          ...prev,
          cases: response.data!.cases,
          total: response.data!.total,
          isLoading: false,
        }));
      } else {
        setState((prev) => ({
          ...prev,
          isLoading: false,
          error: response.error || 'Failed to fetch cases',
        }));
      }
    } catch (error) {
      setState((prev) => ({
        ...prev,
        isLoading: false,
        error: error instanceof Error ? error.message : 'Failed to fetch cases',
      }));
    }
  }, [state.filters]);

  const fetchCase = useCallback(async (id: string) => {
    setState((prev) => ({ ...prev, isLoading: true, error: null }));

    try {
      const response = await api.getCase(id);

      if (response.success && response.data) {
        setState((prev) => ({
          ...prev,
          currentCase: response.data!,
          isLoading: false,
        }));
      } else {
        setState((prev) => ({
          ...prev,
          isLoading: false,
          error: response.error || 'Failed to fetch case',
        }));
      }
    } catch (error) {
      setState((prev) => ({
        ...prev,
        isLoading: false,
        error: error instanceof Error ? error.message : 'Failed to fetch case',
      }));
    }
  }, []);

  const createCase = useCallback(async (caseData: CaseData): Promise<CaseData | null> => {
    setState((prev) => ({ ...prev, isLoading: true, error: null }));

    try {
      const response = await api.createCase(caseData);

      if (response.success && response.data) {
        setState((prev) => ({
          ...prev,
          cases: [response.data!, ...prev.cases],
          total: prev.total + 1,
          isLoading: false,
        }));
        return response.data;
      } else {
        setState((prev) => ({
          ...prev,
          isLoading: false,
          error: response.error || 'Failed to create case',
        }));
        return null;
      }
    } catch (error) {
      setState((prev) => ({
        ...prev,
        isLoading: false,
        error: error instanceof Error ? error.message : 'Failed to create case',
      }));
      return null;
    }
  }, []);

  const updateCase = useCallback(
    async (id: string, caseData: Partial<CaseData>): Promise<boolean> => {
      setState((prev) => ({ ...prev, isLoading: true, error: null }));

      try {
        const response = await api.updateCase(id, caseData);

        if (response.success && response.data) {
          setState((prev) => ({
            ...prev,
            cases: prev.cases.map((c) => (c.id === id ? response.data! : c)),
            currentCase: prev.currentCase?.id === id ? response.data! : prev.currentCase,
            isLoading: false,
          }));
          return true;
        } else {
          setState((prev) => ({
            ...prev,
            isLoading: false,
            error: response.error || 'Failed to update case',
          }));
          return false;
        }
      } catch (error) {
        setState((prev) => ({
          ...prev,
          isLoading: false,
          error: error instanceof Error ? error.message : 'Failed to update case',
        }));
        return false;
      }
    },
    []
  );

  const deleteCase = useCallback(async (id: string): Promise<boolean> => {
    setState((prev) => ({ ...prev, isLoading: true, error: null }));

    try {
      const response = await api.deleteCase(id);

      if (response.success) {
        setState((prev) => ({
          ...prev,
          cases: prev.cases.filter((c) => c.id !== id),
          total: prev.total - 1,
          currentCase: prev.currentCase?.id === id ? null : prev.currentCase,
          isLoading: false,
        }));
        return true;
      } else {
        setState((prev) => ({
          ...prev,
          isLoading: false,
          error: response.error || 'Failed to delete case',
        }));
        return false;
      }
    } catch (error) {
      setState((prev) => ({
        ...prev,
        isLoading: false,
        error: error instanceof Error ? error.message : 'Failed to delete case',
      }));
      return false;
    }
  }, []);

  const updateCaseStatus = useCallback(async (id: string, status: string): Promise<boolean> => {
    setState((prev) => ({ ...prev, isLoading: true, error: null }));

    try {
      const response = await api.updateCaseStatus(id, status);

      if (response.success && response.data) {
        setState((prev) => ({
          ...prev,
          cases: prev.cases.map((c) => (c.id === id ? { ...c, status } : c)),
          currentCase:
            prev.currentCase?.id === id ? { ...prev.currentCase, status } : prev.currentCase,
          isLoading: false,
        }));
        return true;
      } else {
        setState((prev) => ({
          ...prev,
          isLoading: false,
          error: response.error || 'Failed to update case status',
        }));
        return false;
      }
    } catch (error) {
      setState((prev) => ({
        ...prev,
        isLoading: false,
        error: error instanceof Error ? error.message : 'Failed to update case status',
      }));
      return false;
    }
  }, []);

  const setFilters = useCallback((filters: CasesState['filters']) => {
    setState((prev) => ({ ...prev, filters }));
  }, []);

  const clearError = useCallback(() => {
    setState((prev) => ({ ...prev, error: null }));
  }, []);

  const clearCurrentCase = useCallback(() => {
    setState((prev) => ({ ...prev, currentCase: null }));
  }, []);

  const value: CasesContextValue = {
    ...state,
    fetchCases,
    fetchCase,
    createCase,
    updateCase,
    deleteCase,
    updateCaseStatus,
    setFilters,
    clearError,
    clearCurrentCase,
  };

  return <CasesContext.Provider value={value}>{children}</CasesContext.Provider>;
};

export const useCasesStore = (): CasesContextValue => {
  const context = useContext(CasesContext);
  if (context === undefined) {
    throw new Error('useCasesStore must be used within a CasesProvider');
  }
  return context;
};
