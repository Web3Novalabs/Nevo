import { Injectable } from '@nestjs/common';
import { InjectRepository } from '@nestjs/typeorm';
import { Repository } from 'typeorm';
import { User } from './user.entity';
import { Pool } from '../pools/pool.entity';

export interface PublicUserProfile {
  publicKey: string;
  displayName: string | null;
  joinedAt: Date;
  poolsCreated: number;
  totalRaised: number;
}

@Injectable()
export class UsersService {
  constructor(
    @InjectRepository(User)
    private readonly userRepo: Repository<User>,
    @InjectRepository(Pool)
    private readonly poolRepo: Repository<Pool>,
  ) {}

  /**
   * Returns null when no user exists for the given publicKey;
   * UsersController maps that to a 404.
   */
  async updateDisplayName(
    publicKey: string,
    displayName: string,
  ): Promise<User | null> {
    const user = await this.userRepo.findOne({ where: { publicKey } });
    if (!user) return null;
    user.displayName = displayName;
    return this.userRepo.save(user);
  }

  findByPublicKey(publicKey: string): Promise<User | null> {
    return this.userRepo.findOne({ where: { publicKey } });
  }

  async findOrCreate(publicKey: string): Promise<User> {
    const existing = await this.userRepo.findOne({ where: { publicKey } });
    if (existing) return existing;

    return this.userRepo.save(
      this.userRepo.create({ publicKey, displayName: null }),
    );
  }

  /**
   * Increments the user's tokenVersion, invalidating all previously issued
   * JWTs. Returns null when no user exists for the given publicKey.
   */
  async incrementTokenVersion(publicKey: string): Promise<User | null> {
    const user = await this.userRepo.findOne({ where: { publicKey } });
    if (!user) return null;
    user.tokenVersion = (user.tokenVersion ?? 0) + 1;
    return this.userRepo.save(user);
  }

  /**
   * Returns a public profile DTO for the given publicKey, or null when the
   * user is unknown. Never exposes internal ids or private fields.
   */
  async findPublicProfile(
    publicKey: string,
  ): Promise<PublicUserProfile | null> {
    const user = await this.userRepo.findOne({ where: { publicKey } });
    if (!user) return null;

    const aggregate = await this.poolRepo
      .createQueryBuilder('pool')
      .select('COUNT(pool.id)', 'poolsCreated')
      .addSelect('COALESCE(SUM(pool.totalRaised), 0)', 'totalRaised')
      .where('pool.creatorPublicKey = :publicKey', { publicKey })
      .getRawOne<{ poolsCreated: string; totalRaised: string }>();

    return {
      publicKey: user.publicKey,
      displayName: user.displayName ?? null,
      joinedAt: user.createdAt,
      poolsCreated: Number(aggregate?.poolsCreated ?? 0),
      totalRaised: Number(aggregate?.totalRaised ?? 0),
    };
  }
}