import type { Request, Response } from 'express';
import { LoggingMiddleware } from './logging.middleware';
import { getRequestId } from './request-context';

describe('LoggingMiddleware', () => {
  let middleware: LoggingMiddleware;
  let res: Response;

  beforeEach(() => {
    middleware = new LoggingMiddleware();
    res = {
      setHeader: jest.fn(),
      on: jest.fn(),
    } as unknown as Response;
  });

  const makeReq = (headers: Record<string, string> = {}) =>
    ({ headers, method: 'GET', url: '/test' }) as unknown as Request;

  it('generates an x-request-id when none is provided', () => {
    let contextId: string | undefined;
    const next = jest.fn(() => {
      contextId = getRequestId();
    });

    middleware.use(makeReq(), res, next);

    const [header, generated] = (res.setHeader as jest.Mock).mock.calls[0];
    expect(header).toBe('X-Request-Id');
    expect(typeof generated).toBe('string');
    expect(generated).not.toHaveLength(0);
    expect(contextId).toBe(generated);
  });

  it('reuses an existing x-request-id header', () => {
    let contextId: string | undefined;
    const next = jest.fn(() => {
      contextId = getRequestId();
    });

    middleware.use(makeReq({ 'x-request-id': 'existing-id' }), res, next);

    expect(res.setHeader).toHaveBeenCalledWith('X-Request-Id', 'existing-id');
    expect(contextId).toBe('existing-id');
  });

  it('calls next() exactly once', () => {
    const next = jest.fn();

    middleware.use(makeReq(), res, next);

    expect(next).toHaveBeenCalledTimes(1);
  });
});
