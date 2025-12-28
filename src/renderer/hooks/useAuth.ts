/**
 * useAuth Hook - Convenience hook for authentication
 */

import { useAuthStore, AuthContextValue } from '../store/authStore';

export type { AuthContextValue };

export const useAuth = (): AuthContextValue => {
  return useAuthStore();
};

export default useAuth;
