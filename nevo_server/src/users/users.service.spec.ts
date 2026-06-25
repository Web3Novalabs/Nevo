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
      username: 'alice',
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
    expect(saved?.username).toBeNull();
  });
});
