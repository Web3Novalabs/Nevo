import { Test, TestingModule } from '@nestjs/testing';
import { getRepositoryToken } from '@nestjs/typeorm';
import {
  BadRequestException,
  ForbiddenException,
  NotFoundException,
} from '@nestjs/common';
import { Repository } from 'typeorm';
import { CommentsService } from './comments.service';
import { Comment } from './comment.entity';
import { Pool } from '../pools/pool.entity';

describe('CommentsService', () => {
  let service: CommentsService;
  let commentRepo: jest.Mocked<Repository<Comment>>;
  let poolRepo: jest.Mocked<Repository<Pool>>;

  const poolId = 'pool-1';
  const creatorWallet = 'creator-wallet';
  const authorWallet = 'author-wallet';
  const otherWallet = 'other-wallet';

  const mockPool = { id: poolId, creatorWallet } as unknown as Pool;

  const makeComment = (overrides: Partial<Comment> = {}): Comment => ({
    id: 'c-1',
    poolId,
    authorWallet,
    text: 'hello',
    parentId: null,
    createdAt: new Date('2024-01-01T00:00:00Z'),
    updatedAt: new Date('2024-01-01T00:00:00Z'),
    deletedAt: null,
    ...overrides,
  });

  beforeEach(async () => {
    const module: TestingModule = await Test.createTestingModule({
      providers: [
        CommentsService,
        {
          provide: getRepositoryToken(Comment),
          useValue: {
            find: jest.fn(),
            findAndCount: jest.fn(),
            findOne: jest.fn(),
            create: jest.fn(),
            save: jest.fn(),
          },
        },
        {
          provide: getRepositoryToken(Pool),
          useValue: { findOne: jest.fn() },
        },
      ],
    }).compile();

    service = module.get(CommentsService);
    commentRepo = module.get(getRepositoryToken(Comment));
    poolRepo = module.get(getRepositoryToken(Pool));
  });

  describe('findForPool', () => {
    it('returns top-level comments with their replies and pagination info', async () => {
      const parent = makeComment({ id: 'p-1' });
      const reply = makeComment({ id: 'c-2', parentId: 'p-1' });
      commentRepo.findAndCount.mockResolvedValue([[parent], 1]);
      commentRepo.find.mockResolvedValue([reply]);

      const result = await service.findForPool(poolId, 1, 10);

      expect(result).toEqual({
        data: [{ ...parent, replies: [reply] }],
        total: 1,
        page: 1,
        limit: 10,
      });
    });

    it('applies skip/take for the requested page', async () => {
      commentRepo.findAndCount.mockResolvedValue([[], 25]);

      await service.findForPool(poolId, 3, 10);

      expect(commentRepo.findAndCount).toHaveBeenCalledWith(
        expect.objectContaining({ skip: 20, take: 10 }),
      );
    });

    it('falls back to sane defaults for invalid page/limit', async () => {
      commentRepo.findAndCount.mockResolvedValue([[], 0]);

      const result = await service.findForPool(poolId, 0, -5);

      expect(result).toMatchObject({ page: 1, limit: 20 });
    });

    it('only queries non-deleted comments of the pool', async () => {
      commentRepo.findAndCount.mockResolvedValue([[], 0]);

      await service.findForPool(poolId);

      const args = commentRepo.findAndCount.mock.calls[0][0];
      expect(args?.where).toMatchObject({
        poolId,
        parentId: expect.anything(),
        deletedAt: expect.anything(),
      });
    });
  });

  describe('create', () => {
    it('creates a top-level comment', async () => {
      const created = makeComment();
      poolRepo.findOne.mockResolvedValue(mockPool);
      commentRepo.create.mockReturnValue(created);
      commentRepo.save.mockResolvedValue(created);

      const result = await service.create(poolId, authorWallet, {
        text: 'hello',
      });

      expect(commentRepo.create).toHaveBeenCalledWith({
        poolId,
        authorWallet,
        text: 'hello',
        parentId: null,
      });
      expect(result).toBe(created);
    });

    it('throws NotFound when the pool does not exist', async () => {
      poolRepo.findOne.mockResolvedValue(null);

      await expect(
        service.create(poolId, authorWallet, { text: 'hello' }),
      ).rejects.toBeInstanceOf(NotFoundException);
      expect(commentRepo.save).not.toHaveBeenCalled();
    });

    it('creates a reply when the parent is a top-level comment of the pool', async () => {
      const parent = makeComment({ id: 'p-1' });
      const created = makeComment({ id: 'c-2', parentId: 'p-1' });
      poolRepo.findOne.mockResolvedValue(mockPool);
      commentRepo.findOne.mockResolvedValue(parent);
      commentRepo.create.mockReturnValue(created);
      commentRepo.save.mockResolvedValue(created);

      const result = await service.create(poolId, authorWallet, {
        text: 'reply',
        parentId: 'p-1',
      });

      expect(result.parentId).toBe('p-1');
    });

    it('rejects a reply to a missing or soft-deleted parent', async () => {
      poolRepo.findOne.mockResolvedValue(mockPool);
      commentRepo.findOne.mockResolvedValueOnce(null);
      await expect(
        service.create(poolId, authorWallet, { text: 'r', parentId: 'x' }),
      ).rejects.toBeInstanceOf(NotFoundException);

      commentRepo.findOne.mockResolvedValueOnce(
        makeComment({ id: 'p-1', deletedAt: new Date() }),
      );
      await expect(
        service.create(poolId, authorWallet, { text: 'r', parentId: 'p-1' }),
      ).rejects.toBeInstanceOf(NotFoundException);
    });

    it('rejects a parent that belongs to another pool', async () => {
      poolRepo.findOne.mockResolvedValue(mockPool);
      commentRepo.findOne.mockResolvedValue(
        makeComment({ id: 'p-1', poolId: 'other-pool' }),
      );

      await expect(
        service.create(poolId, authorWallet, { text: 'r', parentId: 'p-1' }),
      ).rejects.toBeInstanceOf(BadRequestException);
    });

    it('rejects nested replies beyond one level', async () => {
      poolRepo.findOne.mockResolvedValue(mockPool);
      commentRepo.findOne.mockResolvedValue(
        makeComment({ id: 'p-1', parentId: 'grandparent' }),
      );

      await expect(
        service.create(poolId, authorWallet, { text: 'r', parentId: 'p-1' }),
      ).rejects.toBeInstanceOf(BadRequestException);
    });
  });

  describe('update', () => {
    it('lets the author edit their comment', async () => {
      const comment = makeComment();
      commentRepo.findOne.mockResolvedValue(comment);
      commentRepo.save.mockImplementation((c) => Promise.resolve(c as Comment));

      const result = await service.update('c-1', authorWallet, {
        text: 'edited',
      });

      expect(result.text).toBe('edited');
    });

    it('lets the pool creator moderate any comment', async () => {
      commentRepo.findOne.mockResolvedValue(makeComment());
      poolRepo.findOne.mockResolvedValue(mockPool);
      commentRepo.save.mockImplementation((c) => Promise.resolve(c as Comment));

      const result = await service.update('c-1', creatorWallet, {
        text: 'moderated',
      });

      expect(result.text).toBe('moderated');
    });

    it('forbids anyone else', async () => {
      commentRepo.findOne.mockResolvedValue(makeComment());
      poolRepo.findOne.mockResolvedValue(mockPool);

      await expect(
        service.update('c-1', otherWallet, { text: 'hacked' }),
      ).rejects.toBeInstanceOf(ForbiddenException);
      expect(commentRepo.save).not.toHaveBeenCalled();
    });

    it('throws NotFound for missing or soft-deleted comments', async () => {
      commentRepo.findOne.mockResolvedValueOnce(null);
      await expect(
        service.update('missing', authorWallet, { text: 'x' }),
      ).rejects.toBeInstanceOf(NotFoundException);

      commentRepo.findOne.mockResolvedValueOnce(
        makeComment({ deletedAt: new Date() }),
      );
      await expect(
        service.update('c-1', authorWallet, { text: 'x' }),
      ).rejects.toBeInstanceOf(NotFoundException);
    });
  });

  describe('remove', () => {
    it('soft-deletes the comment for its author', async () => {
      commentRepo.findOne.mockResolvedValue(makeComment());

      await service.remove('c-1', authorWallet);

      expect(commentRepo.save).toHaveBeenCalledWith(
        expect.objectContaining({ id: 'c-1', deletedAt: expect.any(Date) }),
      );
    });

    it('soft-deletes the comment for the pool creator', async () => {
      commentRepo.findOne.mockResolvedValue(makeComment());
      poolRepo.findOne.mockResolvedValue(mockPool);

      await service.remove('c-1', creatorWallet);

      expect(commentRepo.save).toHaveBeenCalledWith(
        expect.objectContaining({ deletedAt: expect.any(Date) }),
      );
    });

    it('forbids anyone else', async () => {
      commentRepo.findOne.mockResolvedValue(makeComment());
      poolRepo.findOne.mockResolvedValue(mockPool);

      await expect(service.remove('c-1', otherWallet)).rejects.toBeInstanceOf(
        ForbiddenException,
      );
      expect(commentRepo.save).not.toHaveBeenCalled();
    });

    it('throws NotFound when the comment does not exist', async () => {
      commentRepo.findOne.mockResolvedValue(null);

      await expect(
        service.remove('missing', authorWallet),
      ).rejects.toBeInstanceOf(NotFoundException);
    });
  });
});
