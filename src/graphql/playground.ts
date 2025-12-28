/**
 * GraphQL Playground Configuration
 * AccuScene Enterprise v0.2.0 - GraphQL Federation API
 */

/**
 * GraphQL Playground configuration
 * Provides an interactive IDE for exploring the GraphQL API
 */
export const playgroundConfig = {
  // Enable playground in development only
  enabled: process.env.NODE_ENV !== 'production',

  // Playground settings
  settings: {
    'editor.theme': 'dark',
    'editor.cursorShape': 'line',
    'editor.reuseHeaders': true,
    'tracing.hideTracingResponse': false,
    'queryPlan.hideQueryPlanResponse': false,
    'editor.fontSize': 14,
    'editor.fontFamily': "'Source Code Pro', 'Consolas', 'Monaco', monospace",
    'request.credentials': 'include',
  },

  // Tabs configuration
  tabs: [
    {
      name: 'Health Check',
      endpoint: '/graphql',
      query: `# Health Check Query
query HealthCheck {
  health {
    status
    version
    uptime
    timestamp
  }
}`,
    },
    {
      name: 'Get Cases',
      endpoint: '/graphql',
      query: `# Get Cases with Pagination
query GetCases($page: Int, $limit: Int) {
  cases(pagination: { page: $page, limit: $limit }) {
    items {
      id
      caseNumber
      title
      status
      priority
      incidentDate
      investigator {
        id
        fullName
        email
      }
    }
    total
    page
    totalPages
    hasNextPage
  }
}`,
      variables: `{
  "page": 1,
  "limit": 10
}`,
    },
    {
      name: 'Create Case',
      endpoint: '/graphql',
      query: `# Create a New Case
mutation CreateCase($input: CreateCaseInput!) {
  createCase(input: $input) {
    id
    caseNumber
    title
    status
    priority
    incidentDate
    location {
      address
      city
      state
    }
  }
}`,
      variables: `{
  "input": {
    "title": "Highway Collision - I-95",
    "description": "Multi-vehicle accident on Interstate 95",
    "incidentDate": "2025-12-28T10:30:00Z",
    "priority": "HIGH",
    "location": {
      "address": "I-95 Mile Marker 245",
      "city": "Richmond",
      "state": "VA",
      "zipCode": "23220",
      "country": "USA"
    }
  }
}`,
    },
    {
      name: 'Run Simulation',
      endpoint: '/graphql',
      query: `# Create and Start a Simulation
mutation RunSimulation($input: CreateSimulationInput!) {
  createSimulation(input: $input) {
    id
    name
    status
    parameters {
      timeStep
      duration
      physicsEngine
    }
  }
}`,
      variables: `{
  "input": {
    "caseId": "case-id-here",
    "sceneId": "scene-id-here",
    "name": "Impact Analysis",
    "parameters": {
      "timeStep": 0.016,
      "duration": 10.0,
      "iterations": 100,
      "physicsEngine": "ADVANCED",
      "environmentalFactors": {
        "roadFriction": 0.7,
        "airDensity": 1.225,
        "gravity": 9.81
      }
    }
  }
}`,
    },
    {
      name: 'Generate Report',
      endpoint: '/graphql',
      query: `# Generate a Case Report
mutation GenerateReport($input: GenerateReportInput!) {
  generateReport(input: $input) {
    id
    type
    format
    status
    title
    fileUrl
  }
}`,
      variables: `{
  "input": {
    "caseId": "case-id-here",
    "type": "TECHNICAL",
    "format": "PDF",
    "options": {
      "includeSimulations": true,
      "includePhotos": true,
      "includeMetrics": true
    }
  }
}`,
    },
    {
      name: 'Subscription - Case Updates',
      endpoint: '/graphql',
      query: `# Subscribe to Case Updates
subscription CaseUpdates($caseId: ID) {
  caseUpdated(caseId: $caseId) {
    case {
      id
      caseNumber
      title
      status
      updatedAt
    }
    mutation
    userId
  }
}`,
      variables: `{
  "caseId": "case-id-here"
}`,
    },
    {
      name: 'Subscription - Simulation Progress',
      endpoint: '/graphql',
      query: `# Subscribe to Simulation Progress
subscription SimulationProgress($simulationId: ID!) {
  simulationProgress(simulationId: $simulationId) {
    simulation {
      id
      name
      status
    }
    progress
    currentFrame
    totalFrames
  }
}`,
      variables: `{
  "simulationId": "simulation-id-here"
}`,
    },
  ],

  // Headers to include with requests
  headers: {
    // Example: Add authentication header
    // Authorization: 'Bearer YOUR_TOKEN_HERE'
  },
};

/**
 * Get example queries for documentation
 */
export function getExampleQueries() {
  return {
    queries: [
      {
        name: 'Get Case by ID',
        query: `query GetCase($id: ID!) {
  case(id: $id) {
    id
    caseNumber
    title
    status
    vehicles {
      id
      make
      model
      year
    }
    scenes {
      id
      name
      type
    }
  }
}`,
      },
      {
        name: 'Search Cases',
        query: `query SearchCases($query: String!) {
  searchCases(query: $query) {
    items {
      id
      caseNumber
      title
      status
    }
    total
  }
}`,
      },
      {
        name: 'Get My Cases',
        query: `query MyCases {
  myCases {
    items {
      id
      caseNumber
      title
      status
      priority
    }
  }
}`,
      },
    ],
    mutations: [
      {
        name: 'Update Case',
        query: `mutation UpdateCase($id: ID!, $input: UpdateCaseInput!) {
  updateCase(id: $id, input: $input) {
    id
    title
    status
    updatedAt
  }
}`,
      },
      {
        name: 'Add Vehicle',
        query: `mutation AddVehicle($input: CreateVehicleInput!) {
  createVehicle(input: $input) {
    id
    make
    model
    year
    type
  }
}`,
      },
    ],
    subscriptions: [
      {
        name: 'Case Updates',
        query: `subscription {
  caseUpdated {
    case {
      id
      title
      status
    }
    mutation
  }
}`,
      },
    ],
  };
}
