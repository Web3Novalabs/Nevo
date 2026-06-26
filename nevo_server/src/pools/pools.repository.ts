import { Injectable } from '@nestjs/common';
import { Pool } from './entities/pool.entity';

// TODO: Replace with real TypeORM Repository<Pool> from issue #16
@Injectable()
export class MockPoolRepository {
  private pools: Pool[] = [
    {
      id: '1',
      title: 'Clean Water Initiative',
      description: 'Providing clean drinking water to rural communities in need.',
      category: 'Humanitarian',
      status: 'Active',
      target: 10000,
      raised: 6800,
      imageColor: '#27926e',
      creator: 'GABCDE1234567890ABCDE1234567890ABCDE1234567890ABCDE1234567890',
      createdAt: new Date('2025-03-01T00:00:00Z'),
    },
    {
      id: '2',
      title: 'Open Source Dev Fund',
      description: 'Supporting open source contributors building on Stellar.',
      category: 'Technology',
      status: 'Active',
      target: 5000,
      raised: 5000,
      imageColor: '#1c7459',
      creator: 'GB222222222222222222222222222222222222222222222222222222222',
      createdAt: new Date('2025-01-15T00:00:00Z'),
    },
    {
      id: '3',
      title: 'Community Garden Project',
      description: 'Building urban gardens to improve food security locally.',
      category: 'Environment',
      status: 'Completed',
      target: 3000,
      raised: 3200,
      imageColor: '#47ae88',
      creator: 'GC333333333333333333333333333333333333333333333333333333333',
      createdAt: new Date('2024-11-10T00:00:00Z'),
    },
    {
      id: '4',
      title: 'Local Animal Shelter Relief',
      description: 'Funding medical supplies and food for rescued animals.',
      category: 'Animal Welfare',
      status: 'Active',
      target: 8000,
      raised: 1200,
      imageColor: '#ae4747',
      creator: 'GABCDE1234567890ABCDE1234567890ABCDE1234567890ABCDE1234567890',
      createdAt: new Date('2025-04-12T00:00:00Z'),
    },
    {
      id: '5',
      title: 'Blockchain Education for Youth',
      description: 'Providing free workshops on Web3 and blockchain development to high school students.',
      category: 'Education',
      status: 'Active',
      target: 15000,
      raised: 500,
      imageColor: '#476bae',
      creator: 'GD444444444444444444444444444444444444444444444444444444444',
      createdAt: new Date('2025-05-20T00:00:00Z'),
    },
    {
      id: '6',
      title: 'Reforestation Campaign',
      description: 'Planting trees to restore local forests and fight climate change.',
      category: 'Environment',
      status: 'Active',
      target: 20000,
      raised: 14500,
      imageColor: '#2b8a3e',
      creator: 'GE555555555555555555555555555555555555555555555555555555555',
      createdAt: new Date('2025-06-01T00:00:00Z'),
    },
    {
      id: '7',
      title: 'Disaster Relief Fund',
      description: 'Emergency supply packages and medical assistance for flood victims.',
      category: 'Humanitarian',
      status: 'Completed',
      target: 50000,
      raised: 50500,
      imageColor: '#e03131',
      creator: 'GF666666666666666666666666666666666666666666666666666666666',
      createdAt: new Date('2025-02-10T00:00:00Z'),
    },
    {
      id: '8',
      title: 'Stellar Smart Contract Auditing',
      description: 'Funding auditing services for open-source Soroban smart contracts.',
      category: 'Technology',
      status: 'Active',
      target: 12000,
      raised: 2000,
      imageColor: '#1971c2',
      creator: 'GG777777777777777777777777777777777777777777777777777777777',
      createdAt: new Date('2025-06-15T00:00:00Z'),
    },
    {
      id: '9',
      title: 'Stray Cat Rescue Program',
      description: 'Trap-neuter-return and shelter rehabilitation for stray cats.',
      category: 'Animal Welfare',
      status: 'Completed',
      target: 4000,
      raised: 4050,
      imageColor: '#f08c00',
      creator: 'GH888888888888888888888888888888888888888888888888888888888',
      createdAt: new Date('2024-12-05T00:00:00Z'),
    },
    {
      id: '10',
      title: 'Renewable Energy Research',
      description: 'Funding open-source solar and wind energy research models.',
      category: 'Technology',
      status: 'Completed',
      target: 25000,
      raised: 25000,
      imageColor: '#099268',
      creator: 'GI999999999999999999999999999999999999999999999999999999999',
      createdAt: new Date('2024-10-22T00:00:00Z'),
    },
    {
      id: '11',
      title: 'Medical Aid for Orphans',
      description: 'Providing essential vaccines, medicine, and healthcare checkups to children.',
      category: 'Healthcare',
      status: 'Active',
      target: 15000,
      raised: 9200,
      imageColor: '#d6336c',
      creator: 'GJ000000000000000000000000000000000000000000000000000000000',
      createdAt: new Date('2025-05-10T00:00:00Z'),
    },
    {
      id: '12',
      title: 'Veterinary Clinic Support',
      description: 'Renovating operations theatre and purchasing diagnostic tools for rescue clinic.',
      category: 'Animal Welfare',
      status: 'Active',
      target: 9000,
      raised: 850,
      imageColor: '#ae3ec9',
      creator: 'GK111111111111111111111111111111111111111111111111111111111',
      createdAt: new Date('2025-05-25T00:00:00Z'),
    },
    {
      id: '13',
      title: 'Scholarships for STEM Girls',
      description: 'Supporting high school girls from lower-income backgrounds to enter STEM.',
      category: 'Education',
      status: 'Completed',
      target: 6000,
      raised: 6000,
      imageColor: '#3b5bdb',
      creator: 'GL222222222222222222222222222222222222222222222222222222222',
      createdAt: new Date('2025-03-15T00:00:00Z'),
    },
    {
      id: '14',
      title: 'Solar Power Installation',
      description: 'Installing solar panels at community center to lower utility costs.',
      category: 'Environment',
      status: 'Active',
      target: 7500,
      raised: 3100,
      imageColor: '#0b7285',
      creator: 'GM333333333333333333333333333333333333333333333333333333333',
      createdAt: new Date('2025-06-20T00:00:00Z'),
    },
    {
      id: '15',
      title: 'Indigenous Community Support',
      description: 'Fostering trade opportunities and providing education on heritage conservation.',
      category: 'Humanitarian',
      status: 'Active',
      target: 10000,
      raised: 1500,
      imageColor: '#f59f00',
      creator: 'GN444444444444444444444444444444444444444444444444444444444',
      createdAt: new Date('2025-06-24T00:00:00Z'),
    },
  ];

