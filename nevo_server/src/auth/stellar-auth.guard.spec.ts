import { ExecutionContext, UnauthorizedException } from '@nestjs/common';
import { Reflector } from '@nestjs/core';
import { AuthGuard } from '@nestjs/passport';
import { JwtAuthGuard } from './jwt-auth.guard';
import { StellarAuthGuard } from './stellar-auth.guard';

/**
 * Tests for StellarAuthGuard as a JwtAuthGuard alias.
 *
 * Issue #1493: StellarAuthGuard must be behaviorally identical to JwtAuthGuard.
 * These tests document the alias relationship so any future divergence fails the suite.
 */
describe('StellarAuthGuard (alias of JwtAuthGuard)', () => {
  const createMockContext = (): ExecutionContext =>
    ({
      getHandler: jest.fn(),
      getClass: jest.fn(),
      switchToHttp: jest.fn().mockReturnValue({
        getRequest: jest.fn().mockReturnValue({ headers: {} }),
        getResponse: jest.fn().mockReturnValue({}),
      }),
    }) as unknown as ExecutionContext;

  describe('alias relationship', () => {
    it('should extend JwtAuthGuard', () => {
      expect(StellarAuthGuard.prototype).toBeInstanceOf(JwtAuthGuard);
    });

    it('should be a subclass of JwtAuthGuard', () => {
      expect(Object.getPrototypeOf(StellarAuthGuard.prototype)).toBe(
        JwtAuthGuard.prototype,
      );
    });

    it('should be an instance of AuthGuard("jwt") via inheritance', () => {
      const reflector = new Reflector();
      const guard = new StellarAuthGuard(reflector);
      expect(guard).toBeInstanceOf(JwtAuthGuard);
      expect(guard).toBeInstanceOf(AuthGuard('jwt'));
    });
  });

  describe('behavioral parity with JwtAuthGuard', () => {
    let reflector: Reflector;

    beforeEach(() => {
      reflector = new Reflector();
    });

    it('should reject unauthenticated requests identically (both throw UnauthorizedException)', async () => {
      const context = createMockContext();

      const jwtGuard = new JwtAuthGuard(reflector);
      const stellarGuard = new StellarAuthGuard(reflector);

      // Both guards should reject unauthenticated requests the same way.
      // passport's AuthGuard throws UnauthorizedException when no user is attached.
      const jwtResult = await jwtGuard
        .canActivate(context)
        .catch((err) => err);
      const stellarResult = await stellarGuard
        .canActivate(context)
        .catch((err) => err);

      expect(jwtResult).toBeInstanceOf(UnauthorizedException);
      expect(stellarResult).toBeInstanceOf(UnauthorizedException);

      // Same outcome type and message.
      expect(stellarResult.constructor).toBe(jwtResult.constructor);
      expect((stellarResult as UnauthorizedException).message).toBe(
        (jwtResult as UnauthorizedException).message,
      );
    });

    it('should produce the same canActivate outcome for the same inputs', async () => {
      const context = createMockContext();

      const jwtGuard = new JwtAuthGuard(reflector);
      const stellarGuard = new StellarAuthGuard(reflector);

      const jwtOutcome = await jwtGuard
        .canActivate(context)
        .then(
          (value) => ({ resolved: true, value }),
          (error) => ({ resolved: false, error }),
        );
      const stellarOutcome = await stellarGuard
        .canActivate(context)
        .then(
          (value) => ({ resolved: true, value }),
          (error) => ({ resolved: false, error }),
        );

      expect(stellarOutcome.resolved).toBe(jwtOutcome.resolved);
      if (!jwtOutcome.resolved && !stellarOutcome.resolved) {
        expect(stellarOutcome.error.constructor).toBe(
          jwtOutcome.error.constructor,
        );
      }
    });

    it('should delegate to the same "jwt" AuthGuard strategy', () => {
      const jwtGuard = new JwtAuthGuard(reflector);
      const stellarGuard = new StellarAuthGuard(reflector);

      // Both guards should resolve the same passport strategy name.
      const jwtStrategy = (jwtGuard as unknown as { options: { defaultStrategy?: string } })
        .options?.defaultStrategy;
      const stellarStrategy = (
        stellarGuard as unknown as { options: { defaultStrategy?: string } }
      ).options?.defaultStrategy;

      expect(stellarStrategy).toBe(jwtStrategy);
      expect(stellarStrategy).toBe('jwt');
    });

    it('should share the same canActivate implementation reference', () => {
      // If StellarAuthGuard ever overrides canActivate, this will fail,
      // signaling a potential behavioral divergence.
      expect(StellarAuthGuard.prototype.canActivate).toBe(
        JwtAuthGuard.prototype.canActivate,
      );
    });
  });
});