import {
  CanActivate,
  ExecutionContext,
  ForbiddenException,
  Injectable,
} from '@nestjs/common';
import { ConfigService } from '@nestjs/config';

/**
 * Allows only wallets listed in `ADMIN_PUBLIC_KEYS` (comma-separated). Must run
 * after `JwtAuthGuard`, which populates `request.user.publicKey`. With no
 * configured admins every request is rejected.
 */
@Injectable()
export class AdminGuard implements CanActivate {
  constructor(private readonly config: ConfigService) {}

  canActivate(context: ExecutionContext): boolean {
    const request = context
      .switchToHttp()
      .getRequest<{ user?: { publicKey?: string } }>();
    const publicKey = request.user?.publicKey;
    const admins = (this.config.get<string>('ADMIN_PUBLIC_KEYS') ?? '')
      .split(',')
      .map((key) => key.trim())
      .filter(Boolean);

    if (!publicKey || !admins.includes(publicKey)) {
      throw new ForbiddenException('Admin access required');
    }
    return true;
  }
}
