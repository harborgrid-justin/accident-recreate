/**
 * AccuScene Enterprise v0.3.0 - Chat System
 *
 * Real-time text chat with threads, mentions, and reactions
 */

import { EventEmitter } from 'events';
import {
  ChatMessage,
  MessageId,
  SessionId,
  UserId,
  User,
  Attachment
} from './types';

interface ChatConfig {
  maxMessageLength: number;
  maxAttachments: number;
  enableThreads: boolean;
  enableReactions: boolean;
  enableMentions: boolean;
}

export class ChatSystem extends EventEmitter {
  private messages: Map<MessageId, ChatMessage> = new Map();
  private threads: Map<MessageId, ChatMessage[]> = new Map();
  private sessionId: SessionId | null = null;
  private currentUser: User | null = null;

  private config: ChatConfig;

  // Typing indicators
  private typingUsers: Map<UserId, NodeJS.Timeout> = new Map();
  private readonly TYPING_TIMEOUT = 3000; // 3 seconds

  constructor(config: Partial<ChatConfig> = {}) {
    super();

    this.config = {
      maxMessageLength: 5000,
      maxAttachments: 5,
      enableThreads: true,
      enableReactions: true,
      enableMentions: true,
      ...config
    };
  }

  // ============================================================================
  // Initialization
  // ============================================================================

  initialize(sessionId: SessionId, user: User): void {
    this.sessionId = sessionId;
    this.currentUser = user;
  }

  // ============================================================================
  // Message Operations
  // ============================================================================

  async sendMessage(content: string, attachments?: Attachment[], threadId?: MessageId): Promise<ChatMessage> {
    if (!this.currentUser || !this.sessionId) {
      throw new Error('Chat system not initialized');
    }

    if (content.length > this.config.maxMessageLength) {
      throw new Error(`Message exceeds maximum length of ${this.config.maxMessageLength}`);
    }

    if (attachments && attachments.length > this.config.maxAttachments) {
      throw new Error(`Too many attachments (max: ${this.config.maxAttachments})`);
    }

    // Extract mentions
    const mentions = this.config.enableMentions ? this.extractMentions(content) : [];

    const message: ChatMessage = {
      id: this.generateMessageId(),
      sessionId: this.sessionId,
      userId: this.currentUser.id,
      content,
      timestamp: Date.now(),
      threadId,
      mentions,
      attachments,
      reactions: new Map()
    };

    this.messages.set(message.id, message);

    // Add to thread if applicable
    if (threadId && this.config.enableThreads) {
      this.addToThread(threadId, message);
    }

    // Clear typing indicator
    this.setTyping(this.currentUser.id, false);

    this.emit('messageSent', message);

    return message;
  }

  async editMessage(messageId: MessageId, newContent: string): Promise<void> {
    const message = this.messages.get(messageId);
    if (!message) {
      throw new Error(`Message not found: ${messageId}`);
    }

    if (message.userId !== this.currentUser?.id) {
      throw new Error('Cannot edit message from another user');
    }

    if (newContent.length > this.config.maxMessageLength) {
      throw new Error(`Message exceeds maximum length of ${this.config.maxMessageLength}`);
    }

    message.content = newContent;
    message.edited = true;
    message.editedAt = Date.now();

    // Update mentions
    if (this.config.enableMentions) {
      message.mentions = this.extractMentions(newContent);
    }

    this.messages.set(messageId, message);
    this.emit('messageEdited', message);
  }

  async deleteMessage(messageId: MessageId): Promise<void> {
    const message = this.messages.get(messageId);
    if (!message) {
      throw new Error(`Message not found: ${messageId}`);
    }

    if (message.userId !== this.currentUser?.id) {
      throw new Error('Cannot delete message from another user');
    }

    this.messages.delete(messageId);

    // Remove from thread
    if (message.threadId) {
      this.removeFromThread(message.threadId, messageId);
    }

    this.emit('messageDeleted', { messageId, message });
  }

  receiveMessage(message: ChatMessage): void {
    this.messages.set(message.id, message);

    // Add to thread if applicable
    if (message.threadId && this.config.enableThreads) {
      this.addToThread(message.threadId, message);
    }

    this.emit('messageReceived', message);

    // Notify mentions
    if (this.config.enableMentions && message.mentions) {
      for (const userId of message.mentions) {
        if (userId === this.currentUser?.id) {
          this.emit('mentioned', message);
          break;
        }
      }
    }
  }

  // ============================================================================
  // Reactions
  // ============================================================================

  async addReaction(messageId: MessageId, emoji: string): Promise<void> {
    if (!this.config.enableReactions) {
      throw new Error('Reactions are disabled');
    }

    if (!this.currentUser) {
      throw new Error('No current user');
    }

    const message = this.messages.get(messageId);
    if (!message) {
      throw new Error(`Message not found: ${messageId}`);
    }

    if (!message.reactions) {
      message.reactions = new Map();
    }

    const users = message.reactions.get(emoji) || [];
    if (!users.includes(this.currentUser.id)) {
      users.push(this.currentUser.id);
      message.reactions.set(emoji, users);
      this.emit('reactionAdded', { messageId, emoji, userId: this.currentUser.id });
    }
  }

  async removeReaction(messageId: MessageId, emoji: string): Promise<void> {
    if (!this.currentUser) {
      throw new Error('No current user');
    }

    const message = this.messages.get(messageId);
    if (!message || !message.reactions) {
      return;
    }

    const users = message.reactions.get(emoji);
    if (users) {
      const index = users.indexOf(this.currentUser.id);
      if (index !== -1) {
        users.splice(index, 1);
        if (users.length === 0) {
          message.reactions.delete(emoji);
        }
        this.emit('reactionRemoved', { messageId, emoji, userId: this.currentUser.id });
      }
    }
  }

