import { describe, test, expect } from 'vitest';
import { cn } from './cn';

describe('cn utility', () => {
  test('merges class names correctly', () => {
    expect(cn('foo', 'bar')).toBe('foo bar');
  });

  test('handles conditional classes', () => {
    expect(cn('foo', false && 'bar', 'baz')).toBe('foo baz');
  });
});
