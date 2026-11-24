import { ValidationPipe } from '@nestjs/common';
import { ConfigService } from '@nestjs/config';
import { NestFactory } from '@nestjs/core';
import { NestExpressApplication } from '@nestjs/platform-express';
import { DocumentBuilder, OpenAPIObject, SwaggerModule } from '@nestjs/swagger';
import helmet from 'helmet';
import { AppModule } from './app.module';
import { parseTrustProxy } from './config/environment';

function buildSwaggerConfig() {
  return new DocumentBuilder()
    .setTitle('Nevo API')
    .setDescription(
      'REST API for Nevo donation pools on Stellar. Authenticated endpoints ' +
        'expect the JWT issued by POST /auth/verify as a bearer token.',
    )
    .setVersion('1.0')
    .addBearerAuth(
      { type: 'http', scheme: 'bearer', bearerFormat: 'JWT' },
      'bearer',
    )
    .addTag('auth', 'Stellar wallet challenge/response authentication')
    .addTag('pools', 'Donation pools')
    .addTag('donations', 'Donations to pools')
    .addTag('stats', 'Aggregate platform statistics')
    .addTag('transactions', 'Signed transaction submission')
    .addTag('contract', 'Smart contract transaction submission')
    .build();
}

/**
 * Builds the OpenAPI document without starting the HTTP listener, so it can be
 * exported from the build (see `npm run generate:openapi`) and diffed in CI.
 */
export async function generateOpenApiDocument(): Promise<OpenAPIObject> {
  const app = await NestFactory.create(AppModule, { logger: false });
  try {
    const document = SwaggerModule.createDocument(app, buildSwaggerConfig());
    return document;
  } finally {
    await app.close();
  }
}

async function bootstrap() {
  const app = await NestFactory.create<NestExpressApplication>(AppModule, {
    abortOnError: false,
  });
  const config = app.get(ConfigService);
  app.set('trust proxy', parseTrustProxy(config.get<string>('TRUST_PROXY')));
  app.use(helmet());

  const corsOrigins = process.env.CORS_ORIGIN ?? process.env.FRONTEND_ORIGIN;
  app.enableCors({
    origin: corsOrigins ? corsOrigins.split(',') : [],
    allowedHeaders: ['Content-Type', 'Authorization'],
  });
  //global validation pipe to validate incoming requests
  app.useGlobalPipes(
    new ValidationPipe({
      whitelist: true,
      forbidNonWhitelisted: true,
      transform: true,
    }),
  );

  app.enableShutdownHooks();

  const document = SwaggerModule.createDocument(app, buildSwaggerConfig());
  SwaggerModule.setup('api-docs', app, document);

  await app.listen(process.env.PORT ?? 3000);
}

// Only start the server when this file is executed directly (not when it is
// imported by the OpenAPI export script).
if (require.main === module) {
  bootstrap().catch((err: unknown) => {
    console.error(err instanceof Error ? err.message : err);
    process.exit(1);
  });
}
