import { Test, TestingModule } from '@nestjs/testing';
import { getRepositoryToken } from '@nestjs/typeorm';
import { UsersService } from './users.service';
import { User } from './user.entity';

describe('UsersService', () => {
  const publicKey = 'GABC1234567890';

  async function buildService(existing: User | null) {
    let lastSaved: User | undefined;
    const repo = {
      findOne: jest.fn().mockResolvedValue(existing),
      save: jest.fn().mockImplementation((u: User) => {
        lastSaved = u;
        return Promise.resolve({ ...u, id: 'new-uuid' });
      }),
      create: jest
        .fn()
        .mockImplementation((d: Partial<User>) => ({ ...d }) as User),
    };
    const module: TestingModule = await Test.createTestingModule({
      providers: [
        UsersService,
        { provide: getRepositoryToken(User), useValue: repo },
      ],
    }).compile();
    return {
      service: module.get(UsersService),
      repo,
      savedArg: () => lastSaved,
    };
  }

  it('returns existing user without creating a duplicate', async () => {
    const existing: User = {
      id: 'uuid-1',
      publicKey,
      displayName: 'alice',
      createdAt: new Date(),
      updatedAt: new Date(),
    };
    const { service, repo } = await buildService(existing);

    const result = await service.findOrCreate(publicKey);

    expect(result).toBe(existing);
    expect(repo.create).not.toHaveBeenCalled();
    expect(repo.save).not.toHaveBeenCalled();
  });

  it('creates new user with username null when none exists', async () => {
    const { service, savedArg } = await buildService(null);

    await service.findOrCreate(publicKey);

    const saved = savedArg();
    expect(saved?.publicKey).toBe(publicKey);
    expect(saved?.displayName).toBeNull();
  });

  it('updates the display name for an existing user', async () => {
    const existing: User = {
      id: 'uuid-1',
      publicKey,
      displayName: 'alice',
      createdAt: new Date(),
      updatedAt: new Date(),
    };
    const { service, repo, savedArg } = await buildService(existing);

    const result = await service.updateDisplayName(publicKey, 'Alice Cooper');

    expect(repo.findOne).toHaveBeenCalledWith({ where: { publicKey } });
    expect(savedArg()).toBe(existing);
    expect(repo.save).toHaveBeenCalledWith(existing);
    expect(result).toEqual({ ...existing, id: 'new-uuid' });
    expect(existing.displayName).toBe('Alice Cooper');
  });

  it('returns null when updating display name for a missing user', async () => {
    const { service, repo } = await buildService(null);

    const result = await service.updateDisplayName(publicKey, 'Alice Cooper');

    expect(repo.findOne).toHaveBeenCalledWith({ where: { publicKey } });
    expect(result).toBeNull();
    expect(repo.save).not.toHaveBeenCalled();
  });

  describe('findOrCreate race condition handling', () => {
    it('two concurrent findOrCreate calls for a new key both pass findOne but only one succeeds on save', async () => {
      const testKey = 'GABC1234567890NEW';
      const savedUsers: User[] = [];
      let saveCallCount = 0;

      const repo = {
        findOne: jest.fn().mockResolvedValue(null), // Both calls see no existing user
        save: jest.fn().mockImplementation((u: User) => {
          saveCallCount++;
          if (saveCallCount === 1) {
            // First save succeeds
            const saved = { ...u, id: 'uuid-first', createdAt: new Date() };
            savedUsers.push(saved);
            return Promise.resolve(saved);
          } else {
            // Second save fails with unique constraint violation
            const error: any = new Error('duplicate key value');
            error.code = '23505'; // PostgreSQL unique constraint code
            return Promise.reject(error);
          }
        }),
        create: jest
          .fn()
          .mockImplementation((d: Partial<User>) => ({ ...d }) as User),
      };

      const module: TestingModule = await Test.createTestingModule({
        providers: [
          UsersService,
          { provide: getRepositoryToken(User), useValue: repo },
        ],
      }).compile();

      const service = module.get(UsersService);

      // Start two concurrent findOrCreate calls
      const promise1 = service.findOrCreate(testKey);
      const promise2 = service.findOrCreate(testKey);

      // First call should succeed
      const result1 = await promise1;
      expect(result1).toBeDefined();
      expect(result1.publicKey).toBe(testKey);
      expect(result1.id).toBe('uuid-first');

      // Second call should fail with the unique constraint error
      try {
        await promise2;
        fail('Expected second findOrCreate to throw');
      } catch (error: any) {
        expect(error.code).toBe('23505');
      }

      // Verify save was called twice (both attempted)
      expect(repo.save).toHaveBeenCalledTimes(2);
      expect(repo.findOne).toHaveBeenCalledTimes(2);
    });

    it('sequential calls for the same key return the existing row on the second call', async () => {
      const testKey = 'GSEQ1234567890';
      const firstUser: User = {
        id: 'uuid-seq-1',
        publicKey: testKey,
        displayName: null,
        createdAt: new Date(),
        updatedAt: new Date(),
      };

      let findOneCallCount = 0;

      const repo = {
        findOne: jest.fn().mockImplementation(() => {
          findOneCallCount++;
          if (findOneCallCount === 1) {
            // First call: user does not exist
            return Promise.resolve(null);
          } else {
            // Subsequent calls: user now exists
            return Promise.resolve(firstUser);
          }
        }),
        save: jest.fn().mockImplementation((u: User) => {
          return Promise.resolve({ ...u, id: 'uuid-seq-1', createdAt: new Date() });
        }),
        create: jest
          .fn()
          .mockImplementation((d: Partial<User>) => ({ ...d }) as User),
      };

      const module: TestingModule = await Test.createTestingModule({
        providers: [
          UsersService,
          { provide: getRepositoryToken(User), useValue: repo },
        ],
      }).compile();

      const service = module.get(UsersService);

      // First call: creates new user
      const result1 = await service.findOrCreate(testKey);
      expect(result1).toBeDefined();
      expect(repo.save).toHaveBeenCalledTimes(1);

      // Second call: should find existing user and not create
      const result2 = await service.findOrCreate(testKey);
      expect(result2).toEqual(firstUser);
      expect(result2).toBe(firstUser);

      // save should still only be called once (from first call)
      expect(repo.save).toHaveBeenCalledTimes(1);

      // findOne should be called twice
      expect(repo.findOne).toHaveBeenCalledTimes(2);
    });

    it('three sequential calls all return the same existing row after first creation', async () => {
      const testKey = 'GTRIPLE1234567890';
      const createdUser: User = {
        id: 'uuid-triple',
        publicKey: testKey,
        displayName: null,
        createdAt: new Date(),
        updatedAt: new Date(),
      };

      let findOneCallCount = 0;

      const repo = {
        findOne: jest.fn().mockImplementation(() => {
          findOneCallCount++;
          if (findOneCallCount === 1) {
            return Promise.resolve(null);
          } else {
            return Promise.resolve(createdUser);
          }
        }),
        save: jest.fn().mockResolvedValue({ ...createdUser, createdAt: new Date() }),
        create: jest
          .fn()
          .mockImplementation((d: Partial<User>) => ({ ...d }) as User),
      };

      const module: TestingModule = await Test.createTestingModule({
        providers: [
          UsersService,
          { provide: getRepositoryToken(User), useValue: repo },
        ],
      }).compile();

      const service = module.get(UsersService);

      const result1 = await service.findOrCreate(testKey);
      const result2 = await service.findOrCreate(testKey);
      const result3 = await service.findOrCreate(testKey);

      expect(result1).toBeDefined();
      expect(result2).toEqual(createdUser);
      expect(result3).toEqual(createdUser);

      expect(repo.save).toHaveBeenCalledTimes(1);
      expect(repo.create).toHaveBeenCalledTimes(1);
      expect(repo.findOne).toHaveBeenCalledTimes(3);
    });

    it('concurrent calls with different keys do not interfere', async () => {
      const key1 = 'GKEY1_NEW';
      const key2 = 'GKEY2_NEW';
      const results: Record<string, User> = {};

      const repo = {
        findOne: jest.fn().mockResolvedValue(null),
        save: jest.fn().mockImplementation((u: User) => {
          const saved = { ...u, id: `uuid-${u.publicKey}`, createdAt: new Date() };
          results[u.publicKey] = saved;
          return Promise.resolve(saved);
        }),
        create: jest
          .fn()
          .mockImplementation((d: Partial<User>) => ({ ...d }) as User),
      };

      const module: TestingModule = await Test.createTestingModule({
        providers: [
          UsersService,
          { provide: getRepositoryToken(User), useValue: repo },
        ],
      }).compile();

      const service = module.get(UsersService);

      const promise1 = service.findOrCreate(key1);
      const promise2 = service.findOrCreate(key2);

      const user1 = await promise1;
      const user2 = await promise2;

      expect(user1.publicKey).toBe(key1);
      expect(user2.publicKey).toBe(key2);
      expect(user1.id).toBe(`uuid-${key1}`);
      expect(user2.id).toBe(`uuid-${key2}`);
      expect(repo.save).toHaveBeenCalledTimes(2);
    });
  });
});
