import { Injectable } from '@nestjs/common';
import { InjectRepository } from '@nestjs/typeorm';
import { Repository } from 'typeorm';
import { User } from './user.entity';

@Injectable()
export class UsersService {
  constructor(
    @InjectRepository(User)
    private readonly userRepo: Repository<User>,
  ) {}

  async updateDisplayName(publicKey: string, displayName: string): Promise<User | null> {
    const user = await this.userRepo.findOne({ where: { publicKey } });
    if (!user) return null;
    user.displayName = displayName;
    return this.userRepo.save(user);
  }

  async findOrCreate(publicKey: string): Promise<User> {
    const existing = await this.userRepo.findOne({ where: { publicKey } });
    if (existing) return existing;

    return this.userRepo.save(
      this.userRepo.create({ publicKey, displayName: null }),
    );
  }
}
