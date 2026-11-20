import { ExecutionContext, ForbiddenException } from '@nestjs/common';
import { ConfigService } from '@nestjs/config';
import { AdminGuard } from './admin.guard';
import { ReconciliationController } from './reconciliation.controller';
import { ReconciliationService } from './reconciliation.service';

const ctx = (user?: { publicKey?: string }) =>
  ({
    switchToHttp: () => ({ getRequest: () => ({ user }) }),
  }) as unknown as ExecutionContext;

const guardWith = (admins?: string) =>
  new AdminGuard({ get: () => admins } as unknown as ConfigService);

describe('AdminGuard', () => {
  it('allows a listed admin', () => {
    expect(
      guardWith('GADMIN, GOTHER').canActivate(ctx({ publicKey: 'GOTHER' })),
    ).toBe(true);
  });

  it('rejects an authenticated non-admin', () => {
    expect(() =>
      guardWith('GADMIN').canActivate(ctx({ publicKey: 'GUSER' })),
    ).toThrow(ForbiddenException);
  });

  it('rejects everyone when no admins are configured', () => {
    expect(() =>
      guardWith(undefined).canActivate(ctx({ publicKey: 'GADMIN' })),
    ).toThrow(ForbiddenException);
    expect(() => guardWith('').canActivate(ctx({ publicKey: '' }))).toThrow(
      ForbiddenException,
    );
  });

  it('rejects requests with no user', () => {
    expect(() => guardWith('GADMIN').canActivate(ctx())).toThrow(
      ForbiddenException,
    );
  });
});

describe('ReconciliationController', () => {
  it('manual trigger runs reconciliation with the manual trigger', async () => {
    const summary = { status: 'completed' };
    const service = {
      reconcile: jest.fn().mockResolvedValue(summary),
      getStats: jest.fn().mockReturnValue({ runs: 3 }),
    };
    const controller = new ReconciliationController(
      service as unknown as ReconciliationService,
    );

    await expect(controller.trigger()).resolves.toBe(summary);
    expect(service.reconcile).toHaveBeenCalledWith('manual');
    expect(controller.status()).toEqual({ runs: 3 });
  });

  it('is protected by JwtAuthGuard then AdminGuard', () => {
    const guards = Reflect.getMetadata(
      '__guards__',
      ReconciliationController,
    ) as unknown[];
    expect(guards.map((g) => (g as { name: string }).name)).toEqual([
      'JwtAuthGuard',
      'AdminGuard',
    ]);
  });
});
