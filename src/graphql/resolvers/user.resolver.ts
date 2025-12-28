/**
 * User Resolvers
 * AccuScene Enterprise v0.2.0 - GraphQL Federation API
 */

import {
  GraphQLContext,
  User,
  UserRole,
} from '../types';
import { v4 as uuidv4 } from 'uuid';
import { GraphQLError } from 'graphql';
import bcrypt from 'bcrypt';
import { createJWT } from '../context';

export const userQueries = {
  async me(
    _: unknown,
    __: unknown,
    context: GraphQLContext
  ): Promise<User | null> {
    if (!context.user) {
      throw new GraphQLError('Authentication required', {
        extensions: { code: 'UNAUTHENTICATED' },
      });
    }

    return context.dataloaders.userLoader.load(context.user.id);
  },

  async user(
    _: unknown,
    { id }: { id: string },
    context: GraphQLContext
  ): Promise<User | null> {
    return context.dataloaders.userLoader.load(id);
  },

  async users(
    _: unknown,
    { pagination }: { pagination?: any },
    context: GraphQLContext
  ): Promise<any> {
    // TODO: Implement database query
    return {
      items: [],
      total: 0,
      page: 1,
      limit: 20,
      totalPages: 0,
      hasNextPage: false,
      hasPreviousPage: false,
    };
  },

  async searchUsers(
    _: unknown,
    { query, pagination }: { query: string; pagination?: any },
    context: GraphQLContext
  ): Promise<any> {
    // TODO: Implement database query
    return {
      items: [],
      total: 0,
      page: 1,
      limit: 20,
      totalPages: 0,
      hasNextPage: false,
      hasPreviousPage: false,
    };
  },

  async userActivity(
    _: unknown,
    { userId, pagination }: { userId: string; pagination?: any },
    context: GraphQLContext
  ): Promise<any> {
    // TODO: Implement database query
    return {
      items: [],
      total: 0,
      page: 1,
      limit: 20,
      totalPages: 0,
      hasNextPage: false,
      hasPreviousPage: false,
    };
  },
};

