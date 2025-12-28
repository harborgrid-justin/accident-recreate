/**
 * AccuScene Enterprise - Merkle Tree for Synchronization
 * v0.2.0
 *
 * Merkle tree implementation for efficient state comparison and sync
 */

import { createHash } from 'crypto';
import { Operation, MerkleNode } from '../types';

/**
 * Merkle Tree for efficient operation synchronization
 */
export class MerkleTree {
  private root: MerkleNode | null = null;
  private operations: Operation[] = [];
  private leafSize: number;

  constructor(operations: Operation[] = [], leafSize: number = 8) {
    this.leafSize = leafSize;
    this.operations = [...operations].sort((a, b) =>
      a.timestamp.counter - b.timestamp.counter
    );
    this.build();
  }

  /**
   * Build the merkle tree from operations
   */
  private build(): void {
    if (this.operations.length === 0) {
      this.root = null;
      return;
    }

    const leaves = this.createLeaves();
    this.root = this.buildTree(leaves);
  }

  /**
   * Create leaf nodes from operations
   */
  private createLeaves(): MerkleNode[] {
    const leaves: MerkleNode[] = [];

    for (let i = 0; i < this.operations.length; i += this.leafSize) {
      const chunk = this.operations.slice(i, i + this.leafSize);
      const hash = this.hashOperations(chunk);

      leaves.push({
        hash,
        data: chunk,
      });
    }

    return leaves;
  }

  /**
   * Build tree from leaf nodes
   */
  private buildTree(nodes: MerkleNode[]): MerkleNode {
    if (nodes.length === 0) {
      return { hash: this.hashString('') };
    }

    if (nodes.length === 1) {
      return nodes[0];
    }

    const parents: MerkleNode[] = [];

    for (let i = 0; i < nodes.length; i += 2) {
      const left = nodes[i];
      const right = nodes[i + 1];

      if (right) {
        const hash = this.hashString(left.hash + right.hash);
        parents.push({
          hash,
          left,
          right,
        });
      } else {
        parents.push(left);
      }
    }

    return this.buildTree(parents);
  }

  /**
   * Hash a list of operations
   */
  private hashOperations(operations: Operation[]): string {
    const data = operations
      .map(op => `${op.id}:${op.timestamp.counter}`)
      .join('|');
    return this.hashString(data);
  }

  /**
   * Hash a string using SHA-256
   */
  private hashString(data: string): string {
    return createHash('sha256').update(data).digest('hex');
  }

  /**
   * Get the root hash
   */
  getRootHash(): string {
    return this.root?.hash || this.hashString('');
  }

  /**
   * Get the root node
   */
  getRoot(): MerkleNode | null {
    return this.root;
  }

  /**
   * Add operations and rebuild tree
   */
  addOperations(operations: Operation[]): void {
    this.operations.push(...operations);
    this.operations.sort((a, b) => a.timestamp.counter - b.timestamp.counter);
    this.build();
  }

  /**
   * Find missing operations by comparing with another tree
   */
  findMissing(otherTree: MerkleTree): Operation[] {
    if (!this.root || !otherTree.root) {
      return [];
    }

    if (this.root.hash === otherTree.root.hash) {
      return [];
    }

    return this.compareTrees(this.root, otherTree.root);
  }

  /**
   * Compare two tree nodes recursively to find differences
   */
  private compareTrees(node1: MerkleNode, node2: MerkleNode): Operation[] {
    if (node1.hash === node2.hash) {
      return [];
    }

    // Leaf nodes - return operations that differ
    if (node1.data && node2.data) {
      const ops1Set = new Set(node1.data.map(op => op.id));
      const ops2Set = new Set(node2.data.map(op => op.id));

      const missing: Operation[] = [];

      for (const op of node2.data) {
        if (!ops1Set.has(op.id)) {
          missing.push(op);
        }
      }

      return missing;
    }

    // Only one is a leaf
    if (node1.data) {
      return node2.data || [];
    }
    if (node2.data) {
      return [];
    }

    // Both are internal nodes
    const missing: Operation[] = [];

    if (node1.left && node2.left) {
      missing.push(...this.compareTrees(node1.left, node2.left));
    } else if (node2.left) {
      missing.push(...this.collectOperations(node2.left));
    }

    if (node1.right && node2.right) {
      missing.push(...this.compareTrees(node1.right, node2.right));
    } else if (node2.right) {
      missing.push(...this.collectOperations(node2.right));
    }

    return missing;
  }

  /**
   * Collect all operations from a subtree
   */
  private collectOperations(node: MerkleNode): Operation[] {
    if (node.data) {
      return node.data;
    }

    const operations: Operation[] = [];

    if (node.left) {
      operations.push(...this.collectOperations(node.left));
    }
    if (node.right) {
      operations.push(...this.collectOperations(node.right));
    }

    return operations;
  }

  /**
   * Get all operations in the tree
   */
  getOperations(): Operation[] {
    return [...this.operations];
  }

  /**
   * Get the number of operations
   */
  size(): number {
    return this.operations.length;
  }

  /**
   * Verify the integrity of the tree
   */
  verify(): boolean {
    if (!this.root) {
      return this.operations.length === 0;
    }

    return this.verifyNode(this.root);
  }

  /**
   * Verify a node and its children
   */
  private verifyNode(node: MerkleNode): boolean {
    // Leaf node
    if (node.data) {
      const expectedHash = this.hashOperations(node.data);
      return node.hash === expectedHash;
    }

    // Internal node
    if (!node.left || !node.right) {
      // Single child
      if (node.left) {
        return this.verifyNode(node.left);
      }
      if (node.right) {
        return this.verifyNode(node.right);
      }
      return false;
    }

    const expectedHash = this.hashString(node.left.hash + node.right.hash);
    return (
      node.hash === expectedHash &&
      this.verifyNode(node.left) &&
      this.verifyNode(node.right)
    );
  }

  /**
   * Export tree state for serialization
   */
  toJSON(): { operations: Operation[]; leafSize: number; rootHash: string } {
    return {
      operations: this.operations,
      leafSize: this.leafSize,
      rootHash: this.getRootHash(),
    };
  }

  /**
   * Import tree state from serialized data
   */
  static fromJSON(data: { operations: Operation[]; leafSize: number }): MerkleTree {
    return new MerkleTree(data.operations, data.leafSize);
  }

  /**
   * Get a proof path for an operation (for efficient verification)
   */
  getProof(operationId: string): MerkleNode[] {
    const proof: MerkleNode[] = [];

    if (!this.root) {
      return proof;
    }

    this.buildProof(this.root, operationId, proof);
    return proof;
  }

  /**
   * Build a proof path for an operation
   */
  private buildProof(node: MerkleNode, operationId: string, proof: MerkleNode[]): boolean {
    // Leaf node
    if (node.data) {
      return node.data.some(op => op.id === operationId);
    }

    // Check left subtree
    if (node.left && this.buildProof(node.left, operationId, proof)) {
      if (node.right) {
        proof.push(node.right);
      }
      return true;
    }

    // Check right subtree
    if (node.right && this.buildProof(node.right, operationId, proof)) {
      if (node.left) {
        proof.push(node.left);
      }
      return true;
    }

    return false;
  }
}
