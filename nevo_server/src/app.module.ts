import { MiddlewareConsumer, Module, NestModule } from '@nestjs/common';
import { validateEnvironment } from './config/environment';
import { ConfigModule } from '@nestjs/config';
import { TypeOrmModule } from '@nestjs/typeorm';
import { ScheduleModule } from '@nestjs/schedule';
import { APP_FILTER, APP_GUARD } from '@nestjs/core';
import { ThrottlerGuard } from '@nestjs/throttler';
import { AppController } from './app.controller.js';
import { AppService } from './app.service.js';
import { AuthModule } from './auth/auth.module.js';
import { CommentsModule } from './comments/comments.module.js';
import { ContractModule } from './contract/contract.module.js';
import { ThrottlingModule } from './common/throttling.module';
import { LoggingMiddleware } from './common/logging.middleware.js';
import { GlobalExceptionFilter } from './common/global-exception.filter.js';
import { buildDataSourceOptions } from './db.config.js';
import { DonationsModule } from './donations/donations.module.js';
import { PoolsModule } from './pools/pools.module.js';
import { StatsModule } from './stats/stats.module.js';
import { SyncModule } from './sync/sync.module.js';

import { TransactionsModule } from './transactions/transactions.module.js';
import { UsersModule } from './users/users.module.js';

@Module({
  imports: [
    ConfigModule.forRoot({ isGlobal: true, validate: validateEnvironment }),
    TypeOrmModule.forRootAsync({
      imports: [ConfigModule],
      useFactory: () => buildDataSourceOptions(['dist/migrations/*.js']),
    }),
    ScheduleModule.forRoot(),
    ThrottlingModule,
    AuthModule,
    CommentsModule,
    ContractModule,
    DonationsModule,
    SyncModule,
    StatsModule,
    TransactionsModule,
    PoolsModule,
    UsersModule,
  ],
  controllers: [AppController],
  providers: [
    AppService,
    {
      provide: APP_FILTER,
      useClass: GlobalExceptionFilter,
    },
    {
      provide: APP_GUARD,
      useClass: ThrottlerGuard,
    },
  ],
})
export class AppModule implements NestModule {
  configure(consumer: MiddlewareConsumer): void {
    consumer.apply(LoggingMiddleware).forRoutes('*');
  }
}
