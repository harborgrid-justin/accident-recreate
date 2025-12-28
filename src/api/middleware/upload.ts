/**
 * File Upload Middleware
 * AccuScene Enterprise Accident Recreation Platform
 */

import multer, { StorageEngine, FileFilterCallback } from 'multer';
import { Request } from 'express';
import path from 'path';
import fs from 'fs';
import { AppError } from './errorHandler';
import { ErrorCode } from '../responses';

// File upload configuration
const UPLOAD_DIR = process.env.UPLOAD_DIR || './uploads';
const MAX_FILE_SIZE = 50 * 1024 * 1024; // 50MB
const MAX_FILES = 10;

// Ensure upload directory exists
if (!fs.existsSync(UPLOAD_DIR)) {
  fs.mkdirSync(UPLOAD_DIR, { recursive: true });
}

// Allowed file types
const ALLOWED_IMAGE_TYPES = [
  'image/jpeg',
  'image/jpg',
  'image/png',
  'image/gif',
  'image/webp',
];

const ALLOWED_DOCUMENT_TYPES = [
  'application/pdf',
  'application/msword',
  'application/vnd.openxmlformats-officedocument.wordprocessingml.document',
  'application/vnd.ms-excel',
  'application/vnd.openxmlformats-officedocument.spreadsheetml.sheet',
  'text/plain',
  'text/csv',
];

const ALLOWED_VIDEO_TYPES = [
  'video/mp4',
  'video/mpeg',
  'video/quicktime',
  'video/x-msvideo',
];

const ALLOWED_EVIDENCE_TYPES = [
  ...ALLOWED_IMAGE_TYPES,
  ...ALLOWED_DOCUMENT_TYPES,
  ...ALLOWED_VIDEO_TYPES,
];

/**
 * Storage configuration with custom filename
 */
const storage: StorageEngine = multer.diskStorage({
  destination: (req, file, cb) => {
    // Organize by evidence type
    const subDir = getSubDirectory(file.mimetype);
    const uploadPath = path.join(UPLOAD_DIR, subDir);

    // Create subdirectory if it doesn't exist
    if (!fs.existsSync(uploadPath)) {
      fs.mkdirSync(uploadPath, { recursive: true });
    }

    cb(null, uploadPath);
  },
  filename: (req, file, cb) => {
    // Generate unique filename: timestamp-random-originalname
    const uniqueSuffix = `${Date.now()}-${Math.round(Math.random() * 1e9)}`;
    const ext = path.extname(file.originalname);
    const basename = path.basename(file.originalname, ext)
      .replace(/[^a-zA-Z0-9]/g, '_')
      .substring(0, 50);

    cb(null, `${basename}-${uniqueSuffix}${ext}`);
  },
});

/**
 * Memory storage for temporary uploads
 */
const memoryStorage = multer.memoryStorage();

/**
 * Get subdirectory based on MIME type
 */
function getSubDirectory(mimetype: string): string {
  if (ALLOWED_IMAGE_TYPES.includes(mimetype)) {
    return 'images';
  } else if (ALLOWED_DOCUMENT_TYPES.includes(mimetype)) {
    return 'documents';
  } else if (ALLOWED_VIDEO_TYPES.includes(mimetype)) {
    return 'videos';
  }
  return 'other';
}

/**
 * File filter for evidence uploads
 */
const evidenceFileFilter = (
  req: Request,
  file: Express.Multer.File,
  cb: FileFilterCallback
): void => {
  if (ALLOWED_EVIDENCE_TYPES.includes(file.mimetype)) {
    cb(null, true);
  } else {
    cb(
      new AppError(
        `File type not allowed. Allowed types: images, documents, videos`,
        400,
        ErrorCode.VALIDATION_ERROR
      )
    );
  }
};

/**
 * File filter for images only
 */
const imageFileFilter = (
  req: Request,
  file: Express.Multer.File,
  cb: FileFilterCallback
): void => {
  if (ALLOWED_IMAGE_TYPES.includes(file.mimetype)) {
    cb(null, true);
  } else {
    cb(
      new AppError(
        `Only image files are allowed`,
        400,
        ErrorCode.VALIDATION_ERROR
      )
    );
  }
};

/**
 * File filter for documents only
 */
const documentFileFilter = (
  req: Request,
  file: Express.Multer.File,
  cb: FileFilterCallback
): void => {
  if (ALLOWED_DOCUMENT_TYPES.includes(file.mimetype)) {
    cb(null, true);
  } else {
    cb(
      new AppError(
        `Only document files are allowed`,
        400,
        ErrorCode.VALIDATION_ERROR
      )
    );
  }
};

/**
 * Evidence file upload (supports all allowed types)
 */
export const uploadEvidence = multer({
  storage,
  fileFilter: evidenceFileFilter,
  limits: {
    fileSize: MAX_FILE_SIZE,
    files: MAX_FILES,
  },
});

/**
 * Image upload only
 */
export const uploadImage = multer({
  storage,
  fileFilter: imageFileFilter,
  limits: {
    fileSize: MAX_FILE_SIZE,
    files: 1,
  },
});

/**
 * Document upload only
 */
export const uploadDocument = multer({
  storage,
  fileFilter: documentFileFilter,
  limits: {
    fileSize: MAX_FILE_SIZE,
    files: 1,
  },
});

/**
 * Memory storage for temporary processing
 */
export const uploadToMemory = multer({
  storage: memoryStorage,
  limits: {
    fileSize: MAX_FILE_SIZE,
  },
});

/**
 * Single file upload
 */
export const uploadSingle = (fieldName: string) => uploadEvidence.single(fieldName);

/**
 * Multiple files upload
 */
export const uploadMultiple = (fieldName: string, maxCount: number = MAX_FILES) =>
  uploadEvidence.array(fieldName, maxCount);

/**
 * Multiple fields upload
 */
export const uploadFields = (fields: { name: string; maxCount: number }[]) =>
  uploadEvidence.fields(fields);

/**
 * Get file information
 */
export interface FileInfo {
  fieldName: string;
  originalName: string;
  fileName: string;
  mimeType: string;
  size: number;
  path: string;
  url: string;
}

/**
 * Convert Multer file to FileInfo
 */
export function getFileInfo(file: Express.Multer.File): FileInfo {
  return {
    fieldName: file.fieldname,
    originalName: file.originalname,
    fileName: file.filename,
    mimeType: file.mimetype,
    size: file.size,
    path: file.path,
    url: `/uploads/${getSubDirectory(file.mimetype)}/${file.filename}`,
  };
}

/**
 * Get multiple files info
 */
export function getFilesInfo(files: Express.Multer.File[]): FileInfo[] {
  return files.map(file => getFileInfo(file));
}

/**
 * Delete uploaded file
 */
export function deleteFile(filePath: string): Promise<void> {
  return new Promise((resolve, reject) => {
    fs.unlink(filePath, (err) => {
      if (err) {
        reject(err);
      } else {
        resolve();
      }
    });
  });
}

/**
 * Validate file extension
 */
export function isValidFileExtension(filename: string, allowedExtensions: string[]): boolean {
  const ext = path.extname(filename).toLowerCase();
  return allowedExtensions.includes(ext);
}

/**
 * Get file size in human-readable format
 */
export function formatFileSize(bytes: number): string {
  if (bytes === 0) return '0 Bytes';

  const k = 1024;
  const sizes = ['Bytes', 'KB', 'MB', 'GB'];
  const i = Math.floor(Math.log(bytes) / Math.log(k));

  return Math.round((bytes / Math.pow(k, i)) * 100) / 100 + ' ' + sizes[i];
}
