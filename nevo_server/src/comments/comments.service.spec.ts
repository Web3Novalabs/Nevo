import { Test, TestingModule } from '@nestjs/testing';
import { getRepositoryToken } from '@nestjs/typeorm';
import { ForbiddenException, NotFoundException, BadRequestException } from '@nestjs/common';
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

  const makeComment = (overrides: Partial<Comment> = {}): Comment =>
    ({
      id: 'c-1',
      poolId,
      authorWallet,
      text: 'hello',
      parentId: null,
      createdAt: new Date('2024-01-01T00:00:00Z'),
      updatedAt: new Date('2024-01-01T00:00:00Z'),
      deletedAt: null,
      ...overrides,
    } as Comment);

  beforeEach(async () => {
    const commentRepoMock = {
      find: jest.fn(),
      findOne: jest.fn(),
      create: jest.fn(),
      save: jest.fn(),
      update: jest.fn(),
      count: jest.fn(),
    };
    const poolRepoMock = {
      findOne: jest.fn(),
    };

    const module: TestingModule = await Test.createTestingModule({
      providers: [
        CommentsService,
        { provide: getRepositoryToken(Comment), useValue: commentRepoMock },
        { provide: getRepositoryToken(Pool), useValue: poolRepoMock },
      ],
    }).compile();

    service = module.get<CommentsService>(CommentsService);
    commentRepo = module.get(getRepositoryToken(Comment));
    poolRepo = module.get(getRepositoryToken(Pool));
  });

  describe('list', () => {
    it('returns threaded comments with pagination shape', async () => {
      const parent = makeComment({ id: 'p-1', parentId: null });
      const child = makeComment({ id: 'c-2', parentId: 'p-1' });
      commentRepo.find.mockResolvedValue([parent, child]);
      commentRepo.count.mockResolvedValue(2);

      const result = await service.list(poolId, { page: 1, limit: 10 });

      expect(result).toEqual({
        data: [
          {
            ...parent,
            replies: [{ ...child, replies: [] }],
          },
        ],
        meta: { page: 1, limit: 10, total: 2, totalPages: 1 },
      });
    });

    it('nests replies one level under their parent', async () => {
      const parent = makeComment({ id: 'p-1', parentId: null });
      const childA = makeComment({ id: 'c-a', parentId: 'p-1' });
      const childB = makeComment({ id: 'c-b', parentId: 'p-1' });
      commentRepo.find.mockResolvedValue([parent, childA, childB]);
      commentRepo.count.mockResolvedValue(3);

      const result = await service.list(poolId, { page: 1, limit: 10 });

      expect(result.data).toHaveLength(1);
      expect(result.data[0].replies).toHaveLength(2);
      expect(result.data[0].replies.map((r) => r.id)).toEqual(['c-a', 'c-b']);
    });

    it('excludes soft-deleted comments', async () => {
      commentRepo.find.mockResolvedValue([]);
      commentRepo.count.mockResolvedValue(0);

      await service.list(poolId, { page: 1, limit: 10 });

      const findArgs = commentRepo.find.mock.calls[0][0] as any;
      expect(findArgs.where).toMatchObject({ poolId, deletedAt: expect.anything() });
    });

    it('computes totalPages correctly', async () => {
      commentRepo.find.mockResolvedValue([]);
      commentRepo.count.mockResolvedValue(25);

      const result = await service.list(poolId, { page: 2, limit: 10 });

      expect(result.meta).toEqual({ page: 2, limit: 10, total: 25, totalPages: 3 });
    });
  });

  describe('create', () => {
    it('creates a top-level comment', async () => {
      const created = makeComment();
      commentRepo.create.mockReturnValue(created);
      commentRepo.save.mockResolvedValue(created);

      const result = await service.create(poolId, authorWallet, { text: 'hello' });

      expect(commentRepo.create).toHaveBeenCalledWith(
        expect.objectContaining({ poolId, authorWallet, text: 'hello', parentId: null }),
      );
      expect(result).toEqual(created);
    });

    it('creates a reply when parentId is provided and parent exists', async () => {
      const parent = makeComment({ id: 'p-1', parentId: null });
      commentRepo.findOne.mockResolvedValue(parent);
      const created = makeComment({ id: 'c-2', parentId: 'p-1' });
      commentRepo.create.mockReturnValue(created);
      commentRepo.save.mockResolvedValue(created);

      const result = await service.create(poolId, authorWallet, {
        text: 'reply',
        parentId: 'p-1',
      });

      expect(result.parentId).toBe('p-1');
    });

    it('rejects reply to a non-existent parent', async () => {
      commentRepo.findOne.mockResolvedValue(null);

      await expect(
        service.create(poolId, authorWallet, { text: 'reply', parentId: 'missing' }),
      ).rejects.toBeInstanceOf(NotFoundException);
    });

    it('rejects nested replies beyond one level', async () => {
      const parent = makeComment({ id: 'p-1', parentId: 'grandparent' });
      commentRepo.findOne.mockResolvedValue(parent);

      await expect(
        service.create(poolId, authorWallet, { text: 'reply', parentId: 'p-1' }),
      ).rejects.toBeInstanceOf(BadRequestException);
    });

    it('rejects text exceeding the length limit', async () => {
      const longText = 'a'.repeat(2001);

      await expect(
        service.create(poolId, authorWallet, { text: longText }),
      ).rejects.toBeInstanceOf(BadRequestException);
    });

    it('rejects empty text', async () => {
      await expect(
        service.create(poolId, authorWallet, { text: '   ' }),
      ).rejects.toBeInstanceOf(BadRequestException);
    });
  });

  describe('update', () => {
    it('allows the author to update their comment', async () => {
      const comment = makeComment();
      commentRepo.findOne.mockResolvedValue(comment);
      commentRepo.save.mockResolvedValue({ ...comment, text: 'edited' });

      const result = await service.update('c-1', authorWallet, { text: 'edited' });

      expect(result.text).toBe('edited');
    });

    it('allows the pool creator to update any comment', async () => {
      const comment = makeComment();
      commentRepo.findOne.mockResolvedValue(comment);
      poolRepo.findOne.mockResolvedValue(mockPool);
      commentRepo.save.mockResolvedValue({ ...comment, text: 'moderated' });

      const result = await service.update('c-1', creatorWallet, { text: 'moderated' });

      expect(result.text).toBe('moderated');
    });

    it('forbids a non-author non-creator from updating', async () => {
      const comment = makeComment();
      commentRepo.findOne.mockResolvedValue(comment);
      poolRepo.findOne.mockResolvedValue(mockPool);

      await expect(
        service.update('c-1', otherWallet, { text: 'hacked' }),
      ).rejects.toBeInstanceOf(ForbiddenException);
    });

    it('throws NotFound when comment does not exist', async () => {
      commentRepo.findOne.mockResolvedValue(null);

      await expect(
        service.update('missing', authorWallet, { text: 'x' }),
      ).rejects.toBeInstanceOf(NotFoundException);
    });

    it('rejects text exceeding the length limit', async () => {
      const comment = makeComment();
      commentRepo.findOne.mockResolvedValue(comment);

      await expect(
        service.update('c-1', authorWallet, { text: 'a'.repeat(2001) }),
      ).rejects.toBeInstanceOf(BadRequestException);
    });
  });

  describe('remove', () => {
    it('allows the author to soft-delete their comment', async () => {
      const comment = makeComment();
      commentRepo.findOne.mockResolvedValue(comment);
      commentRepo.save.mockResolvedValue({ ...comment, deletedAt: new Date() });

      await service.remove('c-1', authorWallet);

      expect(commentRepo.save).toHaveBeenCalledWith(
        expect.objectContaining({ id: 'c-1', deletedAt: expect.any(Date) }),
      );
    });

    it('allows the pool creator to soft-delete any comment', async () => {
      const comment = makeComment();
      commentRepo.findOne.mockResolvedValue(comment);
      poolRepo.findOne.mockResolvedValue(mockPool);
      commentRepo.save.mockResolvedValue({ ...comment, deletedAt: new Date() });

      await service.remove('c-1', creatorWallet);

      expect(commentRepo.save).toHaveBeenCalledWith(
        expect.objectContaining({ id: 'c-1', deletedAt: expect.any(Date) }),
      );
    });

    it('forbids a non-author non-creator from deleting', async () => {
      const comment = makeComment();
      commentRepo.findOne.mockResolvedValue(comment);
      poolRepo.findOne.mockResolvedValue(mockPool);

      await expect(service.remove('c-1', otherWallet)).rejects.toBeInstanceOf(
        ForbiddenException,
      );
      expect(commentRepo.save).not.toHaveBeenCalled();
    });

    it('throws NotFound when comment does not exist', async () => {
      commentRepo.findOne.mockResolvedValue(null);

      await expect(service.remove('missing', authorWallet)).rejects.toBeInstanceOf(
        NotFoundException,
      );
    });
  });
});
```

File: nevo_server/src/comments/comments.controller.spec.ts
Change type: create
Step: Add unit tests for CommentsService (threading, pagination shape, author/creator permission checks, soft delete) and controller tests (public GET, JWT POST, throttling, 403 for non-author/non-creator PATCH/DELETE).

```typescript
import { Test, TestingModule } from '@nestjs/testing';
import { ForbiddenException, INestApplication } from '@nestjs/common';
import { ThrottlerGuard, ThrottlerModule } from '@nestjs/throttler';
import { APP_GUARD } from '@nestjs/core';
import * as request from 'supertest';
import { CommentsController } from './comments.controller';
import { CommentsService } from './comments.service';
import { JwtAuthGuard } from '../auth/jwt-auth.guard';

