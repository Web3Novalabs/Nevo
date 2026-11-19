import {
  ArgumentsHost,
  BadRequestException,
  HttpException,
  HttpStatus,
  Logger,
} from '@nestjs/common';
import { GlobalExceptionFilter } from './global-exception.filter';

describe('GlobalExceptionFilter', () => {
  let filter: GlobalExceptionFilter;
  let json: jest.Mock;
  let status: jest.Mock;
  let host: ArgumentsHost;

  beforeEach(() => {
    jest.spyOn(Logger.prototype, 'error').mockImplementation(() => undefined);
    filter = new GlobalExceptionFilter();
    json = jest.fn();
    status = jest.fn().mockReturnValue({ json });
    host = {
      switchToHttp: () => ({
        getResponse: () => ({ status }),
        getRequest: () => ({ url: '/test' }),
      }),
    } as unknown as ArgumentsHost;
  });

  afterEach(() => jest.restoreAllMocks());

  const body = () => json.mock.calls[0][0];

  it('handles an HttpException with a string response', () => {
    filter.catch(new HttpException('Forbidden!', HttpStatus.FORBIDDEN), host);

    expect(status).toHaveBeenCalledWith(HttpStatus.FORBIDDEN);
    expect(body()).toMatchObject({
      statusCode: HttpStatus.FORBIDDEN,
      message: 'Forbidden!',
      error: 'HttpException',
      path: '/test',
    });
    expect(typeof body().timestamp).toBe('string');
  });

  it('handles an HttpException with an object response', () => {
    filter.catch(
      new BadRequestException([
        'name must be a string',
        'age must be a number',
      ]),
      host,
    );

    expect(status).toHaveBeenCalledWith(HttpStatus.BAD_REQUEST);
    expect(body()).toMatchObject({
      statusCode: HttpStatus.BAD_REQUEST,
      message: ['name must be a string', 'age must be a number'],
      error: 'Bad Request',
      path: '/test',
    });
    expect(typeof body().timestamp).toBe('string');
  });

  it('handles a plain Error as a 500', () => {
    filter.catch(new TypeError('boom'), host);

    expect(status).toHaveBeenCalledWith(HttpStatus.INTERNAL_SERVER_ERROR);
    expect(body()).toMatchObject({
      statusCode: HttpStatus.INTERNAL_SERVER_ERROR,
      message: 'boom',
      error: 'TypeError',
      path: '/test',
    });
    expect(typeof body().timestamp).toBe('string');
  });

  it('handles a non-Error thrown value as a 500', () => {
    filter.catch('something odd', host);

    expect(status).toHaveBeenCalledWith(HttpStatus.INTERNAL_SERVER_ERROR);
    expect(body()).toMatchObject({
      statusCode: HttpStatus.INTERNAL_SERVER_ERROR,
      message: 'Internal server error',
      error: 'Internal Server Error',
      path: '/test',
    });
    expect(typeof body().timestamp).toBe('string');
  });
});
