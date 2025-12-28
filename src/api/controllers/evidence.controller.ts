/**
 * Evidence Controller
 * AccuScene Enterprise Accident Recreation Platform
 */

import { Request, Response, NextFunction } from 'express';
import { success, created, noContent, paginated } from '../responses';
import { asyncHandler, NotFoundError } from '../middleware/errorHandler';
import { getFileInfo, getFilesInfo } from '../middleware/upload';
import { EvidenceType } from '../validators/evidence.schemas';

/**
 * Get all evidence
 * GET /api/evidence
 */
export const getAllEvidence = asyncHandler(
  async (req: Request, res: Response, next: NextFunction) => {
    const filters = req.query;

    // TODO: Implement evidence service call
    // const evidenceService = new EvidenceService();
    // const result = await evidenceService.findAll(filters);

    // Mock response
    const mockEvidence = [
      {
        id: '523e4567-e89b-12d3-a456-426614174000',
        caseId: '223e4567-e89b-12d3-a456-426614174000',
        type: EvidenceType.PHOTO,
        title: 'Collision scene photo 1',
        description: 'Front view of intersection',
        fileUrl: '/uploads/images/scene-001.jpg',
        fileName: 'scene-001.jpg',
        fileSize: 2048576,
        mimeType: 'image/jpeg',
        tags: ['scene', 'intersection'],
        capturedAt: new Date(),
        createdAt: new Date(),
        updatedAt: new Date(),
      },
    ];

    res.status(200).json(
      paginated(
        mockEvidence,
        Number(filters.page) || 1,
        Number(filters.limit) || 20,
        1,
        (req as any).id
      )
    );
  }
);

/**
 * Get evidence by ID
 * GET /api/evidence/:evidenceId
 */
export const getEvidenceById = asyncHandler(
  async (req: Request, res: Response, next: NextFunction) => {
    const { evidenceId } = req.params;

    // TODO: Implement evidence service call
    // const evidenceService = new EvidenceService();
    // const evidence = await evidenceService.findById(evidenceId);
    // if (!evidence) throw new NotFoundError('Evidence', evidenceId);

    // Mock response
    const mockEvidence = {
      id: evidenceId,
      caseId: '223e4567-e89b-12d3-a456-426614174000',
      type: EvidenceType.PHOTO,
      title: 'Collision scene photo 1',
      description: 'Front view of intersection',
      fileUrl: '/uploads/images/scene-001.jpg',
      fileName: 'scene-001.jpg',
      fileSize: 2048576,
      mimeType: 'image/jpeg',
      tags: ['scene', 'intersection'],
      capturedAt: new Date(),
      location: {
        latitude: 39.7817,
        longitude: -89.6501,
      },
      metadata: {
        camera: 'iPhone 12',
        resolution: '4032x3024',
      },
      createdAt: new Date(),
      updatedAt: new Date(),
    };

    res.status(200).json(
      success(mockEvidence, undefined, (req as any).id)
    );
  }
);

/**
 * Upload evidence
 * POST /api/evidence/upload
 */
export const uploadEvidence = asyncHandler(
  async (req: Request, res: Response, next: NextFunction) => {
    const files = req.files as Express.Multer.File[];
    const metadata = req.body;
    const userId = req.user?.userId;

    if (!files || files.length === 0) {
      throw new NotFoundError('No files uploaded');
    }

    // TODO: Implement evidence service call to save file metadata
    // const evidenceService = new EvidenceService();
    // const evidence = await evidenceService.create(metadata, files, userId);

    // Mock response
    const filesInfo = getFilesInfo(files);
    const mockEvidence = filesInfo.map((fileInfo, index) => ({
      id: `523e4567-e89b-12d3-a456-42661417400${index}`,
      caseId: metadata.caseId,
      type: metadata.type,
      title: metadata.title || fileInfo.originalName,
      description: metadata.description,
      fileUrl: fileInfo.url,
      fileName: fileInfo.fileName,
      fileSize: fileInfo.size,
      mimeType: fileInfo.mimeType,
      tags: metadata.tags || [],
      uploadedById: userId,
      createdAt: new Date(),
    }));

    res.status(201).json(
      created(
        mockEvidence.length === 1 ? mockEvidence[0] : mockEvidence,
        'Evidence uploaded successfully',
        (req as any).id
      )
    );
  }
);

/**
 * Update evidence metadata
 * PUT /api/evidence/:evidenceId
 */
export const updateEvidence = asyncHandler(
  async (req: Request, res: Response, next: NextFunction) => {
    const { evidenceId } = req.params;
    const updateData = req.body;

    // TODO: Implement evidence service call
    // const evidenceService = new EvidenceService();
    // const evidence = await evidenceService.update(evidenceId, updateData);
    // if (!evidence) throw new NotFoundError('Evidence', evidenceId);

    // Mock response
    const mockEvidence = {
      id: evidenceId,
      ...updateData,
      updatedAt: new Date(),
    };

    res.status(200).json(
      success(mockEvidence, 'Evidence updated successfully', (req as any).id)
    );
  }
);

/**
 * Delete evidence
 * DELETE /api/evidence/:evidenceId
 */
export const deleteEvidence = asyncHandler(
  async (req: Request, res: Response, next: NextFunction) => {
    const { evidenceId } = req.params;

    // TODO: Implement evidence service call
    // const evidenceService = new EvidenceService();
    // const deleted = await evidenceService.delete(evidenceId);
    // if (!deleted) throw new NotFoundError('Evidence', evidenceId);

    res.status(200).json(
      noContent('Evidence deleted successfully', (req as any).id)
    );
  }
);

/**
 * Download evidence file
 * GET /api/evidence/:evidenceId/download
 */
export const downloadEvidence = asyncHandler(
  async (req: Request, res: Response, next: NextFunction) => {
    const { evidenceId } = req.params;

    // TODO: Implement evidence service call to get file path
    // const evidenceService = new EvidenceService();
    // const evidence = await evidenceService.findById(evidenceId);
    // if (!evidence) throw new NotFoundError('Evidence', evidenceId);
    // res.download(evidence.filePath, evidence.fileName);

    // For now, send mock download URL
    res.status(200).json(
      success(
        {
          downloadUrl: `/uploads/evidence/${evidenceId}/file.jpg`,
          expiresAt: new Date(Date.now() + 3600000), // 1 hour
        },
        'Download URL generated',
        (req as any).id
      )
    );
  }
);

/**
 * Get evidence by case
 * GET /api/evidence/case/:caseId
 */
export const getEvidenceByCase = asyncHandler(
  async (req: Request, res: Response, next: NextFunction) => {
    const { caseId } = req.params;
    const filters = req.query;

    // TODO: Implement evidence service call
    // const evidenceService = new EvidenceService();
    // const result = await evidenceService.findByCase(caseId, filters);

    // Mock response
    const mockEvidence = [
      {
        id: '523e4567-e89b-12d3-a456-426614174000',
        caseId,
        type: EvidenceType.PHOTO,
        title: 'Scene photo 1',
        fileUrl: '/uploads/images/scene-001.jpg',
        createdAt: new Date(),
      },
    ];

    res.status(200).json(
      paginated(
        mockEvidence,
        Number(filters.page) || 1,
        Number(filters.limit) || 20,
        1,
        (req as any).id
      )
    );
  }
);
