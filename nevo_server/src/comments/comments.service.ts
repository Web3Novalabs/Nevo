import {
  BadRequestException,
  ForbiddenException,
  Injectable,
  NotFoundException,
} from '@nestjs/common';
import { InjectRepository } from '@nestjs/typeorm';
import { Repository } from 'typeorm';
import { Comment } from './comment.entity';
import { Pool } from '../pools/pool.entity';
import { CreateCommentDto } from './dto/create-comment.dto';
import { UpdateCommentDto } from './dto/update-comment.dto';

export interface PaginatedComments {
  data: Comment[];
  total: number;
  page: number;
  limit: number;
}

@Injectable()
export class CommentsService {
  constructor(
    @InjectRepository(Comment)
    private readonly commentRepository: Repository<Comment>,
    @InjectRepository(Pool)
    private readonly poolRepository: Repository<Pool>,
  ) {}

  async findForPool(
    poolId: string,
    page = 1,
    limit = 20,
  ): Promise<PaginatedComments> {
    const safePage = page > 0 ? page : 1;
    const safeLimit = limit > 0 ? limit : 20;

    const [topLevel, total] = await this.commentRepository.findAndCount({
      where: {
        poolId,
        parentId: null,
        deletedAt: null,
      },
      order: { createdAt: 'ASC' },
      skip: (safePage - 1) * safeLimit,
      take: safeLimit,
    });

    const data: Comment[] = [];
    for (const comment of topLevel) {
      const replies = await this.commentRepository.find({
        where: {
          poolId,
          parentId: comment.id,
          deletedAt: null,
        },
        order: { createdAt: 'ASC' },
      });
      data.push(Object.assign(comment, { replies }));
    }

    return {
      data,
      total,
      page: safePage,
      limit: safeLimit,
    };
  }

  async create(
    poolId: string,
    authorWallet: string,
    dto: CreateCommentDto,
  ): Promise<Comment> {
    const pool = await this.poolRepository.findOne({ where: { id: poolId } });
    if (!pool) {
      throw new NotFoundException('Pool not found');
    }

    if (dto.parentId) {
      const parent = await this.commentRepository.findOne({
        where: { id: dto.parentId },
      });
      if (!parent || parent.deletedAt) {
        throw new NotFoundException('Parent comment not found');
      }
      if (parent.poolId !== poolId) {
        throw new BadRequestException(
          'Parent comment does not belong to this pool',
        );
      }
      if (parent.parentId) {
        throw new BadRequestException(
          'Replies are only allowed one level deep',
        );
      }
    }

    const comment = this.commentRepository.create({
      poolId,
      authorWallet,
      text: dto.text,
      parentId: dto.parentId ?? null,
    });

    return this.commentRepository.save(comment);
  }

  async update(
    id: string,
    publicKey: string,
    dto: UpdateCommentDto,
  ): Promise<Comment> {
    const comment = await this.getActiveComment(id);
    await this.assertCanModify(comment, publicKey);

    comment.text = dto.text;
    return this.commentRepository.save(comment);
  }

  async remove(id: string, publicKey: string): Promise<void> {
    const comment = await this.getActiveComment(id);
    await this.assertCanModify(comment, publicKey);

    comment.deletedAt = new Date();
    await this.commentRepository.save(comment);
  }

  private async getActiveComment(id: string): Promise<Comment> {
    const comment = await this.commentRepository.findOne({ where: { id } });
    if (!comment || comment.deletedAt) {
      throw new NotFoundException('Comment not found');
    }
    return comment;
  }

  private async assertCanModify(
    comment: Comment,
    publicKey: string,
  ): Promise<void> {
    if (comment.authorWallet === publicKey) {
      return;
    }

    const pool = await this.poolRepository.findOne({
      where: { id: comment.poolId },
    });
    if (pool && pool.creatorWallet === publicKey) {
      return;
    }

    throw new ForbiddenException(
      'Only the author or the pool creator can modify this comment',
    );
  }
}