export const userMutations = {
  async register(
    _: unknown,
    { input }: { input: any },
    context: GraphQLContext
  ): Promise<any> {
    // Hash password
    const hashedPassword = await bcrypt.hash(input.password, 10);

    // Create organization
    const organizationId = uuidv4();
    const organization = {
      id: organizationId,
      name: input.organizationName,
      type: input.organizationType,
      licenseType: 'TRIAL',
      createdAt: new Date(),
      updatedAt: new Date(),
    };

    // Create user
    const userId = uuidv4();
    const now = new Date();
    const user: User = {
      id: userId,
      email: input.email,
      firstName: input.firstName,
      lastName: input.lastName,
      role: UserRole.ADMIN,
      organizationId,
      permissions: [],
      avatar: undefined,
      preferences: {
        theme: 'LIGHT',
        notifications: {
          email: true,
          push: true,
          inApp: true,
        },
        defaultMeasurementUnit: 'FEET',
        locale: 'en-US',
        timezone: 'America/New_York',
      },
      lastLoginAt: now,
      createdAt: now,
      updatedAt: now,
    };

    // TODO: Save to database

    // Generate tokens
    const token = createJWT(context.user || {
      id: user.id,
      email: user.email,
      role: user.role,
      organizationId: user.organizationId,
      permissions: [],
    });
    const refreshToken = createJWT(context.user || {
      id: user.id,
      email: user.email,
      role: user.role,
      organizationId: user.organizationId,
      permissions: [],
    }, '30d');

    return {
      token,
      refreshToken,
      user,
      expiresIn: 7 * 24 * 60 * 60, // 7 days in seconds
    };
  },

  async login(
    _: unknown,
    { input }: { input: any },
    context: GraphQLContext
  ): Promise<any> {
    // TODO: Find user by email in database
    // const user = await findUserByEmail(input.email);

    // Mock user for now
    const mockUser: User = {
      id: uuidv4(),
      email: input.email,
      firstName: 'John',
      lastName: 'Doe',
      role: UserRole.INVESTIGATOR,
      organizationId: uuidv4(),
      permissions: [],
      preferences: {
        theme: 'LIGHT',
        notifications: { email: true, push: true, inApp: true },
        defaultMeasurementUnit: 'FEET',
        locale: 'en-US',
        timezone: 'America/New_York',
      },
      lastLoginAt: new Date(),
      createdAt: new Date(),
      updatedAt: new Date(),
    };

    // TODO: Verify password
    // const isValidPassword = await bcrypt.compare(input.password, user.password);

    const token = createJWT({
      id: mockUser.id,
      email: mockUser.email,
      role: mockUser.role,
      organizationId: mockUser.organizationId,
      permissions: mockUser.permissions,
    });

    const refreshToken = createJWT({
      id: mockUser.id,
      email: mockUser.email,
      role: mockUser.role,
      organizationId: mockUser.organizationId,
      permissions: mockUser.permissions,
    }, '30d');

    return {
      token,
      refreshToken,
      user: mockUser,
      expiresIn: 7 * 24 * 60 * 60,
    };
  },

  async refreshToken(
    _: unknown,
    { refreshToken }: { refreshToken: string },
    context: GraphQLContext
  ): Promise<any> {
    // TODO: Verify refresh token and generate new tokens
    throw new GraphQLError('Not implemented', {
      extensions: { code: 'NOT_IMPLEMENTED' },
    });
  },

  async logout(
    _: unknown,
    __: unknown,
    context: GraphQLContext
  ): Promise<boolean> {
    // TODO: Invalidate token
    return true;
  },

  async updateProfile(
    _: unknown,
    { input }: { input: any },
    context: GraphQLContext
  ): Promise<User> {
    if (!context.user) {
      throw new GraphQLError('Authentication required', {
        extensions: { code: 'UNAUTHENTICATED' },
      });
    }

    const user = await context.dataloaders.userLoader.load(context.user.id);
    if (!user) {
      throw new GraphQLError('User not found', {
        extensions: { code: 'NOT_FOUND' },
      });
    }

    const updatedUser: User = {
      ...user,
      ...input,
      updatedAt: new Date(),
    };

    // TODO: Save to database
    return updatedUser;
  },

  async changePassword(
    _: unknown,
    { currentPassword, newPassword }: { currentPassword: string; newPassword: string },
    context: GraphQLContext
  ): Promise<boolean> {
    if (!context.user) {
      throw new GraphQLError('Authentication required', {
        extensions: { code: 'UNAUTHENTICATED' },
      });
    }

    // TODO: Verify current password and update to new password
    return true;
  },

  async requestPasswordReset(
    _: unknown,
    { email }: { email: string },
    context: GraphQLContext
  ): Promise<boolean> {
    // TODO: Generate reset token and send email
    return true;
  },

  async resetPassword(
    _: unknown,
    { token, newPassword }: { token: string; newPassword: string },
    context: GraphQLContext
  ): Promise<boolean> {
    // TODO: Verify reset token and update password
    return true;
  },

  async createUser(
    _: unknown,
    { input }: { input: any },
    context: GraphQLContext
  ): Promise<User> {
    if (!context.user) {
      throw new GraphQLError('Authentication required', {
        extensions: { code: 'UNAUTHENTICATED' },
      });
    }

    const hashedPassword = await bcrypt.hash(input.password, 10);
    const now = new Date();

    const newUser: User = {
      id: uuidv4(),
      email: input.email,
      firstName: input.firstName,
      lastName: input.lastName,
      role: input.role,
      organizationId: input.organizationId,
      permissions: [],
      preferences: {
        theme: 'LIGHT',
        notifications: { email: true, push: true, inApp: true },
        defaultMeasurementUnit: 'FEET',
        locale: 'en-US',
        timezone: 'America/New_York',
      },
      createdAt: now,
      updatedAt: now,
    };

    // TODO: Save to database
    return newUser;
  },

  async updateUserRole(
    _: unknown,
    { userId, input }: { userId: string; input: any },
    context: GraphQLContext
  ): Promise<User> {
    const user = await context.dataloaders.userLoader.load(userId);
    if (!user) {
      throw new GraphQLError('User not found', {
        extensions: { code: 'NOT_FOUND' },
      });
    }

    const updatedUser: User = {
      ...user,
      role: input.role,
      permissions: input.permissions || user.permissions,
      updatedAt: new Date(),
    };

    // TODO: Save to database
    return updatedUser;
  },

  async deleteUser(
    _: unknown,
    { userId }: { userId: string },
    context: GraphQLContext
  ): Promise<boolean> {
    // TODO: Delete from database
    return true;
  },

  async uploadAvatar(
    _: unknown,
    { file }: { file: any },
    context: GraphQLContext
  ): Promise<User> {
    if (!context.user) {
      throw new GraphQLError('Authentication required', {
        extensions: { code: 'UNAUTHENTICATED' },
      });
    }

    // TODO: Upload file and get URL
    const avatarUrl = `https://storage.accuscene.com/avatars/${uuidv4()}.jpg`;

    const user = await context.dataloaders.userLoader.load(context.user.id);
    if (!user) {
      throw new GraphQLError('User not found', {
        extensions: { code: 'NOT_FOUND' },
      });
    }

    const updatedUser: User = {
      ...user,
      avatar: avatarUrl,
      updatedAt: new Date(),
    };

    // TODO: Save to database
    return updatedUser;
  },
};

export const userFieldResolvers = {
  fullName(parent: User) {
    return `${parent.firstName} ${parent.lastName}`;
  },

  async organization(parent: User, _: unknown, context: GraphQLContext) {
    // TODO: Load organization
    return null;
  },
};

export const userReference = {
  __resolveReference: async (
    reference: { __typename: string; id: string },
    context: GraphQLContext
  ): Promise<User | null> => {
    return context.dataloaders.userLoader.load(reference.id);
  },
};
