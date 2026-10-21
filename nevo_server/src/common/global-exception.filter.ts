import {
  ArgumentsHost,
  Catch,
  ExceptionFilter,
  HttpException,
  HttpStatus,
  Logger,
} from '@nestjs/common';
import type { Request, Response } from 'express';
import { getRequestId } from './request-context';

@Catch()
export class GlobalExceptionFilter implements ExceptionFilter {
  private readonly logger = new Logger(GlobalExceptionFilter.name);

  catch(exception: unknown, host: ArgumentsHost): void {
    const ctx = host.switchToHttp();
    const response = ctx.getResponse<Response>();
    const request = ctx.getRequest<Request>();

    // Fallback to header or 'unknown' if getRequestId() returns undefined
    const requestId =
      getRequestId() ??
      (request.headers['x-request-id'] as string) ??
      'unknown';
    const requestId = getRequestId() || (request.headers['x-request-id'] as string) || 'unknown';

    let statusCode = HttpStatus.INTERNAL_SERVER_ERROR;
    let message: string | string[] = 'An internal error occurred';
    let error = 'Internal Server Error';

    if (exception instanceof HttpException) {
      statusCode = exception.getStatus();
      const responseBody = exception.getResponse();
      if (typeof responseBody === 'string') {
        message = responseBody;
        error = exception.name;
      } else if (typeof responseBody === 'object' && responseBody !== null) {
        const resObj = responseBody as Record<string, any>;
        message = resObj.message ?? exception.message;
        error = resObj.error ?? exception.name;
      }
    } else if (exception instanceof Error) {
      // Log full error details server-side for debugging
      this.logger.error(
        `[${requestId}] Unhandled exception: ${exception.message}`,
        exception.stack,
      );
      // Mask sensitive error details from client response
      message = 'An internal error occurred';
      error = 'Internal Server Error';
    } else {
      this.logger.error(`[${requestId}] Unhandled exception: ${String(exception)}`);
      message = 'An internal error occurred';
      error = 'Internal Server Error';
    }

    response.status(statusCode).json({
      statusCode,
      message,
      error,
      timestamp: new Date().toISOString(),
      path: request.url,
      requestId,
    });
  }
}
