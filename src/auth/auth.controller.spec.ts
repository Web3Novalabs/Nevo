import { Test, TestingModule } from '@nestjs/testing';
import { BadRequestException } from '@nestjs/common';
import { AuthController } from './auth.controller';
import { AuthService } from './auth.service';
import { NonceService } from './nonce.service';

describe('AuthController', () => {
  let controller: AuthController;
  let nonceService: NonceService;

  beforeEach(async () => {
    const module: TestingModule = await Test.createTestingModule({
      controllers: [AuthController],
      providers: [
        AuthService,
        {
          provide: NonceService,
          useValue: {
            generateNonce: jest.fn().mockResolvedValue('test-nonce'),
          },
        },
      ],
    }).compile();

    controller = module.get<AuthController>(AuthController);
    nonceService = module.get<NonceService>(NonceService);
  });

  it('should be defined', () => {
    expect(controller).toBeDefined();
  });

  describe('challenge', () => {
    it('should reject a malformed Stellar public key with 400', async () => {
      await expect(
        controller.challenge('not-a-real-key'),
      ).rejects.toBeInstanceOf(BadRequestException);
      expect(nonceService.generateNonce).not.toHaveBeenCalled();
    });

    it('should generate a nonce for a valid Stellar public key', async () => {
      const validPublicKey =
        'GAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAWHF';
      const result = await controller.challenge(validPublicKey);
      expect(result).toEqual({ nonce: 'test-nonce' });
      expect(nonceService.generateNonce).toHaveBeenCalledWith(validPublicKey);
    });
  });
});
