/**
 * Scene GraphQL Schema
 * AccuScene Enterprise v0.2.0 - GraphQL Federation API
 */

import { gql } from 'graphql-tag';

export const sceneTypeDefs = gql`
  """
  Represents a reconstructed accident scene
  """
  type Scene @key(fields: "id") {
    id: ID!
    caseId: ID!
    case: Case
    name: String!
    description: String
    type: SceneType!
    elements: [SceneElement!]!
    dimensions: SceneDimensions!
    viewState: ViewState!
    metadata: JSONObject
    createdAt: DateTime!
    updatedAt: DateTime!
  }

  """
  Scene type enumeration
  """
  enum SceneType {
    INTERSECTION
    HIGHWAY
    PARKING_LOT
    RESIDENTIAL
    CUSTOM
  }

  """
  Individual element within a scene
  """
  type SceneElement {
    id: ID!
    type: ElementType!
    position: Position!
    rotation: Float!
    properties: JSONObject
  }

  """
  Element type enumeration
  """
  enum ElementType {
    VEHICLE
    ROAD
    TRAFFIC_SIGN
    TRAFFIC_LIGHT
    BUILDING
    TREE
    MARKER
    MEASUREMENT
  }

  """
  Position coordinates in 2D/3D space
  """
  type Position {
    x: Float!
    y: Float!
    z: Float
  }

  """
  Scene dimensions and scale
  """
  type SceneDimensions {
    width: Float!
    height: Float!
    scale: Float!
    unit: MeasurementUnit!
  }

  """
  View state for scene camera
  """
  type ViewState {
    zoom: Float!
    center: Position!
    rotation: Float!
  }

  """
  Input type for creating a scene
  """
  input CreateSceneInput {
    caseId: ID!
    name: String!
    description: String
    type: SceneType!
    dimensions: SceneDimensionsInput
  }

  """
  Input type for updating a scene
  """
  input UpdateSceneInput {
    name: String
    description: String
    type: SceneType
    dimensions: SceneDimensionsInput
    viewState: ViewStateInput
  }

  """
  Input type for scene dimensions
  """
  input SceneDimensionsInput {
    width: Float!
    height: Float!
    scale: Float!
    unit: MeasurementUnit!
  }

  """
  Input type for position
  """
  input PositionInput {
    x: Float!
    y: Float!
    z: Float
  }

  """
  Input type for view state
  """
  input ViewStateInput {
    zoom: Float!
    center: PositionInput!
    rotation: Float!
  }

  """
  Input type for scene element
  """
  input SceneElementInput {
    type: ElementType!
    position: PositionInput!
    rotation: Float!
    properties: JSONObject
  }

  extend type Query {
    """
    Get a single scene by ID
    """
    scene(id: ID!): Scene @auth(requires: [CASE_READ])

    """
    Get all scenes for a case
    """
    scenesByCase(caseId: ID!): [Scene!]! @auth(requires: [CASE_READ])

    """
    Get scene template by type
    """
    sceneTemplate(type: SceneType!): Scene @auth(requires: [CASE_READ])
  }

  extend type Mutation {
    """
    Create a new scene
    """
    createScene(input: CreateSceneInput!): Scene! @auth(requires: [CASE_UPDATE])

    """
    Update an existing scene
    """
    updateScene(id: ID!, input: UpdateSceneInput!): Scene! @auth(requires: [CASE_UPDATE])

    """
    Delete a scene
    """
    deleteScene(id: ID!): Boolean! @auth(requires: [CASE_UPDATE])

    """
    Add element to scene
    """
    addSceneElement(sceneId: ID!, element: SceneElementInput!): Scene! @auth(requires: [CASE_UPDATE])

    """
    Update scene element
    """
    updateSceneElement(sceneId: ID!, elementId: ID!, element: SceneElementInput!): Scene! @auth(requires: [CASE_UPDATE])

    """
    Remove element from scene
    """
    removeSceneElement(sceneId: ID!, elementId: ID!): Scene! @auth(requires: [CASE_UPDATE])

    """
    Duplicate a scene
    """
    duplicateScene(id: ID!): Scene! @auth(requires: [CASE_UPDATE])

    """
    Update scene view state
    """
    updateSceneViewState(sceneId: ID!, viewState: ViewStateInput!): Scene! @auth(requires: [CASE_UPDATE])
  }
`;
