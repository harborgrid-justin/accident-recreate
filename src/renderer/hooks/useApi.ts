/**
 * useApi Hook - Convenience hook for API calls with loading and error handling
 */

import { useState, useCallback } from 'react';

interface UseApiOptions<T> {
  onSuccess?: (data: T) => void;
  onError?: (error: string) => void;
}

interface UseApiReturn<T, Args extends any[]> {
  data: T | null;
  error: string | null;
  isLoading: boolean;
  execute: (...args: Args) => Promise<T | null>;
  reset: () => void;
}

export function useApi<T = any, Args extends any[] = any[]>(
  apiFunction: (...args: Args) => Promise<{ success: boolean; data?: T; error?: string }>,
  options: UseApiOptions<T> = {}
): UseApiReturn<T, Args> {
  const [data, setData] = useState<T | null>(null);
  const [error, setError] = useState<string | null>(null);
  const [isLoading, setIsLoading] = useState(false);

  const execute = useCallback(
    async (...args: Args): Promise<T | null> => {
      setIsLoading(true);
      setError(null);

      try {
        const response = await apiFunction(...args);

        if (response.success && response.data) {
          setData(response.data);
          setIsLoading(false);
          options.onSuccess?.(response.data);
          return response.data;
        } else {
          const errorMessage = response.error || 'Request failed';
          setError(errorMessage);
          setIsLoading(false);
          options.onError?.(errorMessage);
          return null;
        }
      } catch (err) {
        const errorMessage = err instanceof Error ? err.message : 'Request failed';
        setError(errorMessage);
        setIsLoading(false);
        options.onError?.(errorMessage);
        return null;
      }
    },
    [apiFunction, options]
  );

  const reset = useCallback(() => {
    setData(null);
    setError(null);
    setIsLoading(false);
  }, []);

  return {
    data,
    error,
    isLoading,
    execute,
    reset,
  };
}

export default useApi;
