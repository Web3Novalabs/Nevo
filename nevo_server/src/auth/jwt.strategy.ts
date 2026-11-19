import { Injectable, UnauthorizedException } from '@nestjs/common';
import { PassportStrategy } from '@nestjs/passport';
import { ExtractJwt, Strategy } from 'passport-jwt';
import { getJwtSecret } from './jwt.config.js';
import { UsersService } from '../users/users.service.js';

@Injectable()
export class JwtStrategy extends PassportStrategy(Strategy) {
  constructor(private readonly usersService: UsersService) {
    super({
      jwtFromRequest: ExtractJwt.fromAuthHeaderAsBearerToken(),
      ignoreExpiration: false,
      secretOrKey: getJwtSecret(),
    });
  }

  async validate(payload: {
    sub?: string;
    publicKey?: string;
    tokenVersion?: number;
  }) {
    const publicKey = payload.publicKey ?? payload.sub;

    const user = await this.usersService.findByPublicKey(publicKey);

    if (!user) {
      throw new UnauthorizedException();
    }

    if (payload.tokenVersion !== user.tokenVersion) {
      throw new UnauthorizedException();
    }

    return {
      sub: payload.sub,
      publicKey,
    };
  }
}