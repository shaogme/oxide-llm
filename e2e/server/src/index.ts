import { createMockServer } from './server';

const PORT = Number(process.env.PORT) || 3000;
const server = createMockServer(PORT);

async function main() {
  const url = await server.start();
  console.log(`Mock AI Server running at ${url}`);
}

main().catch((err) => {
  console.error(err);
  process.exit(1);
});
