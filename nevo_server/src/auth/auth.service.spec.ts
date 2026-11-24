import { Test, TestingModule } from '@nestjs/testing';
import { JwtService } from '@nestjs/jwt';
import { UnauthorizedException } from '@nestjs/common';

jest.mock('@nestjs/schedule', () => ({
  Cron: () => () => {},
  CronExpression: {},
  ScheduleModule: {},
  SchedulerRegistry: jest.fn(),
}));

import { AuthService } from './auth.service';
import { VerifyAuthDto } from './dto/verify-auth.dto';
import { UsersService } from '../users/users.service';
import { NonceService } from './nonce.service';
import { User } from '../users/user.entity';
import * as StellarSdk from '@stellar/stellar-sdk';

describe('AuthService', () => {
  let service: AuthService;
  let usersService: UsersService;
  let nonceService: NonceService;
  let jwtService: JwtService;

  const mockUser: User = {
    id: 'uuid-1',
    publicKey: 'GBM3T7V2NNWJVSQ5Q7WPEMMO5G2E2UZY4D2Z24W73SHZJ2E4A5F2D3FZ',
    displayName: null,
    tokenVersion: 0,
    createdAt: new Date(),
    updatedAt: new Date(),
  };

  beforeEach(async () => {
    const module: TestingModule = await Test.createTestingModule({
      providers: [
        AuthService,
        {
          provide: UsersService,
          useValue: {
            findOrCreate: jest.fn().mockResolvedValue(mockUser),
            findByPublicKey: jest.fn().mockResolvedValue(mockUser),
            incrementTokenVersion: jest
              .fn()
              .mockResolvedValue({ ...mockUser, tokenVersion: 1 }),
          },
        },
        {
          provide: NonceService,
          useValue: {
            generateNonce: jest.fn(),
            consumeNonce: jest.fn(),
            deleteExpiredNonces: jest.fn(),
          },
        },
        {
          provide: JwtService,
          useValue: {
            sign: jest.fn().mockReturnValue('jwt-token'),
          },
        },
      ],
    }).compile();

    service = module.get(AuthService);
    usersService = module.get(UsersService);
    nonceService = module.get(NonceService);
    jwtService = module.get(JwtService);
  });

  afterEach(() => {
    jest.clearAllMocks();
  });

  describe('dead challenge code (#1490)', () => {
    it('does not expose generateChallenge or an in-memory challenge store', () => {
      const internal = service as unknown as {
        generateChallenge?: unknown;
        challenges?: unknown;
        cleanupExpiredChallenges?: unknown;
      };

      expect(internal.generateChallenge).toBeUndefined();
      expect(internal.challenges).toBeUndefined();
      expect(internal.cleanupExpiredChallenges).toBeUndefined();
    });

    it('does not create an interval when instantiated', () => {
      const setIntervalSpy = jest
        .spyOn(global, 'setInterval')
        .mockImplementation(() => 0 as unknown as NodeJS.Timeout);

      try {
        new AuthService(jwtService, usersService, nonceService);
        expect(setIntervalSpy).not.toHaveBeenCalled();
      } finally {
        setIntervalSpy.mockRestore();
      }
    });
  });

  describe('verifySignature - Valid Stellar Signature', () => {
    it('should return true for a valid Stellar Ed25519 signature', () => {
      // Generate a test keypair
      const keypair = StellarSdk.Keypair.random();
      const message = 'test-message';
      const signature = keypair.sign(Buffer.from(message)).toString('hex');

      // Use the private method indirectly through verify
      const result = (service as any).verifySignature(
        keypair.publicKey(),
        signature,
        message,
      );

      expect(result).toBe(true);
    });

    it('should successfully verify a signature when message matches', () => {
      const keypair = StellarSdk.Keypair.random();
      const message = 'authenticate-user-request';
      const signature = keypair.sign(Buffer.from(message)).toString('hex');

      const result = (service as any).verifySignature(
        keypair.publicKey(),
        signature,
        message,
      );

      expect(result).toBe(true);
    });
  });

  describe('verifySignature - Wrong Keypair', () => {
    it('should return false when signature is from a different keypair', () => {
      const keypair1 = StellarSdk.Keypair.random();
      const keypair2 = StellarSdk.Keypair.random();
      const message = 'test-message';

      // Sign with keypair1
      const signature = keypair1.sign(Buffer.from(message)).toString('hex');

      // Verify with keypair2's public key - should fail
      const result = (service as any).verifySignature(
        keypair2.publicKey(),
        signature,
        message,
      );

      expect(result).toBe(false);
    });

    it('should return false when using a different public key than the signer', () => {
      const keypair1 = StellarSdk.Keypair.random();
      const keypair2 = StellarSdk.Keypair.random();
      const message = 'important-transaction';
      const signature = keypair1.sign(Buffer.from(message)).toString('hex');

      const result = (service as any).verifySignature(
        keypair2.publicKey(),
        signature,
        message,
      );

      expect(result).toBe(false);
    });
  });

  describe('verifySignature - Tampered Message', () => {
    it('should return false when the message has been tampered with', () => {
      const keypair = StellarSdk.Keypair.random();
      const originalMessage = 'original-message';
      const tamperedMessage = 'tampered-message';

      // Sign original message
      const signature = keypair
        .sign(Buffer.from(originalMessage))
        .toString('hex');

      // Verify with tampered message - should fail
      const result = (service as any).verifySignature(
        keypair.publicKey(),
        signature,
        tamperedMessage,
      );

      expect(result).toBe(false);
    });

    it('should return false when message content is modified', () => {
      const keypair = StellarSdk.Keypair.random();
      const message = 'sign-this-exact-message';
      const modifiedMessage = 'sign-this-exact-message-modified';

      const signature = keypair.sign(Buffer.from(message)).toString('hex');
      const result = (service as any).verifySignature(
        keypair.publicKey(),
        signature,
        modifiedMessage,
      );

      expect(result).toBe(false);
    });

    it('should return false when even a single character is changed in the message', () => {
      const keypair = StellarSdk.Keypair.random();
      const message = 'authenticate';
      const tampered = 'authenticate2'; // Added single character

      const signature = keypair.sign(Buffer.from(message)).toString('hex');
      const result = (service as any).verifySignature(
        keypair.publicKey(),
        signature,
        tampered,
      );

      expect(result).toBe(false);
    });
  });

  describe('verifySignature - Invalid Input', () => {
    it('should return false for an invalid public key format', () => {
      const invalidPublicKey = 'not-a-valid-stellar-key';
      const message = 'test-message';
      const signature = 'test-signature';

      const result = (service as any).verifySignature(
        invalidPublicKey,
        signature,
        message,
      );

      expect(result).toBe(false);
    });

    it('should return false for an invalid signature format', () => {
      const keypair = StellarSdk.Keypair.random();
      const message = 'test-message';
      const invalidSignature = 'not-a-hex-signature';

      const result = (service as any).verifySignature(
        keypair.publicKey(),
        invalidSignature,
        message,
      );

      expect(result).toBe(false);
    });
  });

  describe('verify - Expired Nonce', () => {
    it('should throw UnauthorizedException when nonce is expired', async () => {
      const keypair = StellarSdk.Keypair.random();
      const message = 'expired-nonce';
      const signature = keypair.sign(Buffer.from(message)).toString('hex');

      const dto: VerifyAuthDto = {
        publicKey: keypair.publicKey(),
        signature,
        message,
      };

      jest.spyOn(nonceService, 'consumeNonce').mockResolvedValue(false); // Nonce validation fails

      await expect(service.verify(dto)).rejects.toThrow(UnauthorizedException);
      await expect(service.verify(dto)).rejects.toThrow(
        'Invalid or expired nonce',
      );
    });

    it('should throw UnauthorizedException when nonce is not found', async () => {
      const keypair = StellarSdk.Keypair.random();
      const message = 'nonexistent-nonce';
      const signature = keypair.sign(Buffer.from(message)).toString('hex');

      const dto: VerifyAuthDto = {
        publicKey: keypair.publicKey(),
        signature,
        message,
      };

      jest.spyOn(nonceService, 'consumeNonce').mockResolvedValue(false);

      await expect(service.verify(dto)).rejects.toThrow(UnauthorizedException);
    });
  });

  describe('verify - Nonce Lifecycle', () => {
    it('should mark nonce as used after successful verification', async () => {
      const keypair = StellarSdk.Keypair.random();
      const nonce = 'fresh-nonce';
      const signature = keypair.sign(Buffer.from(nonce)).toString('hex');

      const dto: VerifyAuthDto = {
        publicKey: keypair.publicKey(),
        signature,
        message: nonce,
      };

      jest.spyOn(nonceService, 'consumeNonce').mockResolvedValue(true);
      jest.spyOn(usersService, 'findOrCreate').mockResolvedValue({
        ...mockUser,
        publicKey: keypair.publicKey(),
      });
      jest.spyOn(jwtService, 'sign').mockReturnValue('jwt-token');

      await service.verify(dto);
      expect(nonceService.consumeNonce).toHaveBeenCalledWith(
        nonce,
        dto.publicKey,
      );
    });

    it('should fail on second verify call with same nonce (nonce already used)', async () => {
      const keypair = StellarSdk.Keypair.random();
      const nonce = 'used-nonce';
      const signature = keypair.sign(Buffer.from(nonce)).toString('hex');

      const dto: VerifyAuthDto = {
        publicKey: keypair.publicKey(),
        signature,
        message: nonce,
      };

      // First call - nonce is fresh
      jest.spyOn(nonceService, 'consumeNonce').mockResolvedValueOnce(true);
      jest.spyOn(usersService, 'findOrCreate').mockResolvedValue({
        ...mockUser,
        publicKey: keypair.publicKey(),
      });
      jest.spyOn(jwtService, 'sign').mockReturnValue('jwt-token');

      // First verify - should succeed
      await service.verify(dto);

      // Second call - nonce is now used/invalid
      jest.spyOn(nonceService, 'consumeNonce').mockResolvedValueOnce(false);

      // Second verify - should fail
      await expect(service.verify(dto)).rejects.toThrow(UnauthorizedException);
      await expect(service.verify(dto)).rejects.toThrow(
        'Invalid or expired nonce',
      );
    });

    it('should throw when nonce consumption returns false (nonce used or expired)', async () => {
      const keypair = StellarSdk.Keypair.random();
      const nonce = 'reused-nonce';
      const signature = keypair.sign(Buffer.from(nonce)).toString('hex');

      const dto: VerifyAuthDto = {
        publicKey: keypair.publicKey(),
        signature,
        message: nonce,
      };

      jest.spyOn(nonceService, 'consumeNonce').mockResolvedValue(false); // Nonce already used or expired

      await expect(service.verify(dto)).rejects.toThrow(
        new UnauthorizedException('Invalid or expired nonce'),
      );
    });
  });

  describe('verify - Invalid Signature', () => {
    it('should throw UnauthorizedException when signature is invalid', async () => {
      const keypair = StellarSdk.Keypair.random();
      const message = 'test-message';
      const invalidSignature = 'invalid-signature-hex';

      const dto: VerifyAuthDto = {
        publicKey: keypair.publicKey(),
        signature: invalidSignature,
        message,
      };

      // Should throw due to invalid signature before checking nonce
      await expect(service.verify(dto)).rejects.toThrow(UnauthorizedException);
      await expect(service.verify(dto)).rejects.toThrow('Invalid signature');
    });

    it('should throw when signature is from wrong keypair', async () => {
      const keypair1 = StellarSdk.Keypair.random();
      const keypair2 = StellarSdk.Keypair.random();
      const message = 'test-message';
      const signature = keypair1.sign(Buffer.from(message)).toString('hex');

      const dto: VerifyAuthDto = {
        publicKey: keypair2.publicKey(),
        signature,
        message,
      };

      // Should throw due to signature verification failure
      await expect(service.verify(dto)).rejects.toThrow(UnauthorizedException);
    });
  });

  describe('verify - Complete Happy Path', () => {
    it('should return accessToken and user on valid signature and nonce', async () => {
      const keypair = StellarSdk.Keypair.random();
      const nonce = 'valid-fresh-nonce';
      const signature = keypair.sign(Buffer.from(nonce)).toString('hex');

      const dto: VerifyAuthDto = {
        publicKey: keypair.publicKey(),
        signature,
        message: nonce,
      };

      const user: User = {
        id: 'user-uuid-1',
        publicKey: keypair.publicKey(),
        displayName: null,
        tokenVersion: 0,
        createdAt: new Date(),
        updatedAt: new Date(),
      };

      jest.spyOn(nonceService, 'consumeNonce').mockResolvedValue(true);
      jest.spyOn(usersService, 'findOrCreate').mockResolvedValue(user);
      jest.spyOn(jwtService, 'sign').mockReturnValue('jwt-token');

      const result = await service.verify(dto);

      expect(result).toEqual({
        accessToken: 'jwt-token',
        user,
      });
      expect(nonceService.consumeNonce).toHaveBeenCalledWith(
        nonce,
        dto.publicKey,
      );
      expect(usersService.findOrCreate).toHaveBeenCalledWith(
        keypair.publicKey(),
      );
      expect(jwtService.sign).toHaveBeenCalledWith({
        sub: user.id,
        publicKey: user.publicKey,
        tokenVersion: user.tokenVersion,
      });
    });

    it('should create a new user if they do not exist during verification', async () => {
      const keypair = StellarSdk.Keypair.random();
      const nonce = 'new-user-nonce';
      const signature = keypair.sign(Buffer.from(nonce)).toString('hex');

      const dto: VerifyAuthDto = {
        publicKey: keypair.publicKey(),
        signature,
        message: nonce,
      };

      const newUser: User = {
        id: 'user-uuid-new',
        publicKey: keypair.publicKey(),
        displayName: null,
        tokenVersion: 0,
        createdAt: new Date(),
        updatedAt: new Date(),
      };

      jest.spyOn(nonceService, 'consumeNonce').mockResolvedValue(true);
      jest.spyOn(usersService, 'findOrCreate').mockResolvedValue(newUser);
      jest.spyOn(jwtService, 'sign').mockReturnValue('jwt-token');

      const result = await service.verify(dto);

      expect(usersService.findOrCreate).toHaveBeenCalledWith(
        keypair.publicKey(),
      );
      expect(result.user).toEqual(newUser);
    });
  });

  describe('logout - JWT revocation', () => {
    it('increments the token version of the user with that public key', async () => {
      const incrementSpy = jest
        .spyOn(usersService, 'incrementTokenVersion')
        .mockResolvedValue({ ...mockUser, tokenVersion: 1 });

      await service.logout(mockUser.publicKey);

      expect(usersService.findByPublicKey).toHaveBeenCalledWith(
        mockUser.publicKey,
      );
      expect(incrementSpy).toHaveBeenCalledWith(mockUser.publicKey);
    });

    it('throws UnauthorizedException when the user does not exist', async () => {
      jest.spyOn(usersService, 'findByPublicKey').mockResolvedValue(null);

      await expect(service.logout('GMISSING')).rejects.toThrow(
        UnauthorizedException,
      );
      expect(usersService.incrementTokenVersion).not.toHaveBeenCalled();
    });
  });

  describe('removed in-memory challenge flow (Issue #1490)', () => {
    it('no longer exposes generateChallenge or a challenges map', () => {
      const untyped = service as unknown as Record<string, unknown>;
      expect(untyped.generateChallenge).toBeUndefined();
      expect(untyped.challenges).toBeUndefined();
    });

    it('verify() rejects when NonceService has no record of the nonce', async () => {
      const keypair = StellarSdk.Keypair.random();
      const message = 'never-issued-nonce';
      const dto: VerifyAuthDto = {
        publicKey: keypair.publicKey(),
        signature: keypair.sign(Buffer.from(message)).toString('hex'),
        message,
      };
      jest.spyOn(nonceService, 'consumeNonce').mockResolvedValue(false);

      await expect(service.verify(dto)).rejects.toThrow(
        'Invalid or expired nonce',
      );
    });
  });
});
