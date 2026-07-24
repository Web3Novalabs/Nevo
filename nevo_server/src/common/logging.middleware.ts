import { Injectable, Logger, NestMiddleware } from '@nestjs/common';
import { NextFunction, Request, Response } from 'express';
import { randomUUID } from 'node:crypto';
import { requestContext } from './request-context';


@Injectable()
export class LoggingMiddleware implements NestMiddleware {
  private readonly logger = new Logger('HTTP');

  use(req: Request, res: Response, next: NextFunction): void {
    const start = Date.now();

    const requestId = (req.headers['x-request-id'] as string) || randomUUID();

    res.setHeader('X-Request_ID', requestId);

    res.on('finish', () => {
      const duration = Date.now() - start;
      this.logger.log(
        `[${requestId}] ${req.method} ${req.originalUrl || req.url} ${res.statusCode} ${duration}ms`,
      );
    });
    requestContext.run({ requestId }, () => {
      next();
    });

  }
}