  // ============================================================================
  // Threads
  // ============================================================================

  private addToThread(threadId: MessageId, message: ChatMessage): void {
    const thread = this.threads.get(threadId) || [];
    thread.push(message);
    this.threads.set(threadId, thread);
    this.emit('threadUpdated', { threadId, message });
  }

  private removeFromThread(threadId: MessageId, messageId: MessageId): void {
    const thread = this.threads.get(threadId);
    if (thread) {
      const index = thread.findIndex(m => m.id === messageId);
      if (index !== -1) {
        thread.splice(index, 1);
        if (thread.length === 0) {
          this.threads.delete(threadId);
        }
      }
    }
  }

  getThread(messageId: MessageId): ChatMessage[] {
    return this.threads.get(messageId) || [];
  }

  getAllThreads(): Map<MessageId, ChatMessage[]> {
    return new Map(this.threads);
  }

  // ============================================================================
  // Typing Indicators
  // ============================================================================

  setTyping(userId: UserId, isTyping: boolean): void {
    const existing = this.typingUsers.get(userId);

    if (isTyping) {
      // Clear existing timeout
      if (existing) {
        clearTimeout(existing);
      }

      // Set new timeout
      const timeout = setTimeout(() => {
        this.typingUsers.delete(userId);
        this.emit('typingChanged', { userId, isTyping: false });
      }, this.TYPING_TIMEOUT);

      this.typingUsers.set(userId, timeout);
      this.emit('typingChanged', { userId, isTyping: true });

    } else {
      if (existing) {
        clearTimeout(existing);
        this.typingUsers.delete(userId);
        this.emit('typingChanged', { userId, isTyping: false });
      }
    }
  }

  getTypingUsers(): UserId[] {
    return Array.from(this.typingUsers.keys());
  }

  // ============================================================================
  // Mentions
  // ============================================================================

  private extractMentions(content: string): UserId[] {
    const mentionRegex = /@(\w+)/g;
    const mentions: UserId[] = [];
    let match;

    while ((match = mentionRegex.exec(content)) !== null) {
      mentions.push(match[1]);
    }

    return mentions;
  }

  getMentions(userId: UserId): ChatMessage[] {
    return Array.from(this.messages.values())
      .filter(m => m.mentions?.includes(userId));
  }

  // ============================================================================
  // Message Retrieval
  // ============================================================================

  getMessage(messageId: MessageId): ChatMessage | null {
    return this.messages.get(messageId) || null;
  }

  getMessages(limit = 100, before?: number): ChatMessage[] {
    let messages = Array.from(this.messages.values())
      .filter(m => !m.threadId); // Only root messages

    if (before) {
      messages = messages.filter(m => m.timestamp < before);
    }

    return messages
      .sort((a, b) => b.timestamp - a.timestamp)
      .slice(0, limit);
  }

  getMessagesByUser(userId: UserId): ChatMessage[] {
    return Array.from(this.messages.values())
      .filter(m => m.userId === userId);
  }

  searchMessages(query: string): ChatMessage[] {
    const lowerQuery = query.toLowerCase();

    return Array.from(this.messages.values())
      .filter(m => m.content.toLowerCase().includes(lowerQuery));
  }

  // ============================================================================
  // Attachments
  // ============================================================================

  getMessagesWithAttachments(): ChatMessage[] {
    return Array.from(this.messages.values())
      .filter(m => m.attachments && m.attachments.length > 0);
  }

  getAttachments(): Attachment[] {
    const attachments: Attachment[] = [];

    for (const message of this.messages.values()) {
      if (message.attachments) {
        attachments.push(...message.attachments);
      }
    }

    return attachments;
  }

  // ============================================================================
  // Statistics
  // ============================================================================

  getStatistics() {
    const messages = Array.from(this.messages.values());
    const userMessageCounts = new Map<UserId, number>();

    for (const message of messages) {
      userMessageCounts.set(
        message.userId,
        (userMessageCounts.get(message.userId) || 0) + 1
      );
    }

    const mostActiveUser = Array.from(userMessageCounts.entries())
      .sort((a, b) => b[1] - a[1])[0];

    return {
      totalMessages: messages.length,
      totalThreads: this.threads.size,
      messagesWithAttachments: this.getMessagesWithAttachments().length,
      totalReactions: messages.reduce((sum, m) => {
        if (!m.reactions) return sum;
        return sum + Array.from(m.reactions.values()).reduce((s, users) => s + users.length, 0);
      }, 0),
      uniqueUsers: new Set(messages.map(m => m.userId)).size,
      mostActiveUser: mostActiveUser ? {
        userId: mostActiveUser[0],
        messageCount: mostActiveUser[1]
      } : null,
      typingUsers: this.getTypingUsers().length
    };
  }

  // ============================================================================
  // Cleanup
  // ============================================================================

  clear(): void {
    this.messages.clear();
    this.threads.clear();
    for (const timeout of this.typingUsers.values()) {
      clearTimeout(timeout);
    }
    this.typingUsers.clear();
    this.emit('cleared');
  }

  destroy(): void {
    this.clear();
    this.removeAllListeners();
  }

  // ============================================================================
  // Helpers
  // ============================================================================

  private generateMessageId(): MessageId {
    return `msg-${Date.now()}-${Math.random().toString(36).substr(2, 9)}`;
  }
}
