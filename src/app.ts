import express, { type Express } from 'express';
import path from 'path';
import swaggerUi from 'swagger-ui-express';
import YAML from 'yamljs';

const OPENAPI_FILE = path.join(__dirname, '..', 'api-spec.yaml');

export function createApp(): Express {
  const app = express();

  app.use(express.json());

  app.get('/health', (_req, res) => {
    res.status(200).json({ status: 'ok' });
  });

  const nodeEnv = process.env.NODE_ENV ?? 'staging';
  if (nodeEnv === 'development' || nodeEnv === 'staging') {
    const swaggerDocument = YAML.load(OPENAPI_FILE);
    app.use('/api-docs', swaggerUi.serve, swaggerUi.setup(swaggerDocument));
  }

  return app;
}
