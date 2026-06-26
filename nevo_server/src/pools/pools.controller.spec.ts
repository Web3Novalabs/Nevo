import { Test, TestingModule } from '@nestjs/testing';
import { PoolsController } from './pools.controller';
import { PoolsService } from './pools.service';
import { MockPoolRepository } from './pools.repository';

describe('PoolsController', () => {
  let controller: PoolsController;
  let service: PoolsService;

  beforeEach(async () => {
    const module: TestingModule = await Test.createTestingModule({
      controllers: [PoolsController],
      providers: [
        PoolsService,
        MockPoolRepository,
      ],
    }).compile();

    controller = module.get<PoolsController>(PoolsController);
    service = module.get<PoolsService>(PoolsService);
  });

  it('should be defined', () => {
    expect(controller).toBeDefined();
  });

  it('should call service.findAll with query parameters', async () => {
    const query = { page: '2', limit: '5', search: 'Water' };
    const spy = jest.spyOn(service, 'findAll');

    const result = await controller.findAll(query);

    expect(spy).toHaveBeenCalledWith(query);
    expect(result.page).toBe(2);
    expect(result.limit).toBe(5);
  });
});
