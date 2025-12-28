/**
 * API Service - Centralized API client for AccuScene Enterprise
 * Handles all HTTP requests to the backend REST API
 */

export interface ApiResponse<T = any> {
  success: boolean;
  data?: T;
  error?: string;
  message?: string;
}

export interface LoginCredentials {
  email: string;
  password: string;
}

export interface AuthResponse {
  token: string;
  user: {
    id: string;
    email: string;
    firstName: string;
    lastName: string;
    role: string;
    isActive: boolean;
  };
}

export interface CaseData {
  id?: string;
  caseNumber?: string;
  title: string;
  description?: string;
  status?: string;
  priority?: string;
  userId?: string;
  clientName?: string;
  clientPhone?: string;
  clientEmail?: string;
  assignedTo?: string;
  dueDate?: string;
  notes?: string;
  tags?: string[];
  createdAt?: string;
  updatedAt?: string;
}

export interface VehicleData {
  id?: string;
  accidentId: string;
  vehicleNumber: number;
  type: string;
  make: string;
  model: string;
  year?: number;
  color?: string;
  licensePlate?: string;
  licensePlateState?: string;
  vin?: string;
  driverName: string;
  driverPhone?: string;
  driverLicense?: string;
  speed?: number;
  damageSeverity?: string;
  damage?: string;
}

export interface AccidentData {
  id?: string;
  caseId: string;
  dateTime: string;
  location: string;
  latitude?: number;
  longitude?: number;
  weather?: string;
  roadConditions?: string;
  lightConditions?: string;
  description?: string;
  policeReportNumber?: string;
  injuries?: number;
  fatalities?: number;
  vehicles?: VehicleData[];
  diagram?: {
    elements: any[];
    width?: number;
    height?: number;
    scale?: number;
  };
}

class ApiService {
  private baseURL: string;
  private token: string | null = null;

  constructor() {
    // Get API URL from Electron or use default
    this.baseURL = this.getApiUrl();
    // Try to load token from localStorage
    this.token = this.getStoredToken();
  }

  private getApiUrl(): string {
    if (typeof window !== 'undefined' && (window as any).electronAPI) {
      // In Electron environment
      return 'http://localhost:3001/api';
    }
    return process.env.API_URL || 'http://localhost:3001/api';
  }

  private getStoredToken(): string | null {
    if (typeof window !== 'undefined' && window.localStorage) {
      return localStorage.getItem('auth_token');
    }
    return null;
  }

  private setStoredToken(token: string): void {
    if (typeof window !== 'undefined' && window.localStorage) {
      localStorage.setItem('auth_token', token);
    }
    this.token = token;
  }

  private removeStoredToken(): void {
    if (typeof window !== 'undefined' && window.localStorage) {
      localStorage.removeItem('auth_token');
    }
    this.token = null;
  }

  private getHeaders(): HeadersInit {
    const headers: HeadersInit = {
      'Content-Type': 'application/json',
    };

    if (this.token) {
      headers['Authorization'] = `Bearer ${this.token}`;
    }

    return headers;
  }

  private async request<T>(
    endpoint: string,
    options: RequestInit = {}
  ): Promise<ApiResponse<T>> {
    try {
      const url = `${this.baseURL}${endpoint}`;
      const response = await fetch(url, {
        ...options,
        headers: {
          ...this.getHeaders(),
          ...options.headers,
        },
      });

      const data = await response.json();

      if (!response.ok) {
        return {
          success: false,
          error: data.error || data.message || 'Request failed',
        };
      }

      return {
        success: true,
        data: data.data || data,
        message: data.message,
      };
    } catch (error) {
      console.error('API request error:', error);
      return {
        success: false,
        error: error instanceof Error ? error.message : 'Network error',
      };
    }
  }

  // Authentication
  async login(credentials: LoginCredentials): Promise<ApiResponse<AuthResponse>> {
    const response = await this.request<AuthResponse>('/auth/login', {
      method: 'POST',
      body: JSON.stringify(credentials),
    });

    if (response.success && response.data?.token) {
      this.setStoredToken(response.data.token);
    }

    return response;
  }

  async logout(): Promise<void> {
    this.removeStoredToken();
  }

