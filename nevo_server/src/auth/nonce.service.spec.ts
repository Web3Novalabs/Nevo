import { Test, TestingModule } from '@nestjs/testing';
import { getRepositoryToken } from '@nestjs/typeorm';
import { FindOptionsWhere, Repository } from 'typeorm';
import { NonceService } from './nonce.service';
import { Nonce } from './nonce.entity';

describe('NonceService', () => {
  let service: NonceService;
  let repository: Repository<Nonce>;

  const mockNonce: Nonce = {
    id: 'nonce-uuid-1',
    nonce: 'test-nonce-12345',
    publicKey: 'GABC123',
    createdAt: new Date(),
    expiresAt: new Date(Date.now() + 5 * 60 * 1000),
    used: false,
  };

  beforeEach(async () => {
    const module: TestingModule = await Test.createTestingModule({
      providers: [
        NonceService,
        {
          provide: getRepositoryToken(Nonce),
          useValue: {
            save: jest.fn(),
            findOne: jest.fn(),
            update: jest.fn(),
            delete: jest.fn(),
            remove: jest.fn(),
          },
        },
      ],
    }).compile();

    service = module.get(NonceService);
    repository = module.get(getRepositoryToken(Nonce));
  });

  afterEach(() => {
    jest.clearAllMocks();
  });

  describe('generateNonce', () => {
    it('should generate and save a new nonce', async () => {
      const publicKey = 'GABC123';
      jest.spyOn(repository, 'save').mockResolvedValue(mockNonce);

      const nonce = await service.generateNonce(publicKey);

      expect(repository.save).toHaveBeenCalled();
      expect(nonce).toBeDefined();
      expect(typeof nonce).toBe('string');
      expect(nonce.length).toBe(32); // 16 bytes * 2 hex chars
    });

    it('should set expiration time to 5 minutes from now', async () => {
      const publicKey = 'GABC123';
      const beforeTime = Date.now();

      jest.spyOn(repository, 'save').mockResolvedValue(mockNonce);

      await service.generateNonce(publicKey);

      const callArgs = jest.spyOn(repository, 'save').mock.calls[0][0];
      const expiresAt = new Date((callArgs as Nonce).expiresAt);

      expect(expiresAt.getTime()).toBeGreaterThan(beforeTime + 4 * 60 * 1000); // At least 4 minutes
      expect(expiresAt.getTime()).toBeLessThanOrEqual(
        beforeTime + 6 * 60 * 1000,
      ); // At most 6 minutes
    });

    it('should generate unique nonces on each call', async () => {
      const publicKey = 'GABC123';
      jest.spyOn(repository, 'save').mockResolvedValue(mockNonce);

      const nonce1 = await service.generateNonce(publicKey);
      jest.clearAllMocks();
      jest.spyOn(repository, 'save').mockResolvedValue(mockNonce);

      const nonce2 = await service.generateNonce(publicKey);

      expect(nonce1).not.toEqual(nonce2);
    });
  });

  describe('findAndValidateNonce', () => {
    it('should return the nonce if valid and not expired', async () => {
      const nonce = 'test-nonce-12345';
      jest.spyOn(repository, 'findOne').mockResolvedValue({
        ...mockNonce,
        used: false,
        expiresAt: new Date(Date.now() + 1000), // Not expired
      });

      const result = await service.findAndValidateNonce(nonce);

      expect(result).toBeDefined();
      expect(result?.nonce).toBe(nonce);
      expect(result?.used).toBe(false);
    });

    it('should return null if nonce does not exist', async () => {
      jest.spyOn(repository, 'findOne').mockResolvedValue(null);

      const result = await service.findAndValidateNonce('nonexistent-nonce');

      expect(result).toBeNull();
    });

    it('should return null and delete if nonce is expired', async () => {
      const expiredNonce = {
        ...mockNonce,
        expiresAt: new Date(Date.now() - 1000), // Already expired
      };

      jest.spyOn(repository, 'findOne').mockResolvedValue(expiredNonce);
      jest.spyOn(repository, 'remove').mockResolvedValue(expiredNonce);

      const result = await service.findAndValidateNonce('expired-nonce');

      expect(result).toBeNull();
      expect(repository.remove).toHaveBeenCalledWith(expiredNonce);
    });

    it('should return null if nonce has already been used', async () => {
      const usedNonce = {
        ...mockNonce,
        used: true,
      };

      jest.spyOn(repository, 'findOne').mockResolvedValue(usedNonce);

      const result = await service.findAndValidateNonce('used-nonce');

      expect(result).toBeNull();
    });

    it('should validate nonce expiration correctly (just expired)', async () => {
      const justExpiredNonce = {
        ...mockNonce,
        expiresAt: new Date(Date.now() - 1), // Just expired
      };

      jest.spyOn(repository, 'findOne').mockResolvedValue(justExpiredNonce);
      jest.spyOn(repository, 'remove').mockResolvedValue(justExpiredNonce);

      const result = await service.findAndValidateNonce('just-expired-nonce');

      expect(result).toBeNull();
    });

    it('should validate nonce expiration correctly (just about to expire)', async () => {
      const almostExpiredNonce = {
        ...mockNonce,
        expiresAt: new Date(Date.now() + 100), // Expires in 100ms
        used: false,
      };

      jest.spyOn(repository, 'findOne').mockResolvedValue(almostExpiredNonce);

      const result = await service.findAndValidateNonce('almost-expired-nonce');

      expect(result).toBeDefined();
      expect(result?.nonce).toBe(almostExpiredNonce.nonce);
    });
  });

  describe('markNonceAsUsed', () => {
    it('should mark a nonce as used', async () => {
      const nonceId = 'nonce-uuid-1';
      jest.spyOn(repository, 'update').mockResolvedValue({
        affected: 1,
      } as any);

      await service.markNonceAsUsed(nonceId);

      expect(repository.update).toHaveBeenCalledWith(nonceId, { used: true });
    });
  });

  describe('deleteExpiredNonces', () => {
    it('should delete nonces that have expired', async () => {
      jest.spyOn(repository, 'delete').mockResolvedValue({
        affected: 5,
      } as any);

      await service.deleteExpiredNonces();

      expect(repository.delete).toHaveBeenCalled();
      const deleteCall = jest.spyOn(repository, 'delete').mock.calls[0][0];
      expect((deleteCall as FindOptionsWhere<Nonce>).expiresAt).toEqual(
        expect.any(Date),
      );
    });
  });
});
