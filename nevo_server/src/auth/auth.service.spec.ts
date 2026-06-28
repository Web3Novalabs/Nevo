import { Test, TestingModule } from '@nestjs/testing';
import { JwtService } from '@nestjs/jwt';
import { UnauthorizedException } from '@nestjs/common';
import { AuthService, VerifyDto } from './auth.service';
import { UsersService } from '../users/users.service';
import { NonceService } from './nonce.service';
import { User } from '../users/user.entity';
import { Nonce } from './nonce.entity';
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
    createdAt: new Date(),
    updatedAt: new Date(),
  };

  const mockNonce: Nonce = {
    id: 'nonce-uuid-1',
    nonce: 'test-nonce-12345',
    publicKey: 'GABC123',
    createdAt: new Date(),
    expiresAt: new Date(Date.now() + 5 * 60 * 1000), // 5 minutes from now
    used: false,
  };

  beforeEach(async () => {
    const module: TestingModule = await Test.createTestingModule({
      providers: [
        AuthService,
        {
          provide: UsersService,
          useValue: {
            findOrCreate: jest.fn().mockResolvedValue(mockUser),
          },
        },
        {
          provide: NonceService,
          useValue: {
            generateNonce: jest.fn(),
            findAndValidateNonce: jest.fn(),
            markNonceAsUsed: jest.fn(),
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

      const expiredNonce: Nonce = {
        id: 'nonce-uuid-2',
        nonce: message,
        publicKey: keypair.publicKey(),
        createdAt: new Date(Date.now() - 10 * 60 * 1000), // 10 minutes ago
        expiresAt: new Date(Date.now() - 5 * 60 * 1000), // Expired 5 minutes ago
        used: false,
      };

      const dto: VerifyDto = {
        publicKey: keypair.publicKey(),
        signature,
        message,
      };

      jest.spyOn(nonceService, 'findAndValidateNonce').mockResolvedValue(null); // Nonce validation fails

      await expect(service.verify(dto)).rejects.toThrow(UnauthorizedException);
      await expect(service.verify(dto)).rejects.toThrow(
        'Invalid or expired nonce',
      );
    });

    it('should throw UnauthorizedException when nonce is not found', async () => {
      const keypair = StellarSdk.Keypair.random();
      const message = 'nonexistent-nonce';
      const signature = keypair.sign(Buffer.from(message)).toString('hex');

      const dto: VerifyDto = {
        publicKey: keypair.publicKey(),
        signature,
        message,
      };

      jest.spyOn(nonceService, 'findAndValidateNonce').mockResolvedValue(null);

      await expect(service.verify(dto)).rejects.toThrow(UnauthorizedException);
    });
  });

  describe('verify - Nonce Lifecycle', () => {
    it('should mark nonce as used after successful verification', async () => {
      const keypair = StellarSdk.Keypair.random();
      const nonce = 'fresh-nonce';
      const signature = keypair.sign(Buffer.from(nonce)).toString('hex');

      const validNonce: Nonce = {
        id: 'nonce-uuid-3',
        nonce,
        publicKey: keypair.publicKey(),
        createdAt: new Date(),
        expiresAt: new Date(Date.now() + 5 * 60 * 1000),
        used: false,
      };

      const dto: VerifyDto = {
        publicKey: keypair.publicKey(),
        signature,
        message: nonce,
      };

      jest
        .spyOn(nonceService, 'findAndValidateNonce')
        .mockResolvedValue(validNonce);
      jest.spyOn(nonceService, 'markNonceAsUsed').mockResolvedValue();
      jest.spyOn(usersService, 'findOrCreate').mockResolvedValue({
        ...mockUser,
        publicKey: keypair.publicKey(),
      });
      jest.spyOn(jwtService, 'sign').mockReturnValue('jwt-token');

      await service.verify(dto);

      expect(nonceService.markNonceAsUsed).toHaveBeenCalledWith(validNonce.id);
    });

    it('should fail on second verify call with same nonce (nonce already used)', async () => {
      const keypair = StellarSdk.Keypair.random();
      const nonce = 'used-nonce';
      const signature = keypair.sign(Buffer.from(nonce)).toString('hex');

      const dto: VerifyDto = {
        publicKey: keypair.publicKey(),
        signature,
        message: nonce,
      };

      // First call - nonce is fresh
      const validNonce: Nonce = {
        id: 'nonce-uuid-4',
        nonce,
        publicKey: keypair.publicKey(),
        createdAt: new Date(),
        expiresAt: new Date(Date.now() + 5 * 60 * 1000),
        used: false,
      };

      jest
        .spyOn(nonceService, 'findAndValidateNonce')
        .mockResolvedValueOnce(validNonce);
      jest.spyOn(nonceService, 'markNonceAsUsed').mockResolvedValue();
      jest.spyOn(usersService, 'findOrCreate').mockResolvedValue({
        ...mockUser,
        publicKey: keypair.publicKey(),
      });
      jest.spyOn(jwtService, 'sign').mockReturnValue('jwt-token');

      // First verify - should succeed
      await service.verify(dto);

      // Second call - nonce is now used/invalid
      jest
        .spyOn(nonceService, 'findAndValidateNonce')
        .mockResolvedValueOnce(null);

      // Second verify - should fail
      await expect(service.verify(dto)).rejects.toThrow(UnauthorizedException);
      await expect(service.verify(dto)).rejects.toThrow(
        'Invalid or expired nonce',
      );
    });

    it('should throw when nonce validation returns null (nonce used or expired)', async () => {
      const keypair = StellarSdk.Keypair.random();
      const nonce = 'reused-nonce';
      const signature = keypair.sign(Buffer.from(nonce)).toString('hex');

      const dto: VerifyDto = {
        publicKey: keypair.publicKey(),
        signature,
        message: nonce,
      };

      jest.spyOn(nonceService, 'findAndValidateNonce').mockResolvedValue(null); // Nonce already used or expired

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

      const validNonce: Nonce = {
        id: 'nonce-uuid-5',
        nonce: message,
        publicKey: keypair.publicKey(),
        createdAt: new Date(),
        expiresAt: new Date(Date.now() + 5 * 60 * 1000),
        used: false,
      };

      const dto: VerifyDto = {
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

      const dto: VerifyDto = {
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

      const validNonce: Nonce = {
        id: 'nonce-uuid-6',
        nonce,
        publicKey: keypair.publicKey(),
        createdAt: new Date(),
        expiresAt: new Date(Date.now() + 5 * 60 * 1000),
        used: false,
      };

      const dto: VerifyDto = {
        publicKey: keypair.publicKey(),
        signature,
        message: nonce,
      };

      const user: User = {
        id: 'user-uuid-1',
        publicKey: keypair.publicKey(),
        displayName: null,
        createdAt: new Date(),
        updatedAt: new Date(),
      };

      jest
        .spyOn(nonceService, 'findAndValidateNonce')
        .mockResolvedValue(validNonce);
      jest.spyOn(nonceService, 'markNonceAsUsed').mockResolvedValue();
      jest.spyOn(usersService, 'findOrCreate').mockResolvedValue(user);
      jest.spyOn(jwtService, 'sign').mockReturnValue('jwt-token');

      const result = await service.verify(dto);

      expect(result).toEqual({
        accessToken: 'jwt-token',
        user,
      });
      expect(nonceService.findAndValidateNonce).toHaveBeenCalledWith(nonce);
      expect(nonceService.markNonceAsUsed).toHaveBeenCalledWith(validNonce.id);
      expect(usersService.findOrCreate).toHaveBeenCalledWith(
        keypair.publicKey(),
      );
      expect(jwtService.sign).toHaveBeenCalledWith({
        sub: user.id,
        publicKey: user.publicKey,
      });
    });

    it('should create a new user if they do not exist during verification', async () => {
      const keypair = StellarSdk.Keypair.random();
      const nonce = 'new-user-nonce';
      const signature = keypair.sign(Buffer.from(nonce)).toString('hex');

      const validNonce: Nonce = {
        id: 'nonce-uuid-7',
        nonce,
        publicKey: keypair.publicKey(),
        createdAt: new Date(),
        expiresAt: new Date(Date.now() + 5 * 60 * 1000),
        used: false,
      };

      const dto: VerifyDto = {
        publicKey: keypair.publicKey(),
        signature,
        message: nonce,
      };

      const newUser: User = {
        id: 'user-uuid-new',
        publicKey: keypair.publicKey(),
        displayName: null,
        createdAt: new Date(),
        updatedAt: new Date(),
      };

      jest
        .spyOn(nonceService, 'findAndValidateNonce')
        .mockResolvedValue(validNonce);
      jest.spyOn(nonceService, 'markNonceAsUsed').mockResolvedValue();
      jest.spyOn(usersService, 'findOrCreate').mockResolvedValue(newUser);
      jest.spyOn(jwtService, 'sign').mockReturnValue('jwt-token');

      const result = await service.verify(dto);

      expect(usersService.findOrCreate).toHaveBeenCalledWith(
        keypair.publicKey(),
      );
      expect(result.user).toEqual(newUser);
    });
  });
});
