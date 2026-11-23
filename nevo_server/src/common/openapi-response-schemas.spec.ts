import { Test } from '@nestjs/testing';
import { DocumentBuilder, SwaggerModule } from '@nestjs/swagger';
import type { OpenAPIObject } from '@nestjs/swagger';
import { AuthController } from '../auth/auth.controller';
import { AuthService } from '../auth/auth.service';
import { JwtAuthGuard } from '../auth/jwt-auth.guard';
import { NonceService } from '../auth/nonce.service';
import { ContractService } from '../contract/contract.service';
import { DonationsController } from '../donations/donations.controller';
import { DonationsService } from '../donations/donations.service';
import { PoolsController } from '../pools/pools.controller';
import { PoolsService } from '../pools/pools.service';

jest.mock('../contract/contract.service', () => ({
  ContractService: class ContractService {},
}));

type Schema = { properties?: Record<string, unknown> };

describe('OpenAPI response schemas', () => {
  let document: OpenAPIObject;

  beforeAll(async () => {
    const moduleRef = await Test.createTestingModule({
      controllers: [AuthController, DonationsController, PoolsController],
      providers: [
        { provide: AuthService, useValue: {} },
        { provide: NonceService, useValue: {} },
        { provide: DonationsService, useValue: {} },
        { provide: PoolsService, useValue: {} },
        { provide: ContractService, useValue: {} },
      ],
    })
      .overrideGuard(JwtAuthGuard)
      .useValue({ canActivate: () => true })
      .compile();
    const app = moduleRef.createNestApplication();
    document = SwaggerModule.createDocument(
      app,
      new DocumentBuilder().build(),
    );
    await app.close();
  });

  it.each([
    'PoolResponseDto',
    'PoolDetailResponseDto',
    'PaginatedPoolsDto',
    'DonationResponseDto',
    'PaginatedDonationsDto',
    'AuthResultDto',
  ])('contains the %s schema', (name) => {
    const schema = document.components?.schemas?.[name] as Schema;
    expect(Object.keys(schema?.properties ?? {}).length).toBeGreaterThan(0);
  });

  it('documents the on-chain fields of a pool detail', () => {
    const schema = document.components?.schemas?.[
      'PoolDetailResponseDto'
    ] as Schema;
    expect(Object.keys(schema.properties ?? {})).toEqual(
      expect.arrayContaining(['title', 'raisedOnChain', 'closedOnChain']),
    );
  });

  it('references a named schema from every pools, donations and auth success response', () => {
    const operations = Object.entries(document.paths)
      .filter(([path]) => /^\/(pools|auth|users\/me\/donations)/.test(path))
      .flatMap(([path, item]) =>
        ['get', 'post', 'patch'].flatMap((method) => {
          const op = (item as Record<string, any>)[method];
          return op ? [{ id: `${method} ${path}`, op }] : [];
        }),
      );

    expect(operations.length).toBeGreaterThanOrEqual(10);
    for (const { id, op } of operations) {
      const success = op.responses['200'] ?? op.responses['201'];
      const ref = success?.content?.['application/json']?.schema?.$ref;
      expect([id, ref]).toEqual([id, expect.stringContaining('Dto')]);
    }
  });
});
