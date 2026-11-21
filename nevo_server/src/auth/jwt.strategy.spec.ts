import { UnauthorizedException } from '@nestjs/common';
import { JwtStrategy } from './jwt.strategy';
import { UsersService } from '../users/users.service';

describe('JwtStrategy', () => {
  let strategy: JwtStrategy;
  let usersService: { findByPublicKey: jest.Mock };

  beforeEach(() => {
    process.env.JWT_SECRET = 'test-secret';
    usersService = { findByPublicKey: jest.fn() };
    strategy = new JwtStrategy(usersService as unknown as UsersService);
  });

  it('returns sub and publicKey when both are present', async () => {
    usersService.findByPublicKey.mockResolvedValue({
      id: 'GSUB',
      tokenVersion: 0,
    });

    await expect(
      strategy.validate({ sub: 'GSUB', publicKey: 'GPUBKEY', tokenVersion: 0 }),
    ).resolves.toEqual({
      sub: 'GSUB',
      publicKey: 'GPUBKEY',
    });
  });

  it('falls back to sub when publicKey is missing', async () => {
    usersService.findByPublicKey.mockResolvedValue({
      id: 'GSUB',
      tokenVersion: 0,
    });

    await expect(
      strategy.validate({ sub: 'GSUB', tokenVersion: 0 }),
    ).resolves.toEqual({
      sub: 'GSUB',
      publicKey: 'GSUB',
    });
  });

  it('rejects when the user is missing', async () => {
    usersService.findByPublicKey.mockResolvedValue(null);

    await expect(
      strategy.validate({ sub: 'GSUB', publicKey: 'GPUBKEY', tokenVersion: 0 }),
    ).rejects.toBeInstanceOf(UnauthorizedException);
  });

  it('rejects when the token version is stale', async () => {
    usersService.findByPublicKey.mockResolvedValue({
      id: 'GSUB',
      tokenVersion: 1,
    });

    await expect(
      strategy.validate({ sub: 'GSUB', publicKey: 'GPUBKEY', tokenVersion: 0 }),
    ).rejects.toBeInstanceOf(UnauthorizedException);
  });

  it('accepts when the token version matches', async () => {
    usersService.findByPublicKey.mockResolvedValue({
      id: 'GSUB',
      tokenVersion: 2,
    });

    await expect(
      strategy.validate({ sub: 'GSUB', publicKey: 'GPUBKEY', tokenVersion: 2 }),
    ).resolves.toEqual({
      sub: 'GSUB',
      publicKey: 'GPUBKEY',
    });
  });
});
