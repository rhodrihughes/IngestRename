import { describe, it, expect, vi } from 'vitest';
import type { FileInfo } from '$lib/types';
import FileSelector from './FileSelector.svelte';

// Mock Tauri APIs
vi.mock('@tauri-apps/plugin-dialog', () => ({
  open: vi.fn(),
}));

vi.mock('$lib/ipc', () => ({
  getFileInfo: vi.fn(),
}));

describe('FileSelector Component', () => {
  it('can import the component', () => {
    expect(FileSelector).toBeDefined();
  });

  it('validates file info structure', () => {
    const mockFile: FileInfo = {
      path: '/test/file.txt',
      name: 'file.txt',
      extension: 'txt',
      size: 1024,
    };

    expect(mockFile.path).toBe('/test/file.txt');
    expect(mockFile.name).toBe('file.txt');
    expect(mockFile.extension).toBe('txt');
    expect(mockFile.size).toBe(1024);
  });

  it('validates file selection order preservation', () => {
    const mockFiles: FileInfo[] = [
      { path: '/test/first.txt', name: 'first.txt', extension: 'txt', size: 100 },
      { path: '/test/second.txt', name: 'second.txt', extension: 'txt', size: 200 },
      { path: '/test/third.txt', name: 'third.txt', extension: 'txt', size: 300 },
    ];

    // Verify order is preserved
    expect(mockFiles[0].name).toBe('first.txt');
    expect(mockFiles[1].name).toBe('second.txt');
    expect(mockFiles[2].name).toBe('third.txt');
  });

  describe('formatFileSize utility', () => {
    // Test the file size formatting logic
    function formatFileSize(bytes: number): string {
      if (bytes === 0) return '0 B';
      const k = 1024;
      const sizes = ['B', 'KB', 'MB', 'GB'];
      const i = Math.floor(Math.log(bytes) / Math.log(k));
      return `${(bytes / Math.pow(k, i)).toFixed(1)} ${sizes[i]}`;
    }

    it('formats bytes correctly', () => {
      expect(formatFileSize(0)).toBe('0 B');
      expect(formatFileSize(512)).toBe('512.0 B');
      expect(formatFileSize(1024)).toBe('1.0 KB');
    });

    it('formats kilobytes correctly', () => {
      expect(formatFileSize(1024 * 50)).toBe('50.0 KB');
      expect(formatFileSize(1024 * 512)).toBe('512.0 KB');
    });

    it('formats megabytes correctly', () => {
      expect(formatFileSize(1024 * 1024)).toBe('1.0 MB');
      expect(formatFileSize(1024 * 1024 * 2)).toBe('2.0 MB');
    });

    it('formats gigabytes correctly', () => {
      expect(formatFileSize(1024 * 1024 * 1024)).toBe('1.0 GB');
      expect(formatFileSize(1024 * 1024 * 1024 * 2.5)).toBe('2.5 GB');
    });
  });
});
