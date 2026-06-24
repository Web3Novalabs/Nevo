import { Module } from '@nestjs/common';
import { AppController } from './app.controller';
import { AppService } from './app.service';
import { PoolsModule } from './pools/pools.module';

@Module({
  imports: [PoolsModule],
  controllers: [AppController],
  providers: [AppService],
})
export class AppModule {}

