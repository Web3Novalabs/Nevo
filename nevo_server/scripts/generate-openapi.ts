import { writeFileSync } from 'fs';
import { resolve } from 'path';
import { generateOpenApiDocument } from '../src/main';

async function main() {
  const document = await generateOpenApiDocument();
  const target = resolve(__dirname, '../openapi.json');
  writeFileSync(target, JSON.stringify(document, null, 2) + '\n');
  console.log(`OpenAPI spec written to ${target}`);
}

main().catch((err: unknown) => {
  console.error(err);
  process.exit(1);
});