  async getCurrentUser(): Promise<ApiResponse<any>> {
    return this.request('/auth/me');
  }

  // Cases
  async getCases(params?: {
    status?: string;
    priority?: string;
    search?: string;
    page?: number;
    limit?: number;
  }): Promise<ApiResponse<{ cases: CaseData[]; total: number }>> {
    const queryParams = new URLSearchParams();
    if (params?.status) queryParams.append('status', params.status);
    if (params?.priority) queryParams.append('priority', params.priority);
    if (params?.search) queryParams.append('search', params.search);
    if (params?.page) queryParams.append('page', params.page.toString());
    if (params?.limit) queryParams.append('limit', params.limit.toString());

    const query = queryParams.toString();
    return this.request(`/cases${query ? `?${query}` : ''}`);
  }

  async getCase(id: string): Promise<ApiResponse<CaseData>> {
    return this.request(`/cases/${id}`);
  }

  async createCase(caseData: CaseData): Promise<ApiResponse<CaseData>> {
    return this.request('/cases', {
      method: 'POST',
      body: JSON.stringify(caseData),
    });
  }

  async updateCase(id: string, caseData: Partial<CaseData>): Promise<ApiResponse<CaseData>> {
    return this.request(`/cases/${id}`, {
      method: 'PUT',
      body: JSON.stringify(caseData),
    });
  }

  async deleteCase(id: string): Promise<ApiResponse<void>> {
    return this.request(`/cases/${id}`, {
      method: 'DELETE',
    });
  }

  async updateCaseStatus(id: string, status: string): Promise<ApiResponse<CaseData>> {
    return this.request(`/cases/${id}/status`, {
      method: 'PATCH',
      body: JSON.stringify({ status }),
    });
  }

  // Accidents
  async getAccident(caseId: string): Promise<ApiResponse<AccidentData>> {
    return this.request(`/accidents/case/${caseId}`);
  }

  async createAccident(accidentData: AccidentData): Promise<ApiResponse<AccidentData>> {
    return this.request('/accidents', {
      method: 'POST',
      body: JSON.stringify(accidentData),
    });
  }

  async updateAccident(
    id: string,
    accidentData: Partial<AccidentData>
  ): Promise<ApiResponse<AccidentData>> {
    return this.request(`/accidents/${id}`, {
      method: 'PUT',
      body: JSON.stringify(accidentData),
    });
  }

  // Vehicles
  async getVehicles(accidentId: string): Promise<ApiResponse<VehicleData[]>> {
    return this.request(`/vehicles/accident/${accidentId}`);
  }

  async createVehicle(vehicleData: VehicleData): Promise<ApiResponse<VehicleData>> {
    return this.request('/vehicles', {
      method: 'POST',
      body: JSON.stringify(vehicleData),
    });
  }

  async updateVehicle(
    id: string,
    vehicleData: Partial<VehicleData>
  ): Promise<ApiResponse<VehicleData>> {
    return this.request(`/vehicles/${id}`, {
      method: 'PUT',
      body: JSON.stringify(vehicleData),
    });
  }

  async deleteVehicle(id: string): Promise<ApiResponse<void>> {
    return this.request(`/vehicles/${id}`, {
      method: 'DELETE',
    });
  }

  // Reports
  async generateReport(caseId: string, reportType: string): Promise<ApiResponse<any>> {
    return this.request('/reports/generate', {
      method: 'POST',
      body: JSON.stringify({ caseId, reportType }),
    });
  }

  async getReports(caseId: string): Promise<ApiResponse<any[]>> {
    return this.request(`/reports/case/${caseId}`);
  }

  // Statistics
  async getStats(): Promise<
    ApiResponse<{
      totalCases: number;
      activeCases: number;
      completedCases: number;
      recentCases: CaseData[];
    }>
  > {
    return this.request('/stats/dashboard');
  }

  // Health check
  async healthCheck(): Promise<ApiResponse<{ status: string; timestamp: string }>> {
    return this.request('/health');
  }

  // Utility
  isAuthenticated(): boolean {
    return this.token !== null;
  }

  getToken(): string | null {
    return this.token;
  }

  setToken(token: string): void {
    this.setStoredToken(token);
  }
}

// Export singleton instance
export const api = new ApiService();
export default api;