  async findAndCount(options: {
    where?: {
      search?: string;
      category?: string;
      status?: string;
    };
    order?: { [key: string]: 'ASC' | 'DESC' };
    take?: number;
    skip?: number;
  }): Promise<[Pool[], number]> {
    let result = [...this.pools];

    // 1. Where filters
    if (options.where) {
      const { search, category, status } = options.where;

      // Category exact match
      if (category) {
        result = result.filter(
          (pool) => pool.category.toLowerCase() === category.toLowerCase(),
        );
      }

      // Status exact match
      if (status) {
        result = result.filter(
          (pool) => pool.status.toLowerCase() === status.toLowerCase(),
        );
      }

      // Text search (case-insensitive title ILIKE %search% or description ILIKE %search%)
      if (search) {
        const searchLower = search.toLowerCase();
        result = result.filter(
          (pool) =>
            pool.title.toLowerCase().includes(searchLower) ||
            pool.description.toLowerCase().includes(searchLower),
        );
      }
    }

    // 2. Sorting
    if (options.order) {
      const orderKeys = Object.keys(options.order);
      if (orderKeys.length > 0) {
        const key = orderKeys[0];
        const dir = options.order[key];
        result.sort((a, b) => {
          let valA = a[key];
          let valB = b[key];

          if (valA instanceof Date) valA = valA.getTime();
          if (valB instanceof Date) valB = valB.getTime();

          if (dir === 'DESC') {
            return valA < valB ? 1 : valA > valB ? -1 : 0;
          } else {
            return valA > valB ? 1 : valA < valB ? -1 : 0;
          }
        });
      }
    } else {
      // Default: newest first (createdAt DESC)
      result.sort((a, b) => b.createdAt.getTime() - a.createdAt.getTime());
    }

    const total = result.length;

    // 3. Pagination
    const skip = options.skip ?? 0;
    const take = options.take ?? 10;
    result = result.slice(skip, skip + take);

    return [result, total];
  }
}
