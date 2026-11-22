import { Module } from '@nestjs/common';
import { ThrottlerModule } from '@nestjs/throttler';
import { PostgresThrottlerStorage } from './postgres-throttler.storage';

@Module({
  providers: [PostgresThrottlerStorage],
  exports: [PostgresThrottlerStorage],
})
class ThrottlerStorageModule {}

@Module({
  imports: [
    ThrottlerModule.forRootAsync({
      imports: [ThrottlerStorageModule],
      inject: [PostgresThrottlerStorage],
      useFactory: (storage: PostgresThrottlerStorage) => ({
        throttlers: [{ ttl: 60_000, limit: 100 }],
        storage,
      }),
    }),
  ],
  exports: [ThrottlerModule],
})
export class ThrottlingModule {}