describe('CommentsController', () => {
  let app: INestApplication;
  let service: jest.Mocked<CommentsService>;

  const poolId = 'pool-1';
  const authorWallet = 'author-wallet';

  const mockComment = {
    id: 'c-1',
    poolId,
    authorWallet,
    text: 'hello',
    parentId: null,
    createdAt: new Date('2024-01-01T00:00:00Z'),
    updatedAt: new Date('2024-01-01T00:00:00Z'),
    deletedAt: null,
  };

  const mockJwtGuard = {
    canActivate: (ctx: any) => {
      const req = ctx.switchToHttp().getRequest();
      const auth = req.headers['authorization'];
      if (!auth || !auth.startsWith('Bearer ')) {
        return false;
      }
      req.user = { wallet: auth.replace('Bearer ', '') };
      return true;
    },
  };

  beforeAll(async () => {
    service = {
      list: jest.fn(),
      create: jest.fn(),
      update: jest.fn(),
      remove: jest.fn(),
    } as unknown as jest.Mocked<CommentsService>;

    const module: TestingModule = await Test.createTestingModule({
      imports: [
        ThrottlerModule.forRoot([
          { name: 'comments', ttl: 60000, limit: 3 },
        ]),
      ],
      controllers: [CommentsController],
      providers: [
        { provide: CommentsService, useValue: service },
        { provide: APP_GUARD, useClass: ThrottlerGuard },
      ],
    })
      .overrideGuard(JwtAuthGuard)
      .useValue(mockJwtGuard)
      .compile();

    app = module.createNestApplication();
    await app.init();
  });

  afterAll(async () => {
    await app.close();
  });

  beforeEach(() => {
    jest.clearAllMocks();
  });

  describe('GET /pools/:id/comments', () => {
    it('is public and returns paginated threaded comments', async () => {
      service.list.mockResolvedValue({
        data: [{ ...mockComment, replies: [] }],
        meta: { page: 1, limit: 10, total: 1, totalPages: 1 },
      } as any);

      const res = await request(app.getHttpServer())
        .get(`/pools/${poolId}/comments`)
        .query({ page: 1, limit: 10 })
        .expect(200);

      expect(service.list).toHaveBeenCalledWith(poolId, { page: 1, limit: 10 });
      expect(res.body.data).toHaveLength(1);
      expect(res.body.meta).toEqual({ page: 1, limit: 10, total: 1, totalPages: 1 });
    });

    it('does not require authentication', async () => {
      service.list.mockResolvedValue({
        data: [],
        meta: { page: 1, limit: 10, total: 0, totalPages: 0 },
      } as any);

      await request(app.getHttpServer())
        .get(`/pools/${poolId}/comments`)
        .expect(200);
    });
  });

  describe('POST /pools/:id/comments', () => {
    it('requires JWT authentication', async () => {
      await request(app.getHttpServer())
        .post(`/pools/${poolId}/comments`)
        .send({ text: 'hello' })
        .expect(401);
    });

    it('creates a comment for an authenticated user', async () => {
      service.create.mockResolvedValue(mockComment as any);

      const res = await request(app.getHttpServer())
        .post(`/pools/${poolId}/comments`)
        .set('Authorization', `Bearer ${authorWallet}`)
        .send({ text: 'hello' })
        .expect(201);

      expect(service.create).toHaveBeenCalledWith(poolId, authorWallet, { text: 'hello' });
      expect(res.body.id).toBe('c-1');
    });

    it('throttles repeated POST requests', async () => {
      service.create.mockResolvedValue(mockComment as any);

      for (let i = 0; i < 3; i++) {
        await request(app.getHttpServer())
          .post(`/pools/${poolId}/comments`)
          .set('Authorization', `Bearer ${authorWallet}`)
          .send({ text: `hello ${i}` })
          .expect(201);
      }

      await request(app.getHttpServer())
        .post(`/pools/${poolId}/comments`)
        .set('Authorization', `Bearer ${authorWallet}`)
        .send({ text: 'too many' })
        .expect(429);
    });
  });

  describe('PATCH /pools/:id/comments/:commentId', () => {
    it('requires JWT authentication', async () => {
      await request(app.getHttpServer())
        .patch(`/pools/${poolId}/comments/c-1`)
        .send({ text: 'edited' })
        .expect(401);
    });

    it('updates a comment for the author', async () => {
      service.update.mockResolvedValue({ ...mockComment, text: 'edited' } as any);

      const res = await request(app.getHttpServer())
        .patch(`/pools/${poolId}/comments/c-1`)
        .set('Authorization', `Bearer ${authorWallet}`)
        .send({ text: 'edited' })
        .expect(200);

      expect(service.update).toHaveBeenCalledWith('c-1', authorWallet, { text: 'edited' });
      expect(res.body.text).toBe('edited');
    });

    it('returns 403 for a non-author non-creator', async () => {
      service.update.mockRejectedValue(new ForbiddenException());

      await request(app.getHttpServer())
        .patch(`/pools/${poolId}/comments/c-1`)
        .set('Authorization', 'Bearer other-wallet')
        .send({ text: 'hacked' })
        .expect(403);
    });
  });

  describe('DELETE /pools/:id/comments/:commentId', () => {
    it('requires JWT authentication', async () => {
      await request(app.getHttpServer())
        .delete(`/pools/${poolId}/comments/c-1`)
        .expect(401);
    });

    it('soft-deletes a comment for the author', async () => {
      service.remove.mockResolvedValue(undefined as any);

      await request(app.getHttpServer())
        .delete(`/pools/${poolId}/comments/c-1`)
        .set('Authorization', `Bearer ${authorWallet}`)
        .expect(200);

      expect(service.remove).toHaveBeenCalledWith('c-1', authorWallet);
    });

    it('returns 403 for a non-author non-creator', async () => {
      service.remove.mockRejectedValue(new ForbiddenException());

      await request(app.getHttpServer())
        .delete(`/pools/${poolId}/comments/c-1`)
        .set('Authorization', 'Bearer other-wallet')
        .expect(403);
    });
  });
});