import {
  CanActivate,
  ExecutionContext,
  Injectable,
  UnauthorizedException,
} from '@nestjs/common';
import { JwtService } from '@nestjs/jwt';
import { Request } from 'express';

@Injectable()
export class StellarAuthGuard implements CanActivate {
  private readonly jwtService = new JwtService();

  canActivate(context: ExecutionContext): boolean {
    const req = context.switchToHttp().getRequest<Request>();
    const auth = req.headers['authorization'];
    if (!auth?.startsWith('Bearer ')) throw new UnauthorizedException();
    const token = auth.slice(7);
    try {
      const payload = this.jwtService.verify<{
        sub: string;
        publicKey: string;
      }>(token, {
        secret: process.env.JWT_SECRET ?? 'dev-secret',
      });
      (req as Request & { user: unknown }).user = payload;
      return true;
    } catch {
      throw new UnauthorizedException();
    }
  }
}
